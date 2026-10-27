// Existing public WASM boot/key/tick route, with declared pickup descriptors and streamed actual State observations.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {createGzip} from 'node:zlib';
import {once} from 'node:events';
import {fileURLToPath} from 'node:url';
import {resetBootElapsed} from './verify.mjs';

const sha=p=>createHash('sha256').update(fs.readFileSync(p)).digest('hex');
// composition-run.py's declared Native intro before the shared Title path.
export const DECLARED_NATIVE_INTRO=[{type:'step',frames:180},{type:'key',key:'Return',down:true},{type:'step',frames:12},
 {type:'key',key:'Return',down:false},{type:'step',frames:72}];
const journalScope='Actual public-WASM ABI observations; no DOM, renderer, audio backend or Native-input claim.';

// The caller supplies only declared public inputs and owns its input journal.
export function createPublicWasmRuntime({wasm,expectedWasmSha}) {
 assert.match(expectedWasmSha,/^[a-f0-9]{64}$/);
 assert.equal(sha(wasm),expectedWasmSha);
 const e=new WebAssembly.Instance(new WebAssembly.Module(fs.readFileSync(wasm)),{}).exports;
 const decode=(p,n)=>new TextDecoder().decode(new Uint8Array(e.memory.buffer,p,n));
 const snapshot=()=>JSON.parse(decode(e.snapshot_ptr(),e.snapshot_len()));
 // bootElapsed is verify.mjs resetBootElapsed(declared Native tape): the fresh clock's
 // elapsed time at the original's first rendered Title, as web/engine.ts applies it.
 function reset(initialSeed,bootEpochSeed,resources,audioPreferences,fullscreenPreferences,bootElapsed) {
  assert.ok(Number.isInteger(initialSeed)&&initialSeed>=1&&initialSeed<=0xffffffff);
  assert.ok(Number.isInteger(bootEpochSeed)&&bootEpochSeed>=0&&bootEpochSeed<=0xffffffff);
  if(audioPreferences!==undefined) {
   assert.deepEqual(Object.keys(audioPreferences).sort(),['musicVolume','sfxVolume']);
   assert.ok([audioPreferences.sfxVolume,audioPreferences.musicVolume].every(value=>Number.isFinite(value)&&value>=0&&value<=1));
  }
  if(fullscreenPreferences!==undefined) {
   assert.deepEqual(Object.keys(fullscreenPreferences),['fullscreen']);assert.equal(typeof fullscreenPreferences.fullscreen,'boolean');
  }
  const reset=(withResources)=>{
   if(fullscreenPreferences!==undefined) {
    const method=withResources?e.reset_with_boot_resources_and_fullscreen_preferences:e.reset_with_fullscreen_preferences;
    assert.equal(typeof method,'function','Actual declared startup preferences require the real additive ABI');
    return method(initialSeed,bootEpochSeed,Number(fullscreenPreferences.fullscreen),Number(audioPreferences!==undefined),audioPreferences?.sfxVolume??0,audioPreferences?.musicVolume??0);
   }
   if(audioPreferences!==undefined) {
    const method=withResources?e.reset_with_boot_resources_and_audio_preferences:e.reset_with_audio_preferences;
    assert.equal(typeof method,'function','Actual declared audio preferences require the real additive ABI');
    return method(initialSeed,bootEpochSeed,audioPreferences.sfxVolume,audioPreferences.musicVolume);
   }
   return withResources?e.reset_with_boot_resources(initialSeed,bootEpochSeed):e.reset(initialSeed,bootEpochSeed);
  };
  let result;
  if(resources===undefined) result=reset(false);
  else {
   const bytes=new TextEncoder().encode(JSON.stringify(resources));
   assert.ok(bytes.length<=512*1024);
   const p=e.boot_resource_ptr(bytes.length);assert.ok(p);
   new Uint8Array(e.memory.buffer,p,bytes.length).set(bytes);
   result=reset(true);
  }
  assert.equal(result,0,decode(e.error_ptr(),e.error_len()));
  if(bootElapsed!==undefined) {
   const {milliseconds,subMillisecondNanoseconds}=bootElapsed;
   assert.equal(e.set_boot_elapsed(milliseconds,subMillisecondNanoseconds),0,decode(e.error_ptr(),e.error_len()));
  }
 }
 function command(value) {
  const b=new TextEncoder().encode(JSON.stringify(value));const q=e.request_ptr(b.length);assert.ok(q);
  new Uint8Array(e.memory.buffer,q,b.length).set(b);
  assert.equal(e.command(),0,decode(e.error_ptr(),e.error_len()));
 }
 return {reset,command,snapshot};
}

export function createPublicWasmJournal(directory,{scope=journalScope}={}) {
 const journalPath=directory+'/actual-states.jsonl.gz';
 const file=fs.createWriteStream(journalPath,{flags:'wx'}),gz=createGzip({level:6});gz.pipe(file);
 const rawHash=createHash('sha256');let rawBytes=0,records=0;
 const append=async x=>{const raw=Buffer.from(JSON.stringify(x)+'\n');rawHash.update(raw);rawBytes+=raw.length;records++;if(!gz.write(raw))await once(gz,'drain');};
 async function close({completed}) {
  assert.equal(typeof completed,'boolean');
  gz.end();await once(file,'close');
  const journal=fs.statSync(journalPath);
  const metadata={schema:1,status:completed?'closed':'partial',encoding:'gzip-jsonl',path:journalPath,records,rawBytes,rawSha256:rawHash.digest('hex'),committedArchiveBytes:journal.size,archiveSha256:sha(journalPath),scope};
  fs.writeFileSync(journalPath+'.meta.json',JSON.stringify(metadata,null,2)+'\n',{flag:'wx'});
  return metadata;
 }
 return {append,close};
}

async function main() {
const [out,wasm,expectedWasmSha,casePath,expectedCaseSha]=process.argv.slice(2);
assert.ok(out&&wasm&&casePath&&/^[a-f0-9]{64}$/.test(expectedWasmSha)&&/^[a-f0-9]{64}$/.test(expectedCaseSha));
const donor=fileURLToPath(import.meta.url);
assert.equal(sha(wasm),expectedWasmSha);
assert.equal(sha(casePath),expectedCaseSha);
const c=JSON.parse(fs.readFileSync(casePath));
const inputPaths={wasm,case:casePath,xml:c.fixture.authoredResourcePath,stb:c.fixture.specialRoomsPath,donor};
const before=Object.fromEntries(Object.entries(inputPaths).map(([k,p])=>[k,{path:p,sha256:sha(p)}]));
const runtime=createPublicWasmRuntime({wasm,expectedWasmSha});
const resources={challengesXml:fs.readFileSync(inputPaths.xml,'utf8'),specialRooms:[...fs.readFileSync(inputPaths.stb)]};
const audioPreferences={sfxVolume:c.sfxVolume,musicVolume:0},fullscreenPreferences={fullscreen:false};
assert.ok(Number.isFinite(audioPreferences.sfxVolume)&&audioPreferences.sfxVolume>=0&&audioPreferences.sfxVolume<=1);
const resetInput={initialSeed:1,bootEpochSeed:c.bootEpochSeed,resources,audioPreferences,fullscreenPreferences,bootElapsed:resetBootElapsed(DECLARED_NATIVE_INTRO)};
const producerBytes=fs.readFileSync(donor);
fs.writeFileSync(out+'/composition-wasm-public-source.mjs',producerBytes,{flag:'wx'});
const startupRecord={schema:1,protocol:'declared-preboot-audio-fullscreen-v1',phase:'before-reset',resetInput,
 captureSource:{path:donor,sha256:createHash('sha256').update(producerBytes).digest('hex'),archive:'composition-wasm-public-source.mjs'}};
fs.writeFileSync(out+'/declared-startup-inputs.json',JSON.stringify(startupRecord)+'\n',{flag:'wx'});
fs.writeFileSync(out+'/declared-reset-input.json',JSON.stringify(resetInput)+'\n',{flag:'wx'});
assert.ok(Buffer.byteLength(JSON.stringify(resetInput))<=512*1024);
runtime.reset(1,c.bootEpochSeed,resources,audioPreferences,fullscreenPreferences,resetInput.bootElapsed);
const journal=createPublicWasmJournal(out),append=journal.append;
let presentation=0,gameplayPresentation=null,observations=0,completedTape=false;
const commands=[],checkpoints=[];
const snapshot=runtime.snapshot;
async function observe(kind){const state=snapshot();await append({kind,presentation,gameplayPresentation,state});observations++;return state;}
function command(v){runtime.command(v);commands.push(v);if(v.type==='tick')presentation+=v.count;}
const names={Return:'Enter',Tab:'Tab',Right:'ArrowRight',Left:'ArrowLeft',Up:'ArrowUp',Down:'ArrowDown',space:'Space',Control_L:'ControlLeft'},code=k=>names[k]??'Key'+k.toUpperCase();
function key(k,down){command({type:'key',code:code(k),down});}
async function press(k,n,label,dense=false){key(k,true);if(dense){for(let i=0;i<12;i++){command({type:'tick',count:1});await observe('prelude');}}else command({type:'tick',count:12});key(k,false);if(dense){for(let i=0;i<n;i++){command({type:'tick',count:1});await observe('prelude');}}else command({type:'tick',count:n});const s=snapshot();checkpoints.push({label,presentation,screen:s.screen});}
let failure;
try{
 await append({kind:'start',scope:'Own natural Card/Pill Init through public menu/key/tick/resource inputs. No DOM/render acceptance and no Native state/RNG/timing inputs.',before,caseName:c.name,initialSeed:1,bootEpochSeed:c.bootEpochSeed,startupRecord});

 command({type:'tick',count:63});checkpoints.push({label:'title',presentation,screen:snapshot().screen});
 if(c.publicMenuRoute==='seeded-normal'){
  assert.equal(typeof c.publicSeedCode,'string');assert.match(c.publicSeedCode,/^[ABCDEFGHJKLMNPQRSTWXYZ01234V6789]{8}$/);
  await press('Return',72,'profile');await press('Return',72,'main');await press('Return',72,'character');await press('Tab',108,'seed-entry');
  for(const text of c.publicSeedCode){command({type:'text',text});command({type:'tick',count:12});command({type:'tick',count:6});}
  assert.equal(snapshot().screen,'seed');await press('Return',72,'character-seeded');await press('Return',300,'gameplay-start',true);
  assert.equal(snapshot().runSeedValue,c.publicRunSeed);assert.equal(snapshot().runSeed.replaceAll(' ',''),c.publicSeedCode);
 }else{
  assert.equal(c.publicMenuRoute,'retained-unfiltered-challenge');
  await press('Return',72,'profile');await press('Return',72,'main');await press('Down',72,'challenge-selected');await press('Return',72,'challenges');await press('Return',300,'gameplay-start',true);
 }
 gameplayPresentation=0;await observe('state');
 if(c.initialPhaseOffset){assert.equal(c.initialPhaseOffset,1);command({type:'tick',count:1});gameplayPresentation++;await observe('state');}
 let held=new Set;
 for(const a of c.actions){const next=new Set(a.keys);for(const k of[...held].filter(k=>!next.has(k)).sort())key(k,false);for(const k of[...next].filter(k=>!held.has(k)).sort())key(k,true);for(let i=0;i<a.presentations;i++){command({type:'tick',count:1});gameplayPresentation++;await observe('state');}checkpoints.push({label:a.label,presentation,gameplayPresentation,screen:snapshot().screen});held=next;}
 for(const k of[...held].sort())key(k,false);completedTape=true;
}catch(error){failure={error:String(error),stack:error.stack};}
await append({kind:'beforeAssertions',completedTape,presentation,gameplayPresentation,observations,commands,checkpoints,failure});
await journal.close({completed:completedTape&&!failure});
const after=Object.fromEntries(Object.entries(inputPaths).map(([k,p])=>[k,{path:p,sha256:sha(p)}]));
fs.writeFileSync(out+'/actual-replay-result.json',JSON.stringify({startupProtocol:startupRecord.protocol,startupRecord,status:completedTape?'CAPTURE_COMPLETE':'FAIL',scope:'Functional public-WASM whole game only; independent Original comparison follows actual persisted output. DOM, renderer, screenshots, audio backend and PCM unavailable.',before,after,completedTape,presentation,gameplayPresentation,observations,commands,checkpoints,failure,captureBackend:'wasm-node'},null,2)+'\n',{flag:'wx'});
assert.deepEqual(after,before);assert.equal(completedTape,true);assert.equal(gameplayPresentation,c.initialPhaseOffset+c.actions.reduce((n,a)=>n+a.presentations,0));
console.log(JSON.stringify({completedTape,presentation,gameplayPresentation,observations,output:out}));
}

if(process.argv[1]&&fileURLToPath(import.meta.url)===fs.realpathSync(process.argv[1])) await main();
