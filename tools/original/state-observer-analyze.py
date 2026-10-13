#!/usr/bin/env python3
"""Validate observed reference seed relationships against public GL data.

Reports data relationships only. No original addresses/layouts enter the port.
Full internal-state parity and the upstream room-generation route stay unknown.
"""
import argparse
import json
from pathlib import Path
import struct
import xml.etree.ElementTree as ET
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]


def next_u32(seed, shifts):
    a, b, c = shifts
    seed ^= seed >> a
    seed ^= seed << b & 0xffffffff
    seed ^= seed >> c
    return seed & 0xffffffff


def float32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def fx_offsets(seed):
    draws = []
    for _ in range(3):
        seed = next_u32(seed, (5, 9, 7))
        draws.append(seed)
    offsets = [float32(float32(float32(value) * float32(2 ** -32)) * float32(100)) for value in draws[1:]]
    return draws, offsets


def selected_gl_draws(trace, frames):
    found = {}
    with trace.open() as lines:
        for line in lines:
            item = json.loads(line)
            frame = item.get('frame')
            if frame not in frames or item['op'] != 'draw' or item.get('renderTargetTexture') != 0:
                continue
            vertices = item.get('vertices', [])
            if item.get('textureSize') == [1024, 512] and len(vertices) == 4 and [v['xy'] for v in vertices] == [[0, 0], [480, 0], [0, 270], [480, 270]]:
                if not all(v.get('ok') and v.get('uvOk') for v in vertices):
                    raise ValueError('Observed foreground quad must have valid submitted positions and UVs')
                found[frame] = {'texture': item['texture'], 'epoch': item.get('textureEpoch'),
                                'sourceCrop': [vertices[0]['uv'][0] * 1024, vertices[0]['uv'][1] * 512],
                                'uv': vertices[0]['uv']}
    return found


def analyze(manifest):
    capture = json.loads(manifest.read_text())
    if capture.get('status') != 'pass' or not capture.get('debugObservations'):
        raise ValueError('Successful actual-original debug captures are required')
    observations = []
    for entry in capture['debugObservations']:
        context, observed = entry['context'], entry['observation']
        spawn = context.get('publicLogState', {}).get('spawnSeed')
        if spawn is None:
            continue
        candidates = []
        for match in observed['matches']:
            if match['u32'] != spawn or 'actual-public-log:spawnSeed' not in match['labels']:
                continue
            words = match['neighborU32']
            offset = (int(match['address'], 16) - int(match['neighborStart'], 16)) // 4
            if offset < 1 or offset + 1 >= len(words):
                continue
            before, after = words[offset - 1], words[offset + 1]
            if next_u32(before, (2, 7, 9)) == spawn and next_u32(spawn, (2, 7, 9)) == after:
                draws, offsets = fx_offsets(before)
                candidates.append({'address': match['address'], 'beforeSpawnU32': before, 'spawnSeed': spawn,
                                   'afterSpawnU32': after, 'publicShift12ChainMatches': True,
                                   'fx35Draws': draws, 'fx35Offsets': offsets, 'replacementIndex': draws[0] % 5})
        unique = {(item['beforeSpawnU32'], item['spawnSeed'], item['afterSpawnU32']) for item in candidates}
        observations.append({'label': context['label'], 'presentedFrame': context['presentedFrame'],
                             'candidates': candidates, 'uniqueSeedRelationship': len(unique) == 1,
                             'memoryCoverageComplete': observed['coverageComplete'], 'bytesRead': observed['bytesRead']})
    trace = capture.get('graphicsTrace', {}).get('path')
    graphics = selected_gl_draws(Path(trace), {item['presentedFrame'] for item in observations}) if trace else {}
    authored = (ROOT / 'references/isaac/rebirth/resources/fxlayers.xml').read_text()
    resources = ET.fromstring('<resources>' + authored + '</resources>')
    catalog = [item.attrib['path'] for item in resources.findall("./fxLayers/fx[@id='2']/gfx[@roomsize='0']")]
    for observed in observations:
        draw = graphics.get(observed['presentedFrame'])
        observed['publicGL'] = draw
        if draw and observed['label'] == 'gameplay-start' and observed['uniqueSeedRelationship']:
            candidate = observed['candidates'][0]
            expected = [91.5 + candidate['fx35Offsets'][0] + 240 * .025,
                        79.5 + candidate['fx35Offsets'][1] + 213 * .025]
            observed['neutralSpawnOffsetPrediction'] = {
                'sourceCrop': expected, 'actualSourceCrop': draw['sourceCrop'],
                'absoluteResidual': [abs(a - b) for a, b in zip(expected, draw['sourceCrop'])],
                'scope': 'float32/decimal UV rounding remains separately testable; neutral rendered anchor is a captured observation',
                'predictedAsset': catalog[candidate['replacementIndex']],
            }
            asset_path = ROOT / 'references/isaac/rebirth/resources/gfx' / catalog[candidate['replacementIndex']].lower()
            image = Image.open(asset_path).convert('RGBA')
            padded = Image.new('RGBA', (1024, 512)); padded.paste(image, (0, 0))
            raw_path = Path(trace).parent / f"texture-{draw['texture']}-1024x512.rgba"
            observed['neutralSpawnOffsetPrediction']['authoredAssetBytesMatchOriginalUpload'] = raw_path.read_bytes() == padded.tobytes()
    relationships = {tuple(item['candidates'][0][key] for key in ('beforeSpawnU32', 'spawnSeed', 'afterSpawnU32'))
                     for item in observations if item['uniqueSeedRelationship']}
    return {'schema': 1, 'sourceManifest': str(manifest), 'seed': capture['seed'], 'seedValue': capture['seedValue'],
            'status': 'pass' if observations and all(item['uniqueSeedRelationship'] for item in observations) and len(relationships) == 1 else 'fail',
            'scope': 'reference-only observed seed adjacency and public RNG relationships; no production oracle or native layout translation',
            'upstreamDecorationInitialization': 'unresolved', 'fullInternalState': False,
            'temporalSeedRelationshipStable': len(relationships) == 1, 'observations': observations}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifest', type=Path)
    args = parser.parse_args()
    result = analyze(args.manifest)
    output = args.manifest.with_name('state-observations.json')
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'path': str(output), 'seedValue': result['seedValue'],
                      'temporalSeedRelationshipStable': result['temporalSeedRelationshipStable'],
                      'observations': len(result['observations'])}))
    return 0 if result['status'] == 'pass' else 1


if __name__ == '__main__':
    raise SystemExit(main())
