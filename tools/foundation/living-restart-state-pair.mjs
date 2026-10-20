// Actual independent Restart owners over the existing Controls/paired capture.
import assert from 'node:assert/strict';
import {compareControls,readSavedKeyboard} from './controls-state-pair.mjs';
import {scopedState} from './paired-game.mjs';

export function restartComparer(data) {
  const contract=data.livingRestart.contract;
  // The case producer validates the compact code against the existing seed
  // codec. Source5E1170 formats that declaration before System retains it;
  // Browser BootState/Context retains the compact public code independently.
  const compactSeed=data.livingRestart.expectedSeed;
  assert.match(compactSeed,/^[ABCDEFGHJKLMNPQRSTWXYZ01234V6789]{8}$/);
  assert.equal(compactSeed,data.seed,'Restart must retain the independently declared public seed');
  const nativeSeed=compactSeed.slice(0,4)+' '+compactSeed.slice(4);
  const equal=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
  return function compareRestart(source,state,gameplay,frame) {
    const expected=contract[frame.label];
    if (!expected) return compareControls(source,state,gameplay,frame);
    const rows=[];
    const check=(field,expected,actual)=>rows.push({field,expected,actual,status:equal(expected,actual)?'pass':'fail'});
    const owner=source?.restart;
    const browser=frame.controls;
    check('Browser.Controls.reader.status','observed',browser?.status);
    check('Browser.Controls.sameTick',state.tick,browser?.tick);
    check('Browser.Controls.sameScreen',state.screen,browser?.screen);
    check('Browser.Controls.sameSystemCounter',state.systemClock.systemCounter,browser?.systemCounter);
    check('Browser.Controls.feedbackComplete',false,browser?.feedback?.truncated);
    check('Browser.Controls.halted',false,browser?.feedback?.runtimeFailed);
    check('Browser.Controls.capabilityFailures',[],browser?.feedback?.events);
    check('Browser.Controls.IOComplete',false,browser?.io?.truncated);
    check('Browser.Controls.storageKey','isaac-rebirth.inputconfigs.dat.v1',browser?.storage?.key);
    check('Browser.Controls.storageReadable','read',browser?.storage?.status);
    check('Browser.Controls.bootInputConfigs',data.startup.inputConfigs,browser?.bootInputConfigs);
    check('Browser.Controls.bootAudio',data.startup.audioPreferences,browser?.bootAudioPreferences);
    check('Browser.Controls.bootFullscreen',data.startup.fullscreenPreferences,browser?.bootFullscreenPreferences);
    check('Browser.Controls.actualSFXVolume',data.startup.audioPreferences.sfxVolume,state.audio.sfxVolume);
    check('Browser.Controls.actualMusicVolume',data.startup.audioPreferences.musicVolume,state.music.volume);
    check('Browser.Controls.actualFullscreen',false,state.fullscreen.requested);
    const bindings=expected.menu?data.expectedBindingsBefore:data.expectedBindingsAfter;
    check('Browser.Controls.Keyboard24x2',bindings,browser?.keyboard?.bindings);
    const held=!['living-before-r','living-release-resets','living-new-release'].includes(frame.label);
    check('Browser.Controls.actualPhysicalKeys',held?['KeyR']:[],browser?.physicalPressed);
    check('Native.Restart.owner.status','pass',owner?.status);
    check('Native.Restart.owner.readOnly',false,owner?.memoryWritten);
    check('Native.Restart.owner.noInvocation',false,owner?.functionsInvoked);
    check('Native.Restart.owner.noCodeRead',false,owner?.executableMappingsRead);
    check('Native.Restart.owner.bounded',true,Number.isInteger(owner?.bytesRead)&&owner.bytesRead<=4096);
    assert.equal(source?.status,'pass');
    check('Native.actualSystemState',expected.menu?1:2,owner.systemState);
    check('Browser.actualScreen',expected.menu?'main':'playing',state.screen);
    check('Native.hold61Law',expected.heldPolls??0,owner.heldPolls);
    check('Browser.hold61Law',expected.heldPolls??0,state.livingRestart?.heldPolls);
    if (expected.menu) {
      check('Native.menuDoesNotRequest',false,owner.pending);
      check('Browser.menuDoesNotRequest',null,state.livingRestart?.pending);
      check('Browser.Controls.noEarlySave',0,browser?.io?.events?.filter(event=>event.operation==='save').length);
      check('Browser.Controls.noEarlyStoredFile',null,browser?.storage?.text);
      return rows;
    }
    const keyboard=source.playingKeyboard;
    check('Native.Controls.reader.status','pass',keyboard?.status);
    check('Native.Controls.reader.readOnly',false,keyboard?.memoryWritten);
    check('Native.Controls.reader.noCodeRead',false,keyboard?.executableMappingsRead);
    check('Native.Controls.KeyboardId',0,keyboard?.input?.keyboardId);
    check('Native.Controls.KeyboardEnabled',true,keyboard?.input?.enabled);
    check('Native.Controls.Keyboard24x2',bindings,keyboard?.input?.bindings);
    check('Native.Controls.actualPhysicalKeys',held?[82]:[],keyboard?.input?.currentPhysical);
    check('Native.Controls.savedComplete24x2',[{name:'Keyboard',bindings}],keyboard?.file?.devices);
    check('Native.Controls.savedFileStatus','pass',keyboard?.file?.status);
    check('Browser.Controls.savedComplete24x2',[{name:'Keyboard',bindings}],readSavedKeyboard(browser.storage.text));
    const writes=browser.io.events.filter(event=>event.operation==='save');
    check('Browser.Controls.oneBackSave',1,writes.length);
    check('Browser.Controls.savedSequence',0,writes[0]?.sequence);
    check('Browser.Controls.savedStatus','pass',writes[0]?.status);
    check('Browser.Controls.retainedConfigs',{devices:[{name:'Keyboard',bindings}]},browser.retainedConfigurations);
    check('independent.Controls.held24',keyboard?.input?.derivedSourceQueryResults?.held,browser.actions.held);
    check('independent.Controls.triggered24',keyboard?.input?.derivedSourceQueryResults?.triggered,browser.actions.triggered);
    check('independent.Controls.rawFirstNew',keyboard?.input?.derivedSourceQueryResults?.rawKey,browser.actions.rawKey??-1);
    rows.push(...scopedState(source.standard,state,true));
    check('Native.pending',expected.pending,owner.pending);
    check('Browser.pending',expected.pending,Boolean(state.livingRestart?.pending));
    check('Native.seededSaveEnabled',!expected.pending,owner.saveEnabled);
    check('Browser.seededSaveEnabled',!expected.pending,state.runQuit?.saveEnabled);
    check('Native.actualPrimaryIsaac',0,owner.primary?.characterType);
    check('Native.actualPrimaryLiving',false,owner.primary?.dead);
    check('Browser.actualPrimaryLiving',false,state.player.dead);
    check('Native.retainedSelector',0,owner.context?.characterSelector);
    check('Native.retainedChallenge',0,owner.context?.challengeId);
    check('Native.retainedR8',0,owner.context?.sourceR8);
    check('Native.retainedR9',true,owner.context?.sourceR9);
    check('Native.retainedLoader',false,owner.context?.savedLoader);
    check('Native.retainedSpecialStart',false,owner.context?.specialStart);
    check('Native.retainedTypedSeed',nativeSeed,owner.context?.seedCode);
    check('Browser.retainedTypedSeed',compactSeed,state.livingRestart?.context?.seedCode);
    check('Browser.actualRunSeed',compactSeed,state.runSeed);
    check('Native.seededInhibit',true,owner.persistent?.inhibit);
    check('Browser.seededInhibit',true,state.persistentRuntime?.inhibit);
    const keys=source.playingKeyboard?.input?.derivedSourceQueryResults;
    check('Native.actualR16Held',held,keys?.held?.[16]);
    check('independent.R16Held',keys?.held?.[16],state.publicActions?.held?.[16]);
    check('Browser.noUnavailableR16',false,state.controlsMenu?.unavailableActions?.includes(16));
    if (data.livingRestart.paused&&!expected.dispatch) {
      check('Browser.pausedBeforeDispatch',true,state.paused);
      check('Native.pausedBeforeDispatch',true,source.standard.pause?.paused);
    }
    if (expected.dispatch) {
      check('Browser.restartUnpauses',false,state.paused);
      check('Browser.restartedGameAge',true,state.gameUpdateCount<=1);
      check('Native.restartedGameAge',true,owner.fullGameUpdates<=1);
    }
    return rows;
  };
}
