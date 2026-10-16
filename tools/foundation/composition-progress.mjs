// Actual observed browser rows only. A committed prefix is never a full pass.
import {createHash} from 'node:crypto';
import {open,rename,writeFile} from 'node:fs/promises';
import {createReadStream} from 'node:fs';
import {createInterface} from 'node:readline';
import {createGunzip,gzipSync} from 'node:zlib';

export class BrowserProgress {
  static async create(directory) {
    const result=new BrowserProgress(directory);
    result.file=await open(result.path,'wx');
    try {await result.metadata('running');return result;}
    catch(error){await result.file.close();throw error;}
  }
  constructor(directory) {
    this.path=directory+'/browser-progress.jsonl.gz';this.pending=[];
    this.rawHash=createHash('sha256');this.archiveHash=createHash('sha256');
    this.rawBytes=0;this.archiveBytes=0;this.records=0;
  }
  async metadata(status) {
    const path=this.path+'.meta.json',temporary=path+'.part';
    await writeFile(temporary,JSON.stringify({schema:1,status,encoding:'gzip-members-jsonl',path:this.path,
      records:this.records,rawBytes:this.rawBytes,rawSha256:this.rawHash.copy().digest('hex'),
      committedArchiveBytes:this.archiveBytes,archiveSha256:this.archiveHash.copy().digest('hex'),
      scope:'Actual received browser observations committed in bounded batches; running/partial prefixes are incomplete, never game acceptance.'},null,2)+'\n');
    await rename(temporary,path);
  }
  async append(kind,value) {
    if(!this.file) {
      this.file=await open(this.path,'a');
      if((await this.file.stat()).size!==this.archiveBytes)throw Error('Committed browser journal changed before append');
    }
    this.pending.push(JSON.stringify({kind,value})+'\n');
    if(this.pending.length>=8)await this.flush();
  }
  async flush() {
    if(!this.pending.length)return;
    const raw=Buffer.from(this.pending.join('')),member=gzipSync(raw,{level:1,mtime:0});
    let written=0;
    while(written<member.length) {
      const result=await this.file.write(member,written,member.length-written);
      if(!result.bytesWritten)throw Error('Short committed browser journal write');
      written+=result.bytesWritten;
    }
    await this.file.sync();
    this.rawHash.update(raw);this.archiveHash.update(member);
    this.rawBytes+=raw.length;this.archiveBytes+=member.length;this.records+=this.pending.length;
    this.pending=[];await this.metadata('running');
  }
  async closeForRead() {
    try {await this.flush();}
    catch(error) {
      await writeFile(this.path+'.pending.jsonl',this.pending.join(''),{flag:'wx'});
      throw error;
    } finally {await this.file.close();this.file=null;}
  }
  async *values(kind) {
    if(this.file)throw Error('Close the committed browser journal before streaming evidence');
    const input=createReadStream(this.path),decoded=createGunzip();
    input.on('error',error=>decoded.destroy(error));input.pipe(decoded);
    const lines=createInterface({input:decoded,crlfDelay:Infinity});
    try {
      for await(const line of lines) {
        const row=JSON.parse(line);
        if(row.kind===kind)yield row.value;
      }
    } finally {lines.close();input.destroy();decoded.destroy();}
  }
  async close(status) {
    try {await this.flush();}
    finally {if(this.file){await this.file.close();this.file=null;}}
    await this.metadata(status);
  }
}
