"""Monstro229 real charge/release cohorts, independently in each captured game.

Exact crossgame identities/seeds are never replaced by burst/ordinal order.
A state chain is not a factory, RNG, callback or immediate-Full event witness.
"""
import math

SCHEMA = 'monstro229-charge-release-kind1-burst-v1'


def integer(value):
    if type(value) is not int:
        raise ValueError('Actual integer owner required')
    return value


def boolean(value):
    if type(value) is not bool:
        raise ValueError('Actual boolean owner required')
    return value


class MonstrosLungComparer:
    def __init__(self, shared, case):
        self.b, self.case = shared, case
        self.declaration = case['requires']['monstrosLungFunctional']
        if self.declaration['schema'] != SCHEMA:
            raise ValueError('Explicit Monstro229 marker required')
        c = case['requires']['sourceExpectedActorContract']
        if tuple(c[k] for k in ('primaryC08','family','variant','subtype','controlledBy329')) != (7,2,1,0,False):
            raise ValueError('Exact C08=7 and neutral2/1/0 contract required')
        if self.declaration['expectedTearsPerNeutralBurst'] != 13:
            raise ValueError('Selected neutral caller requires13 actual actors')
        self.own = {side: dict(last=None,pending=set(),grants=[],stages=set(),births={},bodies=[],
            undercharged=[],bursts=[],birthCohorts=[],saturatedHeld=[],chargeRows=[],diagonal={0:[],1:[]},bodyJoins=0)
            for side in ('native','port')}

    def action(self, gameplay_presentation):
        if type(gameplay_presentation) is not int or gameplay_presentation <= self.case['initialPhaseOffset']:
            return None
        end = self.case['initialPhaseOffset']
        for a in self.case['actions']:
            end += integer(a['presentations'])
            if gameplay_presentation <= end:
                return a
        raise ValueError('Actual own gameplay boundary outside frozen complete tape')

    def port_identity(self, state, actor):
        ref = dict(kind='tear',id=integer(actor['id']))
        ids = [x for x in state['actorScheduler']['identities'] if x['actor'] == ref]
        if len(ids) != 1:
            raise ValueError('Actual canonical Tear scheduler identity required')
        return (2,integer(actor['sourceVariant']),0,integer(ids[0]['sequenceId']),integer(actor['spawnSeed']))

    def own_snapshot(self,side,source,state,player,t,native_counter,gameplay_presentation):
        b,h,native = self.b,self.own[side],side=='native'
        f,label = b['f32'],'Monstro229 '+side+' '
        weapon = source['players'][0] if native else state['attackRuntime']
        owner = player if native else state['playerItems']
        counts = owner['ownedCounts']
        if len(counts)!=347 or any(type(x)is not int for x in counts):
            raise ValueError('Actual complete signed inventory required')
        counter = integer(native_counter if native else state['systemClock']['systemCounter'])
        age = integer(source['gameAge'] if native else state['gameUpdateCount'])
        primary = integer(weapon['primaryWeaponC08'] if native else state['weaponCache']['primary'])
        charge = integer(weapon['chargeC0c'] if native else weapon['chargeC0c'])
        maximum = integer(player['cachedStats']['maxFireDelay'] if native else state['maxFireDelay'])
        stats = player['cachedStats'] if native else state['playerStats']
        cache = (weapon['attackFlagsC98'],tuple(f(x)for x in weapon['cachedColorCA0']),f(stats['damage'])) if native else (
            b['flags'](state['weaponCache']['flagsC98']),tuple(b['color'](state['weaponCache']['colorCa0'])),f(stats['damage']))
        pending = integer(owner['cacheFlags'] if native else owner['unevaluatedCacheFlags'])
        queued = owner['queuedId']
        shots = integer(player['shotsFired'] if native else state['player']['shotsFired'])
        # Legacy Native DB0 is opposed-pair, not the BPL HasShoot argument.
        # Its actual actions4..7 query result owner is absent from this capture.
        held = None if native else boolean(state['player']['shooting'])
        raw_opposed_db0 = boolean(player['shootingPressed']) if native else None
        if queued is not None:
            h['pending'].add(integer(queued))
        old = h['last']
        if old is not None:
            for item in self.case['requires']['orderedRealPedestalGrants']:
                if old['counts'][item]==0 and counts[item]==1 and item in h['pending'] and queued is None and pending==0:
                    h['grants'].append(dict(itemId=item,presentation=t,scope='Own real queued config then count0→1/cache-complete state; not atomic Add order'))
                    h['pending'].discard(item)
        stage = None
        for i,s in enumerate(self.case['requires']['actorStages']):
            expected = [0]*347
            for entry in s['ownedCounts']:
                expected[integer(entry['itemId'])]=integer(entry['count'])
            if expected==counts:
                stage=i;h['stages'].add(i)
        ready = primary==7 and counts[229]>0 and queued is None and pending==0 and stage is not None
        if native:
            caller = weapon['monstroCaller']
            controls = boolean(player['controlsEnabled']) and boolean(weapon['canShoot3075']) and integer(player['inputCooldown'])<=0 and not boolean(player['pickupAnimationGuard']) and integer(player['activeStateD80'])==0 and integer(caller['shootingLock7dc'])==0
        else:
            controls = boolean(state['player']['controlsEnabled']) and boolean(state['player']['canShoot']) and integer(state['player']['inputCooldown'])<=0 and not boolean(owner['pickupAnimationGuard']) and not boolean(state['playerAnimation']['guard']) and integer(owner['activeStateD80'])==0 and integer(state['player']['shootingLock'])==0
        ready = ready and controls
        if not native:
            #record_unadmitted retains the first historical diagnostic. It is
            #not a current-callback eligibility owner and cannot veto observed
            #canonical births after later admitted callbacks. Keep its failure
            #separate from actual current cache/controls/cleanup readiness.
            b['check'](label+'historical first controller diagnostic absent', None,
                state['weaponAdmission']['unadmittedController'], t)
            ready = ready and state['weaponAdmission']['pendingPrimaryCleanup'] is None
        # Native's actual319&40 guards the source C0C increment. Port's
        # qualified Phase.Clock selects the callback before counter increments;
        # its after-gate odd counter is Update. This is not raw Tear322.
        # Manager454F54 clears319bit40 after the callback, before observation.
        # Keep Native callback phase unknown; port Phase.Clock is independent.
        callback_fast = None if native else counter%2==0
        record = dict(presentation=t,counter=counter,age=age,counts=list(counts),queued=queued,pending=pending,
            primary=primary,charge=charge,maximum=maximum,cache=cache,shots=shots,held=held,ready=ready,
            db4=b['vector'](weapon['shootingAimDb4']),gameplayPresentation=gameplay_presentation,callbackFast=callback_fast,
            rawOpposedPairDb0=raw_opposed_db0,
            callbackPhaseOwner='unobservedAfterManagerClears319'if native else'qualifiedPhaseClockAfterGateCounter')
        actors = ([a for a in source['attacks'] if b['identity_tuple'](a['identity'])[:3]==(2,1,0) and boolean(a['inActiveManager'])]
            if native else [a for a in state['tears'] if a['sourceVariant']==1 and dict(kind='tear',id=a['id'])in state['actorScheduler']['actors']])
        born = []
        current_actors = {}
        for a in actors:
            identity = b['identity_tuple'](a['identity']) if native else self.port_identity(state,a)
            current_actors[identity]=a
            props = (a['attackFlags7f0'],tuple(f(x)for x in a['permanentColor3a0']),f(a['collisionDamage'])) if native else (
                b['flags'](a['attackFlags']),tuple(b['color'](a['sourceColor'])),f(a['collisionDamage']))
            birth_age = integer(a['bornGameAge'] if native else a['birthGameUpdate'])
            actual_fast = boolean(a['fast322']) if native else (
                boolean(a['fast'])if 'fast'in a and a['fast']is not None else None)
            if ready and counts[4]>0:
                h['bodies'].append(dict(presentation=t,identity=identity,actualFast322=actual_fast,
                    counter=counter,properties=props,bornGameAge=birth_age))
            if identity in h['births']:
                b['check'](label+'same own Tear retains born flags/color/damage',h['births'][identity]['properties'],props,t)
            if ready and birth_age==age and identity not in h['births']:
                dense = old is not None and old['presentation']+1==t and old['counter']+1==counter and old['ready'] and old['counts']==counts and old['cache']==cache
                ordinal = integer(a['monstroShotOrdinal838'] if native else a['shotOrdinal'])
                witness = dict(identity=identity,presentation=t,bornGameAge=birth_age,properties=props,
                    cache=cache,counts=list(counts),ordinal=ordinal,densePriorCache=dense,
                    scope='Actual own canonical identity and birthGameAge. No crossgame ordinal/body remapping or factory event inferred.')
                h['births'][identity]=witness;born.append(witness)
                b['check'](label+'birth has dense prior completed cache',True,dense,t)
                if dense:
                    b['check'](label+'born flags/currentC98',cache[0],props[0],t)
                    b['check'](label+'born permanent color/currentCA0',cache[1],props[1],t)
                    b['check'](label+'born collision damage/currentC88',cache[2],props[2],t)
                    b['check_field'](label+'born base83C/currentC88',lambda:cache[2],lambda:f(a['monstroBaseDamage83c'] if native else a['baseDamage']),t)
                    b['check_field'](label+'born fixed postsetter acceleration',lambda:f(f(0.5)+f(stats['tearFallingAcceleration'])),lambda:f(a['fallingAcceleration7dc'] if native else a['motion']['fallingAcceleration']),t)
        if ready:
            b['check'](label+'actual positive C78',True,maximum>0,t)
            b['check'](label+'actual neutral C18 NULL',True,(weapon['actorSlots']['c18']is None)if native else weapon['persistent']is None,t)
            a=self.action(gameplay_presentation)
            if a is not None and 'diagonal' in a['label'] and held:
                component=f(f(1.0)/f(math.sqrt(f(2.0))))
                expected=[component,f(-component)]
                b['check'](label+'actual diagonal normalized DB4',expected,record['db4'],t)
                if b['equal'](expected,record['db4']):
                    h['diagonal'][counter%2].append(dict(presentation=t,counter=counter,db4=record['db4'],callbackFast=callback_fast,callbackPhaseOwner=record['callbackPhaseOwner'],scope='Own exact DB4 state at source Player319 or qualified port callbackPhase.Clock; not an atomic callback/raw Tear322 witness'))
            dense = old is not None and old['ready'] and old['presentation']+1==t and old['counter']+1==counter and old['counts']==counts and old['maximum']==maximum and old['cache']==cache
            if dense:
                delta=(shots-old['shots'])&0xffffffff
                if born:
                    # Actual42C/C84/cache states define a birth cohort without
                    # inferring held/released input or a callback phase/event.
                    expected_ordinals=[(old['shots']+i)&0xffffffff for i in range(13)]
                    ordered=sorted(born,key=lambda x:(x['ordinal']-old['shots'])&0xffffffff)
                    b['check'](label+'actual13 birth cohort C84 advance',13,delta,t)
                    b['check'](label+'actual13 birth cohort distinct identities',13,len(born),t)
                    b['check'](label+'actual13 birth cohort own ordinal range',expected_ordinals,[x['ordinal']for x in ordered],t)
                    b['check'](label+'actual13 birth cohort C0C reset state',0,charge,t)
                    b['check'](label+'actual13 birth cohort prior charge at least C78',True,old['charge']>=maximum,t)
                    complete=(delta==13 and len(born)==13 and len({x['identity']for x in born})==13
                        and [x['ordinal']for x in ordered]==expected_ordinals and charge==0 and old['charge']>=maximum
                        and all(x['densePriorCache']and x['properties']==x['cache']for x in born))
                    h['birthCohorts'].append(dict(presentation=t,counts=list(counts),priorCharge=old['charge'],maximum=maximum,
                        births=[x['identity']for x in ordered],shotsDelta=delta,complete=complete,
                        scope='Own actual13 born-age identities/C84/cache/C0C state cohort; held/phase/factory/callback/RNG events are not inferred'))
            if dense and held is not None and callback_fast is not None:
                delta=(shots-old['shots'])&0xffffffff
                if old['held'] and held:
                    expected_charge = old['charge'] + int(not callback_fast and old['charge']<maximum)
                    b['check'](label+'held C0C observed gate-clock law',expected_charge,charge,t)
                    b['check'](label+'held no automatic firing ordinal advance',0,delta,t)
                    b['check'](label+'held no actual new Tear births',0,len(born),t)
                    h['chargeRows'].append(dict(presentation=t,counter=counter,charge=charge))
                    if old['charge']>=maximum:
                        h['saturatedHeld'].append(dict(presentation=t,charge=charge,shotsDelta=delta,births=len(born),qualified=delta==0 and len(born)==0 and charge==old['charge']))
                if old['held'] and not held:
                    b['check'](label+'released C0C clears',0,charge,t)
                    if old['charge']<maximum:
                        b['check'](label+'undercharged release no firing advance',0,delta,t)
                        b['check'](label+'undercharged release no actual births',0,len(born),t)
                        h['undercharged'].append(dict(presentation=t,priorCharge=old['charge'],maximum=maximum,shotsDelta=delta,births=len(born),qualified=delta==0 and len(born)==0 and charge==0))
                    else:
                        b['check'](label+'charged release actual13 firing advances',13,delta,t)
                        b['check'](label+'charged release actual13 distinct births',13,len(born),t)
                        expected_ordinals=[(old['shots']+i)&0xffffffff for i in range(13)]
                        b['check'](label+'own actual copied C84 ordinal range',expected_ordinals,sorted(x['ordinal']for x in born),t)
                        complete=delta==13 and len(born)==13 and len({x['identity']for x in born})==13 and all(x['densePriorCache']and x['properties']==x['cache']for x in born) and charge==0 and sorted(x['ordinal']for x in born)==expected_ordinals
                        h['bursts'].append(dict(presentation=t,counts=list(counts),priorCharge=old['charge'],maximum=maximum,
                            births=[x['identity']for x in born],shotsDelta=delta,complete=complete,
                            scope='Dense own charged-release state chain only; no atomic FireTear/SharedRaw/audio/callback credit'))
        h['last']=record
        return current_actors

    def body(self,native,actual,state,t):
        b,f,v,cf=self.b,self.b['f32'],self.b['vector'],self.b['check_field']
        for name,left,right in (
                ('rawFast322',lambda:boolean(native['fast322']),lambda:boolean(actual['fast'])),
                ('physical448',lambda:v(native['position']),lambda:v(actual['motion']['presentedPosition'])),
                ('velocity464',lambda:v(native['velocity']),lambda:v(actual['motion']['velocity'])),
                ('height7D4',lambda:f(native['height7d4']),lambda:f(actual['motion']['height'])),
                ('falling7D8',lambda:f(native['fallingSpeed7d8']),lambda:f(actual['motion']['fallingSpeed'])),
                ('acceleration7DC',lambda:f(native['fallingAcceleration7dc']),lambda:f(actual['motion']['fallingAcceleration'])),
                ('damage48C',lambda:f(native['collisionDamage']),lambda:f(actual['collisionDamage'])),
                ('base83C',lambda:f(native['monstroBaseDamage83c']),lambda:f(actual['baseDamage'])),
                ('ordinal838',lambda:integer(native['monstroShotOrdinal838']),lambda:integer(actual['shotOrdinal'])),
                ('born42C',lambda:integer(native['bornGameAge']),lambda:integer(actual['birthGameUpdate'])),
                ('flags7F0',lambda:native['attackFlags7f0'],lambda:b['flags'](actual['attackFlags'])),
                ('permanentColor3A0',lambda:[f(x)for x in native['permanentColor3a0']],lambda:b['color'](actual['sourceColor'])),
                ('radius474',lambda:f(native['collisionRadius']),lambda:f(actual['collisionRadius'])),
                ('actorRNG7C0',lambda:native['actorRngWords7c0'],lambda:[integer(actual['actorRng'][k])for k in ('state','rightA','left','rightB')])):
            cf('Monstro229 strict exact-identity body '+name,left,right,t)
        if boolean(native['fast322']):
            cf('Monstro229 strict actualFast saved450',lambda:v(native['savedPosition450']),lambda:v(actual['savedPosition450']),t)

    def snapshot(self,source,state,t,native_counter,player,*,native_gameplay_presentation,port_gameplay_presentation):
        sides={}
        for side,gp in (('native',native_gameplay_presentation),('port',port_gameplay_presentation)):
            try:sides[side]=self.own_snapshot(side,source,state,player,t,native_counter,gp)
            except Exception as error:self.b['missing']('Monstro229 '+side+' own actual execution',t,error)
        if len(sides)==2:
            self.b['check']('Monstro229 strict independent complete identities/seeds',sorted(sides['native']),sorted(sides['port']),t)
            for identity in sides['native'].keys()&sides['port'].keys():
                self.body(sides['native'][identity],sides['port'][identity],state,t)
                self.own['native']['bodyJoins']+=1;self.own['port']['bodyJoins']+=1

    def finish(self):
        result=dict(schema=SCHEMA,sides={},scope='Own dense charged-release and born-cache state chains. All independent crossIDs/seeds remain strict; actual copied ordinals only checked within own birth cohort, never used as correspondence. Missing callbacks/RNG/rawbody phase stayFAIL.')
        for side,h in self.own.items():
            label='Monstro229 '+side+' '
            metrics=dict(grants=len(h['grants']),completeMixedBursts=sum(x['complete']and x['counts'][4]>0 for x in h['bursts'])if side=='port'else None,
                actualComplete13BirthCohorts=sum(x['complete']for x in h['birthCohorts']),
                actualCompleteMixed13BirthCohorts=sum(x['complete']and x['counts'][4]>0 for x in h['birthCohorts']),
                actualInputOwnerAvailable=side=='port',actualCallbackPhaseOwnerAvailable=side=='port',
                actualMixedFullBodyRows=sum(x['actualFast322']is False for x in h['bodies']),actualMixedFastBodyRows=sum(x['actualFast322']is True for x in h['bodies']),
                unknownRawBodyPhaseRows=sum(x['actualFast322']is None for x in h['bodies']),underchargedReleases=sum(x['qualified']for x in h['undercharged'])if side=='port'else None,saturatedHeldRows=sum(x['qualified']for x in h['saturatedHeld'])if side=='port'else None,exactIdentityBodyJoins=h['bodyJoins'])
            for name,key in [('minimumActualGrantsPerSide','grants'),('minimumCompleteMixedBurstsPerSide','completeMixedBursts'),('minimumActualMixedFullRowsPerSide','actualMixedFullBodyRows'),('minimumActualMixedFastRowsPerSide','actualMixedFastBodyRows')]:
                if metrics[key] is None:
                    self.b['missing'](label+'required '+key,None,'Actual Native HasShoot/input and callback phase owners are unobserved; DB0/opposed pair is not held input')
                else:
                    self.b['check'](label+'required '+key,True,metrics[key]>=integer(self.declaration[name])>0,None)
            self.b['check'](label+'required actual mixed13 birth/C84/cache cohorts',True,
                metrics['actualCompleteMixed13BirthCohorts']>=integer(self.declaration['minimumCompleteMixedBurstsPerSide'])>0,None)
            for key in ('underchargedReleases','saturatedHeldRows','exactIdentityBodyJoins'):
                if metrics[key] is None:
                    self.b['missing'](label+'required '+key,None,'Actual Native held/released owner is not captured; no zero-observation claim')
                else:
                    self.b['check'](label+'required '+key,True,metrics[key]>0,None)
            self.b['check'](label+'actual ordered queued grants',self.case['requires']['orderedRealPedestalGrants'],[x['itemId']for x in h['grants']],None)
            self.b['check'](label+'actual all inventory stages',list(range(len(self.case['requires']['actorStages']))),sorted(h['stages']),None)
            for parity in (0,1):
                if side=='native':
                    self.b['missing'](label+'required normalized diagonal DB4 gate parity'+str(parity),None,
                        'Actual Native input/callback phase unknown; postgate319/parity/DB4 cannot substitute')
                else:
                    self.b['check'](label+'required normalized diagonal DB4 gate parity'+str(parity),True,len(h['diagonal'][parity])>0,None)
            for field in ('actual callback held/release/C0C instruction order','actual perTear SharedRaw/RNG demand and audio sequence',
                    'actual kind1 factory/immediateFull/subtract/postsetter order','actual incomingDD4/currentP+DC4 factory arguments',
                    'actual wardrobe registration/pose callback and renderer','actual contact/Remove/Uninit lifecycle'):
                self.b['missing'](label+'required '+field,None,'Public state endpoints do not expose this exact event/owner; no inference from scalarcache or ordinal')
            result['sides'][side]=dict(metrics=metrics,grants=h['grants'],inventoryStages=sorted(h['stages']),births=list(h['births'].values()),
                bursts=h['bursts']if side=='port'else None,birthCohorts=h['birthCohorts'],undercharged=h['undercharged']if side=='port'else None,saturatedHeld=h['saturatedHeld']if side=='port'else None,chargeClockRows=h['chargeRows'],diagonalGateStates=h['diagonal']if side=='port'else None,bodyHistory=h['bodies'])
        return result
