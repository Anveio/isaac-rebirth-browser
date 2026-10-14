// Mandatory typed comparisons of actual native bodies and production WASM.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { scopedState } from './paired-game.mjs';

const authored = JSON.parse(readFileSync(new URL('../../web/familiar-data.json', import.meta.url), 'utf8'));
const assets = {17: authored['gfx/1000.017_poof04.anm2'], 42: authored['gfx/1000.042_blood gush.anm2']};

const f32 = value => {
  assert.equal(typeof value, 'number');
  assert.ok(Number.isFinite(value));
  return Math.fround(value);
};
const vector = value => [f32(value[0] ?? value.x), f32(value[1] ?? value.y)];
const floats = values => values.map(f32);
const freshExecution = () => Object.fromEntries(['bossAdmission', 'bossDamage', 'bossDeath',
  'poof04', 'bloodGush', 'sound72', 'matchedPoof04', 'matchedBloodGush', 'full', 'fast', 'gushCleanup'].map(key => [key, false]));

export function bloodStartComparator({ scenario, bossIdentity }) {
  const execution = { native: freshExecution(), browser: freshExecution() };
  const counts = { snapshots: 0, matchedPoof04: 0, matchedBloodGush: 0, bodyFields: 0 };
  const prior = { native: new Map(), browser: new Map() };
  const priorContext = { native: null, browser: null };
  const diagnostics = [];
  const append = (rows, field, expected, actual) => {
    assert.notEqual(expected, undefined, field); assert.notEqual(actual, undefined, field);
    rows.push({ field, expected, actual, status: JSON.stringify(expected) === JSON.stringify(actual) ? 'pass' : 'fail' });
  };
  return {
    compare(source, actual, gameplay) {
      const checks = scopedState(source, actual, gameplay);
      if (!gameplay) return checks;
      const captured = source.bloodStart;
      assert.equal(captured?.status, 'pass'); assert.equal(captured.memoryWritten, false);
      const phase = captured.systemCounter % 2;
      counts.snapshots++;
      append(checks, 'blood.system.phase', phase, actual.systemClock.systemCounter % 2);
      append(checks, 'blood.game.absoluteAge', captured.gameUpdateCount, actual.gameUpdateCount);
      append(checks, 'blood.room.type', source.gameplay.room.type, actual.room.descriptor.template.type);
      append(checks, 'blood.room.descriptor', source.gameplay.room.descriptorIndex, actual.currentRoom);
      append(checks, 'blood.room.gridDimensions', [source.gameplay.room.gridWidth, source.gameplay.room.gridHeight],
        [actual.room.grid.width, actual.room.grid.height]);
      const nativeBosses = source.gameplay.entities.filter(actor => actor.type === 20 && actor.variant === 0 && actor.subtype === 0);
      const portBosses = actual.room.npcs.filter(actor => actor.kind === 20 && actor.variant === 0 && !actor.removed);
      append(checks, 'blood.boss.seedOrder', nativeBosses.map(actor => actor.spawnSeed), portBosses.map(actor => actor.spawnSeed));
      if (nativeBosses.length || portBosses.length) {
        append(checks, 'blood.nativeBoss.expectedSeed', [bossIdentity.spawnSeed], nativeBosses.map(actor => actor.spawnSeed));
        append(checks, 'blood.browserBoss.expectedSeed', [bossIdentity.spawnSeed], portBosses.map(actor => actor.spawnSeed));
        append(checks, 'blood.nativeBoss.expectedRoom', [bossIdentity.type, bossIdentity.descriptorIndex, bossIdentity.gridIndex],
          [source.gameplay.room.type, source.gameplay.room.descriptorIndex, source.gameplay.room.descriptorGridIndex]);
        const descriptor = actual.room.descriptor;
        append(checks, 'blood.browserBoss.expectedRoom', [bossIdentity.type, bossIdentity.descriptorIndex, bossIdentity.gridIndex,
          bossIdentity.variant, bossIdentity.name], [descriptor.template.type, descriptor.index, descriptor.gridIndex,
          descriptor.template.variant, descriptor.template.name]);
      }
      for (const [name, bosses] of [['native', nativeBosses], ['browser', portBosses]]) {
        if (bosses.length) execution[name].bossAdmission = true;
        if (bosses.some(actor => actor.hp < 250 || (name === 'native' ? actor.pendingDamage : actor.damage.pendingDamage) > 0)) execution[name].bossDamage = true;
        if (bosses.some(actor => actor.animation === 'Death' && (name === 'native' ? actor.removed : actor.dead))) execution[name].bossDeath = true;
      }
      const interval = {
        native: nativeBosses.some(actor => actor.animation === 'Death' && actor.removed &&
          actor.animationFrame >= 12 && actor.animationFrame < 77),
        browser: portBosses.some(actor => actor.animation === 'Death' && actor.dead &&
          actor.animationClock >= 12 && actor.animationClock < 77),
      };
      const nativeEffects = captured.effects.filter(actor => actor.active);
      const portEffects = actual.spriteEffects.filter(actor => actor.active && [17, 42].includes(actor.variant));
      append(checks, 'blood.effect.identityOrder', nativeEffects.map(actor => [actor.variant, actor.spawnSeed]),
        portEffects.map(actor => [actor.variant, actor.spawnSeed]));
      for (const native of nativeEffects) {
        const port = portEffects.find(actor => actor.variant === native.variant && actor.spawnSeed === native.spawnSeed);
        if (!port) continue;
        const start = checks.length;
        const parent = port.parent ? actual.room.npcs.find(actor => actor.id === port.parent.actor.id) : null;
        const linkedParent = !port.parent ? null : parent
          ? { type: parent.kind, variant: parent.variant, spawnSeed: parent.spawnSeed }
          : { unresolvedActorRef: port.parent.actor };
        const fields = {
          subtype: [native.subtype, port.subtype], cachedSpawner: [native.spawner, [port.spawnerType, port.spawnerVariant]],
          birthGameUpdate: [native.birthGameUpdate, port.birthGameUpdate],
          linkedParent: [native.parent, linkedParent],
          active: [native.active, port.active], dead: [native.dead, port.removed], visible: [native.visible, port.visible],
          position: [vector(native.position), vector(phase === 0 ? port.presentedPosition : port.position)],
          velocity: [vector(native.velocity), vector(port.velocity)], offset: [vector(native.spriteOffset), vector(port.sprite.offset)],
          scale: [vector(native.scale), vector(port.sprite.scale)], rotation: [f32(native.rotation), f32(port.sprite.rotation)],
          flipX: [native.flipX, port.sprite.flipX], flipY: [native.flipY, port.sprite.flipY],
          layerFlips: [native.layerFlips, assets[port.variant].Layers.map(layer => port.sprite.layerFlips[layer.Id] || [false, false])],
          tint: [floats(native.tint), floats(port.sprite.tint)], colorOffsets: [floats(native.colorOffsets), floats(port.sprite.offsets)],
          animation: [native.animation, port.sprite.animation], frame: [f32(native.frame), f32(port.sprite.frame)],
          playing: [native.playing, port.sprite.playing], flags: [native.flags, port.flags],
          targetFrame: [native.targetFrame, port.lifetimeDivisor], ttl: [native.ttl, port.timeToLive],
          raw7f4: [native.raw7f4, port.raw7f4], raw7f8: [f32(native.raw7f8), f32(port.raw7f8)],
        };
        for (const [field, pair] of Object.entries(fields)) append(checks, `blood.${native.spawnSeed}.${field}`, ...pair);
        counts.bodyFields += checks.length - start;
        if (interval.native && interval.browser && checks.slice(start).every(row => row.status === 'pass')) {
          const name = native.variant === 17 ? 'matchedPoof04' : 'matchedBloodGush';
          counts[name]++; execution.native[name] = true; execution.browser[name] = true;
        }
      }
      const sound = captured.sound72, portSound = actual.audio.slots[72];
      assert.ok(portSound);
      const nativeSoundAge = (captured.systemCounter >> 1) - sound.lastTick;
      const portSoundAge = portSound.lastTick === null ? null : (actual.systemClock.systemCounter >> 1) - portSound.lastTick;
      if (sound.loaded || portSound.loaded) {
        append(checks, 'blood.sfx72.loaded', sound.loaded, portSound.loaded);
        append(checks, 'blood.sfx72.sampleCount', sound.sampleCount, portSound.samples.length);
        append(checks, 'blood.sfx72.volume', f32(sound.volume), f32(portSound.lastVolume));
        append(checks, 'blood.sfx72.cooldownLength', sound.nextAllowed - sound.lastTick,
          portSound.lastTick === null ? null : portSound.nextAllowed - portSound.lastTick);
        append(checks, 'blood.sfx72.sampleSettings', sound.samples.map(sample => [sample.looping, f32(sample.gain), f32(sample.pitch)]),
          portSound.samples.map(sample => [sample.looping, f32(sample.gain), f32(sample.pitch)]));
        append(checks, 'blood.sfx72.ageFromOwnCounter', nativeSoundAge, portSoundAge);
      }
      if (interval.native && sound.loaded && sound.nextAllowed >= 0 && nativeSoundAge >= 0 && nativeSoundAge <= 1) execution.native.sound72 = true;
      if (interval.browser && portSound.lastTick !== null && portSoundAge >= 0 && portSoundAge <= 1) execution.browser.sound72 = true;
      diagnostics.push({ sourceGameAge: captured.gameUpdateCount, browserGameAge: actual.gameUpdateCount,
        sourceSystemCounter: captured.systemCounter, browserSystemCounter: actual.systemClock.systemCounter,
        scope: 'Independent actual absolute counters; no imported time, draw padding or fitted baseline.' });
      for (const [name, effects, counter, room] of [['native', nativeEffects, captured.systemCounter, source.gameplay.room.descriptorIndex],
        ['browser', portEffects.map(actor => ({variant: actor.variant, spawnSeed: actor.spawnSeed, dead: actor.removed})), actual.systemClock.systemCounter, actual.currentRoom]]) {
        for (const effect of effects) {
          if (interval[name]) {
            execution[name][effect.variant === 17 ? 'poof04' : 'bloodGush'] = true;
            if (counter % 2) execution[name].full = true; else execution[name].fast = true;
          }
        }
        for (const [seed, old] of prior[name]) {
          const previous = priorContext[name];
          if (old.variant === 42 && old.dead && previous?.room === room && counter % 2 === 1 &&
            counter > previous.counter && counter <= previous.counter + 2 &&
            !effects.some(actor => actor.spawnSeed === seed)) execution[name].gushCleanup = true;
        }
        prior[name] = new Map(effects.map(effect => [effect.spawnSeed, effect]));
        priorContext[name] = { room, counter };
      }
      return checks;
    },
    finish(required) {
      const checks = [];
      for (const name of ['native', 'browser']) for (const requirement of required)
        append(checks, `${name}.${requirement}`, true, execution[name][requirement]);
      return { scenario, execution, counts, checks, diagnostics,
        status: checks.every(row => row.status === 'pass') ? 'pass' : 'fail',
        scope: 'Only actual admitted matched bodies pass body requirements. Native generic variant presence is not producer/body acceptance. Sparse unobserved birth/background paint remains unaccepted.' };
    },
  };
}
