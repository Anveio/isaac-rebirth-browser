#!/usr/bin/env python3
"""Compare public texture filtering/blending formulas against menu captures.

All candidates are rendered from authored PNG/ANM2 assets. Reference pixels are
used only to score candidates, never as a rendering input or correction table.
"""
import argparse
from collections import Counter
import json
from pathlib import Path

import numpy as np
from PIL import Image


def read_rgba(path):
    return np.asarray(Image.open(path).convert('RGBA'), dtype=np.float64)


def bilinear(source, scale=2, premultiplied=False):
    source = source.copy()
    if premultiplied:
        source[:, :, :3] *= source[:, :, 3:4] / 255
    height, width, _ = source.shape
    xs = np.clip((np.arange(width * scale) + .5) / scale - .5, 0, width - 1)
    ys = np.clip((np.arange(height * scale) + .5) / scale - .5, 0, height - 1)
    x0, y0 = np.floor(xs).astype(int), np.floor(ys).astype(int)
    x1, y1 = np.minimum(x0 + 1, width - 1), np.minimum(y0 + 1, height - 1)
    wx, wy = (xs - x0)[None, :, None], (ys - y0)[:, None, None]
    top = source[y0[:, None], x0[None, :]] * (1 - wx) + source[y0[:, None], x1[None, :]] * wx
    bottom = source[y1[:, None], x0[None, :]] * (1 - wx) + source[y1[:, None], x1[None, :]] * wx
    return top * (1 - wy) + bottom * wy


def quantize(values, mode):
    if mode == 'floor':
        return np.floor(values)
    if mode == 'nearest-even':
        return np.rint(values)
    if mode == 'nearest-up':
        return np.floor(values + .5)
    if mode == 'none':
        return values
    raise ValueError(mode)


def source_over(destination, source, premultiplied, texture_round, framebuffer_round):
    source = quantize(source, texture_round)
    alpha = source[:, :, 3:4] / 255
    rgb = source[:, :, :3] if premultiplied else source[:, :, :3] * alpha
    return np.clip(quantize(rgb + destination * (1 - alpha), framebuffer_round), 0, 255)


def source_main(scene, assets, scale=2):
    """The observed New Run menu's static authored layers; no capture texture."""
    actor = scene['gfx/ui/main menu/gamemenu.anm2']
    animation = next(animation for animation in actor['Animations'] if animation['Name'] == 'Idle')
    destination = np.zeros((270 * scale, 480 * scale, 3), dtype=np.float64)
    for layer in animation['LayerAnimations']:
        if not layer['Visible']:
            continue
        frame = layer['Frames'][0].copy()
        if layer['LayerId'] == 2:
            frame.update(XCrop=160, YCrop=336)
        definition = next(definition for definition in actor['Layers'] if definition['Id'] == layer['LayerId'])
        sheet = next(sheet for sheet in actor['Spritesheets'] if sheet['Id'] == definition['SpritesheetId'])
        relative = (actor['BaseSpritesheetPath'] + '/' + sheet['Path']).replace('\\', '/').lower()
        if relative.startswith('resources/'):
            relative = relative[len('resources/'):]
        path = assets / relative
        image = read_rgba(path)
        sx, sy, width, height = [int(frame[key]) for key in ['XCrop', 'YCrop', 'Width', 'Height']]
        image = image[sy:sy + height, sx:sx + width]
        assert frame['Rotation'] == 0 and frame['XScale'] == 100 and frame['YScale'] == 100
        image = image.repeat(scale, axis=0).repeat(scale, axis=1)
        left = int(frame['XPosition'] - frame['XPivot']) * scale
        top = int(frame['YPosition'] - frame['YPivot']) * scale
        right, bottom = left + width * scale, top + height * scale
        x0, y0, x1, y1 = max(0, left), max(0, top), min(destination.shape[1], right), min(destination.shape[0], bottom)
        cropped = image[y0 - top:y1 - top, x0 - left:x1 - left]
        alpha = cropped[:, :, 3:4] / 255
        destination[y0:y1, x0:x1] = cropped[:, :, :3] * alpha + destination[y0:y1, x0:x1] * (1 - alpha)
    return destination


def statistics(candidate, reference):
    difference = candidate.astype(np.int16) - reference.astype(np.int16)
    return {
        'meanAbsoluteChannelError': float(np.abs(difference).mean()),
        'changedPixels': int(np.any(difference != 0, axis=2).sum()),
        'maxChannelError': int(np.abs(difference).max()),
        'channelBias': [float(difference[:, :, channel].mean()) for channel in range(3)],
        'channelResiduals': [{str(value): count for value, count in Counter(difference[:, :, channel].ravel().tolist()).most_common(12)} for channel in range(3)],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native', type=Path, default=Path('.cache/foundation/native-baseline/frames/main.png'))
    parser.add_argument('--browser', type=Path, default=Path('/tmp/foundation-menu-preview/main.png'))
    parser.add_argument('--scene', type=Path, default=Path('web/scene-data.json'))
    parser.add_argument('--assets', type=Path, default=Path('references/isaac/rebirth/resources'))
    parser.add_argument('--output', type=Path, default=Path('.cache/foundation/alpha-calibration'))
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    native, browser = read_rgba(args.native)[:, :, :3], read_rgba(args.browser)[:, :, :3]
    scene = json.loads(args.scene.read_text())
    base = source_main(scene, args.assets)
    assert base.shape == native.shape == browser.shape
    shadow = read_rgba(args.assets / 'gfx/ui/main menu/menushadow.png')[:149]
    overlay = read_rgba(args.assets / 'gfx/ui/main menu/menuoverlay.png')
    models = []
    best = None
    for premultiplied in [False, True]:
        filtered_shadow, filtered_overlay = bilinear(shadow, premultiplied=premultiplied), bilinear(overlay, premultiplied=premultiplied)
        for texture_round in ['none', 'floor', 'nearest-even', 'nearest-up']:
            for framebuffer_round in ['floor', 'nearest-even', 'nearest-up', 'none']:
                candidate = base.copy()
                count = min(filtered_shadow.shape[0], candidate.shape[0] - 244)
                candidate[244:244 + count, :512] = source_over(candidate[244:244 + count, :512], filtered_shadow[:count], premultiplied, texture_round, framebuffer_round)
                candidate = source_over(candidate, filtered_overlay, premultiplied, texture_round, framebuffer_round)
                candidate = quantize(candidate, 'nearest-up').astype(np.uint8)
                label = f'{"premultiplied" if premultiplied else "straight"}-texture-{texture_round}-framebuffer-{framebuffer_round}'
                score = statistics(candidate, native)
                record = {'model': label, 'native': score, 'browser': statistics(candidate, browser)}
                models.append(record)
                if best is None or score['meanAbsoluteChannelError'] < best[0]:
                    best = (score['meanAbsoluteChannelError'], candidate, label)
    models.sort(key=lambda model: model['native']['meanAbsoluteChannelError'])
    Image.fromarray(base.astype(np.uint8)).save(args.output / 'source-base.png')
    Image.fromarray(best[1]).save(args.output / 'best-formula.png')
    difference = np.abs(best[1].astype(np.int16) - native.astype(np.int16))
    Image.fromarray(np.minimum(difference * 32, 255).astype(np.uint8)).save(args.output / 'best-difference.png')
    report = {'schema': 1, 'scope': 'authored main-menu source layers and global public sampling/blending formulas; no captured-image renderer corrections',
              'native': str(args.native), 'browser': str(args.browser), 'currentBrowser': statistics(browser, native), 'models': models}
    (args.output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'best': models[:4], 'currentBrowserMAE': report['currentBrowser']['meanAbsoluteChannelError'], 'output': str(args.output)}))


if __name__ == '__main__':
    main()
