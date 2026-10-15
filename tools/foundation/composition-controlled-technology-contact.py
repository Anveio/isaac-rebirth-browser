"""Distinct7/2/1 Line50/mode1 NPC14/18 contact states, no Brim actor alias."""
import importlib.util
from pathlib import Path


def compare_contact(case, rows, common):
    spec = importlib.util.spec_from_file_location(
        'typed_ring_parentplayer_receiver_core',
        Path(__file__).with_name('composition-controlled-brim-contact.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    #Only receiver/FIFO/HP state laws are shared. The actual schema, C08,
    #7/2/1 C18 and source-scoped TechnologyLine writer remain explicit.
    result = module.compare_typed_ring_contact(case, rows, common,
        marker='controlledTechnologyContact',
        schema='controlled-technology-ring-contact-v1', primary=3, variant=2)
    result['scope'] = ('Observed compatible ParentPlayer FIFO append and ordered following-Full HP endpoint. '
        'A negative HP/dead endpoint is separate from subclass/Kill/Remove/Uninit completion. '
        'Exact Laser attribution and collision delivery still require actual callback witnesses.')
    result['registeredImpacts'] = registered_impacts(rows, common)
    return result


def impact_reference(state, ref):
    """Canonical nested Impact50 with an explicit stored sourceSubtype0 owner."""
    assert ref['kind'] == 'effect', 'Actual registered Effect handle required'
    bodies = [a['impact'] for a in state['lasers']
        if a['impact'] is not None and a['impact']['id'] == ref['id']]
    identities = [i for i in state['actorScheduler']['identities'] if i['actor'] == ref]
    assert len(bodies) == len(identities) == 1, 'Actual Impact reference must resolve uniquely'
    actual = bodies[0]
    assert type(actual['sourceSubtype']) is int and actual['sourceSubtype'] == 0
    #Nested Impact is the canonical Effect50 store. Its subtype is stored;
    #the older shared reference() remains source-scoped to Brim subtype1.
    return dict(kind=1000, variant=50, subtype=actual['sourceSubtype'],
        sequenceId=identities[0]['sequenceId'], spawnSeed=actual['spawnSeed'])


def registered_impacts(rows, common):
    """Resolve each game's actual Child4A0; never infer it from actor age."""
    identity, reference = common['identity_tuple'], common['reference']
    check, field, missing = (common[k] for k in ('check', 'check_field', 'missing'))
    observations = {side: {} for side in ('native', 'port')}
    for captured in rows:
        t, source, state = (captured[k] for k in ('presentation', 'source', 'state'))
        children = {}
        for side in ('native', 'port'):
            try:
                native = side == 'native'
                slot = source['players'][0]['actorSlots']['c18'] if native else state['attackRuntime']['persistent']
                if slot is None:
                    continue
                actors = ([a for a in source['attacks'] if a['identity'] == slot['identity']] if native
                    else [a for a in state['lasers'] if slot['kind'] == 'laser' and a['id'] == slot['id']])
                assert len(actors) == 1, 'One actual own C18 Laser required'
                actor = actors[0]
                parent = identity(actor['identity'] if native else reference(state, slot))
                assert parent[:3] == (7, 2, 1), 'Actual Technology7/2/1 owner required'
                child = actor['laser']['child'] if native else actor['child']
                assert child is not None, 'Actual registered canonical Impact required; no Brim no-Impact alias'
                key = identity(child if native else impact_reference(state, child))
                assert key[:3] == (1000, 50, 0), 'Actual canonical Impact50/0 role required'
                bodies = ([a for a in source['impacts'] if a['identity'] == child] if native
                    else [actor['impact']] if actor['impact'] is not None and actor['impact']['id'] == child['id'] else [])
                assert len(bodies) == 1, 'Actual registered child body required'
                impact = bodies[0]
                linked_parent = identity(impact['linkedParent'] if native else reference(state, impact['parent']))
                manager = impact['inActiveManager'] if native else child in state['actorScheduler']['actors']
                assert type(manager) is bool, 'Actual manager membership required'
                check('controlled Technology contact ' + side + ' Impact actual ParentLaser', parent, linked_parent, t)
                check('controlled Technology contact ' + side + ' Impact actual registration', True, manager, t)
                children[side] = key
                entry = observations[side].setdefault(key, {'identity': key, 'rows': 0, 'registeredRows': 0})
                entry['rows'] += 1
                entry['registeredRows'] += int(manager and linked_parent == parent)
            except Exception as error:
                missing('controlled Technology contact ' + side + ' actual registered Impact owner', t, error)
        if children:
            field('controlled Technology contact registered Impact exact identity/seed',
                lambda: children['native'], lambda: children['port'], t)
    for side in ('native', 'port'):
        check('required controlled Technology contact ' + side + ' registered canonical Impact', True,
            any(entry['registeredRows'] > 0 for entry in observations[side].values()), None)
    return {side: list(values.values()) for side, values in observations.items()}
