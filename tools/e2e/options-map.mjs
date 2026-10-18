// Bounded first Map pair. No generator, retries, entropy or fixture rewriting.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {flag, writeJson} from './lib/cli.mjs';
import {ORIGINAL_SHA, assertOriginalBinary} from './lib/original.mjs';
import {nativeCapture,browserCapture,declarePublicClockPlan,validateNativeClockPlan,scopedState} from '../foundation/paired-game.mjs';
import {startProductionServer} from '../foundation/production-server.mjs';
import {browserLaunchOptions,inspectBrowserGraphics} from '../foundation/browser.mjs';
import {pixelDifference,crop} from '../foundation/compare.mjs';
import {validateMapDeclaration,compareOptionsMap} from '../foundation/options-map-state-pair.mjs';
import {mapObservedBrowser} from './options-map-browser.mjs';
import {gammaObservedBrowser} from './options-gamma-browser.mjs';
import {validateGammaDeclaration,compareOptionsGamma} from '../foundation/options-gamma-state-pair.mjs';

const ROOT=path.resolve(fileURLToPath(new URL('../..',import.meta.url)));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const digest=name=>hash(fs.readFileSync(name));
const read=name=>JSON.parse(fs.readFileSync(name,'utf8'));
for(const name of ['--binding','--binding-sha256','--output'])assert.ok(flag(name)?.length,`${name} must be explicit and nonempty`);
const bindingPath=path.resolve(flag('--binding')),bindingHash=flag('--binding-sha256');
assert.equal(digest(bindingPath),bindingHash,'Exact separately reviewed launch binding required');
const binding=read(bindingPath);
assert.equal(binding.schema,1);assert.ok(['reviewed-options-map-fast31-v1','reviewed-options-gamma-v1'].includes(binding.mode));
const gammaMode=binding.mode==='reviewed-options-gamma-v1';
assert.equal(path.resolve(binding.sourceRoot),ROOT,'Execute only the independently frozen Source named by the binding');
for(const name of ['ISAAC_CLOCK_GATE_RENDER_ALL','ISAAC_CLOCK_GATE_WINDOW_QUERIES'])
  assert.equal(process.env[name],binding.nativeEnvironment[name],`Declared Native host mode ${name} required`);
assert.deepEqual(binding.nativeEnvironment,{ISAAC_CLOCK_GATE_RENDER_ALL:'1',ISAAC_CLOCK_GATE_WINDOW_QUERIES:'1'});
const data=read(binding.caseFile),contract=read(binding.contractFile);
assert.equal(digest(binding.caseFile),binding.caseSha256);assert.equal(digest(binding.contractFile),binding.contractSha256);
(gammaMode?validateGammaDeclaration:validateMapDeclaration)(data,contract);
const output=path.resolve(flag('--output'));assert.ok(!fs.existsSync(output),'Use a fresh immutable output directory');
const identity=()=>Object.fromEntries(Object.keys(binding.sourceFiles).sort().map(name=>{
  const p=path.resolve(ROOT,name);assert.ok(p.startsWith(ROOT+path.sep)&&fs.statSync(p).isFile(),'Source pin outside frozen Source');
  return[name,digest(p)];
}));
const before=identity();assert.deepEqual(before,binding.sourceFiles,'Complete reviewed frozen Source inventory required');
const artifactInventory=()=>{
  const rows=[];
  const visit=directory=>{for(const entry of fs.readdirSync(directory,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))) {
    const p=path.join(directory,entry.name);assert.ok(!entry.isSymbolicLink(),'Frozen artifact cannot depend on a symlink');
    if(entry.isDirectory())visit(p);else {assert.ok(entry.isFile());rows.push({path:path.relative(binding.artifact.directory,p).split(path.sep).join('/'),bytes:fs.statSync(p).size,sha256:digest(p)});}
  }};
  visit(binding.artifact.directory);return rows;
};
const artifactBefore=artifactInventory();assert.deepEqual(artifactBefore,binding.artifact.inventory,'Full reviewed artifact inventory required');
const own=path.relative(ROOT,fileURLToPath(import.meta.url));assert.equal(before[own],digest(fileURLToPath(import.meta.url)),'Executing driver must be in frozen Source pins');
for(const name of ['tools/e2e/options-map-browser.mjs','tools/foundation/options-map-state-pair.mjs','tools/original/options-map-observe.py',
  'tools/foundation/paired-game.mjs','tools/e2e/lib/game.mjs','tools/original/foundation-clock-gate.c','tools/original/foundation-clock-run.py',
  'tools/original/options-owner-reader.py','tools/original/audio-observer.py','tools/original/audio-trace.c'])assert.ok(before[name],`Missing executing Source pin ${name}`);
fs.mkdirSync(output,{recursive:true});
writeJson(path.join(output,'launch-binding.json'),binding);writeJson(path.join(output,'source-before.json'),before);
for(const name of Object.keys(before).filter(name=>/^(tools\/.*\.(?:mjs|py|c)|engine\/src\/.*\.rs|web\/.*\.ts)$/.test(name))) {
  const target=path.join(output,'executing-source',name);fs.mkdirSync(path.dirname(target),{recursive:true});fs.copyFileSync(path.join(ROOT,name),target);
}
writeJson(path.join(output,'case.json'),data);writeJson(path.join(output,'tape.json'),data.tape);writeJson(path.join(output,'map-contract.json'),contract);
const clock=declarePublicClockPlan(data);writeJson(path.join(output,'public-clock-plan.json'),clock);
await assertOriginalBinary();
const library=path.join(output,'audio-trace.so');
execFileSync('cc',['-std=c11','-O2','-Wall','-Wextra','-Werror','-fPIC','-shared','tools/original/audio-trace.c','-o',library,'-ldl','-pthread'],{cwd:ROOT});
const libraryHash=digest(library);
const audioCatalogue=read(path.join(ROOT,'web/audio-data.json'));
const sound=audioCatalogue.sounds.find(s=>s.id===237);assert.equal(sound.samples.length,1);
const samplePath=path.join(ROOT,'references/isaac/rebirth/resources',sound.samples[0].path),sample=fs.readFileSync(samplePath);
assert.equal(hash(sample),audioCatalogue.assets[sound.samples[0].path].sha256,'Actual original237 input differs');
let pcm;
assert.equal(sample.toString('ascii',0,4),'RIFF');assert.equal(sample.toString('ascii',8,12),'WAVE');
for(let at=12;at+8<=sample.length;) {
  const size=sample.readUInt32LE(at+4),start=at+8;assert.ok(start+size<=sample.length);
  if(sample.toString('ascii',at,at+4)==='data')pcm=sample.subarray(start,start+size);
  at=start+size+(size&1);
}
assert.ok(pcm);const pcm237Sha256=hash(pcm);
const observer=path.join(ROOT,gammaMode?'tools/original/options-gamma-observe.py':'tools/original/options-map-observe.py');
const launchIdentity={schema:1,mode:gammaMode?'options-gamma-paired-v1':'options-map-fast31-paired-v1',binarySha256:ORIGINAL_SHA,bindingSha256:bindingHash,
  caseSha256:binding.caseSha256,contractSha256:binding.contractSha256,captureProducerSha256:before['tools/foundation/paired-game.mjs'],
  observerSha256:before[gammaMode?'tools/original/options-gamma-observe.py':'tools/original/options-map-observe.py'],audioLibrarySha256:libraryHash,
  publicClockSha256:hash(JSON.stringify(clock)),publicStartup:{audio:{sfxVolume:1,musicVolume:1},fullscreen:false,mapPreferences:null},nativeEnvironment:binding.nativeEnvironment};
writeJson(path.join(output,'observer-identity.json'),launchIdentity);
const report={schema:1,status:'running',identity:launchIdentity,sourceBefore:before,frames:[],diagnostics:[],
  scope:gammaMode?'One declared21Native/20Browser Gamma pair, actual settings/config/number/final GPU and V4 Back plus separate real Browser reload. No old Native regrade, HQ4X, Popups, Native reload or complete MT acceptance.':'One new fixed31Native/30Browser Map pair, complete public clock and inputs. Native-only probes and old84/V2 results do not qualify this run. No reload/migration/banner/controller/fullmix/completeMT acceptance.'};
let server,browser,native,candidate,comparison,error;
const observed={checkpoints:[],errors:[],failures:[]};
const retain=value=>fs.writeFileSync(path.join(output,'actual-browser-owners.json.gz'),gzipSync(JSON.stringify(value)));
try {
  server=await startProductionServer({directory:binding.artifact.directory});report.production=server.build;
  assert.equal(server.build.sha256,binding.artifact.sha256);assert.equal(server.build.files,binding.artifact.files);
  // The actual Native profile lives here. This guard follows the real immutable
  // production snapshot and never substitutes a pre-copy or another-FS budget.
  const free=fs.statfsSync(output);report.actualNativeFreeAfterSnapshot=Number(free.bavail)*Number(free.bsize);
  assert.ok(report.actualNativeFreeAfterSnapshot>=4*1024**3,'Native requires actual4GiB after the production snapshot');
  try {native=await nativeCapture(data,path.join(output,'original'),120,observer,true,1,false,false,true,false,clock,library,true,1);}
  catch(failure){native={status:'fail',error:String(failure.stack??failure)};}
  try {
    browser=await chromium.launch({...browserLaunchOptions(),headless:false});report.graphics=await inspectBrowserGraphics(browser);
    candidate=await browserCapture((gammaMode?gammaObservedBrowser:mapObservedBrowser)(browser,data,contract,observed,retain),server,data,path.join(output,'browser'),true,true,false,clock,{sfxVolume:1,musicVolume:1},false);
  } catch(failure){candidate={status:'fail',failures:[String(failure.stack??failure)],frames:[]};}
  finally {try{await browser?.close();}catch(failure){observed.failures.push(String(failure.stack??failure));}browser=undefined;retain(observed);}
  try {if(native.deferredEvidence)native=await native.deferredEvidence();if(native.status==='pass')native.publicClockValidation=validateNativeClockPlan(data,clock,native);}
  catch(failure){native.status='fail';native.error=String(failure.stack??failure);}
  writeJson(path.join(output,'native-status.json'),{status:native.status,error:native.error,provenance:native.provenance,clock:native.publicClockValidation,hostExecution:native.manifest?.hostExecution});
  writeJson(path.join(output,'browser-status.json'),{status:candidate.status,failures:candidate.failures});
  const calls=fs.readFileSync(path.join(output,'original/audio-trace/calls.jsonl'),'utf8').trim().split('\n').map(JSON.parse);
  assert.equal(calls[0].api,'trace-start');calls.slice(1).forEach((c,i)=>assert.equal(c.seq,i+1));
  comparison=(gammaMode?compareOptionsGamma:compareOptionsMap)({data,contract,clock,native,browser:candidate,observed,calls,scopedState,pcm237Sha256});
  writeJson(path.join(output,'functional-comparison.json'),comparison);
  const nativeFrames=new Map((native.manifest?.frames??[]).map(f=>[f.label,f]));const browserFrames=new Map(candidate.frames.map(f=>[f.label,f]));
  const nativePNGInputs=Object.fromEntries([...nativeFrames].map(([label,n])=>[label,{path:path.resolve(output,'original',n.path),sha256:digest(path.resolve(output,'original',n.path))}]));
  writeJson(path.join(output,'actual-native-png-inputs.json'),nativePNGInputs);
  fs.mkdirSync(path.join(output,'comparison'),{recursive:true});
  for(const label of contract.pairedLabels) {
    const row={label,status:'fail',regions:[]};
    try {
      const n=nativeFrames.get(label),b=browserFrames.get(label);assert.ok(n&&b,'Missing actual paired PNG');
      const npath=path.resolve(output,'original',n.path),nb=fs.readFileSync(npath),bb=fs.readFileSync(b.path);
      assert.equal(hash(nb),nativePNGInputs[label].sha256,'Actual retained Native PNG changed');assert.equal(hash(bb),b.sha256,'Actual Browser PNG hash differs');
      const np=PNG.sync.read(nb),bp=PNG.sync.read(bb);assert.deepEqual([np.width,np.height,bp.width,bp.height],[960,540,960,540]);
      row.sourcePNGHash=hash(nb);row.browserPNGHash=hash(bb);
      const diff=pixelDifference(np,bp),{difference,...values}=diff;
      fs.writeFileSync(path.join(output,'comparison',label+'-full-difference.png'),PNG.sync.write(difference));
      row.fullRGBA={...values,status:values.exactRGBA?'pass':'fail',required:false};
      for(const [name,spec] of Object.entries(contract.rectangles))if(spec.labels.includes(label)) {
        const d=pixelDifference(crop(np,spec.region),crop(bp,spec.region)),{difference,...stats}=d;
        fs.writeFileSync(path.join(output,'comparison',label+'-'+name+'-difference.png'),PNG.sync.write(difference));
        row.regions.push({name,region:spec.region,...stats,status:stats.exactRGBA?'pass':'fail',required:spec.required,scope:spec.scope});
      }
      row.status=row.regions.every(r=>r.status==='pass')?'pass':'fail';
    }catch(failure){row.error=String(failure.stack??failure);}
    report.frames.push(row);
  }
  for(const item of Object.values(nativePNGInputs))assert.equal(digest(item.path),item.sha256,'Actual Native PNG changed during comparison');
}catch(failure){error={message:String(failure.message),stack:failure.stack};}
finally {
  try{await browser?.close();}catch(failure){observed.failures.push(String(failure.stack??failure));}
  try{await server?.close();}catch(failure){error??={message:String(failure.stack??failure)};}
  try {
    const after=identity();writeJson(path.join(output,'source-after.json'),after);assert.deepEqual(after,before,'Frozen executing Source changed');
    assert.equal(digest(bindingPath),bindingHash);assert.equal(digest(binding.caseFile),binding.caseSha256);assert.equal(digest(binding.contractFile),binding.contractSha256);
    assert.equal(digest(library),libraryHash);assert.equal(digest(samplePath),audioCatalogue.assets[sound.samples[0].path].sha256);
    assert.deepEqual(artifactInventory(),artifactBefore,'Frozen artifact donor changed');
    report.sourceAfterUnchanged=true;
  }catch(failure){error??={message:String(failure.message),stack:failure.stack};}
  retain(observed);report.error=error;report.functional=comparison;
  report.status=!error&&native?.status==='pass'&&candidate?.status==='pass'&&comparison?.status==='pass'&&observed.errors.length===0&&observed.failures.length===0&&report.frames.length===contract.pairedLabels.length&&report.frames.every(r=>r.status==='pass')?'pass':'fail';
  writeJson(path.join(output,'report.json'),report);
}
console.log(JSON.stringify({status:report.status,report:path.join(output,'report.json')}));process.exitCode=report.status==='pass'?0:1;
