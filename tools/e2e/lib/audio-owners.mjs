import {installBrowserAudioObserver} from '../audio-api-observer.mjs';

// Existing forwarding API observer first; add read-only same-call ownership.
// Snapshot data never changes commands, clocks, game state or audio calls.
function installExitAudioOwnerObservation() {
  const start=AudioBufferSourceNode.prototype.start;
  AudioBufferSourceNode.prototype.start=function(...args){
    const trace=window.__audioApi,before=trace.events.length;
    const result=start.apply(this,args);
    for(const event of trace.events.slice(before).filter(row=>row.type==='start')){
      try{
        const state=window.game?.snapshot();
        event.observedSystemCounter=state?.systemClock.systemCounter??null;
        event.observedAdmittedSounds=state?.audio.events??null;
        event.observedMusicCurrentId=state?.music.currentId??null;
      }catch(error){event.ownerSnapshotError=String(error);}
    }
    return result;
  };
}

export function audioObservedBrowser(browser,saveApi){
  return {async newContext(options){
    const context=await browser.newContext(options);
    // One script gives explicit ordering; separate addInitScript calls do not.
    await context.addInitScript(`(${installBrowserAudioObserver.toString()})();\n(${installExitAudioOwnerObservation.toString()})();`);
    const close=context.close.bind(context);
    context.close=async()=>{
      try{
        const pages=context.pages();
        if(pages.length!==1)throw new Error('Expected exactly one canonical capture page');
        const api=await pages[0].evaluate(async()=>{
          await Promise.all(window.__audioApi.pending);
          return {truncated:window.__audioApi.truncated,contexts:window.__audioApi.contextTimes(),
            events:window.__audioApi.events.map(({pcmBase64,pcmPrefix,...event})=>event),
            scope:'Actual forwarding Web Audio calls; complete-buffer hashes retained, PCM payloads not copied. Same-call engine snapshots are read-only request-owner observations.'};
        });
        saveApi(api);
      }finally{await close();}
    };
    return context;
  }};
}
