// Public living Quit/Continue and natural own-storage reload; no Native inputs.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash,randomBytes} from 'node:crypto';
import {copyFileSync,existsSync,mkdirSync,readFileSync,readdirSync,writeFileSync} from 'node:fs';
import {dirname,relative,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {runPairedGate} from '../foundation/paired-game.mjs';
import {saveOwnerState} from '../foundation/save-continue-state-pair.mjs';
import {OWN_PROFILE_SCHEMA} from '../foundation/save-continue-guards.mjs';
import {ROOT,flag,selectCases} from './lib/cli.mjs';
import {ORIGINAL_SHA,assertOriginalBinary} from './lib/original.mjs';

for (const name of ['--output','--production-directory','--scenarios','--reuse-original'])
  assert.ok(flag(name)===undefined || flag(name).length>0,`${name} requires a nonempty value`);
const output=resolve(flag('--output',`.cache/e2e/save-continue/${Date.now()}-${process.pid}`));
assert.ok(!existsSync(output),'Save qualification needs fresh evidence paths');
mkdirSync(output,{recursive:true});
const save=(path,value)=>writeFileSync(path,JSON.stringify(value,null,2)+'\n',{flag:'wx'});
const fileDigest=path=>createHash('sha256').update(readFileSync(path)).digest('hex');
const helper='tools/foundation/save-continue-cases.py';
const observer='tools/original/save-continue-observe.py';
const fixed='references/isaac/cases/save-continue/initial-room-footprints.json';
const stationary='references/isaac/cases/save-continue/initial-room-stationary.json';
const count=Number(flag('--count','1'));
assert.ok(Number.isInteger(count)&&count>=0&&count<=16,'OS count must be0..16');
const recorded=flag('--scenarios'),reuse=flag('--reuse-original');
// Archive executing Source before entropy or either game. This includes the
// unchanged canonical producer's dynamic Python/C and pixel-worker imports.
const tools=[];
const visit=directory=>{
  for(const entry of readdirSync(directory,{withFileTypes:true})){
    const path=resolve(directory,entry.name);
    if(entry.isDirectory()&&entry.name!=='__pycache__')visit(path);
    else if(entry.isFile()&&/\.(?:mjs|js|ts|py|c|h|sh|json)$/.test(entry.name))tools.push(relative(ROOT,path));
  }
};
visit(resolve(ROOT,'tools'));
const sourcePaths=[...new Set([...tools,'package-lock.json',fixed,stationary,
  'references/isaac/foundation/combat/tapes.json'])].sort();
const sourceIdentity=()=>Object.fromEntries(sourcePaths.map(path=>[path,fileDigest(resolve(ROOT,path))]));
let before,bank,fatal;
const results=[];
try{
  before=sourceIdentity();
  assert.equal(fileDigest(fileURLToPath(import.meta.url)),before['tools/e2e/save-continue.mjs'],'Actual executing driver must be archived');
  for(const path of sourcePaths){
    const destination=resolve(output,'executing-source',path);mkdirSync(dirname(destination),{recursive:true});
    copyFileSync(resolve(ROOT,path),destination);assert.equal(fileDigest(destination),before[path]);
  }
  save(resolve(output,'source-before.json'),before);
  save(resolve(output,'producer-identity.json'),{schema:1,mode:'save-continue-v1',ownProfileSchema:OWN_PROFILE_SCHEMA,binarySha256:ORIGINAL_SHA,
    captureProducerSha256:before['tools/foundation/paired-game.mjs'],observerSha256:before[observer],sources:before,
    publicPreferences:{sfxVolume:0,musicVolume:0,fullscreen:false,inputConfigs:{devices:[]}}});
  if(recorded!==undefined)bank=JSON.parse(readFileSync(resolve(recorded),'utf8'));
  else{
    bank={schema:1,randomCount:count,osDraws:[],cases:[JSON.parse(readFileSync(resolve(ROOT,fixed),'utf8'))]};
    for(let ordinal=0;ordinal<count;ordinal++){
      const bytes=randomBytes(4),draw={ordinal,u32:bytes.readUInt32LE(),bytesHex:bytes.toString('hex'),source:'OS randomBytes(4) before mapping'};
      bank.osDraws.push(draw);
      // Even reserved-zero/mapping failures retain the original draw. There
      // is no floor sampler, redraw, Native feedback or case replacement.
      save(resolve(output,`os-draw-${ordinal}.json`),draw);
      bank.cases.push(JSON.parse(execFileSync('python3',[helper,'--seed-value',String(draw.u32),'--ordinal',String(ordinal)],
        {cwd:ROOT,encoding:'utf8',maxBuffer:4*1024*1024})));
    }
  }
  const generated=resolve(output,'generated-scenarios.json');save(generated,bank);
  execFileSync('python3',[helper,'--validate-bank',generated],{cwd:ROOT,encoding:'utf8',maxBuffer:8*1024*1024});
  bank.cases=selectCases(bank.cases,data=>data.scenario.id);
  if(flag('--case'))bank.caseFilter=flag('--case');
  const scenarios=resolve(output,'scenarios.json');save(scenarios,bank);
  execFileSync('python3',[helper,'--validate-bank',scenarios],{cwd:ROOT,encoding:'utf8',maxBuffer:8*1024*1024});
  assert.ok(reuse===undefined||(recorded!==undefined&&bank.cases.length===1),'Native reuse requires exactly one recorded public case');
  await assertOriginalBinary();
  for(const [index,data] of bank.cases.entries()){
    const publicCase=resolve(output,`replay-${index}.json`);save(publicCase,data);
    const directory=resolve(output,`${String(index).padStart(2,'0')}-${data.scenario.id}`);
    let report,error;
    try{
      report=await runPairedGate({output:directory,selected:[],randomCount:0,replayCases:[publicCase],
        productionDirectory:flag('--production-directory'),reuseOriginalDirectory:reuse&&resolve(reuse),
        originalObserver:observer,stateComparator:saveOwnerState,originalSfxVolume:0,originalMusicVolume:0,
        archiveNativeEvidence:true,archiveBrowserEvidence:true,publicClockSchedule:true,saveContinue:true,
        exactPixels:process.argv.includes('--require-exact-pixels'),nativeFrameTimeout:Number(flag('--native-frame-timeout','120'))});
      assert.equal(report.cases.length,1,'Each declared case must execute');
    }catch(caught){error=String(caught.stack??caught);}
    results.push({id:data.scenario.id,status:!error&&report?.status==='pass'?'pass':'fail',error,
      publicInputSha256:fileDigest(publicCase),report:report?resolve(directory,'report.json'):undefined});
    save(resolve(output,`result-${index}.json`),results.at(-1));
  }
}catch(error){fatal=String(error.stack??error);}
finally{
  if(before)try{
    const after=sourceIdentity();save(resolve(output,'source-after.json'),after);
    assert.deepEqual(after,before,'Executing Save Source changed during capture');
  }catch(error){fatal=[fatal,String(error.stack??error)].filter(Boolean).join('\n');}
}
const report={schema:1,ownProfileSchema:OWN_PROFILE_SCHEMA,status:!fatal&&results.length===bank?.cases?.length&&results.every(row=>row.status==='pass')?'pass':'fail',
  fatal,cases:results,caseFilter:bank?.caseFilter,
  selectedScope:bank?.caseFilter?'Explicit recorded or CLI subset; no complete default-bank acceptance':'Complete declared fixed plus OS bank',
  scope:'Selected moving-before-Save public inputs and recorded unfiltered OS seeds; actual living Quit file/packet, menu readmission, same-process Continue, resumed controls, fresh Room paint and natural same-origin reload. Native own files remain observations only. Strict canonical/current-System-family rows, all real IO failures and complete transport/clock/physical guards remain required; strict RGBA retained as diagnostics. Initial-room Isaac/normal/neutral-items admission only; no arbitrary actors/items/travel, Native process-reload proof, full MT/audio/pixels or whole Save claim.'};
save(resolve(output,'report.json'),report);console.log(JSON.stringify(report));process.exitCode=report.status==='pass'?0:1;
