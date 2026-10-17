import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const equal = (a, b) => a !== undefined && b !== undefined && JSON.stringify(a) === JSON.stringify(b);
const word = n => { assert.ok(typeof n === 'number' && Number.isFinite(n)); const b = Buffer.alloc(4); b.writeFloatLE(n); return b.readUInt32LE(); };
//TNT's local source RNG[1,5,16], used only for a required nonrare scope guard.
const firstRareRoll = seed => { let x = seed >>> 0; x ^= x >>> 1; x ^= x << 5; x ^= x >>> 16; return (x >>> 0) % 100; };
const code = key => ({Return: 'Enter', Right: 'ArrowRight', Left: 'ArrowLeft', Up: 'ArrowUp', Down: 'ArrowDown', Tab: 'Tab', Escape: 'Escape'})[key]
  ?? (/^\d$/.test(key) ? 'Digit' + key : 'Key' + key.toUpperCase());

export function compareTnt({data, clock, native, browser, observed, calls, samples, nativeAudioDirectory, scopedState}) {
  const checks = [], diagnostics = [], errors = [];
  const check = (label, field, expected, actual, metadata) => checks.push({label, field,
    expected: expected === undefined ? null : expected, actual: actual === undefined ? null : actual,
    ...(expected === undefined || actual === undefined ? {missing: true} : {}),
    ...(metadata ? {metadata} : {}), status: equal(expected, actual) ? 'pass' : 'fail'});
  const f32 = (label, field, expected, actual) => {
    try { check(label, field + '.f32word', word(expected), word(actual)); }
    catch { check(label, field + '.finite', true, false); }
  };
  const attempt = (label, fn) => { try { fn(); } catch (error) { errors.push({label, error: String(error.stack ?? error)}); check(label, 'requiredOwner', true, false); } };
  check('capture', 'native', 'pass', native.status); check('capture', 'browser', 'pass', browser.status);
  check('capture', 'provenance', 'pass', native.provenance?.status); check('capture', 'publicClock', 'pass', native.publicClockValidation?.status);
  check('capture', 'actualEveryTapeStep', true, native.manifest?.debugObserver?.everyTapeStep);
  check('capture', 'declaredAudioOptions', {sfxVolume: 1, musicVolume: 1}, native.manifest?.audioOptions);
  check('capture', 'browserErrors', [], observed.errors); check('capture', 'ownerJournalError', null, observed.returns?.error);
  check('capture', 'ownerJournalComplete', false, observed.returns?.truncated);
  check('capture', 'physicalInputComplete', false, observed.returns?.inputTruncated);
  check('capture', 'trustedPhysicalInput', true, observed.returns?.inputs?.every(row => row.trusted === true));
  check('capture', 'exactPhysicalTape', data.pairedTape.filter(row => row.type === 'key').map(row => ({type: row.down ? 'keydown' : 'keyup', code: code(row.key), repeat: false})),
    observed.returns?.inputs?.map(({type, code, repeat}) => ({type, code, repeat})));
  check('capture', 'apiComplete', false, observed.api?.truncated);
  check('capture', 'inputTraceComplete', false, observed.trace?.inputTraceTruncated);
  check('capture', 'backendStateComplete', false, observed.trace?.backendStateTruncated);
  check('capture', 'backendStateTruncationReason', null, observed.trace?.backendStateTruncationReason);
  check('capture', 'actualReturnedTickCommands', clock.browserTickCommands, observed.returns?.rows?.map(row => row.command));
  check('capture', 'SFXruntimeTraceComplete', false, observed.runtimeAudio?.truncated);
  check('capture', 'volumeTraceComplete', false, observed.runtimeAudio?.volumeCallbacksTruncated);
  check('capture', 'feedbackComplete', false, observed.feedback?.truncated);
  check('capture', 'runtimeNotHalted', false, observed.feedback?.runtimeFailed);
  check('capture', 'feedbackFailures', [], observed.feedback?.events?.filter(row => row.status === 'fail'));
  diagnostics.push({field: 'completeStateHistory', status: observed.trace?.stateHistoryTruncated ? 'fail' : 'pass',
    retainedStates: observed.trace?.states?.length, publicCommands: observed.trace?.commands?.length,
    bytes: observed.trace?.stateEncodedBytes, maxBytes: observed.trace?.maxStateEncodedBytes});
  const observations = new Map((native.manifest?.debugObservations ?? []).map(row => [row.context.label, row]));
  const frames = new Map(browser.frames.map(row => [row.label, row]));
  const labels = data.pairedTape.filter(row => row.type === 'capture').map(row => row.label);
  const boundary = native.manifest?.frames?.find(row => row.label === data.bootstrap.boundaryLabel);
  const gameplayAt = labels.indexOf('gameplay-start');
  for (const [index, label] of labels.entries()) attempt(label, () => {
    const source = observations.get(label), actual = frames.get(label), original = native.manifest.frames.find(row => row.label === label);
    assert.ok(source && actual && original && boundary, 'Missing exact selected capture');
    check(label, 'pairedPresentation', original.presentedFrame - boundary.presentedFrame, actual.browserPresentation);
    for (const row of scopedState(source.observation, actual.state, index >= gameplayAt)) checks.push({label, ...row});
  });
  const returns = new Map((observed.returns?.rows ?? []).map(row => [row.presentation, row]));
  const stepEnds = []; let presentation = 0;
  for (const action of data.pairedTape) if (action.type === 'step') { presentation += action.frames; stepEnds.push(presentation); }
  const nativeSteps = (native.manifest?.debugObservations ?? []).filter(row => row.context.label.startsWith('step-') && row.context.presentedFrame > clock.nativeEntryPresentation);
  check('capture', 'exactNativeStepEndpoints', stepEnds, nativeSteps.map(row => row.context.presentedFrame - clock.nativeEntryPresentation));
  const seenNative = {tnt: false, states: new Set(), fuse: new Set(), effect1: new Set(), wood: new Set(), coinsMove: false, fast: false};
  const seenBrowser = {tnt: false, states: new Set(), fuse: new Set(), effect1: new Set(), wood: new Set(), coinsMove: false, fast: false};
  const nativeBlasts = [], browserBlasts = [];
  const pennyHistory = {Native: null, Browser: null};
  const observePennies = (side, label, coins, balance) => {
    assert.ok(Number.isInteger(balance) && balance >= 0, `${side} actual coin inventory owner`);
    const seeds = coins.map(coin => coin.spawnSeed);
    assert.ok(seeds.every(seed => Number.isInteger(seed) && seed >= 0 && seed <= 0xffffffff));
    check(label, side + '.coins.uniqueSeeds', coins.length, new Set(seeds).size);
    if (!pennyHistory[side]) {
      check(label, side + '.coins.initialFour', data.tnt.coinCount, coins.length);
      check(label, side + '.coins.initialInventory', 0, balance);
      for (const coin of coins) {
        const velocity = side === 'Native' ? coin.velocity : [coin.velocity.x, coin.velocity.y];
        f32(label, side + `.coin${coin.spawnSeed}.initialVx`, 0, velocity[0]);
        f32(label, side + `.coin${coin.spawnSeed}.initialVy`, 0, velocity[1]);
      }
      pennyHistory[side] = {initialBalance: balance, seeds: new Set(seeds), collected: new Set(), retired: new Set()};
    }
    const history = pennyHistory[side], current = new Set(seeds);
    let uncollected = 0;
    for (const coin of coins) {
      const seed = coin.spawnSeed, collected = side === 'Native' ? coin.removed : coin.collected;
      assert.equal(typeof collected, 'boolean', `${side} actual Collect/dead31f owner`);
      check(label, side + `.coin${seed}.initialIdentity`, true, history.seeds.has(seed));
      check(label, side + `.coin${seed}.noReadmissionAfterRetirement`, false, history.retired.has(seed));
      check(label, side + `.coin${seed}.pennySubtype`, 1, coin.subtype);
      check(label, side + `.coin${seed}.noRegrant`, true, !history.collected.has(seed) || collected);
      if (side === 'Browser') {
        assert.equal(typeof coin.removed, 'boolean', 'Actual Browser retirement owner');
        check(label, side + `.coin${seed}.retirementRequiresCollect`, true, !coin.removed || collected);
      }
      if (collected) {
        history.collected.add(seed);
        const velocity = side === 'Native' ? coin.velocity : [coin.velocity.x, coin.velocity.y];
        check(label, side + `.coin${seed}.collectAnimation`, 'Collect', coin.animation);
        check(label, side + `.coin${seed}.collectPlaying`, true, coin.animationPlaying);
        f32(label, side + `.coin${seed}.collectVx`, 0, velocity[0]);
        f32(label, side + `.coin${seed}.collectVy`, 0, velocity[1]);
      } else uncollected++;
    }
    for (const seed of history.seeds) if (!current.has(seed)) {
      check(label, side + `.coin${seed}.retiredAfterObservedCollect`, true, history.collected.has(seed));
      history.retired.add(seed);
    }
    check(label, side + '.coins.inventoryMatchesObservedGrants', history.collected.size, balance - history.initialBalance);
    return uncollected + balance - history.initialBalance;
  };
  let previous;
  for (const entry of nativeSteps) {
    const label = entry.context.label, at = entry.context.presentedFrame - clock.nativeEntryPresentation;
    attempt(label, () => {
      const source = entry.observation, actual = returns.get(at);
      assert.ok(actual, 'Missing actual public step return');
      check(label, 'sourceObserver', 'pass', source.status);
      if (source.gameplay?.status === 'skip') { check(label, 'preRunLifetime', false, actual.screen === 'playing'); return; }
      assert.equal(source.gameplay?.status, 'pass'); assert.equal(source.tnt?.status, 'pass'); assert.equal(source.tntEffects?.status, 'pass'); assert.equal(source.tntAudio?.status, 'pass');
      const world = source.gameplay;
      for (const owner of ['tnt', 'tntEffects', 'tntAudio']) {
        check(label, owner + '.readOnly', false, source[owner].memoryWritten);
        check(label, owner + '.noExecutableMappings', false, source[owner].executableMappingsRead);
      }
      check(label, 'tnt.readBudget', true, source.tnt.bytesRead <= source.tnt.maxBytes);
      f32(label, 'actualGlobalSFX', 1, source.tntAudio.globalSfxVolume);
      check(label, 'screen', 'playing', actual.screen);
      check(label, 'phase', world.nativePhase.systemIterations % 2, actual.systemCounter % 2);
      check(label, 'fullGameUpdateCount', world.nativePhase.fullGameUpdates, actual.gameUpdateCount);
      check(label, 'room.descriptor', world.room.descriptorIndex, actual.room?.descriptorIndex);
      check(label, 'room.gridIndex', world.room.descriptorGridIndex, actual.room?.gridIndex);
      const nativeGrids = world.grid.filter(row => row.type === 12), actualGrids = actual.grid?.tnt ?? [];
      check(label, 'TNT.admission', nativeGrids.map(row => row.index), actualGrids.map(row => row.index));
      for (const grid of nativeGrids) {
        const owner = source.tnt.grids.find(row => row.index === grid.index), cell = actualGrids.find(row => row.index === grid.index), controller = cell?.sourceOwner?.tnt;
        assert.ok(owner, 'Actual TNT controller observation missing');
        for (const field of ['variant', 'state', 'collisionClass', 'spawnSeed']) check(label, 'TNT.' + field, grid[field], cell?.[field]);
        check(label, 'TNT.countdown', owner.countdown, controller?.countdown);
        check(label, 'TNT.animation', owner.animation, controller?.sprite?.animation);
        f32(label, 'TNT.frame', owner.frame, controller?.sprite?.frame);
        check(label, 'TNT.playing', owner.playing, controller?.sprite?.playing);
        check(label, 'TNT.pathCost', world.pathCosts[grid.index], actual.pathCosts?.[grid.index]);
        check(label, 'TNT.noReachedRareFrontier', false, Boolean(controller?.rewardFrontier));
        check(label, 'TNT.NativeSourceRollIsNonrare', true, grid.spawnSeed !== 0 && firstRareRoll(grid.spawnSeed) !== 0);
        check(label, 'TNT.BrowserSourceRollIsNonrare', true, Boolean(cell?.spawnSeed) && firstRareRoll(cell.spawnSeed) !== 0);
        seenNative.tnt = true; seenNative.states.add(grid.state); seenNative.fuse.add(owner.countdown);
        if (cell && controller) { seenBrowser.tnt = true; seenBrowser.states.add(cell.state); seenBrowser.fuse.add(controller.countdown); }
        const previousOwner = previous?.source.tnt.grids.find(row => row.index === grid.index);
        const previousController = previous?.actual.grid?.tnt.find(row => row.index === grid.index)?.sourceOwner?.tnt;
        if (previous?.at + 1 === at && previousOwner?.countdown === 1 && owner.countdown === 0)
          nativeBlasts.push({at, context: entry.context, source, previous: previous.source});
        if (previous?.at + 1 === at && previousController?.countdown === 1 && controller?.countdown === 0)
          browserBlasts.push({at, actual, previous: previous.actual});
        if (previous && previous.at + 1 === at && previous.full === world.nativePhase.fullGameUpdates) {
          const prior = previous.source.tnt.grids.find(row => row.index === grid.index);
          const priorActual = previous.actual.grid?.tnt.find(row => row.index === grid.index)?.sourceOwner?.tnt;
          if (prior) {
            check(label, 'NativeTNT.NonFullPreservesFuse', prior.countdown, owner.countdown);
            check(label, 'NativeTNT.NonFullPreservesSprite', [prior.animation, prior.frame, prior.playing], [owner.animation, owner.frame, owner.playing]);
            seenNative.fast = true;
          }
        }
        if (previous?.at + 1 === at && previous.actual.gameUpdateCount === actual.gameUpdateCount && previousController && controller) {
          check(label, 'BrowserTNT.NonFullPreservesFuse', previousController.countdown, controller.countdown);
          check(label, 'BrowserTNT.NonFullPreservesSprite', previousController.sprite, controller.sprite);
          seenBrowser.fast = true;
        }
      }
      if (world.room.descriptorIndex === data.tnt.descriptorIndex) {
        check(label, 'TNT.expectedGridIndex', [data.tnt.gridIndex], nativeGrids.map(row => row.index));
        check(label, 'TNT.expectedCatalogue', data.tnt.catalogIndex, actual.room?.catalogIndex);
        check(label, 'Native.noUnsupportedRare350', [], world.entities.filter(row => row.type === 5 && row.variant === 350));
        check(label, 'Browser.noUnsupportedRare350', [], actual.pickups.filter(row => row.variant === 350));
        // Native Entity+448 is the predicted owner after4446a0 on Fast,
        // restored by550860 before Full. Browser retains both owners.
        // The independently observed phase equality above remains mandatory.
        const position = actor => actual.systemCounter % 2 === 0 ? actor?.presentedPosition : actor?.position;
        const coins = world.entities.filter(row => row.type === 5 && row.variant === 20), actualCoins = actual.pickups.filter(row => row.variant === 20);
        const pickupWorld = source.tntEffects?.world;
        const nativeBalance = pickupWorld?.players?.[0]?.coinInventory, browserBalance = actual.inventory?.coins;
        check(label, 'coins.PlayerInventory', nativeBalance, browserBalance);
        let nativeConservation, browserConservation;
        // New owner failures must not suppress the established motion/effect
        // grades, and each game must still attempt its independent ledger.
        attempt(label + '.Native.PennyOwners', () => {
          assert.equal(pickupWorld?.status, 'pass', 'Actual Native pickup/inventory owner');
          assert.ok(Array.isArray(pickupWorld.players), 'Actual Native Player vector');
          assert.equal(pickupWorld.players.length, 1, 'One actual Native Player inventory');
          nativeConservation = observePennies('Native', label, coins, nativeBalance);
        });
        attempt(label + '.Browser.PennyOwners', () => {
          browserConservation = observePennies('Browser', label, actualCoins, browserBalance);
        });
        check(label, 'coins.nonvacuousNative', data.tnt.coinCount, nativeConservation,
          {measurement: 'Uncollected listed Pennies + actual Player coin inventory - actual initial TNT-room inventory; collected dead31f actors are excluded. Historical live-four-forever failures remain immutable.'});
        check(label, 'Browser.coins.conservation', data.tnt.coinCount, browserConservation);
        check(label, 'coins.nonvacuousBrowser', coins.length, actualCoins.length);
        for (const [index, coin] of coins.entries()) {
          const candidate = actualCoins[index];
          for (const field of ['subtype', 'spawnSeed']) check(label, `coin${index}.${field}`, coin[field], candidate?.[field]);
          check(label, `coin${index}.collectedDead31f`, coin.removed, candidate?.collected);
          for (const field of ['animation', 'animationPlaying']) check(label, `coin${index}.${field}`, coin[field], candidate?.[field]);
          f32(label, `coin${index}.animationFrame`, coin.animationFrame, candidate?.animationFrame);
          for (const [field, expected, value] of [['x', coin.position[0], position(candidate)?.x], ['y', coin.position[1], position(candidate)?.y], ['vx', coin.velocity[0], candidate?.velocity.x], ['vy', coin.velocity[1], candidate?.velocity.y]]) f32(label, `coin${index}.${field}`, expected, value);
        }
        const effectRows = source.tntEffects.actors.filter(row => row.type === 1000 && [1, 27].includes(row.variant));
        const actualEffects = actual.effects.filter(row => [1, 27].includes(row.variant));
        check(label, 'effects.currentFamilyOrder', effectRows.map(row => row.variant), actualEffects.map(row => row.variant));
        for (const [index, effect] of effectRows.entries()) {
          const candidate = actualEffects[index], identity = actual.sequenceIdentities.find(row => row.actor.kind === 'effect' && row.actor.id === candidate?.id);
          for (const [field, expected, value] of [['spawnSeed', effect.spawnSeed, candidate?.spawnSeed], ['sequenceId', effect.sequenceId, identity?.sequenceId], ['active', effect.active, candidate?.active], ['removed', effect.removed, candidate?.removed], ['animation', effect.animation, candidate?.sprite.animation], ['playing', effect.animationPlaying, candidate?.sprite.playing]]) check(label, `effect${index}.${field}`, expected, value);
          for (const [field, expected, value] of [['x', effect.position[0], position(candidate)?.x], ['y', effect.position[1], position(candidate)?.y], ['vx', effect.velocity[0], candidate?.velocity.x], ['vy', effect.velocity[1], candidate?.velocity.y], ['frame', effect.animationFrame, candidate?.sprite.frame]]) f32(label, `effect${index}.${field}`, expected, value);
          const family = effect.variant === 1 ? 'effect1' : 'wood'; seenNative[family].add(effect.sequenceId);
        }
        // Track each side independently, including unexpected extra births.
        for (const effect of actualEffects) seenBrowser[effect.variant === 1 ? 'effect1' : 'wood'].add(effect.id);
      }
      previous = {at, full: world.nativePhase.fullGameUpdates, source, actual};
    });
  }
  check('coverage', 'Native.oneExactFuseZeroTransition', 1, nativeBlasts.length);
  check('coverage', 'Browser.oneExactFuseZeroTransition', 1, browserBlasts.length);
  check('coverage', 'sameDeclaredBlastPresentation', nativeBlasts.map(row => row.at), browserBlasts.map(row => row.at));
  for (const [side, events, seen] of [['Native', nativeBlasts, seenNative], ['Browser', browserBlasts, seenBrowser]]) attempt(side + '.blastImpulse', () => {
    assert.equal(events.length, 1, 'An actual consecutive 1→0 fuse transition is required');
    const event = events[0];
    const oldCoins = side === 'Native' ? event.previous.gameplay.entities.filter(row => row.type === 5 && row.variant === 20) : event.previous.pickups.filter(row => row.variant === 20);
    const newCoins = side === 'Native' ? event.source.gameplay.entities.filter(row => row.type === 5 && row.variant === 20) : event.actual.pickups.filter(row => row.variant === 20);
    const velocity = row => side === 'Native' ? row.velocity : [row.velocity.x, row.velocity.y];
    check('coverage', side + '.impulseHasFourRetainedCoins', data.tnt.coinCount, oldCoins.length);
    check('coverage', side + '.impulseCoinOwnersStable', oldCoins.map(row => row.spawnSeed), newCoins.map(row => row.spawnSeed));
    check('coverage', side + '.coinsStationaryBeforeBlast', true, oldCoins.length === data.tnt.coinCount && oldCoins.every(row => velocity(row).every(value => value === 0)));
    const pathCost = side === 'Native' ? event.source.gameplay.pathCosts[data.tnt.gridIndex] : event.actual.pathCosts?.[data.tnt.gridIndex];
    check('coverage', side + '.postBlastMarkerChangedFromInit1300', true, Number.isInteger(pathCost) && pathCost !== 1300);
    seen.coinsMove = newCoins.length === data.tnt.coinCount && newCoins.every(row => velocity(row).some(value => value !== 0));
  });
  for (const [side, seen] of [['Native', seenNative], ['Browser', seenBrowser]]) {
    check('coverage', side + '.actualTNT', true, seen.tnt); check('coverage', side + '.hurtState4', true, seen.states.has(4));
    check('coverage', side + '.fuse3_2_1_0', [true, true, true, true], [3, 2, 1, 0].map(value => seen.fuse.has(value)));
    check('coverage', side + '.oneExplosion1', 1, seen.effect1.size);
    check('coverage', side + '.wood2to4', true, seen.wood.size >= 2 && seen.wood.size <= 4);
    check('coverage', side + '.coinImpulse', true, seen.coinsMove); check('coverage', side + '.NonFullInactivity', true, seen.fast);
  }
  attempt('SFX182', () => {
    const starts = calls.filter(row => row.api === 'alSourcePlay' && row.sfxOwner?.references?.some(ref => ref.id === 182));
    check('SFX182', 'oneActualNativeBackendStart', 1, starts.length);
    const output = observed.runtimeAudio.events.filter(row => row.id === 182);
    check('SFX182', 'oneActualBrowserRequest', 1, output.length);
    for (const [index, event] of starts.entries()) {
      const owner = event.sfxOwner, ref = owner.references[0], actual = output[index], sample = samples[ref.sampleIndex];
      assert.equal(nativeBlasts.length, 1); assert.equal(browserBlasts.length, 1);
      const blast = nativeBlasts[0], candidateBlast = browserBlasts[0];
      check('SFX182', 'actualNativeClockDomain', 'gate-main-thread', event.clockDomain);
      check('SFX182', 'sameFullAsActualBlast', blast.source.gameplay.nativePhase.fullGameUpdates, owner.gameUpdateCount);
      const beforeBlast = nativeSteps.find(row => row.context.presentedFrame === blast.context.presentedFrame - 1);
      assert.ok(beforeBlast, 'Actual preceding presentation required for bounded SFX window');
      check('SFX182', 'actualStartWithinBlastPresentation', true,
        Number.isSafeInteger(event.virtualElapsedNs) && event.virtualElapsedNs > beforeBlast.context.virtualElapsedNs && event.virtualElapsedNs <= blast.context.virtualElapsedNs);
      const admitted = candidateBlast.actual.audioEvents.filter(row => row.id === 182);
      check('SFX182', 'sameCallActualBrowserAdmission', 1, admitted.length);
      assert.equal(admitted.length, 1);
      check('SFX182', 'sameAdmittedBrowserSequence', admitted[0].sequence, actual?.sequence);
      check('SFX182', 'sameAdmittedBrowserCounter', admitted[0].systemCounter, actual?.systemCounter);
      assert.ok(sample && actual, 'Missing configured182 owner/sample or actual Browser request');
      check('SFX182', 'uniqueActualOwner', 'UNIQUE_ACTUAL_TABLE_REFERENCE', owner.status);
      check('SFX182', 'oneTableReference', 1, owner.references.length); check('SFX182', 'actualRequestId', 182, ref.id);
      check('SFX182', 'allocated', true, event.actualSourceAllocationKnown); check('SFX182', 'allocationCap', false, event.sourceAllocationOverflow);
      check('SFX182', 'actualNativePlaying', 0x1012, event.postPlayState); check('SFX182', 'actualMixerComplete', true, event.actualMixer?.complete);
      check('SFX182', 'configuredPath', 'resources/' + sample.path, ref.path?.toLowerCase());
      check('SFX182', 'requestVolume', word(1), ref.requestedVolumeBits); check('SFX182', 'delay2', 2, ref.delayBits);
      check('SFX182', 'mixerGain', word(1), event.actualMixer?.gainBits); check('SFX182', 'mixerPitch', word(1), event.actualMixer?.pitchBits);
      check('SFX182', 'mixerLoop', 0, event.actualMixer?.loopRaw);
      check('SFX182', 'actualSampleIndex', ref.sampleIndex, actual.sampleIndex);
      check('SFX182', 'actualBrowserPath', sample.path, actual.path); check('SFX182', 'actualBrowserOutput', 'started', actual.output);
      check('SFX182', 'actualBrowserPCM', sample.pcmSha256, actual.pcmSha256);
      const upload = calls.findLast(row => row.api === 'alBufferData' && row.buffer === event.actualMixer?.buffer && row.seq < event.seq);
      assert.ok(upload, 'Missing submitted Native PCM owner');
      check('SFX182', 'nativePCMComplete', false, upload.truncated); check('SFX182', 'nativePCMBytes', sample.pcmBytes, upload.artifactBytes);
      check('SFX182', 'nativeSubmittedPCMBytes', sample.pcmBytes, upload.bytes);
      check('SFX182', 'nativePCMFormat', sample.channels === 1 ? 0x1101 : 0x1103, upload.format);
      check('SFX182', 'nativePCMRate', sample.sampleRate, upload.rate);
      check('SFX182', 'nativePCM', sample.pcmSha256, hash(fs.readFileSync(path.join(nativeAudioDirectory, upload.pcmArtifact))));
      const api = observed.api.events.filter(row => row.type === 'start' && row.pcmSha256 === sample.pcmSha256 && row.observedAdmittedSounds?.some(sound => sound.id === 182));
      check('SFX182', 'oneSameCallBrowserAPI', 1, api.length);
      for (const row of api) {
        check('SFX182', 'browserBackendCounter', candidateBlast.actual.systemCounter, row.observedSystemCounter);
        check('SFX182', 'browserSameCallRequestOwner', admitted, row.observedAdmittedSounds?.filter(sound => sound.id === 182));
        check('SFX182', 'browserAPIRate', sample.sampleRate, row.sampleRate);
        check('SFX182', 'browserAPIChannels', sample.channels, row.channels);
        check('SFX182', 'browserAPIFrames', sample.pcmBytes / sample.channels / 2, row.frames);
        check('SFX182', 'browserAPIImmediateStart', 0, row.when);
        check('SFX182', 'browserAPIDestination', true, row.reachesDestination);
        f32('SFX182', 'browserAPIGain', 1, row.gain); f32('SFX182', 'browserAPIPitch', 1, row.pitch);
        check('SFX182', 'browserAPILoop', false, row.looping);
      }
    }
  });
  diagnostics.push({field: 'transient18.seedAndCallback', status: 'fail', original: 'UNOBSERVED_PRESENTATION_END_RAM',
    browserBackground18: Math.max(0, ...(observed.returns?.rows ?? []).map(row => row.background?.filter(effect => effect.variant === 18).length ?? 0)),
    scope: 'No missing-equals-zero/pass. Later actual GL/FBO paint witness is separate; no executable hook.'});
  diagnostics.push({field: 'fullRGBA', status: 'not-computed', scope: 'Actual selected Native/Browser PNGs retained; no pixel equality claimed'});
  return {schema: 1, status: checks.length && checks.every(row => row.status === 'pass') ? 'pass' : 'fail',
    scope: data.tnt.scope, checks, diagnostics, errors,
    remaining: ['transient18 actual seed/callback', 'rare trinket50/41 pool and350 pickup', 'mixed TNT/bomb-rock/spike callback order', 'PlayerTNTdamage/death', 'room reentry', 'whole shared MT', 'fullRGBA']};
}
