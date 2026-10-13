#!/usr/bin/env python3
"""Reject missing clock/input/state/pixel evidence and corrupt seed banks."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from PIL import Image

spec = importlib.util.spec_from_file_location('clock_verify', Path(__file__).with_name('foundation-clock-verify.py'))
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


class ClockVerificationChecks(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix='clock-compare-test-')
        self.addCleanup(self.directory.cleanup)
        path = Path(self.directory.name) / 'pixels.png'
        Image.new('RGBA', (960, 540), (100, 80, 60, 255)).save(path)
        vector = verify.codec.random_vector()
        self.vector = vector
        value = vector['seedValue']
        self.run = {'status': 'pass', 'seed': vector['code'], 'seedValue': value,
                    'seedReadbackMatchesCodec': True, 'profile': {'fresh': True, 'isolated': True},
                    'observedState': {'seed': vector['code'], 'seedValue': value, 'stage': 1, 'altStage': 0,
                                      'stageSeed': verify.codec.initial_stage_seed(value), 'rooms': 8, 'room': 'start', 'spawnSeed': 123},
                    'simulationTicks': False, 'fullInternalState': False, 'bootstrapPresentedFrames': 30,
                    'clock': {'stepNs': 16667000},
                    'tape': [{'type': 'key', 'key': 'd', 'down': True}, {'type': 'step', 'frames': 1}, {'type': 'capture', 'label': 'movement'}],
                    'inputs': [{'key': 'd', 'down': True, 'keysym': 100, 'presentedFrame': 30, 'virtualElapsedNs': 500010000}],
                    'presentations': [{'presentedFrame': f, 'virtualElapsedNs': f*16667000, 'inputOverflow': False,
                                      'consumedX11Events': [] if f != 31 else [{'down': True, 'keysym': 100, 'presentedFramePhase': 31, 'virtualElapsedNs': 516677000}]} for f in [0, 30, 31]],
                    'frames': [{'label': 'movement', 'path': str(path), 'width': 960, 'height': 540, 'gateBlocked': True,
                                'simulationTick': None, 'presentedFrame': 31, 'virtualElapsedNs': 516677000}]}

    def repeat(self):
        other = copy.deepcopy(self.run)
        path = Path(self.directory.name) / 'repeat.png'
        with Image.open(self.run['frames'][0]['path']) as image:
            image.save(path)
        other['frames'][0]['path'] = str(path)
        return other

    def reject(self, mutate):
        other = self.repeat()
        mutate(other)
        result = verify.compare(self.run, other)
        self.assertEqual(result['status'], 'fail', result)
        self.assertTrue(result['failures'])

    def test_complete_evidence_passes(self):
        self.assertEqual(verify.compare(self.run, self.repeat())['status'], 'pass')

    def test_shared_capture_is_red(self):
        result = verify.compare(self.run, copy.deepcopy(self.run))
        self.assertEqual(result['status'], 'fail')

    def test_missing_log_is_red(self):
        self.reject(lambda x: x.pop('observedState'))

    def test_wrong_actual_seed_is_red(self):
        self.reject(lambda x: x['observedState'].update(seedValue=0))

    def test_wrong_actual_stage_model_is_red(self):
        self.reject(lambda x: x['observedState'].update(stageSeed=x['observedState']['stageSeed'] ^ 1))

    def test_missing_capture_is_red(self):
        self.reject(lambda x: x.update(frames=[]))

    def test_missing_selected_capture_is_red(self):
        self.reject(lambda x: x['tape'].append({'type': 'capture', 'label': 'unobserved'}))

    def test_duplicate_capture_is_red(self):
        self.reject(lambda x: x['frames'].append(copy.deepcopy(x['frames'][0])))

    def test_wrong_capture_phase_is_red(self):
        self.reject(lambda x: x['frames'][0].update(presentedFrame=32))

    def test_unreadable_pixels_is_red(self):
        self.reject(lambda x: x['frames'][0].update(path='/no/such/frame.png'))

    def test_one_channel_difference_is_red(self):
        path = Path(self.directory.name) / 'changed.png'
        image = Image.new('RGBA', (960, 540), (100, 80, 60, 255))
        image.putpixel((480, 270), (101, 80, 60, 255))
        image.save(path)
        self.reject(lambda x: x['frames'][0].update(path=str(path)))

    def test_missing_presentations_is_red(self):
        self.reject(lambda x: x.pop('presentations'))

    def test_clock_time_difference_is_red(self):
        self.reject(lambda x: x['presentations'][-1].update(virtualElapsedNs=0))

    def test_missing_consumed_key_is_red(self):
        self.reject(lambda x: x['presentations'][-1].update(consumedX11Events=[]))

    def test_autorepeat_invented_key_is_red(self):
        self.reject(lambda x: x['presentations'][-1]['consumedX11Events'].append(copy.deepcopy(x['presentations'][-1]['consumedX11Events'][0])))

    def test_input_overflow_is_red(self):
        self.reject(lambda x: x['presentations'][-1].update(inputOverflow=True))

    def test_hidden_state_claim_is_red(self):
        self.reject(lambda x: x.update(fullInternalState=True))

    def test_malformed_evidence_is_red_without_exception(self):
        self.reject(lambda x: x.update(frames=None, presentations=None))

    def test_empty_evidence_cannot_match(self):
        result = verify.compare({'status': 'pass'}, {'status': 'pass'})
        self.assertEqual(result['status'], 'fail')
        self.assertFalse(result['publicLogStateMatches'])
        self.assertFalse(result['publicInputConsumptionMatches'])

    def test_valid_unique_random_bank(self):
        bank = {'schema': 1, 'vectors': [self.vector]}
        self.assertEqual(verify.random_seeds.validate_bank(bank, 1), [self.vector])

    def test_duplicate_bank_is_red(self):
        with self.assertRaises(ValueError):
            verify.random_seeds.validate_bank({'schema': 1, 'vectors': [self.vector, self.vector]}, 2)

    def test_count_mismatch_is_red(self):
        with self.assertRaises(ValueError):
            verify.random_seeds.validate_bank({'schema': 1, 'vectors': [self.vector]}, 2)


if __name__ == '__main__':
    unittest.main()
