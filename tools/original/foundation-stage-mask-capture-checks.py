#!/usr/bin/env python3
"""Guard controlled mask phases against hidden extra frames and input drift."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from copy import deepcopy

spec = importlib.util.spec_from_file_location('stage_mask_capture', Path(__file__).with_name('foundation-stage-mask-capture.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def schedule(tape, checkpoint):
    frames = 0
    events = []
    for action in tape:
        if action['type'] == 'step':
            frames += action['frames']
        elif action['type'] == 'key':
            events.append((action['key'], action['down'], frames))
        elif action.get('label') == checkpoint:
            return frames, events
    raise AssertionError('checkpoint missing')


class ControlledMaskCaptureContracts(unittest.TestCase):
    def test_snapshot_arming_preserves_all_original_input_and_presentation_phases(self):
        for seed in ['3BMB MFTE', 'J4G3 9N4Q']:
            original = module.runner.default_tape(seed)
            controlled = module.build_tape(seed)
            for checkpoint in ['gameplay-start', 'movement']:
                self.assertEqual(schedule(original, checkpoint), schedule(controlled, checkpoint))
            neutral_frames, _ = schedule(controlled, 'gameplay-start')
            movement_frames, _ = schedule(controlled, 'movement')
            bomb_frames, _ = schedule(controlled, 'bomb-planted')
            self.assertEqual(movement_frames - neutral_frames, 57)
            self.assertEqual(bomb_frames - movement_frames, 2)

    def fixture(self):
        return {'frames': [{'label': 'gameplay-start', 'presentedFrame': 1206},
                          {'label': 'movement', 'presentedFrame': 1263},
                          {'label': 'bomb-planted', 'presentedFrame': 1265}],
                'presentations': [{'consumedX11Events': [
                    {'keysym': 100, 'down': True, 'presentedFramePhase': 1207},
                    {'keysym': 100, 'down': False, 'presentedFramePhase': 1234},
                    {'keysym': 101, 'down': True, 'presentedFramePhase': 1264},
                    {'keysym': 101, 'down': False, 'presentedFramePhase': 1265}], 'inputOverflow': False}]}

    def test_shifted_or_missing_actual_x11_consumption_fails(self):
        module.validate_input_phases(self.fixture())
        for index in range(4):
            wrong = self.fixture()
            wrong['presentations'][0]['consumedX11Events'][index]['presentedFramePhase'] += 1
            with self.assertRaisesRegex(ValueError, 'consume'):
                module.validate_input_phases(wrong)
        missing = self.fixture()
        missing['presentations'][0]['consumedX11Events'].pop(0)
        with self.assertRaises(ValueError):
            module.validate_input_phases(missing)

    def test_missing_release_interval_extra_idle_frame_and_event_overflow_fail(self):
        for index in [1, 2]:
            wrong = self.fixture()
            wrong['frames'][index]['presentedFrame'] += 1
            with self.assertRaises(ValueError):
                module.validate_input_phases(wrong)
        overflow = self.fixture()
        overflow['presentations'][0]['inputOverflow'] = True
        with self.assertRaisesRegex(ValueError, 'overflow'):
            module.validate_input_phases(overflow)

    def test_reference_coordinate_export_omits_native_addresses(self):
        candidate = {'offset': 0, 'address': 'private-address', 'f32': [320, 400, 0, 0, 0, 0, 0, 2, 0]}
        result = module.position_candidates({'candidates': [candidate]})
        self.assertEqual(result, [{'worldPairCandidate': [320, 400], 'velocityPairCandidate': [2, 0]}])
        self.assertNotIn('address', str(result))

    def test_duplicate_checkpoints_and_unrequested_gameplay_keys_fail(self):
        duplicate = self.fixture()
        duplicate['frames'].append(duplicate['frames'][0].copy())
        with self.assertRaisesRegex(ValueError, 'unique'):
            module.validate_input_phases(duplicate)
        extra = self.fixture()
        extra['presentations'][0]['consumedX11Events'].append(
            {'keysym': 119, 'down': True, 'presentedFramePhase': 1260})
        with self.assertRaisesRegex(ValueError, 'exactly'):
            module.validate_input_phases(extra)

    def test_missing_selected_masks_duplicate_requests_and_duplicate_gl_labels_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            native = self.fixture()
            native.update({'status': 'pass', 'debugObservations': [], 'graphicsTrace': {'path': str(root / 'gl.jsonl')}})
            manifest = root / 'manifest.json'
            scenarios = [
                ([], [], 'Every selected'),
                ([{'label': 'gameplay-start'}, {'label': 'gameplay-start'}], [], 'requests must be unique'),
                ([], [{'op': 'frameStart', 'label': 'movement', 'frame': 1},
                      {'op': 'frameStart', 'label': 'movement', 'frame': 2}], 'frame labels must be unique'),
            ]
            for requests, records, message in scenarios:
                native['debugObservations'] = [{'observation': {'graphicsRequest': request}} for request in requests]
                manifest.write_text(json.dumps(native))
                (root / 'gl.jsonl').write_text('\n'.join(json.dumps(record) for record in records))
                with patch.object(module.graphics, 'observe', return_value={'frames': [], 'snapshots': []}), \
                        self.assertRaisesRegex(ValueError, message):
                    module.extract(manifest, root / 'targets')

    def test_explosion_masks_use_requested_public_phases_without_extra_input(self):
        tape = module.build_tape('3BMB MFTE', explosion=True)
        movement, _ = schedule(tape, 'movement')
        for label, phase in [('explosion-start', 121), ('explosion-faded', 123),
                             ('explosion-middle', 127), ('explosion-late', 131), ('explosion-removed', 133)]:
            at, _ = schedule(tape, label)
            self.assertEqual(at - movement, phase)

    def test_missing_or_changed_depth_state_and_attachment_fail(self):
        state = {'depthEnabled': True, 'depthFunction': [516], 'depthWriteMask': [1],
                 'depthBits': [24], 'depthClearValue': [0], 'depthRange': [0, 1],
                 'depthAttachment': {'framebuffer': 1, 'available': True, 'objectType': 36161,
                    'object': 1, 'renderbufferParametersAvailable': True, 'width': 1024, 'height': 1024,
                    'internalFormat': 33190, 'depthBits': 24, 'stencilBits': 0, 'samples': 0}}
        self.assertEqual(module.depth_contract(state), state)
        for key, replacement in [('depthEnabled', False), ('depthEnabled', 1), ('depthFunction', [519]),
                                 ('depthWriteMask', [0]), ('depthBits', [16]),
                                 ('depthClearValue', [1]), ('depthRange', [1, 0]), ('depthRange', [0, True])]:
            wrong = deepcopy(state)
            wrong[key] = replacement
            with self.subTest(field=key), self.assertRaisesRegex(ValueError, 'depth contract'):
                module.depth_contract(wrong)
        for key, replacement in [('available', False), ('objectType', 0), ('object', 0),
                                 ('framebuffer', 0), ('renderbufferParametersAvailable', False),
                                 ('width', 512), ('height', 512), ('internalFormat', 33189),
                                 ('depthBits', 16), ('stencilBits', 8), ('samples', 4)]:
            wrong = deepcopy(state)
            wrong['depthAttachment'][key] = replacement
            with self.subTest(field=key), self.assertRaisesRegex(ValueError, 'renderbuffer'):
                module.depth_contract(wrong)
        for wrong in [None, {}, {key: value for key, value in state.items() if key != 'depthAttachment'},
                      {**state, 'depthAttachment': []}]:
            with self.assertRaises(ValueError):
                module.depth_contract(wrong)


if __name__ == '__main__':
    unittest.main()
