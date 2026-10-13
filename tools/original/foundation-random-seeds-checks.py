#!/usr/bin/env python3
"""Regression cases for empty/duplicate native seed banks and failure continuation."""
import importlib.util
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('native_random',Path(__file__).with_name('foundation-random-seeds.py'))
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)


class RandomBankContracts(unittest.TestCase):
    def bank(self):
        return {'schema':1,'vectors':[module.codec.random_vector(),module.codec.random_vector()]}

    def test_empty_bank_fails(self):
        with self.assertRaises(ValueError):module.validate_bank({'schema':1,'vectors':[]},0)

    def test_requested_count_must_match(self):
        with self.assertRaises(ValueError):module.validate_bank(self.bank(),3)

    def test_duplicate_bank_fails(self):
        bank=self.bank();bank['vectors'][1]=bank['vectors'][0]
        with self.assertRaises(ValueError):module.validate_bank(bank,2)

    def test_nonempty_unique_bank_passes(self):
        self.assertEqual(len(module.validate_bank(self.bank(),2)),2)

    def test_malformed_vector_fails(self):
        for bad in (None,{}, {'code':'T1PK6Y0A','seedValue':1}):
            with self.subTest(bad=bad),self.assertRaises(ValueError):module.validate_bank({'schema':1,'vectors':[bad]},1)

    def test_failure_becomes_case_and_following_probe_runs(self):
        bank=self.bank();seen=[]
        def probe(pair):
            index,_=pair;seen.append(index)
            if index==0:raise ValueError('malformed native JSON')
            return {'status':'pass'}
        results=[module.safe_probe(probe,pair) for pair in enumerate(bank['vectors'])]
        self.assertEqual(seen,[0,1]);self.assertEqual([result['status'] for result in results],['fail','pass'])


if __name__=='__main__':unittest.main()
