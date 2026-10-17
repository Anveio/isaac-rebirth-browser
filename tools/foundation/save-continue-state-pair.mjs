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
