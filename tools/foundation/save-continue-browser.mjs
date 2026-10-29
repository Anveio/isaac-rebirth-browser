// Game-level actual own-storage capture. No Native values or files are inputs.
import assert from 'node:assert/strict';
import {evaluateJson} from '../e2e/lib/game.mjs';
import {assertPublicRightMovement} from './save-continue-movement.mjs';
import {unwrapOwnEnvelope} from './save-continue-guards.mjs';
import {createHash} from 'node:crypto';
import {mkdir,readFile,writeFile} from 'node:fs/promises';
import {join} from 'node:path';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const save=(path,value)=>writeFile(path,JSON.stringify(value,null,2)+'\n',{flag:'wx'});
export async function captureSaveOwn(page,directory,label,state) {
  const actual=await evaluateJson(page,()=>{
    const state=window.game.snapshot(), io=window.game.saveGames();
    let storeText=null,storeError=null;
    try{storeText=localStorage.getItem(io.key);}catch(error){storeError=String(error);}
    return {stateJoin:{screen:state.screen,tick:state.tick,systemClock:state.systemClock,saveRevision:state.saveRevision},
      io,storeText,storeError,url:location.href,timeOrigin:performance.timeOrigin};
  });
  assert.deepEqual(actual.stateJoin,{screen:state.screen,tick:state.tick,systemClock:state.systemClock,saveRevision:state.saveRevision},'Save observer must join this exact state, without an intervening command');
  await mkdir(join(directory,'save-packets'),{recursive:true});
  const archive=async text=>{
    assert.equal(typeof text,'string');assert.ok(Buffer.byteLength(text)<=4*1024*1024);
    const sha256=hash(text),path=join(directory,'save-packets',sha256+'.json');
    try{await writeFile(path,text,{flag:'wx'});}catch(error){if(error.code!=='EEXIST')throw error;assert.equal(await readFile(path,'utf8'),text);}
    return {path,sha256,bytes:Buffer.byteLength(text)};
  };
  if(actual.storeText!==null)actual.store=await archive(actual.storeText);
  delete actual.storeText;
  for(const event of actual.io.events)if(Object.hasOwn(event,'text')){
    if(event.text===undefined&&event.status==='fail'&&typeof event.error==='string'&&event.error.length>0){
      event.packetUnavailable={reason:'event-text-undefined',ownTextProperty:true};delete event.text;
    }else{event.packet=await archive(event.text);delete event.text;}
  }
  await save(join(directory,label+'.save-own.json'),actual);
  return actual;
}

export async function reloadOwnSavedWorld(page,directory,caseData) {
  const out=join(directory,'own-reload');await mkdir(out,{recursive:false});
  const rows=[],commands=[],errors=[];
  const checkpoint=async label=>{
    const captured=await evaluateJson(page,()=>window.game.capture());
    const bytes=Buffer.from(captured.png,'base64'),path=join(out,label+'.png');await writeFile(path,bytes,{flag:'wx'});
    const own=await captureSaveOwn(page,out,label,captured.state);
    const row={label,path,sha256:hash(bytes),state:captured.state,own};
    await save(join(out,label+'.json'),row);rows.push(row);return row;
  };
  const step=async count=>{await page.evaluate(n=>{window.game.step(n);},count);commands.push({type:'tick',count});};
  const key=async(code,down)=>{await page.keyboard[down?'down':'up'](code);commands.push({type:'DOM-key',code,down});};
  const press=async(code,settle)=>{await key(code,true);await step(12);await key(code,false);await step(settle);};
  try {
    const before=await checkpoint('before-natural-reload');
    assert.equal(before.own.storeError,null);assert.ok(before.own.store,'Actual own saved bytes required before reload');
    const packet=unwrapOwnEnvelope(JSON.parse(await readFile(before.own.store.path,'utf8')));
    const record=packet.profiles[0].saved;assert.ok(record,'Profile1 must actually retain its own saved run');
    const code=caseData.seed.replaceAll(' ','').toUpperCase();
    assert.match(code,/^[ABCDEFGHJKLMNPQRSTWXYZ01234V6789]{8}$/);
    assert.equal(record.run.runSeed,code.slice(0,4)+' '+code.slice(4),'Actual fresh-writer Saved+EF80 text');
    assert.equal(record.context.runSeedValue,caseData.seedValue);
    //No clear(), setItem(), addInitScript(), storageState(), reset() or Save ABI
    //is used here. The real constructor reads the same origin's existing bytes.
    await page.reload();await page.waitForFunction(()=>['true','failed'].includes(document.body.dataset.ready));
    assert.equal(await page.evaluate(()=>document.body.dataset.ready),'true');
    await page.evaluate(()=>window.game.pause());
    const boot=await checkpoint('natural-reloaded-title');
    assert.equal(boot.own.url,before.own.url);assert.notEqual(boot.own.timeOrigin,before.own.timeOrigin);
    assert.equal(boot.state.screen,'title');assert.equal(boot.own.store.sha256,before.own.store.sha256);
    assert.ok(boot.own.io.events.some(event=>event.operation==='load'&&event.status==='pass'&&event.packet?.sha256===before.own.store.sha256),'Actual constructor load of the own packet is mandatory');
    assert.equal(boot.state.savedLoadValid,false,'Import alone is not a successful profile callback');
    //Independent public Title->Profile->Main->Continue route, declared before
    //capture. Natural boot may have real automatic ticks before pause; keep
    //its actual counter. No claim of zero ticks or Native clock equality.
    await step(63);await press('Enter',72);await checkpoint('reloaded-profile');
    await press('Enter',72);const main=await checkpoint('reloaded-main');
    assert.equal(main.state.screen,'main');assert.equal(main.state.mainSelection,1);assert.equal(main.state.savedLoadValid,true);
    await press('Enter',300);const resumed=await checkpoint('reloaded-playing');
    assert.equal(resumed.state.screen,'playing');assert.equal(resumed.state.runSeedValue,caseData.seedValue);
    assert.equal(resumed.state.runSeed,caseData.seed.replaceAll(' ','').toUpperCase(),'Actual internal compact run code');assert.equal(resumed.state.stage,record.run.stage);
    assert.equal(resumed.state.currentRoom,record.run.currentRoom);
    assert.deepEqual(resumed.state.floor,record.run.floor,'Continue must actually reconstruct the stored floor, not just enable a menu');
    for(const field of ['hp','maxHp','soulHp','blackHeartMask','eternalHearts','goldenKey']){
      assert.ok(Object.hasOwn(record.run.player,field)&&Object.hasOwn(resumed.state.player,field),'Missing Player owner '+field);
      assert.deepEqual(resumed.state.player[field],record.run.player[field],'Actual restored Player.'+field);
    }
    assert.deepEqual(resumed.state.inventory,record.run.inventory);
    for(const field of ['ownedCounts','activeId','activeCharge','reserveCharge']){
      assert.ok(Object.hasOwn(record.run.playerItems,field)&&Object.hasOwn(resumed.state.playerItems,field),'Missing PlayerItems owner '+field);
      assert.deepEqual(resumed.state.playerItems[field],record.run.playerItems[field],'Actual restored PlayerItems.'+field);
    }
    for(const [live,saved,count] of [['itemRngs','itemRngStates',347],['trinketRngs','trinketRngStates',62],['pocketRngs','pocketRngStates',41]]){
      assert.equal(resumed.state.playerItems[live]?.length,count);assert.equal(record.run.playerItems[saved]?.length,count);
      assert.ok(resumed.state.playerItems[live].every(rng=>Number.isInteger(rng.state)),'Actual RNG.state owner required');
      assert.deepEqual(resumed.state.playerItems[live].map(rng=>rng.state),record.run.playerItems[saved],'Actual restored '+live);
    }
    assert.ok(record.run.room.descriptor&&resumed.state.room.descriptor,'Actual reconstructed Room descriptors required');
    for(const field of ['index','gridIndex','catalogIndex','template','doorMask','neighbors'])assert.deepEqual(resumed.state.room.descriptor[field],record.run.room.descriptor[field],'Actual Room descriptor.'+field);
    assert.ok(resumed.state.room.grid?.cells.length>0,'A real room grid must exist after reload');
    let restoredGridKinds=0;
    for(const [index,recorded] of record.run.gridRecords.entries())if(recorded&&recorded.kind!==0){
      assert.equal(resumed.state.room.grid.cells[index]?.entity?.kind,recorded.kind,'Actual restored Grid kind '+index);restoredGridKinds++;
    }
    assert.ok(restoredGridKinds>0,'Actual nonempty saved grid-record reconstruction required');
    assert.equal(resumed.state.room.npcs.length,0,'First slice requires the declared empty initial room');
    assert.ok(Array.isArray(resumed.state.room.backgroundStains),'Actual reloaded Room paint owner is mandatory');
    assert.equal(resumed.state.room.backgroundStains.length,0,'Reloaded Room.Init must clear transient paint before declared movement');
    await key('d',true);await step(8);await key('d',false);await step(24);
    const moved=await checkpoint('reloaded-movement');
    assertPublicRightMovement(resumed.state,moved.state,record.run.floor,record.run.currentRoom);
  }catch(error){errors.push(String(error));}
  finally {
    await save(join(out,'report.json'),{schema:1,status:errors.length?'fail':'pass',errors,commands,labels:rows.map(row=>row.label),
      scope:'Actual same-origin natural document reload and own-storage Continue. No Native file/data, storage injection or reset. Natural boot automatic ticks are observed and not claimed clock-equivalent; no new Native process reload proof.'});
  }
  assert.equal(errors.length,0,errors.join('\n'));
}
