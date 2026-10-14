// Original and immutable production browser, through identical public inputs.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync,mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {gunzipSync} from 'node:zlib';
import {runPairedGate,scopedState} from '../foundation/paired-game.mjs';
const option=(name,fallback)=>{const i=process.argv.indexOf(name);return i<0?fallback:process.argv[i+1];};
const output=resolve(option('--output',`.cache/e2e/player-death/${Date.now()}-${process.pid}`));mkdirSync(output,{recursive:true});
const save=(p,d)=>writeFileSync(p,JSON.stringify(d,null,2)+'\n');
const manifest=p=>JSON.parse((existsSync(p)?readFileSync(p):gunzipSync(readFileSync(p+'.gz'))).toString('utf8'));
const sources=['tools/e2e/player-death.mjs','tools/original/player-death-observe.py','tools/original/player-death-scenarios.py','tools/original/audio-observer.py','references/isaac/foundation/player-death/source-contract.json'];
const hashes=()=>Object.fromEntries(sources.map(p=>[p,createHash('sha256').update(readFileSync(p)).digest('hex')]));
const before=hashes();
const scenarioFile=option('--scenario-file',undefined);
const scenarioInput=scenarioFile?readFileSync(resolve(scenarioFile)):undefined;
const scenarios=JSON.parse(scenarioInput||execFileSync('python3',['tools/original/player-death-scenarios.py','--count',option('--count','2')],{encoding:'utf8',maxBuffer:8*1024*1024}));
assert.ok(Array.isArray(scenarios.cases)&&scenarios.cases.length>=4,'Death gate requires both contrived routes and at least two recorded random cases');
assert.ok(scenarios.cases.some(c=>c.scenario.id==='death-spikes-restart')&&scenarios.cases.some(c=>c.scenario.id==='death-spikes-exit'),'Both mandatory public popup actions must remain in the cohort');
save(resolve(output,'scenarios.json'),scenarios);
const paths=scenarios.cases.map((c,n)=>{const p=resolve(output,`replay-${n}.json`);save(p,c);return p;});
function compare(source,actual,gameplay){
 assert.equal(source?.death?.status,'pass','Every checkpoint needs an actual bounded death/System owner');
 const d=source.death;
 const checks=scopedState(source,actual,gameplay&&d.systemState===2);
 const check=(field,expected,value)=>checks.push({field,expected,actual:value,status:JSON.stringify(expected)===JSON.stringify(value)?'pass':'fail'});
 check('popup.state',d.popupState,actual.playerDeath.gameOver?.state||0);
 if(d.systemState===2){
  check('player.dead',d.dead,actual.player.dead);check('player.visible',d.visible,actual.player.visible);
  check('player.controls7d4',d.controlsEnabled,actual.player.controlsEnabled);
  check('Game.fullUpdates',d.fullGameUpdates,actual.gameUpdateCount);
  if(d.mainAnimation.name==='Death'){
   check('Death.current','Death',actual.playerAnimation.current);check('Death.frame',d.mainAnimation.frame,actual.playerAnimation.frame);
   check('Death.playing',d.mainAnimation.playing,actual.playerAnimation.playing);check('Death.guard',d.animationGuard,actual.playerAnimation.guard);
  }
  if(d.popupState===2){check('Diary.frame',d.deathScreen.frame,actual.playerDeath.gameOver?.diaryFrame);check('Diary.playing',d.deathScreen.playing,actual.playerDeath.gameOver?.diaryPlaying);}
 }
 assert.equal(source.deathSound?.status,'pass','SFX30 needs actual original wrapper observation');
 const slot=source.deathSound.slot30,port=actual.audio.slots['30'];
 if(slot?.loaded||port?.loaded){
  check('SFX30.loaded',slot?.loaded,port?.loaded);
  // Both source lastVolume and the owned port field are binary32. Rust's
  // shortest f32 JSON text and Python's widened memory read serialize it
  // differently; compare the exact original field domain, with no tolerance.
  const values=[slot?.lastVolume,port?.lastVolume];
  assert.ok(values.every(v=>typeof v==='number'&&Number.isFinite(v)),'SFX30 requires two actual finite numeric lastVolume values');
  check('SFX30.lastVolume',Math.fround(values[0]),Math.fround(values[1]));
 }
 return checks;
}
const report=await runPairedGate({output:resolve(output,'paired'),selected:[],randomCount:0,replayCases:paths,
 originalObserver:'tools/original/player-death-observe.py',stateComparator:compare,additionalSources:sources,archiveBrowserEvidence:true});
const coverage=scenarios.cases.map((c,n)=>{
 try {
 const native=manifest(resolve(report.cases[n].directory,'original/manifest.json'));
 const settled=native.debugObservations.find(r=>r.context.label==='death-settled')?.observation.death;
 const candidate=manifest(resolve(report.cases[n].directory,'browser/manifest.json')).frames.find(r=>r.label==='death-settled')?.state;
 const complete=settled?.dead===true&&settled.popupState===2&&candidate?.player.dead===true&&candidate.playerDeath.gameOver?.state===2;
 return {id:c.scenario.id,required:c.deathRequired,actualSource:settled,executedInBoth:complete,status:!c.deathRequired||complete?'pass':'fail'};
 } catch (error) {
  return {id:c.scenario.id,required:c.deathRequired,executedInBoth:false,status:'fail',error:String(error)};
 }
});
const stable=JSON.stringify(hashes())===JSON.stringify(before);
const result={schema:1,status:report.status==='pass'&&stable&&coverage.every(c=>c.status==='pass')?'pass':'fail',pairedStatus:report.status,sourcesStable:stable,sources:before,coverage,scope:'Normal Isaac death/57-frame authored clip, GameOver freeze and public deferred popup actions. Contrived death coverage and all source/player/menu/audio fields are blocking; full pixels remain diagnostics. Every unfiltered OS outcome and original/backend/input evidence retained; complete revival/diary/other-character acceptance pending.'};
if(scenarioInput)result.scenarioInput={path:resolve(scenarioFile),sha256:createHash('sha256').update(scenarioInput).digest('hex'),bytes:scenarioInput.length};
save(resolve(output,'report.json'),result);console.log(JSON.stringify(result));process.exitCode=result.status==='pass'?0:1;
