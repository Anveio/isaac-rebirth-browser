"""Common SKU typed owner laws over complete independent game journals."""
import argparse
import collections
import gzip
import hashlib
import importlib.util
import json
import math
import struct
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
MISSING={'unavailable':'missing or malformed actual owner'}
def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
evidence=load('common_sku_native_evidence',ROOT/'tools/original/native-evidence.py')
capture=load('common_sku_public_grammar',ROOT/'tools/original/common-sku-run.py')
p=argparse.ArgumentParser(allow_abbrev=False)
p.add_argument('--case',type=Path,required=True)
p.add_argument('--directory',type=Path,required=True)
p.add_argument('--original-directory',type=Path)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
case=json.loads(a.case.read_text())
assert case['commonSku']['schema']=='source-public-common-sku-v1'
branch=case['commonSku'].get('branch',case['name'].removeprefix('battery90-'))
assert branch in {'free-no-active','paid-no-active','paid-undercharged','paid100'}
rows=[]
unavailable=[]
diagnostics=[]
def check(label,field,actual,expected=True,scope=None):
    ok=actual!=MISSING and expected!=MISSING and type(actual)is type(expected)and actual==expected
    rows.append(dict(label=label,field=field,actual=actual,expected=expected,status='pass'if ok else'fail',scope=scope))
    return ok
def required(label,field,fn):
    try:return fn()
    except(KeyError,TypeError,ValueError,AssertionError,IndexError,StopIteration)as error:
        check(label,field,{'unavailable':str(error)},'actual typed owner')
        return None
def integer(x):
    assert type(x)is int
    return x
def boolean(x):
    assert type(x)is bool
    return x
def queued_touched(player):
    raw=integer(player['queuedTouchedRaw'])
    assert raw in(0,1)
    value=player['queuedTouched']
    if type(value)is bool:
        assert value==(raw==1)
        return value
    # Historical actual captures keep the validated0/1 reader wire, never rewritten.
    assert type(value)is int and value in(0,1)and value==raw
    return value==1

def check_f32(label,field,actual,expected):
    actual,expected=vec(actual),vec(expected)
    words=lambda values:[struct.pack('<f',value).hex()for value in values]
    aw,ew=words(actual),words(expected)
    check(label,field,actual,expected)
    rows[-1].update(status='pass'if aw==ew else'fail',f32Words={'actual':aw,'expected':ew},
                    representation='Exact binary32 owner words; raw wire values retained')

def finite(x):
    assert type(x)in(int,float)and math.isfinite(x)
    return float(x)
def vec(x):
    if isinstance(x,dict):x=[x['x'],x['y']]
    assert isinstance(x,list)and len(x)==2
    return [finite(v)for v in x]
def one(xs):
    assert isinstance(xs,list)and len(xs)==1
    return xs[0]
def source_clip(s):
    name=s['animation']
    assert name is None or isinstance(name,str)and name
    if name is None:return [None,None,None]
    return [name,finite(s['animationFrame']),boolean(s['animationPlaying'])]
def digest(path):
    h=hashlib.sha256()
    with Path(path).open('rb')as f:
        for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
    return h.hexdigest()

native_dir=a.original_directory or a.directory/'original'
progress=evidence.progress_path(native_dir)
assert progress is not None
metadata=json.loads(Path(str(progress)+'.meta.json').read_text())
evidence.validate_progress_format(progress,metadata)
assert metadata['status']=='closed'and digest(progress)==metadata['archiveSha256']
labels=capture.heavy_labels(case)
native={}
raw_hash=hashlib.sha256()
raw_bytes=records=0
with evidence.open_progress(progress)as stream:
    for raw in stream:
        raw_hash.update(raw);raw_bytes+=len(raw);records+=1
        row=json.loads(raw)
        if row['kind']!='debugObservation':continue
        entry=row['value'];context=entry['context'];o=entry['observation']
        check(context['label'],'Native basic observer passes',o['status'],'pass')
        if context['label']not in labels:continue
        assert context['label']not in native,'Duplicate required Native capture'
        native[context['label']]=dict(context=context,observation=o)
assert(records,raw_bytes,raw_hash.hexdigest())==(metadata['records'],metadata['rawBytes'],metadata['rawSha256'])
browser_path=a.directory/'browser/browser-progress.jsonl.gz'
browser_meta=json.loads(Path(str(browser_path)+'.meta.json').read_text())
assert browser_meta['status']=='closed'and digest(browser_path)==browser_meta['archiveSha256']
browser={}
raw_hash=hashlib.sha256();raw_bytes=records=0
with gzip.open(browser_path,'rb')as stream:
    for raw in stream:
        raw_hash.update(raw);raw_bytes+=len(raw);records+=1
        entry=json.loads(raw)
        if entry['kind']!='capture':continue
        v=entry['value']
        if v['label']in labels:
            assert v['label']not in browser,'Duplicate required Browser capture'
            browser[v['label']]=v
assert(records,raw_bytes,raw_hash.hexdigest())==(browser_meta['records'],browser_meta['rawBytes'],browser_meta['rawSha256'])
for label in labels:
    check(label,'required Native capture',label in native)
    check(label,'required actual Browser capture',label in browser)
aliases={x['publicLabel']:case['actions'][x['actionIndex']]['label']for x in case['commonSku']['publicCheckpointAliases']}
def at(alias):
    label=aliases[alias]
    return label,native[label]['observation'],browser[label]['state']
free=branch=='free-no-active'
collectible=branch=='paid100'
before_alias='battery-before-contact'if free else'paid-before-contact'
outcome_alias='battery-outcome'if free else'paid-outcome'
before_label,before_native,before_state=required('coverage','actual before-contact owners',lambda:at(before_alias))or(None,None,None)
target_seed=None
if before_native is not None:
    offer=required(before_label,'unique actual target',lambda:one([e for e in before_native['shopFinance']['contents']['entities']if e['type']==5 and(e['variant']==90 if not collectible else e['variant']==100 and e['subtype']==63)and(free or e['shopSlotAc4']==(2 if collectible else 3))]))
    if offer is not None:target_seed=integer(offer['spawnSeed'])
def target_source(o):
    return one([e for e in o['shopFinance']['contents']['entities']if e['type']==5 and e['spawnSeed']==target_seed])
def target_actual(s):return one([e for e in s['room']['pickups']if e['spawnSeed']==target_seed])
def sprite_source(o):return one([e for e in o['shopPresentation']['pickups']if e['spawnSeed']==target_seed])
word_fields=['activeId','activeCharge','activeSubordinateCharge','activeStateD80','activeStateD84','reserveCharge']
contact=[]
contact_all=[]
before=None
for label in labels:
    if label not in native or label not in browser:continue
    o=native[label]['observation'];s=browser[label]['state']
    def compare_row():
        items=o['commonSkuItemOwners'];f=o['shopFinance']['contents'];presentation=o['shopPresentation']
        for name,owner in [('items',items),('finance',f),('presentation',presentation),('audio',o['commonSkuAudio'])]:
            check(label,name+' actual status',owner['status'],'pass')
        check(label,'actual seed-effect mode0',integer(items['rawGameA1D8']),0)
        check(label,'Browser actual challenge0',integer(s['challengeId']),0)
        check(label,'same-turn item/physical owner',integer(items['systemCounter']),integer(f['nativePhase']['systemIterations']))
        check(label,'source/Browser phase',integer(s['systemClock']['systemCounter'])%2,integer(items['systemCounter'])%2)
        n=one(items['players']);physical=one(f['players']);own=s['playerItems'];assert isinstance(own,dict)
        check(label,'Native Player identity join',integer(n['entitySpawnSeed']),integer(physical['spawnSeed']))
        nc=[integer(v)for v in n['ownedCounts']];bc=[integer(v)for v in own['ownedCounts']]
        assert len(nc)==len(bc)==347
        check(label,'owned collectible counts',bc,nc)
        check(label,'actual Player spawn seed',integer(own['entitySpawnSeed']),integer(n['entitySpawnSeed']))
        check(label,'actual Player grid class',integer(s['player']['gridCollisionClass']),integer(physical['gridCollisionClass']))
        for field in word_fields:check(label,'player.'+field,integer(own[field]),integer(n[field]))
        check(label,'canonical active word provenance',own['activeChargeHistory']['unowned'],{})
        assert n['queuedId']is None or type(n['queuedId'])is int
        assert own['queuedId']is None or type(own['queuedId'])is int
        check(label,'queue ID',own['queuedId'],n['queuedId'])
        if n['queuedId']is not None:
            check(label,'active queue charge',integer(own['queuedCharge']),integer(n['queuedCharge']))
            check(label,'active queue touched',boolean(own['queuedTouched']),queued_touched(n))
        check(label,'actual inventory counts',s['inventory'],n['inventory'])
        check(label,'health',[integer(s['player'][k])for k in ['maxHp','hp','soulHp','blackHeartMask']],n['health'])
        check(label,'pickup guard',boolean(own['pickupAnimationGuard']),boolean(n['pickupAnimationGuard']))
        check(label,'Player controls',boolean(s['player']['controlsEnabled']),boolean(n['controlsEnabled']))
        check(label,'Player input cooldown',integer(s['player']['inputCooldown']),integer(n['inputCooldown']))
        if n['mainSprite']['animation']=='Pickup':
            check(label,'main Pickup legacy clock',[finite(own['pickupAnimationFrame']),boolean(own['pickupAnimationPlaying']),boolean(own['pickupAnimationGuard'])],
                  [finite(n['mainSprite']['animationFrame']),boolean(n['mainSprite']['animationPlaying']),boolean(n['pickupAnimationGuard'])])
            check(label,'actual explicit main presentation',s['playerAnimation']['current'],None)
        ns=[e for e in f['entities']if e['type']==5 and e['spawnSeed']==target_seed]
        bs=[e for e in s['room']['pickups']if e['spawnSeed']==target_seed]
        check(label,'target actual manager membership',len(bs),len(ns))
        if ns:
            nt,bt=one(ns),one(bs)
            check(label,'target SKU',[integer(bt[k])for k in ['variant','subtype','spawnSeed']],[integer(nt[k])for k in ['variant','subtype','spawnSeed']])
            check(label,'target body clip',[bt['animation'],finite(bt['animationFrame']),boolean(bt['animationPlaying'])],source_clip(nt))
            check(label,'target radius',finite(bt['collisionRadius']),finite(nt['collisionRadius']))
            check(label,'target mass',finite(bt['collisionMass']),finite(nt['collisionMass']))
            check(label,'target actual classes',[integer(bt['regularCollision'][k])for k in ['gridCollisionClass','entityCollisionClass']],[integer(nt['gridCollisionClass']),integer(nt['entityCollisionClass'])])
            check(label,'target ACC',integer(bt['collectionDelay']),integer(nt['collectionDelay']))
            check(label,'target AD0',integer(bt['pocketContactDelay']),integer(nt['pickupCooldown']))
            check(label,'target active/dead',[not boolean(bt['removed']),boolean(bt['removed'])],[boolean(nt['active']),boolean(nt['removed'])])
            if not free:
                price=bt['shopPrice'];assert isinstance(price,dict)
                check(label,'actual owned price/slot',[integer(price['value']),integer(price['slot'])],[integer(nt['pickupPrice']),integer(nt['shopSlotAc4'])])
                extra=sprite_source(o)
                check(label,'live saved payload',[integer(bt['charge']),boolean(bt['touched']),integer(bt['saveByteAbd'])],[integer(extra['pickupChargeAb8']),boolean(extra['pickupTouchedAbc']),integer(extra['pickupSavedFlagAbdRaw'])])
                ps=extra['priceSprite']
                if boolean(ps['loaded']):
                    check(label,'price active clip',[price['sprite']['animation'],finite(price['sprite']['animationFrame']),boolean(price['sprite']['animationPlaying'])],source_clip(ps))
                    layers=ps['layers'];assert isinstance(layers,list)and [integer(x['id'])for x in layers]==[0,1,2]
                    check(label,'actual price global layer count',integer(ps['globalLayerCount']),3)
                    nf=[integer(x['selectedKeyframeIndex'])if boolean(x['visible'])else None for x in layers]
                    bf=price['sprite']['layerFrames'];assert isinstance(bf,list)and len(bf)==3 and all(x is None or type(x)is int and 0<=x<=11 for x in bf)
                    check(label,'actual numeric price layer frames',bf,nf)
                else:
                    check(label,'present target actual source price clear',[ps['asset'],boolean(ps['loaded']),ps['animation'],ps['animationFrame'],ps['animationPlaying'],integer(ps['globalLayerCount']),ps['layers']],['',False,None,None,None,0,[]])
                    check(label,'present target actual Browser price clear','sprite'in price and price['sprite']is None)
        public_aliases=[x['publicLabel']for x in case['commonSku']['publicCheckpointAliases']if case['actions'][x['actionIndex']]['label']==label]
        is_contact=any(x.startswith('battery-contact-')or x.startswith('paid-contact-')for x in public_aliases)
        if is_contact:
            contact_all.append(dict(label=label,items=n,state=s,observation=o,counter=integer(items['systemCounter'])))
        if (label==before_label or is_contact) and ns:
            nt=target_source(o);bt=target_actual(s)
            actual=dict(label=label,source=nt,own=bt,physical=physical,items=n,state=s,presentation=presentation,observation=o)
            if label==before_label:before_holder.append(actual)
            if is_contact:contact.append(actual)
            # Native448 is the current presented owner; integration remains separate.
            check_f32(label,'actual target P',bt['presentedPosition'],nt['position'])
            rows[-1]['integrationPosition']=vec(bt['position'])
            check_f32(label,'actual target V',bt['velocity'],nt['velocity'])
            check(label,'actual Player logical P',[finite(s['player']['x']),finite(s['player']['y'])],[finite(physical['position'][0])*.65+32,finite(physical['position'][1])*.65-46])
            check(label,'actual Player logical V',[finite(s['player']['vx']),finite(s['player']['vy'])],[finite(physical['velocity'][0])*.65,finite(physical['velocity'][1])*.65])
    before_holder=[]
    required(label,'complete typed actual owners',compare_row)
    if before_holder:before=before_holder[0]
    def seed_effect_owner():
        source=o['commonSkuItemOwners']
        assert source['status']=='pass' and boolean(source['gamePresent'])
        check(label,'source seed-effect mode owner',integer(s['runSeeds']['sourceEffectMode']),
              integer(source['rawGameA1D8']))
    #Keep the independent owner obligation separate: a missing mode must not
    #prevent the existing gameplay comparisons from executing.
    required(label,'source seed-effect mode owner',seed_effect_owner)

def source_witness():
    assert before is not None and target_seed is not None and contact
    expected_id=45 if branch=='paid-undercharged'else 0
    check('witness','Native precontact activeId',integer(before['items']['activeId']),expected_id)
    check('witness','Native precontact charge',integer(before['items']['activeCharge']),0)
    check('witness','Native precontact reserve',integer(before['items']['reserveCharge']),0)
    check('witness','Native initially stationary target',vec(before['source']['velocity']),[0.0,0.0])
    check('witness','Native actual Have63/64 controls',[integer(before['items']['ownedCounts'][i])for i in(63,64)],[0,0])
    eligible=[]
    for row in contact:
        e,pl,it=row['source'],row['physical'],row['items']
        if e['removed']:continue
        radius=finite(e['collisionRadius'])+finite(pl['collisionRadius'])
        distance=math.dist(vec(e['position']),vec(pl['position']))
        if distance<radius:
            eligible.append(row)
            check(row['label'],'strict source contact',distance<radius)
            check(row['label'],'source actual collision scales',[vec(e['collisionScale']),vec(pl['collisionScale'])],[[1.0,1.0],[1.0,1.0]])
            check(row['label'],'source eligible body class',integer(e['entityCollisionClass']),4 if free else 1)
            check(row['label'],'source eligible Player class',integer(pl['entityCollisionClass']),4)
            check(row['label'],'source eligible ACC/AD0',[integer(e['collectionDelay']),integer(e['pickupCooldown'])],[0,0])
            check(row['label'],'source actual normal Player',[integer(pl['type']),integer(pl['variant']),boolean(pl['removed'])],[1,0,False])
            check(row['label'],'source eligible body active',boolean(e['active']),True)
            check(row['label'],'source eligible queue/guard/cooldown',[it['queuedId'],boolean(it['pickupAnimationGuard']),integer(it['inputCooldown'])],[None,False,0])
            main_name=it['mainSprite']['animation']
            assert isinstance(main_name,str)and main_name
            check(row['label'],'source actual CanCollect neutral-guard branch',[boolean(it['pickupAnimationGuard']),main_name!='Death'],[False,True])
            #Declared neutral route guard, not a general50C880 controls veto.
            check(row['label'],'source declared route controls enabled',boolean(it['controlsEnabled']),True)
            if not free:
                check(row['label'],'source paid50C780 item-name veto clear',main_name not in {'Pickup','LiftItem','HideItem','UseItem'})
    check('coverage','actual eligible strict contact observed',bool(eligible))
    outcome_label,no,so=at(outcome_alias)
    n=one(no['commonSkuItemOwners']['players']);ni=before['items']['inventory'];cost=0 if free else integer(before['source']['pickupPrice'])
    if free or branch=='paid-no-active':
        check('witness','refusal actual inventory unchanged',n['inventory'],ni)
        check('witness','refusal actual active words unchanged',[n[k]for k in word_fields],[before['items'][k]for k in word_fields])
        target=target_source(no)
        check('witness','refused body remains active',[target['active'],target['removed']],[True,False])
        if free:
            response=[]
            sequence=[before]+contact
            for left,right in zip(sequence,sequence[1:]):
                lc=integer(left['observation']['commonSkuItemOwners']['systemCounter']);rc=integer(right['observation']['commonSkuItemOwners']['systemCounter'])
                strict=math.dist(vec(left['source']['position']),vec(left['physical']['position']))<finite(left['source']['collisionRadius'])+finite(left['physical']['collisionRadius'])
                if rc==lc+1 and rc%2==1 and strict and vec(right['source']['velocity'])!=vec(left['source']['velocity']) and vec(right['source']['velocity'])!=[0.0,0.0]:response.append(right['label'])
            check('coverage','source AL0-law eligible Full physical-response edge',bool(response))
            diagnostics.append(dict(field='actual eligible free-response edges',labels=response,scope='Strict pre-Full stored overlap and subsequent actual Full velocity change; no AL register observation.'))
        else:
            #A Fast-only overlap plus no grant does not qualify paid AL1.
            #Require the strict eligible observation itself on an actual Full:
            #source odd System phase and adjacent actual Game-update advance.
            full_contact=[]
            sequence=[before]+contact
            for row in eligible:
                counter=integer(row['observation']['commonSkuItemOwners']['systemCounter'])
                if counter%2!=1:continue
                current_full=integer(row['observation']['shopFinance']['contents']['nativePhase']['fullGameUpdates'])
                previous=[left for left in sequence if integer(left['observation']['commonSkuItemOwners']['systemCounter'])==counter-1]
                if len(previous)==1 and current_full==integer(previous[0]['observation']['shopFinance']['contents']['nativePhase']['fullGameUpdates'])+1:
                    full_contact.append(dict(label=row['label'],systemCounter=counter,fullGameUpdates=current_full))
            check('coverage','actual strict eligible paid-refusal Full contact',bool(full_contact))
            diagnostics.append(dict(field='actual paid-refusal Full contact',observations=full_contact,scope='Strict eligible current Full observation plus adjacent actual source Game-update increment; Fast-only overlap cannot pass.'))
            check('witness','actual affordable refusal',ni['coins']>=cost and cost==5)
            check('witness','source AL1-law anchored body',[vec(target['position']),vec(target['velocity'])],[vec(before['source']['position']),[0.0,0.0]])
    else:
        check('witness','actual funded price',ni['coins']>=cost and cost==(15 if collectible else 5))
        check('witness','actual once debit',integer(n['inventory']['coins']),integer(ni['coins'])-cost)
        coin_sequence=[integer(ni['coins'])]+[integer(r['items']['inventory']['coins'])for r in contact_all]+[integer(n['inventory']['coins'])]
        changes=[right-left for left,right in zip(coin_sequence,coin_sequence[1:])if right!=left]
        check('witness','actual single price-sized debit edge',changes,[-cost])
        if collectible:
            check('witness','actual deferred grant count',integer(n['ownedCounts'][63]),1)
            check('coverage','actual queued63 before count1',any(r['items']['queuedId']==63 and r['items']['ownedCounts'][63]==0 for r in contact))
        else:
            all_items=[one(v['observation']['commonSkuItemOwners']['players'])for v in native.values()]
            check('coverage','actual free45 queued before grant',any(x['queuedId']==45 and x['ownedCounts'][45]==0 for x in all_items))
            check('coverage','actual45 fullcharge before public use',any(x['activeId']==45 and x['activeCharge']==4 and x['ownedCounts'][45]==1 for x in all_items))
            config=one([x for x in no['commonSkuItemOwners']['configs']if x['id']==45])
            check('witness','source actual45 maximum',integer(config['maxCharges']),4)
            check('witness','actual undercharge setter words',[integer(n[k])for k in ['activeId','activeCharge','reserveCharge']],[45,integer(config['maxCharges']),0])
        retired=[r for r in contact if r['source']['removed']]
        check('coverage','actual present retired paid member',bool(retired))
        check('coverage','actual later source manager erasure',not any(e['type']==5 and e['spawnSeed']==target_seed for e in no['shopFinance']['contents']['entities']))
        if retired:
            r=retired[0];extra=sprite_source(r['observation']);copied=one(r['presentation']['players'])['pickupSprite'];main=one(r['observation']['commonSkuItemOwners']['players'])['mainSprite']
            check('witness','actual main paid Pickup',main['animation'],'Pickup')
            if not collectible:
                check('witness','actual copied native regular asset from own body',copied['asset'],extra['bodySprite']['asset'])
            sound_id=130 if collectible else 218
            initial_audio=one([x for x in before['observation']['commonSkuAudio']['slots']if x['id']==sound_id])
            current_audio=one([x for x in r['observation']['commonSkuAudio']['slots']if x['id']==sound_id])
            own_audio=r['state']['audio']['slots'][str(sound_id)]
            check('witness','actual new source sound slot admission',integer(current_audio['lastTick'])!=integer(initial_audio['lastTick']))
            check('witness','actual relative source sound slot',[integer(own_audio['lastTick'])-(integer(r['state']['systemClock']['systemCounter'])>>1),integer(own_audio['nextAllowed'])-integer(own_audio['lastTick']),finite(own_audio['lastVolume']),boolean(own_audio['loaded'])],[integer(current_audio['lastTick'])-(integer(r['observation']['commonSkuAudio']['systemCounter'])>>1),integer(current_audio['nextAllowed'])-integer(current_audio['lastTick']),finite(current_audio['lastVolume']),boolean(current_audio['loaded'])])
            own=r['state']['playerItems']['pickupPresentation'];assert isinstance(own,dict)
            if collectible:
                check('witness','actual retired collectible subtype',integer(r['source']['subtype']),0)
                check('witness','actual Native collectible Sparkle',copied['animation'],'PlayerPickupSparkle')
                check('witness','actual deferred collectible owner',integer(own['itemId']),63)
                check('witness','actual deferred copy clock',[finite(own['animationFrame']),boolean(own['animationPlaying'])],[finite(copied['animationFrame']),boolean(copied['animationPlaying'])])
            else:
                regular=own['regular'];assert isinstance(regular,dict)
                check('witness','actual paid regular owner',[regular['variant'],regular['subtype']],[90,0])
                check('witness','actual regular copied clock',[regular['animation'],finite(regular['animationFrame']),boolean(regular['animationPlaying'])],source_clip(copied))
                check('witness','Browser copy equals own prebody actor',regular['actor'],before['own']['actor'])
required('witness','complete mandatory source/functional witness',source_witness)
unavailable += [dict(field='private callback AL register/counts',reason='Source branch law plus actual eligibility/physical behavior is compared; no register/event probe.'),dict(field='charge HUD/effect48, backend played audio/PCM and pixels',reason='Not established by word equality, source table snapshots or PNG retention.'),dict(field='direct Native saved container / other three branch controls',reason='Outside this first-four public fixture.')]
failed=[r for r in rows if r['status']=='fail']
result=dict(schema=1,status='fail'if failed else'pass',case=case['name'],checks=len(rows),failures=len(failed),rows=rows,unavailable=unavailable,diagnostics=diagnostics,
  pins={str(p):digest(p)for p in [a.case,progress,Path(str(progress)+'.meta.json'),browser_path,Path(str(browser_path)+'.meta.json'),Path(__file__)]},
  scope='Actual independently captured Native expected owners versus actual DOM/WASM owners, plus mandatory source branch witness. No generated Native observations, defaults for absent data, retiming, tolerance, or historical report mutation.')
a.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
print(json.dumps({k:result[k]for k in ['status','case','checks','failures']}))
raise SystemExit(bool(failed))
