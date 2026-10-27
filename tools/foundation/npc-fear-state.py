"""Additive Horf Fear laws for the existing live active-items comparer.

This module only grades actual canonical journals; it generates no inputs and
is not a standalone component harness. Missing owners remain unavailable.
"""
import collections


class FearComparison:
    def __init__(self, case, check, integer, boolean, f32, vector):
        self.case, self.check = case, check
        self.integer, self.boolean, self.f32, self.vector = integer, boolean, f32, vector
        self.contract = case['activeItems']['npcFear']
        assert self.contract['schema'] == 'source-public-npc-fear-v1'

    def random(self, value):
        index = self.integer(value['index'])
        words = [self.integer(word) for word in value['words']]
        assert 0 <= index <= 624 and len(words) == 624 and all(0 <= word <= 0xffffffff for word in words)
        return dict(index=index, words=words)

    def native(self, base, observation):
        extra = observation['npcFear']
        assert extra['status'] == 'pass', 'Complete independent Fear owners required'
        measured = {(self.integer(row['sequenceId']), self.integer(row['spawnSeed'])): row
                    for row in extra['actors']}
        colors = {(row['sequenceId'], row['spawnSeed']): row['timedColors']
                  for row in observation['npcFreeze']['actors']}
        combat = {(row['sequenceId'], row['spawnSeed']): row for row in observation['combat']['entities']}
        for actor in base['actors']:
            actual = measured.pop((actor['lifetimeId'], actor['key'][3]))
            flags = self.integer(actual['entityFlags'])
            actor.update(fearTimer=self.integer(actual['fearTimer']), feared=bool(flags & 0x800),
                statusMask=flags & 0x1fe0,
                pathVectors={key:self.vector(actual['vectors'][key]) for key in ('target', 'wanderDirection10')},
                nativePreservedPath=actual['preservedPath'], sequenceId=actor['lifetimeId'],
                nativeTimedColors=colors[(actor['lifetimeId'], actor['key'][3])],
                gridContact=self.boolean(combat[(actor['lifetimeId'], actor['key'][3])]['collidedWithGrid']))
            assert actor['applicationLockout'] == self.integer(actual['applicationLockout'])
        assert not measured, 'Every Fear actor joins the independent room list'
        base.update(sourceEffectMode=self.integer(extra['sourceEffectMode']),
            sharedRandom=self.random(extra['sharedRandom']),
            projectiles={(self.integer(row['sequenceId']), self.integer(row['spawnSeed']))
                         for row in observation['combat']['entities'] if row['type'] == 9},
            # Ordinary Horf shots need not have Entity.parent+498 set. This
            # one-Horf fixture has no other authored projectile producer;
            # retain all actual type9 births instead of inventing a parent.
            horfProjectiles={(self.integer(row['sequenceId']), self.integer(row['spawnSeed']))
                for row in observation['combat']['entities'] if row['type'] == 9})
        return base

    def browser(self, base, state, gp):
        measured = {(row['id'], row['spawnSeed']):row for row in state['room']['npcs']}
        identities = {row['actor']['id']:row['sequenceId'] for row in state['actorScheduler']['identities']
                      if row['actor']['kind'] == 'npc'}
        for actor in base['actors']:
            actual = measured.pop((actor['lifetimeId'], actor['key'][3]))
            status = actual.get('status')
            try:
                assert type(status) is dict
                vectors = actual['pathfinderVectors']
                assert actual['pathfinder'] is None and type(vectors) is dict, 'Horf has exactly its narrow embedded owner'
                actor.update(fearTimer=self.integer(status['fearTimer']), feared=self.boolean(status['feared']),
                    pathVectors={key:self.vector(vectors[key]) for key in ('target', 'wanderDirection10')},
                    sequenceId=self.integer(identities[actor['lifetimeId']]),
                    gridContact=self.boolean(actual['gridCollided']))
            except (KeyError, TypeError, ValueError, AssertionError) as error:
                actor.update(statusAvailable=False, fearUnavailable=str(error))
                self.check('Own:fear actual timer/flag/vector owners', str(error), 'complete actual owners', gp)
        assert not measured
        base.update(sourceEffectMode=self.integer(state['runSeeds']['sourceEffectMode']),
            sharedRandom=self.random(state['bootLifecycle']['shared']),
            projectiles={(self.integer(row['id']), self.integer(row['spawnSeed'])) for row in state['enemyProjectiles']},
            horfProjectiles={(self.integer(row['id']), self.integer(row['spawnSeed']))
                for row in state['enemyProjectiles']})
        return base

    def paired(self, own, native, gp):
        check = self.check
        for field in ('sourceEffectMode', 'sharedRandom'):
            check(f'NPC fear paired {field}', own[field], native[field], gp)
        left = {tuple(row['key'][:3]):row for row in own['actors']}
        right = {tuple(row['key'][:3]):row for row in native['actors']}
        check('NPC fear paired target families', set(left), set(right), gp)
        for key in sorted(set(left) & set(right)):
            for field in ('key', 'position', 'presentedPosition', 'velocity', 'hp', 'maxHp',
                    'baseFriction', 'currentFriction', 'freezeTimer', 'frozen', 'fearTimer', 'feared',
                    'applicationLockout', 'state', 'frameAge', 'animation', 'animationClock',
                    'animationPlaying', 'visible', 'dead', 'raw494', 'boss', 'champion', 'parentNull',
                    'childNull', 'entityCollisionClass', 'spriteColor', 'colorStack', 'pathVectors', 'shooter',
                    'sequenceId', 'gridContact'):
                check(f'NPC{key}:fear {field}', left[key].get(field, {'unavailable':field}), right[key][field], gp)
            if left[key]['baseUpdateAdmitted'] is True:
                check(f'NPC{key}:fear admitted current470', left[key]['friction'], right[key]['friction'], gp)

    def witnesses(self, name, rows, callback):
        check, f32 = self.check, self.f32
        expected = {(row['type'], row['variant'], row['subtype']) for row in self.contract['targets']}
        assert expected == {(12, 0, 0)}, 'First Fear route is explicitly Horf only'
        key = (12, 0, 0)
        control = self.contract['control']
        space, cursor = [], 0
        for action in self.case['actions']:
            if 'space' in action['keys']:
                space.append((cursor, cursor + action['presentations'] + 1))
            cursor += action['presentations']
        purple = [f32(.5), f32(.1), f32(.5), f32(1)] + [f32(0)] * 7
        identity = hp = previous = prior_gp = first = phase = expiry = resume = post_fast = shooter_resume = None
        first_full = first_fast = None
        first_boundary_complete, unavailable_slots, unclassified_slots = True, 0, 0
        held = used = use_animation = pre_shot = anm_progress = random_progress = False
        pre_shooter, pre_anm = set(), set()
        blocked, full_count, fast_count = collections.Counter(), 0, 0
        native_path_sentinels, zero_vectors, control_random = False, True, False
        application_vectors = None
        for gp in sorted(rows):
            player, value = rows[gp], rows[gp]['npcFear']
            check(f'{name}:fear actual sourceEffectMode0', value['sourceEffectMode'], 0, gp)
            information_available = 'unavailable' not in value['informationAnimation']
            check(f'{name}:fear information owner available', information_available, gp=gp)
            unavailable_slots += int(not information_available)
            actors = {tuple(row['key'][:3]):row for row in value['actors']}
            if set(actors) != expected and identity is None:
                continue
            check(f'{name}:fear exact living target roster', set(actors), expected, gp)
            if set(actors) != expected:
                if first is not None and (first_full is None or first_fast is None):
                    first_boundary_complete = False
                previous = prior_gp = None
                continue
            actor = actors[key]
            if identity is None:
                identity, hp = [actor['key'], actor['lifetimeId'], actor.get('sequenceId')], actor['hp']
            check(f'{name}:fear stable target identity', [actor['key'], actor['lifetimeId'], actor.get('sequenceId')], identity, gp)
            check(f'{name}:fear unchanged target HP', actor['hp'], hp, gp)
            check(f'{name}:fear status owner available', actor['statusAvailable'], gp=gp)
            if not actor['statusAvailable'] or 'fearTimer' not in actor:
                unavailable_slots += 1
                if first is not None and (first_full is None or first_fast is None):
                    first_boundary_complete = False
                previous = prior_gp = None
                continue
            held |= player['activeId'] == 41 and player['activeCharge'] == 3
            used |= any(row[0] == 41 for row in player['temporary']) and player['activeCharge'] == 0
            before_use = not space or gp < space[0][0]
            use_animation |= not before_use and player['clip'] == 'UseItem'
            if before_use:
                pre_shooter.add(tuple(actor['shooter']))
                pre_anm.add((actor['animation'], actor['animationClock'], actor['animationPlaying']))
                pre_shot |= bool(value['horfProjectiles'])
            actual = callback(name, value, rows[prior_gp]['npcFear'], gp, 'fear') if previous is not None and gp == prior_gp + 1 else None
            if previous is not None:
                classified = gp == prior_gp + 1 and actual is not None and information_available
                check(f'{name}:fear actual consecutive callback classified', classified, gp=gp)
                unclassified_slots += int(not classified)
                if first is not None and (first_full is None or first_fast is None) and not classified:
                    first_boundary_complete = False
            if previous is not None:
                control_random |= value['sharedRandom'] != rows[prior_gp]['npcFear']['sharedRandom']
            if actual in ('BlockedFull', 'BlockedFast'):
                blocked[actual] += 1
                for field in ('position', 'presentedPosition', 'velocity', 'currentFriction', 'fearTimer',
                        'feared', 'freezeTimer', 'frozen', 'applicationLockout', 'shooter',
                        'animation', 'animationClock', 'animationPlaying', 'colorStack', 'spriteColor', 'pathVectors'):
                    check(f'{name}:fear information-blocked {field} held', actor[field], previous[field], gp)
            if control:
                if held:
                    check(f'{name}:fear control retains held41/full3 after acquisition',
                        [player['activeId'], player['activeCharge']], [41, 3], gp)
                check(f'{name}:fear control zero independent statuses/lockout', [actor['freezeTimer'], actor['frozen'],
                    actor['fearTimer'], actor['feared'], actor['applicationLockout']], [0, False, 0, False, 0], gp)
                if name == 'Native':
                    check('Native:fear control no source status bits', actor['statusMask'], 0, gp)
                check(f'{name}:fear control no TEMP41 or discharge', not any(row[0] == 41 for row in player['temporary'])
                      and (player['activeId'] != 41 or player['activeCharge'] == 3), gp=gp)
                if name == 'Own':
                    check('Own:fear control ordinary Base owner retained', actor['baseUpdateAdmitted'], False, gp)
            if actor['feared'] and first is None:
                first, phase = gp, actual
                first_boundary_complete &= information_available and previous is not None and actual in ('Full', 'Fast')
                check(f'{name}:fear application physical Space', any(start <= gp <= end for start, end in space), gp=gp)
                check(f'{name}:fear prior actual target', previous is not None, gp=gp)
                check(f'{name}:fear actual application callback', actual in ('Full', 'Fast'), gp=gp)
                check(f'{name}:fear target ordinary eligible past Appear', actor['state'] != 1 and actor['visible']
                    and not actor['dead'] and not actor['boss'] and not actor['champion']
                    and actor['parentNull'] and actor['childNull'] and not actor['raw494'] and actor['hp'] > 0
                    and actor['entityCollisionClass'] not in (0, 1), gp=gp)
                if previous is not None:
                    application_vectors = dict(before=previous['pathVectors'], after=actor['pathVectors'])
                    prior_player = rows[prior_gp]
                    check(f'{name}:fear held41 charge3 before use', [prior_player['activeId'], prior_player['activeCharge']], [41, 3], gp)
                    check(f'{name}:fear transfers actual current470', previous['currentFriction'], previous['baseFriction'], gp)
                    check(f'{name}:fear application retains shooter', actor['shooter'], previous['shooter'], gp)
                    check(f'{name}:fear no prior Freeze or Fear', [previous['freezeTimer'], previous['frozen'],
                        previous['fearTimer'], previous['feared']], [0, False, 0, False], gp)
                    if name == 'Native':
                        check('Native:fear application preserves other path fields', actor['nativePreservedPath'], previous['nativePreservedPath'], gp)
                        native_path_sentinels |= any(word not in (0, False, [0., 0.]) for word in previous['nativePreservedPath'].values())
                        check('Native:fear prior ordinary eligibility flags', previous['immune'] is False
                            and previous['eligibleFlag'] and previous['statusMask'] == 0, gp=gp)
                if actual in ('Full', 'Fast'):
                    check(f'{name}:fear application timer120/119', actor['fearTimer'], 119 if actual == 'Full' else 120, gp)
                if actual == 'Fast':
                    check(f'{name}:fear immediate Fast vectors zero', actor['pathVectors'],
                          dict(target=[0., 0.], wanderDirection10=[0., 0.]), gp)
            if first is not None:
                if name == 'Own':
                    check('Own:fear reached Base owner retained', actor['baseUpdateAdmitted'], True, gp)
                check(f'{name}:fear current470 reset from base46c', actor['currentFriction'], actor['baseFriction'], gp)
                check(f'{name}:fear shared420 remains zero', actor['applicationLockout'], 0, gp)
                check(f'{name}:fear independent Freeze remains clear', [actor['freezeTimer'], actor['frozen']], [0, False], gp)
                if name == 'Native':
                    check('Native:fear no other source status bits', actor['statusMask'], 0x800 if actor['feared'] else 0, gp)
                if actor['fearTimer'] > 0:
                    check(f'{name}:fear actual purple timed color', any(row['priority'] == 255 and row['color'] == purple
                        and row['duration'] == 120 and row['remaining'] == actor['fearTimer'] for row in actor['colorStack']), gp=gp)
                    check(f'{name}:fear actual current purple', actor['spriteColor'], purple, gp)
                    if name == 'Native':
                        check('Native:fear actual no-fade propagated color', any(row['priority'] == 255
                            and row['color'] == purple and not row['fade'] and row['propagate']
                            for row in actor['nativeTimedColors']), gp=gp)
                zero_vectors &= all(not any(vector) for vector in actor['pathVectors'].values())
            if previous is not None and first is not None and actual in ('Full', 'Fast'):
                if actual == 'Full' and first_full is None:
                    first_full = gp
                if actual == 'Fast' and first_fast is None:
                    first_fast = gp
                if gp > first:
                    check(f'{name}:fear countdown and cleared-state law', actor['fearTimer'],
                        max(0, previous['fearTimer'] - int(actual == 'Full')), gp)
                    check(f'{name}:fear cleared-state flag follows timer', actor['feared'], actor['fearTimer'] > 0, gp)
                if previous['fearTimer'] > 0:
                    check(f'{name}:fear countdown only actual Full', actor['fearTimer'], max(0, previous['fearTimer'] - int(actual == 'Full')), gp)
                    check(f'{name}:fear flag follows timer', actor['feared'], actor['fearTimer'] > 0, gp)
                    check(f'{name}:fear A00 gate holds Horf timers', actor['shooter'], previous['shooter'], gp)
                    check(f'{name}:fear no new Horf projectiles', value['horfProjectiles'] <= rows[prior_gp]['npcFear']['horfProjectiles'], gp=gp)
                    if actual == 'Full':
                        full_count += 1
                        anm_progress |= (actor['animation'], actor['animationClock'], actor['animationPlaying']) != (previous['animation'], previous['animationClock'], previous['animationPlaying'])
                        random_progress |= value['sharedRandom'] != rows[prior_gp]['npcFear']['sharedRandom']
                    else:
                        fast_count += 1
                if actual == 'Full':
                    velocity = [f32(word * f32(previous['currentFriction'] * f32(.7))) for word in previous['velocity']]
                    check(f'{name}:fear ordinary Horf Full velocity', actor['velocity'], velocity, gp)
                    check(f'{name}:fear ordinary Full integration', actor['position'], [f32(p + v) for p, v in zip(previous['position'], velocity)], gp)
                    if previous['fearTimer'] == 1:
                        expiry = gp
                        check(f'{name}:fear timer1 Full clears timer/flag', [actor['fearTimer'], actor['feared']], [0, False], gp)
                        check(f'{name}:fear expiry removes purple entry', not any(row['priority'] == 255 for row in actor['colorStack']), gp=gp)
                    elif expiry is not None and resume is None:
                        resume = gp
                        # OnFrame7 may legitimately hold cooldown on this first
                        # callback. Check the forced timer laws, then require a
                        # later actual change instead of inventing frame+1.
                        cooldown, delay = previous['shooter']
                        gate = ((value['fullUpdates'] + actor['sequenceId']) & 0xffffffff) % 7 == 0
                        if cooldown > 0 or not gate:
                            check(f'{name}:fear next Full retained cooldown law', actor['shooter'][0], cooldown - int(gate and cooldown > 0), gp)
                            check(f'{name}:fear next Full retained delay law', actor['shooter'][1], delay - 1 if delay > 0 else -1 if delay == 0 else delay, gp)
                        else:
                            check(f'{name}:fear next Full resumed selection or retained delay', actor['shooter'] in
                                ([4, 6], [0, delay - 1 if delay > 0 else -1 if delay == 0 else delay]), gp=gp)
                    if expiry is not None and gp > expiry and actor['shooter'] != previous['shooter'] and shooter_resume is None:
                        shooter_resume = gp
                else:
                    check(f'{name}:fear admitted Fast physical hold', actor['position'], previous['position'], gp)
                    check(f'{name}:fear admitted Fast velocity hold', actor['velocity'], previous['velocity'], gp)
                    check(f'{name}:fear admitted Fast ANM hold', [actor['animation'], actor['animationClock'], actor['animationPlaying']],
                        [previous['animation'], previous['animationClock'], previous['animationPlaying']], gp)
                    preview = [f32(p + f32(f32(v * previous['currentFriction']) / f32(2))) for p, v in zip(previous['position'], previous['velocity'])]
                    check(f'{name}:fear admitted Fast current470 preview', actor['presentedPosition'], preview, gp)
                    if resume is not None:
                        post_fast = gp
            previous, prior_gp = actor, gp
        check(f'{name}:fear actual held41/full3', held)
        check(f'{name}:fear preuse Horf shooter progression', len(pre_shooter) > 1)
        check(f'{name}:fear preuse Horf ANM progression', len(pre_anm) > 1)
        check(f'{name}:fear preuse Horf emitted actual projectile', pre_shot)
        if control:
            check(f'{name}:fear control actual ordinary shared draws', control_random)
            check(f'{name}:fear control complete target tape', identity is not None and max(rows, default=0) >= cursor - 4)
        if not control:
            check(f'{name}:fear actual TEMP41/discharge', used)
            check(f'{name}:fear actual UseItem animation', use_animation)
            check(f'{name}:fear actual positive application', first is not None)
            check(f'{name}:fear first admitted Full boundary witnessed', first is not None and first_full is not None)
            check(f'{name}:fear first admitted Fast witnessed', first_fast is not None)
            check(f'{name}:fear complete first admitted Full/Fast boundaries', first_boundary_complete
                and first_full is not None and first_fast is not None)
            check(f'{name}:fear continuing ANM under Fear', anm_progress)
            check(f'{name}:fear continuing shared draws under Fear', random_progress)
            check(f'{name}:fear countdown Full coverage', full_count >= 119)
            check(f'{name}:fear admitted Fast coverage', fast_count > 0)
            check(f'{name}:fear expiry and next Full resumption', expiry is not None and resume is not None)
            check(f'{name}:fear actual postexpiry shooter progression', shooter_resume is not None)
            check(f'{name}:fear postexpiry admitted Fast', post_fast is not None)
            check(f'{name}:fear complete expiry margin', first is not None and max(rows, default=0) > first + 300 and post_fast is not None)
        return dict(control=control, applicationPhase=phase, applicationPresentation=first,
            firstFullPresentation=first_full, firstFastPresentation=first_fast,
            firstBoundaryComplete=first_boundary_complete, unavailableOwnerSlots=unavailable_slots,
            unclassifiedCallbackSlots=unclassified_slots,
            informationBlockedSlots=dict(blocked), fearedFullCallbacks=full_count,
            fearedFastCallbacks=fast_count, expiryPresentation=expiry, resumePresentation=resume,
            shooterProgressPresentation=shooter_resume,
            nativeNonzeroPathSentinelsObserved=native_path_sentinels if name == 'Native' else None,
            applicationVectors=application_vectors, postApplicationVectorsZeroThroughout=zero_vectors,
            limits=['First-use zeros do not witness a reset write.',
                'Unchanged Horf constructor sentinels do not prove preservation of a dynamically changed full Pathfinder.',
                'Other Horf Pathfinder state is Native-only; Browser owns just the two vectors.',
                'Shared MT advancement does not attribute Horf\'s ephemeral angle call.',
                'JSON can erase negative zero; erased sign bits are unobservable.'])
