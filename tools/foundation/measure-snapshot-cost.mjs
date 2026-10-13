// Chromium production-WASM tick/snapshot diagnostic; no original parity claim.
import {chromium} from '@playwright/test';
import {readdir,writeFile} from 'node:fs/promises';
import {join} from 'node:path';
import {browserLaunchOptions} from './browser.mjs';
import {startProductionServer} from './production-server.mjs';
const value=flag=>{const i=process.argv.indexOf(flag);return i<0?undefined:process.argv[i+1];};
const baseline=value('--baseline');if(!baseline)throw Error('Supply --baseline with the baseline production directory');
const candidate=value('--candidate')||'dist';const output=value('--output')||'.cache/snapshot-cost.json';
const browser=await chromium.launch(browserLaunchOptions());
const report={scope:'Production WASM in Chromium: one ordinary gameplay tick, state serialization, UTF8 decode and JSON parse; excludes renderer, trace storage and screenshot encoding.',results:[]};
try{
for(const [name,directory] of [['baseline',baseline],['candidate',candidate]]){
 const server=await startProductionServer({directory});const page=await browser.newPage();
 try{
  await page.goto(server.url+'/?manual=1');const files=await readdir(join(directory,'assets'));const wasm=files.find(f=>f.endsWith('.wasm'));
  const result=await page.evaluate(async({url})=>{
   const {instance}=await WebAssembly.instantiate(await(await fetch(url)).arrayBuffer());const w=instance.exports;const encode=new TextEncoder();const decode=new TextDecoder();
   const snapshot=()=>{const p=w.snapshot_ptr();const bytes=new Uint8Array(w.memory.buffer,p,w.snapshot_len());return JSON.parse(decode.decode(bytes));};
   const command=c=>{const bytes=encode.encode(JSON.stringify(c));const p=w.request_ptr(bytes.length);new Uint8Array(w.memory.buffer,p,bytes.length).set(bytes);if(w.command()!==0)throw Error('Command failed');return snapshot();};
   w.reset(745239347,1700000000);
   for(let i=0;i<4;i++){command({type:'key',code:'Enter',down:true});command({type:'key',code:'Enter',down:false});}
   command({type:'key',code:'KeyD',down:true});
   for(let i=0;i<100;i++)command({type:'tick',count:1});
   const state=snapshot();const serializedBytes=encode.encode(JSON.stringify(state)).length;
   const batches=[];for(let b=0;b<6;b++){const start=performance.now();for(let i=0;i<200;i++)command({type:'tick',count:1});batches.push((performance.now()-start)/200);}
   const sorted=[...batches].sort((a,b)=>a-b);
   return {screen:state.screen,serializedBytes,bootLifecycleBytes:state.bootLifecycle?encode.encode(JSON.stringify(state.bootLifecycle)).length:0,batchMillisecondsPerTickSnapshot:batches,medianMillisecondsPerTickSnapshot:(sorted[2]+sorted[3])/2};
  },{url:server.url+'/assets/'+wasm});
  report.results.push({name,production:server.build,...result});
 }finally{await page.close();await server.close();}
}
}finally{await browser.close();await writeFile(output,JSON.stringify(report,null,2)+'\n');}
console.log(JSON.stringify(report,null,2));
