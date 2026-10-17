// Actual game-level readback only. No input producer or reconstructed snapshots.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {optionsPreferenceContract} from './options-preference-contract.mjs';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const word=value=>{assert.ok(typeof value==='number'&&Number.isFinite(value));const b=Buffer.alloc(4);b.writeFloatLE(value);return b.readUInt32LE();};
const number=bits=>{const b=Buffer.alloc(4);b.writeUInt32LE(bits);return b.readFloatLE();};
const eq=(a,b)=>a!==undefined&&b!==undefined&&JSON.stringify(a)===JSON.stringify(b);

export function compareOptionsAudio({data,clock,native,browser,observed,calls,nativeAudioDirectory,metadata,scopedState,preferencesSchema=2}) {
 const checks=[],diagnostics=[];
 const check=(field,expected,actual)=>checks.push({field,expected,actual,status:eq(expected,actual)?'pass':'fail'});
 const f32=(field,expected,actual)=>checks.push({field,domain:'declared f32 owners, exact words',rawExpected:expected,rawActual:actual,
  expected:typeof expected==='number'?word(expected):undefined,actual:typeof actual==='number'?word(actual):undefined,
  status:typeof expected==='number'&&typeof actual==='number'&&word(expected)===word(actual)?'pass':'fail'});
 let error,preferenceContract;
 try{
  preferenceContract=optionsPreferenceContract(preferencesSchema);
  if(preferencesSchema===3)check('declaredBrowserPreferenceContract',preferenceContract,observed.preferenceContract);
  assert.equal(native.status,'pass');assert.equal(native.provenance.status,'pass');assert.equal(native.publicClockValidation.status,'pass');
  assert.equal(browser.status,'pass');assert.ok(observed?.trace);assert.deepEqual(observed.failures,[]);
  check('actualBrowserErrors',[],observed.errors);check('actualAPIComplete',false,observed.api.truncated);
  check('actualGainComplete',false,observed.gain.truncated);check('actualInputComplete',false,observed.input.truncated);
  check('actualKeysTrusted',true,observed.input.events.every(e=>e.trusted===true));
  const code=k=>({Return:'Enter',Up:'ArrowUp',Down:'ArrowDown',Left:'ArrowLeft',Right:'ArrowRight',Tab:'Tab',Escape:'Escape'})[k]??(/^\d$/.test(k)?'Digit'+k:'Key'+k.toUpperCase());
  check('actualPhysicalKeyTape',data.pairedTape.filter(a=>a.type==='key').map(a=>({type:a.down?'keydown':'keyup',code:code(a.key),repeat:false})),
   observed.input.events.map(e=>({type:e.type,code:e.code,repeat:e.repeat})));
  check('publicInputTraceComplete',false,observed.trace.inputTraceTruncated);check('backendStateTraceComplete',false,observed.trace.backendStateTruncated);
  diagnostics.push({field:'completePerCommandStateHistory',status:observed.trace.stateHistoryTruncated?'fail':'pass',
   retainedStates:observed.trace.states.length,publicCommands:observed.trace.commands.length,bytes:observed.trace.stateEncodedBytes,
   maxBytes:observed.trace.maxStateEncodedBytes,truncated:observed.trace.stateHistoryTruncated});
  const originals=new Map(native.manifest.frames.map(f=>[f.label,f]));
  const observations=new Map(native.manifest.debugObservations.map(f=>[f.context.label,f.observation]));
  const frames=new Map(browser.frames.map(f=>[f.label,f]));const owners=new Set(data.menuOwnerLabels);
  const pages={1:'title',2:'profile',3:'main',4:'character',8:'options'};
  for(const label of data.pairedTape.filter(a=>a.type==='capture').map(a=>a.label)){
   const source=observations.get(label),frame=frames.get(label),original=originals.get(label);assert.ok(source&&frame&&original,label);
   check(label+'.pairedPresentation',original.presentedFrame-originals.get(data.bootstrap.boundaryLabel).presentedFrame,frame.browserPresentation);
   check(label+'.sourceObservation','pass',source.status);
   if(!owners.has(label)){
    check(label+'.optionsScopeSkip','skip',source.optionsAudio?.status);
    check(label+'.menuOnlyReaderNotCalled',false,source.optionsAudio?.optionsReaderCalled);
    for(const row of scopedState(source.standard,frame.state,label==='gameplay-start'))checks.push({...row,field:label+'.standard.'+row.field});
    continue;
   }
   const selected=source.optionsAudio,actual=frame.state;assert.equal(selected?.status,'pass');
   check(label+'.page',pages[selected.menu.menu.screen],actual.screen);
   for(const [field,port]of[['selection','selection'],['leftHeld','leftHeld'],['rightHeld','rightHeld'],['verticalHeld','verticalHeld']])
    check(label+'.options.'+field,selected.menu.options[field],actual.optionsMenu[port]);
   const settings=selected.menu.settings;
   check(label+'.sfx.word',settings.sfxVolume.bits,word(actual.audio.sfxVolume));
   check(label+'.rawMusic.word',settings.rawMusicVolume.bits,word(actual.music.volume));
   check(label+'.musicEnabled',settings.musicEnabled,actual.music.musicEnabled);
   check(label+'.effectiveMusic.word',settings.effectiveMusicVolume.bits,word(actual.music.musicEnabled?actual.music.volume:0));
   check(label+'.fullscreenRequested',settings.fullscreenRequested,actual.fullscreen.requested);
   const marker=observed.checkpoints.find(c=>c.label===label);assert.ok(marker,label+' actual checkpoint owner');
   if(preferencesSchema===3){
    check(label+'.preferences.activeKey',preferenceContract.storageKey,marker.preferences.key);
    check(label+'.storage.declaredReadKey',preferenceContract.storageKey,marker.storageKey);
    f32(label+'.mapOpacity.untouchedSetting',Math.fround(.3),actual.mapSettings?.opacity);
   }
   check(label+'.IOComplete',false,marker.preferences.truncated);check(label+'.feedbackComplete',false,marker.feedback.truncated);
   check(label+'.IOFailures',[],marker.preferences.events.filter(e=>e.status==='fail'));
   check(label+'.unexpectedCapabilityFailures',[],marker.feedback.events.filter(e=>e.status==='fail'));
   check(label+'.runtimeSFXComplete',false,frame.browserAudio.truncated);check(label+'.volumeCallbacksComplete',false,frame.browserAudio.volumeCallbacksTruncated);
   check(label+'.runtimeMusicComplete',false,frame.browserAudio.music.truncated);check(label+'.runtimeMusicErrors',[],frame.browserAudio.music.errors);
  }
  const range=(before,after)=>{const lo=originals.get(before).virtualElapsedNs,hi=originals.get(after).virtualElapsedNs;return calls.filter(c=>c.clockDomain==='gate-main-thread'&&c.virtualElapsedNs>lo&&c.virtualElapsedNs<=hi);};
  const browserRange=(before,after,kind)=>{const lo=observed.checkpoints.find(c=>c.label===before),hi=observed.checkpoints.find(c=>c.label===after);return observed[kind].events.slice(lo[kind+'Events'],hi[kind+'Events']);};
  const sfxCalls=range('options-sfx-before','options-sfx-burst-after');
  const starts=sfxCalls.filter(c=>c.api==='alSourcePlay'&&c.sfxOwner?.references?.some(r=>r.id===237));
  check('SFX237.twoActualNativeConfiguredRestarts',2,starts.length);assert.ok(starts.length===2,'Missing actual first/second configured start');
  const first=starts[0],second=starts[1],before=word(Math.fround(1-Math.fround(.1)));
  for(const [i,event]of starts.entries()){
   check(`SFX237.start${i}.uniqueActualOwner`,'UNIQUE_ACTUAL_TABLE_REFERENCE',event.sfxOwner.status);
   check(`SFX237.start${i}.uniqueReferenceCount`,1,event.sfxOwner.references.length);
   check(`SFX237.start${i}.allocatedSource`,true,event.actualSourceAllocationKnown);check(`SFX237.start${i}.allocationComplete`,false,event.sourceAllocationOverflow);
   check(`SFX237.start${i}.actualPlaying`,0x1012,event.postPlayState);check(`SFX237.start${i}.mixerComplete`,true,event.actualMixer?.complete);
   check(`SFX237.start${i}.gain.word`,i===0?before:word(1),event.actualMixer?.gainBits);
   const reference=event.sfxOwner.references[0];check(`SFX237.start${i}.configuredPath`,'resources/sfx/plop.wav',reference.path?.toLowerCase());
   check(`SFX237.start${i}.requestVolume.word`,word(1),reference.requestedVolumeBits);check(`SFX237.start${i}.sourceCooldownDelay`,0,reference.delayBits);
   const upload=calls.findLast(c=>c.api==='alBufferData'&&c.buffer===event.actualMixer?.buffer&&c.seq<event.seq);assert.ok(upload);
   check(`SFX237.start${i}.PCMFormat`,0x1101,upload.format);check(`SFX237.start${i}.PCMRate`,metadata.waveFormat.sampleRate,upload.rate);
   check(`SFX237.start${i}.PCMByteCount`,metadata.pcm237Bytes,upload.bytes);check(`SFX237.start${i}.PCMComplete`,false,upload.truncated);
   check(`SFX237.start${i}.artifactBytes`,upload.bytes,upload.artifactBytes);check(`SFX237.start${i}.actualPCM`,metadata.pcm237Sha256,sha(fs.readFileSync(path.join(nativeAudioDirectory,upload.pcmArtifact))));
  }
  check('SFX237.sameConfiguredBackend',first.sfxOwner.backendObject,second.sfxOwner.backendObject);
  check('SFX237.sameConfiguredWrapper',first.sfxOwner.references[0].wrapperAddress,second.sfxOwner.references[0].wrapperAddress);
  const selected=sfxCalls.filter(c=>c.optionsOwner?.status==='ACTUAL_CONFIGURED_237_WRAPPER');
  const operation=(api,name)=>selected.filter(c=>c.api===api&&c.optionsOwner.operation===name);
  const queries=operation('alGetSourcei','global-volume-is-playing').filter(c=>c.seq>first.seq),gains=operation('alSourcef','global-volume-live-gain');
  const requests=operation('alSourcef','configured-feedback-admitted-gain'),materialized=operation('alSourcef','backend-play-materialized-gain');
  const restarts=operation('alSourceStop','sample-restart-stop'),resets=operation('alSourceStop','pool-reset-before-binding');
  for(const[name,count,actual]of[['playingQuery',1,queries],['liveGain',1,gains],['directConfiguredGainAfterAllocation',1,requests],['materializedStoredGain',2,materialized],['classRestartStop',1,restarts],['poolResetBeforeBinding',2,resets]])check('SFX237.Native.'+name,count,actual.length);
  check('SFX237.Native.noUnknownRequiredOwner',0,calls.filter(c=>c.optionsOwner?.status?.startsWith('UNKNOWN')).length);
  assert.ok(queries.length===1&&gains.length===1&&requests.length===1&&materialized.length===2&&restarts.length===1&&resets.length===2,'Missing actual operation-specific owner evidence');
  const query=queries[0],gain=gains[0],request=requests[0],restart=restarts[0];
  for(const event of selected){
   check(`SFX237.Native.seq${event.seq}.liveDerivedClass`,'0x67cf10',event.optionsOwner.backendClassVtable);
   check(`SFX237.Native.seq${event.seq}.actualBackend`,first.sfxOwner.backendObject,event.optionsOwner.backendObject);
   check(`SFX237.Native.seq${event.seq}.actualWrapper`,first.sfxOwner.references[0].wrapperAddress,event.optionsOwner.wrapperAddress);
   if(event.optionsOwner.operation!=='pool-reset-before-binding')check(`SFX237.Native.seq${event.seq}.currentHardwareSource`,event.source,event.optionsOwner.actualBackendSource);
  }
  check('SFX237.Native.actualPlayingQueryResult',0x1012,query.value);
  check('SFX237.Native.liveAndDirectGains.word',[word(1),word(1)],[gain.valueBits,request.valueBits]);
  check('SFX237.Native.materializedStoredGains.word',[before,word(1)],materialized.map(e=>e.valueBits));
  check('SFX237.Native.materializedPublicParents',['0x5a81f4','0x5a8337'],materialized.map(e=>e.optionsOwner.parentCallSite));
  check('SFX237.Native.firstGainHasNoInventedDirectAPI',0,requests.filter(e=>e.seq<first.seq).length);
  check('SFX237.Native.beforeRestartSameSource',[first.source,first.source,first.source,first.source],[query.source,gain.source,request.source,restart.source]);
  check('SFX237.Native.materializedSourceJoins',[first.source,second.source],materialized.map(e=>e.source));
  check('SFX237.Native.poolSourceJoins',[first.source,second.source],resets.map(e=>e.source));
  for(const [i,event]of resets.entries()){
   check(`SFX237.Native.pool${i}.pendingHardwareBinding`,'pending-pool-claim',event.optionsOwner.sourceHandleBinding);
   check(`SFX237.Native.pool${i}.actualPoolSource`,event.source,event.optionsOwner.actualPoolSource);
   check(`SFX237.Native.pool${i}.boundedPoolIndex`,true,Number.isInteger(event.optionsOwner.actualPoolIndex)&&event.optionsOwner.actualPoolIndex>=0&&event.optionsOwner.actualPoolIndex<16);
  }
  const ordered=[resets[0],materialized[0],first,query,gain,request,restart,resets[1],materialized[1],second];
  check('SFX237.Native.actualCausalOrder',true,ordered.every((e,i)=>i===0||ordered[i-1].seq<e.seq));
  check('SFX237.Native.noStopBeforeLiveRetune',0,calls.filter(c=>c.seq>first.seq&&c.seq<gain.seq&&c.source===first.source&&c.api==='alSourceStop').length);
  check('SFX237.Native.noDeletionBeforeLiveRetune',0,calls.filter(c=>c.seq>first.seq&&c.seq<gain.seq&&c.api==='alDeleteSources'&&c.ids?.includes(first.source)).length);
  check('SFX237.Native.noRebindBeforeLiveRetune',0,calls.filter(c=>c.seq>first.seq&&c.seq<gain.seq&&c.source===first.source&&c.api==='alSourcei'&&c.parameter===0x1009).length);
  const nativePhysicalStops=selected.filter(c=>c.api==='alSourceStop');check('SFX237.Native.physicalResetCount',3,nativePhysicalStops.length);
  const apiStarts=browserRange('options-sfx-before','options-sfx-burst-after','api').filter(e=>e.type==='start'&&e.pcmSha256===metadata.pcm237Sha256);
  check('SFX237.Browser.actualConfiguredRestarts',2,apiStarts.length);assert.ok(apiStarts.length===2);
  const gainEvents=browserRange('options-sfx-before','options-sfx-burst-after','gain');
  const sourceStarts=apiStarts.map(e=>gainEvents.find(g=>g.type==='source-start'&&g.apiNode===e.node));assert.ok(sourceStarts.every(Boolean));
  for(const [i,event]of apiStarts.entries()){
   check(`SFX237.Browser.start${i}.PCMRate`,metadata.waveFormat.sampleRate,event.sampleRate);check(`SFX237.Browser.start${i}.PCMChannels`,metadata.waveFormat.channels,event.channels);
   check(`SFX237.Browser.start${i}.PCMFrames`,metadata.pcm237Bytes/(metadata.waveFormat.channels*metadata.waveFormat.bits/8),event.frames);
   check(`SFX237.Browser.start${i}.immediateAPIStart`,0,event.when);check(`SFX237.Browser.start${i}.immediateObservedStart`,0,sourceStarts[i].when);
   check(`SFX237.Browser.start${i}.actualStartTime`,true,Number.isFinite(sourceStarts[i].audioTime)&&sourceStarts[i].audioTime>=0);
   check(`SFX237.Browser.start${i}.destination`,true,event.reachesDestination);check(`SFX237.Browser.start${i}.running`,'running',sourceStarts[i].contextState);
   check(`SFX237.Browser.start${i}.gain`,number(starts[i].actualMixer.gainBits),sourceStarts[i].actualGain);
   check(`SFX237.Browser.start${i}.pitch`,number(starts[i].actualMixer.pitchBits),sourceStarts[i].pitch);
   check(`SFX237.Browser.start${i}.loop`,starts[i].actualMixer.loopRaw===1,sourceStarts[i].looping);
  }
  const sourceStart=sourceStarts[0],browserStops=gainEvents.filter(e=>e.type==='source-stop'&&e.source===sourceStart.source);
  check('SFX237.Browser.actualFirstVoiceStop',1,browserStops.length);assert.ok(browserStops.length===1);
  check('SFX237.Browser.immediateFirstVoiceStop',0,browserStops[0].when);
  check('SFX237.Browser.actualStopTime',true,Number.isFinite(browserStops[0].audioTime)&&browserStops[0].audioTime>=0);
  const live=gainEvents.find(e=>e.type==='gain-write'&&e.gainNode===sourceStart.gainNode&&e.actual===number(gain.valueBits)&&e.contextState==='running'&&
   e.sources.some(s=>s.source===sourceStart.source&&!s.ended&&e.audioTime>=sourceStart.when&&e.audioTime-sourceStart.audioTime<sourceStart.frames/sourceStart.sampleRate/sourceStart.pitch));
  check('SFX237.Browser.actualStillPlayingRetune',true,!!live);
  check('SFX237.Browser.actualRestartOrder',true,!!live&&gainEvents.indexOf(sourceStarts[0])<gainEvents.indexOf(live)&&gainEvents.indexOf(live)<gainEvents.indexOf(browserStops[0])&&gainEvents.indexOf(browserStops[0])<gainEvents.indexOf(sourceStarts[1]));
  diagnostics.push({field:'actualPhysicalBackendOperations',status:'reported separately; each stage has mandatory checks',NativePhysicalStops:nativePhysicalStops.length,BrowserVoiceStops:browserStops.length,NativeStarts:starts.length,BrowserStarts:apiStarts.length,
   scope:'Native source-derived class stop and pool-reset-before-binding are distinct stages. No physical-reset/voice-stop count equality, audible/fullmix PCM or old-v3 regrade is claimed.'});
  check('actualStepOwnerJournalComplete',false,observed.returns.truncated);
  check('actualStepOwnerJournalFailure',null,observed.returns.error);
  check('actualStepOwnerJournalCoverage',clock.browserTickCommands.length,observed.returns.rows.length);
  check('actualStepOwnerJournalCommands',clock.browserTickCommands,observed.returns.rows.map(row=>row.command));
  check('actualStepOwnerJournalPresentations',true,observed.returns.rows.every((row,index)=>row.presentation===index+1&&row.tick===index+1));
  let commandCount=0;const counts=new Map();
  for(const a of data.pairedTape){if(a.type==='key')commandCount++;else if(a.type==='step')commandCount+=a.frames;else counts.set(a.label,commandCount);if(a.type==='capture'&&a.label==='options-back-main-saved')break;}
  const events=(before,after)=>{const lo=counts.get(before),hi=counts.get(after);assert.ok(observed.trace.states.length>hi,'Selected actual request states must survive the fixed history cap');return observed.trace.states.slice(lo+1,hi+1).flatMap(s=>s.audio.events);};
  check('SFX237.actualBrowserAdmittedRequests',2,events('options-sfx-before','options-sfx-burst-after').filter(e=>e.id===237).length);
  const musicCalls=range('options-music-before','options-music-saved-decreased');
  check('Music.actualNative237Requests',0,musicCalls.filter(c=>c.optionsOwner?.operation==='configured-feedback-admitted-gain'&&c.optionsOwner.status==='ACTUAL_CONFIGURED_237_WRAPPER').length);
  check('Music.actualNative237Starts',0,musicCalls.filter(c=>c.api==='alSourcePlay'&&c.sfxOwner?.references?.some(r=>r.id===237)).length);
  check('Music.actualBrowser237Requests',0,events('options-music-before','options-music-saved-decreased').filter(e=>e.id===237).length);
  const musicNodes=new Set(observed.api.events.filter(e=>e.type==='start'&&metadata.music61InputSha256s.includes(e.encodedInput?.sha256)).map(e=>e.node));
  const musicStarts=observed.gain.events.filter(e=>e.type==='source-start'&&musicNodes.has(e.apiNode));
  const musicSources=new Map(musicStarts.map(e=>[e.source,e]));
  check('Music.actualTitleStreamIdentity',true,musicStarts.length>0);
  for(const [before,after]of[['options-music-before','options-music-decreased'],['options-music-decreased','options-music-restored'],['options-music-restored','options-music-saved-decreased']]){
   const e=observations.get(after).optionsAudio.music.music,active=e.channels[e.activeChannel],a=frames.get(after).state.music.channels[e.activeChannel];
   assert.ok(active.openalSource>0);f32(after+'.actualWrapperGain',active.wrapperGain,a.gain);
   check(after+'.actualNativeBackendGain',true,range(before,after).some(c=>c.api==='alSourcef'&&c.source===active.openalSource&&c.parameter===0x100a&&c.valueBits===word(active.wrapperGain)));
   const writes=browserRange(before,after,'gain').filter(c=>c.type==='gain-write'&&c.contextState==='running'&&c.actual===active.wrapperGain);
   check(after+'.actualBrowserBackendGain',true,writes.some(w=>w.sources.some(s=>!s.ended&&musicSources.has(s.source)&&musicSources.get(s.source).when<=w.audioTime)));
  }
  const back=observations.get('options-back-main-saved').optionsAudio.menu;
  const ini=key=>{const matches=[...back.file.text.matchAll(new RegExp('^'+key+'=([^\\r\\n]+)$','gm'))];assert.equal(matches.length,1);return matches[0][1];};
  check('Back.actualNativeSerializedSFX','1.0000',ini('SFXVolume'));check('Back.actualNativeSerializedMusic','0.9000',ini('MusicVolume'));
  const backRows=observed.returns.rows.filter(row=>row.beforeScreen==='options'&&row.screen==='main').map(row=>({command:row.command,page:row.page}));
  check('Back.actualSameIterationTransitionCoverage',1,backRows.length);for(const [i,row]of backRows.entries())check('Back.actualSameIterationDestination.'+i,'game',row.page);
  const saved=observed.checkpoints.find(c=>c.label==='options-back-main-saved');
  if(preferencesSchema===2)check('Back.actualBrowserStorage',{schema:2,sfxVolume:'1.0000',musicVolume:'0.9000',fullscreen:false},JSON.parse(saved.storage));
  else{
   check('Back.actualNativeSerializedMapOpacity','0.3000',ini('MapOpacity'));
   check('Back.actualBrowserStorage',preferenceContract.saved,JSON.parse(saved.storage));
   const loads=saved.preferences.events.filter(e=>e.operation==='load'&&e.status==='pass');
   check('V3.actualInitialStorageLoadObserved',true,loads.length>0);
   check('V3.actualInitialStorageLoadKeyAndValues',true,loads.length>0&&loads.every(e=>e.key===preferenceContract.storageKey&&eq(e.value,preferenceContract.initial)));
   const saves=saved.preferences.events.filter(e=>e.operation==='save'&&e.status==='pass');
   check('V3.actualBackSaveKeyAndValues',true,saves.length===1&&saves.every(e=>e.key===preferenceContract.storageKey&&eq(e.value,preferenceContract.saved)));
  }
  check('Back.actualBrowserSaveSuccesses',1,saved.preferences.events.filter(e=>e.operation==='save'&&e.status==='pass').length);
 }catch(e){error={message:String(e.message),stack:e.stack??null};}
 return{schema:preferencesSchema===3?2:1,...(preferencesSchema===3?{preferenceContract}:{}),status:!error&&checks.length>0&&checks.every(c=>c.status==='pass')?'pass':'fail',checks,diagnostics,error,
  scope:'One declared actual Native/Browser audio-arrow/changed-value Back case with complete public seed transport. Actual direct configured gain, stored-gain materialization, playing query/live gain, class restart and pool reset remain separate blocking rows. Native physical resets and Browser voice stops have independently declared mandatory stages; their raw counts remain explicit. No audible/fullmix PCM, RGBA, complete MT, process-reload or whole Options parity.'};
}
