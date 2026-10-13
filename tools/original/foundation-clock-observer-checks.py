#!/usr/bin/env python3
"""Guard the optional observer's explicit pass/skip/error contract."""
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest

spec = importlib.util.spec_from_file_location('clock_run', Path(__file__).with_name('foundation-clock-run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class ObserverContract(unittest.TestCase):
    def observe(self, value):
        return runner.checked_observation(SimpleNamespace(observe=lambda *_: value), 123, {'label': 'intro'})

    def test_pass_and_deliberate_out_of_scope_skip_survive(self):
        for status in ('pass', 'skip'):
            value = {'status': status, 'scope': 'bounded original fields'}
            self.assertIs(self.observe(value), value)

    def test_missing_failed_and_unrecognized_observations_fail(self):
        for value in ({}, {'status': 'fail'}, {'status': 'running'}, [], None):
            with self.assertRaises(RuntimeError):
                self.observe(value)

    def test_nonfinite_observations_cannot_enter_portable_evidence(self):
        with self.assertRaises(ValueError):
            self.observe({'status': 'pass', 'height': float('nan')})


if __name__ == '__main__':
    unittest.main()
