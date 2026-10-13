// Live original and production-browser contracts; no fixture state enters WASM.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { waitForReady } from '../foundation/verify.mjs';
import { TearContracts } from './tear-contracts.mjs';
const ROOT=resolve(fileURLToPath(new URL('../..',import.meta.url)));
const run=promisify(execFile);
const epoch=1700000000;
const ORIGINAL_SHA='724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13';
const GATE_SOURCES = ["tools/e2e/weapons.mjs", "tools/e2e/tear-contracts.mjs", "tools/original/foundation-physics-observer.py", "tools/original/native-identity.py", "tools/original/state-observer.py", "tools/original/foundation-clock-run.py", "tools/original/foundation-clock-gate.c", "tools/original/foundation-seeds.py", "tools/original/debug-run.py", "tools/original/with-display.sh", "tools/foundation/browser.mjs", "tools/foundation/production-server.mjs", "tools/foundation/verify.mjs", "tools/original/weapons-scenarios.py"];
const sourceHashes = async () => Object.fromEntries(await Promise.all(GATE_SOURCES.map(async path => [path, createHash('sha256').update(await readFile(join(ROOT, path))).digest('hex')])));
const gateSources = await sourceHashes();
const value=(flag)=>{const index=process.argv.indexOf(flag);return index<0?undefined:process.argv[index+1];};
const lifecycle=process.argv.includes('--lifecycle');
const heldPresentations=lifecycle?1:54,releasePresentations=lifecycle?110:26;
const output=resolve(value('--output')||join(ROOT,'.cache/e2e/weapons',String(Date.now())));
const randomCount=Number(value('--count')||4);
assert.ok(Number.isInteger(randomCount)&&randomCount>=2&&randomCount<=16,'Random coverage must contain2..16 recorded seeds');
await mkdir(output,{recursive:true});
const encoded=await run('python3',[join(ROOT,'tools/original/foundation-seeds.py'),'--count',String(randomCount)],{cwd:ROOT});
const vectors=JSON.parse(encoded.stdout).vectors;
const directions=['Right','Left','Up','Down'];
let cases=[{name:'full-first-shot',code:'QSYVKMSE',direction:'Right',delay:0},{name:'intermediate-first-shot',code:'ZG800TRJ',direction:'Right',delay:1},
 ...vectors.map((vector,index)=>({name:`random-${index}`,code:vector.code.replaceAll(' ',''),direction:directions[index%4],delay:index%2,seedValue:vector.seedValue}))];
if(value('--scenarios')){
 const replay=JSON.parse(await readFile(resolve(value('--scenarios')),'utf8'));
 assert.equal(replay.schema,1);assert.equal(replay.bootEpochSeed,epoch);cases=replay.cases;
}
assert.ok(Array.isArray(cases)&&cases.length>=4&&cases.length<=18);
assert.equal(new Set(cases.map(scenario=>scenario.name)).size,cases.length);
for(const scenario of cases){
 assert.ok(/^[a-zA-Z0-9_-]{1,32}$/.test(scenario.name));
 assert.ok(typeof scenario.code==='string'&&scenario.code.length===8);
 assert.ok(directions.includes(scenario.direction)&&[0,1].includes(scenario.delay));
}
const sourceBinary=await readFile(join(ROOT,'.cache/original-runtime/offline-depot/isaac.x64'));
assert.equal(createHash('sha256').update(sourceBinary).digest('hex'),ORIGINAL_SHA,'Original executable differs from the source-located observer contract');
await writeFile(join(output,'scenarios.json'),JSON.stringify({schema:1,bootEpochSeed:epoch,source:'Contrived full/intermediate births plus OS-random recorded seed coverage',cases},null,2)+'\n');
const production=await startProductionServer();
const browser=await chromium.launch(browserLaunchOptions());
const report={schema:1,status:'running',gateSources,originalBinarySha256:ORIGINAL_SHA,scope:lifecycle?'Live original/browser source tear ground/height recurrence, half prediction, PRE range/grid death and next-full removal; RNG-dependent numeric mismatches are reported separately from these source contracts. Poof effects and complete shared stream parity remain incomplete':'Paired live original/browser weapon cooldown, held/released head clock, shot births, actor birth fence and horizontal birth positions',production:production.build,graphics:await inspectBrowserGraphics(browser),cases:[]};
try {
 for(const scenario of cases){
  const directory=join(output,scenario.name);await mkdir(directory,{recursive:true});
  let page;const states=[];let births=0;let comparisons=0;const contracts=lifecycle?new TearContracts():undefined;
  const result={...scenario,status:'running'};report.cases.push(result);
  try {
  const tape=await run('python3',[join(ROOT,'tools/original/weapons-scenarios.py'),'--seed',scenario.code,'--direction',scenario.direction,'--delay',String(scenario.delay),'--frames',String(heldPresentations),'--release-frames',String(releasePresentations),...(lifecycle?['--lifecycle']:[])],{cwd:ROOT});
  await writeFile(join(directory,'tape.json'),tape.stdout);
  await run('bash',[join(ROOT,'tools/original/with-display.sh'),'python3',join(ROOT,'tools/original/foundation-clock-run.py'),'--seed',scenario.code,'--tape',join(directory,'tape.json'),'--observe-every-step','--state-observer',join(ROOT,'tools/original/foundation-physics-observer.py'),'--output',join(directory,'original')],{cwd:ROOT,timeout:180000,maxBuffer:1024*1024});
  const raw=await readFile(join(directory,'original/manifest.json'));const native=JSON.parse(raw);
  result.nativeManifestSha256=createHash('sha256').update(raw).digest('hex');result.nativeManifest=join(directory,'original/manifest.json');
  assert.equal(native.status,'pass');assert.deepEqual(native.failures,[]);
  const baseline=native.debugObservations.find(row=>row.context.label==='gameplay-start');
  assert.ok(baseline&&baseline.observation.memoryWritten===false);
  assert.equal(baseline.observation.nativeBinarySha256,ORIGINAL_SHA);
  assert.equal(baseline.observation.systemPhaseCounter%2,0,'Native normalized gameplay boundary must begin a full phase; do not fit a per-seed clock offset');
  page=await browser.newPage();const errors=[];page.on('pageerror',error=>errors.push(String(error)));
  await page.goto(`${production.url}/?manual=1`);await waitForReady(page);
  const initialSeed=scenario.seedValue||native.seedValue;
  await page.evaluate(({initialSeed,epoch})=>{window.game.pause();window.game.reset(initialSeed,{trace:true,bootEpochSeed:epoch});},{initialSeed,epoch});
  for(let i=0;i<3;i++)await page.keyboard.press('Enter');
  await page.keyboard.press('Tab');await page.keyboard.type(scenario.code);await page.keyboard.press('Enter');await page.keyboard.press('Enter');
  const start=await page.evaluate(()=>window.game.snapshot());assert.equal(start.screen,'playing');assert.equal(start.systemClock.systemCounter%2,0);
  assert.equal(start.fireCooldown,baseline.observation.players[0].fireCooldown);
  if(scenario.delay)await page.evaluate(()=>window.game.step(1));
  if(lifecycle){await page.keyboard.down('KeyA');await page.evaluate(()=>window.game.step(30));await page.keyboard.up('KeyA');await page.evaluate(()=>window.game.step(30));}
  const key=`Arrow${scenario.direction}`;await page.keyboard.down(key);
  const rows=new Map(native.debugObservations.map(row=>[row.context.presentedFrame,row.observation]));
  const seen=new Set();
  for(let index=1;index<=heldPresentations+releasePresentations;index++){
   if(index===heldPresentations+1)await page.keyboard.up(key);
   await page.evaluate(()=>window.game.step(1));const state=await page.evaluate(()=>window.game.snapshot());states.push(state);
   const nativeFrame=baseline.context.presentedFrame+scenario.delay+(lifecycle?60:0)+index;const observed=rows.get(nativeFrame);
   assert.ok(observed&&observed.memoryWritten===false,`${scenario.name}:missing readonly native phase${nativeFrame}`);
   assert.equal(observed.nativeBinarySha256,ORIGINAL_SHA,`${scenario.name}:${index}:running executable`);
   const player=observed.players[0];
   for(const [field,actual] of [['fireCooldown',state.fireCooldown],['maxFireDelay',state.maxFireDelay],['headTimer',state.headTimer]])assert.equal(actual,player[field],`${scenario.name}:${index}:${field}`);
   assert.equal(state.player.headFrame,player.headAnimationFrame,`${scenario.name}:${index}:head frame`);
   assert.equal(state.systemClock.systemCounter%2,observed.systemPhaseCounter%2,`${scenario.name}:${index}:phase`);
   const admitted=[];
   for(const tear of observed.tears){
    if(tear.active!==true||tear.dead===true)continue;
    const id=`${tear.referenceAddress}:${tear.opaqueBaseBytes.slice(16,24)}`;
    if(!seen.has(id)){seen.add(id);births++;admitted.push(tear);}
   }
   assert.equal(state.nextTear-1,births,`${scenario.name}:${index}:birth admission`);
   if(admitted.length){
    assert.equal(admitted.length,1,'Regular fresh Isaac admits one tear per shot');
    const tear=state.tears.find(tear=>tear.id===state.nextTear-1);assert.ok(tear);
    assert.equal(tear.age,0,'Newborn tear is excluded from its birth full update; fast birth only predicts');
    if(['Left','Right'].includes(scenario.direction))assert.equal(Math.fround((tear.x-32)/.65),admitted[0].position[0],`${scenario.name}:${index}:horizontal birth projection`);
   }
   const handles=state.actorScheduler.actors;assert.deepEqual(handles[0],{kind:'player',id:0});
   assert.deepEqual(handles.filter(actor=>actor.kind==='tear').map(actor=>actor.id),state.tears.map(tear=>tear.id));
   contracts?.step(state,observed,index);
   comparisons++;
  }
  assert.deepEqual(errors,[]);
  result.status='pass';
  } catch(error) {result.status='fail';result.error=String(error);}
  finally {
   result.comparedPresentations=comparisons;result.comparedHeldPresentations=Math.min(comparisons,heldPresentations);result.comparedReleasePresentations=Math.max(0,comparisons-heldPresentations);result.observedBirths=births;
   if(contracts)result.lifecycle=contracts.summary;
   await writeFile(join(directory,'browser-states.json'),JSON.stringify({schema:1,states},null,2)+'\n');
   if(page)await page.close().catch(error=>{result.status='fail';result.cleanupError=String(error);});
   await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
  }
 }
 report.gateSourcesEnd=await sourceHashes();
 report.gateSourcesStable=JSON.stringify(report.gateSources)===JSON.stringify(report.gateSourcesEnd);
 assert.equal(report.gateSourcesStable,true,'Weapons gate source files changed during capture');
 report.status=report.cases.every(result=>result.status==='pass')?'pass':'fail';
 if(report.status==='fail')throw new AggregateError(report.cases.filter(result=>result.status==='fail').map(result=>new Error(`${result.name}:${result.error}`)),'Selected live weapons cases failed; every case was attempted');
} catch(error){report.status='fail';report.error=String(error);throw error;}
finally{await writeFile(join(output,'report.json'),JSON.stringify(report,null,2)+'\n');await browser.close();await production.close();console.log(JSON.stringify({status:report.status,report:join(output,'report.json'),cases:report.cases.length}));}
