// Registered Map inputs: fixed regression plus unfiltered OS seed/boot cases.
// Each case delegates to the reviewed public-clock pair; no observer is an input.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync, spawnSync} from 'node:child_process';
import {flag, selectCases} from './lib/cli.mjs';

const ROOT=path.resolve(fileURLToPath(new URL('../..',import.meta.url)));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const digest=name=>hash(fs.readFileSync(name));
const read=name=>JSON.parse(fs.readFileSync(name,'utf8'));
const save=(name,value)=>fs.writeFileSync(name,JSON.stringify(value,null,2)+'\n',{flag:'wx'});
for(const name of ['--output','--production-directory','--scenarios','--count'])
  assert.ok(flag(name)===undefined||flag(name).length>0,`${name} requires a nonempty value`);
assert.equal(flag('--reuse-original'),undefined,'Registered Map requires fresh Native for every declared case');
const output=path.resolve(flag('--output',`.cache/e2e/options-map/${Date.now()}-${process.pid}`));
assert.ok(!fs.existsSync(output),'Use a fresh Map evidence directory');fs.mkdirSync(output,{recursive:true});
const gammaMode=process.argv.includes('--gamma');
const generator=gammaMode?'tools/original/options-gamma-scenarios.py':'tools/original/options-map-scenarios.py',pair='tools/e2e/options-map.mjs';
const preferencesSchema=flag('--preferences-schema','4');
assert.ok(preferencesSchema==='3'||preferencesSchema==='4','Map preferences schema must be explicitly3 or4');
assert.ok(!gammaMode||preferencesSchema==='4','Gamma requires explicit PreferencesV4');
const contractName=gammaMode?'references/isaac/cases/options-gamma/gamma-clamps-back-surface-contract-v1.json':preferencesSchema==='3'
  ?'references/isaac/cases/options-map/fast31-render-persistence-contract-v2.json'
  :'references/isaac/cases/options-map/fast31-render-persistence-contract-v4.json';
const sourcePaths=new Set([
  'tools/foundation/options-gamma-state-pair.mjs','tools/e2e/options-gamma-browser.mjs',
  'tools/original/options-map-scenarios.py',
  ...(gammaMode?['tools/original/options-gamma-scenarios.py','tools/original/options-gamma-observe.py',
    'references/isaac/cases/options-gamma/gamma-clamps-back-surface-v1.json']:[]),
  'tools/e2e/options-map-suite.mjs',pair,'tools/e2e/options-map-browser.mjs',generator,
  'tools/foundation/options-map-state-pair.mjs','tools/original/options-map-observe.py',
  'references/isaac/cases/options-map/map-adjust-back-surface-fast31-v2.json',contractName,
  'tools/e2e/lib/cli.mjs','tools/e2e/lib/original.mjs','tools/e2e/lib/game.mjs',
  'tools/e2e/lib/audio-owners.mjs','tools/e2e/audio-api-observer.mjs',
  'tools/foundation/controls-browser-observe.mjs','tools/foundation/paired-game.mjs',
  'tools/foundation/save-continue-browser.mjs','tools/foundation/save-continue-guards.mjs',
  'tools/foundation/save-continue-movement.mjs',
  'tools/foundation/production-server.mjs','tools/foundation/compare.mjs','tools/foundation/browser.mjs',
  'tools/foundation/pixel-pairs.mjs','tools/foundation/native-evidence.mjs',
  'tools/foundation/browser-evidence.mjs','tools/foundation/native-journal-zstd.mjs','tools/foundation/mt-draws.mjs',
  'tools/original/native-identity.py','tools/original/state-observer.py',
  'tools/original/options-owner-reader.py','tools/original/audio-observer.py','tools/original/audio-trace.c',
  'tools/original/foundation-paired-observer.py','tools/original/foundation-menu-observer.py',
  'tools/original/foundation-combat-observe.py','tools/original/foundation-clock-gate.c',
  'tools/original/foundation-clock-run.py','tools/original/foundation-clock-verify.py',
  'tools/original/native-evidence.py','tools/original/native-journal-zstd.py',
  'tools/original/foundation-paired-scenarios.py','tools/original/foundation-clock-scenarios.py',
  'tools/original/foundation-bomb-scenario.py','tools/original/foundation-presentation-scenario.py',
  'tools/original/foundation-title-scenario.py','tools/original/foundation-combat-scenario.py',
  'tools/original/foundation-random-seeds.py','tools/original/foundation-seeds.py',
  'tools/original/debug-run.py','tools/original/debug-config.json','tools/original/with-display.sh',
  'package.json','package-lock.json','engine/Cargo.toml','engine/Cargo.lock',
]);
// Pin the actual runtime Source and generated local JSON used by this artifact;
// unrelated Root docs, Git HEAD and index are not runtime acceptance guards.
const runtimeFiles=directory=>{for(const entry of fs.readdirSync(directory,{withFileTypes:true})) {
  const name=path.join(directory,entry.name);
  if(entry.isDirectory())runtimeFiles(name);
  else if(entry.isFile()&&/\.(?:rs|ts|json)$/.test(name))sourcePaths.add(path.relative(ROOT,name));
}};
runtimeFiles(path.join(ROOT,'engine/src'));runtimeFiles(path.join(ROOT,'web'));
const identity=()=>Object.fromEntries([...sourcePaths].sort().map(name=>[name,digest(path.join(ROOT,name))]));
const inventory=directory=>{
  const rows=[],tree=createHash('sha256');
  const visit=current=>{for(const entry of fs.readdirSync(current,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))) {
    const name=path.join(current,entry.name);assert.ok(!entry.isSymbolicLink(),'Production input cannot depend on a symlink');
    if(entry.isDirectory())visit(name);else {
      assert.ok(entry.isFile());const relative=path.relative(directory,name).split(path.sep).join('/'),bytes=fs.readFileSync(name);
      rows.push({path:relative,bytes:bytes.length,sha256:hash(bytes)});tree.update(relative).update('\0').update(String(bytes.length)).update('\0').update(bytes);
    }
  }};visit(directory);return{sha256:tree.digest('hex'),files:rows.length,inventory:rows};
};
let before,artifactBefore,artifactDirectory,bank,failure;
const cases=[];
try {
  before=identity();save(path.join(output,'source-before.json'),before);
  assert.equal(before['tools/e2e/options-map-suite.mjs'],digest(fileURLToPath(import.meta.url)),'Archive the actual executing driver');
  for(const name of Object.keys(before)) {
    const target=path.join(output,'executing-source',name);fs.mkdirSync(path.dirname(target),{recursive:true});
    fs.copyFileSync(path.join(ROOT,name),target);assert.equal(digest(target),before[name]);
  }
  // Source is archived before the generator can make its first logical draw.
  // The replay branch invokes only validation; it never enters fresh generation.
  const recorded=flag('--scenarios'),args=recorded===undefined?['--count',flag('--count','1')]:['--scenarios',path.resolve(recorded)];
  const generatorLog=fs.openSync(path.join(output,'generator-actual.log'),'wx');let generated;
  try {
    generated=spawnSync('python3',[generator,...args],{cwd:ROOT,encoding:'utf8',
      maxBuffer:4*1024*1024,stdio:['ignore','pipe',generatorLog]});
  }finally {fs.closeSync(generatorLog);}
  fs.writeFileSync(path.join(output,'generator-stdout.json'),generated.stdout??'',{flag:'wx'});
  assert.ok(!generated.error&&generated.status===0,`Map generator failed; raw stdout/stderr retained: ${generated.error??generated.status}`);
  bank=JSON.parse(generated.stdout);
  const generatedBank=path.join(output,'generated-scenarios.json');save(generatedBank,bank);
  execFileSync('python3',[generator,'--validate-bank',generatedBank],{cwd:ROOT,encoding:'utf8'});
  bank.cases=selectCases(bank.cases,data=>data.scenario.id);
  if(flag('--case')!==undefined)bank.caseFilter=flag('--case');
  const scenarios=path.join(output,'scenarios.json');save(scenarios,bank);
  execFileSync('python3',[generator,'--validate-bank',scenarios],{cwd:ROOT,encoding:'utf8'});
  artifactDirectory=path.resolve(flag('--production-directory',path.join(ROOT,'dist')));
  artifactBefore=inventory(artifactDirectory);save(path.join(output,'artifact-before.json'),artifactBefore);
  const contractFile=path.join(output,'map-contract.json');fs.copyFileSync(path.join(ROOT,contractName),contractFile);
  assert.equal(digest(contractFile),before[contractName]);
  for(const [index,data] of bank.cases.entries()) {
    const prefix=String(index).padStart(2,'0')+'-'+data.scenario.id;
    const caseFile=path.join(output,prefix+'-case.json');save(caseFile,data);
    const binding={schema:1,mode:gammaMode?'reviewed-options-gamma-v1':'reviewed-options-map-fast31-v1',sourceRoot:ROOT,
      sourceFiles:before,caseFile,caseSha256:digest(caseFile),contractFile,contractSha256:digest(contractFile),
      artifact:{directory:artifactDirectory,...artifactBefore},
      nativeEnvironment:{ISAAC_CLOCK_GATE_RENDER_ALL:'1',ISAAC_CLOCK_GATE_WINDOW_QUERIES:'1'}};
    const bindingFile=path.join(output,prefix+'-binding.json');save(bindingFile,binding);
    const directory=path.join(output,prefix),log=path.join(output,prefix+'-actual.log');
    const descriptor=fs.openSync(log,'wx');let result;
    try {
      result=spawnSync(process.execPath,[pair,'--binding',bindingFile,'--binding-sha256',digest(bindingFile),'--output',directory],
        {cwd:ROOT,env:{...process.env,...binding.nativeEnvironment},stdio:['ignore',descriptor,descriptor]});
    }finally {fs.closeSync(descriptor);}
    let actual,error;
    try {actual=read(path.join(directory,'report.json'));}catch(caught){error=String(caught.stack??caught);}
    const row={id:data.scenario.id,status:!result.error&&result.status===0&&actual?.status==='pass'?'pass':'fail',
      exitCode:result.status,signal:result.signal,error:error??(result.error?String(result.error):undefined),
      report:path.join(directory,'report.json'),actualLog:log,bindingSha256:digest(bindingFile)};
    cases.push(row);save(path.join(output,`result-${index}.json`),row);
    // Failed or partial cases remain retained; every later declared case runs.
  }
}catch(error){failure=String(error.stack??error);}
finally {
  if(before)try {
    const after=identity();save(path.join(output,'source-after.json'),after);assert.deepEqual(after,before,'Executing Map Source changed');
    if(artifactBefore)assert.deepEqual(inventory(artifactDirectory),artifactBefore,'Production input changed');
  }catch(error){failure=[failure,String(error.stack??error)].filter(Boolean).join('\n');}
}
const report={schema:1,status:!failure&&bank?.cases?.length===cases.length&&cases.length>0&&cases.every(row=>row.status==='pass')?'pass':'fail',
  failure,cases,sourceBefore:before,artifact:artifactBefore,
  scope:gammaMode?'Gamma21/20 regression plus default one fresh unfiltered OS seed/boot route; complete recorded-bank replay makes no entropy draws. Each pair requires public clock/input/provenance, actual Gamma/config/number/final GPU owners, V4 Back IO, separate actual Browser reload and strict required number regions. Historical V2/V3 evidence is unchanged; full-frame RGBA, Native reload, HQ4X, Popups and completeMT remain separate.':'Fixed31/30 regression plus default one fresh unfiltered OS seed/boot Map route; complete recorded-bank replay makes no entropy draws. Each pair requires public clock/input/provenance, actual Map owners, Back IO, configured237 and strict required renderer regions. Old84/V2 evidence is unchanged. Full-frame RGBA remains diagnostic; reload/migration/banner/controller/fullmix/completeMT remain separate.'};
save(path.join(output,'report.json'),report);
console.log(JSON.stringify({status:report.status,report:path.join(output,'report.json')}));process.exitCode=report.status==='pass'?0:1;
