#!/usr/bin/env python3
"""Integrity checks for portable/native supplied-seed layout verification."""
import copy
import importlib.util
import os
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('source_layout_verify',
    Path(__file__).with_name('foundation-layout-verify.py'))
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


class IntegrityChecks(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalogs, cls.authored = verify.authored_catalogs()
        cls.cases = verify.portable_cases(verify.REFERENCES / 'native-observations.json')

    def test_every_retained_source_case_validates_full_grid_template_and_rng_chain(self):
        self.assertEqual(len(self.cases), 8)
        for case in self.cases:
            verify.validate_case(case, self.catalogs)

    def test_original_initial_log_phase_is_preserved_after_later_real_traversal(self):
        for case in self.cases[:2]:
            self.assertEqual(verify.starting_log_state(case['originalLogText']),
                             case['observation']['actualPublicLog'])
            self.assertNotEqual(verify.debug.observed_state(case['originalLogText'])['room'],
                                case['actualLog']['room'])

    def test_invalid_native_input_pairs_and_incomplete_tables_are_rejected(self):
        changes = [
            lambda case: case['provenance'].update(memoryWritten=True),
            lambda case: case['observation'].update(nativeBinarySha256='0' * 64),
            lambda case: case['provenance']['originalCheckpoint']['context'].update(gateBlocked=False),
            lambda case: case['provenance']['originalCheckpoint']['context'].update(presentedFrame=999),
            lambda case: case['provenance']['originalCheckpoint']['context'].update(virtualElapsedNs=1),
            lambda case: case['provenance']['originalCheckpoint']['context'].update(seed='ZZZZZZZZ'),
            lambda case: case['growthInput'].update(generatorSeed=case['growthInput']['generatorSeed'] ^ 1),
            lambda case: case['observation']['grid'].pop(),
            lambda case: case['observation']['grid'].__setitem__(84, True),
            lambda case: case['observation']['descriptors'][0].update(index=1),
            lambda case: case['observation']['descriptors'][0].update(spawnSeed=123),
            lambda case: case['observation']['descriptors'][0].update(doorMask=256),
            lambda case: case['observation']['descriptors'][0]['template'].update(variant=0xffffffff),
            lambda case: case['observation']['descriptors'][0]['cells'].pop(),
            lambda case: case['observation']['descriptors'][0]['doorGridIndices'].pop(),
            lambda case: case['controller']['presentations'][3]['consumedX11Events'][0].update(presentedFramePhase=1),
            lambda case: case['controller']['presentations'].pop(0),
            lambda case: case['controller']['presentations'].__delitem__(slice(0, 2)),
            lambda case: case['controller']['tape'].append({'type': 'capture', 'label': 'gameplay-start'}),
        ]
        for change in changes:
            case = copy.deepcopy(self.cases[0])
            change(case)
            with self.assertRaises((ValueError, KeyError, TypeError)):
                verify.validate_case(case, self.catalogs)

    def test_duplicate_source_identities_fail_without_suppressing_later_cases(self):
        with patch.object(verify, 'verify_case', return_value={'status': 'pass'}) as checker:
            report = verify.verify_all([self.cases[0], self.cases[0], self.cases[1]],
                Path('/unused'), self.catalogs, 'templates')
        self.assertEqual(report['status'], 'fail')
        self.assertEqual(len(report['cases']), 3)
        self.assertEqual(checker.call_count, 2)
        self.assertIn('Duplicate', report['cases'][1]['error'])
        self.assertEqual(report['cases'][2]['status'], 'pass')

    def test_failed_selected_case_does_not_stop_independent_verification(self):
        with patch.object(verify, 'verify_case', side_effect=[ValueError('first failure'),
                {'status': 'pass'}, {'status': 'pass'}]) as checker:
            report = verify.verify_all(self.cases[:3], Path('/unused'), self.catalogs, 'templates')
        self.assertEqual(report['status'], 'fail')
        self.assertEqual(checker.call_count, 3)
        self.assertEqual(len(report['cases']), 3)

    def test_source_observer_rejects_nonchild_without_stopping_process(self):
        result = verify.observer.observe(os.getpid(), {'label': 'gameplay-start', 'gateBlocked': False})
        self.assertEqual(result['status'], 'fail')
        self.assertFalse(result['memoryWritten'])
        self.assertIn('direct original child', result['error'])
        self.assertEqual(verify.observer.observe(os.getpid(), {'label': 'title'})['status'], 'skip')


if __name__ == '__main__':
    unittest.main()
