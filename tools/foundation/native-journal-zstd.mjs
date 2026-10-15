// Exact Native journal transport v1: one unknown-size, no-dictionary Zstd
// frame, windowLog23 and checksum. Decoder still validates payload/checksum.
import {Transform} from 'node:stream';
import * as zlib from 'node:zlib';
const HEADER=Buffer.from('28b52ffd0468','hex');
export function journalFrameGuard(){
 let state='header',pending=Buffer.alloc(0),remaining=6,last=false;
 return new Transform({
  transform(chunk,_encoding,done){
   try{
    let offset=0;
    while(offset<chunk.length){
     if(state==='done') throw new Error('Trailing data or extra Native journal frame');
     const take=Math.min(remaining,chunk.length-offset);
     if(state==='header'||state==='block') pending=Buffer.concat([pending,chunk.subarray(offset,offset+take)]);
     offset+=take;remaining-=take;
     if(remaining) continue;
     if(state==='header'){
      if(!pending.equals(HEADER)) throw new Error('Unknown Native journal Zstd header/window');
      pending=Buffer.alloc(0);state='block';remaining=3;
     }else if(state==='block'){
      const block=pending.readUIntLE(0,3),kind=(block>>>1)&3,size=block>>>3;
      if(kind===3||size>131072) throw new Error('Invalid Native journal Zstd block');
      pending=Buffer.alloc(0);last=Boolean(block&1);state='payload';remaining=kind===1?1:size;
      if(!remaining){state=last?'checksum':'block';remaining=last?4:3;}
     }else if(state==='payload'){
      state=last?'checksum':'block';remaining=last?4:3;
     }else if(state==='checksum'){
      state='done';remaining=0;
     }
    }
    done(null,chunk);
   }catch(error){done(error);}
  },
  flush(done){done(state==='done'?undefined:new Error('Native journal Zstd frame is unfinished'));}
 });
}

export function nativeJournalZstdDecoders(){
 if(typeof zlib.createZstdDecompress!=='function'||typeof zlib.constants.ZSTD_d_windowLogMax!=='number')
  throw new Error('Zstd Native journals require Node24 built-in Zstd support');
 return [journalFrameGuard(),zlib.createZstdDecompress({chunkSize:65536,
  params:{[zlib.constants.ZSTD_d_windowLogMax]:23}})];
}
