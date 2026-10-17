"""Independent canonical pickup/item/audio owners at one blocked Native gate."""
import importlib.util
from pathlib import Path


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


items = load('pocket_literal_item_owners', 'item-observer.py')
pickups = load('pocket_literal_pickup_owners', 'foundation-pickup-observe.py')
audio = load('pocket_literal_audio_owners', 'audio-observer.py')


def observe(pid, context):
    selected = dict(context, heldPocketRecords=True)
    if context.get('pillRangeUse') is True:
        selected['persistentItemInputs'] = True
    results = {}
    for name, module in (('pickup', pickups), ('items', items)):
        try:
            results[name] = module.observe(pid, selected)
        except Exception as error:
            results[name] = dict(status='fail', error=repr(error), memoryWritten=False, executableMappingsRead=False)
    if results['items'].get('gamePresent') is True:
        try:
            actual = audio.observe(pid, selected)
            keys = ('schema', 'status', 'memoryWritten', 'executableMappingsRead', 'bytesRead',
                    'nativeBinarySha256', 'soundCatalogSha256', 'checkpoint', 'originalPid',
                    'scope', 'systemCounter', 'cooldownTick', 'mtIndex', 'globalSfxVolume',
                    'managerRoot', 'gamePresent', 'configuredSounds', 'sampleReferences',
                    'catalogCountsMatch', 'pathMismatches', 'error')
            result = {key: actual[key] for key in keys if key in actual}
            result['slots'] = [slot for slot in actual.get('slots', []) if slot['id'] in (8, 252)]
            if result.get('status') == 'pass' and {slot['id'] for slot in result['slots']} != {8, 252}:
                result.update(status='fail', error='Actual canonical pocket sound slot roster incomplete')
            results['audio'] = result
        except Exception as error:
            results['audio'] = dict(status='fail', error=repr(error), memoryWritten=False, executableMappingsRead=False)
    else:
        results['audio'] = dict(schema=1, status='skip', reason='No actual gameplay owner',
                                memoryWritten=False, executableMappingsRead=False)
    return dict(schema=1, status='pass' if all(value.get('status') in ('pass', 'skip') for value in results.values()) else 'fail',
                scope='Independent current pickup, held-pocket and selected sound8/252 table owners. Raw Game+A1D8 is not a challenge-key alias. Slot/sample snapshots do not establish a callback event, unique selected sample, backend playing or audible PCM.',
                memoryWritten=False, executableMappingsRead=False, **results)
