"""Normal public resource-fixture capture; preparation launches neither game."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def public_tape(case,clock):
    tape=clock.default_tape(case['publicSeedCode'])
    end=next(i for i,a in enumerate(tape)if a.get('label')=='gameplay-start')
    tape=tape[:end+1]
    expected=[dict(type='key',key='Return',down=True),dict(type='step',frames=12),dict(type='key',key='Return',down=False),dict(type='step',frames=300),dict(type='capture',label='gameplay-start')]
    if tape[-5:]!=expected:raise ValueError('Canonical normal prefix changed')
    tape[-5:]=[expected[0]]+[dict(type='step',frames=1)for _ in range(12)]+[expected[2]]+[dict(type='step',frames=1)for _ in range(300)]+[expected[4]]
    held=set()
    for a in case['actions']:
        keys=set(a['keys'])
        if len(keys)!=len(a['keys'])or not keys<={'Right','Left','Up','Down','w','a','s','d','space','e','q'}:
            raise ValueError('Undeclared public gameplay key')
        if type(a['presentations'])is not int or not 1<=a['presentations']<=400:raise ValueError('Invalid declared interval')
        tape += [dict(type='key',key=k,down=False)for k in sorted(held-keys)]
        tape += [dict(type='key',key=k,down=True)for k in sorted(keys-held)]
        for i in range(1,a['presentations']+1):
            tape.append(dict(type='step',frames=1))
            if i in(3,4):tape.append(dict(type='capture',label=a['label']+'-observe-step'+str(i)))
        tape.append(dict(type='capture',label=a['label']))
        held=keys
    tape += [dict(type='key',key=k,down=False)for k in sorted(held)]
    clock.validate_tape(tape)
    return tape

def heavy_labels(case):
    labels=['gameplay-start']
    for a in case['actions']:
        labels += [a['label']+'-observe-step'+str(i)for i in(3,4)if a['presentations']>=i]
        labels.append(a['label'])
    if len(labels)!=len(set(labels)):raise ValueError('Duplicate declared observation label')
    return labels

def main():
    parser=argparse.ArgumentParser(allow_abbrev=False)
    parser.add_argument('--case',type=Path,required=True)
    parser.add_argument('--source-plans',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    mode=parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--prepare-only',action='store_true')
    mode.add_argument('--capture',action='store_true')
    args=parser.parse_args()
    case=json.loads(args.case.read_text())
    if not isinstance(case,dict):raise ValueError('One resolved public case required')
    if case['commonSku'].get('branch')not in {'free-no-active','paid-no-active','paid-undercharged','paid100'}:
        raise ValueError('Unadmitted common SKU branch')
    if(case['commonSku']['schema']!='source-public-common-sku-v1'or case['publicMenuRoute']!='seeded-normal'
       or case['fixture']['challengeId']!=0 or case['fixture']['startingItems']!=[]or case['initialPhaseOffset']!=0):
        raise ValueError('Only explicitly declared normal challenge0/offset0 source fixtures admitted')
    composition=load('common_sku_resource_owner',ROOT/'tools/original/composition-run.py')
    clock=load('common_sku_canonical_clock',ROOT/'tools/original/foundation-clock-run.py')
    if clock.codec.decode_seed(case['publicSeedCode'])!=case['publicRunSeed']:raise ValueError('Declared public seed mismatch')
    xml,stb=map(Path,[case['fixture']['authoredResourcePath'],case['fixture']['specialRoomsPath']])
    if sha(xml)!=sha(ROOT/'references/isaac/rebirth/resources/challenges.xml')or sha(stb)!=case['fixture']['specialRoomsSha256']:
        raise ValueError('Exact unchanged XML/authored STB input required')
    plans=json.loads(args.source_plans.read_text())
    observer_module=load('common_sku_source_observer',HERE/'common-sku-observe.py')
    observer_module.create_observer(plans,heavy_labels(case)) # validate source-only plan before capture
    pins={str(p):sha(p)for p in [Path(__file__),args.case,args.source_plans,
      HERE/'common-sku-observe.py',HERE/'shop-finance-observe.py',HERE/'shop-finance-law.py',
      HERE/'shop-finance-layout-observe.py',HERE/'shop-finance-presentation-observe.py',
      HERE/'item-observer.py',HERE/'audio-observer.py',xml,stb,
      ROOT/'tools/original/composition-run.py',ROOT/'tools/original/foundation-clock-run.py']}
    tape=public_tape(case,clock)
    output=args.output.resolve()
    output.mkdir(parents=True,exist_ok=False)
    composition.stage_view(xml,output/'resource-view',stb)
    (output/'case.json').write_text(json.dumps(case,indent=2)+'\n')
    tape_path=output/'declared-tape.json'
    tape_path.write_text(json.dumps(tape,indent=2)+'\n')
    hashes={'challengesXml':sha(xml),'specialRooms':sha(stb)}
    metadata={'schema':1,'scope':'Source-authored normal common SKU resource fixture; no Native observation is an input',
      'execution':'prepared-not-executed','nativeDepotChanged':False,'nativeObservationUsedAsInput':False,
      'declaredResourceHashes':hashes,'declaredSfxVolume':case['sfxVolume'],'requiredCommonSkuContract':case['commonSku'],
      'heavyLabels':heavy_labels(case),'sourcePins':pins,'bootstrapBoundary':'Canonical Native bootstrap/intro versus separate Browser first-Title boundary; no invented shared boot tick'}
    # This wrapper is completely declared before either game. It adds no observed input.
    observer_path=output/'declared-observer.py'
    observer_path.write_text('import importlib.util, json\nfrom pathlib import Path\n'
      + 'spec=importlib.util.spec_from_file_location("common_sku_bound",'+repr(str(HERE/'common-sku-observe.py'))+')\n'
      + 'module=importlib.util.module_from_spec(spec)\nspec.loader.exec_module(module)\n'
      + 'observe=module.create_observer(json.loads('+repr(json.dumps(plans))+'),json.loads('
      + repr(json.dumps(metadata['heavyLabels']))+'))\n')
    metadata['declaredObserverSha256']=sha(observer_path)
    meta_path=output/'composition-input.json'
    meta_path.write_text(json.dumps(metadata,indent=2)+'\n')
    if args.prepare_only:
        print(json.dumps({'status':'prepared-not-executed','directory':str(output)}))
        return 0
    expected=composition.canonical_original()
    previous=os.environ.get('ISAAC_ORIGINAL_SHA')
    os.environ['ISAAC_ORIGINAL_SHA']=expected
    clock.ROOT=output/'resource-view'
    old_argv=sys.argv
    metadata.update(execution='original-started',nativeBinarySha256=expected)
    try:
        sys.argv=[str(ROOT/'tools/original/foundation-clock-run.py'),'--seed',case['publicSeedCode'],
          '--boot-epoch-seed',str(case['bootEpochSeed']),'--tape',str(tape_path),'--output',str(output),
          '--state-observer',str(observer_path),'--observe-every-step','--sfx-volume',str(case['sfxVolume']),
          '--archive-evidence','--journal-codec','zstd','--manifest-codec','zstd']
        result=clock.main()
        metadata.update(execution='original-closed',originalExitStatus=result)
        return result
    except BaseException as error:
        metadata.update(execution='original-failed',originalError=str(error))
        raise
    finally:
        sys.argv=old_argv
        if previous is None:os.environ.pop('ISAAC_ORIGINAL_SHA',None)
        else:os.environ['ISAAC_ORIGINAL_SHA']=previous
        after={'challengesXml':sha(xml),'specialRooms':sha(stb)}
        metadata.update(declaredResourceHashesAfter=after,declaredResourcesBeforeEqualsEnd=after==hashes,
                        sourcePinsAfter={p:sha(p)for p in pins})
        meta_path.write_text(json.dumps(metadata,indent=2)+'\n')
        if after!=hashes or metadata['sourcePinsAfter']!=pins:raise RuntimeError('Frozen public resource/source changed during capture')

if __name__=='__main__':raise SystemExit(main())
