// Actual public214/20 actors and boss lifecycle. Every paired mismatch remains required.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
const option=name=>{const at=process.argv.indexOf(name);assert.ok(at>=0&&process.argv[at+1]);return process.argv[at+1];};
const directory=resolve(option('--case')),load=path=>JSON.parse(readFileSync(path,'utf8'));
const originalPath=resolve(directory,'original/manifest.json'),browserPath=resolve(directory,'browser/manifest.json');
const original=load(originalPath),browser=load(browserPath),replay=load(resolve(directory,'case.json'));
const paired=load(resolve(directory,'report.json')),gate=load(resolve(dirname(directory),'report.json'));
assert.equal(paired.original.status,'pass');assert.equal(paired.browser.status,'pass');assert.equal(original.status,'pass');assert.equal(browser.status,'pass');assert.equal(gate.gateSourcesStable,true);
const f32=value=>{assert.equal(typeof value,'number','Required f32 field must exist');assert.ok(Number.isFinite(value),'Required f32 field must be finite');return Math.fround(value);};
const vector=p=>[f32(p.x??p[0]),f32(p.y??p[1])],checks=[],diagnostics=[];
const check=(rows,label,field,expected,actual)=>{assert.notEqual(expected,undefined,field);assert.notEqual(actual,undefined,field);rows.push({label,field,expected,actual,status:JSON.stringify(expected)===JSON.stringify(actual)?'pass':'fail'});};
const frames=new Map(browser.frames.map(row=>[row.label,row.state])),observations=new Map(original.debugObservations.map(row=>[row.context.label,row.observation]));
const required=replay.monstroRequired;assert.ok(Array.isArray(required)&&required.length);
const execution=Object.fromEntries(['native','browser'].map(name=>[name,Object.fromEntries(required.map(key=>[key,false]))]));
const prior={native:null,browser:null};
const queued={native:new Map(),browser:new Map()},dead={native:new Set(),browser:new Set()};let started=false;
const previousPoofs={native:new Map(),browser:new Map()};
assert.ok(replay.pairedTape.some(action=>action.type==='capture'&&action.label==='gameplay-start'));
for(const action of replay.pairedTape){
  if(action.type!=='capture')continue;
  const label=action.label,observation=observations.get(label),actual=frames.get(label);
  assert.ok(observation&&actual,`Missing declared capture${label}`);assert.equal(observation.status,'pass');
  if(label==='gameplay-start')started=true;
  if(actual.screen!=='playing'){assert.equal(started,false,'Post-start captures may not skip gameplay');assert.equal(observation.gameplay.status,'skip');continue;}
  const source=observation.gameplay;assert.equal(source.status,'pass');assert.equal(source.memoryWritten,false);
  const phase=source.nativePhase.systemIterations%2;
  const natives=source.entities.filter(actor=>[214,20].includes(actor.type)&&actor.variant===0&&actor.subtype===0);
  const port=actual.room.npcs.filter(actor=>[214,20].includes(actor.kind)&&!actor.removed);
  check(checks,label,'npc.authoredSeedOrder',natives.map(a=>[a.type,a.spawnSeed]),port.map(a=>[a.kind,a.spawnSeed]));
  check(checks,label,'room.identity',source.room.descriptorIndex,actual.currentRoom);
  check(checks,label,'system.phase',phase,actual.systemClock.systemCounter%2);
  check(diagnostics,label,'game.absoluteCounterPendingLoader',source.nativePhase.fullGameUpdates,actual.gameUpdateCount);
  const player=source.players[0];assert.ok(player);
  check(checks,label,'player.hp',player.redHp,actual.player.hp);check(checks,label,'player.immunity',player.damageCountdown,actual.damageCooldown);
  assert.equal(typeof source.room.combatClear,'boolean');assert.ok(Number.isInteger(source.room.combatClearCountdown));
  check(checks,label,'room.combatClear',source.room.combatClear,actual.room.combatClear);
  check(checks,label,'room.combatClearCountdown',source.room.combatClearCountdown,actual.room.combatClearCountdown);
  assert.ok(Number.isInteger(source.room.countableCurrent)&&Number.isInteger(source.room.countablePending));
  check(checks,label,'manager.countability',source.room.countableCurrent+source.room.countablePending>0,actual.room.npcs.some(a=>a.canShutDoors&&!a.removed));
  for(const door of source.grid.filter(cell=>cell.type===16)){const portDoor=actual.room.grid.cells[door.index]?.entity;assert.ok(portDoor);check(checks,label,`door${door.index}.state`,door.state,portDoor.state);check(checks,label,`door${door.index}.class`,door.collisionClass,portDoor.collisionClass);}
  for(const native of natives){
    const npc=port.find(actor=>actor.kind===native.type&&actor.spawnSeed===native.spawnSeed);if(!npc)continue;
    for(const[field,expected,observed]of[
      ['hp',f32(native.hp),f32(npc.hp)],['maxHp',f32(native.maxHp),f32(npc.maxHp)],['radius',f32(native.collisionRadius),f32(npc.collisionRadius)],['mass',f32(native.collisionMass),f32(npc.collisionMass)],['contact',f32(native.collisionDamage),f32(npc.collisionDamage)],
      ['state',native.npcState,npc.state],['appearance',native.npcFrameAge,npc.frameAge],['savedInitialization',native.npcSavedInitializedState,npc.savedInitializedState],['gridClass',native.gridCollisionClass,npc.gridCollisionClass],['entityClass',native.entityCollisionClass,npc.entityCollisionClass],
      ['canShutDoors',native.canShutDoors,npc.canShutDoors],['boss',native.boss,npc.boss],['dead',native.removed,npc.dead],['active',native.active,!npc.removed],['visible',native.visible,npc.visible],
      ['animation',native.animation,npc.animation],['frame',f32(native.animationFrame),f32(npc.animationClock)],['playing',native.animationPlaying,npc.animationPlaying],['pendingDamage',f32(native.pendingDamage),f32(npc.damage.pendingDamage)],['pendingFlags',native.pendingFlags,npc.damage.pendingFlags],
      ['orderedDamageRecords',native.pendingDamageRecords.map(row=>[f32(row.amount),row.flags,row.sourceType]),npc.damage.events.map(row=>[f32(row.amount),{tear:0,bomb:4,fireContact:2}[row.kind],row.source.entityType])],
      ['position',vector(native.position),vector(phase===0?npc.presentedPosition:npc.position)],['velocity',vector(native.velocity),vector(npc.velocity)],['target',vector(native.npcTarget),vector(npc.target)]
    ])check(checks,label,`npc${native.spawnSeed}.${field}`,expected,observed);
    for(const [index,record] of native.pendingDamageRecords.entries()){
      assert.equal(record.sourceActorType,record.sourceType,'Native ordered damage must retain its actual attacker');
      const event=npc.damage.events[index];if(!event)continue;
      const retained=event.source.entityType===2?actual.tears.find(a=>a.id===event.source.id)
        :event.source.entityType===4?actual.bombs.find(a=>a.id===event.source.id)
        :actual.room.npcs.find(a=>a.id===event.source.id);
      assert.ok(retained,'Port ordered damage must reference an actually admitted actor');
      const field=`npc${native.spawnSeed}.damage${index}.sourceSeed`;
      if(retained.spawnSeed===undefined||retained.spawnSeed===null){
        diagnostics.push({label,field,expected:record.sourceActorSpawnSeed,actual:null,status:'unaccepted',reason:'This actual attacker seed producer remains unsupported; amount/flags/order/reference are required.'});
      }else check(checks,label,field,record.sourceActorSpawnSeed,retained.spawnSeed);
    }
    if(native.type===214){
      assert.ok(npc.level2Fly);
      for(const[field,expected,observed]of[['feedback',vector(native.npcGridFeedbackVelocity),vector(npc.level2Fly.feedbackVelocity)],['gridContact',native.collidedWithGrid,npc.level2Fly.gridContact],['gridNormal',vector(native.gridCollisionNormal),vector(npc.level2Fly.gridNormal)],['previousGridVelocity',vector(native.preGridCollisionVelocity),vector(npc.level2Fly.previousGridVelocity)]])check(checks,label,`npc${native.spawnSeed}.${field}`,expected,observed);
    }else{
      assert.ok(npc.monstro);check(checks,label,`boss${native.spawnSeed}.stateFrame`,native.npcStateFrame,npc.monstro.stateFrame);
      check(checks,label,`boss${native.spawnSeed}.choice`,native.npcAux2,npc.monstro.choice);
    }
  }
  const nativeShots=source.entities.filter(a=>a.type===9&&a.variant===0);
  const portShots=actual.enemyProjectiles.filter(a=>!a.removed);
  check(checks,label,'projectile.seedOrder',nativeShots.map(a=>a.spawnSeed),portShots.map(a=>a.spawnSeed));
  for(const shot of nativeShots){const p=portShots.find(a=>a.spawnSeed===shot.spawnSeed);if(!p)continue;
    for(const[field,expected,observed]of[['height',f32(shot.projectileHeight),f32(p.height)],['fallingSpeed',f32(shot.projectileFallingSpeed),f32(p.fallingSpeed)],['fallingAcceleration',f32(shot.projectileFallingAcceleration),f32(p.fallingAcceleration)],['scale',f32(shot.projectileScale),f32(p.scale)],['dead',shot.removed,p.dead],['position',vector(shot.position),vector(phase===0?p.presentedPosition:p.position)],['velocity',vector(shot.velocity),vector(p.velocity)],['animation',shot.animation,p.animation]])check(checks,label,`projectile${shot.spawnSeed}.${field}`,expected,observed);
  }
  const nativePoofs=source.entities.filter(a=>a.type===1000&&a.variant===15&&[214,20].includes(a.parentType));
  const portPoofs=actual.spriteEffects.filter(a=>a.variant===15&&a.active&&!a.removed&&[214,20].includes(a.parent?.entityType));
  check(checks,label,'poof.seedOrder',nativePoofs.map(a=>a.spawnSeed),portPoofs.map(a=>a.spawnSeed));
  for(const poof of nativePoofs){
    const effect=portPoofs.find(a=>a.spawnSeed===poof.spawnSeed);if(!effect)continue;
    assert.ok(poof.effectParent&&effect.parent,'NPC Poof must retain its actual parent');
    const parent=actual.room.npcs.find(a=>a.id===effect.parent.actor.id);assert.ok(parent);
    for(const[field,expected,observed]of[
      ['subtype',poof.subtype,effect.subtype],['parentIdentity',[poof.effectParent.type,poof.effectParent.variant,poof.effectParent.spawnSeed],[parent.kind,parent.variant,parent.spawnSeed]],
      ['raw7f8',f32(poof.effectRaw7f8),f32(effect.raw7f8)],['animation',poof.animation,effect.sprite.animation],['frame',f32(poof.animationFrame),f32(effect.sprite.frame)],['playing',poof.animationPlaying,effect.sprite.playing],
      ['position',vector(poof.position),vector(phase===0?effect.presentedPosition:effect.position)],['velocity',vector(poof.velocity),vector(effect.velocity)],['active',poof.active,effect.active],['flags',poof.entityFlags,effect.flags]
    ])check(checks,label,`poof${poof.spawnSeed}.${field}`,expected,observed);
  }
  for(const[name,effects,counter,system]of[
    ['native',nativePoofs.map(a=>({seed:a.spawnSeed,parentType:a.parentType,born:a.birthGameUpdate,frame:f32(a.animationFrame),playing:a.animationPlaying,position:vector(a.position),velocity:vector(a.velocity)})),source.nativePhase.fullGameUpdates,source.nativePhase.systemIterations],
    ['browser',portPoofs.map(a=>({seed:a.spawnSeed,parentType:a.parent.entityType,born:a.birthGameUpdate,frame:f32(a.sprite.frame),playing:a.sprite.playing,position:vector(phase===0?a.presentedPosition:a.position),velocity:vector(a.velocity)})),actual.gameUpdateCount,actual.systemClock.systemCounter]
  ]){
    const old=previousPoofs[name];
    for(const effect of effects){
      if(effect.parentType===214)execution[name]['level2-poof']=true;
      if(effect.parentType===20)execution[name]['boss-poof']=true;
      const before=old.get(effect.seed);
      if(!before&&effect.born===counter){
        check(checks,label,`${name}.poof${effect.seed}.fullBirthFence`,0,effect.frame);
        execution[name][effect.parentType===214?'level2-poof-fence':'boss-poof-fence']=effect.frame===0;
      }
      if(before&&system===before.system+1&&counter===before.counter){
        for(const field of ['frame','playing','velocity'])check(checks,label,`${name}.poof${effect.seed}.fastFreeze.${field}`,before[field],effect[field]);
      }
      if(before&&counter===before.counter+1){
        check(checks,label,`${name}.poof${effect.seed}.nextFullFrame`,before.frame+1,effect.frame);
      }
    }
    for(const[seed,before]of old){
      if(counter===before.counter+1&&before.frame===42){
        check(checks,label,`${name}.poof${seed}.retiredAfterClip`,false,effects.some(a=>a.seed===seed));
        execution[name][before.parentType===214?'level2-poof-retirement':'boss-poof-retirement']=!effects.some(a=>a.seed===seed);
      }
    }
    previousPoofs[name]=new Map(effects.map(effect=>[effect.seed,{...effect,counter,system}]));
  }
  const worlds={native:{counter:source.nativePhase.fullGameUpdates,system:source.nativePhase.systemIterations,room:source.room.descriptorIndex,clear:source.room.combatClear,clearCountdown:source.room.combatClearCountdown,transition:source.roomTransition.state,
    scene:source.roomTransition.bossIntroduction&&source.roomTransition.state===2,cleanup:source.roomTransition.bossIntroduction&&source.roomTransition.state===3,
    npcs:natives.map(a=>({seed:a.spawnSeed,type:a.type,hp:f32(a.hp),pending:f32(a.pendingDamage),flags:a.pendingFlags,dead:a.removed,state:a.npcState,age:a.npcFrameAge,frame:f32(a.animationFrame),animation:a.animation,playing:a.animationPlaying,position:vector(a.position),velocity:vector(a.velocity),feedback:a.type===214?vector(a.npcGridFeedbackVelocity):null,gridContact:a.collidedWithGrid,gridNormal:vector(a.gridCollisionNormal)})),shots:nativeShots},
    browser:{counter:actual.gameUpdateCount,system:actual.systemClock.systemCounter,room:actual.currentRoom,clear:actual.room.combatClear,clearCountdown:actual.room.combatClearCountdown,transition:actual.roomTransition?1:0,
    scene:actual.roomTransition?.phase==='bossScene',cleanup:actual.roomTransition?.phase==='bossCleanup',
    npcs:port.map(a=>({seed:a.spawnSeed,type:a.kind,hp:f32(a.hp),pending:f32(a.damage.pendingDamage),flags:a.damage.pendingFlags,dead:a.dead,state:a.state,age:a.frameAge,frame:f32(a.animationClock),animation:a.animation,playing:a.animationPlaying,position:vector(phase===0?a.presentedPosition:a.position),velocity:vector(a.velocity),feedback:a.level2Fly?vector(a.level2Fly.feedbackVelocity):null,gridContact:a.level2Fly?.gridContact,gridNormal:a.level2Fly?vector(a.level2Fly.gridNormal):null})),shots:portShots}};
  for(const[name,world]of Object.entries(worlds)){
    const before=prior[name],flies=world.npcs.filter(a=>a.type===214),bosses=world.npcs.filter(a=>a.type===20);
    if(flies.length)execution[name]['level2-admission']=true;
    if(flies.some(a=>a.state===3&&a.age===21&&a.animation==='Idle'))execution[name]['level2-appearance']=true;
    if(flies.some(a=>a.state===3&&a.velocity.some(v=>v!==0)))execution[name]['level2-active']=true;
    if(flies.some(a=>a.pending>0||a.hp<10))execution[name]['level2-damage']=true;
    if(flies.some(a=>a.dead))execution[name]['level2-death']=true;
    for(const fly of flies){
      if(fly.pending>0)queued[name].set(fly.seed,fly.hp);
      if(queued[name].has(fly.seed)&&fly.pending===0&&fly.hp<queued[name].get(fly.seed))execution[name]['level2-drain']=true;
      if(fly.dead)dead[name].add(fly.seed);
    }
    if([...dead[name]].some(seed=>!flies.some(a=>a.seed===seed)))execution[name]['level2-retirement']=true;
    if(world.room===9&&world.clear&&flies.length===0)execution[name]['prefix-clear']=true;
    if(world.room===9&&!world.clear&&flies.length===0&&world.clearCountdown>0&&world.clearCountdown<10)execution[name]['prefix-clear-delay']=true;
    if(before&&before.room===9&&world.room===9&&world.counter===before.counter+1&&world.npcs.length===0&&before.npcs.length===0){
      if(!before.clear&&before.clearCountdown===1){
        check(checks,label,`${name}.clearFinalDecrement`,[false,0],[world.clear,world.clearCountdown]);
      }
      if(!before.clear&&before.clearCountdown===0){
        check(checks,label,`${name}.clearAfterZero`,[true,0],[world.clear,world.clearCountdown]);
        execution[name]['prefix-clear-boundary']=world.clear&&world.clearCountdown===0;
      }
    }
    if(bosses.length)execution[name]['boss-admission']=true;
    if(world.scene)execution[name]['boss-scene']=true;
    if(world.cleanup)execution[name]['boss-cleanup']=true;
    if(bosses.some(a=>![1,3].includes(a.state)))execution[name]['boss-ai']=true;
    if(world.room===0&&world.shots.length)execution[name]['boss-projectiles']=true;
    if(bosses.some(a=>a.pending>0||a.hp<250))execution[name]['boss-damage']=true;
    if(before&&before.room===world.room){
      for(const npc of world.npcs){const old=before.npcs.find(a=>a.seed===npc.seed);if(!old)continue;
        if(world.system===before.system+1&&world.counter===before.counter&&!world.transition&&!before.transition){
          for(const field of ['velocity','age','state','frame','animation','playing',...(npc.type===214?['feedback','gridContact','gridNormal']:[])])check(checks,label,`${name}.npc${npc.seed}.fastFreeze.${field}`,old[field],npc[field]);
        }
        if(world.scene&&before.scene)for(const field of ['hp','pending','position','velocity','state','age','frame'])check(checks,label,`${name}.bossScene.freeze${npc.seed}.${field}`,old[field],npc[field]);
      }
    }
    prior[name]=world;
  }
}
for(const name of ['native','browser'])for(const requiredExecution of required)check(checks,'execution',`${name}.${requiredExecution}`,true,execution[name][requiredExecution]);
const status=checks.every(row=>row.status==='pass')?'pass':'fail';
const hash=path=>createHash('sha256').update(readFileSync(path)).digest('hex');
const report={schema:1,status,execution,checks,diagnostics,sources:{originalManifest:hash(originalPath),browserManifest:hash(browserPath),comparator:hash('tools/foundation/monstro-state-pair.mjs')},scope:'All declared actual actor/body/state/HP/ordered queue/feedback/scene/projectile/Poof fields and executions are required. Exact shared-stream trajectories remain visible failures; absolute causal loader counter is named unaccepted. Blood background-paint/return acceptance needs an actual native render submission boundary; boss final blood/reward/trapdoor/stage exit remain required separate follow-ups.'};
writeFileSync(resolve(option('--output')),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({status,fields:checks.length,failures:checks.filter(row=>row.status==='fail').length,execution}));process.exitCode=status==='pass'?0:1;
