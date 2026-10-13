#!/usr/bin/env python3
"""Public codec regression/property checks; fixed native vectors are test data."""
import importlib.util
from pathlib import Path
import secrets
import unittest

spec = importlib.util.spec_from_file_location('codec', Path(__file__).with_name('foundation-seeds.py'))
codec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(codec)


class PublicSeedCodec(unittest.TestCase):
    def test_native_observed_vectors(self):
        for value, code in ((2184569263, 'T1PK6Y0W'), (588143529, 'FXSWJZBM')):
            self.assertEqual(codec.encode_seed(value), code)
            self.assertEqual(codec.decode_seed(code), value)

    def test_u32_boundaries(self):
        for value in (1, 2, 0x7fffffff, 0x80000000, 0xfffffffe, 0xffffffff):
            self.assertEqual(codec.decode_seed(codec.encode_seed(value)), value)

    def test_new_random_roundtrips(self):
        for _ in range(2048):
            value = secrets.randbelow(0xffffffff)+1
            code = codec.encode_seed(value)
            with self.subTest(value=value, code=code):
                self.assertEqual(codec.decode_seed(code), value)
                self.assertEqual(len(code), 8)
                self.assertTrue(set(code).issubset(codec.ALPHABET))

    def test_case_and_spacing(self):
        self.assertEqual(codec.decode_seed('t1pk 6y0w'), 2184569263)

    def test_invalid_integer_inputs(self):
        for value in (0, -1, 1 << 32, True, 1.0, None):
            with self.subTest(value=value), self.assertRaises(ValueError):
                codec.encode_seed(value)

    def test_invalid_alphabet_length_checksum(self):
        for code in ('', 'ABCDEFG', 'ABCDEFGHI', 'IIIIIIII', 'OOOOOOOO', 'UUUUUUUU', '55555555', 'T1PK6Y0A', None):
            with self.subTest(code=code), self.assertRaises(ValueError):
                codec.decode_seed(code)

    def test_initial_stage_model_observed_vectors(self):
        self.assertEqual(codec.initial_stage_seed(2184569263), 1427977769)
        self.assertEqual(codec.initial_stage_seed(588143529), 868377342)


if __name__ == '__main__':
    unittest.main()
