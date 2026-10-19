// Game-level production Browser diagnostic. Observe actual production WASM calls and origin storage.
// This observer never issues a public command, writes storage, changes arguments or
// replaces a WASM result. Save export bytes are copied BEFORE any other snapshot call.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdir,writeFile} from 'node:fs/promises';
import {join} from 'node:path';
export function installSaveAbiObserver() {
  const logs={schema:1,modules:[],events:[],records:[],errors:[],scope:'Actual production WASM invocation and origin-local storage observations; PORT consistency only, no Original parity'};
  const interned=new Map();
  const bytesRef=bytes=>{
    let binary='';for(let i=0;i<bytes.length;i+=8192)binary+=String.fromCharCode(...bytes.subarray(i,i+8192));
    const base64=btoa(binary);if(interned.has(base64))return interned.get(base64);
    const id=logs.records.length;logs.records.push({id,base64,bytes:bytes.length});interned.set(base64,id);return id;
  };
  const textRef=text=>text===null?null:bytesRef(new TextEncoder().encode(text));
  const observe=(fn,fallback=null)=>{try{return fn();}catch(error){logs.errors.push(String(error.stack??error));return fallback;}};
  const originalInstantiate=WebAssembly.instantiate;
  WebAssembly.instantiate=async function(...args){
    const result=await originalInstantiate.apply(this,args);
    const actual=result instanceof WebAssembly.Instance?result:result.instance;
    const ex=actual.exports;let resetOrdinal=0,commandOrdinal=0,inputPointer=0,inputLength=0;
    const copy=(ptr,len)=>{
      if(!Number.isSafeInteger(ptr)||!Number.isSafeInteger(len)||ptr<0||len<0||ptr+len>ex.memory.buffer.byteLength)throw new Error('Observed ABI invalid byte bounds');
      return new Uint8Array(ex.memory.buffer,ptr,len).slice();
    };
    const errorRef=()=>bytesRef(copy(ex.error_ptr(),ex.error_len()));
    const store=()=>observe(()=>({bytes:textRef(localStorage.getItem('isaac-rebirth.savegames.v1')),error:null}),{bytes:null,error:'observer-storage-read-failed'});
    const state=()=>observe(()=>{
      const ptr=ex.snapshot_ptr();const s=JSON.parse(new TextDecoder().decode(copy(ptr,ex.snapshot_len())));
      return {screen:s.screen,tick:s.tick,systemClock:s.systemClock,gameUpdateCount:s.gameUpdateCount,currentRoom:s.currentRoom,stage:s.stage,
        saveRevision:s.saveRevision,saveWritePending:s.saveWritePending,saveReadmissionPending:s.saveReadmissionPending,savedLoadValid:s.savedLoadValid,
        mainSelection:s.mainSelection,runSeed:s.runSeed,playerDead:s.player.dead,controlsEnabled:s.player.controlsEnabled,stains:s.room.backgroundStains.length};
    });
    const facade=Object.create(null);
    const event=name=>({name,resetOrdinal,commandOrdinal});
    for(const name of Object.keys(ex)){
      const original=ex[name];let wrapped=original;
      if(typeof original==='function'&&name.startsWith('reset'))wrapped=(...values)=>{resetOrdinal++;commandOrdinal=0;return original(...values);};
      else if(name==='command')wrapped=(...values)=>{commandOrdinal++;return original(...values);};
      else if(name==='save_input_ptr')wrapped=(...values)=>{const ptr=original(...values);inputPointer=ptr;inputLength=values[0];return ptr;};
      else if(name==='save_data_ptr')wrapped=(...values)=>{
        const returned=original(...values);
        // Do not call snapshot_ptr here: production Engine still needs this shared
        // SNAPSHOT return buffer and snapshot_len to decode its exported packet.
        observe(()=>logs.events.push({...event(name),args:values,returned,error:errorRef(),packet:returned===0?null:bytesRef(copy(returned,ex.snapshot_len()))}));
        return returned;
      };
      else if(name==='complete_saved_game_write'||name==='complete_save_menu_readmission')wrapped=(...values)=>{
        const row=event(name);observe(()=>Object.assign(row,{args:values,before:{state:state(),store:store()},input:name==='complete_save_menu_readmission'&&values[2]===1?bytesRef(copy(inputPointer,inputLength)):null}));
        try{
          const returned=original(...values);
          observe(()=>Object.assign(row,{returned,error:errorRef(),after:{state:state(),store:store()}}));
          logs.events.push(row);return returned;
        }catch(error){observe(()=>{row.thrown=String(error);logs.events.push(row);});throw error;}
      };
      Object.defineProperty(facade,name,{value:wrapped,enumerable:true});
    }
    const instance=new Proxy(actual,{get(target,key){return key==='exports'?facade:Reflect.get(target,key,target);}});
    const moduleInfo={exports:Object.keys(ex),abi:ex.abi_version(),imports:result.module?WebAssembly.Module.imports(result.module):null};
    observe(()=>logs.modules.push(moduleInfo));
    if(args[0] instanceof ArrayBuffer||ArrayBuffer.isView(args[0])){
      const raw=args[0] instanceof ArrayBuffer?new Uint8Array(args[0]):new Uint8Array(args[0].buffer,args[0].byteOffset,args[0].byteLength);
      moduleInfo.bytes=raw.length;
      moduleInfo.sha256=[...new Uint8Array(await crypto.subtle.digest('SHA-256',raw))].map(b=>b.toString(16).padStart(2,'0')).join('');
    }
    return result instanceof WebAssembly.Instance?instance:{...result,instance};
  };
  Object.defineProperty(window,'__saveCapacityAbiWitness',{value:{snapshot:()=>structuredClone(logs)}});
}
export async function archiveAndGradeSaveAbi(page,output,inputMode,check,expectedWasmSha256){
  const observed=await page.evaluate(()=>window.__saveCapacityAbiWitness.snapshot());
  await mkdir(join(output,'wasm-abi-bytes'),{recursive:false});
  const byId=new Map();
  for(const row of observed.records){
    const raw=Buffer.from(row.base64,'base64');assert.equal(raw.length,row.bytes);
    const sha256=createHash('sha256').update(raw).digest('hex'),path=join(output,'wasm-abi-bytes',sha256+'.bin');
    if(!byId.has(sha256))await writeFile(path,raw,{flag:'wx'});
    const reference={id:row.id,path,sha256,bytes:raw.length};byId.set(row.id,{raw,...reference});byId.set(sha256,reference);
    delete row.base64;Object.assign(row,reference);
  }
  const get=id=>byId.get(id)?.raw;
  const text=id=>get(id)?.toString('utf8');
  await writeFile(join(output,'actual-wasm-save-abi.json'),JSON.stringify(observed,null,2)+'\n',{flag:'wx'});
  check('abi:exact-production-module-and-observer-completeness',()=>{
    assert.equal(observed.modules.length,1);assert.match(expectedWasmSha256,/^[a-f0-9]{64}$/);assert.equal(observed.modules[0].sha256,expectedWasmSha256);
    assert.equal(observed.modules[0].abi,2);assert.deepEqual(observed.modules[0].imports,[]);assert.deepEqual(observed.errors,[]);
  });
  const exports=observed.events.filter(e=>e.name==='save_data_ptr');
  const acks=observed.events.filter(e=>e.name==='complete_saved_game_write');
  const reads=observed.events.filter(e=>e.name==='complete_save_menu_readmission');
  check('abi:every-save-export-has-one-real-ack-and-preserves-error-bytes',()=>{
    assert.equal(exports.length,2);assert.equal(acks.length,2);
    for(let i=0;i<exports.length;i++){
      const e=exports[i],a=acks[i];assert.equal(e.resetOrdinal,a.resetOrdinal);assert.equal(e.commandOrdinal,a.commandOrdinal);
      assert.equal(a.before.state.saveWritePending,true);assert.equal(a.before.state.saveRevision,a.args[0]+a.args[1]*0x100000000);
      assert.equal(a.returned,0);assert.equal(get(a.error).length,0);
      if(e.returned!==0){assert.ok(get(e.packet).length<=4*1024*1024);assert.equal(get(e.error).length,0);assert.equal(a.args[2],1);assert.deepEqual(get(e.packet),get(a.before.store.bytes));}
      else{assert.equal(e.packet,null);assert.equal(text(e.error),'saved profiles exceed the supported 4 MiB format');assert.equal(a.args[2],0);}
    }
  });
  check('abi:actual-readmission-input-is-the-real-store-and-clears-pending',()=>{
    assert.equal(reads.length,1);const r=reads[0];assert.equal(r.args[2],1);assert.equal(r.returned,0);assert.equal(get(r.error).length,0);
    assert.equal(r.before.state.saveReadmissionPending,true);assert.deepEqual(get(r.input),get(r.before.store.bytes));
    assert.deepEqual(get(r.input),get(r.after.store.bytes));assert.equal(r.after.state.saveReadmissionPending,false);assert.equal(r.after.state.savedLoadValid,true);
  });
  if(inputMode==='fresh-normal-over')check('abi:capacity-failedAck-keeps-old-store-and-revision-then-real-readmission',()=>{
    assert.equal(exports[1].returned,0);const a=acks[1],r=reads[0];assert.deepEqual(a.args,[2,0,0]);
    assert.deepEqual(get(a.before.store.bytes),get(exports[0].packet));assert.deepEqual(get(a.after.store.bytes),get(exports[0].packet));
    assert.equal(a.before.state.saveRevision,2);assert.equal(a.after.state.saveRevision,2);assert.equal(a.after.state.saveWritePending,false);
    assert.deepEqual(get(r.input),get(exports[0].packet));
  });
  else check('abi:under-boundary-complete-packet-is-written-and-readmitted-byte-exact',()=>{
    assert.notEqual(exports[1].returned,0);assert.deepEqual(acks[1].args,[2,0,1]);
    assert.deepEqual(get(exports[1].packet),get(acks[1].after.store.bytes));assert.deepEqual(get(reads[0].input),get(exports[1].packet));
  });
}
