// Independent actual-field checks. No observation is a runtime input.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {scopedState} from './paired-game.mjs';
const data=JSON.parse(fs.readFileSync(new URL('../../references/isaac/cases/controls/recorded-four-fixed-and-9xzmzpr0-v1.json',import.meta.url))).cases[0];
const labels=data.pairedTape.filter(a=>a.type==='capture').map(a=>a.label);
const changed=labels.indexOf(data.bindingChangedLabel),saved=labels.indexOf(data.backSaveLabel);
const pageScreens={1:'title',2:'profile',3:'main',8:'options',9:'controls',10:'keyboard'};
const playing=new Set(data.playingOwnerLabels),controlled=new Set(data.controlsMenuOwnerLabels);
const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
function recorder(){const rows=[];return{rows,check:(field,expected,actual)=>rows.push({field,expected,actual,status:same(expected,actual)?'pass':'fail'})};}

//This fixed fresh-profile case admits only actual Keyboard, with every one of
//the48 saved signed-i32 attributes. Runtime serializer/decoder is never imported.
export function readSavedKeyboard(text){
 assert.equal(typeof text,'string');assert.ok(Buffer.byteLength(text,'utf8')<=128*1024);
 const body=text.replace(/^\uFEFF/,'').trim().replace(/^<\?xml\s+[^?]*\?>\s*/,'');
 const root=/^<KeyConfigs\s*>([\s\S]*)<\/KeyConfigs\s*>$/.exec(body);assert.ok(root);
 const node=/^\s*<Controller\b((?:[^"'<>]|"[^"]*"|'[^']*')*)(?:\/>|>\s*<\/Controller\s*>)\s*$/.exec(root[1]);assert.ok(node);
 const attrs=new Map();let remaining=node[1];
 while(remaining.trim()){
  const m=/^\s+(Name|\d+-[01])\s*=\s*(?:"([^"]*)"|'([^']*)')/.exec(remaining);assert.ok(m);assert.ok(!attrs.has(m[1]));attrs.set(m[1],m[2]??m[3]);remaining=remaining.slice(m[0].length);
 }
 assert.equal(attrs.size,49);assert.equal(attrs.get('Name'),'Keyboard');
 const bindings=Array.from({length:24},(_,action)=>[0,1].map(column=>{const raw=attrs.get(`${action}-${column}`);assert.match(raw??'',/^-?\d+$/);const key=Number(raw);assert.ok(Number.isInteger(key)&&key>=-2147483648&&key<=2147483647);return key;}));
 return[{name:'Keyboard',bindings}];
}

export function compareControls(source,state,gameplay,frame){
 assert.equal(source?.status,'pass');assert.equal(source.memoryWritten,false);
 const label=frame.label,index=labels.indexOf(label);assert.ok(index>=0);
 assert.equal(gameplay,playing.has(label));
 const {rows,check}=recorder(),b=frame.controls;assert.equal(b?.status,'observed');
 check('reader.sameActualTick',state.tick,b.tick);check('reader.sameActualScreen',state.screen,b.screen);
 assert.equal(source.startupOptions?.status,'pass');
 check('actualNative.Options.SFXVolume',data.startup.audioPreferences.sfxVolume,source.startupOptions.sfxVolume);
 check('actualNative.Options.MusicVolume',data.startup.audioPreferences.musicVolume,source.startupOptions.musicVolume);
 check('actualNative.Options.Fullscreen',0,source.startupOptions.fullscreen);
 check('actualBrowser.SFXOwner',data.startup.audioPreferences.sfxVolume,state.audio.sfxVolume);
 check('actualBrowser.MusicOwner',data.startup.audioPreferences.musicVolume,state.music.volume);
 check('actualBrowser.FullscreenOwner',false,state.fullscreen.requested);
 const expectedBindings=index>=changed?data.expectedBindingsAfter:data.expectedBindingsBefore;
 check('actualBrowser.Keyboard24x2',expectedBindings,b.keyboard.bindings);
 check('actualBrowser.bootInputConfigs',data.startup.inputConfigs,b.bootInputConfigs);
 check('actualBrowser.bootAudio',data.startup.audioPreferences,b.bootAudioPreferences);
 check('actualBrowser.bootFullscreen',data.startup.fullscreenPreferences,b.bootFullscreenPreferences);
 check('actualBrowser.storageKey','isaac-rebirth.inputconfigs.dat.v1',b.storage.key);
 check('actualBrowser.IO.complete',false,b.io.truncated);check('actualBrowser.feedback.complete',false,b.feedback.truncated);
 check('actualBrowser.halted',false,b.feedback.runtimeFailed);check('actualBrowser.capabilityFailures',[],b.feedback.events);
 const own=gameplay?source.playingKeyboard:source.controls;
 if(gameplay){
  rows.push(...scopedState(source.standard,state,true));
  check('Native.menuReaderDuringPlaying',false,source.controls.menuReaderCalled);
 }else{
  const menu=source.standard?.menu;assert.equal(menu?.status,'pass');assert.equal(menu.menuInitialized,true);
  check('actualNative.menuPage',data.menuOwnerPages[label],menu.menuScreen);
  const expectedScreen=menu.menuScreen===4?(label==='seed-entry'?'seed':'character'):pageScreens[data.menuOwnerPages[label]];
  check('actualBrowser.menuScreen',expectedScreen,state.screen);
  if([1,2,3,4].includes(data.menuOwnerPages[label]))rows.push(...scopedState(source.standard,state,false));
 }
 if(controlled.has(label)||gameplay){
  assert.equal(own?.status,'pass');assert.equal(own.memoryWritten,false);assert.equal(own.executableMappingsRead,false);
  check('actualNative.KeyboardId',0,own.input.keyboardId);check('actualNative.KeyboardEnabled',true,own.input.enabled);
  check('actualNative.Keyboard24x2',expectedBindings,own.input.bindings);
  check('independent.Keyboard24x2',own.input.bindings,b.keyboard.bindings);
  check('independent.held24',own.input.derivedSourceQueryResults.held,b.actions.held);
  check('independent.triggered24',own.input.derivedSourceQueryResults.triggered,b.actions.triggered);
  check('independent.rawFirstNew',own.input.derivedSourceQueryResults.rawKey,b.actions.rawKey??-1);
  const expectedPhysical=label==='remapped-i-held'?[73]:label==='old-w-held-no-movement'?[87]:[];
  const expectedCodes=label==='remapped-i-held'?['KeyI']:label==='old-w-held-no-movement'?['KeyW']:[];
  check('actualNative.physicalKeys',expectedPhysical,own.input.currentPhysical);
  check('actualBrowser.physicalKeys',expectedCodes,b.physicalPressed);
  check('actualNative.action2Held',label==='remapped-i-held',own.input.derivedSourceQueryResults.held[2]);
  check('actualBrowser.action2Held',label==='remapped-i-held',b.actions.held[2]);
  if(!gameplay){
   check('selectedNative.menuPage',data.menuOwnerPages[label],own.menu.page);
   if(own.menu.page===9){
    check('actualNative.deviceRows',[{id:0,name:'Keyboard'}],own.devices.rows);
    check('deviceSelection',own.devices.selection,b.configuration.deviceSelection);
   }else{
    check('actualNative.selectedDevice',0,own.menu.selectedDevice);check('actualBrowser.selectedDevice',0,b.configuration.selectedDevice);
    check('row',own.configuration.selection,b.configuration.selection);check('column',own.configuration.column,b.configuration.column);
    check('waiting',own.configuration.waiting,b.configuration.waiting);
    check('declaredNoOpeningDoubleDispatch',label==='binding-await-key',own.configuration.waiting);
    check('declaredRow',0,own.configuration.selection);check('declaredColumn',0,own.configuration.column);
   }
  }
  if(index>=saved){
   assert.equal(own.file?.status,'pass');check('actualNative.savedComplete24x2',[{name:'Keyboard',bindings:data.expectedBindingsAfter}],own.file.devices);
   assert.equal(b.storage.status,'read');const actualBrowserFile=readSavedKeyboard(b.storage.text);
   check('actualBrowser.savedComplete24x2',[{name:'Keyboard',bindings:data.expectedBindingsAfter}],actualBrowserFile);
   check('independent.saved24x2',own.file.devices,actualBrowserFile);
  }else check('actualNative.fileBeforeBack','absent',own.file?.status);
 }
 if(index>=saved){
  const writes=b.io.events.filter(event=>event.operation==='save');check('actualBrowser.oneSave',1,writes.length);
  check('actualBrowser.saveStatus','pass',writes[0]?.status);check('actualBrowser.savedSequence',0,writes[0]?.sequence);
  check('actualBrowser.retainedConfigs',{devices:[{name:'Keyboard',bindings:data.expectedBindingsAfter}]},b.retainedConfigurations);
 }else{check('actualBrowser.noEarlySave',0,b.io.events.filter(event=>event.operation==='save').length);check('actualBrowser.noEarlyStoredFile',null,b.storage.text);}
 return rows;
}

export function compareMovement(native,browser){
 const {rows,check}=recorder();
 const n=new Map(native.manifest.debugObservations.map(row=>[row.context.label,row.observation]));
 const b=new Map(browser.frames.map(frame=>[frame.label,frame]));
 for(const[kind,get]of[['Native',label=>{const s=n.get(label);assert.equal(s?.standard?.gameplay?.status,'pass');return s.standard.gameplay.players[0];}],['Browser',label=>{const s=b.get(label)?.state;assert.equal(s?.screen,'playing');return{position:[s.player.x,s.player.y],velocity:[s.player.vx,s.player.vy]};}]]){
  const start=get('remap-gameplay-start'),held=get('remapped-i-held'),released=get('remapped-i-released'),old=get('old-w-held-no-movement'),end=get('old-w-released');
  for(const row of[start,held,released,old,end])assert.ok([...row.position,...row.velocity].every(Number.isFinite));
  check(`${kind}.newI.actualUpDisplacement`,true,held.position[1]<start.position[1]);
  check(`${kind}.newI.actualUpVelocity`,true,held.velocity[1]<0);
  check(`${kind}.IRelease.decelerates`,true,Math.abs(released.velocity[1])<Math.abs(held.velocity[1]));
  check(`${kind}.oldW.noRenewedAcceleration`,true,Math.abs(old.velocity[1])<Math.abs(released.velocity[1]));
  check(`${kind}.oldWRelease.decelerates`,true,Math.abs(end.velocity[1])<=Math.abs(old.velocity[1]));
  check(`${kind}.unchangedHorizontalWorldPosition`,start.position[0],[held,released,old,end].every(row=>row.position[0]===start.position[0])?start.position[0]:null);
 }
 for(const label of data.playingOwnerLabels)check(`${label}.actualNative.noGridCollision`,false,n.get(label)?.standard?.gameplay?.players[0]?.collidedWithGrid);
 return rows;
}
