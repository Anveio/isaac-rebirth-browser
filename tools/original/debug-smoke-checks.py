#!/usr/bin/env python3
"""Regression checks for missing native captures; no game launch is required."""
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('native_smoke', Path(__file__).with_name('debug-smoke.py'))
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)


class CaptureContract(unittest.TestCase):
    labels = ['seed-entry', 'gameplay-start', 'pause-seed', 'before-input', 'idle-control', 'movement', 'shooting']

    def frames(self, labels=None):
        return [{'label': label} for label in self.labels if labels is None or label in labels]

    def test_complete_matching_checkpoints_pass(self):
        self.assertTrue(smoke.checkpoint_contract(self.frames(), self.frames())[0])

    def test_empty_matching_checkpoints_fail(self):
        self.assertFalse(smoke.checkpoint_contract([], [])[0])

    def test_missing_checkpoint_on_one_run_fails(self):
        self.assertFalse(smoke.checkpoint_contract(self.frames(), self.frames(self.labels[:-1]))[0])

    def test_missing_checkpoint_on_both_runs_fails(self):
        incomplete = self.frames(self.labels[:-1])
        self.assertFalse(smoke.checkpoint_contract(incomplete, incomplete)[0])

    def test_duplicate_checkpoint_labels_fail(self):
        duplicate = self.frames() + [{'label': 'shooting'}]
        self.assertFalse(smoke.checkpoint_contract(duplicate, duplicate)[0])

    def test_reordered_checkpoints_fail(self):
        self.assertFalse(smoke.checkpoint_contract(self.frames(), list(reversed(self.frames())))[0])

    def test_complete_core_key_checkpoints_pass(self):
        config = json.loads(Path(__file__).with_name('debug-config.json').read_text())
        labels = self.labels[:-2] + [item['capture'] for item in config['cases']['core-keys']]
        complete = [{'label': label} for label in labels]
        self.assertTrue(smoke.checkpoint_contract(complete, complete, 'core-keys')[0])

    def test_missing_core_key_checkpoint_on_both_runs_fails(self):
        self.assertFalse(smoke.checkpoint_contract(self.frames(), self.frames(), 'core-keys')[0])


if __name__ == '__main__':
    unittest.main()
