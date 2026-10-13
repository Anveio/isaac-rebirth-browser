#!/usr/bin/env python3
"""Base STB complete-consumption, corruption and original-resource contracts."""
import importlib.util
from pathlib import Path
import struct
import unittest

spec = importlib.util.spec_from_file_location('base_room_decoder', Path(__file__).with_name('decode-rooms.py'))
rooms = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rooms)


def fixture():
    return [{'type': 1, 'variant': 2, 'difficulty': 1, 'name': 'Start Room',
             'weight': 1.0, 'width': 13, 'height': 7,
             'doors': [{'x': -1, 'y': 3, 'enabled': True}, {'x': 6, 'y': -1, 'enabled': False}],
             'spawns': [{'x': 10, 'y': 2, 'entries': [
                 {'type': 0, 'variant': 20, 'subtype': 0, 'weight': 1.0},
                 {'type': 3000, 'variant': 9, 'subtype': 12, 'weight': 0.25}]}]}]


class RoomContracts(unittest.TestCase):
    def test_preserves_signed_door_coordinates_weighted_selectors_and_unknown_types(self):
        expected = fixture()
        raw = rooms.encode_bytes(expected)
        self.assertEqual(rooms.decode_bytes(raw), expected)
        self.assertEqual(rooms.encode_bytes(rooms.decode_bytes(raw)), raw)

    def test_rejects_every_truncated_record_boundary_and_trailing_bytes(self):
        raw = rooms.encode_bytes(fixture())
        for size in range(len(raw)):
            with self.subTest(size=size), self.assertRaisesRegex(ValueError, 'truncated STB'):
                rooms.decode_bytes(raw[:size])
        with self.assertRaisesRegex(ValueError, 'trailing STB bytes'):
            rooms.decode_bytes(raw + b'\0')

    def test_rejects_corrupt_counts_and_door_flags(self):
        with self.assertRaisesRegex(ValueError, 'room count exceeds'):
            rooms.decode_bytes(struct.pack('<I', rooms.MAX_ROOMS + 1))
        raw = bytearray(rooms.encode_bytes(fixture()))
        first_door = 4 + struct.calcsize('<IIBH') + len('Start Room') + struct.calcsize('<fBBBH')
        raw[first_door + 4] = 2
        with self.assertRaisesRegex(ValueError, 'invalid door enabled flag'):
            rooms.decode_bytes(raw)

    def test_rejects_nonfinite_and_negative_room_or_spawn_weights(self):
        raw = rooms.encode_bytes(fixture())
        room_weight = 4 + struct.calcsize('<IIBH') + len('Start Room')
        spawn_weight = len(raw) - 4
        for offset in (room_weight, spawn_weight):
            for weight in (float('nan'), float('inf'), -1.0):
                changed = bytearray(raw)
                struct.pack_into('<f', changed, offset, weight)
                with self.subTest(offset=offset, weight=weight), self.assertRaisesRegex(ValueError, 'invalid .* weight'):
                    rooms.decode_bytes(changed)

    def test_rejects_unsupported_nonascii_base_names_and_preserves_empty_sets(self):
        raw = bytearray(rooms.encode_bytes(fixture()))
        raw[4 + struct.calcsize('<IIBH')] = 255
        with self.assertRaisesRegex(ValueError, 'non-ASCII base room name'):
            rooms.decode_bytes(raw)
        self.assertEqual(rooms.decode_bytes(struct.pack('<I', 0)), [])

    def test_all_pinned_original_stage_assets_round_trip_without_lost_or_retyped_records(self):
        expected = {
            '00.special rooms': 492, '01.basement': 572, '02.cellar': 570,
            '03.caves': 528, '04.catacombs': 528, '05.depths': 506,
            '06.necropolis': 504, '07.womb': 503, '08.utero': 503,
            '09.sheol': 239, '10.cathedral': 186, '11.dark room': 95, '12.chest': 88,
        }
        assets = rooms.base_assets()
        self.assertEqual(len(assets), len(expected))
        decoded = {}
        for path, raw in assets:
            with self.subTest(source=path):
                actual = rooms.decode_bytes(raw, source=path)
                self.assertEqual(rooms.encode_bytes(actual), raw)
                self.assertEqual(len(actual), expected[Path(path).stem])
                decoded[Path(path).stem] = actual
        self.assertEqual(sum(map(len, decoded.values())), 5314)
        start = [room for room in decoded['00.special rooms'] if room['name'] == 'Start Room']
        self.assertEqual(len(start), 1)
        self.assertEqual((start[0]['type'], start[0]['variant'], start[0]['width'], start[0]['height']), (1, 2, 13, 7))
        self.assertEqual([(door['x'], door['y']) for door in start[0]['doors']], [(-1, 3), (6, 7), (13, 3), (6, -1)])
        self.assertEqual([(spawn['x'], spawn['y'], spawn['entries']) for spawn in start[0]['spawns']], [
            (10, 2, [{'type': 0, 'variant': 20, 'subtype': 0, 'weight': 1.0}]),
            (9, 4, [{'type': 0, 'variant': 20, 'subtype': 0, 'weight': 1.0}]),
        ])


if __name__ == '__main__':
    unittest.main()
