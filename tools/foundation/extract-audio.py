#!/usr/bin/env python3
"""Complete authored base-Rebirth audio from the pinned Linux asset archives."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
RESOURCES = ROOT / 'references/isaac/rebirth/resources'
MANIFEST = ROOT / 'references/isaac/manifest.json'
LINUX_MANIFEST = ROOT / 'references/isaac/linux/manifest.json'
DECODER = ROOT / 'tools/foundation/extract-animations.py'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def authored_paths():
    result = {}
    for name, archive in [('sounds.xml', 'sfx.a'), ('music.xml', 'music.a')]:
        document = ET.parse(RESOURCES / name).getroot()
        paths = set()
        for entry in document:
            samples = entry.findall('sample') if name == 'sounds.xml' else [entry]
            fields = ['path'] if name == 'sounds.xml' else ['path', 'intro', 'layer', 'layerintro']
            for sample in samples:
                for field in fields:
                    if sample.get(field):
                        path = (document.get('root') + sample.get(field)).replace('\\', '/').lower()
                        if not path.startswith('resources/') or '..' in path.split('/'):
                            raise ValueError('Invalid authored audio path: ' + path)
                        paths.add(path)
        result[archive] = sorted(paths)
    return result


def complete(check_only=False):
    manifest = json.loads(MANIFEST.read_text())
    pinned = {item['path']: item for item in manifest['files']}
    groups = authored_paths()
    pending = {}
    for archive, paths in groups.items():
        pending[archive] = []
        for path in paths:
            destination = RESOURCES.parent / path
            relative = str(destination.relative_to(ROOT))
            pin = pinned.get(relative)
            if not destination.is_file() or pin is None:
                pending[archive].append(path)
            elif destination.stat().st_size != pin['bytes'] or digest(destination) != pin['sha256']:
                raise ValueError('Changed pinned audio asset: ' + relative)
    missing = sum(map(len, pending.values()))
    if check_only and missing:
        raise ValueError(f'{missing} authored audio assets are missing or unpinned')
    if missing:
        spec = importlib.util.spec_from_file_location('base_audio_decoder', DECODER)
        decoder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(decoder)
        linux = json.loads(LINUX_MANIFEST.read_text())
        archives = {item['path']: item for item in linux['files']}
        provenance = []
        additions = []
        (ROOT / '.cache').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='audio-extraction-', dir=ROOT / '.cache') as scratch:
            for archive, paths in pending.items():
                if not paths:
                    continue
                archive_path = 'resources/packed/' + archive
                pin = archives[archive_path]
                source = ROOT / 'references/isaac/linux/depot_250903' / archive_path
                expected = manifest['source']['base_archive_sha256'][archive]
                if pin['sha256'] != expected or source.stat().st_size != pin['bytes'] or digest(source) != expected:
                    raise ValueError('Pinned base archive mismatch: ' + archive)
                decoder.extract_named_assets(source, paths, Path(scratch))
                provenance.append({**pin, 'path': 'references/isaac/linux/depot_250903/' + archive_path})
                for path in paths:
                    contents = (Path(scratch) / path).read_bytes()
                    valid = (contents[:4] == b'RIFF' and contents[8:12] == b'WAVE') if path.endswith('.wav') else contents[:4] == b'OggS'
                    if not valid:
                        raise ValueError('Invalid decoded audio container: ' + path)
                    destination = RESOURCES.parent / path
                    if destination.exists() and destination.read_bytes() != contents:
                        raise ValueError('Existing unpinned audio differs from source: ' + path)
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    destination.write_bytes(contents)
                    additions.append({'path': str(destination.relative_to(ROOT)), 'bytes': len(contents), 'sha256': hashlib.sha256(contents).hexdigest()})
        additions.sort(key=lambda item: item['path'])
        manifest['files'].extend(additions)
        manifest['source'].setdefault('supplementary_extractions', []).append({
            'kind': 'Missing authored base audio resources', 'depot_id': linux['depot_id'], 'manifest_id': linux['manifest_id'],
            'extractor': 'tools/foundation/extract-audio.py',
            'decoder': {'path': str(DECODER.relative_to(ROOT)), 'sha256': digest(DECODER)},
            'archives': provenance, 'output_paths': [item['path'] for item in additions],
        })
        MANIFEST.write_text(json.dumps(manifest, indent=2) + '\n')
    return {'status': 'pass', 'sounds': len(groups['sfx.a']), 'music': len(groups['music.a']), 'restored': missing}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Validate complete authored audio and pinned bytes without writing')
    print(json.dumps(complete(parser.parse_args().check)))
