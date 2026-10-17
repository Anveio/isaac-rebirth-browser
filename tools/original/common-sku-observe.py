"""Explicit common-SKU composition of bounded existing read-only owners."""
import importlib.util
from pathlib import Path


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def create_observer(source_plans, heavy_labels):
    if (not isinstance(heavy_labels, list) or not heavy_labels
            or len(heavy_labels) != len(set(heavy_labels))
            or not all(isinstance(label, str) and label for label in heavy_labels)):
        raise ValueError('Unique independently declared common SKU labels required')
    heavy = frozenset(heavy_labels)
    finance = load('common_sku_shared_finance', 'shop-finance-observe.py')
    actual = finance.create_observer(source_plans, common_sku_owners=True)
    paired = load('common_sku_existing_paired', 'foundation-paired-observer.py')
    audio = load('common_sku_existing_audio', 'audio-observer.py')

    def observe(pid, context):
        if context.get('label') not in heavy:
            result = paired.observe(pid, context)
            result['executableMappingsRead'] = False
            result['commonSkuScope'] = dict(status='skip', scope=
                'Basic actual paired owners outside predeclared heavy labels; no SKU fields inferred')
            return result
        routed = dict(context, heldPocketRecords=True, pickupResourceSound=True,
                      includePickupWorld=True)
        result = actual(pid, routed)
        result['executableMappingsRead'] = False
        owners = result.get('commonSkuItemOwners', {})
        if context.get('publicLogState', {}).get('stage') != 1:
            result['status'] = 'fail'
            owners.update(status='fail', error='Required common SKU label is not stage1 gameplay')
            return result
        sound = audio.observe(pid, routed)
        result['commonSkuAudio'] = sound
        if (owners.get('status') != 'pass' or owners.get('gamePresent') is not True
                or owners.get('rawGameA1D8') != 0
                or type(owners.get('systemCounter')) is not int
                or owners.get('systemCounter') != result.get('shopPlayerPresentation', {}).get('systemCounter')
                or sound.get('status') != 'pass' or sound.get('originalPid') != pid
                or sound.get('systemCounter') != owners.get('systemCounter')):
            result['status'] = 'fail'
            owners.update(status='fail', error='Actual normal item/audio/finance same-PID/turn join failed')
        return result

    return observe
