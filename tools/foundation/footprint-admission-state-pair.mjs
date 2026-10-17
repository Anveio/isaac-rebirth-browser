// Admission-only regression composed from existing exact entity contracts.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {entityRows,loadManifest,markers,npcFields} from './entity-compare.mjs';
import {scopedState} from './paired-game.mjs';
const flag=name=>{const i=process.argv.lastIndexOf(name);assert.ok(i>=0&&process.argv[i+1],`Missing ${name}`);return process.argv[i+1];};
const output=resolve(flag('--output')),directory=resolve(flag('--case'));
const checks=[],errors=[];
let execution;
const spec={name:'footprint-admission',roomType:6,matches:()=>false,
  roomFields(source,actual,phase){
    const native=source.entities.filter(n=>n.type>=10&&n.type<1000),port=actual.room.npcs.filter(n=>!n.removed);
    const keyed=(actors,ported)=>{const seen=new Map();return actors.map(n=>{const k=JSON.stringify([ported?n.kind:n.type,n.variant,n.spawnSeed]);const occurrence=seen.get(k)??0;seen.set(k,occurrence+1);return[JSON.stringify([k,occurrence]),n];});};
    const candidates=new Map(keyed(port,true));
    return keyed(native,false).flatMap(([key,n])=>{const actor=candidates.get(key),name=`admission.npc${key}`;
      return [[name+'.managerMember',true,actor?true:undefined],...(actor?npcFields(n,actor,phase).map(([field,expected,actual])=>[name+'.'+field,expected,actual]):[])];});
  }};
try{
  const c=JSON.parse(readFileSync(resolve(directory,'case.json'),'utf8'));
  const endpoint=c.pairedTape.at(-1);
  assert.ok(endpoint?.type==='capture'&&['entry-1','entry-40'].includes(endpoint.label),'Require a declared pre-shoot admission endpoint');
  assert.equal(c.pairedTape.filter(a=>a.type==='capture'&&a.label===endpoint.label).length,1,'Require one literal admission endpoint');
  assert.deepEqual(endpoint,{type:'capture',label:endpoint.label});
  if(c.entity?.stopAfterCapture!==undefined)assert.equal(c.entity.stopAfterCapture,endpoint.label);
  assert.deepEqual(c.tape,[...c.bootstrap.sourceOnlyPrelude,...c.pairedTape]);
  const native=loadManifest(directory,'original'),browser=loadManifest(directory,'browser');
  assert.equal(native.status,'pass');assert.equal(browser.status,'pass');
  assert.deepEqual(native.tape,c.tape,'Require the actual Native public prefix');
  assert.deepEqual(browser.frames.map(f=>f.label),c.pairedTape.filter(a=>a.type==='capture').map(a=>a.label));
  const observations=new Map(native.debugObservations.map(row=>[row.context.label,row.observation]));
  const frames=new Map(browser.frames.map(row=>[row.label,row]));
  let started=false,nativeNpcAdmission=false,browserNpcAdmission=false;
  for(const action of c.pairedTape){
    if(action.type!=='capture')continue;
    started ||= action.label==='gameplay-start';if(!started)continue;
    const rows=[];
    try{
      const observation=observations.get(action.label),source=observation?.gameplay,frame=frames.get(action.label);
      assert.equal(source?.status,'pass',`Missing actual Native gameplay owner ${action.label}`);
      assert.equal(source.memoryWritten,false);assert.equal(frame?.state?.screen,'playing',`Missing actual Browser gameplay owner ${action.label}`);
      const actual=frame.state;rows.push(...scopedState(observation,actual,true));
      entityRows(rows,source,actual,spec,frame.render);
      nativeNpcAdmission ||= source.room.type===6&&source.entities.some(n=>n.type>=10&&n.type<1000&&!n.removed&&n.active&&n.hp>0);
      browserNpcAdmission ||= actual.room.descriptor?.template?.type===6&&actual.room.npcs.some(n=>!n.removed&&!n.dead&&n.hp>0);
    }catch(error){errors.push({label:action.label,message:error.message,stack:error.stack});checks.push({label:action.label,field:'admission.actualOwners',expected:'complete actual owners',actual:error.message,status:'fail'});}
    finally{checks.push(...rows.map(row=>({label:action.label,...row})));}
  }
  assert.ok(checks.length,'Every case must compare actual gameplay fields');
  execution=markers(directory,spec);
  for(const side of ['original','browser'])for(const [field,actual]of [['roomType',execution[side].roomType],['npcAdmission',side==='original'?nativeNpcAdmission:browserNpcAdmission]])checks.push({label:'execution',field:side+'.'+field,expected:true,actual,status:actual?'pass':'fail'});
}catch(error){errors.push({message:error.message,stack:error.stack});}
const status=!errors.length&&checks.length>0&&checks.every(row=>row.status==='pass')?'pass':'fail';
writeFileSync(output,JSON.stringify({schema:1,status,checks,errors,execution,
  scope:'Admission prefix through its declared entry-1 or entry-40 endpoint only. Existing exact generic/player/room/render contracts and every initial NPC field by actual identity/seed/occurrence; both games actually enter miniboss room6 with a living NPC. No shooting, lifecycle, clear or complete shared-MT acceptance.'},null,2)+'\n');
console.log(JSON.stringify({status,checks:checks.length,failures:checks.filter(r=>r.status!=='pass').length,errors:errors.length}));
process.exitCode=status==='pass'?0:1;
