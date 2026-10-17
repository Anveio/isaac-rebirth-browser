// Original and immutable production browser, through identical public inputs.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync,mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {gunzipSync} from 'node:zlib';
import {runPairedGate,scopedState} from '../foundation/paired-game.mjs';
import {flag,selectCases} from './lib/cli.mjs';
const option=(name,fallback)=>{const i=process.argv.indexOf(name);return i<0?fallback:process.argv[i+1];};
const output=resolve(option('--output',`.cache/e2e/player-death/${Date.now()}-${process.pid}`));mkdirSync(output,{recursive:true});
const save=(p,d)=>writeFileSync(p,JSON.stringify(d,null,2)+'\n');
const manifest=p=>JSON.parse((existsSync(p)?readFileSync(p):gunzipSync(readFileSync(p+'.gz'))).toString('utf8'));
const scenarioFile=option('--scenario-file',undefined);
const scenarioInput=scenarioFile?readFileSync(resolve(scenarioFile)):undefined;
const scenarios=JSON.parse(scenarioInput||execFileSync('python3',['tools/original/player-death-scenarios.py','--count',option('--count','2')],{encoding:'utf8',maxBuffer:8*1024*1024}));
// A bank recorded under --case holds only the cases that ran; each one it holds
// still meets its mandatory-death and OS-row checks.
const whole=!scenarios.caseFilter;
assert.ok(Array.isArray(scenarios.cases)&&(!whole||scenarios.cases.length>=4),'Death gate requires both contrived routes and at least two recorded random cases');
const requiredIds=new Set(['death-spikes-restart','death-spikes-exit']);
const ids=scenarios.cases.map(c=>c.scenario?.id);
assert.ok(ids.every(id=>typeof id==='string')&&new Set(ids).size===ids.length,'Every declared death case requires a unique ID');
for(const id of requiredIds){const found=scenarios.cases.find(c=>c.scenario.id===id);assert.ok(found?found.deathRequired===true:!whole,'Both mandatory public popup actions require actual death regardless of supplied bank metadata');}
const randomCases=scenarios.cases.filter(c=>/^death-unfiltered-\d+$/.test(c.scenario.id));
assert.ok((!whole||randomCases.length>=2)&&randomCases.every(c=>
 c.entropy?.source==='OS secrets; unfiltered nonzero32-bit public seed'&&
 c.entropy.draw?.seedValue===c.seedValue),'At least two complete recorded unfiltered OS rows must remain in the bank');
scenarios.cases=selectCases(scenarios.cases,c=>c.scenario.id);if(flag('--case'))scenarios.caseFilter=flag('--case');
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
 productionDirectory:option('--production-directory'),
 originalObserver:'tools/original/player-death-observe.py',stateComparator:compare,archiveBrowserEvidence:true,publicClockSchedule:true});
const coverage=scenarios.cases.map((c,n)=>{
 try {
 const native=manifest(resolve(report.cases[n].directory,'original/manifest.json'));
 const settled=native.debugObservations.find(r=>r.context.label==='death-settled')?.observation.death;
 const candidate=manifest(resolve(report.cases[n].directory,'browser/manifest.json')).frames.find(r=>r.label==='death-settled')?.state;
 const complete=settled?.dead===true&&settled.popupState===2&&candidate?.player.dead===true&&candidate.playerDeath.gameOver?.state===2;
 const required=requiredIds.has(c.scenario.id)||c.deathRequired;
 return {id:c.scenario.id,required,actualSource:settled,executedInBoth:complete,status:!required||complete?'pass':'fail'};
 } catch (error) {
  return {id:c.scenario.id,required:requiredIds.has(c.scenario.id)||c.deathRequired,executedInBoth:false,status:'fail',error:String(error)};
 }
});
const result={schema:1,status:report.status==='pass'&&coverage.every(c=>c.status==='pass')?'pass':'fail',pairedStatus:report.status,caseFilter:scenarios.caseFilter,coverage,scope:'Independently declared public wall-clock inputs on both games. Normal Isaac death/57-frame authored clip, GameOver freeze and public deferred popup actions. Contrived death coverage and all source/player/menu/audio fields are blocking; full pixels remain diagnostics. Every unfiltered OS outcome and original/backend/input evidence retained; complete revival/diary/other-character acceptance pending.'};
if(scenarioInput)result.scenarioInput={path:resolve(scenarioFile),sha256:createHash('sha256').update(scenarioInput).digest('hex'),bytes:scenarioInput.length};
save(resolve(output,'report.json'),result);console.log(JSON.stringify(result));process.exitCode=result.status==='pass'?0:1;
