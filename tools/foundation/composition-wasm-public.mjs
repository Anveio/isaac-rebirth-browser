// Existing public WASM boot/key/tick route, with declared pickup descriptors and streamed actual State observations.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {createGzip} from 'node:zlib';
import {once} from 'node:events';
import {fileURLToPath} from 'node:url';
const [out,wasm,expectedWasmSha,casePath,expectedCaseSha]=process.argv.slice(2);
assert.ok(out&&wasm&&casePath&&/^[a-f0-9]{64}$/.test(expectedWasmSha)&&/^[a-f0-9]{64}$/.test(expectedCaseSha));
const donor=fileURLToPath(import.meta.url);
const sha=p=>createHash('sha256').update(fs.readFileSync(p)).digest('hex');
assert.equal(sha(wasm),expectedWasmSha);
assert.equal(sha(casePath),expectedCaseSha);
const c=JSON.parse(fs.readFileSync(casePath));
const inputPaths={wasm,case:casePath,xml:c.fixture.authoredResourcePath,stb:c.fixture.specialRoomsPath,donor};
const before=Object.fromEntries(Object.entries(inputPaths).map(([k,p])=>[k,{path:p,sha256:sha(p)}]));
const e=new WebAssembly.Instance(new WebAssembly.Module(fs.readFileSync(wasm)),{}).exports;
const decode=(p,n)=>new TextDecoder().decode(new Uint8Array(e.memory.buffer,p,n));
const resources={challengesXml:fs.readFileSync(inputPaths.xml,'utf8'),specialRooms:[...fs.readFileSync(inputPaths.stb)]};
const resetInput={initialSeed:1,bootEpochSeed:c.bootEpochSeed,resources};
fs.writeFileSync(out+'/declared-reset-input.json',JSON.stringify(resetInput)+'\n',{flag:'wx'});
assert.ok(Buffer.byteLength(JSON.stringify(resetInput))<=512*1024);
const bytes=new TextEncoder().encode(JSON.stringify(resources));let p=e.boot_resource_ptr(bytes.length);assert.ok(p);new Uint8Array(e.memory.buffer,p,bytes.length).set(bytes);assert.equal(e.reset_with_boot_resources(1,c.bootEpochSeed),0,decode(e.error_ptr(),e.error_len()));
const file=fs.createWriteStream(out+'/actual-states.jsonl.gz',{flags:'wx'}),gz=createGzip({level:6});gz.pipe(file);
const rawHash=createHash('sha256');let rawBytes=0,records=0;
const append=async x=>{const raw=Buffer.from(JSON.stringify(x)+'\n');rawHash.update(raw);rawBytes+=raw.length;records++;if(!gz.write(raw))await once(gz,'drain');};
let presentation=0,gameplayPresentation=null,observations=0,completedTape=false;
const commands=[],checkpoints=[];
const snapshot=()=>JSON.parse(decode(e.snapshot_ptr(),e.snapshot_len()));
async function observe(kind){const state=snapshot();await append({kind,presentation,gameplayPresentation,state});observations++;return state;}
function command(v){const b=new TextEncoder().encode(JSON.stringify(v));const q=e.request_ptr(b.length);assert.ok(q);new Uint8Array(e.memory.buffer,q,b.length).set(b);assert.equal(e.command(),0,decode(e.error_ptr(),e.error_len()));commands.push(v);if(v.type==='tick')presentation+=v.count;}
const names={Return:'Enter',Tab:'Tab',Right:'ArrowRight',Left:'ArrowLeft',Up:'ArrowUp',Down:'ArrowDown',space:'Space'},code=k=>names[k]??'Key'+k.toUpperCase();
function key(k,down){command({type:'key',code:code(k),down});}
async function press(k,n,label,dense=false){key(k,true);if(dense){for(let i=0;i<12;i++){command({type:'tick',count:1});await observe('prelude');}}else command({type:'tick',count:12});key(k,false);if(dense){for(let i=0;i<n;i++){command({type:'tick',count:1});await observe('prelude');}}else command({type:'tick',count:n});const s=snapshot();checkpoints.push({label,presentation,screen:s.screen});}
let failure;
try{
 await append({kind:'start',scope:'Own natural Card/Pill Init through public menu/key/tick/resource inputs. No DOM/render acceptance and no Native state/RNG/timing inputs.',before,caseName:c.name,initialSeed:1,bootEpochSeed:c.bootEpochSeed});
 assert.ok(Number.isFinite(c.sfxVolume)&&c.sfxVolume>=0&&c.sfxVolume<=1);
 command({type:'audioVolume',sfx_volume:c.sfxVolume});
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
await append({kind:'beforeAssertions',completedTape,presentation,gameplayPresentation,observations,commands,checkpoints,failure});gz.end();await once(file,'close');
const journalPath=out+'/actual-states.jsonl.gz',journal=fs.statSync(journalPath);
fs.writeFileSync(journalPath+'.meta.json',JSON.stringify({schema:1,status:completedTape&&!failure?'closed':'partial',encoding:'gzip-jsonl',path:journalPath,records,rawBytes,rawSha256:rawHash.digest('hex'),committedArchiveBytes:journal.size,archiveSha256:sha(journalPath),scope:'Actual public-WASM ABI observations; no DOM, renderer, audio backend or Native-input claim.'},null,2)+'\n',{flag:'wx'});
const after=Object.fromEntries(Object.entries(inputPaths).map(([k,p])=>[k,{path:p,sha256:sha(p)}]));
fs.writeFileSync(out+'/actual-replay-result.json',JSON.stringify({status:completedTape?'CAPTURE_COMPLETE':'FAIL',scope:'Functional public-WASM whole game only; independent Original comparison follows actual persisted output. DOM, renderer, screenshots, audio backend and PCM unavailable.',before,after,completedTape,presentation,gameplayPresentation,observations,commands,checkpoints,failure,captureBackend:'wasm-node'},null,2)+'\n',{flag:'wx'});
assert.deepEqual(after,before);assert.equal(completedTape,true);assert.equal(gameplayPresentation,c.initialPhaseOffset+c.actions.reduce((n,a)=>n+a.presentations,0));
console.log(JSON.stringify({completedTape,presentation,gameplayPresentation,observations,output:out}));
