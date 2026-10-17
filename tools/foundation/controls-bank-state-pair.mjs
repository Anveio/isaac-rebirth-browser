// Actual independent fields for the three predeclared fixed bank routes.
// Source observations remain outputs and never enter either runtime.
import assert from 'node:assert/strict';
import {readSavedKeyboard} from './controls-state-pair.mjs';
import {scopedState} from './paired-game.mjs';
const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
function recorder(){const rows=[];return{rows,check:(field,expected,actual)=>rows.push({field,expected,actual,status:same(expected,actual)?'pass':'fail'})};}
const pageScreens={1:'title',2:'profile',3:'main',8:'options',9:'controls',10:'keyboard'};

export function comparersFor(data){
 const labels=data.pairedTape.filter(a=>a.type==='capture').map(a=>a.label);
 assert.equal(labels.length,new Set(labels).size);
 const saved=labels.indexOf(data.saveLabel);assert.ok(saved>=0);
 const playing=new Set(data.playingOwnerLabels),controlled=new Set(data.controlsMenuOwnerLabels);
 const transitions=data.expectedBindingTransitions.map(t=>({...t,index:labels.indexOf(t.label)}));
 assert.ok(transitions.every((t,i)=>t.index>=0&&(i===0||t.index>transitions[i-1].index)));
 const finalBindings=transitions.at(-1).bindings;
 function bindingsAt(index){let result=data.expectedBindingsBefore;for(const t of transitions)if(index>=t.index)result=t.bindings;return result;}
 function physicalAt(label){return label==='column0-w-held'||label==='default-w-held'?{native:[87],browser:['KeyW'],up:true}:label==='column1-i-held'?{native:[73],browser:['KeyI'],up:true}:{native:[],browser:[],up:false};}
 function compareControls(source,state,gameplay,frame){
  assert.equal(source?.status,'pass');assert.equal(source.memoryWritten,false);
  const label=frame.label,index=labels.indexOf(label);assert.ok(index>=0);assert.equal(gameplay,playing.has(label));
  const {rows,check}=recorder(),b=frame.controls,expectedBindings=bindingsAt(index);assert.equal(b?.status,'observed');
  check('reader.sameActualTick',state.tick,b.tick);check('reader.sameActualScreen',state.screen,b.screen);
  assert.equal(source.startupOptions?.status,'pass');
  check('actualNative.Options.SFXVolume',data.startup.audioPreferences.sfxVolume,source.startupOptions.sfxVolume);
  check('actualNative.Options.MusicVolume',data.startup.audioPreferences.musicVolume,source.startupOptions.musicVolume);
  check('actualNative.Options.Fullscreen',0,source.startupOptions.fullscreen);
  check('actualBrowser.SFXOwner',data.startup.audioPreferences.sfxVolume,state.audio.sfxVolume);
  check('actualBrowser.MusicOwner',data.startup.audioPreferences.musicVolume,state.music.volume);
  check('actualBrowser.FullscreenOwner',false,state.fullscreen.requested);
  check('actualBrowser.Keyboard24x2',expectedBindings,b.keyboard.bindings);
  check('actualBrowser.bootInputConfigs',data.startup.inputConfigs,b.bootInputConfigs);
  check('actualBrowser.bootAudio',data.startup.audioPreferences,b.bootAudioPreferences);
  check('actualBrowser.bootFullscreen',data.startup.fullscreenPreferences,b.bootFullscreenPreferences);
  check('actualBrowser.storageKey','isaac-rebirth.inputconfigs.dat.v1',b.storage.key);
  check('actualBrowser.IO.complete',false,b.io.truncated);check('actualBrowser.feedback.complete',false,b.feedback.truncated);
  check('actualBrowser.halted',false,b.feedback.runtimeFailed);check('actualBrowser.capabilityFailures',[],b.feedback.events);
  const own=gameplay?source.playingKeyboard:source.controls;
  if(gameplay){rows.push(...scopedState(source.standard,state,true));check('Native.menuReaderDuringPlaying',false,source.controls.menuReaderCalled);}
  else{
   const menu=source.standard?.menu;assert.equal(menu?.status,'pass');assert.equal(menu.menuInitialized,true);
   const page=data.menuOwnerPages[label];check('actualNative.menuPage',page,menu.menuScreen);
   check('actualBrowser.menuScreen',page===4?(label==='seed-entry'?'seed':'character'):pageScreens[page],state.screen);
   if([1,2,3,4].includes(page))rows.push(...scopedState(source.standard,state,false));
  }
  if(controlled.has(label)||gameplay){
   assert.equal(own?.status,'pass');assert.equal(own.memoryWritten,false);assert.equal(own.executableMappingsRead,false);
   check('actualNative.KeyboardId',0,own.input.keyboardId);check('actualNative.KeyboardEnabled',true,own.input.enabled);
   check('actualNative.Keyboard24x2',expectedBindings,own.input.bindings);check('independent.Keyboard24x2',own.input.bindings,b.keyboard.bindings);
   check('independent.held24',own.input.derivedSourceQueryResults.held,b.actions.held);
   check('independent.triggered24',own.input.derivedSourceQueryResults.triggered,b.actions.triggered);
   check('independent.rawFirstNew',own.input.derivedSourceQueryResults.rawKey,b.actions.rawKey??-1);
   const physical=physicalAt(label);check('actualNative.physicalKeys',physical.native,own.input.currentPhysical);check('actualBrowser.physicalKeys',physical.browser,b.physicalPressed);
   check('actualNative.action2Held',physical.up,own.input.derivedSourceQueryResults.held[2]);check('actualBrowser.action2Held',physical.up,b.actions.held[2]);
   if(!gameplay){
    check('selectedNative.menuPage',data.menuOwnerPages[label],own.menu.page);
    if(own.menu.page===9){check('actualNative.deviceRows',[{id:0,name:'Keyboard'}],own.devices.rows);check('deviceSelection',own.devices.selection,b.configuration.deviceSelection);}
    else{
     check('actualNative.selectedDevice',0,own.menu.selectedDevice);check('actualBrowser.selectedDevice',0,b.configuration.selectedDevice);
     for(const key of['selection','column','waiting']){
      check('actualNative.configuration.'+key,data.expectedMenuConfigurations[label][key],own.configuration[key]);
      check('actualBrowser.configuration.'+key,data.expectedMenuConfigurations[label][key],b.configuration[key]);
     }
    }
   }
   if(index>=saved){
    assert.equal(own.file?.status,'pass');assert.equal(b.storage.status,'read');const browserFile=readSavedKeyboard(b.storage.text);
    check('actualNative.savedComplete24x2',[{name:'Keyboard',bindings:finalBindings}],own.file.devices);
    check('actualBrowser.savedComplete24x2',[{name:'Keyboard',bindings:finalBindings}],browserFile);check('independent.saved24x2',own.file.devices,browserFile);
   }else check('actualNative.fileBeforeBack','absent',own.file?.status);
  }
  if(index>=saved){
   const writes=b.io.events.filter(e=>e.operation==='save');check('actualBrowser.oneSave',1,writes.length);check('actualBrowser.saveStatus','pass',writes[0]?.status);check('actualBrowser.savedSequence',0,writes[0]?.sequence);
   check('actualBrowser.retainedConfigs',{devices:[{name:'Keyboard',bindings:finalBindings}]},b.retainedConfigurations);
  }else{check('actualBrowser.noEarlySave',0,b.io.events.filter(e=>e.operation==='save').length);check('actualBrowser.noEarlyStoredFile',null,b.storage.text);}
  if(data.audioOwnerLabels.includes(label)){
   const actual=source.audio187;assert.equal(actual?.status,'pass');assert.equal(actual.memoryWritten,false);assert.equal(actual.executableMappingsRead,false);assert.ok(actual.bytesRead<=2*1024*1024);
   check('actualNative.SFX187.globalVolume',1,actual.globalSfxVolume);
   const slot=state.audio.slots['187'];assert.ok(slot);check('SFX187.actualLoaded',actual.slot.loaded,slot.loaded);
   if(label==='back-refused-sfx187'){
    check('actualNative.SFX187.lastVolume',1,actual.slot.lastVolume);check('actualBrowser.SFX187.lastVolume',1,slot.lastVolume);
    check('actualNative.SFX187.cooldownDelay',2,actual.slot.nextAllowed-actual.slot.lastTick);check('actualBrowser.SFX187.cooldownDelay',2,slot.nextAllowed-slot.lastTick);
    check('actualNative.SFX187.rawCounterOwner',source.controls.systemCounter,actual.systemCounter);check('actualNative.SFX187.menuCounterOwner',source.standard.menu.systemCounter,actual.systemCounter);
    check('actualBrowser.SFX187.rawCounterOwner',state.systemClock.systemCounter,b.systemCounter);
    check('actualBrowser.SFX187.admitted',true,Number.isInteger(slot.lastTick));
   }
  }
  return rows;
 }
 function compareMovement(native,browser){
  assert.ok(data.movementKind,'No movement claim is declared for the Escape binding route');
  const {rows,check}=recorder(),n=new Map(native.manifest.debugObservations.map(r=>[r.context.label,r.observation])),b=new Map(browser.frames.map(f=>[f.label,f]));
  for(const[kind,get]of[['Native',label=>{const s=n.get(label);assert.equal(s?.standard?.gameplay?.status,'pass');return s.standard.gameplay.players[0];}],['Browser',label=>{const s=b.get(label)?.state;assert.equal(s?.screen,'playing');return{position:[s.player.x,s.player.y],velocity:[s.player.vx,s.player.vy]};}]]){
   const route=data.movementKind==='both-up-columns'?[['gameplay-start','column0-w-held','column0-w-released'],['column0-w-released','column1-i-held','column1-i-released']]:[['gameplay-start','default-w-held','default-w-released']];
   for(const[startLabel,heldLabel,releasedLabel]of route){
    const start=get(startLabel),held=get(heldLabel),released=get(releasedLabel);for(const row of[start,held,released])assert.ok([...row.position,...row.velocity].every(Number.isFinite));
    check(`${kind}.${heldLabel}.actualUpDisplacement`,true,held.position[1]<start.position[1]);check(`${kind}.${heldLabel}.actualUpVelocity`,true,held.velocity[1]<0);
    check(`${kind}.${heldLabel}.renewedAcceleration`,true,Math.abs(held.velocity[1])>Math.abs(start.velocity[1]));check(`${kind}.${releasedLabel}.decelerates`,true,Math.abs(released.velocity[1])<Math.abs(held.velocity[1]));
    check(`${kind}.${heldLabel}.unchangedHorizontalPosition`,start.position[0],held.position[0]);check(`${kind}.${releasedLabel}.unchangedHorizontalPosition`,start.position[0],released.position[0]);
   }
  }
  for(const label of data.playingOwnerLabels)check(`${label}.actualNative.noGridCollision`,false,n.get(label)?.standard?.gameplay?.players[0]?.collidedWithGrid);
  if(data.movementKind==='defaults-up'){
   const before=n.get('down-cleared-up-s-bound').audio187,after=n.get('back-refused-sfx187').audio187;
   assert.equal(before?.status,'pass');assert.equal(after?.status,'pass');
   const beforeLabel='down-cleared-up-s-bound',afterLabel='back-refused-sfx187';
   for(const label of[beforeLabel,afterLabel]){assert.equal(data.menuOwnerPages[label],10);assert.equal(n.get(label).controls.menu.page,10);assert.equal(b.get(label).state.screen,'keyboard');}
   assert.equal(n.get('title-ready').standard.menu.menuScreen,1);assert.equal(b.get('title-ready').state.screen,'title');
   const old=b.get(beforeLabel).state.audio.slots['187'],now=b.get(afterLabel).state.audio.slots['187'];
   const half=raw=>{assert.ok(Number.isInteger(raw)&&raw>=0&&raw<=0xffffffff);return(raw|0)>>1;};
   const nativeBefore=before.systemCounter,nativeAfter=after.systemCounter;
   const browserBefore=b.get(beforeLabel).state.systemClock.systemCounter,browserAfter=b.get(afterLabel).state.systemClock.systemCounter;
   const nativeOrigin=n.get('title-ready').standard.menu.systemCounter,browserOrigin=b.get('title-ready').state.systemClock.systemCounter;
   assert.ok(nativeAfter>=nativeBefore&&browserAfter>=browserBefore);
   check('independent.actualTitleCounterParity',nativeOrigin%2,browserOrigin%2);
   check('actualNative.SFX187.priorVirginCooldown',-1,before.slot.nextAllowed);check('actualBrowser.SFX187.priorVirginCooldown',-1,old.nextAllowed);
   check('actualBrowser.SFX187.priorNoAdmission',null,old.lastTick);
   check('actualNative.SFX187.virginCooldownChanged',true,after.slot.nextAllowed!==before.slot.nextAllowed);
   check('actualBrowser.SFX187.actualSlotChanged',true,now.lastTick!==old.lastTick&&now.nextAllowed!==old.nextAllowed&&now.lastVolume!==old.lastVolume);
   check('actualNative.SFX187.inclusiveActualCounterInterval',true,after.slot.lastTick>=half(nativeBefore)&&after.slot.lastTick<=half(nativeAfter));
   check('actualBrowser.SFX187.inclusiveActualCounterInterval',true,Number.isInteger(now.lastTick)&&now.lastTick>=half(browserBefore)&&now.lastTick<=half(browserAfter));
   check('independent.SFX187.elapsedAdmissionHalfTicksSinceActualTitle',after.slot.lastTick-half(nativeOrigin),now.lastTick-half(browserOrigin));
   check('independent.SFX187.elapsedNextAllowedHalfTicksSinceActualTitle',after.slot.nextAllowed-half(nativeOrigin),now.nextAllowed-half(browserOrigin));
   check('actualNative.SFX187.actualBeforeCounterOwner',n.get(beforeLabel).controls.systemCounter,nativeBefore);
   check('actualBrowser.SFX187.actualBeforeCounterOwner',browserBefore,b.get(beforeLabel).controls.systemCounter);
  }
  return rows;
 }
 return{compareControls,compareMovement:data.movementKind?compareMovement:undefined};
}
