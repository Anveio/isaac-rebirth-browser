#!/usr/bin/env python3
"""Per-clip alpha masks for actors whose gameplay samples sprite texels.

GetTexel 4109d0 accepts a texel whose alpha exceeds the caller's threshold;
Creep 462b99 asks for alpha > 0.5 under the player's centre. Every clip of the
selected actor keeps one crop for its whole run (the frame tint never changes
alpha), so one mask per clip is exact. Each mask row is a hex bitfield, bit x
set when texel (x, row) has alpha > threshold; `left`/`top` place the crop in
sprite pixels relative to the actor position (layer position minus pivot).

Regenerate: python3 tools/foundation/extract-alpha-masks.py
"""
import argparse
import json
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--actor', default='gfx/1000.022_creep (red).anm2')
    parser.add_argument('--animations', default=str(ROOT / 'web/familiar-data.json'))
    parser.add_argument('--resources', default=str(ROOT / 'references/isaac/rebirth/resources'))
    parser.add_argument('--threshold', type=float, default=0.5)
    parser.add_argument('--out', default=str(ROOT / 'web/creep-mask-data.json'))
    args = parser.parse_args()
    actor = json.loads(Path(args.animations).read_text())[args.actor]
    root = Path(args.resources) / 'gfx' / actor['BaseSpritesheetPath']
    sheets = {}
    for sheet in actor['Spritesheets']:
        path = next(candidate for candidate in root.parent.rglob('*')
                    if candidate.as_posix().lower().endswith(('/' + sheet['Path']).lower()))
        sheets[sheet['Id']] = Image.open(path).convert('RGBA')
    layer_sheet = {layer['Id']: layer['SpritesheetId'] for layer in actor['Layers']}
    masks = {}
    for clip in actor['Animations']:
        (layer,) = [layer for layer in clip['LayerAnimations'] if layer['Frames']]
        crops = {(f['XCrop'], f['YCrop'], f['Width'], f['Height'], f['XPosition'] - f['XPivot'],
                  f['YPosition'] - f['YPivot'], f['XScale'], f['YScale'], f['AlphaTint']) for f in layer['Frames']}
        if len(crops) != 1:
            raise ValueError(f"{clip['Name']} changes crop, placement or alpha between frames")
        x0, y0, width, height, left, top, x_scale, y_scale, alpha_tint = crops.pop()
        if (x_scale, y_scale, alpha_tint) != (100.0, 100.0, 255.0):
            raise ValueError(f"{clip['Name']} scales or fades its layer; the mask would not be exact")
        image = sheets[layer_sheet[layer['LayerId']]]
        rows = []
        for y in range(int(height)):
            bits = 0
            for x in range(int(width)):
                if image.getpixel((int(x0) + x, int(y0) + y))[3] / 255.0 > args.threshold:
                    bits |= 1 << x
            rows.append(f'{bits:x}')
        masks[clip['Name']] = {'left': left, 'top': top, 'width': int(width), 'height': int(height), 'rows': rows}
    Path(args.out).write_text(json.dumps({'actor': args.actor, 'threshold': args.threshold, 'masks': masks},
                                         separators=(',', ':')) + '\n')


if __name__ == '__main__':
    main()
