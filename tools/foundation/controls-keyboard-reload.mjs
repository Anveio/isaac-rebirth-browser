// Actual own Keyboard save -> natural same-origin document reload. No Native
// readback, storage injection, runtime setters or hidden binding inputs.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdirSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {evaluateJson,readTrace} from '../e2e/lib/game.mjs';
import {readBrowserControls} from './controls-browser-observe.mjs';
import {readSavedKeyboard} from './controls-state-pair.mjs';
import {writeJsonEvidence} from './browser-evidence.mjs';

const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const save=(path,data)=>writeFileSync(path,JSON.stringify(data,null,2)+'\n',{flag:'wx'});
const allowed=new Set(['controls-movement-up-w-to-i-v1','controls-os-movement-up-w-to-i-v1']);
function suffix(data) {
  assert.ok(allowed.has(data.scenario.id),'Own reload is declared only for the public first-column W-to-I route');
  const at=data.pairedTape.findIndex(action=>action.type==='capture'&&action.label==='main-new-run-selected');
  assert.ok(at>0);assert.equal(data.pairedTape.filter(action=>action.label==='main-new-run-selected').length,1);
  const press=(key,settle)=>[{type:'key',key,down:true},{type:'step',frames:12},{type:'key',key,down:false},{type:'step',frames:settle}];
  //Actual own Game Save from the preceding public route makes Continue1 valid.
  //The static Up edge selects New Run0. No observed selection changes this tape.
  return [{type:'capture',label:'reset-title'},{type:'step',frames:63},
    ...press('Return',72),{type:'capture',label:'profile-ready'},
    ...press('Return',72),{type:'capture',label:'main-ready'},
    ...press('Up',24),{type:'capture',label:'main-new-run-selected'},
    ...data.pairedTape.slice(at+1)];
}
const scalarOwners=state=>({screen:state.screen,tick:state.tick,systemCounter:state.systemClock.systemCounter});

export async function reloadOwnKeyboard(page,directory,data,original,transport) {
  const out=join(directory,'keyboard-own-reload');mkdirSync(out,{recursive:false});
  const tape=suffix(data),checks=[],frames=[],commands=[],errors=[];
  save(join(out,'declared-public-suffix.json'),{schema:1,protocol:'controls-own-keyboard-reload-v1',initialSeed:data.initialSeed,
    bootEpochSeed:data.bootEpochSeed,seed:data.seed,seedValue:data.seedValue,tape,
    resetOptionsKeys:['trace','bootEpochSeed','audioPreferences','fullscreenPreferences'],inputConfigsProperty:'omitted',
    preferencesStorageKey:'isaac-rebirth.options.v4',inputConfigsStorageKey:'isaac-rebirth.inputconfigs.dat.v1'});
  const check=(field,expected,actual)=>{
    assert.notEqual(expected,undefined,'Missing expected owner '+field);assert.notEqual(actual,undefined,'Missing actual owner '+field);
    let pass=true;try{assert.deepEqual(actual,expected);}catch{pass=false;}
    checks.push({field,expected,actual,status:pass?'pass':'fail'});assert.ok(pass,field);
  };
  const passive=()=>evaluateJson(page,()=>{
    const state=window.game.snapshot(),io=window.game.inputConfigs(),trace=window.game.trace(),preferences=window.game.preferences();
    let storage;try{storage={status:'read',key:io.key,text:localStorage.getItem(io.key)};}
    catch(error){storage={status:'fail',key:io.key,error:String(error)};}
    return {state,io,trace,preferences,feedback:window.game.feedback(),storage,url:location.href,timeOrigin:performance.timeOrigin,
      navigationType:performance.getEntriesByType('navigation')[0]?.type,
      physical:window.__controlsPhysicalInputs()};
  });
  const checkpoint=async label=>{
    const captured=await evaluateJson(page,()=>{const stateJson=window.game.snapshotJson();const {state:ignored,...result}=window.game.capture();return{stateJson,...result};});
    const state=JSON.parse(captured.stateJson),controls=await readBrowserControls(page);
    check(label+'.actualReaderTick',state.tick,controls.tick);check(label+'.actualReaderScreen',state.screen,controls.screen);
    const bytes=Buffer.from(captured.png,'base64'),path=join(out,label+'.png');writeFileSync(path,bytes,{flag:'wx'});
    const row={label,path,sha256:hash(bytes),state,controls,render:captured.render};save(join(out,label+'.json'),row);frames.push(row);return row;
  };
  try {
    assert.ok(allowed.has(data.scenario.id));
    const before=await passive();save(join(out,'before-reload.json'),before);
    //Persist every old-document owner before destroying it; the canonical
    //paired frames remain a separate unchanged result, never replaced by these.
    await writeJsonEvidence(join(out,'original-document-manifest.json.gz'),original);
    check('beforeReload.actualPlaying','playing',before.state.screen);
    check('beforeReload.IOcomplete',false,before.io.truncated);check('beforeReload.feedbackComplete',false,before.feedback.truncated);
    check('beforeReload.runtimeFailed',false,before.feedback.runtimeFailed);check('beforeReload.feedbackEvents',[],before.feedback.events);
    check('beforeReload.storageRead','read',before.storage.status);check('beforeReload.storageKey','isaac-rebirth.inputconfigs.dat.v1',before.storage.key);
    check('beforeReload.preferencesV4Key','isaac-rebirth.options.v4',before.preferences.key);
    check('beforeReload.separateInputConfigsOwner',true,before.io.key!==before.preferences.key);
    const ownDevices=readSavedKeyboard(before.storage.text),ownConfigs={devices:ownDevices};
    check('beforeReload.independentPublicBindings',[{name:'Keyboard',bindings:data.expectedBindingsAfter}],ownDevices);
    check('beforeReload.applied24x2',ownDevices[0].bindings,before.state.keyboard.bindings);
    const writes=before.io.events.filter(event=>event.operation==='save');check('beforeReload.exactlyOneActualSave',1,writes.length);
    check('beforeReload.actualSaveStatus','pass',writes[0]?.status);check('beforeReload.actualSaveText',before.storage.text,writes[0]?.text);
    check('beforeReload.actualSaveConfigs',ownConfigs,writes[0]?.configs);check('beforeReload.trustedPhysicalComplete',original.physical,before.physical);
    const textSha256=hash(before.storage.text);writeFileSync(join(out,'own-inputconfigs.dat'),before.storage.text,{flag:'wx'});
    //Natural load happens before any public reset. No clear/setItem/storageState
    //or copied Native/expected bindings are supplied to either constructor.
    await page.reload();await page.waitForFunction(()=>['true','failed'].includes(document.body.dataset.ready));
    check('naturalBoot.productionReady','true',await page.evaluate(()=>document.body.dataset.ready));
    await page.evaluate(()=>window.game.pause());
    const boot=await passive();save(join(out,'natural-constructor-load.json'),boot);
    check('naturalBoot.preferencesV4Key','isaac-rebirth.options.v4',boot.preferences.key);
    check('naturalBoot.separateInputConfigsOwner',true,boot.io.key!==boot.preferences.key);
    check('naturalBoot.sameOriginURL',before.url,boot.url);check('naturalBoot.navigationType','reload',boot.navigationType);
    check('naturalBoot.distinctDocument',true,Number.isFinite(boot.timeOrigin)&&boot.timeOrigin!==before.timeOrigin);
    check('naturalBoot.storageUnchanged',before.storage,boot.storage);check('naturalBoot.IOcomplete',false,boot.io.truncated);
    check('naturalBoot.noSave',0,boot.io.events.filter(event=>event.operation==='save').length);
    check('naturalBoot.exactConstructorLoad',1,boot.io.events.length);check('naturalBoot.actualLoadOperation','load',boot.io.events[0]?.operation);
    check('naturalBoot.actualLoadStatus','pass',boot.io.events[0]?.status);check('naturalBoot.actualLoadText',before.storage.text,boot.io.events[0]?.text);
    check('naturalBoot.actualLoadConfigs',ownConfigs,boot.io.events[0]?.configs);check('naturalBoot.bootInputConfigs',ownConfigs,boot.trace.bootInputConfigs);
    check('naturalBoot.applied24x2',ownDevices[0].bindings,boot.state.keyboard?.bindings);
    check('naturalBoot.retainedConfigs',ownConfigs,boot.state.inputConfigs);check('naturalBoot.runtimeFailed',false,boot.feedback.runtimeFailed);
    check('naturalBoot.feedbackComplete',false,boot.feedback.truncated);check('naturalBoot.feedbackEvents',[],boot.feedback.events);
    check('naturalBoot.traceInitiallyDisabled',false,boot.trace.enabled);
    //This public reset separately reads the actual same-origin owner again.
    //inputConfigs is deliberately absent; never pass the observed table back.
    const initialCommandState=await evaluateJson(page,({initialSeed,bootEpochSeed,startup})=>window.game.reset(initialSeed,{trace:true,bootEpochSeed,
      audioPreferences:startup.audioPreferences,fullscreenPreferences:startup.fullscreenPreferences}),data);
    let expectedSeedDialog=false,lastCommandState;const pressed=new Set();
    const lastCommandIndex=tape.findLastIndex(action=>action.type!=='capture');
    assert.equal(tape[lastCommandIndex]?.type,'step','The declared suffix ends with a public Step before its final checkpoint');
    for(const [actionIndex,action]of tape.entries()){
      if(action.type==='capture'){await checkpoint(action.label);continue;}
      if(action.type==='step'){
        if(actionIndex===lastCommandIndex)lastCommandState=await evaluateJson(page,count=>window.game.step(count),action.frames);
        else await page.evaluate(count=>{window.game.step(count);},action.frames);
        commands.push({type:'tick',count:action.frames});continue;
      }
      assert.equal(action.type,'key');const code=transport.browserKeyCode(action.key);
      assert.ok(/^(Enter|Tab|ArrowUp|Key[A-Z]|Digit[0-9])$/.test(code),'Only the declared modifier-free suffix is admitted');
      const text=expectedSeedDialog?transport.seedInputText(code,new Set()):null;
      await page.keyboard[action.down?'down':'up'](code);
      if(!action.down){if(pressed.delete(code))commands.push({type:'key',code,down:false});}
      else if(text!==null)commands.push({type:'text',text});
      else{assert.ok(!pressed.has(code));pressed.add(code);commands.push({type:'key',code,down:true});if(code==='Tab')expectedSeedDialog=!expectedSeedDialog;else if(code==='Enter')expectedSeedDialog=false;}
    }
    const trace=await readTrace(page),after=await passive();await writeJsonEvidence(join(out,'public-reset-trace.json.gz'),trace);save(join(out,'after-public-suffix.json'),after);
    check('publicReset.completeTrace',true,trace.enabled);check('publicReset.inputTraceTruncated',false,trace.inputTraceTruncated);
    check('publicReset.backendTruncated',false,trace.backendStateTruncated);check('publicReset.backendReason',null,trace.backendStateTruncationReason);
    check('publicReset.stateHistoryTruncated',false,trace.stateHistoryTruncated);
    check('publicReset.stateHistoryReason',null,trace.stateHistoryTruncationReason);
    check('publicReset.commandStateCount',commands.length+1,Array.isArray(trace.states)?trace.states.length:undefined);
    check('publicReset.commandStatesPresent',true,Array.isArray(trace.states)&&trace.states.length>0&&trace.states.every(state=>state&&typeof state==='object'&&!Array.isArray(state)));
    check('publicReset.initialCommandState',initialCommandState,trace.states[0]);
    check('publicReset.finalCommandState',lastCommandState,trace.states.at(-1));
    //Music callbacks have a separate optional journal; zero real callbacks is
    //not command-history presence. Every actual callback must retain its state.
    check('publicReset.musicBackendStatesComplete',true,Array.isArray(trace.backendEvents)&&trace.backendEvents.every(event=>event.state&&typeof event.state==='object'&&!Array.isArray(event.state)));
    check('publicReset.actualCommands',commands,trace.commands);check('publicReset.actualSeed',data.initialSeed,trace.initialSeed);check('publicReset.actualEpoch',data.bootEpochSeed,trace.bootEpochSeed);
    check('publicReset.bootInputConfigs',ownConfigs,trace.bootInputConfigs);check('publicReset.audio',data.startup.audioPreferences,trace.bootAudioPreferences);
    check('publicReset.fullscreen',data.startup.fullscreenPreferences,trace.bootFullscreenPreferences);
    check('publicReset.preferencesV4Key','isaac-rebirth.options.v4',after.preferences.key);
    check('publicReset.separateInputConfigsOwner',true,after.io.key!==after.preferences.key);
    check('publicReset.twoActualLoads',2,after.io.events.length);
    for(const [index,event]of after.io.events.entries()){
      check('publicReset.load'+index+'.operation','load',event.operation);check('publicReset.load'+index+'.status','pass',event.status);
      check('publicReset.load'+index+'.text',before.storage.text,event.text);check('publicReset.load'+index+'.configs',ownConfigs,event.configs);
    }
    check('publicReset.storageText',before.storage.text,after.storage.text);check('publicReset.IOcomplete',false,after.io.truncated);
    check('publicReset.feedbackComplete',false,after.feedback.truncated);check('publicReset.runtimeFailed',false,after.feedback.runtimeFailed);check('publicReset.feedbackEvents',[],after.feedback.events);
    const byLabel=new Map(frames.map(frame=>[frame.label,frame]));
    check('publicReset.allDeclaredCheckpoints',tape.filter(action=>action.type==='capture').map(action=>action.label),frames.map(frame=>frame.label));
    check('publicReset.actualContinueValid',true,byLabel.get('main-ready')?.state.savedLoadValid);
    check('publicReset.actualContinueSelection',1,byLabel.get('main-ready')?.state.mainSelection);
    check('publicReset.actualNewRunSelection',0,byLabel.get('main-new-run-selected')?.state.mainSelection);
    const final=frames.at(-1).state;check('publicReset.committedRunSeed',data.seed.replaceAll(' ','').toUpperCase(),final.runSeed);
    check('publicReset.committedRunValue',data.seedValue,final.runSeedValue);check('publicReset.seededRun',true,final.seededRun);
    for(const row of frames){check(row.label+'.actualKeyboard24x2',ownDevices[0].bindings,row.controls.keyboard.bindings);check(row.label+'.actualAppliedConfigs',ownConfigs,row.controls.retainedConfigurations);}
    const labels=['remap-gameplay-start','remapped-i-held','remapped-i-released','old-w-held-no-movement','old-w-released'];
    const players=labels.map(label=>{const row=byLabel.get(label);assert.equal(row?.state.screen,'playing');const p=row.state.player;assert.ok([p.x,p.y,p.vx,p.vy].every(Number.isFinite));return p;});
    const[start,held,released,old,end]=players;
    for(const label of labels){const row=byLabel.get(label),isI=label==='remapped-i-held',isW=label==='old-w-held-no-movement';
      check(label+'.actualPhysical',isI?['KeyI']:isW?['KeyW']:[],row.controls.physicalPressed);check(label+'.actualActionUp',isI,row.controls.actions.held[2]);}
    check('movement.IUpDisplacement',true,held.y<start.y);check('movement.IUpVelocity',true,held.vy<0);
    check('movement.IReleaseDecelerates',true,Math.abs(released.vy)<Math.abs(held.vy));check('movement.WNoRenewedAcceleration',true,Math.abs(old.vy)<Math.abs(released.vy));
    check('movement.WReleaseDecelerates',true,Math.abs(end.vy)<=Math.abs(old.vy));check('movement.sameHorizontal',true,players.every(p=>p.x===start.x));
    const physical=after.physical;save(join(out,'physical-inputs.json'),physical);check('publicReset.physicalComplete',false,physical.truncated);
    check('publicReset.actualPhysicalEdges',tape.filter(action=>action.type==='key').map(action=>({type:action.down?'keydown':'keyup',code:transport.browserKeyCode(action.key)})),physical.events.map(({type,code})=>({type,code})));
    check('publicReset.trustedModifierFree',true,physical.events.every(event=>event.isTrusted&&!event.repeat&&!event.ctrlKey&&!event.altKey&&!event.metaKey&&!event.shiftKey));
    check('publicReset.sameOwnTextSHA',textSha256,hash(after.storage.text));
  }catch(error){errors.push(String(error.stack??error));}
  finally{
    try{await writeJsonEvidence(join(out,'final-actual-readback.json.gz'),await passive());}catch(error){errors.push('Retention: '+String(error));}
    save(join(out,'report.json'),{schema:1,protocol:'controls-own-keyboard-reload-v1',status:errors.length?'fail':'pass',errors,checks,commands,labels:frames.map(frame=>frame.label),
      scope:'Actual public Back save, natural same-origin constructor load and separate reset-load/gameplay Keyboard witness. Natural boot ticks are observed, not clock-paired. No Native process restart, storage injection, browser OS-process reload, controller/hotplug/corruption, PCM or pixel equality claim.'});
  }
  assert.equal(errors.length,0,errors.join('\n'));
  return{status:'pass',checks:checks.length,checkpoints:frames.length,report:join(out,'report.json')};
}
