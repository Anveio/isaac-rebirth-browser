// Supplementary typed Native/Browser owner checks, never game inputs.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {scopedState} from './paired-game.mjs';
const hash=raw=>createHash('sha256').update(raw).digest('hex');
export function saveOwnerState(source,actual,gameplay,candidate) {
  const checks=[];
  const add=(field,expected,observed)=>checks.push({field,expected,actual:observed,status:expected!==undefined&&observed!==undefined&&JSON.stringify(expected)===JSON.stringify(observed)?'pass':'fail'});
  const required=(field,body)=>{try{body();}catch(error){checks.push({field,status:'fail',error:String(error)});}};
  //Keep the unchanged canonical law and all its failures separately. Its
  //one-way gameplay boundary does not understand returning to a menu.
  required('canonical-paired-scope',()=>{checks.push(...scopedState(source,actual,gameplay,candidate));});
  if(!candidate.label.startsWith('save-')&&candidate.label!=='gameplay-start')return checks;
  // Additive actual owners. Keep every historical canonical/Save row above
  // and below, including all failures; this law is identical for both legs.
  required('actual-shared-MT-owners',()=>{
    const native=source.saveOwn?.sharedMT, browser=actual.bootLifecycle?.shared;
    for(const owner of [native,browser]){
      assert.ok(owner&&Number.isInteger(owner.index)&&owner.index>=0&&owner.index<=624);
      assert.ok(Array.isArray(owner.words)&&owner.words.length===624&&owner.words.every(x=>Number.isInteger(x)&&x>=0&&x<=0xffffffff));
    }
    add('actual-shared-MT.index',native.index,browser.index);
    add('actual-shared-MT.words',native.words,browser.words);
  });
  if(source.saveOwn?.systemState===2)required('actual-ambient-manager-owners',()=>{
    assert.equal(source.gameplay?.status,'pass');assert.equal(source.gameplay.memoryWritten,false);
    assert.ok(Array.isArray(source.gameplay.entities)&&Array.isArray(actual.spriteEffects));
    assert.ok(Array.isArray(actual.actorScheduler?.actors)&&Array.isArray(actual.actorScheduler.identities));
    const nativeCounter=source.gameplay.nativePhase?.systemIterations,browserCounter=actual.systemClock?.systemCounter;
    for(const counter of [nativeCounter,browserCounter])assert.ok(Number.isInteger(counter)&&counter>=0&&counter<=0xffffffff,'Actual System counter owner');
    const phase=nativeCounter%2;
    add('ambient.system-phase',phase,browserCounter%2);
    const f32Words=values=>{
      assert.ok(Array.isArray(values)&&values.length===2&&values.every(Number.isFinite),'Actual finite f32 vector owner');
      const view=new DataView(new ArrayBuffer(4));
      return values.map(value=>{view.setFloat32(0,value,true);assert.ok(Number.isFinite(view.getFloat32(0,true)));return view.getUint32(0,true);});
    };
    for(const entity of source.gameplay.entities){
      for(const key of ['type','variant','subtype'])assert.ok(Number.isInteger(entity[key])&&entity[key]>=0&&entity[key]<=0xffffffff);
    }
    assert.ok(source.gameplay.entities.some(entity=>entity.type===1),'Actual live Native Player manager member');
    const variants=new Set([21,33,68]);
    const native=source.gameplay.entities.filter(row=>row.type===1000&&variants.has(row.variant));
    const effects=new Map(actual.spriteEffects.map(row=>[row.id,row]));
    assert.equal(effects.size,actual.spriteEffects.length,'Unique actual Browser effect handles');
    const browser=actual.actorScheduler.actors.filter(row=>row.kind==='effect').map(actor=>{
      const effect=effects.get(actor.id);assert.ok(effect,'Actual manager/body join');
      const identity=actual.actorScheduler.identities.find(row=>row.actor.kind==='effect'&&row.actor.id===actor.id);
      assert.ok(identity&&Number.isInteger(identity.sequenceId));
      return {...effect,sequenceId:identity.sequenceId};
    }).filter(row=>row.critter&&variants.has(row.variant));
    const keys=rows=>{const seen=new Map();return rows.map(row=>{
      for(const key of ['variant','subtype','spawnSeed','sequenceId'])assert.ok(Number.isInteger(row[key])&&row[key]>=0&&row[key]<=0xffffffff);
      const key=JSON.stringify([row.variant,row.subtype,row.spawnSeed]);const occurrence=seen.get(key)??0;seen.set(key,occurrence+1);
      return [1000,row.variant,row.subtype,row.spawnSeed,occurrence];
    });};
    add('ambient.listed-count',native.length,browser.length);
    add('ambient.manager-order',keys(native),keys(browser));
    // Do not pair siblings by seed alone or reorder either game's manager.
    // Every Native slot remains mandatory even when Browser dropped it.
    for(let i=0;i<native.length;i++){
      const n=native[i],b=browser[i],prefix=`ambient[${i}].`;
      const sourcePosition=n.position,sourceVelocity=n.velocity;
      assert.ok([sourcePosition,sourceVelocity].every(v=>Array.isArray(v)&&v.length===2&&v.every(Number.isFinite)));
      if(b)assert.ok([b.position?.x,b.position?.y,b.presentedPosition?.x,b.presentedPosition?.y,b.velocity?.x,b.velocity?.y].every(Number.isFinite));
      const browserPosition=b?(phase===0?[b.presentedPosition.x,b.presentedPosition.y]:[b.position.x,b.position.y]):undefined;
      const browserVelocity=b?[b.velocity.x,b.velocity.y]:undefined;
      for(const [field,expected,observed] of [['position',sourcePosition,browserPosition],['velocity',sourceVelocity,browserVelocity]]){
        add(prefix+field+'.f32Words',f32Words(expected),observed===undefined?undefined:f32Words(observed));
        Object.assign(checks.at(-1),{rawNativeF32Values:expected,rawBrowserDtoValues:observed??null,nativeSystemIterations:nativeCounter,browserSystemCounter:browserCounter});
        if(field==='position')Object.assign(checks.at(-1),{browserPositionOwner:phase===0?'presentedPosition':'position',rawBrowserPosition:b?[b.position.x,b.position.y]:null,rawBrowserPresentedPosition:b?[b.presentedPosition.x,b.presentedPosition.y]:null,scope:'Actual Native Entity+448 world vector versus actual Browser phase owner; exact binary32 words, no epsilon'});
      }
      assert.equal(typeof n.removed,'boolean','Actual Native31F owner');
      if(b){assert.equal(typeof b.removed,'boolean');assert.equal(typeof b.critter.dying,'boolean');}
      for(const [field,expected,observed] of [
        ['sequenceId',n.sequenceId,b?.sequenceId],['variant',n.variant,b?.variant],['subtype',n.subtype,b?.subtype],
        ['spawnSeed',n.spawnSeed,b?.spawnSeed],['actorRngState',n.actorRngState,b?.actorRng?.state],
        ['birthGameUpdate',n.birthGameUpdate,b?.birthGameUpdate],['active',n.active,b?.active],['removed',n.removed,b?(b.removed||b.critter.dying):undefined],
        ['entityCollisionClass',n.entityCollisionClass,b?.entityCollisionClass],['renderOrderOffset',n.renderOrderOffset,b?.renderOrderOffset],
      ]){
        assert.notEqual(expected,undefined,'Actual Native owner '+field);
        add(prefix+field,expected,observed);
      }
    }
  });
  required('actual-Save-owner-and-IO',()=>{
    const native=source.saveOwn;assert.equal(native?.status,'pass');assert.equal(native.memoryWritten,false);assert.equal(native.executableMappingsRead,false);
    assert.ok([0,1,2,3].includes(native.systemState));assert.equal(native.selectedProfile,1);
    assert.equal(candidate.ownSave?.io?.protocol,1);assert.equal(candidate.ownSave.io.truncated,false);assert.equal(candidate.ownSave.storeError,null);
    add('Save.loaded-valid',native.loadedRunValid,actual.savedLoadValid);
    const row127=actual.bootLifecycle.savedGridSeedSamples.filter(row=>row.roomRecord===127);
    assert.equal(row127.length,3);assert.equal(native.saved.row127GridIndex,-1);
    add('Save.actual-unused-row127',native.saved.row127SeedSamples,row127);
    //Actual System family owns this supplementary source scope. The old
    //canonical report above remains intact, including invalid-family reds.
    checks.push(...scopedState(source,actual,native.systemState===2,candidate).map(row=>({...row,field:'current-System-owner:'+row.field})));
  });
  if(source.saveOwn?.loadedRunValid)required('actual-loaded-file-and-saved-stores',()=>{
    const native=source.saveOwn;assert.equal(native.files.length,2);
    for(const file of native.files){assert.equal(file.status,'pass');assert.equal(file.headerMatchesSource,true);add('Native.actual-file-checksum:'+file.kind,native.profileRunChecksum,file.checksumOwner);}
    add('Native.actual-Saved-checksum',native.profileRunChecksum,native.savedRunChecksum);
    const store=candidate.ownSave.store;assert.ok(store);const raw=readFileSync(store.path);assert.equal(hash(raw),store.sha256);
    const packet=JSON.parse(raw);assert.equal(packet.version,1);assert.equal(packet.profiles.length,3);const saved=packet.profiles[0].saved;assert.ok(saved);
    const run=saved.run,items=run.playerItems;
    const entry=saved.context.levelEntry,sourceEntry=native.saved.entryPosition;
    assert.ok(entry&&entry.position&&sourceEntry,'Actual saved entry owners must be present');
    const values=[entry.position.x,entry.position.y];
    assert.ok(values.every(value=>typeof value==='number'&&Number.isFinite(value)),'Actual Browser f32 entry values required');
    assert.ok(Array.isArray(sourceEntry.values)&&sourceEntry.values.length===2&&sourceEntry.values.every(value=>typeof value==='number'&&Number.isFinite(value)));
    assert.ok(Array.isArray(sourceEntry.words)&&sourceEntry.words.length===2&&sourceEntry.words.every(value=>Number.isInteger(value)&&value>=0&&value<=0xffffffff));
    const actualWords=values.map(value=>{const view=new DataView(new ArrayBuffer(4));view.setFloat32(0,value,true);return view.getUint32(0,true);});
    add('Save.actual-persistent:entryPosition.f32Words',sourceEntry.words,actualWords);
    checks.at(-1).rawNativeF32Values=sourceEntry.values;checks.at(-1).rawBrowserDtoValues=values;
    checks.at(-1).scope='Actual Native Level.Save vector words vs actual Browser saved WorldVector:f32; no current Player-position substitution';
    for(const [field,expected,observed]of [['previousRoomGridIndex',native.saved.previousRoomGridIndex,entry.previousRoom],['entrySlot',native.saved.entrySlot,entry.slot]]){
      assert.ok(Number.isInteger(expected)&&Number.isInteger(observed),'Actual integer saved entry owner '+field);add('Save.actual-persistent:'+field,expected,observed);
    }

    for(const [field,expected,observed] of [
      ['runSeed',native.saved.seedText,run.runSeed],['seededRun',native.saved.seededRun,run.seededRun],['stage',native.saved.stage,run.stage],
      ['gameUpdateCount',native.savedGameUpdateCount,run.gameUpdateCount],['roomGridIndex',native.saved.currentRoomGridIndex,run.floor.descriptors[run.currentRoom].gridIndex],
      ['redHp',native.saved.health.red,run.player.hp],['maxRedHp',native.saved.health.maxRed,run.player.maxHp],
      ['coins',native.saved.inventory.coins,run.inventory.coins],['keys',native.saved.inventory.keys,run.inventory.keys],['bombs',native.saved.inventory.bombs,run.inventory.bombs],['goldenKey',native.saved.inventory.goldenKey,run.player.goldenKey],
      ['activeId',native.saved.activeId,items.activeId],['activeCharge',native.saved.activeCharge,items.activeCharge],['reserveCharge',native.saved.reserveCharge,items.reserveCharge],
      ['ownedCounts',native.saved.ownedCounts,items.ownedCounts],['itemRngStates',native.saved.itemRngStates,items.itemRngStates],
      ['trinketRngStates',native.saved.trinketRngStates,items.trinketRngStates],['pocketRngStates',native.saved.pocketRngStates,items.pocketRngStates],
      ['lastPenaltyStage',native.saved.lastPenaltyStage,saved.context.levelProgress.lastPenaltyStage],['heartCollected',native.saved.heartCollected,saved.context.levelProgress.heartCollected],
    ]){assert.notEqual(expected,undefined);assert.notEqual(observed,undefined);add('Save.actual-persistent:'+field,expected,observed);}
  });
  return checks;
}
