// Two finite public input recipes. The action list and expected public command
// list expand together; no game state or observed result selects an input.
const CASES = {
  "under": {
    "id": "save-under-5003-prefix-through-continue-v1",
    "seed": "T1PK6Y0W",
    "seedValue": 2184569263,
    "inputMode": "under-continue-prefix",
    "movementLegs": [
      {
        "name": "under",
        "cycles": 375,
        "ticksPerSide": 40,
        "movementTicks": 60000,
        "keys": [
          "d",
          "s",
          "a",
          "w"
        ],
        "coastTicks": 24,
        "checkpointEveryCycles": 25
      }
    ],
    "assertions": {
      "under": "Actualfootprinttrue/variant7count>4096; everyothergenericBlood7 retained; actualwholepacket<=4194304; everyactualstain/counter/floor/inventory saved; actualreadmit+Continuefreshtransientpaint/control+progress",
      "scope": "ExactoldfirstbankprefixONLYthroughunder-continued-playing; NOovermovement/Quit/capacity/lateroldtailcoverage",
      "noAdaptiveInputs": true
    }
  },
  "over": {
    "id": "save-capacity-fresh-normal-over-aw160000-v1",
    "seed": "T1PK6Y0W",
    "seedValue": 2184569263,
    "inputMode": "fresh-normal-over",
    "movementLegs": [
      {
        "name": "over",
        "keys": [
          "a",
          "w"
        ],
        "simultaneouslyHeld": true,
        "movementTicks": 160000,
        "chunks": 4000,
        "ticksPerChunk": 40,
        "coastTicks": 24,
        "summaryEveryChunks": 100,
        "liveRoomGuardAfterEveryChunk": true,
        "postContinueMovement": "a8"
      }
    ],
    "assertions": {
      "over": "Actual footprint=true/variant7 count>4096; preserve every actualstain; actualwholearraystructuralminimum>4194304BEFOREQuit; exactlyonecapacityfailedwrite; initialbootstrapoldstoreunchanged/readmitted; Continueandlaterpublicprogress",
      "initialMode": "Freshnormaladmission; no underQuit/Continuebeforehold",
      "noAdaptiveInputs": true
    }
  }
};
const BOOT = {
  "bootEpochSeed": 1700000000,
  "bootAudioPreferences": {
    "sfxVolume": 0,
    "musicVolume": 0
  },
  "bootFullscreenPreferences": {
    "fullscreen": false
  },
  "bootInputConfigs": {
    "devices": []
  },
  "saveHost": {
    "stored": null,
    "failedWrites": []
  }
};

export function capacityPublicInputs(role) {
  if (!Object.hasOwn(CASES,role)) throw new Error('Unknown capacity recipe: '+role);
  const {id,seed,seedValue,inputMode,movementLegs,assertions}=structuredClone(CASES[role]);
  const actions=[],commands=[],checkpoints=[],leg=movementLegs[0];
  const capture=label=>{
    actions.push({type:'capture',label});
    checkpoints.push({label,actionIndex:actions.length-1,afterCommands:commands.length});
  };
  const step=(frames,extra={})=>{
    actions.push({type:'step',frames,...extra});commands.push({type:'tick',count:frames});
  };
  const key=(value,down,text=false)=>{
    actions.push({type:'key',key:value,down});
    // The actual seed-entry keyboard owner emits uppercase text on keydown and
    // no command on keyup. DOM actions still contain both physical edges.
    if(text){if(down)commands.push({type:'text',text:value.toUpperCase()});return;}
    const named={Return:'Enter',Down:'ArrowDown',Escape:'Escape',Tab:'Tab'};
    const code=named[value]??(/^[a-z]$/i.test(value)?'Key'+value.toUpperCase():'Digit'+value);
    commands.push({type:'key',code,down});
  };
  const tap=(value,held,released,text=false)=>{
    key(value,true,text);step(held);key(value,false,text);step(released);
  };
  const hold=(value,frames)=>{key(value,true);step(frames);key(value,false);};
  const edge=(name,frames,captures)=>{
    key('Return',true);
    for(let frame=1;frame<=frames;frame++){
      step(1);
      if(captures.includes(frame))capture(role+'-'+name+'-edge-'+String(frame).padStart(2,'0'));
      if(frame===3)key('Return',false);
    }
  };

  capture('title-ready');step(63);capture('title');
  for(const label of ['profile','main','character']){tap('Return',12,72);capture(label);}
  tap('Tab',12,108);
  for(const value of seed.toLowerCase())tap(value,12,6,true);
  capture('seed-entry');tap('Return',12,72);capture('character-seeded');
  tap('Return',12,300);capture('gameplay-start');step(57);capture('save-run-ready');

  capture(role+'-walk-start');
  if(role==='under'){
    for(let cycle=1;cycle<=leg.cycles;cycle++){
      for(const value of leg.keys)hold(value,leg.ticksPerSide);
      if(cycle%leg.checkpointEveryCycles===0)capture('under-walk-'+String(cycle).padStart(4,'0'));
    }
  }else{
    for(const value of leg.keys)key(value,true);
    for(let chunk=1;chunk<=leg.chunks;chunk++){
      step(leg.ticksPerChunk,{liveRoomGuard:true,holdChunk:chunk});
      if(chunk%leg.summaryEveryChunks===0)capture('over-walk-'+String(chunk).padStart(4,'0'));
    }
    for(const value of leg.keys)key(value,false);
  }
  step(leg.coastTicks);capture(role+'-pre-pause');
  tap('Escape',1,1);step(28);capture(role+'-pause-idle');
  tap('Down',1,1);step(2);capture(role+'-before-quit');
  edge('quit',36,[1,2,3,12,24,36]);step(36);capture(role+'-main-settled');
  step(2);capture(role+'-before-continue');
  edge('continue',72,[1,2,3,24,48,72]);step(300);capture(role+'-continued-playing');
  if(role==='over'){
    step(120);capture('post-capacity-idle');hold('a',8);step(24);capture('post-capacity-movement');
  }
  return {
    bank:{id,seed,seedValue,inputMode,actions,checkpoints,movementLegs,assertions},
    tape:{initialSeed:seedValue,...structuredClone(BOOT),commands}
  };
}
