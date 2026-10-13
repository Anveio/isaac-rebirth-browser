// Fresh source and immutable production browser, including mandatory T1 cases.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { runPairedGate } from '../foundation/paired-game.mjs';

const option=(name,fallback)=>{const at=process.argv.indexOf(name);if(at<0)return fallback;assert.ok(process.argv[at+1]);return process.argv[at+1];};
const output=resolve(option('--output','.cache/e2e/roundworms'));mkdirSync(output,{recursive:true});
const save=(path,value)=>writeFileSync(path,JSON.stringify(value,null,2)+'\n');
const sources=['tools/e2e/roundworms.mjs','tools/original/roundworm-scenarios.py','tools/foundation/roundworm-state-pair.mjs','references/isaac/foundation/combat/roundworms/random-selection.json','references/isaac/foundation/combat/roundworms/hostile-return-tail.json'];
const hashes=()=>Object.fromEntries(sources.map(path=>[path,createHash('sha256').update(readFileSync(path)).digest('hex')]));
const before=hashes();
const scenarios=JSON.parse(execFileSync('python3',['tools/original/roundworm-scenarios.py'],{encoding:'utf8'}));
const selected=option('--case');if(selected){scenarios.cases=scenarios.cases.filter(item=>item.scenario.id===selected);assert.equal(scenarios.cases.length,1);}
save(resolve(output,'scenarios.json'),scenarios);
const tapes=scenarios.cases.map((item,index)=>{const path=resolve(output,`replay-${index}.json`);save(path,item);return path;});
const paired=await runPairedGate({output:resolve(output,'paired'),selected:[],randomCount:0,replayCases:tapes,productionDirectory:option('--production-directory'),nativeFrameTimeout:Number(option('--native-frame-timeout','120'))});
const cases=scenarios.cases.map((scenario,index)=>{
  const directory=resolve(output,'paired',`${String(index).padStart(2,'0')}-${scenario.scenario.id}`), target=resolve(directory,'worm-state.json');
  rmSync(target,{force:true});
  const process=spawnSync('node',['tools/foundation/roundworm-state-pair.mjs','--case',directory,'--output',target],{encoding:'utf8'});
  writeFileSync(resolve(directory,'worm-state-output.txt'),process.stdout+process.stderr);
  let report;try{report=JSON.parse(readFileSync(target,'utf8'));}catch{report={status:'fail',error:process.stderr||'No required comparator report'};}
  return {id:scenario.scenario.id,status:process.status===0&&!process.signal&&report.status==='pass'?'pass':'fail',comparatorExitCode:process.status,comparatorSignal:process.signal,
    fields:report.checks?.length||0,failures:report.checks?.filter(check=>check.status==='fail').length||0,
    diagnosticFields:report.diagnostics?.length||0,diagnosticFailures:report.diagnostics?.filter(check=>check.status==='fail').length||0,
    execution:report.execution,error:report.error||process.error?.message,report:target};
});
assert.deepEqual(hashes(),before,'Round Worm cohort/comparison sources must remain stable');
const report={schema:1,status:paired.status==='pass'&&cases.every(item=>item.status==='pass')?'pass':'fail',pairedStatus:paired.status,sourcesStable:true,sources:before,cases,
  scope:'All real actor sets, appearance/AI fields, projectile defaults/full vertical law, HP/immunity and mandatory T1 locks/return remain required. Exact movement/MT seeds remain diagnostic; retained FlyH2/center regressions are unchanged separate mandatory gates.'};
save(resolve(output,'report.json'),report);console.log(JSON.stringify(report));process.exitCode=report.status==='pass'?0:1;
