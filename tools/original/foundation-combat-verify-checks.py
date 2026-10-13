#!/usr/bin/env python3
"""Integrity checks for original combat evidence and diagnostic admission."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import struct
import subprocess
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


verify = load('combat_source_verify', Path(__file__).with_name('foundation-combat-verify.py'))
observer = load('combat_source_observe', Path(__file__).with_name('foundation-combat-observe.py'))
scenario = load('combat_source_scenario', Path(__file__).with_name('foundation-combat-scenario.py'))


class IntegrityChecks(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.document = json.loads((verify.REFERENCES / 'fixtures.json').read_text())
        archive = json.loads((verify.REFERENCES / 'native-observations.json').read_text())
        cls.proofs = {entry['name']: entry for entry in archive['cases']}
        cls.kernel = ROOT / 'engine/target/debug/combat'

    def test_every_retained_case_pairs_original_fields_input_edges_and_seed(self):
        self.assertEqual(len(self.document['cases']), 8)
        for case in self.document['cases']:
            verify.validate_case(case, self.proofs[case['name']])
        self.assertEqual(len(scenario.cases()), 8)

    def test_proof_integrity_and_unsafe_or_unhashed_observations_are_rejected(self):
        case = self.document['cases'][2]
        changes = [
            lambda proof: proof.update(seedValue=0),
            lambda proof: proof['profile'].update(fresh=False),
            lambda proof: proof['captures'][0]['context'].update(gateBlocked=False),
            lambda proof: proof['captures'][0]['observation'].update(memoryWritten=True),
            lambda proof: proof['captures'][0]['observation'].update(executableMappingsRead=True),
            lambda proof: proof['captures'][0]['observation'].update(nativeBinarySha256='0' * 64),
            lambda proof: proof['captures'].pop(),
        ]
        for change in changes:
            proof = copy.deepcopy(self.proofs[case['name']])
            change(proof)
            with self.assertRaises((ValueError, KeyError)):
                verify.validate_case(case, proof, False)
        proof = copy.deepcopy(self.proofs[case['name']])
        proof['nativeManifestSha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'integrity'):
            verify.validate_case(case, proof)

    def test_changed_native_value_or_public_control_cannot_be_relabelled_as_evidence(self):
        case = copy.deepcopy(self.document['cases'][2])
        proof = self.proofs[case['name']]
        case['observations'][0]['velocity']['y'] = 0.0
        with self.assertRaisesRegex(ValueError, 'source.trajectory'):
            verify.validate_case(case, proof)
        case = copy.deepcopy(self.document['cases'][2])
        case['request']['inputs'][0]['x'] = 1
        with self.assertRaisesRegex(ValueError, 'request.inputs'):
            verify.validate_case(case, proof)

    def test_f32_comparison_rejects_one_ulp_and_nonfinite_values(self):
        native = self.document['cases'][2]['observations'][0]['velocity']['y']
        bits, = struct.unpack('<I', struct.pack('<f', native))
        adjacent, = struct.unpack('<f', struct.pack('<I', bits + 1))
        with self.assertRaisesRegex(ValueError, 'native f32'):
            verify.compare(native, adjacent)
        with self.assertRaises(ValueError):
            verify.compare(native, float('nan'))
        with self.assertRaises(ValueError):
            verify.compare(0.0, -0.0)

    def test_missing_duplicate_and_failed_cases_do_not_suppress_later_checks(self):
        document = copy.deepcopy(self.document)
        document['cases'].insert(1, document['cases'][0])
        with patch.object(verify, 'verify_case', return_value={'status': 'pass'}) as check:
            report = verify.verify_all(document, self.proofs, Path('/unused'))
        self.assertEqual(report['status'], 'fail')
        self.assertEqual(check.call_count, 8)
        self.assertEqual(report['cases'][2]['status'], 'pass')
        document['cases'] = document['cases'][1:]
        document['cases'].pop()
        with patch.object(verify, 'verify_case', return_value={'status': 'pass'}) as check:
            report = verify.verify_all(document, self.proofs, Path('/unused'))
        self.assertEqual(report['status'], 'fail')
        self.assertTrue(report['integrityErrors'])
        self.assertEqual(check.call_count, 7)
        with patch.object(verify, 'verify_case', side_effect=[ValueError('first failure')]
                + [{'status': 'pass'}] * 7) as check:
            report = verify.verify_all(self.document, self.proofs, Path('/unused'))
        self.assertEqual(report['status'], 'fail')
        self.assertEqual(check.call_count, 8)
        self.assertEqual(report['cases'][-1]['status'], 'pass')

    def test_nonchild_observation_fails_before_any_stop_or_memory_read(self):
        with patch.object(observer.os, 'kill') as kill:
            result = observer.observe(os.getpid(), {'label': 'gameplay-start',
                'gateBlocked': False, 'publicLogState': {'stage': 1}})
        self.assertEqual(result['status'], 'fail')
        self.assertFalse(result['memoryWritten'])
        self.assertFalse(result['executableMappingsRead'])
        self.assertEqual(result['bytesRead'], 0)
        kill.assert_not_called()

    def test_cli_rejects_empty_nonfinite_and_skipped_invalid_controls(self):
        request = copy.deepcopy(self.document['cases'][2]['request'])
        request['inputs'] = []
        for field in ('position', 'velocity'):
            invalid = copy.deepcopy(request)
            invalid[field]['x'] = 1e39
            process = subprocess.run([str(self.kernel)], input=json.dumps(invalid),
                text=True, capture_output=True, timeout=10)
            self.assertNotEqual(process.returncode, 0)
            self.assertEqual(process.stdout, '')
            self.assertIn('finite', process.stderr)
        request['clock'] = {'systemCounter': 1, 'interpolationEnabled': False}
        request['inputs'] = [{'x': 2, 'y': 0}]
        process = subprocess.run([str(self.kernel)], input=json.dumps(request),
            text=True, capture_output=True, timeout=10)
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(process.stdout, '')
        self.assertIn('control components', process.stderr)


if __name__ == '__main__':
    unittest.main()
