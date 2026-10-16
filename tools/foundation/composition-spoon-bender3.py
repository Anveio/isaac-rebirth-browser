"""Current canonical flag4 target/body states; callback and RNG events stay unknown."""
SCHEMA='ordinary-spoon-bender3-registered-homing-v1'

def integer(x):
    if type(x)is not int:raise ValueError('Actual integer owner required')
    return x

def boolean(x):
    if type(x)is not bool:raise ValueError('Actual boolean owner required')
    return x

class SpoonBenderComparer:
    def __init__(self,shared,case):
        self.b,self.case=shared,case;self.d=case['requires']['spoonBender3Functional'];c=case['requires']['sourceExpectedActorContract']
        if(self.d['schema']!=SCHEMA or self.d['marker']!='spoonBender3Tears' or case['fixture']['startingItems']!=[] or case['requires']['orderedRealPedestalGrants']!=[3]
                or tuple(c[k]for k in ('primaryC08','family','variant','subtype','bornAttackFlags','controlledBy329','registeredC18DuringActualLifetime'))!=(1,2,0,0,'4',False,False)
                or 'spoonBender3Tears'not in case['requires']['compositionSourceFields'] or self.d['missingToolingMustFail']is not True):raise ValueError('Exact public real3/bornflag4 ordinary Tear contract required')
        self.minimum={'actualGrants':'minimumActualGrantsPerGame','bornHomingTears':'minimumBornHomingTearsPerGame','fullBodyRows':'minimumFullBodyRowsPerGame','fastBodyRows':'minimumFastBodyRowsPerGame','nonnullTargetStates':'minimumActualNonnullTargetStatesPerGame','acquiredTargetTransitions':'minimumAcquiredTargetTransitionsPerGame','laterFullSteeringDiagnostics':'minimumLaterFullSteeringDiagnosticsPerGame'}
        for key in self.minimum.values():
            if integer(self.d[key])<=0:raise ValueError('Positive real coverage minima required')
        self.own={side:dict(last=None,queued=False,grants=[],chains={},current=None,metrics={k:0 for k in self.minimum})for side in ('native','port')}
        self.correspondence=[]

    def body(self,side,state,actor):
        b,n=self.b,side=='native';f=b['f32'];ref=None if n else dict(kind='tear',id=integer(actor['id']))
        identity=b['identity_tuple'](actor['identity']if n else b['reference'](state,ref))
        if identity[:3]!=(2,0,0):raise ValueError('Actual typed2/0/0 required')
        if n:
            if actor['spoonWriterScope']is not True or str(int(actor['attackFlags7f0']))!='4':raise ValueError('Actual normal Player bornflag4 writer required')
            owner=actor['spoonHoming'];fast=boolean(actor['fast322']);position=b['vector'](actor['savedPosition450']if fast else actor['position']);velocity=b['vector'](actor['velocity']);target=owner['target4a8']
            target=None if target is None else b['identity_tuple'](target)
            return dict(identity=identity,ordinal=integer(owner['shotOrdinal838']),born=integer(actor['bornGameAge']),target=target,flags='4',entityFlags=integer(actor['genericFlags318']),multiplier=f(owner['velocityMultiplier7e0']),registry=tuple(integer(i)for i in owner['hitSequences808']),position=position,presented=b['vector'](actor['position']),velocity=velocity,height=f(actor['height7d4']),falling=f(actor['fallingSpeed7d8']),acceleration=f(actor['fallingAcceleration7dc']),damage=f(actor['collisionDamage']),baseDamage=f(owner['baseDamage83c']),color=tuple(f(x)for x in actor['permanentColor3a0']),fast=fast)
        if b['flags'](actor['attackFlags'])!='4' or actor.get('controlled')is not None or ref not in state['actorScheduler']['actors']:raise ValueError('Actual current ordinary bornflag4 manager actor required')
        owner=actor['homing'];target=owner['target'];target=None if target is None else b['identity_tuple'](b['reference'](state,target));clock=state['systemClock'];counter=integer(clock['systemCounter']);fast=(counter-1)%2==1
        if fast and boolean(clock['interpolationEnabled'])is not True:raise ValueError('Actual Fast phase owner unavailable')
        motion=actor['motion']
        return dict(identity=identity,ordinal=integer(actor['shotOrdinal']),born=integer(actor['birthGameUpdate']),target=target,flags='4',entityFlags=integer(owner['entityFlags']),multiplier=f(owner['velocityMultiplier']),registry=tuple(integer(i)for i in actor['persistentContacts']['hitSequences']),position=b['vector'](motion['position']),presented=b['vector'](motion['presentedPosition']),velocity=b['vector'](motion['velocity']),height=f(motion['height']),falling=f(motion['fallingSpeed']),acceleration=f(motion['fallingAcceleration']),damage=f(actor['collisionDamage']),baseDamage=f(actor['baseDamage']),color=tuple(b['color'](actor['sourceColor'])),fast=fast)

    def own_snapshot(self,side,source,state,player,t,native_counter,gameplay_presentation):
        b,n,h=self.b,side=='native',self.own[side];current={};known=True;counter=integer(native_counter if n else state['systemClock']['systemCounter']);age=integer(source['gameAge']if n else state['gameUpdateCount']);last=h['last'];counts=None;label='Spoon3 '+side+' '
        dense=last is not None and integer(gameplay_presentation)==last['gameplayPresentation']+1 and counter==last['counter']+1
        try:
            owner=player if n else state['playerItems'];counts=owner['ownedCounts'];queued=owner['queuedId'];pending=integer(owner['cacheFlags']if n else owner['unevaluatedCacheFlags'])
            if len(counts)!=347 or any(type(x)is not int for x in counts):raise ValueError('Actual complete signed counts required')
            if queued==3:h['queued']=True
            if dense and last['counts']is not None and last['counts'][3]==0 and counts[3]==1 and queued is None and pending==0 and h['queued']:
                h['grants'].append(dict(presentation=t,itemId=3,scope='Own adjacent real queued/count/cache112 endpoint, not Add callback'));h['metrics']['actualGrants']+=1;h['queued']=False
        except Exception as e:b['missing'](label+'actual grant/cache112 owner',t,e)
        h['currentMoveSpeedInputs']=None
        if counts is not None and counts[3]>0:
            try:
                speed=source['players'][0]['spoonMoveSpeedInputs']if n else state['playerItems']['moveSpeedInputs']
                actual=tuple(integer(speed[k])for k in ('sourceD28','sourceD44','source13bc','source13c0'))
                b['check'](label+'actual independent neutral MoveSpeed inputs',(0,0,0,0),actual,t)
                h['currentMoveSpeedInputs']=actual
                speed_value=speed['actualMoveSpeedD60']if n else state['playerStats']['moveSpeed']
                b['check'](label+'actual source neutral MoveSpeed D60',b['f32'](1),b['f32'](speed_value),t)
            except Exception as e:b['missing'](label+'actual independent MoveSpeed history',t,e)
        try:
            weapon=source['players'][0]if n else state['attackRuntime']
            if counts is not None and counts[3]>0:
                b['check'](label+'actual current primaryC08',1,integer(weapon['primaryWeaponC08']if n else state['weaponCache']['primary']),t)
                b['check'](label+'actual C18 remains NULL',None,weapon['actorSlots']['c18']if n else weapon['persistent'],t)
                if not n:b['check'](label+'actual C18 NULL history stays known',True,boolean(state['weaponAdmission']['c18IsKnownEmpty']),t)
        except Exception as e:b['missing'](label+'independent primary and C18 owner',t,e)
        #A malformed/retired actor does not suppress independent grant or peers.
        try:
            if n and 'spoonBender3Tears'not in source['requestedSourceFields']:raise ValueError('Actual optional source marker missing')
            actors=[a for a in source['attacks']if a['identity']['kind']==2 and a['identity']['variant']==0 and a['identity']['subtype']==0 and int(a['attackFlags7f0'])==4 and a['inActiveManager']]if n else[a for a in state['tears']if b['flags'](a['attackFlags'])=='4'and a.get('controlled')is None]
        except Exception as e:actors=[];known=False;b['missing'](label+'actual complete homing actor source',t,e)
        for actor in actors:
            try:
                row=self.body(side,state,actor);identity=row['identity'];ordinal=row['ordinal']
                if ordinal in current:raise ValueError('Ambiguous actual Player838 ordinal; no cross-side birth-order matching')
                if any(existing['identity']==identity for existing in current.values()):raise ValueError('Duplicate actual current actor identity; no correspondence credit')
                current[ordinal]=row;chain=h['chains'].setdefault(identity,dict(identity=identity,rows=[]));prior=chain['rows'][-1]if chain['rows']else None
                h['metrics']['bornHomingTears']=len(h['chains']);h['metrics']['fastBodyRows'if row['fast']else'fullBodyRows']+=1
                if row['target']is not None:h['metrics']['nonnullTargetStates']+=1
                if prior and dense and prior['gameplayPresentation']+1==integer(gameplay_presentation):
                    b['check'](label+'same own born fields retained',(prior['body']['born'],prior['body']['damage'],prior['body']['baseDamage'],prior['body']['flags']),(row['born'],row['damage'],row['baseDamage'],row['flags']),t)
                    if prior['body']['target']is None and row['target']is not None:h['metrics']['acquiredTargetTransitions']+=1
                    if row['fast']and age==prior['gameAge']:
                        b['check'](label+'Fast actual physical velocity and target retained',(prior['body']['position'],prior['body']['velocity'],prior['body']['target']),(row['position'],row['velocity'],row['target']),t)
                    if(not row['fast']and age==prior['gameAge']+1 and prior['body']['target']is not None and row['target']==prior['body']['target']and row['velocity']!=prior['body']['velocity'] and (not actor['dead']if n else actor['motion']['lifetime']=='alive')):
                        h['metrics']['laterFullSteeringDiagnostics']+=1
                        #Current endpoints establish a changed velocity with
                        #retained target, not the RNG return or callback math.
                chain['rows'].append(dict(presentation=t,gameplayPresentation=integer(gameplay_presentation),gameAge=age,body=row,scope='Own actual canonical state; no Init/setter/update completion inference'))
            except Exception as e:known=False;b['missing'](label+'actual body/registered-target owner',t,e)
        h['availableCurrent']=current;h['current']=current if known else None;h['last']=dict(gameplayPresentation=integer(gameplay_presentation),counter=counter,counts=counts)
        return h['current']

    def snapshot(self,source,state,t,native_counter,player,*,native_gameplay_presentation,port_gameplay_presentation):
        b=self.b;left=self.own_snapshot('native',source,None,player,t,native_counter,native_gameplay_presentation);right=self.own_snapshot('port',None,state,None,t,None,port_gameplay_presentation)
        material=('born','target','flags','entityFlags','multiplier','registry','position','presented','velocity','height','falling','acceleration','damage','baseDamage','color')
        maps={'native':left,'port':right};rosters={side:self.own[side]['availableCurrent']for side in maps};complete=left is not None and right is not None
        exact={side:{row['identity']:row for row in roster.values()}for side,roster in rosters.items()}
        joins=exact['native'].keys()&exact['port'].keys()if complete else set()
        record=dict(presentation=t,scope='Exact five-field identity joins only; actual838 ordinal associations are source-role diagnostics',sides={})
        if not complete:b['missing']('Spoon3 strict complete current map',t,'Actual map unavailable; partial roster cannot certify correspondence')
        if complete:
            b['check']('Spoon3 source-role diagnostic actual Player838 role set',sorted(left),sorted(right),t)
            for ordinal in sorted(left.keys()&right.keys()):
                b['check']('Spoon3 source-role diagnostic full identity',left[ordinal]['identity'],right[ordinal]['identity'],t)
        for side,roster in exact.items():
            unmatched=roster.keys()-joins
            record['sides'][side]=dict(currentMapKnown=maps[side]is not None,availableCurrentRows=len(roster),exactIdentityJoinRows=len(joins),unmatchedRows=len(unmatched),ungradedMaterialRows=len(unmatched),ungradedMaterialObligations=len(unmatched)*len(material))
            for identity in sorted(unmatched):
                for field in material:
                    b['missing']('Spoon3 '+side+' UNGRADED exact actor '+field,t,dict(identity=identity,ordinal=roster[identity]['ordinal'],reason='No complete exact-identity partner; source-role diagnostics do not establish correspondence'))
        for identity in sorted(joins):
            for key in exact['native'][identity]:
                if key=='fast':continue
                b['check']('Spoon3 strict exact-identity '+key,exact['native'][identity][key],exact['port'][identity][key],t)
        #Native322 is a stored bit. Actual port Clock classifies presentation
        #only; strict raw322 owner remains independently unavailable.
        b['missing']('Spoon3 strict actual port raw322 owner',t,'Port raw322 is unavailable; presentation phase is not this stored field')
        self.correspondence.append(record)

    def finish(self):
        b=self.b;out={}
        for side,h in self.own.items():
            for name,key in self.minimum.items():b['check']('Spoon3 '+side+' required '+name,True,h['metrics'][name]>=self.d[key],None)
            if side=='port':b['missing']('Spoon3 port required raw322 owner',None,'Actual Clock classifies presentation only; no raw322 or callback completion alias')
            for unknown in self.d['requiredActualUnknowns']:b['missing']('Spoon3 '+side+' required '+unknown,None,'Actual callback/history owner unavailable; state and own lastFull diagnostics do not certify it')
            out[side]=dict(metrics=h['metrics'],grants=h['grants'],chains=list(h['chains'].values()),scope='Same-own canonical target/body state endpoints only')
        return dict(schema=SCHEMA,sides=out,correspondence=dict(status='observed'if self.correspondence else'unobserved',rows=self.correspondence,scope='Complete current maps and exact five-field identities only; unmatched actual actors retain per-field UNGRADED obligations'),scope='Real3/completecache112 and strict current2/0/0 flag4 targets/bodies. ChangedV-with-retainedtarget is state compatibility only; exact target-at-callback, Bound2/setter/Init/Uninit and full stream remain unknownFAIL.')
