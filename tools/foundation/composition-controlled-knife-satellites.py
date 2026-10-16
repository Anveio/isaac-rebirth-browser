"""Opt-in actual Knife4A0 chain observations; callbacks remain separate owners.

Own linked order is source structure, never a cross-game correspondence rule.
Only complete actual identity+seed intersections can compare body fields.
"""
SCHEMA = 'controlled-knife-linked4a0-satellites-v1'
CONTROLLED = 1 << 33
RAW_FLOATS = ('7e0', '7e4', '7e8', '7ec', '7f0', '7f4', '7f8', '80c', '810', '8a4')
RAW_VECTORS = ('7fc', '804')
BODY_FIELDS = ('active', 'dead', 'visible', 'manager', 'damage', 'radius', 'mass', 'hp', 'maxHp',
    'gridClass', 'entityClass', 'raw494', 'entityFlags', 'spawner', 'parent', 'child4a0',
    'rng', 'friction', 'configuredFriction', 'bornAge', 'position', 'savedPosition', 'velocity',
    'renderOffset', 'fast322', 'flags', 'sourceColor', 'currentColor', 'initUniform', 'raw814',
    'raw8a0', 'spriteAnimation', 'spriteFrame', 'spritePlaying', 'spriteRotation', 'spriteScale',
    'spriteOffset') + tuple('raw' + k for k in RAW_FLOATS + RAW_VECTORS)
CALLBACK_OWNERS = ('Get4 current callback input', 'satellite factory copied inputs and RNG draw completion',
    'age0 repeated Full dispatch', 'Full/Fast callback entry', 'formation entry source inputs',
    'Remove and matching registered Uninit')


def integer(value):
    if type(value) is not int:
        raise ValueError('Actual integer owner required')
    return value


def unsigned(value):
    value = integer(value)
    if not 0 <= value <= 0xffffffff:
        raise ValueError('Actual uint32 owner required')
    return value


def boolean(value):
    if type(value) is not bool:
        raise ValueError('Actual boolean owner required')
    return value


class KnifeSatelliteComparer:
    def __init__(self, common, case):
        self.b, self.case = common, case
        self.declaration = case['requires']['controlledKnifeSatellites']
        if (self.declaration['schema'] != SCHEMA or case['fixture']['startingItems'] != [114]
                or case['requires']['orderedRealPedestalGrants'] != [329, 2]
                or 'controlledKnifeSatelliteChain' not in case['requires']['compositionSourceFields']
                or self.declaration['requiredAdditionalNodesPerGame'] != 2):
            raise ValueError('Actual114 ChallengeAdd then329/2 pedestal chain declaration required')
        self.own = {side: dict(last=None, queued=set(), grants=[], stages=set(), currentMap=None,
            currentPresentation=None, linkedRows=[], births={}, bodyRows=0, sourceClockRows=0,
            gameAgeRows=0, fullEndpointRows=0, fastEndpointRows=0)
            for side in ('native', 'port')}
        self.cross = {field: dict(expectedPairedRows=0, attemptedPairedRows=0,
            unmatchedNativeActorRows=0, unmatchedPortActorRows=0,
            nativeUnavailableGates=0, portUnavailableGates=0) for field in BODY_FIELDS}
        self.cross_gates = []

    def identity_key(self, identity):
        #Common identity_tuple is deliberately generic. This new strict join
        #requires exact source32-bit words before hashing/equality; bool and
        #float values must never alias real integer identity/seed words.
        return tuple(unsigned(identity[key]) for key in
            ('kind','variant','subtype','sequenceId','spawnSeed'))

    def validate_handle(self, handle):
        if (not isinstance(handle, dict) or set(handle) != {'kind','id'}
                or type(handle['kind']) is not str):
            raise ValueError('Actual typed string-kind actor handle required')
        unsigned(handle['id'])

    def validate_scheduler_handles(self, state):
        #Validate owners before Python dictionary equality can associate an
        #integer request with a bool/float scheduler id. This is separate from
        #validating the resulting source identity/seed words.
        for entry in state['actorScheduler']['identities']:
            self.validate_handle(entry['actor'])
        for handle in state['actorScheduler']['actors']:
            self.validate_handle(handle)

    def identity(self, state, handle):
        if handle is None:
            return None
        self.validate_handle(handle)
        self.validate_scheduler_handles(state)
        return self.identity_key(self.b['reference'](state, handle))

    def body(self, side, source, state, actor):
        native, b = side == 'native', self.b
        f, v = b['f32'], b['vector']
        if not native and integer(actor['entityType']) != 8:
            raise ValueError('Actual Knife entityType8 owner required')
        ref = None if native else dict(kind='knife', id=unsigned(actor['id']))
        identity = self.identity_key(actor['identity']) if native else self.identity(state, ref)
        if identity[:3] != (8, 0, 0):
            raise ValueError('Actual canonical8/0/0 Knife required')
        if native:
            parent = actor['linkedParent']
            child = actor['knifeSatelliteChild4a0']
            parent = None if parent is None else self.identity_key(parent)
            child = None if child is None else self.identity_key(child)
        else:
            parent_ref = actor['parent']
            if parent_ref is None and actor.get('parentNullKnown') is not True:
                raise ValueError('Missing Parent498 owner cannot become knownNULL')
            parent = self.identity(state, parent_ref)
            edge = actor['childChain4a0']
            if edge == 'empty':
                child = None
            elif isinstance(edge, dict) and set(edge) == {'linked'}:
                child = self.identity(state, edge['linked'])
                if child is None:
                    raise ValueError('Linked4A0 requires its actual nonNULL actor')
            else:
                raise ValueError('Unknown actual Knife4A0 relation')
        fields = dict(parent=parent, child4a0=child)
        for key in ('active', 'dead', 'visible'):
            fields[key] = boolean(actor[key])
        fields['manager'] = boolean(actor['inActiveManager']) if native else ref in state['actorScheduler']['actors']
        for key, n, p in [('damage','collisionDamage','collisionDamage'), ('radius','collisionRadius','collisionRadius'),
                ('mass','collisionMass','collisionMass'), ('hp','hp','hp'), ('maxHp','maxHp','maxHp'),
                ('friction','currentFriction470','currentFriction'), ('configuredFriction','configuredFriction46c','configuredFriction'),
                ('initUniform','initUniform8a8','raw8a8')]:
            fields[key] = f(actor[n if native else p])
        for key, n, p in [('position','position','position'), ('savedPosition','savedPosition450','savedPosition'),
                ('velocity','velocity','velocity'), ('renderOffset','renderOffset','renderOffset')]:
            fields[key] = v(actor[n if native else p])
        fields.update(gridClass=integer(actor['gridCollisionClass']), entityClass=integer(actor['entityCollisionClass']),
            raw494=boolean(actor['raw494']), entityFlags=integer(actor['genericFlags318'] if native else actor['entityFlags']),
            spawner=(integer(actor['cachedSpawner']['kind']), integer(actor['cachedSpawner']['variant'])) if native else
                (integer(actor['spawnerType']), integer(actor['spawnerVariant'])),
            bornAge=integer(actor['bornGameAge'] if native else actor['birthGameUpdate']),
            fast322=boolean(actor['fast322'] if native else actor['fast']),
            flags=int(actor['attackFlags7d8'] if native else b['flags'](actor['attackFlags'])),
            sourceColor=[f(x) for x in actor['permanentColor3a0']] if native else b['color'](actor['sourceColor']),
            currentColor=[f(x) for x in actor['currentColor2d0']] if native else b['color'](actor['currentColor']),
            raw814=boolean(actor['knifeRaw814'] if native else actor['raw814']),
            raw8a0=integer(actor['knifeRaw8a0'] if native else actor['raw8a0']))
        rng = actor['actorRngWords7c0'] if native else [actor['actorRng'][key] for key in ('state','rightA','left','rightB')]
        if len(rng) != 4 or any(type(x) is not int or not 0 <= x <= 0xffffffff for x in rng):
            raise ValueError('Actual four actor RNG words required')
        fields['rng'] = list(rng)
        for key in RAW_FLOATS:
            fields['raw'+key] = f(actor['knifeRawFloats'][key] if native else actor['raw'+key])
        for key in RAW_VECTORS:
            fields['raw'+key] = v(actor['knifeRawVectors'][key] if native else actor['raw'+key])
        sprite = actor['sprite']
        for key in ('animation', 'frame', 'playing', 'rotation', 'scale', 'offset'):
            value = sprite[key]
            fields['sprite'+key[0].upper()+key[1:]] = (v(value) if key in ('scale','offset') else
                f(value) if key in ('frame','rotation') else boolean(value) if key == 'playing' else value)
        if set(fields) != set(BODY_FIELDS):
            raise ValueError('Incomplete canonical Knife body roster')
        return identity, fields

    def actor_map(self, side, source, state):
        if side != 'native':
            self.validate_scheduler_handles(state)
        out = {}
        for actor in source['attacks'] if side == 'native' else state['knives']:
            if side == 'native':
                if self.identity_key(actor['identity'])[:3] != (8,0,0):
                    continue
            else:
                ref = dict(kind='knife', id=unsigned(actor['id']))
                identities = [x for x in state['actorScheduler']['identities'] if x['actor'] == ref]
                #Stored uninitialized DTOs are not current scheduler identities.
                #An actual linked reference to one still FAILs in body(), above.
                if not identities:
                    continue
                if len(identities) != 1:
                    raise ValueError('Actual current Knife identity is ambiguous')
            identity, fields = self.body(side,source,state,actor)
            if identity in out:
                raise ValueError('Duplicate actual Knife identity')
            out[identity] = fields
        return out

    def metadata(self, side, source, state, player, t, native_counter):
        b, h, native = self.b, self.own[side], side == 'native'
        owner = player if native else state['playerItems']
        counts = owner['ownedCounts']
        if len(counts) != 347 or any(type(x) is not int for x in counts):
            raise ValueError('Actual complete signed counts required')
        pending = integer(owner['cacheFlags'] if native else owner['unevaluatedCacheFlags'])
        queued = owner['queuedId']
        if queued is not None:
            h['queued'].add(integer(queued))
        old = h['last']
        if old is not None:
            for item in (329,2):
                if old['counts'][item] == 0 and counts[item] == 1 and item in h['queued'] and queued is None and pending == 0:
                    h['grants'].append(dict(itemId=item,presentation=t,scope='Own actual queue/count/cache endpoints, not atomic Add callback credit'))
                    h['queued'].discard(item)
        stage = None
        for i,d in enumerate(self.case['requires']['actorStages']):
            wanted = [0]*347
            for entry in d['ownedCounts']:
                wanted[integer(entry['itemId'])] = integer(entry['count'])
            if counts == wanted:
                stage = i
                h['stages'].add(i)
        age = integer(source['gameAge'] if native else state['gameUpdateCount'])
        clock = integer(native_counter if native else state['systemClock']['systemCounter'])
        h['sourceClockRows'] += 1
        h['gameAgeRows'] += 1
        slot = source['players'][0]['actorSlots']['c18'] if native else state['attackRuntime']['persistent']
        anchor = (None if slot is None else self.identity_key(slot['identity'])) if native else self.identity(state,slot)
        caller = source['players'][0]['knifeSatelliteCaller'] if native else None
        source_type = integer(caller['playerTypeBfc'] if native else state['weaponAdmission']['sourcePlayerType'])
        if source_type is None:
            raise ValueError('Actual sourcePlayer type owner is unknown')
        d80 = integer(caller['activeStateD80']) if native else integer(owner['activeStateD80'])
        h['last'] = dict(presentation=t,counts=list(counts),age=age,counter=clock,stage=stage,
            pending=pending,queued=queued,anchor=anchor,ready=stage==2 and pending==0 and queued is None
                and source_type==0 and d80==0)
        #Actual raw counters are evidence; endpoint fast322 or modulo alone
        #does not establish a source callback entry or dispatched body.
        return h['last']

    def linked(self, side, record, t):
        b,h = self.b,self.own[side]
        if not record['ready']:
            return
        actors,anchor = h['currentMap'],record['anchor']
        if actors is None or anchor not in actors:
            raise ValueError('Actual current registered anchor and typed map required')
        b['check']('KnifeSatellites '+side+' anchor actualcontrolled bit33',CONTROLLED,actors[anchor]['flags'],t)
        b['check']('KnifeSatellites '+side+' anchor actualParentPlayer',True,
            actors[anchor]['parent'] is not None and actors[anchor]['parent'][:3]==(1,0,0),t)
        ids,seen,current = [],{anchor},actors[anchor]['child4a0']
        while current is not None:
            if current in seen or len(ids) >= 512 or current not in actors:
                raise ValueError('Actual4A0 chain cycle, bound or unresolved target')
            seen.add(current)
            ids.append(current)
            child = actors[current]
            b['check']('KnifeSatellites '+side+' actual chain node family',(8,0,0),current[:3],t)
            b['check']('KnifeSatellites '+side+' Parent498 independently points toanchor',anchor,child['parent'],t)
            b['check']('KnifeSatellites '+side+' satellite actual bit33 only',CONTROLLED,child['flags'],t)
            b['check']('KnifeSatellites '+side+' actual factory cachedSpawner0',(0,0),child['spawner'],t)
            current = child['child4a0']
        b['check']('KnifeSatellites '+side+' actual2 additional nodes',2,len(ids),t)
        if len(ids) == 2:
            h['linkedRows'].append(dict(presentation=t,anchor=anchor,children=ids,sourceCounter=record['counter'],gameAge=record['age']))
            h['bodyRows'] += 3
        for identity in [anchor,*ids]:
            actor = actors[identity]
            if actor['bornAge'] == record['age'] and identity not in h['births']:
                h['births'][identity] = dict(presentation=t,age=record['age'],damage=actor['damage'],
                    flags=actor['flags'],color=actor['sourceColor'],scope='Actual own42C endpoint only; no birthcallback/RNG completion credit')
            if actor['fast322']:
                h['fastEndpointRows'] += 1
            else:
                h['fullEndpointRows'] += 1

    def own_snapshot(self, side, source, state, player, t, native_counter):
        b,h = self.b,self.own[side]
        h['currentMap'],h['currentPresentation'] = None,t
        try:
            h['currentMap'] = self.actor_map(side,source,state)
        except Exception as error:
            b['missing']('KnifeSatellites '+side+' independent current body map',t,error)
        record = None
        try:
            record = self.metadata(side,source,state,player,t,native_counter)
        except Exception as error:
            b['missing']('KnifeSatellites '+side+' independent current metadata',t,error)
        if record is not None:
            try:
                self.linked(side,record,t)
            except Exception as error:
                b['missing']('KnifeSatellites '+side+' actual linked chain',t,error)
        return h['currentMap']

    def snapshot(self, source, state, t, native_counter, player):
        maps = {}
        for side in ('native','port'):
            try:
                maps[side] = self.own_snapshot(side,source,state,player,t,native_counter)
            except Exception as error:
                maps[side] = None
                self.b['missing']('KnifeSatellites '+side+' independent snapshot',t,error)
        left,right = maps['native'],maps['port']
        li,ri = set(left or {}),set(right or {})
        self.b['check']('KnifeSatellites strict complete actual identity/seed sets',sorted(li),sorted(ri),t)
        unmatched_left,unmatched_right = sorted(li-ri),sorted(ri-li)
        for field in BODY_FIELDS:
            row = self.cross[field]
            row['expectedPairedRows'] += len(li|ri)
            row['unmatchedNativeActorRows'] += len(unmatched_left)
            row['unmatchedPortActorRows'] += len(unmatched_right)
            row['nativeUnavailableGates'] += left is None
            row['portUnavailableGates'] += right is None
            for identity in sorted(li&ri):
                row['attemptedPairedRows'] += 1
                self.b['check']('KnifeSatellites strict body '+field,left[identity][field],right[identity][field],t)
        self.cross_gates.append(dict(presentation=t,nativeCount=None if left is None else len(left),
            portCount=None if right is None else len(right),paired=len(li&ri),unmatchedNative=unmatched_left,
            unmatchedPort=unmatched_right,ungradedFields=list(BODY_FIELDS) if unmatched_left or unmatched_right or left is None or right is None else []))

    def finish_own(self, side):
        h,b = self.own[side],self.b
        b['check']('KnifeSatellites '+side+' all actual inventory stages',[0,1,2],sorted(h['stages']),None)
        b['check']('KnifeSatellites '+side+' real queued grant order',[329,2],[g['itemId'] for g in h['grants']],None)
        b['check']('KnifeSatellites '+side+' positive linked rows',True,len(h['linkedRows'])>=1,None)
        b['check']('KnifeSatellites '+side+' positive actual three body rows',True,h['bodyRows']>=3,None)
        b['check']('KnifeSatellites '+side+' positive actual satellite birth endpoints',True,len([i for i in h['births'] if any(i in row['children'] for row in h['linkedRows'])])>=2,None)
        b['check']('KnifeSatellites '+side+' actual System counter observations',True,h['sourceClockRows']>=1,None)
        b['check']('KnifeSatellites '+side+' actual Game age observations',True,h['gameAgeRows']>=1,None)
        #The declared callback/copy minima remain explicitly unmet; reading
        #current cached damage or equal born endpoints cannot satisfy them.
        minimums = {'minimumCurrentGet4ChecksPerGame':0,'minimumCopiedBornDamageChecksPerGame':0}
        for name, actual in minimums.items():
            required = integer(self.declaration[name])
            b['check']('KnifeSatellites '+side+' required '+name,True,actual>=required,None)
        for owner in CALLBACK_OWNERS:
            b['missing']('KnifeSatellites '+side+' required actual '+owner,None,'UNOBSERVED: endpoint facts are not callback/copy/RNG/lifecycle credit')
        return dict(grants=h['grants'],stages=sorted(h['stages']),linkedRows=h['linkedRows'],births=list(h['births'].items()),
            bodyRows=h['bodyRows'],rawSystemRows=h['sourceClockRows'],rawGameAgeRows=h['gameAgeRows'],
            actualFast322EndpointRows=h['fastEndpointRows'],actualNonFast322EndpointRows=h['fullEndpointRows'],
            callbackOwners={owner:'UNKNOWN_REQUIRED_FAIL' for owner in CALLBACK_OWNERS},
            declaredCallbackMinimums={name:dict(required=self.declaration[name],actual=actual,status='UNKNOWN_REQUIRED_FAIL') for name,actual in minimums.items()},
            scope='Own actual typed nodes/order/body endpoints only; no crossgame equality or callback completion credit')

    def finish(self):
        own = {side:self.finish_own(side) for side in ('native','port')}
        self.b['check']('KnifeSatellites positive strict body identity joins',True,
            any(r['attemptedPairedRows']>0 for r in self.cross.values()),None)
        for field,row in self.cross.items():
            if row['unmatchedNativeActorRows'] or row['unmatchedPortActorRows'] or row['nativeUnavailableGates'] or row['portUnavailableGates']:
                self.b['missing']('KnifeSatellites UNGRADED strict body '+field,None,dict(row))
        return dict(schema=SCHEMA,own=own,strictBodyRoster=BODY_FIELDS,strictFieldObligations=self.cross,
            crossGates=self.cross_gates,scope='No global item capability union; source4A0 order never supplies cross correspondence. All unknowns, unmatched actor fields and failed minima retained.')
