// Fresh actual-game Heart10 contracts; original observations never supply port state.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync,mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {gunzipSync} from 'node:zlib';
import {runPairedGate} from '../foundation/paired-game.mjs';
const option=(name,fallback)=>{const at=process.argv.indexOf(name);if(at<0)return fallback;assert.ok(process.argv[at+1]);return process.argv[at+1];};
const output=resolve(option('--output','.cache/e2e/hearts'));mkdirSync(output,{recursive:true});
const save=(p,v)=>writeFileSync(p,JSON.stringify(v,null,2)+'\n');
const digest=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
const artifactBytes=p=>existsSync(p)?readFileSync(p):gunzipSync(readFileSync(p+'.gz'));
const artifactDigest=p=>createHash('sha256').update(artifactBytes(p)).digest('hex');
const sources=['tools/e2e/hearts.mjs','tools/original/heart-scenarios.py','tools/original/foundation-combat-observe.py','engine/src/pickups.rs','engine/src/player_items.rs','engine/src/fireplaces.rs','engine/src/lib.rs','references/isaac/foundation/hearts/first-prepared-cases.json','web/engine.ts','web/renderer.ts'];
const hashes=()=>Object.fromEntries(sources.map(p=>[p,digest(p)])),before=hashes();
const prepared=option('--scenarios');
const scenarios=prepared?JSON.parse(readFileSync(resolve(prepared),'utf8')):JSON.parse(execFileSync('python3',['tools/original/heart-scenarios.py','--count',option('--count','2')],{encoding:'utf8'}));
assert.equal(scenarios.schema,1);assert.ok(Array.isArray(scenarios.cases)&&scenarios.cases.length>=6&&scenarios.cases.length<=8);
const fixedBankPath='references/isaac/foundation/hearts/first-prepared-cases.json';
assert.equal(digest(fixedBankPath),'6860834fd32322bf690d01e33f5b78a3db4fc4ad3e5e897831c15e37fa55e76e','The original prepared bank must remain immutable');
const fixedBank=JSON.parse(readFileSync(fixedBankPath,'utf8'));
const mandatory=new Map(['red','soul'].flatMap(kind=>[0,1].map(phase=>{
 const id=`paired-heart-${kind}-${phase}`,fixture=fixedBank.cases.find(c=>c.scenario.id===id);
 assert.ok(fixture,'Missing sealed mandatory Heart fixture');return[id,fixture];
})));
const ids=scenarios.cases.map(c=>c.scenario.id);assert.equal(new Set(ids).size,ids.length,'Heart case IDs must be unique');
for(const[id,fixture]of mandatory){
 const selected=scenarios.cases.find(c=>c.scenario.id===id);assert.ok(selected,`Missing mandatory ${id}`);
 for(const field of ['seed','seedValue','initialSeed','bootEpochSeed','bootstrap','tape','pairedTape'])
  assert.deepEqual(selected[field],fixture[field],`${id}: immutable fixture identity and declared phase-offset tape`);
 assert.equal(selected.scenario.randomized,false,`${id}: fixed cases cannot become random diagnostics`);
}
const osCases=scenarios.cases.filter(c=>!mandatory.has(c.scenario.id));
assert.ok(osCases.length>=2&&osCases.length<=4&&osCases.every(c=>c.scenario.randomized===true),'Every bank requires at least two recorded OS cases');
assert.deepEqual(scenarios.unfilteredOsSeedValues,osCases.map(c=>c.seedValue),'Every recorded unfiltered OS outcome must be retained');
save(resolve(output,'scenarios.json'),scenarios);
const tapes=scenarios.cases.map((c,i)=>{const p=resolve(output,`replay-${i}.json`);save(p,c);return p;});
const paired=await runPairedGate({output:resolve(output,'paired'),selected:[],randomCount:0,replayCases:tapes,productionDirectory:option('--production-directory'),nativeFrameTimeout:Number(option('--native-frame-timeout','60'))});
const results=[];
for(const [index,c] of scenarios.cases.entries()){
 const directory=resolve(output,'paired',`${String(index).padStart(2,'0')}-${c.scenario.id}`),checks=[],execution={native:{admission:false,damage:false,collection:false},browser:{admission:false,damage:false,collection:false}};
 let error;const originalPath=resolve(directory,'original/manifest.json'),browserPath=resolve(directory,'browser/manifest.json');
 try{
  const original=JSON.parse(artifactBytes(originalPath)),browser=JSON.parse(artifactBytes(browserPath));
  const caseReport=JSON.parse(readFileSync(resolve(directory,'report.json'),'utf8'));
  assert.equal(caseReport.original.status,'pass');assert.equal(caseReport.browser.status,'pass');
  assert.equal(original.status,'pass');assert.equal(browser.status,'pass');assert.equal(paired.gateSourcesStable,true);
  const frames=new Map(browser.frames.map(r=>[r.label,r.state])),observations=new Map(original.debugObservations.map(r=>[r.context.label,r.observation]));
  const check=(label,field,expected,actual)=>{assert.notEqual(expected,undefined,field);assert.notEqual(actual,undefined,field);checks.push({label,field,expected,actual,status:JSON.stringify(expected)===JSON.stringify(actual)?'pass':'fail'});};
  const f32=v=>{assert.equal(typeof v,'number');assert.ok(Number.isFinite(v));return Math.fround(v);};
  const vector=v=>[f32(v.x??v[0]),f32(v.y??v[1])];
  let prior,started=false;
  for(const action of c.pairedTape){
   if(action.type!=='capture')continue;const label=action.label,observation=observations.get(label),state=frames.get(label);
   assert.ok(observation&&state,`Missingdeclared${label}`);assert.equal(observation.status,'pass');
   if(label==='gameplay-start')started=true;if(!started)continue;assert.equal(state.screen,'playing','Every post-start checkpoint requires actualgameplay');const source=observation.gameplay;assert.equal(source.status,'pass');assert.equal(source.memoryWritten,false);
   const player=source.players[0];assert.ok(player);
   const hpNative=[player.maxRedHp,player.redHp,player.soulHp,player.blackHeartMask,player.eternalHp];
   const hpBrowser=[state.player.maxHp,state.player.hp,state.player.soulHp,state.player.blackHeartMask,state.player.eternalHearts];
   assert.ok([...hpNative,...hpBrowser].every(Number.isInteger),'Every declared healthowner must be observed');
   check(label,'player.healthContainers',hpNative,hpBrowser);
   check(label,'player.immunity',player.damageCountdown,state.damageCooldown);
   check(label,'room.identity',source.room.descriptorIndex,state.currentRoom);
   const native=source.entities.filter(a=>a.type===5&&a.variant===10),port=state.room.pickups.filter(a=>a.variant===10&&!a.removed);
   check(label,'heart.authoredSeedOrder',native.map(a=>a.spawnSeed),port.map(a=>a.spawnSeed));
   for(const heart of native){
    const pickup=port.find(p=>p.spawnSeed===heart.spawnSeed);if(!pickup)continue;
    for(const [field,expected,actual] of [['subtype',heart.subtype,pickup.subtype],['animation',heart.animation,pickup.animation],['frame',f32(heart.animationFrame),f32(pickup.animationFrame)],['playing',heart.animationPlaying,pickup.animationPlaying],['collected',heart.removed,pickup.collected],['position',vector(heart.position),vector(source.nativePhase.systemIterations%2===0?pickup.presentedPosition:pickup.position)],['velocity',vector(heart.velocity),vector(pickup.velocity)],['gridClass',heart.gridCollisionClass,pickup.regularCollision?.gridCollisionClass],['entityClass',heart.entityCollisionClass,pickup.regularCollision?.entityCollisionClass]])check(label,`heart${heart.spawnSeed}.${field}`,expected,actual);
   }
   execution.native.admission ||=native.length>0;execution.browser.admission ||=port.length>0;
   execution.native.collection ||=native.some(a=>a.removed);execution.browser.collection ||=port.some(a=>a.collected);
   if(prior){execution.native.damage ||=hpNative[1]+hpNative[2]<prior.native[1]+prior.native[2];execution.browser.damage ||=hpBrowser[1]+hpBrowser[2]<prior.browser[1]+prior.browser[2];}
   prior={native:hpNative,browser:hpBrowser};
  }
  for(const name of ['native','browser'])for(const required of mandatory.has(c.scenario.id)?['admission','damage','collection']:c.heartRequired)check('execution',`${name}.${required}`,true,execution[name][required]);
 }catch(failure){error=String(failure.stack??failure);}
 const result={id:c.scenario.id,status:!error&&checks.length>0&&checks.every(r=>r.status==='pass')?'pass':'fail',execution,checks,error,manifestHashes:{original:()=>artifactDigest(originalPath),browser:()=>artifactDigest(browserPath)}};
 // A failed or missing manifest must remain a report, not abort later cases.
 result.manifestHashes=Object.fromEntries(Object.entries(result.manifestHashes).map(([name,read])=>{try{return[name,read()];}catch{return[name,null];}}));
 save(resolve(directory,'heart-state.json'),result);results.push({id:result.id,status:result.status,fields:checks.length,failures:checks.filter(r=>r.status==='fail').length,execution,error,report:resolve(directory,'heart-state.json')});
}
const stable=JSON.stringify(before)===JSON.stringify(hashes());
const report={schema:1,status:stable&&paired.status==='pass'&&results.every(r=>r.status==='pass')?'pass':'fail',pairedStatus:paired.status,sourcesStable:stable,sources:before,cases:results,scope:'Required paired healthcontainer/immunity/room/Heartbody-spawn-seed-identity-class-animation-collection fields, contrived actualadmission/damage/collection and2unfilteredrecordedOScases. Modifier-completefactory, wholeHUD/informationqueue13, readmission andsoul/blackdamagepriority remain explicit source-qualified follow-ups. Local30 four-word state and source collection SFX are not compared by this leaf.'};
save(resolve(output,'report.json'),report);console.log(JSON.stringify(report));process.exitCode=report.status==='pass'?0:1;
