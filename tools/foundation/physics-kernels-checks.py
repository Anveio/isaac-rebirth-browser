#!/usr/bin/env python3
"""Reject broken retained-original provenance before invoking a physics kernel."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('physics_kernels', Path(__file__).with_name('physics-kernels.py'))
physics = importlib.util.module_from_spec(spec)
spec.loader.exec_module(physics)


class RetainedPhysicsEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.projectiles, _ = physics.read_fixture('native-projectiles.json')
        cls.mt, _ = physics.read_fixture('native-mt.json')

    def case_and_source(self, index=0):
        case = copy.deepcopy(self.projectiles['cases'][index])
        return case, physics.load_source(case['original'])

    def mt_and_source(self):
        fixture = copy.deepcopy(self.mt)
        return fixture, physics.load_source(fixture['original'])

    def test_all_retained_original_samples_are_paired(self):
        physics.validate_projectile_fixture(self.projectiles)
        fixture, source = self.mt_and_source()
        physics.validate_mt_evidence(fixture, source)

    def test_another_binary_cannot_replace_the_observed_source_version(self):
        for digest in ('0' * 64, 'f' * 64):
            origin = copy.deepcopy(self.projectiles['cases'][0]['original'])
            origin['originalBinarySha256'] = digest
            with self.assertRaisesRegex(ValueError, 'different source binary'):
                physics.validate_origin(origin)

    def test_unhashed_source_bytes_cannot_change(self):
        case, _ = self.case_and_source()
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            capture = directory / case['original']['sourceCapture']['path']
            capture.write_bytes((physics.FIXTURES / capture.name).read_bytes() + b'\n')
            with self.assertRaisesRegex(ValueError, 'digest differs'):
                physics.load_source(case['original'], directory)

    def test_source_path_cannot_escape_the_retained_evidence_directory(self):
        case, _ = self.case_and_source()
        case['original']['sourceCapture']['path'] = '../native-projectiles.json'
        with self.assertRaisesRegex(ValueError, 'leaves its evidence directory'):
            physics.load_source(case['original'])

    def test_original_metadata_cannot_be_repaired_only_in_the_derived_fixture(self):
        case, _ = self.case_and_source()
        case['original']['manifestSha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'not paired'):
            physics.load_source(case['original'])

    def test_duplicate_capture_cannot_masquerade_as_an_independent_case(self):
        fixture = copy.deepcopy(self.projectiles)
        fixture['cases'][1] = copy.deepcopy(fixture['cases'][0])
        with self.assertRaisesRegex(ValueError, 'repeat a source origin'):
            physics.validate_projectile_fixture(fixture)

    def test_case_cannot_be_paired_with_another_original_capture(self):
        case, _ = self.case_and_source()
        _, other = self.case_and_source(1)
        with self.assertRaisesRegex(ValueError, 'capture identity differs'):
            physics.validate_projectile_case(case, other)

    def test_phase_and_birth_phase_are_observed_not_accepted_from_labels(self):
        for field in ('sample', 'birth'):
            case, source = self.case_and_source()
            if field == 'sample':
                case['samples'][2]['phase'] = 'interpolate'
            else:
                case['input']['birthPhase'] = 'interpolate'
            with self.assertRaisesRegex(ValueError, 'phase label differs'):
                physics.validate_projectile_case(case, source)

    def test_positions_and_presentation_numbers_remain_paired(self):
        for field in ('position', 'presentation'):
            case, source = self.case_and_source()
            if field == 'position':
                case['samples'][1]['position']['x'] += 1
            else:
                case['samples'][1]['presentation'] += 1
            with self.assertRaises(ValueError):
                physics.validate_projectile_case(case, source)

    def test_another_actor_cannot_be_substituted_in_a_source_transcript(self):
        case, source = self.case_and_source()
        source['samples'][2]['tear']['referenceAddress'] = '0x1234'
        with self.assertRaisesRegex(ValueError, 'entity identity'):
            physics.validate_projectile_case(case, source)

    def test_random_birth_inputs_are_bound_to_observed_consumers(self):
        for field in ('eyeUniform', 'finalFallingUniform'):
            case, source = self.case_and_source()
            case['input'][field] = 0.
            with self.assertRaisesRegex(ValueError, 'native birth consumers'):
                physics.validate_projectile_case(case, source)

    def test_vector_key_order_does_not_change_valid_evidence(self):
        case, source = self.case_and_source(3)
        self.assertTrue(source['birth']['players'])
        vector = case['input']['playerWorld']
        case['input']['playerWorld'] = {'y': vector['y'], 'x': vector['x']}
        physics.validate_projectile_case(case, source)

    def test_removing_a_source_phase_is_not_a_valid_rebaseline(self):
        case, source = self.case_and_source()
        case['samples'].pop()
        with self.assertRaisesRegex(ValueError, 'coverage or birth pairing'):
            physics.validate_projectile_case(case, source)

    def test_mt_observations_must_be_explicitly_read_only_and_successful(self):
        for patch in ({'memoryWritten': True}, {'status': 'skip'}):
            fixture, source = self.mt_and_source()
            source['checkpoints'][0]['observation'].update(patch)
            with self.assertRaisesRegex(ValueError, 'not paired'):
                physics.validate_mt_evidence(fixture, source)

    def test_mt_phase_labels_and_complete_states_remain_paired(self):
        for field in ('sourceLabel', 'presentation', 'mt'):
            fixture, source = self.mt_and_source()
            checkpoint = fixture['checkpoints'][0]
            if field == 'sourceLabel':
                checkpoint[field] = 'title84'
            elif field == 'presentation':
                checkpoint[field] += 1
            else:
                checkpoint[field]['words'][623] ^= 1
            with self.assertRaises(ValueError):
                physics.validate_mt_evidence(fixture, source)

    def test_coordinated_grid_duplicates_cannot_narrow_boundary_coverage(self):
        fixture, source = self.mt_and_source()
        checkpoint = fixture['checkpoints'][0]
        checkpoint['gridSeeds'][-1] = copy.deepcopy(checkpoint['gridSeeds'][0])
        source['checkpoints'][0]['observation']['gridSeedSamples'] = copy.deepcopy(checkpoint['gridSeeds'])
        with self.assertRaisesRegex(ValueError, 'boundary sample coverage'):
            physics.validate_mt_evidence(fixture, source)


if __name__ == '__main__':
    unittest.main()
