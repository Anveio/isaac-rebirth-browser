#!/usr/bin/env python3
"""Measure submitted Isaac/regular-tear quads in a public graphics ABI trace.

This diagnostic uses authored sprite metadata and submitted vertices. Actor
positions below are rendered anchors, not original object memory or hidden
state. It is scoped to fresh normal Isaac/tutorial-room trajectories.
"""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def measure(manifest_path):
    manifest = json.loads(manifest_path.read_text())
    metadata = json.loads((ROOT / 'web/scene-data.json').read_text())
    actor = metadata['gfx/001.000_player.anm2']
    animation = next(a for a in actor['Animations'] if a['Name'] == 'HeadDown')
    layer = next(layer for layer in animation['LayerAnimations'] if layer['LayerId'] == 4)
    head = layer['Frames'][0]
    tears = metadata['gfx/002.000_tear.anm2']
    tear_animation = next(a for a in tears['Animations'] if a['Name'] == tears['DefaultAnimation'])
    tear = next(layer for layer in tear_animation['LayerAnimations'] if layer['Frames'])['Frames'][0]
    start = next(frame['presentedFrame'] for frame in manifest['frames'] if frame['label'] == 'gameplay-start')
    trace = Path(manifest['graphicsTrace']['path'])
    rows = {}
    for line in trace.open():
        call = json.loads(line)
        if call.get('op') != 'draw' or call.get('frame', 0) < start:
            continue
        frame = call['frame']
        width, height = call['textureSize']
        vertices = call['vertices']
        row = rows.setdefault(frame, {'presentedFrame': frame, 'actorHeadQuads': [], 'regularTearQuads': [], 'shadowQuads': [], 'radialLightQuads': [], 'unusableSubmittedQuads': 0})
        if (width, height) not in ((512, 512), (256, 128), (128, 64)):
            continue
        for index in range(0, len(vertices)-3, 4):
            quad = vertices[index:index+4]
            if not all(v.get('ok') and v.get('uvOk') and all(isinstance(c, (int,float)) for c in v['xy']+v['uv']) for v in quad):
                row['unusableSubmittedQuads'] += 1
                continue
            xs, ys = [v['xy'][0] for v in quad], [v['xy'][1] for v in quad]
            us, vs = [v['uv'][0] for v in quad], [v['uv'][1] for v in quad]
            box = [min(xs), min(ys), max(xs), max(ys)]
            if width == 128 and height == 64 and abs(min(us)) < .00001 and abs(min(vs)) < .00001 and abs(max(us)*width-120) < .001 and abs(max(vs)*height-49) < .001:
                row['shadowQuads'].append({'roundedMaskCenter': [(min(xs)+max(xs))/2, (min(ys)+max(ys))/2],
                    'bounds': box, 'corners': [v['xy'] for v in quad], 'sourceAtlas': call['texture'], 'framebuffer': call['framebuffer']})
            if width == 512 and height == 512 and 299 <= max(xs)-min(xs) <= 301 and 299 <= max(ys)-min(ys) <= 301 and abs((max(us)-min(us))*width-300) < .001 and abs((max(vs)-min(vs))*height-300) < .001:
                row['radialLightQuads'].append({'roundedMaskCenter': [(min(xs)+max(xs))/2, (min(ys)+max(ys))/2],
                    'bounds': box, 'corners': [v['xy'] for v in quad], 'sourceAtlas': call['texture'], 'framebuffer': call['framebuffer']})
            if width == 512 and height == 512 and abs(max(xs)-min(xs)-head['Width']) < .001 and abs(max(ys)-min(ys)-head['Height']) < .001 and abs(min(vs)) < .00001 and abs(max(vs)*height-head['Height']) < .001:
                if 48 <= min(ys)+head['YPivot']-head['YPosition'] <= 222:
                    row['actorHeadQuads'].append({'renderedAnchor': [(min(xs)+max(xs))/2, min(ys)+head['YPivot']-head['YPosition']],
                        'bounds': box, 'corners': [v['xy'] for v in quad], 'uvBounds': [min(us), min(vs), max(us), max(vs)], 'sourceAtlas': call['texture'], 'sourceEpoch': call['textureEpoch']})
            if width == 256 and height == 128 and abs(min(us)*width-tear['XCrop']) < .001 and abs(min(vs)*height-tear['YCrop']) < .001 and abs((max(us)-min(us))*width-tear['Width']) < .001 and abs((max(vs)-min(vs))*height-tear['Height']) < .001:
                row['regularTearQuads'].append({'renderedAnchor': [min(xs)-tear['XPosition']+tear['XPivot'], min(ys)-tear['YPosition']+tear['YPivot']],
                    'bounds': box, 'corners': [v['xy'] for v in quad], 'sourceAtlas': call['texture'], 'sourceEpoch': call['textureEpoch']})
    ordered = [rows[frame] for frame in sorted(rows)]
    failures = [f"Expected one submitted Isaac head at phase {row['presentedFrame']}, observed {len(row['actorHeadQuads'])}" for row in ordered if len(row['actorHeadQuads']) != 1]
    end = manifest['presentations'][-1]['presentedFrame']
    if manifest['graphicsTrace'].get('continuous') is not True or [row['presentedFrame'] for row in ordered] != list(range(start, end+1)):
        failures.append('Continuous complete graphics evidence is required for every selected presentation phase')
    return {'schema': 1, 'status': 'fail' if failures else 'pass', 'failures': failures,
            'manifest': str(manifest_path), 'trace': str(trace), 'frames': ordered, 'inputs': manifest['inputs'],
            'scope': 'authored sprite metadata plus public submitted graphics quads; rendered anchors only',
            'simulationTicks': False, 'fullInternalState': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifest', type=Path)
    args = parser.parse_args()
    try:
        report = measure(args.manifest.resolve())
    except (OSError, ValueError, TypeError, KeyError, StopIteration) as error:
        report = {'schema': 1, 'status': 'fail', 'frames': [], 'failures': ['Graphics measurement evidence failed: '+str(error)], 'simulationTicks': False, 'fullInternalState': False}
    path = args.manifest.with_name('graphics-measurements.json')
    path.write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({'status': report['status'], 'report': str(path), 'frames': len(report['frames']), 'failures': report['failures']}))
    return 0 if report['status'] == 'pass' else 1

if __name__ == '__main__':
    raise SystemExit(main())
