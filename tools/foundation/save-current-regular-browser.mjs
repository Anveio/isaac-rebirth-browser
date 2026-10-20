// One fixed public game storage contract, not Original actor-save parity.
// No state/storage injection, ABI observer, Continue or natural reload.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdir,readFile,readdir,stat,writeFile} from 'node:fs/promises';
import {join} from 'node:path';
import {gzipSync} from 'node:zlib';
import {captureSaveOwn} from './save-continue-browser.mjs';
import {OWN_PROFILE_SCHEMA,readOwnProfilePacket} from './save-continue-guards.mjs';
import {inspectBrowserGraphics} from './browser.mjs';
import {evaluateJson,readTrace,step} from '../e2e/lib/game.mjs';
import {acquireHostSlot} from '../e2e/lib/host-slots.mjs';
import {saveCurrentRegularPublicInputs} from '../../references/isaac/cases/save-continue/save-current-regular-public-v1.mjs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const save=(path,value)=>writeFile(path,JSON.stringify(value,null,2)+'\n',{flag:'wx'});

/** Runs the declared fixed recipe on the existing production Playwright page. */
export async function runSaveRegularWriterBrowser(page,output) {
  const releaseSlot=await acquireHostSlot('Game.Save current regular records');
  try { await runWriterCase(page,output); }
  finally { await releaseSlot(); }
}

async function runWriterCase(page,output) {
  const caseData=saveCurrentRegularPublicInputs();
  assert.equal(caseData.scenario.id,'save-current-regular-pbw9ync8-precollect-writer');
  assert.equal(caseData.seed,'PBW9YNC8');assert.equal(caseData.seedValue,1737131469);
  assert.equal(caseData.bootEpochSeed,1700000000);assert.equal(caseData.writerContract.noContinueInvocation,true);
  await mkdir(output,{recursive:false});
  await save(join(output,'declared-case.json'),caseData);
  let buildSha256;
  const onPageError=error=>checks.push({name:'pageerror',status:'fail',error:String(error)});
  const named={Return:'Enter',Down:'ArrowDown',Escape:'Escape',Tab:'Tab'};
  const code=key=>named[key]??(/^[a-z]$/i.test(key)?'Key'+key.toUpperCase():/^[0-9]$/.test(key)?'Digit'+key:assert.fail(key));
  const rows=[],checks=[],commands=[],keys=new Set(),enginePressed=new Set();
  let seedDialog=false,fatal,graphics,paused;
  const check=(name,body)=>{try{body();checks.push({name,status:'pass'});}catch(error){checks.push({name,status:'fail',error:String(error.stack??error)});}};
  const bytes=async dir=>{let n=0;for(const x of await readdir(dir,{withFileTypes:true})){const p=join(dir,x.name);n+=x.isDirectory()?await bytes(p):(await stat(p)).size;}return n;};
  const budget=async()=>assert.ok(await bytes(output)<=32*1024*1024,'Declared32MiB storage witness output cap');
  // Independent field join from the actual ordered live manager, no capture-helper call.
  const projection=state=>{
    assert.ok(Array.isArray(state.actorScheduler?.actors)&&Array.isArray(state.room.pickups));
    const ids=new Set(),body=new Map(state.room.pickups.map(p=>[p.id,p])),records=[];
    assert.equal(body.size,state.room.pickups.length);
    for(const actor of state.actorScheduler.actors)if(actor.kind==='pickup'){
      assert.ok(!ids.has(actor.id));ids.add(actor.id);const p=body.get(actor.id);assert.ok(p,'Actual manager pickup requires body');
      if(p.removed||p.collected)continue;
      assert.equal(p.variant,20,'Declared natural regular coin domain; no silent family filtering');assert.equal(p.shopPrice,undefined);
      assert.ok(p.spawnSeed>0&&p.regularRng?.state>0);assert.deepEqual([p.regularRng.rightA,p.regularRng.left,p.regularRng.rightB],[4,3,17]);
      const r={handle:p.id,variant:p.variant,subtype:p.subtype,position:p.position,spawnSeed:p.spawnSeed,rngSeed:p.regularRng.state,charge:p.charge,touched:p.touched};
      if(Object.hasOwn(p,'saveByteAbd'))r.saveByteAbd=p.saveByteAbd;records.push(r);
    }
    for(const p of state.room.pickups)if(!p.removed&&!p.collected)assert.ok(ids.has(p.id),'No live body absent from manager');
    return records;
  };
  const noQueue=state=>{assert.equal(state.playerItems.queuedId,null);assert.ok(!Object.hasOwn(state.playerItems,'queuedConfigKind')||state.playerItems.queuedConfigKind===null);};
  try{
    graphics=await inspectBrowserGraphics(page.context().browser());
    page.on('pageerror',onPageError);
    const response=await page.goto('/?manual=1');assert.ok(response);
    buildSha256=response.headers()['x-foundation-build'];assert.match(buildSha256,/^[a-f0-9]{64}$/);
    await page.waitForFunction(()=>['true','failed'].includes(document.body.dataset.ready));
    assert.equal(await page.evaluate(()=>document.body.dataset.ready),'true');
    const startup={audioPreferences:{sfxVolume:0,musicVolume:0},fullscreenPreferences:{fullscreen:false},inputConfigs:{devices:[]}};
    await page.evaluate(({initialSeed,bootEpochSeed,startup})=>{window.game.pause();window.game.reset(initialSeed,{bootEpochSeed,trace:true,...startup});},{...caseData,startup});
    for(const action of caseData.pairedTape){
      if(action.type==='step'){await step(page,action.frames);commands.push({type:'tick',count:action.frames});}
      else if(action.type==='key'){
        const c=code(action.key),repeat=action.down&&keys.has(c);if(action.down)keys.add(c);else keys.delete(c);
        const text=seedDialog?(/^Key[A-Z]$/.test(c)?c.slice(3):/^Digit[0-9]$/.test(c)?c.slice(5):null):null;
        await page.keyboard[action.down?'down':'up'](c);
        if(!action.down){if(enginePressed.delete(c))commands.push({type:'key',code:c,down:false});}
        else if(!repeat){if(text!==null)commands.push({type:'text',text});else{commands.push({type:'key',code:c,down:true});const fresh=!enginePressed.has(c);enginePressed.add(c);if(fresh&&c==='Tab')seedDialog=!seedDialog;else if(fresh&&(c==='Enter'||c==='Escape'))seedDialog=false;}}
      }else{
        assert.equal(action.type,'capture');const state=JSON.parse(await page.evaluate(()=>window.game.snapshotJson()));
        const own=await captureSaveOwn(page,output,action.label,state);
        const feedback=await evaluateJson(page,()=>window.game.feedback());
        const capture=await page.evaluate(()=>window.game.capture());
        const png=Buffer.from(capture.png,'base64');await writeFile(join(output,action.label+'.png'),png,{flag:'wx'});
        await writeFile(join(output,action.label+'.state.json.gz'),gzipSync(Buffer.from(JSON.stringify(state))),{flag:'wx'});
        await writeFile(join(output,action.label+'.render-state.json.gz'),gzipSync(Buffer.from(JSON.stringify(capture.state))),{flag:'wx'});
        // frame_json elides every Room's paint; snapshot() reattaches only current-room paint.
        // Retain both complete observations without normalizing either. Join only storage owners.
        for(const field of ['screen','tick','systemClock','gameUpdateCount','currentRoom','stage','paused','saveRevision','saveWritePending','saveReadmissionPending'])assert.deepEqual(capture.state[field],state[field],'Snapshot/render storage owner '+field);
        assert.deepEqual(capture.state.actorScheduler,state.actorScheduler,'Snapshot/render exact manager membership and order');
        assert.deepEqual(projection(capture.state),projection(state),'Snapshot/render ordered current regular bodies');
        assert.deepEqual(capture.state.playerItems,state.playerItems,'Snapshot/render actual queued-grant owner');
        rows.push({label:action.label,state,own,feedback,pngSha256:hash(png)});
        check(action.label+':actual IO and runtime',()=>{assert.equal(feedback.runtimeFailed,false);assert.equal(own.io.declared,true);assert.equal(own.io.storageAvailable,true);assert.equal(own.io.truncated,false);assert.equal(own.storeError,null);assert.ok(own.io.events.every(x=>['pass','absent'].includes(x.status)));assert.equal(state.saveWritePending,false);assert.equal(state.saveReadmissionPending,false);});
        if(action.label==='save-before-quit')check('Positive paused ordered regular bodies',()=>{
          assert.equal(state.screen,'playing');assert.equal(state.paused,true);assert.equal(state.pauseMenu.selection,2);assert.equal(state.player.dead,false);assert.equal(state.roomTransition,null);assert.notEqual(state.currentRoom,state.floor.startingDescriptor);noQueue(state);
          const records=projection(state);assert.ok(records.length>0);paused={state,records};
        });
        if(paused&&state.screen==='playing')check(action.label+':actual paused boundary is frozen',()=>{assert.equal(state.gameUpdateCount,paused.state.gameUpdateCount);assert.equal(state.currentRoom,paused.state.currentRoom);assert.equal(state.paused,true);noQueue(state);assert.deepEqual(projection(state),paused.records);});
        await budget();
      }
    }
    const trace=await readTrace(page);await writeFile(join(output,'trace.json.gz'),gzipSync(Buffer.from(JSON.stringify(trace))),{flag:'wx'});
    check('Exact public commands and initial seeds',()=>{assert.deepEqual(trace.commands,commands);assert.equal(trace.enabled,true);assert.equal(trace.inputTraceTruncated,false);assert.equal(trace.backendStateTruncated,false);assert.equal(trace.bootEpochSeed,caseData.bootEpochSeed);assert.equal(trace.initialSeed,caseData.initialSeed);assert.equal(keys.size,0);});
    const main=rows.find(r=>r.label==='save-main-settled'),prior=rows.find(r=>r.label==='save-before-quit');
    check('Actual newer write and own menu readmission',()=>{assert.ok(paused&&prior&&main?.own.store);assert.equal(main.state.screen,'main');assert.equal(main.state.saveRevision,prior.state.saveRevision+1);assert.equal(main.state.savedLoadValid,true);const writes=main.own.io.events.filter(e=>e.operation==='save'&&e.revision>prior.state.saveRevision);assert.equal(writes.length,1);assert.equal(writes[0].status,'pass');assert.equal(writes[0].packet.sha256,main.own.store.sha256);assert.ok(main.own.io.events.some(e=>e.operation==='readmit'&&e.status==='pass'&&e.packet?.sha256===main.own.store.sha256));});
    if(main?.own.store){
      const raw=await readFile(main.own.store.path);assert.equal(hash(raw),main.own.store.sha256);const packet=readOwnProfilePacket(raw);const record=packet.profiles[0].saved;
      check('Actual Room.Save packet clock and ordered current regular records',()=>{assert.ok(paused&&record);assert.equal(record.run.gameUpdateCount,paused.state.gameUpdateCount);assert.equal(record.run.currentRoom,paused.state.currentRoom);assert.equal(record.context.runSeedValue,caseData.seedValue);assert.equal(record.run.runSeed,'PBW9 YNC8');assert.deepEqual(record.run.room.savedRegularPickups,paused.records);});
    }
  }catch(error){fatal=String(error.stack??error);}
  finally{
    try{await save(join(output,'terminal.json'),await evaluateJson(page,()=>({state:window.game.snapshot(),io:window.game.saveGames(),feedback:window.game.feedback()})));}
    catch(error){checks.push({name:'terminal retention',status:'fail',error:String(error)});}
    page.off('pageerror',onPageError);
    try{await budget();}catch(error){fatal=[fatal,String(error.stack??error)].filter(Boolean).join('\n');}
    const report={schema:1,ownProfileSchema:OWN_PROFILE_SCHEMA,status:!fatal&&checks.every(row=>row.status==='pass')?'pass':'fail',fatal:fatal??null,
      inputId:caseData.scenario.id,buildSha256,graphics,checks,labels:rows.map(row=>row.label),actualCommands:commands.length,
      scope:'Actual public current-room live-body to own packet/write/readmission only. No Original actor-save parity, Native, Continue, reload or legacy Save comparison grade.'};
    await save(join(output,'report.json'),report);
  }
  const report=JSON.parse(await readFile(join(output,'report.json'),'utf8'));
  assert.equal(report.status,'pass',report.fatal??JSON.stringify(report.checks.filter(row=>row.status!=='pass')));
}
