#!/usr/bin/env python3
"""Generate the runtime audio catalogue from pinned base XML and audio files."""
import argparse
import hashlib
import json
from pathlib import Path
import wave
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
RESOURCES = ROOT / 'references/isaac/rebirth/resources'
OUTPUT = ROOT / 'web/audio-data.json'


def generate():
    sources = {}
    assets = {}
    sounds = []
    music = []
    for filename in ['sounds.xml', 'music.xml']:
        source = RESOURCES / filename
        sources[str(source.relative_to(ROOT))] = hashlib.sha256(source.read_bytes()).hexdigest()
        document = ET.parse(source).getroot()
        seen = set()
        for row in document:
            identifier = int(row.get('id'))
            if identifier in seen:
                raise ValueError('Duplicate authored audio ID')
            seen.add(identifier)
            def resource(value):
                path = (document.get('root') + value).replace('\\', '/').removeprefix('resources/').lower()
                if '..' in path.split('/'):
                    raise ValueError('Invalid authored audio path')
                if path not in assets:
                    file = RESOURCES / path
                    assets[path] = {'bytes': file.stat().st_size, 'sha256': hashlib.sha256(file.read_bytes()).hexdigest()}
                    if path.endswith('.wav'):
                        with wave.open(str(file)) as sample:
                            assets[path].update({'sampleRate': sample.getframerate(), 'channels': sample.getnchannels(),
                                                'sampleBytes': sample.getsampwidth(), 'frames': sample.getnframes()})
                return path
            if filename == 'sounds.xml':
                samples = [{'path': resource(sample.get('path')), 'weight': int(sample.get('weight'))} for sample in row.findall('sample')]
                if not 0 <= identifier <= 323 or not 1 <= len(samples) <= 5 or any(sample['weight'] != 1 for sample in samples):
                    raise ValueError('Authored sound exceeds the recovered base catalogue contract')
                sounds.append({'id': identifier, 'samples': samples})
            else:
                track = {'id': identifier, 'name': row.get('name'), 'loop': row.get('loop') == 'true'}
                for field in ['path', 'intro', 'layer', 'layerintro']:
                    if row.get(field):
                        track[field] = resource(row.get(field))
                music.append(track)
    return {'schema': 1, 'sources': sources, 'sounds': sounds, 'music': music, 'assets': dict(sorted(assets.items()))}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    contents = json.dumps(generate(), indent=2) + '\n'
    if args.check:
        if OUTPUT.read_text() != contents:
            raise SystemExit('Runtime audio catalogue is stale; run python3 tools/foundation/audio-catalog.py')
    else:
        OUTPUT.write_text(contents)
    print(json.dumps({'status': 'pass', 'catalogue': str(OUTPUT.relative_to(ROOT))}))
