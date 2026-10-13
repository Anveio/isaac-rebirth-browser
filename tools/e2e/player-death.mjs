// Original and immutable production browser, through identical public inputs.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {runPairedGate,scopedState} from '../foundation/paired-game.mjs';
const option=(name,fallback)=>{const i=process.argv.indexOf(name);return i<0?fallback:process.argv[i+1];};
const output=resolve(option('--output',`.cache/e2e/player-death/${Date.now()}-${process.pid}`));mkdirSync(output,{recursive:true});
const save=(p,d)=>writeFileSync(p,JSON.stringify(d,null,2)+'\n');
const sources=['tools/e2e/player-death.mjs','tools/original/player-death-observe.py','tools/original/player-death-scenarios.py','tools/original/audio-observer.py','references/isaac/foundation/player-death/source-contract.json'];
const hashes=()=>Object.fromEntries(sources.map(p=>[p,createHash('sha256').update(readFileSync(p)).digest('hex')]));
const before=hashes();
const scenarios=JSON.parse(execFileSync('python3',['tools/original/player-death-scenarios.py','--count',option('--count','2')],{encoding:'utf8',maxBuffer:8*1024*1024}));
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
 if(slot?.loaded||port?.loaded){check('SFX30.loaded',slot?.loaded,port?.loaded);check('SFX30.lastVolume',slot?.lastVolume,port?.lastVolume);}
 return checks;
}
const report=await runPairedGate({output:resolve(output,'paired'),selected:[],randomCount:0,replayCases:paths,
 originalObserver:'tools/original/player-death-observe.py',stateComparator:compare,additionalSources:sources});
const coverage=scenarios.cases.map((c,n)=>{
 const native=JSON.parse(readFileSync(resolve(report.cases[n].directory,'original/manifest.json'),'utf8'));
 const settled=native.debugObservations.find(r=>r.context.label==='death-settled')?.observation.death;
 const candidate=JSON.parse(readFileSync(resolve(report.cases[n].directory,'browser/manifest.json'),'utf8')).frames.find(r=>r.label==='death-settled')?.state;
 const complete=settled?.dead===true&&settled.popupState===2&&candidate?.player.dead===true&&candidate.playerDeath.gameOver?.state===2;
 return {id:c.scenario.id,required:c.deathRequired,actualSource:settled,executedInBoth:complete,status:!c.deathRequired||complete?'pass':'fail'};
});
const stable=JSON.stringify(hashes())===JSON.stringify(before);
const result={schema:1,status:report.status==='pass'&&stable&&coverage.every(c=>c.status==='pass')?'pass':'fail',pairedStatus:report.status,sourcesStable:stable,sources:before,coverage,scope:'Normal Isaac death/57-frame authored clip, GameOver freeze and public deferred popup actions. Contrived death coverage and all source/player/menu/audio fields are blocking; full pixels remain diagnostics. Every unfiltered OS outcome and original/backend/input evidence retained; complete revival/diary/other-character acceptance pending.'};
save(resolve(output,'report.json'),result);console.log(JSON.stringify(result));process.exitCode=result.status==='pass'?0:1;
