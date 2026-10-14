// Actual public214/20 actors and boss lifecycle. Every paired mismatch remains required.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { dirname, resolve } from 'node:path';
const option=name=>{const at=process.argv.indexOf(name);assert.ok(at>=0&&process.argv[at+1]);return process.argv[at+1];};
const directory=resolve(option('--case'));
// Hash decoded bytes identically whether lossless archived or still raw.
const artifactBytes=path=>existsSync(path)?readFileSync(path):gunzipSync(readFileSync(path+'.gz'));
const load=path=>JSON.parse(artifactBytes(path).toString('utf8'));
const originalPath=resolve(directory,'original/manifest.json'),browserPath=resolve(directory,'browser/manifest.json');
const checks=[],diagnostics=[];let execution={};
const gateBinaryIdentity={expected:'724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13',observed:null,status:'not-read'};
const coverage={native:{finalHeartFullBirthFence:'pending-unobserved'},browser:{finalHeartFullBirthFence:'pending-unobserved'}};
const run=()=>{
const original=load(originalPath),browser=load(browserPath),replay=load(resolve(directory,'case.json'));
const paired=load(resolve(directory,'report.json')),gate=load(resolve(dirname(directory),'report.json'));
// paired-game.mjs owns sourceBinarySha256; validate observed identity before
// serializing it as proof. Missing/wrong fields stay explicit failures.
gateBinaryIdentity.observed=typeof gate.sourceBinarySha256==='string'?gate.sourceBinarySha256:null;
gateBinaryIdentity.status=gateBinaryIdentity.observed===gateBinaryIdentity.expected?'pass':'fail';
assert.equal(typeof gate.sourceBinarySha256,'string','Paired gate must expose its actual sourceBinarySha256');
assert.equal(gate.sourceBinarySha256,gateBinaryIdentity.expected,'Paired gate original binary identity must match the observer');
assert.equal(paired.original.status,'pass');assert.equal(paired.browser.status,'pass');assert.equal(original.status,'pass');assert.equal(browser.status,'pass');assert.equal(gate.gateSourcesStable,true);
const f32=value=>{assert.equal(typeof value,'number','Required f32 field must exist');assert.ok(Number.isFinite(value),'Required f32 field must be finite');return Math.fround(value);};
const vector=p=>[f32(p.x??p[0]),f32(p.y??p[1])];
const check=(rows,label,field,expected,actual)=>{assert.notEqual(expected,undefined,field);assert.notEqual(actual,undefined,field);rows.push({label,field,expected,actual,status:JSON.stringify(expected)===JSON.stringify(actual)?'pass':'fail'});};
const frames=new Map(browser.frames.map(row=>[row.label,row.state])),observations=new Map(original.debugObservations.map(row=>[row.context.label,row.observation]));
const required=replay.monstroRequired;assert.ok(Array.isArray(required)&&required.length);
const bossClear=replay.bossClear;
if(bossClear){assert.equal(replay.seedValue,978);assert.equal(replay.seed,'B9112M6H');assert.equal(replay.scenario.randomized,false);assert.equal(bossClear.randomized,false);assert.equal(bossClear.firingBudgetFixedBeforeGrading,true);}
execution=Object.fromEntries(['native','browser'].map(name=>[name,Object.fromEntries(required.map(key=>[key,false]))]));
const prior={native:null,browser:null};
const queued={native:new Map(),browser:new Map()},dead={native:new Set(),browser:new Set()};let started=false;
const previousPoofs={native:new Map(),browser:new Map()};
const bossDeaths={native:new Set(),browser:new Set()},previousHearts={native:new Map(),browser:new Map()};
assert.ok(replay.pairedTape.some(action=>action.type==='capture'&&action.label==='gameplay-start'));
for(const action of replay.pairedTape){
  if(action.type!=='capture')continue;
  const label=action.label,observation=observations.get(label),actual=frames.get(label);
  assert.ok(observation&&actual,`Missing declared capture${label}`);assert.equal(observation.status,'pass');
  if(label==='gameplay-start')started=true;
  if(actual.screen!=='playing'){assert.equal(started,false,'Post-start captures may not skip gameplay');assert.equal(observation.gameplay.status,'skip');continue;}
  const source=observation.gameplay;assert.equal(source.status,'pass');assert.equal(source.memoryWritten,false);assert.equal(source.nativeBinarySha256,'724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13');
  const phase=source.nativePhase.systemIterations%2;
  // Preserve the historical214/20 contract independently of the new route.
  const natives=source.entities.filter(actor=>[214,20].includes(actor.type)&&actor.variant===0&&actor.subtype===0);
  const port=actual.room.npcs.filter(actor=>[214,20].includes(actor.kind)&&!actor.removed);
  // The observer sets npcState only after checking the actual NPC vtable.
  // Full concrete manager membership, not a type filter, owns new occupancy.
  const nativeConcreteNpcs=source.entities.filter(actor=>Object.hasOwn(actor,'npcState'));
  const portConcreteNpcs=actual.room.npcs.filter(actor=>!actor.removed);
  const concreteWorlds={
    native:nativeConcreteNpcs.map(a=>({type:a.type,variant:a.variant,subtype:a.subtype,seed:a.spawnSeed,dead:a.removed,animation:a.animation,canShutDoors:a.canShutDoors,boss:a.boss})),
    browser:portConcreteNpcs.map(a=>({type:a.kind,variant:a.variant,subtype:a.subtype,seed:a.spawnSeed,dead:a.dead,animation:a.animation,canShutDoors:a.canShutDoors,boss:a.boss}))
  };
  const concreteIdentity=a=>[a.type,a.variant,a.subtype,a.seed];
  const eligible=a=>a.canShutDoors||a.boss;
  if(bossClear){
    for(const rows of Object.values(concreteWorlds))for(const npc of rows){assert.equal(typeof npc.canShutDoors,'boolean');assert.equal(typeof npc.boss,'boolean');assert.ok(Number.isInteger(npc.seed)&&npc.seed>0);}
    check(checks,label,'contrivedNpc.fullConcreteSeedOrder',concreteWorlds.native.map(concreteIdentity),concreteWorlds.browser.map(concreteIdentity));
    check(checks,label,'contrivedNpc.eligibleSeedOrder',concreteWorlds.native.filter(eligible).map(concreteIdentity),concreteWorlds.browser.filter(eligible).map(concreteIdentity));
  }
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
    }else if(native.type===20){
      assert.ok(npc.monstro);check(checks,label,`boss${native.spawnSeed}.stateFrame`,native.npcStateFrame,npc.monstro.stateFrame);
      check(checks,label,`boss${native.spawnSeed}.choice`,native.npcAux2,npc.monstro.choice);
      if(bossClear){
        assert.ok(npc.monstro.actorRng,'Final callback must own the actual persistent Entity RNG30');
        check(checks,label,`boss${native.spawnSeed}.actorRngState`,native.actorRngState,npc.monstro.actorRng.state);
        check(checks,label,`boss${native.spawnSeed}.entityFlags`,native.entityFlags,npc.monstro.entityFlags);
      }
    }
  }
  const nativeShots=source.entities.filter(a=>a.type===9&&a.variant===0);
  const portShots=actual.enemyProjectiles.filter(a=>!a.removed);
  check(checks,label,'projectile.seedOrder',nativeShots.map(a=>a.spawnSeed),portShots.map(a=>a.spawnSeed));
  for(const shot of nativeShots){const p=portShots.find(a=>a.spawnSeed===shot.spawnSeed);if(!p)continue;
    for(const[field,expected,observed]of[['height',f32(shot.projectileHeight),f32(p.height)],['fallingSpeed',f32(shot.projectileFallingSpeed),f32(p.fallingSpeed)],['fallingAcceleration',f32(shot.projectileFallingAcceleration),f32(p.fallingAcceleration)],['scale',f32(shot.projectileScale),f32(p.scale)],['dead',shot.removed,p.dead],['position',vector(shot.position),vector(phase===0?p.presentedPosition:p.position)],['velocity',vector(shot.velocity),vector(p.velocity)],['animation',shot.animation,p.animation]])check(checks,label,`projectile${shot.spawnSeed}.${field}`,expected,observed);
  }
  const nativePoofs=source.entities.filter(a=>a.type===1000&&a.variant===15&&[214,20].includes(a.parentType));
  const portPoofs=actual.spriteEffects.filter(a=>a.variant===15&&a.active&&!a.removed&&[214,20].includes(a.spawnerType));
  check(checks,label,'poof.seedOrder',nativePoofs.map(a=>a.spawnSeed),portPoofs.map(a=>a.spawnSeed));
  for(const poof of nativePoofs){
    const effect=portPoofs.find(a=>a.spawnSeed===poof.spawnSeed);if(!effect)continue;
    assert.ok(Object.hasOwn(poof,'effectParent')&&Object.hasOwn(effect,'parent'),'Both games must expose the actual linked-parent field');
    assert.ok(Number.isInteger(poof.parentType)&&Number.isInteger(poof.parentVariant)&&Number.isInteger(effect.spawnerType)&&Number.isInteger(effect.spawnerVariant),'Both games must expose actual cached spawner metadata');
    check(checks,label,`poof${poof.spawnSeed}.nativeLinkedParent`,null,poof.effectParent);
    check(checks,label,`poof${poof.spawnSeed}.browserLinkedParent`,null,effect.parent);
    for(const[field,expected,observed]of[
      ['subtype',poof.subtype,effect.subtype],['spawnerMetadata',[poof.parentType,poof.parentVariant],[effect.spawnerType,effect.spawnerVariant]],
      ['raw7f8',f32(poof.effectRaw7f8),f32(effect.raw7f8)],['animation',poof.animation,effect.sprite.animation],['frame',f32(poof.animationFrame),f32(effect.sprite.frame)],['playing',poof.animationPlaying,effect.sprite.playing],
      ['position',vector(poof.position),vector(phase===0?effect.presentedPosition:effect.position)],['velocity',vector(poof.velocity),vector(effect.velocity)],['active',poof.active,effect.active],['flags',poof.entityFlags,effect.flags]
    ])check(checks,label,`poof${poof.spawnSeed}.${field}`,expected,observed);
  }
  for(const[name,effects,counter,system]of[
    ['native',nativePoofs.map(a=>({seed:a.spawnSeed,parentType:a.parentType,born:a.birthGameUpdate,frame:f32(a.animationFrame),playing:a.animationPlaying,position:vector(a.position),velocity:vector(a.velocity)})),source.nativePhase.fullGameUpdates,source.nativePhase.systemIterations],
    ['browser',portPoofs.map(a=>({seed:a.spawnSeed,parentType:a.spawnerType,born:a.birthGameUpdate,frame:f32(a.sprite.frame),playing:a.sprite.playing,position:vector(phase===0?a.presentedPosition:a.position),velocity:vector(a.velocity)})),actual.gameUpdateCount,actual.systemClock.systemCounter]
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
  if(bossClear){
    const achievement33=actual.profiles[actual.profile-1]?.achievements[33];
    assert.equal(typeof source.everythingIsTerrible,'boolean','Actual System51 profile flag must be observed');
    assert.equal(typeof achievement33,'boolean','Port must expose the sole selected profile achievement33');
    check(checks,label,'profile.achievement33',source.everythingIsTerrible,achievement33);
    check(checks,label,'native.profile.freshAchievement33',false,source.everythingIsTerrible);
    check(checks,label,'browser.profile.freshAchievement33',false,achievement33);
    assert.ok(Number.isInteger(source.room.bossCount)&&Number.isInteger(actual.room.bossCount),'Room2490 must be observed separately from NPC presence');
    check(checks,label,'room.bossCount',source.room.bossCount,actual.room.bossCount);
    const nativeHearts=source.entities.filter(a=>a.type===5&&a.variant===10&&a.spawnerType===20&&a.spawnerVariant===0);
    const portHearts=actual.room.pickups.filter(a=>a.variant===10&&!a.removed&&a.spawnerType===20&&a.spawnerVariant===0);
    check(checks,label,'bossFinalHeart.seedOrder',nativeHearts.map(a=>a.spawnSeed),portHearts.map(a=>a.spawnSeed));
    for(const heart of nativeHearts){
      const pickup=portHearts.find(a=>a.spawnSeed===heart.spawnSeed);if(!pickup)continue;
      assert.ok(Object.hasOwn(heart,'pickupParent')&&Object.hasOwn(pickup,'parent'),'Cached caller metadata may not stand in for actual Parent498');
      const identity=actual.actorScheduler.identities.find(row=>row.actor.kind==='pickup'&&row.actor.id===pickup.id);
      assert.ok(identity,'Final Heart must be admitted through the actual mixed actor scheduler');
      for(const[field,expected,observed]of[
        ['subtype',heart.subtype,pickup.subtype],['cachedSpawner',[heart.spawnerType,heart.spawnerVariant],[pickup.spawnerType,pickup.spawnerVariant]],
        ['sequenceId',heart.sequenceId,identity.sequenceId],['animation',heart.animation,pickup.animation],['frame',f32(heart.animationFrame),f32(pickup.animationFrame)],['playing',heart.animationPlaying,pickup.animationPlaying],
        ['position',vector(heart.position),vector(phase===0?pickup.presentedPosition:pickup.position)],['velocity',vector(heart.velocity),vector(pickup.velocity)],
        ['radius',f32(heart.collisionRadius),f32(pickup.collisionRadius)],['mass',f32(heart.collisionMass),f32(pickup.collisionMass)]
      ])check(checks,label,`bossFinalHeart${heart.spawnSeed}.${field}`,expected,observed);
      check(checks,label,`bossFinalHeart${heart.spawnSeed}.nativeLinkedParent`,null,heart.pickupParent);
      check(checks,label,`bossFinalHeart${heart.spawnSeed}.browserLinkedParent`,null,pickup.parent);
    }
    for(const[name,hearts,counter,system,bossCount,worldNpcs,roomClear,room]of[
      ['native',nativeHearts.map(a=>({seed:a.spawnSeed,subtype:a.subtype,born:a.birthGameUpdate,frame:f32(a.animationFrame),animation:a.animation,playing:a.animationPlaying,position:vector(a.position),velocity:vector(a.velocity),parent:a.pickupParent})),source.nativePhase.fullGameUpdates,source.nativePhase.systemIterations,source.room.bossCount,nativeConcreteNpcs,source.room.combatClear,source.room.descriptorIndex],
      ['browser',portHearts.map(a=>({seed:a.spawnSeed,subtype:a.subtype,born:a.birthGameUpdate,frame:f32(a.animationFrame),animation:a.animation,playing:a.animationPlaying,position:vector(phase===0?a.presentedPosition:a.position),velocity:vector(a.velocity),parent:a.parent})),actual.gameUpdateCount,actual.systemClock.systemCounter,actual.room.bossCount,portConcreteNpcs,actual.room.combatClear,actual.currentRoom]
    ]){
      if(room===0&&worldNpcs.some(a=>(a.type??a.kind)===20&&a.spawnSeed===bossClear.bossSeed)){
        check(checks,label,`${name}.contrivedBoss.admittedCount`,1,bossCount);
        execution[name]['boss-count-admitted']=bossCount===1;
      }
      for(const heart of hearts){
        assert.ok(Number.isInteger(heart.born)&&Number.isInteger(heart.seed)&&heart.seed>0);
        execution[name]['boss-final-heart']=room===0&&heart.parent===null&&[1,2,3,4,5,6].includes(heart.subtype);
        const before=previousHearts[name].get(heart.seed);
        if(!before&&heart.born===counter){
          check(checks,label,`${name}.bossFinalHeart${heart.seed}.fullBirthFence`,0,heart.frame);
          coverage[name].finalHeartFullBirthFence=heart.frame===0?'pass-observed':'fail-observed';
        }
        if(before&&system===before.system+1&&counter===before.counter){
          for(const field of ['frame','animation','playing','velocity'])check(checks,label,`${name}.bossFinalHeart${heart.seed}.fastFreeze.${field}`,before[field],heart[field]);
        }
      }
      previousHearts[name]=new Map(hearts.map(heart=>[heart.seed,{...heart,counter,system}]));
      if(room===0&&bossDeaths[name].has(bossClear.bossSeed)&&!worldNpcs.some(a=>(a.type??a.kind)===20&&a.spawnSeed===bossClear.bossSeed)){
        execution[name]['boss-retirement']=true;
        execution[name]['boss-count-zero']=bossCount===0;
        execution[name]['boss-room-clear']=roomClear;
      }
    }
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
    if(bossClear){
      const concrete=concreteWorlds[name],eligibleActors=concrete.filter(eligible);
      for(const [room,seeds] of Object.entries(bossClear.prefixActors)){
        if(world.room!==Number(room))continue;
        const expected=seeds.map(([kind,seed])=>[kind,0,0,seed]);
        const admitted=JSON.stringify(eligibleActors.map(concreteIdentity))===JSON.stringify(expected);
        if(admitted)execution[name][`contrived-prefix-${room}-admission`]=true;
        // Previously observed exact cohort followed by actual full-manager
        // absence proves retirement; an empty214/20 subset cannot prove it.
        if(execution[name][`contrived-prefix-${room}-admission`]&&concrete.length===0){
          execution[name][`contrived-prefix-${room}-retirement`]=true;
          if(world.clear)execution[name][`contrived-prefix-${room}-clear`]=true;
        }
      }
      const concreteBosses=concrete.filter(a=>a.type===20);
      if(concreteBosses.length)check(checks,label,`${name}.contrivedBoss.authoredIdentity`,[[20,0,0,bossClear.bossSeed]],concreteBosses.map(concreteIdentity));
      for(const boss of concreteBosses){
        if(boss.dead){bossDeaths[name].add(boss.seed);execution[name]['boss-death']=true;}
        if(boss.dead&&boss.animation==='Death')execution[name]['boss-death-animation']=true;
      }
    }
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
const hash=path=>createHash('sha256').update(artifactBytes(path)).digest('hex');
const report={schema:1,status,execution,coverage,gateBinaryIdentity,checks,diagnostics,sources:{originalManifest:hash(originalPath),browserManifest:hash(browserPath),case:hash(resolve(directory,'case.json')),pairedGate:gate.gateSources,comparator:hash('tools/foundation/monstro-state-pair.mjs'),originalBinarySha256:gateBinaryIdentity.observed},scope:'All declared actual actor/body/state/HP/ordered queue/feedback/scene/projectile/Poof fields and executions are required. Exact shared-stream trajectories remain visible failures; absolute causal loader counter is named unaccepted. Blood background-paint/return acceptance needs an actual native render submission boundary; The source-selected contrived978 cases require real prefix clear, boss death/retirement, persistent RNG30, Room2490 zero, RoomClear and real final Heart seed/body/sequence and the fence when its admission is actually observed. Sparse compact captures leave unobserved exact0-frame admission coverage explicitly pending; they do not prove initialization order. Actual death effects2/42/77, rewards/Grid17/Stage2 remain mandatory separate dependencies. Final NPC flags and pre-Base callback position after manager retirement cannot be inferred from a removed actor; no event marker substitutes for those source obligations.'};
writeFileSync(resolve(option('--output')),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({status,fields:checks.length,failures:checks.filter(row=>row.status==='fail').length,execution}));process.exitCode=status==='pass'?0:1;
};
try{run();}catch(error){
  const digest=path=>{try{return createHash('sha256').update(artifactBytes(path)).digest('hex');}catch{return null;}};
  const report={schema:1,status:'fail',error:String(error),execution,coverage,gateBinaryIdentity,checks,diagnostics,
    sources:{originalManifest:digest(originalPath),browserManifest:digest(browserPath),case:digest(resolve(directory,'case.json')),comparator:digest('tools/foundation/monstro-state-pair.mjs'),originalBinarySha256:gateBinaryIdentity.observed},
    scope:'Required comparison aborted. Keep the partial checks and all raw/lossless archived captures; no complete acceptance.'};
  writeFileSync(resolve(option('--output')),JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify({status:'fail',error:report.error,fields:checks.length,failures:checks.filter(row=>row.status==='fail').length,execution}));process.exitCode=1;
}
