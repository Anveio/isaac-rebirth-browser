import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {manifestPath,readJsonEvidence} from '../../foundation/browser-evidence.mjs';

export const EXIT_AUDIO_LABELS=['title-ready','gameplay-start','death-settled','request-1','dispatch-1','dispatch-2','dispatch-4'];

// This optional replay is an explicit bounded prefix, never an implicit
// truncation of the normal four-case/OS-random death gate.
export function validateExitAudioCases(cases,caseFilter) {
  assert.ok(Array.isArray(cases)&&cases.length===1,'--exit-audio requires one explicitly kept Exit prefix');
  assert.equal(caseFilter,'death-spikes-exit','--exit-audio requires the matching explicit filtered-bank marker');
  const data=cases[0];
  assert.equal(data?.scenario?.id,'death-spikes-exit');
  assert.equal(data.deathRequired,true,'Exit audio requires actual natural death/popup coverage');
  assert.equal(data.deathAction,'Escape');
  assert.ok(data.pairedIndependent===undefined||(Array.isArray(data.pairedIndependent)&&data.pairedIndependent.length===0),'Exit audio cannot waive independent fields');
  assert.ok(Array.isArray(data.pairedTape)&&Array.isArray(data.tape)&&Array.isArray(data.bootstrap?.sourceOnlyPrelude));
  assert.equal(data.bootstrap.boundaryLabel,'title-ready');
  assert.deepEqual(data.pairedTape.filter(row=>row.type==='capture').map(row=>row.label),EXIT_AUDIO_LABELS,'Require the seven ordered unique Exit prefix labels; do not silently thin a full tape');
  assert.deepEqual(data.bootstrap.sourceOnlyPrelude.filter(row=>row.type==='capture').map(row=>row.label),['intro']);
  const settled=data.pairedTape.findIndex(row=>row.type==='capture'&&row.label==='death-settled');
  assert.deepEqual(data.pairedTape.slice(settled+1),[
    {type:'key',key:'Escape',down:true},{type:'step',frames:1},{type:'capture',label:'request-1'},
    {type:'key',key:'Escape',down:false},{type:'step',frames:1},{type:'capture',label:'dispatch-1'},
    {type:'step',frames:1},{type:'capture',label:'dispatch-2'},
    {type:'step',frames:1},{type:'step',frames:1},{type:'capture',label:'dispatch-4'},
  ],'Exit audio requires the declared public Escape request and exact dispatch window');
  assert.deepEqual(data.tape,[...data.bootstrap.sourceOnlyPrelude,...data.pairedTape]);
}


// All window bounds come from actual observations of the independently declared
// public request/dispatch captures. They are comparisons, never Browser inputs.
export function compareExitAudioWindow({native, candidate, calls, api}) {
  assert.equal(native.status,'pass');assert.equal(native.provenance?.status,'pass');
  assert.equal(candidate.status,'pass');assert.equal(api.truncated,false);
  const observations=new Map(native.manifest.debugObservations.map(r=>[r.context.label,r]));
  const captures=new Map(native.manifest.frames.map(r=>[r.label,r]));
  const frames=new Map(candidate.frames.map(r=>[r.label,r]));
  assert.deepEqual(native.manifest.audioOptions,{sfxVolume:1,musicVolume:1});
  assert.deepEqual(candidate.frames.map(row=>row.label),EXIT_AUDIO_LABELS);
  assert.deepEqual(native.manifest.frames.map(row=>row.label),['intro',...EXIT_AUDIO_LABELS]);
  for(const label of EXIT_AUDIO_LABELS){
    const observed=observations.get(label)?.observation.exitAudio;
    assert.equal(observed?.status,'pass','Every capture needs the complete composed Music/SFX observation');
    assert.equal(observed.music?.status,'pass');assert.equal(observed.sfx?.status,'pass');
    assert.equal(observed.music.music.globalMusicVolume,1);assert.equal(observed.sfx.globalSfxVolume,1);
    const actual=frames.get(label)?.state;
    assert.equal(actual?.audio?.sfxVolume,1);assert.equal(actual?.music?.volume,1);
  }
  const low=captures.get('request-1'),high=captures.get('dispatch-4');
  assert.ok(low&&high&&low.virtualElapsedNs<high.virtualElapsedNs);
  const requested=observations.get('request-1').observation;
  const requestedPort=frames.get('request-1').state;
  const executionCoverage={status:'pass',request:{sourceSystem:requested.death.systemState,
    sourcePopup:requested.death.popupState,sourceDead:requested.death.dead,
    browserScreen:requestedPort.screen,browserPopup:requestedPort.playerDeath.gameOver?.state,
    browserDead:requestedPort.player.dead,browserPending:requestedPort.playerDeath.systemPending},dispatch:[]};
  const request=executionCoverage.request;
  if(request.sourceSystem!==2||request.sourcePopup!==2||request.sourceDead!==true||
    request.browserScreen!=='playing'||request.browserPopup!==2||request.browserDead!==true||request.browserPending!=='exit')executionCoverage.status='fail';
  for(const label of ['dispatch-1','dispatch-2','dispatch-4']){
    const source=observations.get(label).observation,actual=frames.get(label).state;
    const row={label,sourceSystem:source.death.systemState,sourcePopup:source.death.popupState,
      sourceMenu:source.menu?.menuScreen,sourceMusic:source.exitAudio.music.music.currentId,
      browserScreen:actual.screen,browserPopup:actual.playerDeath.gameOver,browserPending:actual.playerDeath.systemPending};
    executionCoverage.dispatch.push(row);
    if(row.sourceSystem!==1||row.sourcePopup!==0||row.sourceMenu!==3||row.sourceMusic!==61||
      row.browserScreen!=='main'||row.browserPopup!==null||row.browserPending!==null)executionCoverage.status='fail';
  }
  const sourceBefore=observations.get('request-1').observation.exitAudio;
  const sourceAfter=observations.get('dispatch-4').observation.exitAudio;
  const slot8Before=sourceBefore.slots.find(row=>row.id===8),slot8After=sourceAfter.slots.find(row=>row.id===8);
  assert.equal(sourceBefore.sfx.globalSfxVolume,1);
  assert.ok(slot8Before.loaded&&slot8Before.count===1&&sourceBefore.sfx.cooldownTick>=slot8Before.nextAllowed,'SFX8 is loaded, unmuted and past cooldown at the request boundary');
  const actualPid=observations.get('request-1').observation.exitAudio.music.originalPid;
  assert.equal(calls[0].api,'trace-start');assert.equal(calls[0].pid,actualPid);
  const events=calls.slice(1);
  assert.ok(events.length>0&&events.every((r,i)=>r.seq===i+1&&r.pid===actualPid),'Complete ordered Native API sequence belongs to directly owned pinned process');
  assert.ok(events.every(r=>Number.isSafeInteger(r.hostMonotonicNs)&&r.hostMonotonicNs>0));
  const pool=events.find(r=>r.api==='alGenSources'&&r.count===16);
  assert.ok(pool&&!pool.truncated&&pool.ids.length===16&&new Set(pool.ids).size===16,'Observed source16voice allocation required');
  const voices=new Set(pool.ids);
  const starts=events.filter(r=>r.api==='alSourcePlay'&&voices.has(r.source));
  assert.ok(starts.length>0,'Actual configured SFX starts required');
  for(const r of starts){
    assert.equal(r.clockDomain,'gate-main-thread');assert.ok(Number.isSafeInteger(r.virtualElapsedNs));
    assert.equal(r.actualSourceAllocationKnown,true);assert.equal(r.sourceAllocationOverflow,false);
    assert.equal(r.sfxOwner?.schema,'actual-openal-sfx-owner-v1');
    assert.equal(r.sfxOwner.callSite,'0x610deb');assert.equal(r.sfxOwner.completeTableAndPathRead,true);
    assert.ok(['UNIQUE_ACTUAL_TABLE_REFERENCE','AMBIGUOUS_ACTUAL_TABLE_REFERENCES'].includes(r.sfxOwner.status));
    assert.equal(r.sfxOwner.referenceCount,r.sfxOwner.references.length);assert.ok(r.sfxOwner.referenceCount>0);
    assert.equal(r.actualMixer?.complete,true);
    if(r.sfxOwner.references.some(ref=>ref.id===8))assert.equal(r.sfxOwner.status,'UNIQUE_ACTUAL_TABLE_REFERENCE','SFX8 aliases remain unresolved, never attributed by PCM');
  }
  const sound8=starts.filter(r=>r.sfxOwner.references[0].id===8);
  const positive=sound8.filter(r=>r.virtualElapsedNs<=low.virtualElapsedNs);
  assert.ok(positive.length>0,'Actual unique Native SFX8 earlier in the same public menu route is the observer positive control');
  const nativeWindow=starts.filter(r=>r.virtualElapsedNs>low.virtualElapsedNs&&r.virtualElapsedNs<=high.virtualElapsedNs);
  const native8=nativeWindow.filter(r=>r.sfxOwner.references[0].id===8);
  const before=frames.get('request-1'),after=frames.get('dispatch-4');
  assert.ok(before&&after&&before.state.systemClock.systemCounter<after.state.systemClock.systemCounter);
  assert.equal(before.browserAudio.truncated,false);assert.equal(after.browserAudio.truncated,false);
  const lowCounter=before.state.systemClock.systemCounter,highCounter=after.state.systemClock.systemCounter;
  const browserEvents=after.browserAudio.events.filter(r=>r.systemCounter>=lowCounter&&r.systemCounter<highCounter);
  const browser8=browserEvents.filter(r=>r.id===8);
  for(const r of api.events.filter(r=>r.type==='start'&&r.channels===1)){
    assert.ok(Number.isSafeInteger(r.observedSystemCounter)&&Array.isArray(r.observedAdmittedSounds),'Every actual mono API start requires complete same-call engine ownership');
    assert.equal(r.ownerSnapshotError,undefined);
  }
  const browserApi=api.events.filter(r=>r.type==='start'&&r.channels===1&&r.observedSystemCounter>lowCounter&&r.observedSystemCounter<=highCounter);
  for(const r of browserApi)assert.ok(Array.isArray(r.observedAdmittedSounds),'Actual API starts require same-call observed runtime owner metadata');
  const api8=browserApi.filter(r=>r.observedAdmittedSounds.some(s=>s.id===8));
  for(const r of api8){
    assert.equal(r.observedAdmittedSounds.length,1,'Do not guess API owner among multiple simultaneous admissions');
    assert.equal(r.observedAdmittedSounds[0].id,8);assert.equal(r.reachesDestination,true);
  }
  assert.equal(api8.length,browser8.filter(r=>r.output==='started'||r.output==='waiting-for-unlock').length,'Each actual Browser SFX8 request joins its observed mono API start');
  const requestCoverage={status:native8.length===0&&browser8.length===0&&api8.length===0?'pass':'fail',
    nativeConfiguredSound8Starts:native8.length,browserObservedSound8Events:browser8.length,browserActualApiSound8Starts:api8.length,
    scope:'Zero configured Native starts and zero observed Browser request/event/API starts. Rejected Native request attempts are not counted.'};
  const checks=[];const check=(field,expected,actual)=>checks.push({field,expected,actual,status:JSON.stringify(expected)===JSON.stringify(actual)?'pass':'fail'});
  check('Exit.SFX8.actualConfiguredStarts',native8.length,api8.length);
  check('Native.SFX8.windowAdmissionState',[slot8Before.lastTick,slot8Before.nextAllowed,slot8Before.lastVolume],[slot8After.lastTick,slot8After.nextAllowed,slot8After.lastVolume]);
  for(const label of ['request-1','dispatch-1','dispatch-2','dispatch-4']){
    const source=observations.get(label).observation.exitAudio.music.music,actual=frames.get(label).state.music;
    for(const field of ['currentId','queuedId','activeChannel']){
      assert.ok(Number.isSafeInteger(source[field])&&Number.isSafeInteger(actual[field]),'Music fields need actual integer observations');
      check(`${label}:Music.${field}`,source[field],actual[field]);
    }
  }
  assert.equal(checks.length,14);
  const browserPositive=api.events.filter(row=>row.type==='start'&&row.channels===1&&
    row.observedSystemCounter<=lowCounter&&row.observedAdmittedSounds.length===1&&row.observedAdmittedSounds[0].id===8);
  const positiveControls={status:'pass',nativeEarlierUniqueSound8Starts:positive,browserEarlierUniqueSound8Starts:browserPositive,
    scope:'Three earlier actual unique-owner SFX8 starts in each runtime are mandatory. No PCM/audible equality follows.'};
  if(positive.length!==3||browserPositive.length!==3||
    !positive.every(row=>row.postPlayState===0x1012&&row.actualMixer.state===0x1012&&
      row.actualMixer.gainBits===0x3f800000&&row.sfxOwner.globalVolumeBits===0x3f800000)||
    !browserPositive.every(row=>row.reachesDestination===true&&row.gain>0&&/^[a-f0-9]{64}$/.test(row.pcmSha256)))positiveControls.status='fail';
  return {schema:1,status:checks.every(r=>r.status==='pass')&&positiveControls.status==='pass'&&executionCoverage.status==='pass'&&requestCoverage.status==='pass'?'pass':'fail',requestOwnerEvidence:'pass',positiveControls,executionCoverage,requestCoverage,
    options:native.manifest.audioOptions,
    window:{sourceLowExclusive:low.virtualElapsedNs,sourceHighInclusive:high.virtualElapsedNs,browserLowInclusiveEventCounter:lowCounter,browserHighExclusiveEventCounter:highCounter,browserPostStateLowExclusive:lowCounter,browserPostStateHighInclusive:highCounter},
    nativePositiveControlSound8:positive,nativeConfiguredStarts:nativeWindow,nativeSound8:native8,
    nativeSlot8:{before:slot8Before,after:slot8After,requestBoundaryCooldownTick:sourceBefore.sfx.cooldownTick},
    browserSound8:browser8,browserActualApiSound8:api8,checks,
    scope:'Actual Native configured-SFX owner and positive-control/API sequence evidence, observed Browser request/API owner metadata and bounded Music snapshots. Zero backend starts is not a raw count of rejected request attempts. No negative claim from muted slot bytes; no audible/perceptual or complete music lifecycle parity.'};
}

// Consume actual closed canonical captures. No Native observations become
// inputs, and an incomplete capture/provenance/clock cannot imply zero starts.
export function compareExitAudioEvidence(caseData,paired) {
  validateExitAudioCases([caseData],'death-spikes-exit');
  assert.equal(paired.original.status,'pass');assert.equal(paired.original.provenance?.status,'pass');
  assert.equal(paired.original.publicClockValidation?.status,'pass');assert.equal(paired.browser.status,'pass');
  assert.deepEqual(paired.checks.map(row=>row.label),EXIT_AUDIO_LABELS);
  assert.ok(paired.checks.every(row=>Array.isArray(row.state)&&row.state.length>0),'All canonical checkpoint fields remain required');
  const directory=paired.directory;
  const actualCase=readJsonEvidence(join(directory,'case.json'));
  for(const key of ['scenario','seed','seedValue','initialSeed','bootEpochSeed','bootstrap','tape','pairedTape'])
    assert.deepEqual(actualCase[key],caseData[key],'Capture must retain its declared public case');
  const native={status:paired.original.status,provenance:paired.original.provenance,manifest:readJsonEvidence(manifestPath(directory,'original'))};
  assert.equal(native.manifest.status,'pass');assert.deepEqual(native.manifest.tape,caseData.tape);
  const candidate=readJsonEvidence(manifestPath(directory,'browser'));
  const calls=readFileSync(join(directory,'original/audio-trace/calls.jsonl'),'utf8').trim().split('\n').map(line=>JSON.parse(line));
  const api=readJsonEvidence(join(directory,'browser/api.json'));
  return compareExitAudioWindow({native,candidate,calls,api});
}
