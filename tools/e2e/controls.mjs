// Actual Native and production Browser public keyboard remap/Back/movement gate.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {copyFileSync,existsSync,mkdirSync,readFileSync,statfsSync,writeFileSync} from 'node:fs';
import {dirname,resolve} from 'node:path';
import {runPairedGate} from '../foundation/paired-game.mjs';
import {compareControls,compareMovement} from '../foundation/controls-state-pair.mjs';
import {comparersFor} from '../foundation/controls-bank-state-pair.mjs';
import {ROOT,flag,selectCases} from './lib/cli.mjs';
import {ORIGINAL_SHA,assertOriginalBinary} from './lib/original.mjs';

for (const name of ['--output','--production-directory','--scenarios'])
  assert.ok(flag(name)===undefined || flag(name).length>0,`${name} requires a nonempty value`);
assert.equal(flag('--reuse-original'),undefined,'Registered Controls requires fresh Native; cached Native reuse is not admitted by this producer');
const recorded=flag('--scenarios');
const controlsOwnReload=process.argv.includes('--keyboard-own-reload');
const livingRestart=process.argv.includes('--living-restart');
assert.ok(!(livingRestart&&controlsOwnReload),'Restart and own Keyboard reload declare separate public routes');
const restartComparer=livingRestart?(await import('../foundation/living-restart-state-pair.mjs')).restartComparer:undefined;
const scenarioProducer=livingRestart?'tools/original/living-restart-scenarios.py':'tools/original/controls-scenarios.py';
const reloadIds=['controls-movement-up-w-to-i-v1','controls-os-movement-up-w-to-i-v1'];
const output=resolve(flag('--output',`.cache/e2e/controls/${Date.now()}-${process.pid}`));
assert.ok(!existsSync(output),'Controls needs a fresh evidence directory');
mkdirSync(output,{recursive:true});
const save=(path,data)=>writeFileSync(path,JSON.stringify(data,null,2)+'\n',{flag:'wx'});
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const fileDigest=path=>digest(readFileSync(path));
const producer='tools/foundation/paired-game.mjs';
const observer=livingRestart?'tools/original/living-restart-observe.py':'tools/original/controls-paired-observer.py';
const sourcePaths=[
  'tools/e2e/lib/game.mjs', 'tools/foundation/pixel-pairs.mjs',
  'tools/e2e/controls.mjs','tools/e2e/lib/cli.mjs','tools/e2e/lib/original.mjs',
  producer,'tools/foundation/verify.mjs','tools/foundation/controls-state-pair.mjs','tools/foundation/controls-bank-state-pair.mjs',
  'tools/foundation/controls-browser-observe.mjs','tools/foundation/browser.mjs',
  ...(controlsOwnReload ? ['tools/foundation/controls-keyboard-reload.mjs',
    'references/isaac/foundation/combat/tapes.json'] : []),
  'tools/foundation/production-server.mjs','tools/foundation/compare.mjs',
  'tools/foundation/native-evidence.mjs','tools/foundation/browser-evidence.mjs',
  'tools/foundation/native-journal-zstd.mjs','tools/foundation/mt-draws.mjs',
  'tools/e2e/lib/audio-owners.mjs','tools/e2e/audio-api-observer.mjs',
  'tools/original/controls-scenarios.py',observer,'tools/original/controls-keyboard-observe.py',
  'tools/original/controls-playing-keyboard-observe.py','tools/original/native-identity.py',
  ...(livingRestart?['tools/original/controls-paired-observer.py',scenarioProducer,
    'tools/foundation/living-restart-state-pair.mjs']:[]),
  'tools/original/audio-observer.py','tools/original/foundation-paired-observer.py',
  'tools/original/foundation-menu-observer.py','tools/original/foundation-combat-observe.py',
  'tools/original/state-observer.py','tools/original/foundation-clock-run.py',
  'tools/original/foundation-clock-gate.c','tools/original/foundation-clock-verify.py',
  'tools/original/native-evidence.py','tools/original/native-journal-zstd.py',
  'tools/original/foundation-paired-scenarios.py','tools/original/foundation-clock-scenarios.py',
  'tools/original/foundation-bomb-scenario.py','tools/original/foundation-presentation-scenario.py',
  'tools/original/foundation-title-scenario.py','tools/original/foundation-combat-scenario.py',
  'tools/original/foundation-random-seeds.py','tools/original/foundation-seeds.py',
  'tools/original/debug-run.py','tools/original/debug-config.json','tools/original/with-display.sh',
  'references/isaac/cases/controls/recorded-four-fixed-and-9xzmzpr0-v1.json','package-lock.json',
];
const sourceIdentity=()=>Object.fromEntries(sourcePaths.map(name=>[name,fileDigest(resolve(ROOT,name))]));
const results=[];
let failure,bank,before,producerIdentity;
try {
  before=sourceIdentity();
  assert.equal(fileDigest(new URL(import.meta.url)),before['tools/e2e/controls.mjs'],'Archive the actual executing driver');
  for (const name of sourcePaths) {
    const target=resolve(output,'executing-source',name);
    mkdirSync(dirname(target),{recursive:true});
    copyFileSync(resolve(ROOT,name),target);
    assert.equal(fileDigest(target),before[name],'Archived executing Source must match its identity');
  }
  save(resolve(output,'source-before.json'),before);
  producerIdentity={schema:1,mode:livingRestart?'controls-living-restart-v1':controlsOwnReload?'controls-own-keyboard-reload-v1':'controls-v1',binarySha256:ORIGINAL_SHA,
    captureProducerSha256:before[producer],observerSha256:before[observer],
    sources:before,publicPreferences:{sfxVolume:1,musicVolume:1,fullscreen:false,inputConfigs:{devices:[]}}};
  save(resolve(output,'producer-identity.json'),producerIdentity);
  bank=JSON.parse(recorded!==undefined ? readFileSync(resolve(recorded),'utf8') :
    execFileSync('python3',[scenarioProducer,...(livingRestart?[]:['--count',flag('--count','1')])],
      {cwd:ROOT,encoding:'utf8',maxBuffer:4*1024*1024}));
  // Every draw is retained before filtering and before either game.
  const generated=resolve(output,'generated-scenarios.json');
  save(generated,bank);
  execFileSync('python3',[scenarioProducer,'--validate-bank',generated],{cwd:ROOT,encoding:'utf8'});
  //Validate/retain the complete generated bank first. This explicit Source
  //mode selects declared route IDs only, before either capture, never outcomes.
  const selectionArgv=controlsOwnReload && flag('--case')===undefined
    ? [...process.argv,'--case',reloadIds.join(',')] : process.argv;
  bank.cases=selectCases(bank.cases,data=>data.scenario.id,selectionArgv);
  if (flag('--case',undefined,selectionArgv)) bank.caseFilter=flag('--case',undefined,selectionArgv);
  if (controlsOwnReload) assert.ok(bank.cases.every(data=>reloadIds.includes(data.scenario.id)),
    'Own Keyboard reload admits only its fixed/OS W-to-I route; no other remapping tape');
  const scenarios=resolve(output,'scenarios.json');
  save(scenarios,bank);
  execFileSync('python3',[scenarioProducer,'--validate-bank',scenarios],{cwd:ROOT,encoding:'utf8'});
  await assertOriginalBinary();
  const paths=bank.cases.map((data,index)=>{
    const path=resolve(output,`replay-${index}.json`);
    save(path,data);return path;
  });
  for (const [index,data] of bank.cases.entries()) {
    let report,error;
    try {
      assert.deepEqual(data.startup,{audioPreferences:{sfxVolume:1,musicVolume:1},fullscreenPreferences:{fullscreen:false},inputConfigs:{devices:[]}},
        'Every case independently declares the actual pre-constructor startup owners');
      const comparers=livingRestart?{compareControls:restartComparer(data)}:data.fixedBankCaseId ? comparersFor(data) : {compareControls,compareMovement};
      const requiredMovement=livingRestart?false:data.fixedBankCaseId ? Boolean(data.movementKind) : true;
      assert.equal(Boolean(comparers.compareMovement),requiredMovement,'Each declared route owns its exact movement aggregate');
      const disk=statfsSync(output);
      assert.ok(Number(disk.bavail)*Number(disk.bsize)>4*1024**3,'Fresh Native Controls keeps the existing4GiB reserve');
      const directory=resolve(output,`${String(index).padStart(2,'0')}-${data.scenario.id}`);
      mkdirSync(directory,{recursive:true});
      save(resolve(directory,'producer-identity.json'),{...producerIdentity,publicInputSha256:fileDigest(paths[index])});
      report=await runPairedGate({output:directory,selected:[],randomCount:0,replayCases:[paths[index]],
        productionDirectory:flag('--production-directory'),originalObserver:observer,
        originalSfxVolume:1,originalMusicVolume:1,publicClockSchedule:true,
        captureControlsOwners:true,controlsOwnReload,stateComparator:comparers.compareControls,movementComparator:comparers.compareMovement,
        archiveNativeEvidence:true,archiveBrowserEvidence:true,
        exactPixels:process.argv.includes('--require-exact-pixels'),nativeFrameTimeout:Number(flag('--native-frame-timeout','60'))});
      assert.equal(report.cases.length,1,'Every declared input case must execute');
      if (controlsOwnReload) {
        const own=report.cases[0].browser.keyboardOwnReload;
        assert.equal(own?.status,'pass','Actual own Keyboard reload is mandatory in the explicit mode');
        assert.ok(Number.isInteger(own.checks)&&own.checks>0&&Number.isInteger(own.checkpoints)&&own.checkpoints>0);
      }
      const aggregates=report.cases[0].checks?.filter(check=>check.label==='required-movement-and-old-key-refusal')??[];
      assert.equal(aggregates.length,Number(requiredMovement),'Required movement aggregate cannot disappear or be invented');
      if (requiredMovement)
        assert.ok(aggregates[0].state.length>0 && aggregates[0].state.every(row=>row.status==='pass'),
          'Actual independent movement/coasting aggregate must pass');
    } catch (caught) { error=String(caught.stack??caught); }
    results.push({id:data.scenario.id,status:!error && report?.status==='pass'?'pass':'fail',error,
      captureProducerSha256:producerIdentity.captureProducerSha256,
      report:report?resolve(output,`${String(index).padStart(2,'0')}-${data.scenario.id}/report.json`):undefined});
    save(resolve(output,`result-${index}.json`),results.at(-1));
  }
} catch (error) { failure=String(error.stack??error); }
finally {
  if (before) try {
    const after=sourceIdentity();save(resolve(output,'source-after.json'),after);
    assert.deepEqual(after,before,'Executing Controls Source changed during the suite');
  } catch (error) { failure=[failure,String(error.stack??error)].filter(Boolean).join('\n'); }
}
const result={schema:1,...(livingRestart&&{livingRestart:true,scope:'Normal seeded Isaac public Controls/Back, menu R exclusion, hold60/release/hold61, actual pending request and next-dispatch/continuous-hold/reset from both phases plus paused control and first unfiltered OS seed/boot. General/dead/Challenge, unseeded request draw, full MT/PCM/RGBA and own-storage reload remain separate.'}),...(controlsOwnReload && {controlsOwnReload:true,fullBank:false,
  additionalScope:'Complete paired W-to-I input route plus actual Browser own-storage natural document load, reset-load and remapped gameplay. Native process reload, corrupt/hotplug/controller and Browser OS-process restart remain unqualified.'}),status:!failure && results.length===bank?.cases?.length && results.every(row=>row.status==='pass')?'pass':'fail',
  failure,producerIdentity,cases:results,
  scope:livingRestart
    ? 'Normal seeded Isaac public Controls/Back and typed seed; three fixed physical hold/release/phase/pause controls plus first unfiltered raw OS seed/boot case. Actual canonical Native identity, input, queue, hold/Persistent owners, source clock and Browser input/errors remain required. No general/dead/Challenge, unseeded request draw, full MT/PCM/RGBA or natural own-storage reload claim.'
    :controlsOwnReload
    ? 'Filtered fixed/first-OS W-to-I paired routes plus mandatory real Browser own Keyboard save, natural document constructor load, separate reset-load and public remapped gameplay. All inherited Native/Browser clock/state/physical/movement guards remain required. No Native process reload, corrupt/hotplug/controller, Browser OS-process restart, fullRGBA or PCM claim.'
    : 'Four fixed routes plus recorded unfiltered OS Keyboard W-to-I: independent actual Menu9/10/24x2/physical-action/file/storage owners and required movement/coasting where declared; corrected duplicate SFX187 Menu-clock protocol. Canonical Native provenance/finalseed/clock and complete actual Browser input/error gates remain required. Executing producer Source is archived before draws/games and unchanged at completion. Every failure/partial is retained and later declared cases still execute. Full RGBA/history-cap diagnostics retained; no reload, alternatephysical overlap/controller/Skipped/PCM or wholeControls claim.'};
save(resolve(output,'report.json'),result);
console.log(JSON.stringify(result));
process.exitCode=result.status==='pass'?0:1;
