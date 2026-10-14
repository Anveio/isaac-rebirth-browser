// Fresh originals and immutable production through the retained public boss route.
import assert from 'node:assert/strict';
import {execFileSync,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdirSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {runPairedGate} from '../foundation/paired-game.mjs';
const option=(name,fallback)=>{const at=process.argv.indexOf(name);if(at<0)return fallback;assert.ok(process.argv[at+1]);return process.argv[at+1];};
const output=resolve(option('--output','.cache/e2e/monstro'));mkdirSync(output,{recursive:true});
const save=(path,data)=>writeFileSync(path,JSON.stringify(data,null,2)+'\n');
const sources=['tools/e2e/monstro.mjs','tools/original/monstro-scenarios.py','tools/foundation/monstro-state-pair.mjs','tools/original/foundation-combat-observe.py','engine/src/level2_flies.rs','engine/src/grid.rs','engine/src/familiars.rs','engine/src/npcs.rs','engine/src/monstro.rs','engine/src/actors.rs','engine/src/room.rs','engine/src/lib.rs','web/npc-data.json','web/familiar-data.json','references/isaac/foundation/combat/monstro/public-route-plan.json'];
const hashes=()=>Object.fromEntries(sources.map(path=>[path,createHash('sha256').update(readFileSync(path)).digest('hex')]));
const before=hashes();
const scenarios=JSON.parse(execFileSync('python3',['tools/original/monstro-scenarios.py'],{encoding:'utf8'}));
const selected=option('--case');if(selected){scenarios.cases=scenarios.cases.filter(c=>c.scenario.id===selected);assert.equal(scenarios.cases.length,1);}
save(resolve(output,'scenarios.json'),scenarios);
const tapes=scenarios.cases.map((c,index)=>{const path=resolve(output,`replay-${index}.json`);save(path,c);return path;});
const paired=await runPairedGate({output:resolve(output,'paired'),selected:[],randomCount:0,replayCases:tapes,productionDirectory:option('--production-directory'),nativeFrameTimeout:Number(option('--native-frame-timeout','120'))});
const cases=scenarios.cases.map((c,index)=>{
  const directory=resolve(output,'paired',`${String(index).padStart(2,'0')}-${c.scenario.id}`),path=resolve(directory,'monstro-state.json');
  rmSync(path,{force:true});
  const child=spawnSync('node',['tools/foundation/monstro-state-pair.mjs','--case',directory,'--output',path],{encoding:'utf8'});
  writeFileSync(resolve(directory,'monstro-state-output.txt'),(child.stdout??'')+(child.stderr??''));
  let direct;try{direct=JSON.parse(readFileSync(path,'utf8'));}catch{direct={status:'fail',error:child.stderr||'Missing required direct comparator report'};}
  return {id:c.scenario.id,status:child.status===0&&!child.signal&&direct.status==='pass'?'pass':'fail',comparatorExitCode:child.status,comparatorSignal:child.signal,
    fields:direct.checks?.length??0,failures:direct.checks?.filter(row=>row.status==='fail').length??0,execution:direct.execution,error:direct.error||child.error?.message,report:path};
});
assert.deepEqual(hashes(),before,'Runtime/resources/comparison must stay frozen across both-game captures');
const report={schema:1,status:paired.status==='pass'&&cases.every(c=>c.status==='pass')?'pass':'fail',pairedStatus:paired.status,sourcesStable:true,sources:before,cases,
  scope:'Required public214 admission/appearance/feedback/damage/death/clear and real Monstro admission/scene cleanup/AI/projectiles. Every actual paired field mismatch stays blocking. Complete boss blood/reward/trapdoor/stage exit and causal loader/shared-stream alignment remain explicit unaccepted follow-ups.'};
save(resolve(output,'report.json'),report);console.log(JSON.stringify(report));process.exitCode=report.status==='pass'?0:1;
