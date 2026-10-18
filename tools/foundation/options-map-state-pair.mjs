// Actual owners only. Expected UI arithmetic is independent of either capture.
import assert from 'node:assert/strict';
const equal = (a,b) => a !== undefined && b !== undefined && JSON.stringify(a) === JSON.stringify(b);
const word = value => {assert.ok(Number.isFinite(value)); const b=Buffer.alloc(4); b.writeFloatLE(value); return b.readUInt32LE();};
export function validateMapDeclaration(data, contract) {
  assert.equal(contract.schema,1); assert.equal(contract.preferenceSchema,3);
  assert.equal(contract.storageKey,'isaac-rebirth.options.v3');
  assert.equal(data.declarationMode,'map-opacity-fast31-source-v1');
  assert.equal(contract.caseDeclarationMode,data.declarationMode);
  assert.deepEqual(data.tape,[...data.bootstrap.sourceOnlyPrelude,...data.pairedTape]);
  for(const [key,tape,count] of [['nativeLabels',data.tape,31],['pairedLabels',data.pairedTape,30]]) {
    const labels=tape.filter(a=>a.type==='capture').map(a=>a.label);
    assert.equal(labels.length,count);assert.equal(new Set(labels).size,count);
    assert.deepEqual(contract[key],labels);assert.deepEqual(data.captureDeclaration[key],labels);
  }
  assert.deepEqual(contract.sourceOnlyLabels,['intro']);
  const groups=[contract.menuLabels,contract.playingLabels,contract.provenanceOnlyLabels];
  assert.deepEqual(groups.map(a=>a.length),[16,11,4]);
  const all=groups.flat();assert.equal(new Set(all).size,31);
  assert.deepEqual([...all].sort(),[...contract.nativeLabels].sort());
  assert.deepEqual(data.declaredAudioOptions,{sfxVolume:1,musicVolume:1});assert.equal(data.declaredFullscreen,false);
  assert.deepEqual(Object.keys(contract.rectangles).sort(),['menuGauge','playingMaps']);
  assert.deepEqual(contract.rectangles.playingMaps.labels,contract.playingLabels);
  assert.deepEqual(contract.rectangles.menuGauge.labels,contract.menuLabels.filter(label=>!['title-ready','options-initial-row-0','options-map-back-saved'].includes(label)));
  for(const spec of Object.values(contract.rectangles)) {
    assert.equal(spec.required,true);assert.ok(spec.labels.length>0);
    for(const label of spec.labels)assert.ok(contract.pairedLabels.includes(label));
    const {x,y,width,height}=spec.region;
    assert.ok([x,y,width,height].every(Number.isInteger)&&x>=0&&y>=0&&width>0&&height>0&&x+width<=960&&y+height<=540);
  }
}

function mapInputLaw(data) {
  const pressed=new Set(), labels=new Map(),requests=[];
  let presentation=0,active=false,left=0,right=0,opacity=Math.fround(.3);
  for(const a of data.pairedTape) {
    if(a.type==='key'){if(a.down)pressed.add(a.key);else pressed.delete(a.key);}
    else if(a.type==='step')for(let n=0;n<a.frames;n++) {
      presentation++;
      if(!active)continue;
      const l=pressed.has('Left'),r=pressed.has('Right');
      const prior=l?left:right;
      if((l||r)&&(prior===0||(prior>49&&prior%10===0))) {
        opacity=Math.fround(Math.max(0,Math.min(1,Math.fround(opacity+Math.fround(l?-.1:.1)))));
        requests.push({presentation,opacity});
      }
      left=l?left+1:0;right=r?right+1:0;
    } else {
      if(a.label==='options-map-row-3')active=true;
      if(active)labels.set(a.label,{presentation,opacity,left,right,requests:requests.length});
      if(a.label==='options-map-back-saved')active=false;
    }
  }
  return{labels,requests};
}

//5B1070..10E3 adds exactly one f32 .1 per mode0 Full. Count the
//literal released interval using each runtime's own post-System counter;
//neither side's clock or alpha supplies the other's expected value.
function smallMapReturn(data, counter) {
  assert.ok(Number.isInteger(counter) && counter >= 0 && counter <= 0xffffffff);
  let active = false, ended = false, presentations = 0;
  for (const action of data.pairedTape) {
    if (action.type === 'capture' && action.label === 'map-second-tab-release-1') {
      assert.equal(active, false);
      active = true;
    } else if (action.type === 'capture' && action.label === 'map-second-tab-release-14') {
      assert.equal(active, true);
      ended = true;
      break;
    } else if (active) {
      assert.equal(action.type, 'step', 'The second released interval must retain only public steps');
      presentations += action.frames;
    }
  }
  assert.equal(ended, true);
  let fullUpdates = 0, alpha = 0;
  for (let step = 0; step < presentations; step++) {
    if (((counter + step) & 1) !== 0) continue;
    fullUpdates++;
    alpha = alpha < Math.fround(.9) ? Math.fround(alpha + Math.fround(.1)) : 1;
  }
  return {presentations, fullUpdates, alpha};
}

export function compareOptionsMap({data,contract,clock,native,browser,observed,calls,scopedState,pcm237Sha256}) {
  const checks=[],diagnostics=[];
  const check=(field,expected,actual)=>checks.push({field,expected,actual,status:equal(expected,actual)?'pass':'fail'});
  const f32=(field,expected,actual)=>check(field+'.word',word(expected),Number.isFinite(actual)?word(actual):undefined);
  let error;
  try {
    validateMapDeclaration(data,contract);
    assert.equal(native.status,'pass');assert.equal(native.provenance.status,'pass');assert.equal(native.publicClockValidation.status,'pass');
    assert.equal(browser.status,'pass');assert.deepEqual(observed.failures,[]);assert.deepEqual(observed.errors,[]);
    const ns=new Map(native.manifest.debugObservations.map(r=>[r.context.label,r.observation]));
    const nf=new Map(native.manifest.frames.map(r=>[r.label,r]));const bf=new Map(browser.frames.map(r=>[r.label,r]));
    assert.deepEqual(native.manifest.frames.map(r=>r.label),contract.nativeLabels);
    assert.deepEqual(browser.frames.map(r=>r.label),contract.pairedLabels);
    assert.equal(native.manifest.optionsAudioOwnerObserver,1);
    assert.deepEqual(native.manifest.audioOptions,{sfxVolume:1,musicVolume:1});
    check('Native.actualReleasedPresentations','drawn',native.manifest.hostExecution?.releasedPresentations);
    check('Native.actualWindowAttributeReplies','forwarded',native.manifest.hostExecution?.windowAttributeReplies);
    const intro=ns.get('intro');assert.equal(intro?.status,'pass');assert.equal(intro.standard.status,'pass');
    check('intro.actualScopeSkip','skip',intro.map.status);
    for(const field of ['optionsReaderCalled','playingReaderCalled','audioReaderCalled'])check('intro.'+field,false,intro.map[field]);
    check('allActualCheckpointLabels',contract.pairedLabels,observed.checkpoints.map(c=>c.label));
    check('bootMapPreferences.actualDefault',null,observed.trace.bootMapPreferences);
    check('publicInputComplete',false,observed.trace.inputTraceTruncated);check('backendHistoryComplete',false,observed.trace.backendStateTruncated);
    check('physicalComplete',false,observed.physical.truncated);
    const key=k=>({Return:'Enter',Escape:'Escape',Tab:'Tab',Left:'ArrowLeft',Right:'ArrowRight',Up:'ArrowUp',Down:'ArrowDown'})[k]??(/^\d$/.test(k)?'Digit'+k:'Key'+k.toUpperCase());
    check('actualPhysicalTape',data.pairedTape.filter(a=>a.type==='key').map(a=>({type:a.down?'keydown':'keyup',code:key(a.key)})),observed.physical.events.map(e=>({type:e.type,code:e.code})));
    check('actualPhysicalTrusted',true,observed.physical.events.every(e=>e.isTrusted&&!e.repeat&&!e.ctrlKey&&!e.altKey&&!e.metaKey&&!e.shiftKey));
    check('actualStepOwnerComplete',false,observed.returns.truncated);check('actualStepOwnerFailure',null,observed.returns.error);
    check('actualStepCount',clock.browserTickCommands.length,observed.returns.rows.length);
    check('actualStepTimes',clock.browserTickCommands.map(c=>c.timestampMillis),observed.returns.rows.map(r=>r.timestampMillis));
    check('actualStepPresentationOrder',true,observed.returns.rows.every((r,i)=>r.presentation===i+1&&r.tick===i+1));
    check('actualAPIComplete',false,observed.api.truncated);
    const law=mapInputLaw(data),title=ns.get('title-ready').map,bt=bf.get('title-ready').state;
    f32('initial.NativeDefault',Math.fround(.3),title.opacity.value);f32('initial.BrowserDefault',Math.fround(.3),bt.mapSettings.opacity);
    const allAdmitted=observed.returns.rows.flatMap(r=>r.audioEvents.filter(e=>e.id===237).map(event=>({presentation:r.presentation,event})));
    const beforeFile=title.options.file;
    diagnostics.push({field:'rawIndependentMenuClockOrigins',Native:title.systemCounter,Browser:bt.systemClock.systemCounter,scope:'Read-only comparison origins; never inputs or an inferred common clock'});
    diagnostics.push({field:'fullStateHistoryCap',status:observed.trace.stateHistoryTruncated?'fail':'pass',states:observed.trace.states.length,commands:observed.trace.commands.length});
    for(const label of contract.pairedLabels) {
      const source=ns.get(label),frame=bf.get(label),original=nf.get(label);assert.ok(source&&frame&&original,label);
      assert.equal(source.status,'pass');assert.equal(source.memoryWritten,false);assert.equal(source.executableMappingsRead,false);
      check(label+'.presentation',original.presentedFrame-nf.get('title-ready').presentedFrame,frame.browserPresentation);
      const io=observed.checkpoints.find(c=>c.label===label);assert.ok(io,label+' IO owner');
      check(label+'.IOComplete',false,io.preferences.truncated);check(label+'.IOAvailable',true,io.preferences.storageAvailable);
      check(label+'.IOErrors',[],io.preferences.events.filter(e=>e.status==='fail'));
      check(label+'.feedbackComplete',false,io.feedback.truncated);check(label+'.feedbackFailures',[],io.feedback.events.filter(e=>e.status==='fail'));
      check(label+'.storageReadKey',contract.storageKey,io.storage.key);check(label+'.runtimeStorageKey',contract.storageKey,io.preferences.key);
      if(contract.provenanceOnlyLabels.includes(label)) {
        check(label+'.mapScopeSkip','skip',source.map.status);check(label+'.noMenuRead',false,source.map.optionsReaderCalled);
        check(label+'.noPlayingRead',false,source.map.playingReaderCalled);check(label+'.noAudioRead',false,source.map.audioReaderCalled);
        for(const row of scopedState(source.standard,frame.state,false))checks.push({...row,field:label+'.standard.'+row.field});
        continue;
      }
      const actual=frame.state,map=source.map;assert.equal(map.status,'pass');
      f32(label+'.mapOpacity',map.opacity.value,actual.mapSettings?.opacity);
      check(label+'.relativeSystemCounter',map.systemCounter-title.systemCounter,actual.systemClock.systemCounter-bt.systemClock.systemCounter);
      if(contract.menuLabels.includes(label)) {
        check(label+'.menuReader',true,map.optionsReaderCalled);check(label+'.noPlayingReader',false,map.playingReaderCalled);check(label+'.audioReader',true,map.audioReaderCalled);
        const menu=map.options;assert.equal(menu.status,'pass');
        check(label+'.screen',({1:'title',2:'profile',3:'main',4:'character',8:'options'})[menu.menu.screen],actual.screen);
        for(const field of ['selection','leftHeld','rightHeld','verticalHeld'])check(label+'.options.'+field,menu.options[field],actual.optionsMenu[field]);
        check(label+'.menuMoving',menu.menu.moving,actual.menuPresentation?.moving);
        for(let axis=0;axis<2;axis++)check(label+'.menuView.'+axis+'.word',menu.menu.rawView?.[axis]?.bits,Number.isFinite(actual.menuPresentation?.view?.[axis])?word(actual.menuPresentation.view[axis]):undefined);
        if(actual.screen==='options') {
          const row=menu.options.selection, target=row<=2?-1080:row<=6?-1080-(row*18-36):-1170;
          check(label+'.sourceNativeOptionsTarget.word',[word(-480),word(target)],Array.isArray(menu.menu.rawRequestedPageTarget)?menu.menu.rawRequestedPageTarget.map(owner=>owner?.bits):undefined);
          f32(label+'.sourceBrowserOptionsTarget',target,actual.menuPresentation?.optionsScroll);
        }
        check(label+'.SFX.word',menu.settings.sfxVolume.bits,word(actual.audio.sfxVolume));
        check(label+'.music.word',menu.settings.rawMusicVolume.bits,word(actual.music.volume));
        check(label+'.musicEnabled',menu.settings.musicEnabled,actual.music.musicEnabled);
        check(label+'.fullscreen',menu.settings.fullscreenRequested,actual.fullscreen.requested);
        assert.equal(map.audio237.status,'pass');assert.equal(map.audio237.slots.length,1);
        f32(label+'.globalSFX',map.audio237.globalSfxVolume,actual.audio.sfxVolume);
        const expected=law.labels.get(label);
        if(expected&&label!=='options-map-back-saved') {
          f32(label+'.independentNativeUI',expected.opacity,map.opacity.value);f32(label+'.independentBrowserUI',expected.opacity,actual.mapSettings.opacity);
          check(label+'.sourceLeft',expected.left,menu.options.leftHeld);check(label+'.sourceRight',expected.right,menu.options.rightHeld);
        }
        if(expected?.requests>0) {
          const n=map.audio237.slots[0],b=actual.audio.slots['237'];assert.ok(b,'Missing actual Browser237 slot');
          check(label+'.237.NativeLoaded',true,n.loaded);check(label+'.237.BrowserLoaded',true,b.loaded);
          check(label+'.237.NativeDelay',0,n.nextAllowed-n.lastTick);check(label+'.237.BrowserDelay',0,b.nextAllowed-b.lastTick);
          f32(label+'.237.lastVolume',n.lastVolume,b.lastVolume);
          check(label+'.237.NativeSampleCount',1,n.samples.length);check(label+'.237.BrowserSampleCount',1,b.samples.length);
          for(const field of ['gain','pitch'])f32(label+'.237.sample.'+field,n.samples[0][field],b.samples[0][field]);
          check(label+'.237.looping',n.samples[0].loop,b.samples[0].looping);
          const last=allAdmitted.filter(e=>e.presentation<=frame.browserPresentation).at(-1);assert.ok(last,'Missing actual admitted237 request');
          check(label+'.237.actualBrowserLastTick',last.event.halfTick,b.lastTick);
          const latest=calls.filter(c=>c.api==='alSourcePlay'&&c.clockDomain==='gate-main-thread'&&c.virtualElapsedNs<=original.virtualElapsedNs&&c.sfxOwner?.references?.some(r=>r.id===237)).at(-1);
          assert.ok(latest,'Missing actual configured Native237 start');
          check(label+'.237.actualNativeLastTick',latest.sfxOwner.references.find(r=>r.id===237).lastTickBits|0,n.lastTick);
        }
        if(label!=='options-map-back-saved') {
          check(label+'.noEagerNativeSave',beforeFile.sha256,menu.file.sha256);
          check(label+'.noEagerBrowserSave',0,io.preferences.events.filter(e=>e.operation==='save').length);
          check(label+'.actualStorageStillAbsent',null,io.storage.text);
        }
      } else {
        check(label+'.noMenuReader',false,map.optionsReaderCalled);check(label+'.playingReader',true,map.playingReaderCalled);check(label+'.noMenuAudioReader',false,map.audioReaderCalled);
        for(const row of scopedState(source.standard,actual,true))checks.push({...row,field:label+'.standard.'+row.field});
        check(label+'.FullCounter',map.gameFullCounter,actual.gameUpdateCount);
        for(const field of ['mode','expandedLatched','heldFrames'])check(label+'.map.'+field,map.presentation[field],actual.minimapPresentation[field]);
        for(const field of ['smallAlpha','expandedAlpha'])check(label+'.map.'+field+'.word',map.presentation[field].bits,word(actual.minimapPresentation[field]));
        check(label+'.ordinaryScale.word',word(1),map.presentation.scale.bits);check(label+'.ordinaryOffsets.word',[word(0),word(0)],map.presentation.offset.map(o=>o.bits));
        assert.ok(Array.isArray(frame.render?.maps),'Missing actual Renderer map submission owner');
        const p=map.presentation,expected=[];
        if(p.smallAlpha.value>0)expected.push('small-frame','small');if(p.expandedAlpha.value>0)expected.push('expanded');
        check(label+'.submittedSurfaces',expected,frame.render.maps.map(r=>r.surface));
        for(const r of frame.render.maps) {
          f32(label+'.render.'+r.surface+'.alpha',r.surface==='expanded'?p.expandedAlpha.value:p.smallAlpha.value,r.alpha);
          if(r.surface==='small-frame')check(label+'.smallFramePosition',[400,16],r.position);
          if(r.surface==='small'){check(label+'.smallSource',[960,540],r.sourceSize);check(label+'.smallDestination',[480,270],r.destinationSize);check(label+'.smallPosition',[0,0],r.position);}
          if(r.surface==='expanded'){
            check(label+'.expandedSource',[230,204],r.sourceSize);check(label+'.expandedDestination',[230,204],r.destinationSize);
            assert.equal(actual.floor.grid.length,169,'Missing actual Browser floor grid for the Source anchoring law');
            const occupied=actual.floor.grid.flatMap((index,grid)=>index>=0&&actual.roomDisplayFlags[index]?[{x:grid%13*17,y:Math.floor(grid/13)*15}]:[]);
            assert.ok(occupied.length>0,'Positive expanded map requires actual occupied cells');
            const position=[480-Math.max(...occupied.map(c=>c.x+26))-24,14-Math.min(...occupied.map(c=>c.y))];
            check(label+'.SourceAnchorOnActualBrowserGrid',position,r.position);
          }
        }
      }
    }
    for(const [label,values] of [
      ['map-small-steady',{mode:0,expandedLatched:false,heldFrames:0,smallAlpha:1,expandedAlpha:0}],
      ['map-first-tab-release-18',{mode:2,expandedLatched:true,heldFrames:0,smallAlpha:0}],
      ['map-second-tab-held-2',{mode:1,heldFrames:0}],
      ['map-second-tab-release-1',{mode:0,expandedLatched:false,heldFrames:0,smallAlpha:0}],
      ['map-second-tab-release-14',{mode:0,expandedLatched:false,heldFrames:0,expandedAlpha:0}],
    ])for(const [field,value] of Object.entries(values)) {
      const n=ns.get(label).map.presentation[field],b=bf.get(label).state.minimapPresentation[field];
      if(field.endsWith('Alpha')) {check(label+'.requiredNative.'+field+'.word',word(value),n.bits);f32(label+'.requiredBrowser.'+field,value,b);}
      else {check(label+'.requiredNative.'+field,value,n);check(label+'.requiredBrowser.'+field,value,b);}
    }
    const returnStart='map-second-tab-release-1',returnEnd='map-second-tab-release-14';
    for(const side of ['Native','Browser']) {
      const start=side==='Native'?ns.get(returnStart).map:bf.get(returnStart).state;
      const end=side==='Native'?ns.get(returnEnd).map:bf.get(returnEnd).state;
      const counter=owner=>side==='Native'?owner.systemCounter:owner.systemClock.systemCounter;
      const full=owner=>side==='Native'?owner.gameFullCounter:owner.gameUpdateCount;
      let expectedAlpha;
      try {
        const counters=[counter(start),counter(end),full(start),full(end)];
        assert.ok(counters.every(value=>Number.isInteger(value)&&value>=0&&value<=0xffffffff), 'Missing actual u32 System/Full return owners');
        const expected=smallMapReturn(data,counter(start));
        expectedAlpha=word(expected.alpha);
        check(returnEnd+'.'+side+'.releasedPublicPresentationCount',expected.presentations,counter(end)-counter(start));
        check(returnEnd+'.'+side+'.sourceFullCallbacks',expected.fullUpdates,full(end)-full(start));
      } catch(failure) {
        check(returnEnd+'.'+side+'.sourceReturnLaw',null,String(failure));
      }
      const alpha=side==='Native'?end.presentation?.smallAlpha?.bits:end.minimapPresentation?.smallAlpha;
      const actual=side==='Native'?alpha:Number.isFinite(alpha)?word(alpha):undefined;
      check(returnEnd+'.required'+side+'.smallAlpha.word',expectedAlpha,actual);
    }
    check('map-first-tab-release-18.actualNativeTarget.word',ns.get('map-first-tab-release-18').map.opacity.bits,ns.get('map-first-tab-release-18').map.presentation.expandedAlpha.bits);
    const final=law.labels.get('map-final-value-tap-4');assert.ok(final&&law.requests.length>0);
    const saved=observed.checkpoints.find(r=>r.label==='options-map-back-saved');
    const back=ns.get('options-map-back-saved').map.options.file;
    const ini=key=>{const matches=[...back.text.matchAll(new RegExp('^'+key+'=([^\\r\\n]+)$','gm'))];assert.equal(matches.length,1);return matches[0][1];};
    const serialized=final.opacity.toFixed(4),expectedStore={schema:3,sfxVolume:'1.0000',musicVolume:'1.0000',fullscreen:false,mapOpacity:serialized};
    check('Back.NativeMapOpacity',serialized,ini('MapOpacity'));check('Back.NativeSFX','1.0000',ini('SFXVolume'));check('Back.NativeMusic','1.0000',ini('MusicVolume'));check('Back.NativeFullscreen','0',ini('Fullscreen'));
    assert.ok(Number.isInteger(back.mode)&&Number.isInteger(back.uid)&&Number.isInteger(back.gid),'Missing actual Native file metadata');
    check('Back.NativeFileActuallyChanged',true,back.sha256!==beforeFile.sha256);
    check('Back.actualBrowserStorage',expectedStore,JSON.parse(saved.storage.text));
    const saves=saved.preferences.events.filter(e=>e.operation==='save'&&e.status==='pass');check('Back.actualSaveCount',1,saves.length);
    check('Back.actualSaveValue',expectedStore,saves[0]?.value);check('Back.actualSaveKey',contract.storageKey,saves[0]?.key);
    const saveRows=observed.returns.rows.flatMap(r=>r.options.saveRequests.map(request=>({presentation:r.presentation,request})));
    check('Back.actualSourceRequestCount',1,saveRows.length);check('Back.actualRequestMap',serialized,saveRows[0]?.request.mapOpacity);
    check('Back.actualSequenceJoin',saveRows[0]?.request.sequence,saves[0]?.sequence);
    check('initialActualAbsentLoad',true,saved.preferences.events.some(e=>e.operation==='load'&&e.status==='absent'&&e.key===contract.storageKey));
    const lo=nf.get('options-map-row-3').virtualElapsedNs,hi=nf.get('options-map-back-saved').virtualElapsedNs;
    check('Map237.noUnknownRequiredOwner',0,calls.filter(c=>c.clockDomain==='gate-main-thread'&&c.virtualElapsedNs>lo&&c.virtualElapsedNs<=hi&&
      (c.optionsOwner?.status?.startsWith('UNKNOWN')||c.sfxOwner?.status?.startsWith('UNKNOWN'))).length);
    const starts=calls.filter(c=>c.clockDomain==='gate-main-thread'&&c.virtualElapsedNs>lo&&c.virtualElapsedNs<=hi&&c.api==='alSourcePlay'&&c.sfxOwner?.references?.some(r=>r.id===237));
    const api=observed.api.events.filter(e=>e.type==='start'&&e.pcmSha256===pcm237Sha256&&e.observedAdmittedSounds?.some(s=>s.id===237));
    check('Map237.NativeActualStarts',law.requests.length,starts.length);check('Map237.BrowserActualStarts',law.requests.length,api.length);
    assert.ok(starts.length===law.requests.length&&api.length===law.requests.length,'Missing configured Map237 admission/API owner');
    const admitted=allAdmitted;
    check('Map237.actualRequestPresentations',law.requests.map(r=>r.presentation),admitted.map(r=>r.presentation));
    for(let i=0;i<starts.length;i++) {
      const s=starts[i],a=api[i],ref=s.sfxOwner.references.find(r=>r.id===237),event=admitted[i].event;
      check(`Map237.${i}.uniqueOwner`,'UNIQUE_ACTUAL_TABLE_REFERENCE',s.sfxOwner.status);check(`Map237.${i}.uniqueCount`,1,s.sfxOwner.references.length);
      check(`Map237.${i}.NativeSpacing`,law.requests[i].presentation-law.requests[0].presentation,s.sfxOwner.systemCounter-starts[0].sfxOwner.systemCounter);
      check(`Map237.${i}.BrowserSpacing`,law.requests[i].presentation-law.requests[0].presentation,event.systemCounter-admitted[0].event.systemCounter);
      check(`Map237.${i}.cooldownDelay`,0,ref.delayBits);check(`Map237.${i}.requestedVolume.word`,word(1),ref.requestedVolumeBits);
      check(`Map237.${i}.NativeMixerComplete`,true,s.actualMixer?.complete);
      check(`Map237.${i}.NativeGain.word`,word(1),s.actualMixer?.gainBits);check(`Map237.${i}.NativePlaying`,0x1012,s.postPlayState);
      check(`Map237.${i}.BrowserGain.word`,s.actualMixer?.gainBits,Number.isFinite(a.gain)?word(a.gain):undefined);
      check(`Map237.${i}.BrowserPitch.word`,s.actualMixer?.pitchBits,Number.isFinite(a.pitch)?word(a.pitch):undefined);
      check(`Map237.${i}.BrowserLooping`,s.actualMixer?.loopRaw===1,a.looping);
      check(`Map237.${i}.configuredPath`,'resources/sfx/plop.wav',ref.path?.toLowerCase());
      check(`Map237.${i}.BrowserImmediate`,0,a.when);check(`Map237.${i}.BrowserDestination`,true,a.reachesDestination);
      check(`Map237.${i}.BrowserActualRequestJoin`,true,a.observedAdmittedSounds.some(e=>e.id===237&&e.sequence===event.sequence));
    }
    diagnostics.push({field:'Map237.independentActualOrigins',Native:starts[0].sfxOwner.systemCounter,Browser:admitted[0].event.systemCounter,requests:law.requests.length,scope:'Literal relative request-spacing comparisons retain both raw origins; no observed clock becomes an input. PCM/fullmix equality not claimed here.'});
  } catch(e) {error={message:String(e.message),stack:e.stack};}
  return{schema:1,status:!error&&checks.length>0&&checks.every(r=>r.status==='pass')?'pass':'fail',checks,diagnostics,error,
    scope:'One fixed31/30 Map public route: actual UI/held/default/Back IO, ordinary map state and submitted Browser surfaces, clock/provenance and actual237 owners. Required strict pixel regions reported separately. No reload/migration/banner/controller/fullmix/completeMT acceptance.'};
}
