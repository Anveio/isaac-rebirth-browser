// Additive game-level green-creep owners. Called after existing entity rows.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const integer = value => {
  assert.ok(Number.isInteger(value) && value >= 0 && value <= 0xffffffff, 'missing/invalid u32 owner');
  return value;
};
const signed = value => {
  assert.ok(Number.isInteger(value) && value >= -0x80000000 && value <= 0x7fffffff, 'missing/invalid i32 owner');
  return value;
};
const boolean = value => { assert.equal(typeof value, 'boolean'); return value; };
const text = value => { assert.equal(typeof value, 'string'); return value; };
const scalar = value => { assert.ok(typeof value === 'number' && Number.isFinite(value)); return value; };
const word = value => {
  const data = new DataView(new ArrayBuffer(4));
  data.setFloat32(0, scalar(value), true);
  assert.ok(Number.isFinite(data.getFloat32(0, true)));
  return data.getUint32(0, true).toString(16).padStart(8, '0');
};
const array = value => { assert.ok(Array.isArray(value), 'missing array owner'); return value; };
const sizedWords = length => value => { assert.equal(array(value).length, length); return value.map(word); };
const vector = value => sizedWords(2)(Array.isArray(value) ? value : [value?.x, value?.y]);
const words = sizedWords(2);
const identity = value => array(value).map(integer);
const own = (get, normalize) => {
  let raw;
  try {
    raw = get();
    assert.notEqual(raw, undefined, 'owner unobserved');
    return { observed: true, raw, value: normalize(raw) };
  } catch (error) {
    return { observed: false, raw: raw === undefined ? null : raw, error: String(error) };
  }
};
const emit = (rows, field, expected, actual, normalize = value => value, scope = 'native/browser') => {
  const left = own(expected, normalize), right = own(actual, normalize);
  const owners = scope === 'native/browser'
    ? { rawNative: left.raw, rawBrowser: right.raw, nativeObserved: left.observed, browserObserved: right.observed }
    : { rawSourceExpectation: left.raw, rawObservedOwner: right.raw, expectationObserved: left.observed, ownerObserved: right.observed };
  rows.push({ field, scope, expected: left.observed ? left.value : null, actual: right.observed ? right.value : null,
    ...owners, ...(left.error && { expectedError: left.error }), ...(right.error && { actualError: right.error }),
    status: left.observed && right.observed && JSON.stringify(left.value) === JSON.stringify(right.value) ? 'pass' : 'fail' });
};
const law = (rows, field, expected, get, normalize = value => value) =>
  emit(rows, field, typeof expected === 'function' ? expected : () => expected, get, normalize, 'Source law / independently observed side');

const key = actor => JSON.stringify([integer(actor.sequence), integer(actor.birth), integer(actor.seed)]);
const nativeActor = actor => ({ raw: actor, sequence: actor.sequenceId, birth: actor.birthGameUpdate, seed: actor.spawnSeed,
  counter: actor.greenCreep?.counter, size: actor.greenCreep?.size, ttl: actor.greenCreep?.timeToLive,
  raw7f4: actor.greenCreep?.raw7f4, clipFrames: actor.greenCreep?.clipFrames, clip: actor.animation,
  frame: actor.animationFrame, scale: actor.spriteScale, tint: actor.spriteTint, dead: actor.removed, active: actor.active });
const browserActor = (actor, identities) => ({ raw: actor,
  sequence: identities.find(row => row.actor?.kind === 'effect' && row.actor.id === actor.id)?.sequenceId,
  birth: actor.birthGameUpdate, seed: actor.spawnSeed, counter: actor.creepEffect?.counter, size: actor.creepEffect?.size,
  ttl: actor.timeToLive, raw7f4: actor.raw7f4, clip: actor.sprite?.animation, frame: actor.sprite?.frame,
  scale: actor.sprite?.scale && [actor.sprite.scale.x, actor.sprite.scale.y], tint: actor.sprite?.tint,
  dead: actor.removed, active: actor.active });

// The Green emitter is a 10.0 champion while the historical Gaper spec
// selects 10.1. Grade both existing family variants additively, without
// changing discovery, scalar spec selection, or any historical row key.
function gaperSpawners(rows, source, actual) {
  const native = own(() => array(source.entities)
    .filter(actor => actor.type === 10 && (actor.variant === 0 || actor.variant === 1)), value => value);
  const browser = own(() => {
    const identities = array(actual.actorScheduler?.identities);
    return array(actual.room?.npcs)
      .filter(actor => actor.kind === 10 && (actor.variant === 0 || actor.variant === 1) && !boolean(actor.removed))
      .map(actor => {
        const id = integer(actor.id);
        const matches = identities.filter(row => row.actor?.kind === 'npc' && integer(row.actor.id) === id);
        return { raw: actor, sequence: matches.length === 1 ? matches[0].sequenceId : undefined };
      });
  }, value => value);
  law(rows, 'green.spawner.nativeObservation', ['pass', false],
    () => [text(source.status), boolean(source.memoryWritten)]);
  law(rows, 'green.spawner.nativeOwner', true, () => native.observed, boolean);
  law(rows, 'green.spawner.browserOwner', true, () => browser.observed, boolean);
  emit(rows, 'green.spawner.phase', () => integer(source.nativePhase?.systemIterations) % 2,
    () => integer(actual.systemClock?.systemCounter) % 2, integer);
  emit(rows, 'green.spawner.fullCounter', () => source.nativePhase?.fullGameUpdates,
    () => actual.gameUpdateCount, integer);
  const nativeIdentity = actor => [actor.sequenceId, actor.type, actor.variant, actor.subtype, actor.spawnSeed];
  const browserIdentity = actor => [actor.sequence, actor.raw.kind, actor.raw.variant, actor.raw.subtype, actor.raw.spawnSeed];
  emit(rows, 'green.spawner.identities', () => array(native.value).map(nativeIdentity),
    () => array(browser.value).map(browserIdentity), value => array(value).map(identity));
  for (const [side, owner, getIdentity] of [['native', native, nativeIdentity], ['browser', browser, browserIdentity]])
    law(rows, `green.spawner.${side}.uniqueManagerSequence`, true, () => {
      const sequences = array(owner.value).map(actor => integer(getIdentity(actor)[0]));
      return new Set(sequences).size === sequences.length;
    }, boolean);
  for (const n of native.value ?? []) {
    const ownedIdentity = own(() => nativeIdentity(n), identity);
    if (!ownedIdentity.observed) {
      law(rows, 'green.spawner.nativeIdentityOwner', true, () => ownedIdentity.observed, boolean);
      continue;
    }
    const id = ownedIdentity.value;
    const p = browser.value?.find(actor => {
      const candidate = own(() => browserIdentity(actor), identity);
      return candidate.observed && JSON.stringify(candidate.value) === JSON.stringify(id);
    })?.raw;
    const prefix = `green.spawner.${id.join('.')}`;
    for (const [field, expected, observed, normalize] of [
      ['position', () => n.position, () => source.nativePhase.systemIterations % 2 === 0 ? p?.presentedPosition : p?.position, vector],
      ['velocity', () => n.velocity, () => p?.velocity, vector],
      ['target', () => n.npcTarget, () => p?.target, vector],
      ['state', () => n.npcState, () => p?.state, signed],
      ['frameAge', () => n.npcFrameAge, () => p?.frameAge, integer],
      ['hp', () => n.hp, () => p?.hp, word], ['maxHp', () => n.maxHp, () => p?.maxHp, word],
      ['pendingDamage', () => n.pendingDamage, () => p?.damage?.pendingDamage, word],
      ['pendingFlags', () => n.pendingFlags, () => p?.damage?.pendingFlags, integer],
      ['animation', () => n.animation, () => p?.animation, text],
      ['animationFrame', () => n.animationFrame, () => p?.animationClock, word],
      ['animationPlaying', () => n.animationPlaying, () => p?.animationPlaying, boolean],
      ['championColor', () => n.npcChampionColor, () => p === undefined ? undefined : p.champion?.color ?? -1, signed],
      ['active', () => n.active, () => p === undefined ? undefined : !boolean(p.removed), boolean],
      ['dead', () => n.removed, () => p?.dead, boolean],
      ['radius', () => n.collisionRadius, () => p?.collisionRadius, word],
      ['mass', () => n.collisionMass, () => p?.collisionMass, word],
      ['contactDamage', () => n.collisionDamage, () => p?.collisionDamage, word],
    ]) {
      emit(rows, `${prefix}.${field}`, expected, observed, normalize);
      if (field === 'position') rows.at(-1).browserPositionOwners = {
        physical: p?.position ?? null, presented: p?.presentedPosition ?? null };
    }
    // Native +470 and the port's retained AI multiplier have different
    // update ownership; no shared friction projection is admitted here.
  }
}

export function createGreenCreepContract(caseData, configuration) {
  const configured = typeof configuration?.requiredRoute?.seed === 'string'
    && Number.isInteger(configuration?.requiredRoute?.slot);
  const required = !configured || (typeof caseData.seed === 'string'
    && caseData.seed.replaceAll(' ', '') === configuration.requiredRoute.seed
    && caseData.entity?.slot === configuration.requiredRoute.slot);
  const state = Object.fromEntries(['native', 'browser'].map(side => [side,
    { previous: new Map(), retired: new Set(), witnesses: new Set(), clock: undefined }]));
  let previousHealth;
  let clipFrames, metadataError;
  try {
    const metadata = JSON.parse(readFileSync(new URL('../../web/familiar-data.json', import.meta.url), 'utf8'));
    const clips = array(metadata['gfx/1000.023_creep (green).anm2']?.Animations);
    clipFrames = new Map(clips.map(clip => [text(clip.Name), integer(clip.FrameNum)]));
  } catch (error) { metadataError = String(error); }

  function lifecycle(rows, side, actors, clock, label) {
    const ledger = state[side], current = new Map();
    let identityComplete = true;
    law(rows, `green.${side}.authoredClipMetadata`, true, () => !metadataError, boolean);
    for (const actor of actors) {
      let id;
      try { id = key(actor); } catch (error) {
        identityComplete = false;
        rows.push({ field: `green.${side}.identityOwner`, expected: 'typed observed identity', actual: null,
          rawIdentity: [actor.sequence ?? null, actor.birth ?? null, actor.seed ?? null], status: 'fail', error: String(error) });
        continue;
      }
      const name = `green.${side}.${id}`;
      law(rows, `${name}.uniqueIdentity`, false, () => current.has(id), boolean);
      if (current.has(id)) { identityComplete = false; continue; }
      current.set(id, actor);
      law(rows, `${name}.notReadmitted`, false, () => ledger.retired.has(id), boolean);
      law(rows, `${name}.birthModulo7`, 0, () => integer(actor.birth) % 7, integer);
      law(rows, `${name}.raw7f4`, 30, () => actor.raw7f4, signed);
      const previous = ledger.previous.get(id);
      if (!previous && actor.birth === clock) {
        ledger.witnesses.add('birth');
        law(rows, `${name}.initial.counter`, 1, () => actor.counter, integer);
        law(rows, `${name}.initial.ttl`, 29, () => actor.ttl, signed);
        law(rows, `${name}.initial.clip`, true, () => /^Blood0[1-6]$/.test(text(actor.clip)), boolean);
        const frames = clipFrames?.get(actor.clip);
        emit(rows, `${name}.initial.frame`, () => clock % integer(frames), () => actor.frame, word, 'Source law / independently observed side');
        if (side === 'native') law(rows, `${name}.initial.clipFrames`, frames, () => actor.clipFrames, integer);
        ledger.witnesses.add('firstUpdate');
      }
      if (previous && ledger.clock !== undefined) {
        const delta = clock - ledger.clock;
        if (delta === 0) {
          for (const field of ['counter', 'ttl'])
            law(rows, `${name}.fast.${field}`, previous[field], () => actor[field], signed);
        } else if (delta === 1) {
          law(rows, `${name}.full.ttl`, () => { const ttl = signed(previous.ttl); return ttl > 0 ? ttl - 1 : ttl; }, () => actor.ttl, signed);
          if (previous.dead === true) {
            law(rows, `${name}.fade.counter`, () => integer(previous.counter) + 1, () => actor.counter, integer);
            law(rows, `${name}.fade.scale`, () => array(previous.scale).map(value => Math.fround(Math.fround(scalar(value)) * Math.fround(.98))), () => actor.scale, words);
            law(rows, `${name}.fade.alpha`, () => Math.fround(Math.fround(scalar(previous.tint?.[3])) * Math.fround(.95)), () => actor.tint?.[3], word);
            ledger.witnesses.add('fade');
          }
        }
      }
      if (actor.ttl === 0 && actor.dead === true) ledger.witnesses.add('timeout');
      if (actor.dead === true && actor.counter === 40) ledger.witnesses.add('lastFade');
    }
    for (const [id, previous] of ledger.previous) if (!current.has(id)) {
      const consecutive = identityComplete && ledger.clock !== undefined && clock - ledger.clock === 1;
      law(rows, `green.${side}.${id}.retirementAfterFade40`, true,
        () => consecutive && previous.dead === true && previous.counter === 40, boolean);
      if (consecutive && previous.dead === true && previous.counter === 40) {
        ledger.retired.add(id); ledger.witnesses.add('retirement');
      }
    }
    ledger.previous = current;
    ledger.clock = clock;
    ledger.label = label;
  }

  return {
    capture(rows, source, actual, label) {
      // Each side and field fails independently; malformed additive owners never suppress old rows.
      const left = own(() => array(source.entities).filter(actor => actor.type === 1000 && actor.variant === 23).map(nativeActor), value => value);
      const right = own(() => {
        const identities = array(actual.actorScheduler?.identities);
        return array(actual.spriteEffects).filter(actor => actor.variant === 23).map(actor => browserActor(actor, identities));
      }, value => value);
      law(rows, 'green.owner.native', true, () => left.observed, boolean);
      law(rows, 'green.owner.browser', true, () => right.observed, boolean);
      emit(rows, 'green.phase', () => integer(source.nativePhase?.systemIterations) % 2,
        () => integer(actual.systemClock?.systemCounter) % 2, integer);
      emit(rows, 'green.fullCounter', () => source.nativePhase?.fullGameUpdates, () => actual.gameUpdateCount, integer);
      emit(rows, 'green.identities', () => array(left.value).map(actor => [actor.sequence, actor.birth, actor.seed]),
        () => array(right.value).map(actor => [actor.sequence, actor.birth, actor.seed]), value => array(value).map(identity));
      emit(rows, 'green.activeManagerOrder', () => array(left.value).map(actor => actor.sequence), () => {
        const actors = array(right.value), order = array(actual.actorScheduler?.actors);
        return order.filter(ref => ref.kind === 'effect' && actors.some(actor => actor.raw.id === ref.id))
          .map(ref => actors.find(actor => actor.raw.id === ref.id).sequence);
      }, identity);
      for (const native of left.value ?? []) {
        const port = right.value?.find(actor => actor.sequence === native.sequence && actor.birth === native.birth && actor.seed === native.seed);
        const n = native.raw, p = port?.raw, prefix = `green.${native.sequence}.${native.birth}.${native.seed}`;
        law(rows, `${prefix}.nativeTailStatus`, 'pass', () => n.greenCreep?.status, text);
        const fields = [
          ['identity', () => [n.type, n.variant, n.subtype], () => [1000, p?.variant, p?.subtype], identity],
          ['spawner', () => [n.greenCreep?.spawnerType, n.greenCreep?.spawnerVariant], () => [p?.spawnerType, p?.spawnerVariant], identity],
          ['parentNull', () => n.greenCreep?.parentNull, () => p?.parent === undefined ? undefined : p.parent === null, boolean],
          ['actorRng', () => n.greenCreep?.actorRng, () => [p?.actorRng?.state, p?.actorRng?.rightA, p?.actorRng?.left, p?.actorRng?.rightB], value => { assert.equal(array(value).length, 4); return identity(value); }],
          ['counter', () => native.counter, () => port?.counter, integer], ['size', () => native.size, () => port?.size, word],
          ['ttl', () => native.ttl, () => port?.ttl, signed], ['raw7f4', () => native.raw7f4, () => port?.raw7f4, signed],
          ['animation', () => native.clip, () => port?.clip, text], ['frame', () => native.frame, () => port?.frame, word],
          ['playing', () => n.animationPlaying, () => p?.sprite?.playing, boolean],
          ['scale', () => native.scale, () => port?.scale, words], ['tint', () => native.tint, () => port?.tint, sizedWords(4)],
          ['colorOffsets', () => { assert.equal(array(n.spriteColor).length, 11); return n.spriteColor.slice(8, 11); }, () => p?.sprite?.offsets, sizedWords(3)],
          ['position', () => n.position, () => source.nativePhase.systemIterations % 2 === 0 ? p?.presentedPosition : p?.position, vector],
          ['velocity', () => n.velocity, () => p?.velocity, vector],
          ['radius', () => n.collisionRadius, () => p?.collisionRadius, word],
          ['flags', () => n.entityFlags, () => p?.flags, integer],
          ['active', () => n.active, () => p?.active, boolean], ['dead', () => n.removed, () => p?.removed, boolean],
          ['visible', () => n.visible, () => p?.visible, boolean],
        ];
        for (const [field, expected, observed, normalize] of fields) {
          emit(rows, `${prefix}.${field}`, expected, observed, normalize);
          if (field === 'position') rows.at(-1).browserPositionOwners = { physical: p?.position ?? null, presented: p?.presentedPosition ?? null };
        }
      }
      for (const [side, owner, clock, phase] of [
        ['native', left, source.nativePhase?.fullGameUpdates, source.nativePhase?.systemIterations],
        ['browser', right, actual.gameUpdateCount, actual.systemClock?.systemCounter],
      ]) {
        try {
          assert.ok(owner.observed, owner.error);
          integer(phase);
          lifecycle(rows, side, owner.value, integer(clock), label);
        } catch (error) {
          rows.push({ field: `green.${side}.lifecycleOwner`, expected: 'typed observed owner', actual: null,
            status: 'fail', error: String(error) });
        }
      }
      // Outcomes are compared directly. Native EntityRef.flags is never renamed to DAMAGE_ACID.
      for (const [field, nativeValue, browserValue] of [
        ['redHp', () => source.players?.[0]?.redHp, () => actual.player?.hp],
        ['soulHp', () => source.players?.[0]?.soulHp, () => actual.player?.soulHp],
        ['immunity', () => source.players?.[0]?.damageCountdown, () => actual.damageCooldown],
      ]) emit(rows, `green.contactOutcome.${field}`, nativeValue, browserValue, integer);
      try {
        const player = array(source.players)[0];
        const health = { native: integer(player.redHp) + integer(player.soulHp),
          browser: integer(actual.player?.hp) + integer(actual.player?.soulHp),
          nativeRed: array(source.entities).filter(actor => actor.type === 1000 && actor.variant === 22).length,
          browserRed: array(actual.spriteEffects).filter(actor => actor.variant === 22).length };
        if (previousHealth && source.nativePhase.fullGameUpdates - previousHealth.clock === 1) {
          const cause = player.lastDamageSource;
          const nativeHit = health.native < previousHealth.native && cause?.type === 1000 && cause.variant === 23
            && array(left.value).some(actor => actor.seed === cause.activeActorSpawnSeed && actor.sequence === cause.activeActorSequenceId);
          const browserHit = integer(actual.gameUpdateCount) - previousHealth.browserClock === 1
            && health.browser < previousHealth.browser && integer(actual.playerDeath?.lastDamageFlags) === 0x10;
          if (nativeHit || browserHit) {
            const unique = health.nativeRed === 0 && health.browserRed === 0
              && previousHealth.nativeRed === 0 && previousHealth.browserRed === 0;
            law(rows, 'green.contactOutcome.uniqueAcidFamily', true, () => unique, boolean);
            emit(rows, 'green.contactOutcome.accepted', () => nativeHit, () => browserHit && unique, boolean);
            if (nativeHit) state.native.witnesses.add('contactOutcome');
            if (browserHit && unique) state.browser.witnesses.add('contactOutcome');
          }
        }
        previousHealth = { ...health, clock: integer(source.nativePhase.fullGameUpdates), browserClock: integer(actual.gameUpdateCount) };
      } catch (error) {
        rows.push({ field: 'green.contactOutcome.owner', expected: 'typed actual player/cause owners', actual: null,
          status: 'fail', error: String(error) });
      }
      gaperSpawners(rows, source, actual);
    },
    finish(rows) {
      law(rows, 'green.required.configuration', true, () => configured, boolean);
      if (!required) return;
      for (const side of ['native', 'browser']) for (const witness of ['birth', 'firstUpdate', 'timeout', 'fade', 'lastFade', 'retirement', 'contactOutcome'])
        law(rows, `green.required.${side}.${witness}`, true, () => state[side].witnesses.has(witness), boolean);
    },
  };
}
