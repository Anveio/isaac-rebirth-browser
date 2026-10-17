// Read only the real public Browser owners. Never dispatches a command or uses
// Native data as a Browser input. Passive observation only; capture never dispatches a game command.
export async function readBrowserControls(page) {
  return page.evaluate(() => {
    const state = window.game.snapshot();
    const trace = window.game.trace();
    const io = window.game.inputConfigs();
    const feedback = window.game.feedback();
    if (!state.keyboard || state.keyboard.bindings.length !== 24 ||
        state.keyboard.bindings.some(pair => pair.length !== 2) ||
        state.publicActions.held.length !== 24 || state.publicActions.triggered.length !== 24)
      throw new Error('Actual Browser keyboard24x2/poll owner is incomplete');
    if (!trace.enabled || trace.inputTraceTruncated || trace.backendStateTruncated)
      throw new Error('Actual Browser Controls trace is disabled or incomplete');
    if (io.truncated || feedback.truncated || io.events.length > 64 || feedback.events.length > 64 ||
        state.audio.events.length > 64 || state.audio.pending.length > 64)
      throw new Error('Actual Browser Controls evidence exceeds its complete bounded reader');
    //Read actual persisted text independently of the IO owner's successful-save
    //claim. An unavailable origin storage is an explicit observed failure.
    let storage;
    try { storage = {status:'read',key:io.key,text:window.localStorage.getItem(io.key)}; }
    catch(error) { storage = {status:'fail',key:io.key,error:String(error)}; }
    return {
      schema:1,status:'observed',screen:state.screen,tick:state.tick,
      systemCounter:state.systemClock.systemCounter,
      menu:state.menuPresentation,configuration:state.controlsMenu,
      keyboard:state.keyboard,retainedConfigurations:state.inputConfigs,
      physicalPressed:state.pressed,actions:state.publicActions,
      actualPlayer:{screen:state.screen,position:{x:state.player.x,y:state.player.y},velocity:{x:state.player.vx,y:state.player.vy}},
      audio:{events:state.audio.events,pending:state.audio.pending},
      bootInputConfigs:trace.bootInputConfigs,
      bootAudioPreferences:trace.bootAudioPreferences,
      bootFullscreenPreferences:trace.bootFullscreenPreferences,
      storage,io,feedback,
      scope:'Actual public Browser owners/actual origin storage read only; no Native inputs, hidden state setter, file equivalence or pixel/audio parity claim',
    };
  });
}

// Independent passive trusted DOM evidence. No engine commands or state writes.
export function installPhysicalObserver() {
  const events=[];let truncated=false;
  const record=event=>{
    if(events.length>=256){truncated=true;return;}
    events.push({type:event.type,isTrusted:event.isTrusted,code:event.code,key:event.key,
      repeat:event.repeat,ctrlKey:event.ctrlKey,altKey:event.altKey,metaKey:event.metaKey,
      shiftKey:event.shiftKey,target:event.target?.id??null});
  };
  window.addEventListener('keydown',record,{capture:true,passive:true});
  window.addEventListener('keyup',record,{capture:true,passive:true});
  Object.defineProperty(window,'__controlsPhysicalInputs',{value:()=>({events:structuredClone(events),truncated}),writable:false});
}
