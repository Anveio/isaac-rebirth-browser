// Whole production game/public input tapes. These fixed Browser contracts do not
// claim Native replay self-equality or Original-game parity. No storage/state injection.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {mkdir,readFile,readdir,stat,writeFile} from 'node:fs/promises';
import {join} from 'node:path';
import {gzipSync} from 'node:zlib';
import {captureSaveOwn} from './save-continue-browser.mjs';
import {unwrapOwnEnvelope} from './save-continue-guards.mjs';
import {installSaveAbiObserver,archiveAndGradeSaveAbi} from './save-capacity-observer.mjs';
import {inspectBrowserGraphics} from './browser.mjs';
import {acquireHostSlot} from '../e2e/lib/host-slots.mjs';
import {capacityPublicInputs} from '../../references/isaac/cases/save-continue/capacity-public-v1.mjs';
const CAPACITY_ERROR='saved profiles exceed the supported 4 MiB format';
const MAX_BYTES=4*1024*1024;
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const save=(path,value)=>writeFile(path,JSON.stringify(value,null,2)+'\n',{flag:'wx'});

/** Runs one fixed recorded public recipe on the existing production Playwright page. */
export async function runSaveCapacityBrowser(page,output,role) {
  const releaseSlot=await acquireHostSlot('Save capacity '+role);
  try { await runCapacityCase(page,output,role); }
  finally { releaseSlot(); }
}

async function runCapacityCase(page,output,role) {
  assert.ok(['under','over'].includes(role));
  const {bank,tape}=capacityPublicInputs(role);
  assert.equal(bank.inputMode,role==='under'?'under-continue-prefix':'fresh-normal-over');
  assert.equal(tape.saveHost.stored,null);assert.deepEqual(tape.saveHost.failedWrites,[]);
  await mkdir(output,{recursive:false});
  await save(join(output,'declared-inputs.json'),{bank,tape,scope:'Authored public inputs only; no Native observations or private resource overrides'});
  const checks=[],rows=[],holdRows=[],before=new Map(),after=new Map(),pageErrors=[],wasmResponses=[];
  let referenceRoom,referenceFloor,underStore,olderStore,bootstrapRevision,capacityRevision,lastAction=-1,fatal,graphics,buildSha256;
  const check=(name,body)=>{try{body();checks.push({name,status:'pass'});return true;}catch(error){checks.push({name,status:'fail',error:String(error.stack??error)});return false;}};
  const heapSession=await page.context().newCDPSession(page);
  const onPageError=error=>pageErrors.push(String(error));
  // Observe the immutable production response; do not route/replace any request.
  const onResponse=response=>{if(new URL(response.url()).pathname.endsWith('.wasm'))wasmResponses.push((async()=>{
    const bytes=await response.body();return {url:response.url(),bytes:bytes.length,sha256:hash(bytes),build:response.headers()['x-foundation-build']};
  })());};
  page.on('pageerror',onPageError);page.on('response',onResponse);
const named={Return:'Enter',Down:'ArrowDown',Escape:'Escape',Tab:'Tab'};
const code=key=>named[key]??(/^[a-z]$/i.test(key)?'Key'+key.toUpperCase():/^[0-9]$/.test(key)?'Digit'+key:assert.fail(key));
const structuralMinimum=value=>JSON.stringify(value,(_key,child)=>{
  if(typeof child==='number'){assert.ok(Number.isFinite(child));return 0;}return child;
});
const summary=state=>({screen:state.screen,tick:state.tick,systemCounter:state.systemClock?.systemCounter,
  gameUpdateCount:state.gameUpdateCount,runTick:state.runTick,stage:state.stage,currentRoom:state.currentRoom,
  controlsEnabled:state.player?.controlsEnabled,playerDead:state.player?.dead,player:{x:state.player?.x,y:state.player?.y},
  paused:state.paused,pauseSelection:state.pauseMenu?.selection,saveRevision:state.saveRevision,
  saveWritePending:state.saveWritePending,saveReadmissionPending:state.saveReadmissionPending,
  stains:state.room?.backgroundStains?.length});
const archiveState=async(label,state)=>{
  const raw=Buffer.from(JSON.stringify(state)),packed=gzipSync(raw),path=join(output,label+'.state.json.gz');
  await writeFile(path,packed,{flag:'wx'});return {path,bytes:raw.length,sha256:hash(raw),gzipBytes:packed.length,gzipSha256:hash(packed)};
};
const outputBytes=async directory=>{
  let bytes=0;for(const entry of await readdir(directory,{withFileTypes:true})){
    const path=join(directory,entry.name);bytes+=entry.isDirectory()?await outputBytes(path):(await stat(path)).size;
  }return bytes;
};
const captureDetailed=async label=>{
  const state=JSON.parse(await page.evaluate(()=>window.game.snapshotJson()));
  const own=await captureSaveOwn(page,output,label,state);
  const feedback=await page.evaluate(()=>window.game.feedback());
  const status=await page.locator('#status').evaluate(node=>({hidden:node.hidden,text:node.textContent}));
  const stateArtifact=await archiveState(label,state);
  const actualOutputBytes=await outputBytes(output);
  assert.ok(actualOutputBytes<=128*1024*1024,'Declared 128 MiB Browser evidence budget exceeded');
  const budget={actualOutputBytes,maximumOutputBytes:128*1024*1024,nodeMemoryBytes:process.memoryUsage(),browserV8Heap:await heapSession.send('Runtime.getHeapUsage'),
    scope:'Measured Node process and Chromium renderer V8 heap only; GPU/process RSS and WASM linear memory are outside this heap reading.'};
  const row={label,...summary(state),own,feedback,status,budget,state:stateArtifact};rows.push(row);
  await save(join(output,label+'.checkpoint.json'),row);
  check(label+':completed-actual-IO',()=>{
    assert.equal(own.io.declared,true);assert.equal(own.io.storageAvailable,true);assert.equal(own.io.truncated,false);
    assert.equal(own.storeError,null);assert.equal(state.saveWritePending,false);assert.equal(state.saveReadmissionPending,false);
    assert.equal(feedback.runtimeFailed,false);
  });
  return {state,own,feedback,status,row};
};
const checkpointUnder=async label=>{
  const detail=label==='save-run-ready'||label==='under-continue-edge-72'||/-(before-quit|main-settled|continued-playing)$/.test(label);
  // Intermediate route guards transfer a summary only; they never alter inputs.
  if(!detail){
    const value=JSON.parse(await page.evaluate(()=>{
      const s=window.game.snapshot();return JSON.stringify({screen:s.screen,room:s.currentRoom,stage:s.stage,dead:s.player.dead,
        tick:s.tick,gameUpdateCount:s.gameUpdateCount,controlsEnabled:s.player.controlsEnabled,
        stains:s.room.backgroundStains.length,runtimeFailed:window.game.feedback().runtimeFailed});
    }));
    rows.push({label,...value});
    if(label.includes('-walk-')||label.endsWith('-pre-pause'))check(label+':same-owned-live-room',()=>{
      assert.equal(value.screen,'playing');assert.equal(value.room,referenceRoom);assert.equal(value.stage,1);
      assert.equal(value.dead,false);assert.equal(value.runtimeFailed,false);assert.equal(value.controlsEnabled,true);
    });
    await save(join(output,label+'.summary.json'),rows.at(-1));return;
  }
  const {state,own,feedback,status,row}=await captureDetailed(label);
  if(label==='save-run-ready'){
    referenceRoom=state.currentRoom;referenceFloor=state.floor;
    check('ordinary-empty-start-room',()=>{
      assert.equal(state.screen,'playing');assert.equal(state.character,'Isaac');assert.equal(state.stage,1);
      assert.equal(state.currentRoom,state.floor.startingDescriptor);assert.equal(state.room.npcs.length,0);
      assert.equal(state.runSeedValue,bank.seedValue);assert.equal(state.player.dead,false);
    });
  }
  if(label.endsWith('-before-quit')){
    const leg=label.split('-')[0];before.set(leg,{state,own});
    const stains=state.room.backgroundStains;
    const measurement={stains:stains.length,actualFootprint7Births:stains.filter(stain=>stain.footprint===true&&stain.variant===7).length,observedStainJsonBytes:Buffer.byteLength(JSON.stringify(stains)),
      structuralMinimumBytes:Buffer.byteLength(structuralMinimum(stains)),
      scope:'Read-only lower bound on the same serialized BackgroundStain array; numeric values replaced with one-byte zero only in measurement, never in a runtime input or exported packet.'};
    row.paintMeasurement=measurement;await save(join(output,leg+'-paint-measurement.json'),measurement);
    check(leg+':actual-paused-full-paint',()=>{
      assert.equal(state.paused,true);assert.equal(state.pauseMenu.selection,2);assert.equal(state.currentRoom,referenceRoom);
      assert.equal(leg,'under');assert.ok(measurement.actualFootprint7Births>4096,'Actualfootprint7births required; generic7paint is retained');
      assert.ok(stains.every(stain=>stain.variant===7),'Only Source-owned Blood7 generic/footprint family admitted; entireactualarray is archived and compared');
    });
  }
  if(label.endsWith('-main-settled')){
    const leg=label.split('-')[0],prior=before.get(leg);assert.ok(prior);after.set(leg,{state,own});
    check(leg+':actual-quit-and-menu-load',()=>{
      assert.equal(state.screen,'main');assert.equal(state.mainSelection,1);assert.equal(state.savedLoadValid,true);
      assert.equal(state.saveRevision,prior.state.saveRevision+1);assert.ok(own.store);
    });
    const writes=own.io.events.filter(event=>event.operation==='save'&&event.revision>prior.state.saveRevision);
      const packet=unwrapOwnEnvelope(JSON.parse(await readFile(own.store.path,'utf8'))),record=packet.profiles[0].saved;
      underStore=own.store;
      check('under:bounded-full-packet-and-every-stain',()=>{
        assert.equal(writes.length,1);assert.equal(writes[0].status,'pass');assert.ok(own.store.bytes<=MAX_BYTES);
        assert.equal(writes[0].packet.sha256,own.store.sha256);
        assert.deepEqual(record.run.room.backgroundStains,prior.state.room.backgroundStains);
        assert.equal(record.run.gameUpdateCount,prior.state.gameUpdateCount);
        assert.equal(record.run.currentRoom,referenceRoom);assert.deepEqual(record.run.floor,referenceFloor);
        assert.equal(record.context.runSeedValue,bank.seedValue);assert.equal(record.context.runSeedValue,prior.state.runSeedValue);
        assert.ok([bank.seed,bank.seed.slice(0,4)+' '+bank.seed.slice(4)].includes(record.run.runSeed));
        assert.equal(record.run.stage,1);
        assert.deepEqual(record.run.inventory,prior.state.inventory);
        assert.ok(own.io.events.every(event=>event.status!=='fail'));
      });
  }
  if(label==='under-continue-edge-72')check(label+':actual-fresh-Room-reconstruction',()=>{
    assert.equal(state.screen,'playing');assert.equal(state.currentRoom,referenceRoom);assert.equal(state.stage,1);
    assert.equal(state.player.dead,false);assert.deepEqual(state.floor,referenceFloor);assert.equal(state.room.backgroundStains.length,0);
    assert.ok(underStore);assert.equal(own.store.sha256,underStore.sha256);assert.equal(own.store.bytes,underStore.bytes);
    assert.equal(state.saveRevision,before.get('under').state.saveRevision+1);
  });
  if(label.endsWith('-continued-playing'))check(label+':real-Continue-world',()=>{
    assert.equal(state.screen,'playing');assert.equal(state.currentRoom,referenceRoom);assert.equal(state.stage,1);
    assert.equal(state.runSeedValue,bank.seedValue);assert.deepEqual(state.floor,referenceFloor);
    assert.equal(state.player.dead,false);assert.equal(state.player.controlsEnabled,true);
    assert.equal(state.room.backgroundStains.length,0,'Actual fresh Room reconstructs transient paint');
  });
  if(label==='under-continued-playing')check(label+':actual-controlled-later-Full-progress-no-retry',()=>{
    const early=rows.find(row=>row.label==='under-continue-edge-72');assert.ok(early);assert.ok(underStore);
    assert.ok(Number.isSafeInteger(early.tick)&&Number.isSafeInteger(state.tick));assert.ok(state.tick>early.tick);
    assert.ok(Number.isInteger(early.gameUpdateCount)&&Number.isInteger(state.gameUpdateCount));assert.ok(state.gameUpdateCount>early.gameUpdateCount);
    assert.equal(state.player.controlsEnabled,true);assert.equal(feedback.runtimeFailed,false);
    assert.equal(state.saveRevision,before.get('under').state.saveRevision+1);
    assert.equal(own.store.sha256,underStore.sha256);assert.equal(own.store.bytes,underStore.bytes);
    const writes=own.io.events.filter(event=>event.operation==='save'&&event.revision>before.get('under').state.saveRevision);
    assert.equal(writes.length,1);assert.equal(writes[0].status,'pass');assert.equal(writes[0].packet.sha256,underStore.sha256);
    assert.ok(own.io.events.every(event=>event.status!=='fail'));
  });

};const checkpointOver=async label=>{
  const detail=label==='save-run-ready'||/-(before-quit|main-settled|continued-playing)$/.test(label)||label.startsWith('post-capacity');
  // Intermediate route guards transfer a summary only; they never alter inputs.
  if(!detail){
    const value=JSON.parse(await page.evaluate(()=>{
      const s=window.game.snapshot();return JSON.stringify({screen:s.screen,room:s.currentRoom,stage:s.stage,dead:s.player.dead,
        tick:s.tick,gameUpdateCount:s.gameUpdateCount,controlsEnabled:s.player.controlsEnabled,
        stains:s.room.backgroundStains.length,runtimeFailed:window.game.feedback().runtimeFailed});
    }));
    rows.push({label,...value});
    let routeValid=true;
    if(label.includes('-walk-')||label.endsWith('-pre-pause'))routeValid=check(label+':same-owned-live-room',()=>{
      assert.equal(value.screen,'playing');assert.equal(value.room,referenceRoom);assert.equal(value.stage,1);
      assert.equal(value.dead,false);assert.equal(value.runtimeFailed,false);assert.equal(value.controlsEnabled,true);
    });
    await save(join(output,label+'.summary.json'),rows.at(-1));
    assert.ok(routeValid,'Unqualified route: retain failure and stop before any further public commands');return;
  }
  const {state,own,feedback,status,row}=await captureDetailed(label);
  if(label==='save-run-ready'){
    referenceRoom=state.currentRoom;referenceFloor=state.floor;
    olderStore=own.store;bootstrapRevision=state.saveRevision;
    check('ordinary-empty-start-room',()=>{
      assert.equal(state.screen,'playing');assert.equal(state.character,'Isaac');assert.equal(state.stage,1);
      assert.equal(state.currentRoom,state.floor.startingDescriptor);assert.equal(state.room.npcs.length,0);
      assert.equal(state.runSeedValue,bank.seedValue);assert.equal(state.player.dead,false);
    });
    check('bootstrap:actual-initial-store-before-long-hold',()=>{
      assert.ok(olderStore);assert.equal(bootstrapRevision,1);assert.ok(olderStore.bytes<=MAX_BYTES);
      const writes=own.io.events.filter(event=>event.operation==='save');
      assert.equal(writes.length,1);assert.equal(writes[0].revision,bootstrapRevision);assert.equal(writes[0].status,'pass');
      assert.equal(writes[0].packet.sha256,olderStore.sha256);assert.equal(writes[0].packet.bytes,olderStore.bytes);
      assert.equal(own.io.events.filter(event=>event.status==='fail').length,0);
      const record=unwrapOwnEnvelope(JSON.parse(readFileSync(olderStore.path,'utf8'))).profiles[0].saved;
      assert.equal(record.context.runSeedValue,bank.seedValue);assert.equal(record.context.runSeedValue,state.runSeedValue);
      assert.ok([bank.seed,bank.seed.slice(0,4)+' '+bank.seed.slice(4)].includes(record.run.runSeed));
      assert.equal(record.run.currentRoom,referenceRoom);assert.equal(record.run.stage,1);
      assert.deepEqual(record.run.floor,referenceFloor);assert.equal(record.run.room.backgroundStains.length,0);
    });
  }
  if(label.endsWith('-before-quit')){
    const leg=label.split('-')[0];before.set(leg,{state,own});
    const stains=state.room.backgroundStains;
    const measurement={stains:stains.length,actualFootprint7Births:stains.filter(stain=>stain.footprint===true&&stain.variant===7).length,observedStainJsonBytes:Buffer.byteLength(JSON.stringify(stains)),
      structuralMinimumBytes:Buffer.byteLength(structuralMinimum(stains)),
      scope:'Read-only lower bound on the same serialized BackgroundStain array; numeric values replaced with one-byte zero only in measurement, never in a runtime input or exported packet.'};
    row.paintMeasurement=measurement;await save(join(output,leg+'-paint-measurement.json'),measurement);
    const qualified=check(leg+':actual-paused-full-paint',()=>{
      assert.equal(leg,'over');assert.equal(state.screen,'playing');assert.equal(state.stage,1);
      assert.equal(state.player.dead,false);assert.equal(state.player.controlsEnabled,true);assert.equal(feedback.runtimeFailed,false);
      assert.equal(state.paused,true);assert.equal(state.pauseMenu.selection,2);assert.equal(state.currentRoom,referenceRoom);
      assert.ok(measurement.actualFootprint7Births>4096,'Count actual footprint7 births; generic Blood7 stains remain retained');
      assert.ok(measurement.structuralMinimumBytes>MAX_BYTES,'Every retained stain stays in the actual array; require >4 MiB before Save attempt');
    });
    assert.ok(qualified,'Unqualified oversize witness: retain raw paint and stop BEFORE Quit/Save attempt');
  }
  if(label.endsWith('-main-settled')){
    const leg=label.split('-')[0],prior=before.get(leg);assert.ok(prior);after.set(leg,{state,own});
    check(leg+':actual-quit-and-menu-load',()=>{
      assert.equal(state.screen,'main');assert.equal(state.mainSelection,1);assert.equal(state.savedLoadValid,true);
      assert.equal(state.saveRevision,prior.state.saveRevision+1);assert.ok(own.store);
    });
    const writes=own.io.events.filter(event=>event.operation==='save'&&event.revision>prior.state.saveRevision);
      capacityRevision=prior.state.saveRevision+1;
      check('over:one-capacity-failure-old-store-and-real-readmission',()=>{
        assert.ok(olderStore);assert.equal(writes.length,1);assert.equal(writes[0].revision,capacityRevision);
        assert.equal(writes[0].status,'fail');assert.equal(writes[0].error,'Error: '+CAPACITY_ERROR);
        assert.equal(writes[0].packet,undefined,'No successful/truncated oversize packet');
        assert.deepEqual(writes[0].packetUnavailable,{reason:'event-text-undefined',ownTextProperty:true});
        assert.equal(own.store.sha256,olderStore.sha256);assert.equal(own.store.bytes,olderStore.bytes);
        const reads=own.io.events.filter(event=>event.operation==='readmit'&&event.revision===capacityRevision);
        assert.equal(reads.length,1);assert.equal(reads[0].status,'pass');assert.equal(reads[0].packet?.sha256,olderStore.sha256);
        assert.equal(own.io.events.filter(event=>event.status==='fail').length,1);
        assert.equal(status.text,'The saved profile could not be loaded or saved.','Actual DOM save failure feedback is retained');
      });
  }
  if(label.endsWith('-continued-playing'))check(label+':real-Continue-world',()=>{
    assert.equal(state.screen,'playing');assert.equal(state.currentRoom,referenceRoom);assert.equal(state.stage,1);
    assert.equal(state.runSeedValue,bank.seedValue);assert.deepEqual(state.floor,referenceFloor);
    assert.equal(state.player.dead,false);assert.equal(state.player.controlsEnabled,true);
    assert.equal(state.room.backgroundStains.length,0,'Actual fresh Room reconstructs transient paint');
  });
  if(label.startsWith('post-capacity')){
    const continued=rows.find(row=>row.label==='over-continued-playing');
    check(label+':no-retry-and-later-progress',()=>{
      assert.ok(continued);assert.ok(state.tick>continued.tick);assert.ok(state.gameUpdateCount>continued.gameUpdateCount);
      assert.equal(state.saveRevision,capacityRevision);assert.equal(own.store.sha256,olderStore.sha256);
      assert.equal(own.io.events.filter(event=>event.operation==='save'&&event.status==='fail'&&event.revision===capacityRevision).length,1);
      assert.equal(state.screen,'playing');assert.equal(state.currentRoom,referenceRoom);assert.equal(state.player.controlsEnabled,true);
      if(label==='post-capacity-movement')assert.ok(state.player.x<rows.find(row=>row.label==='post-capacity-idle').player.x);
    });
  }
};
const checkpoint=bank.inputMode==='under-continue-prefix'?checkpointUnder:checkpointOver;
try{
  await page.addInitScript(installSaveAbiObserver);
  graphics=await inspectBrowserGraphics(page.context().browser());
  const response=await page.goto('/?manual=1');assert.ok(response);buildSha256=response.headers()['x-foundation-build'];assert.match(buildSha256,/^[a-f0-9]{64}$/);
  await page.waitForFunction(()=>['true','failed'].includes(document.body.dataset.ready));
  assert.equal(await page.evaluate(()=>document.body.dataset.ready),'true');
  await page.evaluate(tape=>{window.game.pause();window.game.reset(tape.initialSeed,{bootEpochSeed:tape.bootEpochSeed,
    trace:true,audioPreferences:tape.bootAudioPreferences,fullscreenPreferences:tape.bootFullscreenPreferences,inputConfigs:tape.bootInputConfigs});},tape);
  for(const [index,action] of bank.actions.entries()){
    lastAction=index;
    if(action.type==='key')await page.keyboard[action.down?'down':'up'](code(action.key));
    else if(action.type==='step'){
      const observed=await page.evaluate(({count,guard})=>{
        window.game.step(count);if(!guard)return null;
        const s=window.game.snapshot();return {screen:s.screen,room:s.currentRoom,stage:s.stage,dead:s.player.dead,
          tick:s.tick,gameUpdateCount:s.gameUpdateCount,controlsEnabled:s.player.controlsEnabled,runtimeFailed:window.game.feedback().runtimeFailed};
      },{count:action.frames,guard:action.liveRoomGuard===true});
      if(action.liveRoomGuard===true){
        holdRows.push({chunk:action.holdChunk,...observed});
        const valid=check('over-chunk-'+action.holdChunk+':same-owned-live-room',()=>{
          assert.equal(observed.screen,'playing');assert.equal(observed.room,referenceRoom);assert.equal(observed.stage,1);
          assert.equal(observed.dead,false);assert.equal(observed.controlsEnabled,true);assert.equal(observed.runtimeFailed,false);
        });
        assert.ok(valid,'Unqualified hold route: stop, retain failure, no retry or altered inputs');
      }
    }
    else await checkpoint(action.label);
  }
  const trace=JSON.parse(await page.evaluate(()=>{const t=window.game.trace();return JSON.stringify({...t,states:undefined,
    backendEvents:t.backendEvents.map(({state,...event})=>event)});}));
  await save(join(output,'actual-input-trace.json'),trace);
  check('exact-declared-public-command-trace',()=>{
    assert.equal(trace.inputTraceTruncated,false);assert.deepEqual(trace.commands,tape.commands);
  });
  check('browser-page-errors',()=>assert.deepEqual(pageErrors,[]));
}catch(error){fatal=String(error.stack??error);}
finally{
  try{
    const terminal=await page.evaluate(()=>({state:window.game.snapshotJson(),io:window.game.saveGames(),feedback:window.game.feedback()}));
    await writeFile(join(output,'terminal-actual.json.gz'),gzipSync(Buffer.from(JSON.stringify(terminal))),{flag:'wx'});
  }catch(error){fatal=[fatal,String(error)].filter(Boolean).join('\n');}
  try{
    const responses=await Promise.all(wasmResponses);assert.equal(responses.length,1);assert.equal(responses[0].build,buildSha256);
    await save(join(output,'actual-wasm-response.json'),responses[0]);
    await archiveAndGradeSaveAbi(page,output,bank.inputMode,check,responses[0].sha256);
  }catch(error){fatal=[fatal,String(error.stack??error)].filter(Boolean).join('\n');}
  page.off('pageerror',onPageError);page.off('response',onResponse);await heapSession.detach();
  const finalPayloadBytes=await outputBytes(output);
  if(finalPayloadBytes>128*1024*1024)fatal=[fatal,'Actual Browser evidence exceeds128MiB; raw failure retained'].filter(Boolean).join('\n');
  const report={schema:1,status:!fatal&&checks.every(row=>row.status==='pass')?'pass':'fail',fatal:fatal??null,
    inputMode:bank.inputMode,buildSha256,graphics,lastAction,pageErrors,checks,holdRows,checkpoints:rows.map(({own,feedback,state,...row})=>row),
    outputBudget:{finalPayloadBytes,maximumPayloadBytes:128*1024*1024},
    scope:{directAbi:'Actual production WASM packet/error bytes and real host write/readmission calls',
      browser:'Actual own-origin storage, fresh Room Continue and later controlled progress',
      native:'Native Rust/WASM DST replay self-equality is separate and is not established here',
      original:'No Original capacity, paint, rendering, audio or hidden-state parity claim'}};
  await save(join(output,'report.json'),report);
}
const report=JSON.parse(await readFile(join(output,'report.json'),'utf8'));
assert.equal(report.status,'pass',report.fatal??JSON.stringify(report.checks.filter(row=>row.status!=='pass')));
}
