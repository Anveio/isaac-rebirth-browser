// Actual244/type9 fields from both public-input games; no native data enters WASM.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const option = name => { const at = process.argv.indexOf(name); assert.ok(at >= 0 && process.argv[at + 1]); return process.argv[at + 1]; };
const directory = resolve(option('--case'));
const load = path => JSON.parse(readFileSync(path, 'utf8'));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const originalPath = resolve(directory, 'original/manifest.json'), browserPath = resolve(directory, 'browser/manifest.json');
const original = load(originalPath), browser = load(browserPath), replay = load(resolve(directory, 'case.json'));
const pair = load(resolve(directory, 'report.json'));
assert.equal(pair.original.status, 'pass'); assert.equal(pair.browser.status, 'pass');
assert.equal(original.status, 'pass'); assert.equal(browser.status, 'pass');
const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
const observations = new Map(original.debugObservations.map(row => [row.context.label, row.observation]));
const f32 = value => Math.fround(value), vector = point => [f32(point.x ?? point[0]), f32(point.y ?? point[1])];
const checks = [], diagnostics = [];
const check = (rows, label, field, expected, observed) => { assert.notEqual(expected, undefined, field); assert.notEqual(observed, undefined, field); rows.push({label,field,expected,actual:observed,status:JSON.stringify(expected)===JSON.stringify(observed)?'pass':'fail'}); };
const execution = Object.fromEntries(['native', 'browser'].map(game => [game, Object.fromEntries(['admission','appearance','hidden','relocation','shoot','projectile-contact','locked','hostile-return'].map(name => [name,false]))]));
const initial = {native:new Map(),browser:new Map()}, prior = {native:new Map(),browser:new Map()};
assert.ok(replay.pairedTape.some(action=>action.type==='capture'&&action.label==='gameplay-start'),'Tape must declare its gameplay boundary');
let hostileDescriptor, gameplayStarted=false;
const bodies = [];
for (const action of replay.pairedTape) {
  if (action.type !== 'capture') continue;
  const label = action.label, observation = observations.get(label), actual = frames.get(label)?.state;
  if(label==='gameplay-start')gameplayStarted=true;
  assert.ok(observation && actual, `Missing required capture${label}`);
  assert.equal(observation.status, 'pass');
  const source = observation.gameplay;
  if (actual.screen !== 'playing') {
    assert.equal(gameplayStarted,false,'Gameplay captures may not fall back to pre-game skipping');
    assert.equal(source.status, 'skip', 'Only explicitly pre-game observations may skip');
    continue;
  }
  assert.equal(source.status, 'pass', 'Every gameplay capture requires a real source gameplay read');
  assert.equal(source.memoryWritten, false);
  const phase = source.nativePhase.systemIterations % 2;
  const natives = source.entities.filter(actor => actor.type === 244 && actor.variant === 0 && actor.subtype === 0);
  const port = actual.room.npcs.filter(actor => actor.kind === 244 && actor.variant === 0 && actor.subtype === 0);
  check(checks,label,'npc.spawnSeedSet',natives.map(actor=>actor.spawnSeed),port.map(actor=>actor.spawnSeed));
  check(checks,label,'room.identity',source.room.descriptorIndex,actual.room.descriptor?.index ?? actual.currentRoom);
  check(checks,label,'system.phase',phase,actual.systemClock.systemCounter%2);
  check(diagnostics,label,'game.fullUpdates',source.nativePhase.fullGameUpdates,actual.gameUpdateCount);
  check(checks,label,'bounds.topLeft',source.room.boundsTopLeft,[60,140]);
  check(checks,label,'bounds.bottomRight',source.room.boundsBottomRight,[60+(source.room.gridWidth-2)*40,140+(source.room.gridHeight-2)*40]);
  assert.ok(Array.isArray(source.pathCosts) && Array.isArray(actual.room.pathCosts));
  assert.equal(source.pathCosts.length, 448);
  assert.equal(actual.room.pathCosts.length, 448);
  assert.ok(source.pathCosts.every(Number.isInteger));
  assert.ok(actual.room.pathCosts.every(Number.isInteger));
  check(checks,label,'room.pathCosts',source.pathCosts,actual.room.pathCosts);
  for (const native of natives) {
    const actor=port.find(actor=>actor.spawnSeed===native.spawnSeed); if(!actor)continue;
    for(const [field,expected,observed]of [
      ['hp',f32(native.hp),f32(actor.hp)],['maxHp',f32(native.maxHp),f32(actor.maxHp)],
      ['radius',f32(native.collisionRadius),f32(actor.collisionRadius)],['mass',f32(native.collisionMass),f32(actor.collisionMass)],
      ['contactDamage',f32(native.collisionDamage),f32(actor.collisionDamage)],['canShutDoors',native.canShutDoors,actor.canShutDoors],['boss',native.boss,actor.boss],
      ['savedInitializedState',native.npcSavedInitializedState,actor.savedInitializedState],['appearanceCounter',native.npcFrameAge,actor.frameAge],
      ['state',native.npcState,actor.state],['animation',native.animation,actor.animation],['animationFrame',f32(native.animationFrame),f32(actor.animationClock)],
      ['animationPlaying',native.animationPlaying,actor.animationPlaying],['visible',native.visible,actor.visible],
      ['gridClass',native.gridCollisionClass,actor.gridCollisionClass],['entityClass',native.entityCollisionClass,actor.entityCollisionClass],
      ['dead',native.removed,actor.dead],['active',native.active,!actor.removed],
      ['pendingDamage',f32(native.pendingDamage),f32(actor.damage.pendingDamage)],['pendingFlags',native.pendingFlags,actor.damage.pendingFlags],
    ])check(checks,label,`npc${native.spawnSeed}.${field}`,expected,observed);
    check(checks,label,`npc${native.spawnSeed}.animationSpeed`,f32(native.animationSpeed),1);
    for(const [field,expected,observed]of [['position',vector(native.position),vector(phase===0?actor.presentedPosition:actor.position)],['velocity',vector(native.velocity),vector(actor.velocity)],['target',vector(native.npcTarget),vector(actor.target)]])check(diagnostics,label,`npc${native.spawnSeed}.${field}`,expected,observed);
  }
  assert.equal(typeof source.room.combatClear,'boolean'); assert.ok(Number.isInteger(source.room.combatClearCountdown));
  assert.ok(Number.isInteger(source.room.countableCurrent)&&Number.isInteger(source.room.countablePending));
  check(checks,label,'room.combatClear',source.room.combatClear,actual.room.combatClear);
  check(checks,label,'room.combatClearCountdown',source.room.combatClearCountdown,actual.room.combatClearCountdown);
  check(checks,label,'manager.countablePolicy',source.room.countableCurrent+source.room.countablePending>0,actual.room.npcs.some(actor=>actor.canShutDoors&&!actor.removed));
  for(const native of source.grid.filter(cell=>cell.type===16&&[60,74].includes(cell.index))) {
    const cell=actual.room.grid.cells[native.index]?.entity;assert.ok(cell);
    check(checks,label,`door${native.index}.state`,native.state,cell.state);check(checks,label,`door${native.index}.class`,native.collisionClass,cell.collisionClass);
  }
  const player=source.players[0];assert.ok(player.lastDamageSource);
  check(checks,label,'player.redHp',player.redHp,actual.player.hp);check(checks,label,'player.immunity',player.damageCountdown,actual.damageCooldown);
  const sourceProjectiles=source.entities.filter(actor=>actor.type===9&&actor.variant===0);
  check(checks,label,'projectile.activeCount',sourceProjectiles.filter(actor=>actor.active&&!actor.removed).length,actual.enemyProjectiles.filter(actor=>!actor.dead&&!actor.removed).length);
  check(diagnostics,label,'projectile.seedSet',sourceProjectiles.map(actor=>actor.spawnSeed),actual.enemyProjectiles.map(actor=>actor.spawnSeed));
  const games={native:{npcs:natives.map(actor=>({seed:actor.spawnSeed,state:actor.npcState,saved:actor.npcSavedInitializedState,age:actor.npcFrameAge,animation:actor.animation,position:actor.position})),
    projectiles:sourceProjectiles.map(actor=>({id:actor.sequenceId,seed:actor.spawnSeed,position:actor.position,velocity:actor.velocity,height:actor.projectileHeight,fallingSpeed:actor.projectileFallingSpeed,fallingAcceleration:actor.projectileFallingAcceleration,age:actor.projectileFullUpdates,dead:actor.removed,radius:actor.collisionRadius,mass:actor.collisionMass,gridClass:actor.gridCollisionClass,entityClass:actor.entityCollisionClass,gridContact:actor.collidedWithGrid,flags:actor.projectileFlags,scale:actor.projectileScale,parameter1:actor.projectileParameter1,parameter2:actor.projectileParameter2,animation:actor.animation,animationFrame:actor.animationFrame,animationPlaying:actor.animationPlaying})),clear:source.room.combatClear},
    browser:{npcs:port.map(actor=>({seed:actor.spawnSeed,state:actor.state,saved:actor.savedInitializedState,age:actor.frameAge,animation:actor.animation,position:vector(phase===0?actor.presentedPosition:actor.position)})),
      projectiles:actual.enemyProjectiles.map(actor=>({id:actor.id,seed:actor.spawnSeed,position:vector(phase===0?actor.presentedPosition:actor.position),velocity:vector(actor.velocity),height:actor.height,fallingSpeed:actor.fallingSpeed,fallingAcceleration:actor.fallingAcceleration,age:actor.fullUpdates,dead:actor.dead,radius:actor.collisionRadius,mass:actor.collisionMass,gridClass:actor.gridCollisionClass,entityClass:actor.entityCollisionClass,gridContact:actor.gridContact,flags:0,scale:actor.scale,parameter1:actor.sourceParameter1,parameter2:actor.sourceParameter2,animation:actor.animation,animationFrame:actor.animationClock,animationPlaying:actor.animationPlaying})),clear:actual.room.combatClear}};
  for(const[game,world]of Object.entries(games)) {
    if(world.npcs.length)execution[game].admission=true;
    for(const npc of world.npcs) {
      if(!initial[game].has(npc.seed))initial[game].set(npc.seed,npc.position);
      if(npc.state===6&&npc.saved===6&&npc.age===22)execution[game].appearance=true;
      if(npc.state===4)execution[game].hidden=true;
      if(npc.state===8&&JSON.stringify(npc.position)!==JSON.stringify(initial[game].get(npc.seed)))execution[game].relocation=true;
    }
    if(world.projectiles.length)execution[game].shoot=true;
    if(world.npcs.length&&!world.clear)execution[game].locked=true;
    for(const projectile of world.projectiles) {
      check(checks,label,`${game}.projectile${projectile.seed}.radius`,5,f32(projectile.radius));
      check(checks,label,`${game}.projectile${projectile.seed}.mass`,8,f32(projectile.mass));
      check(checks,label,`${game}.projectile${projectile.seed}.flags`,0,projectile.flags);
      check(checks,label,`${game}.projectile${projectile.seed}.fallingAcceleration`,f32(.1),f32(projectile.fallingAcceleration));
      for(const[field,expected,observed]of [['scale',1,f32(projectile.scale)],['parameter1',1,f32(projectile.parameter1)],['parameter2',f32(.005),f32(projectile.parameter2)],
        ['animation',projectile.age===0?null:'RegularTear6',projectile.animation],['animationFrame',projectile.age===0?-1:projectile.age%4,f32(projectile.animationFrame)],['animationPlaying',projectile.age>0,projectile.animationPlaying]])check(checks,label,`${game}.projectile${projectile.seed}.${field}`,expected,observed);
      if(projectile.age===0){check(checks,label,`${game}.projectile${projectile.seed}.birthHeight`,-23,f32(projectile.height));check(checks,label,`${game}.projectile${projectile.seed}.mode0FallingSpeed`,-3,f32(projectile.fallingSpeed));}
      const previous=prior[game].get(projectile.id);
      if(previous&&source.nativePhase.systemIterations===previous.counter+1) {
        if(phase===0) {
          check(checks,label,`${game}.projectile${projectile.seed}.fastHeight`,f32(previous.body.height),f32(projectile.height));
          check(checks,label,`${game}.projectile${projectile.seed}.fastFallingSpeed`,f32(previous.body.fallingSpeed),f32(projectile.fallingSpeed));
        } else if(!previous.body.dead&&projectile.age===previous.body.age+1) {
          const expected=f32(f32(f32(.9)*f32(previous.body.fallingSpeed))+f32(.1));
          check(checks,label,`${game}.projectile${projectile.seed}.fullFallingSpeed`,f32(expected+f32(projectile.fallingAcceleration)),f32(projectile.fallingSpeed));
          check(checks,label,`${game}.projectile${projectile.seed}.fullHeight`,f32(f32(previous.body.height)+f32(projectile.fallingSpeed)),f32(projectile.height));
          if(projectile.height>-5||previous.body.gridContact&&projectile.height>=-50)check(checks,label,`${game}.projectile${projectile.seed}.sameFullGroundDeath`,true,projectile.dead);
        }
      }
      prior[game].set(projectile.id,{counter:source.nativePhase.systemIterations,body:projectile});
    }
  }
  if(player.damageCountdown===60&&player.lastDamageSource.type===9&&player.lastDamageSource.flags===0
    &&sourceProjectiles.some(actor=>actor.spawnSeed===player.lastDamageSource.activeActorSpawnSeed))execution.native['projectile-contact']=true;
  if(actual.enemyProjectiles.some(actor=>actor.acceptedPlayerHit&&actor.dead)&&actual.damageCooldown===60)execution.browser['projectile-contact']=true;
  if(natives.length)hostileDescriptor=actual.currentRoom;
  if(label.startsWith('return-')||label==='returned')for(const[game,world]of Object.entries(games))if(world.npcs.length&&!world.clear&&actual.currentRoom===hostileDescriptor)execution[game]['hostile-return']=true;
  bodies.push({label,sourceCounter:source.nativePhase.systemIterations,player:{nativeHp:player.redHp,browserHp:actual.player.hp,nativeImmunity:player.damageCountdown,browserImmunity:actual.damageCooldown,source:player.lastDamageSource},npcNative:games.native.npcs,npcBrowser:games.browser.npcs,projectileNative:games.native.projectiles,projectileBrowser:games.browser.projectiles});
}
for(const name of replay.wormRequired)for(const game of ['native','browser'])check(checks,'execution',`${game}.${name}`,true,execution[game][name]);
const report={schema:1,status:checks.every(check=>check.status==='pass')?'pass':'fail',execution,checks,diagnostics,bodies,
  sourceHashes:{original:hash(originalPath),browser:hash(browserPath),comparator:hash(new URL(import.meta.url))},
  scope:'Public244/type9 admission, saved appearance state, dig/relocation, actual projectile defaults/full-only vertical law, HP/immunity and hostile locks. Exact shared-MT trajectory/seed differences remain diagnostic; all required differences remain red. Gib and full FX factory consumers, rewards and complete room-return reconstruction are unaccepted.'};
writeFileSync(resolve(option('--output')),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({status:report.status,fields:checks.length,failures:checks.filter(check=>check.status==='fail').length,execution}));
process.exitCode=report.status==='pass'?0:1;
