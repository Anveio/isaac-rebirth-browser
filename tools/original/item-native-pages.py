#!/usr/bin/env python3
"""Page literal Native observation JSON before the browser process starts."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path


def identity(path):
    value = path.stat()
    return [value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns]


def digest_file(path):
    size = 0
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            size += len(chunk)
            digest.update(chunk)
    return size, digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--raw-sha256')
    parser.add_argument('--raw-bytes', type=int)
    parser.add_argument('--all-row-owners', action='store_true',
                        help='Retain every actual row label/presentation owner and selected provenance headers')
    args = parser.parse_args()
    before = identity(args.manifest)
    encoded_bytes, encoded_sha = digest_file(args.manifest)
    if args.manifest.suffix == '.gz':
        opener, encoding = gzip.open, 'gzip'
    elif args.manifest.suffix == '.json':
        opener, encoding = Path.open, 'json'
    else:
        raise ValueError('Expected an explicit original JSON or gzip manifest')
    with opener(args.manifest, 'rb') as stream:
        raw = stream.read()
    raw_bytes, raw_sha = len(raw), hashlib.sha256(raw).hexdigest()
    if args.raw_sha256 is not None and args.raw_sha256 != raw_sha:
        raise ValueError('Frozen Native raw SHA differs')
    if args.raw_bytes is not None and args.raw_bytes != raw_bytes:
        raise ValueError('Frozen Native raw length differs')
    text = raw.decode('utf8')
    del raw
    decoder = json.JSONDecoder(parse_constant=lambda value: (_ for _ in ()).throw(ValueError('Non-JSON number: ' + value)))
    length = len(text)

    def space(position):
        while position < length and text[position] in ' \t\r\n':
            position += 1
        return position

    def token(position, expected):
        position = space(position)
        if position >= length or text[position] != expected:
            raise ValueError('Expected JSON token ' + expected)
        return position + 1

    args.output.mkdir(parents=True, exist_ok=False)
    pack_path = args.output / 'observations.json.gz.pages'
    index = {'schema': 1, 'scope': 'Derived literal JSON row pages; original manifest remains canonical',
             'source': {'path': str(args.manifest), 'encoding': encoding, 'physical': before,
                        'archiveBytes': encoded_bytes, 'archiveSha256': encoded_sha,
                        'rawBytes': raw_bytes, 'rawSha256': raw_sha},
             'frames': {}, 'baseline': None, 'observations': 0}
    if args.all_row_owners:
        index['rows'] = []
        index['provenance'] = {}
    fields = set()
    position = token(0, '{')
    with pack_path.open('xb') as pack:
        while True:
            position = space(position)
            if text[position] == '}':
                position += 1
                break
            key, position = decoder.raw_decode(text, position)
            if not isinstance(key, str) or key in fields:
                raise ValueError('Duplicate or non-string Native top-level key')
            fields.add(key)
            position = token(position, ':')
            if key == 'debugObservations':
                position = token(position, '[')
                while True:
                    position = space(position)
                    if text[position] == ']':
                        position += 1
                        break
                    start = position
                    row, position = decoder.raw_decode(text, start)
                    # Keep literal numeric tokens, including -0, and every field.
                    literal = text[start:position].encode('utf8')
                    frame = row['context']['presentedFrame']
                    if type(frame) is not int or not 0 <= frame <= 9007199254740991:
                        raise ValueError('Invalid Native presentation identity')
                    compressed = gzip.compress(literal, compresslevel=1, mtime=0)
                    entry = {'offset': pack.tell(), 'bytes': len(compressed), 'rawBytes': len(literal),
                             'rawSha256': hashlib.sha256(literal).hexdigest()}
                    if pack.write(compressed) != len(compressed):
                        raise OSError('Short Native page write')
                    if args.all_row_owners:
                        label = row['context']['label']
                        if not isinstance(label, str):
                            raise ValueError('Invalid Native observation label')
                        index['rows'].append({'label': label, 'presentedFrame': frame, 'entry': entry})
                    index['observations'] += 1
                    index['frames'][str(frame)] = entry  # Same last-wins Map law.
                    if index['baseline'] is None and row['context']['label'] == 'gameplay-start':
                        index['baseline'] = entry  # Same first matching baseline law.
                    del row, literal, compressed
                    position = space(position)
                    if text[position] == ']':
                        position += 1
                        break
                    position = token(position, ',')
                    if text[space(position)] == ']':
                        raise ValueError('Trailing Native observation comma')
            else:
                value, position = decoder.raw_decode(text, space(position))
                if key in ('status', 'failures'):
                    index[key] = value
                if args.all_row_owners and key in ('bootEpochSeed', 'seedMode'):
                    index['provenance'][key] = value
                del value
            position = space(position)
            if text[position] == '}':
                position += 1
                break
            position = token(position, ',')
            if text[space(position)] == '}':
                raise ValueError('Trailing Native object comma')
    if space(position) != length:
        raise ValueError('Trailing Native JSON input')
    if index.get('status') != 'pass' or index.get('failures') != [] or index['baseline'] is None:
        raise ValueError('Incomplete or failing original capture')
    if identity(args.manifest) != before or digest_file(args.manifest) != (encoded_bytes, encoded_sha):
        raise ValueError('Native source changed while paging')
    index['packBytes'], index['packSha256'] = digest_file(pack_path)
    index['packPath'] = str(pack_path)
    index_path = args.output / 'index.json'
    with index_path.open('x') as stream:
        json.dump(index, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'indexPath': str(index_path), 'indexSha256': digest_file(index_path)[1]}))


if __name__ == '__main__':
    main()
