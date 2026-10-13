#!/usr/bin/env python3
"""General public Rebirth seed encoding and observed initial-stage RNG model.

Public seed format: SolarPolarMan's published generator, mirrored at
https://gist.github.com/ChildishGiant/bf39d195dbdc9093417bc8f5b85b57e8
Start RNG shifts (3,23,25), right/left/right, are publicly documented;
Stage1=secondNext matches original log evidence. Other stream routing remains
unverified and must not be inferred from a known-seed lookup table.
"""
import argparse
import json
import secrets

ALPHABET = 'ABCDEFGHJKLMNPQRSTWXYZ01234V6789'
SEED_XOR = 0x0FEF7FFD


def checksum(seed):
    value = 0
    while True:
        value = (value + (seed & 255)) & 255
        value = ((value << 1) + (value >> 7)) & 255
        seed >>= 5
        if seed == 0:
            return value


def encode_seed(seed):
    if not isinstance(seed, int) or isinstance(seed, bool) or not 1 <= seed <= 0xffffffff:
        raise ValueError('Original run seed must be a nonzero u32')
    packed = ((seed ^ SEED_XOR) << 8) | checksum(seed)
    return ''.join(ALPHABET[(packed >> shift) & 31] for shift in range(35, -1, -5))


def decode_seed(code):
    compact = ''.join(code.split()).upper() if isinstance(code, str) else ''
    if len(compact) != 8 or any(character not in ALPHABET for character in compact):
        raise ValueError('Seed must have eight original-alphabet characters')
    packed = 0
    for character in compact:
        packed = (packed << 5) | ALPHABET.index(character)
    seed = (packed >> 8) ^ SEED_XOR
    if seed == 0 or packed & 255 != checksum(seed):
        raise ValueError('Invalid original seed checksum or reserved zero seed')
    return seed


def start_rng_next(seed):
    # Public xorshift unsigned32 shifts: right, left, right.
    seed ^= seed >> 3
    seed ^= (seed << 23) & 0xffffffff
    seed ^= seed >> 25
    return seed & 0xffffffff


def initial_stage_seed(seed):
    return start_rng_next(start_rng_next(seed))


def random_vector():
    seed = secrets.randbelow(0xffffffff) + 1
    code = encode_seed(seed)
    return {'code': code[:4]+' '+code[4:], 'seedValue': seed,
            'expectedStage1Seed': initial_stage_seed(seed)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=4)
    args = parser.parse_args()
    if not 1 <= args.count <= 1000:
        raise SystemExit('Count must be1..1000')
    print(json.dumps({'schema': 1, 'entropy': 'OS secrets', 'vectors': [random_vector() for _ in range(args.count)]}, indent=2))


if __name__ == '__main__':
    main()
