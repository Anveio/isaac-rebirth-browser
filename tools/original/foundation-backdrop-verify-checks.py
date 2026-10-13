#!/usr/bin/env python3
"""Regression guards against accepting incomplete or reused backdrop evidence."""
import copy
import importlib.util
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch
import subprocess

spec = importlib.util.spec_from_file_location('backdrop_verify', Path(__file__).with_name('foundation-backdrop-verify.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class BackdropVerificationContracts(unittest.TestCase):
    def setUp(self):
        self.expected = [{'sourcePatchIndex': 0,
            'sourceRect': {'x': 0, 'y': 0, 'width': 52, 'height': 52},
            'destRect': {'x': 0, 'y': 0, 'width': 52, 'height': 52},
            'flipX': False, 'flipY': False}]
        self.observed = [{'xySubmitted': True, 'uvSubmitted': True,
            'sourceRect': [0, 0, 52, 52], 'xy': [[0, 0], [52, 0], [0, 52], [52, 52]],
            'sourcePixels': [[0, 0], [52, 0], [0, 52], [52, 52]],
            'flipX': False, 'flipY': False}]

    def compare(self, observed):
        return module.compare_quads(observed, self.expected, {'width': 234, 'height': 156}, 2, 'fixture')

    def test_unavailable_geometry_and_missing_quads_fail(self):
        for field in ['xySubmitted', 'uvSubmitted']:
            observed = copy.deepcopy(self.observed)
            observed[0][field] = False
            with self.assertRaises(ValueError):
                self.compare(observed)
        with self.assertRaises(ValueError):
            self.compare([])

    def test_correct_bounds_cannot_hide_a_skewed_destination_vertex(self):
        observed = copy.deepcopy(self.observed)
        observed[0]['xy'][2][0] = 1
        with self.assertRaisesRegex(ValueError, 'destination vertex'):
            self.compare(observed)

    def test_correct_source_crop_cannot_hide_wrong_submitted_uvs(self):
        observed = copy.deepcopy(self.observed)
        observed[0]['sourcePixels'][2][0] = 1
        with self.assertRaisesRegex(ValueError, 'source vertex'):
            self.compare(observed)

    def test_coordinate_budget_tracks_float_spacing_and_rejects_wrong_pixels(self):
        self.assertLess(module.coordinate_bound(52, 512), 0.00001)
        self.assertGreater(module.coordinate_bound(390, 512), 0.00003072)
        self.assertEqual(module.coordinate_bound(0, 512), 0)
        with self.assertRaises(ValueError):
            module.close([52.001], [52], 'source', (512,))

    def test_failure_does_not_skip_later_requested_cases(self):
        with tempfile.TemporaryDirectory() as directory:
            paths = [Path(directory) / f'{index}.json' for index in range(2)]
            for index, path in enumerate(paths):
                path.write_text(str(index))
            with patch.object(module, 'verify', side_effect=[ValueError('bad first case'),
                    {'status': 'pass', 'seedValue': 2, 'decorationSeed': 3}]) as verify:
                report = module.verify_all(paths, Path('generator'))
                self.assertEqual(verify.call_count, 2)
            self.assertEqual(report['status'], 'fail')
            self.assertEqual([case['status'] for case in report['cases']], ['fail', 'pass'])

    def test_duplicate_original_identity_and_seed_reuse_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            paths = [Path(directory) / f'{index}.json' for index in range(2)]
            for path in paths:
                path.write_text('same manifest')
            result = {'status': 'pass', 'seedValue': 2, 'decorationSeed': 3}
            with patch.object(module, 'verify', return_value=result) as verify:
                report = module.verify_all(paths, Path('generator'))
                self.assertEqual(verify.call_count, 1)
            self.assertEqual(report['status'], 'fail')
            self.assertIn('Duplicate', report['cases'][1]['error'])
            paths[1].write_text('independent manifest')
            with patch.object(module, 'verify', return_value=result):
                report = module.verify_all(paths, Path('generator'))
            self.assertEqual(report['status'], 'fail')
            self.assertIn('independent run and decoration seed', report['cases'][1]['error'])

    def test_capture_timeout_cleans_child_session_and_continues_independent_cases(self):
        codec = SimpleNamespace(random_vector=Mock(side_effect=[
            {'code': 'first', 'seedValue': 1}, {'code': 'repeated', 'seedValue': 1},
            {'code': 'second', 'seedValue': 2}]))
        runner = SimpleNamespace(codec=codec, default_tape=lambda seed: [
            {'type': 'step', 'frames': 1}, {'type': 'capture', 'label': 'title'},
            {'type': 'step', 'frames': 2}, {'type': 'capture', 'label': 'gameplay-start'}])
        first = Mock(pid=12345)
        first.communicate.side_effect = [subprocess.TimeoutExpired('original', 180), ('', '')]
        second = Mock()
        second.communicate.return_value = ('controller result', '')
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(module, 'load_module', return_value=runner), \
                patch.object(module.subprocess, 'run'), \
                patch.object(module.subprocess, 'Popen', side_effect=[first, second]) as launch, \
                patch.object(module.os, 'killpg') as kill:
            output = Path(directory) / 'captures'
            paths = module.capture_originals(output, 2, 1)
            self.assertEqual(len(paths), 2)
            self.assertEqual(launch.call_count, 2)
            self.assertEqual(codec.random_vector.call_count, 3)
            kill.assert_called_once_with(first.pid, module.signal.SIGKILL)
            self.assertIn('"status": "fail"', paths[0].read_text())
            tape = (output / 'case-01-tape.json').read_text()
            self.assertIn('gameplay-start', tape)
            self.assertNotIn('title', tape)
            self.assertTrue(launch.call_args_list[0].kwargs['start_new_session'])


if __name__ == '__main__':
    unittest.main()
