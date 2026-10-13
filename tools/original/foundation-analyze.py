#!/usr/bin/env python3
"""Measure public original pixels/video; observed frames are not simulation ticks."""
import argparse
import json
from pathlib import Path
import subprocess
import numpy as np
from PIL import Image, ImageDraw


def actor_pixels(image):
    rgb = np.asarray(image.convert('RGB')).astype(np.int16)
    roi = rgb[120:450, 140:820]
    skin = (roi.min(axis=2) > 140) & ((roi.max(axis=2) - roi.min(axis=2)) < 40)
    points = set(zip(*np.nonzero(skin)))
    largest = []
    while points:
        first = points.pop()
        pending, component = [first], [first]
        while pending:
            y, x = pending.pop()
            for point in ((y-1, x), (y+1, x), (y, x-1), (y, x+1)):
                if point in points:
                    points.remove(point)
                    pending.append(point)
                    component.append(point)
        if len(component) > len(largest):
            largest = component
    if len(largest) < 200:
        return {'identified': False}
    head_y, head_x = np.array(largest).mean(axis=0)
    y, x = np.nonzero(skin)
    nearby = (np.abs(x-head_x) < 32) & (y > head_y-35) & (y < head_y+55)
    x, y = x[nearby] + 140, y[nearby] + 120
    hx, hy = int(head_x+140), int(head_y+120)
    shadow_points = []
    for sy in range(max(0, hy+20), min(image.height, hy+58)):
        background = (int(rgb[sy, max(0, hx-44), 0])+int(rgb[sy, min(image.width-1, hx+44), 0]))/2
        for sx in range(max(0, hx-36), min(image.width, hx+36)):
            if 20 < rgb[sy, sx, 0] < background*.87:
                shadow_points.append((sx, sy))
    body = y > hy+28
    body_x = x[body] if int(body.sum()) >= 20 else x
    foot = [float((body_x.min()+body_x.max()+1)/2), float(y.max()+3)]
    shadow_bounds = None
    if len(shadow_points) >= 60:
        shadow_x, shadow_y = zip(*shadow_points)
        shadow_bounds = [min(shadow_x), min(shadow_y), max(shadow_x)+1, max(shadow_y)+1]
        # Shadow mask is diagnostic only: body/lighting can split its ellipse.
        # The body skin's horizontal midpoint is more stable across head poses.
    return {'identified': True, 'skinCentroidRender': [float(x.mean()), float(y.mean())],
            'skinBoundsRender': [int(x.min()), int(y.min()), int(x.max()) + 1, int(y.max()) + 1],
            'estimatedFootAnchorRender': foot,
            'estimatedFootAnchorLogical': [value/2 for value in foot],
            'shadowBoundsRender': shadow_bounds,
            'scope': 'tutorial-room body-skin midpoint/skin-foot bounds; rendered estimate, head/body animation can shift it, not gameplay memory'}


def projectiles(image, actor):
    rgb = np.asarray(image.convert('RGB')).astype(np.int16)
    mask = (rgb[:, :, 2] - rgb[:, :, 0] > 25) & (rgb[:, :, 1] - rgb[:, :, 0] > 15) & (rgb[:, :, 2] > 100)
    mask[:120] = False
    mask[450:] = False
    mask[:, :140] = False
    mask[:, 820:] = False
    if actor['identified']:
        cx, cy = actor['skinCentroidRender']
        mask[max(0, int(cy)-45):int(cy)+50, max(0, int(cx)-40):int(cx)+40] = False
    points = set(zip(*np.nonzero(mask)))
    blobs = []
    while points:
        first = points.pop()
        pending, blob = [first], [first]
        while pending:
            y, x = pending.pop()
            for p in ((y-1, x), (y+1, x), (y, x-1), (y, x+1)):
                if p in points:
                    points.remove(p)
                    pending.append(p)
                    blob.append(p)
        if len(blob) < 20:
            continue
        y, x = zip(*blob)
        width, height = max(x)-min(x)+1, max(y)-min(y)+1
        if max(width, height) <= 30 and .4 <= width/height <= 2.5:
            blobs.append({'centroidRender': [sum(x)/len(x), sum(y)/len(y)],
                          'boundsRender': [int(min(x)), int(min(y)), int(max(x)+1), int(max(y)+1)],
                          'pixels': len(blob)})
    return blobs


def diff(first, second):
    a, b = np.asarray(first.convert('RGB')).astype(np.int16), np.asarray(second.convert('RGB')).astype(np.int16)
    d = np.abs(a-b)
    return {'changedPixels': int((d.max(axis=2) > 0).sum()), 'meanAbsoluteChannelError': float(d.mean()),
            'maxChannelError': int(d.max()), 'pixels': int(a.shape[0]*a.shape[1]),
            'exact': bool((d == 0).all())}


def analyze(manifest):
    report = json.loads(manifest.read_text())
    rows = []
    for frame in report['frames']:
        with Image.open(frame['path']) as image:
            actor = actor_pixels(image) if frame['label'].startswith('gameplay') or frame['label'] in ('movement', 'shooting', 'after-fire') else None
            blobs = projectiles(image, actor) if actor else []
            annotation = image.convert('RGB').copy()
            draw = ImageDraw.Draw(annotation)
            draw.rectangle((0, image.height-20, image.width, image.height), fill='black')
            draw.text((4, image.height-17), f"{frame['label']} | {image.width}x{image.height} rendered | 480x270 art at 2x | simulation tick unknown", fill='white')
            if actor and actor['identified']:
                draw.rectangle(actor['skinBoundsRender'], outline='yellow', width=1)
                cx, cy = actor['estimatedFootAnchorRender']
                draw.line((cx-5,cy,cx+5,cy), fill='red', width=1)
                draw.line((cx,cy-5,cx,cy+5), fill='red', width=1)
                draw.text((max(0,cx-45), max(0,cy+12)), f"pixel anchor~({cx:.1f},{cy:.1f})", fill='white')
            for blob in blobs:
                draw.rectangle(blob['boundsRender'], outline='cyan', width=1)
            annotations = manifest.parent / 'annotations'
            annotations.mkdir(exist_ok=True)
            annotated = annotations / (frame['label']+'.png')
            annotation.save(annotated)
            rows.append({'label': frame['label'], 'path': frame['path'], 'width': image.width, 'height': image.height,
                         'actor': actor, 'projectiles': blobs, 'annotatedPath': str(annotated)})
    trace = []
    video = report.get('video')
    alignment = [0, 0]
    if video:
        timestamps = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-select_streams', 'v:0',
                                                        '-show_entries', 'frame=best_effort_timestamp_time',
                                                        '-of', 'json', video['path']], text=True))['frames']
        decoder = subprocess.Popen(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-i', video['path'],
                                    '-f', 'rawvideo', '-pix_fmt', 'rgb24', '-fps_mode', 'passthrough', 'pipe:1'], stdout=subprocess.PIPE)
        previous = None
        index = 0
        size = 960*540*3
        while True:
            data = decoder.stdout.read(size)
            if not data:
                break
            if len(data) != size:
                raise RuntimeError('Incomplete decoded frame')
            image = Image.frombytes('RGB', (960, 540), data)
            if index == 0 and not video.get('captureMatchesWindowGeometry'):
                reference = np.asarray(Image.open(next(frame['path'] for frame in report['frames']
                                                        if frame['label'] == 'gameplay-unpaused')).convert('RGB')).astype(np.int16)
                sample = np.asarray(image).astype(np.int16)
                target = reference[130:310, 140:400]
                score, alignment = min((float(np.abs(sample[130+dy:310+dy, 140+dx:400+dx]-target).mean()), [dx, dy])
                                       for dy in range(41) for dx in range(25))
            if alignment != [0, 0]:
                image = image.crop((alignment[0], alignment[1], 960+alignment[0], 540+alignment[1]))
            actor = actor_pixels(image)
            changed = data != previous
            trace.append({'videoFrame': index, 'videoMs': float(timestamps[index]['best_effort_timestamp_time'])*1000,
                          'renderChanged': changed, 'actor': actor, 'projectiles': projectiles(image, actor)})
            previous = data
            index += 1
        if decoder.wait():
            raise RuntimeError('Video decoding failed')
    return {'schema': 1, 'sourceManifest': str(manifest), 'frames': rows, 'videoFrames': trace,
            'measurementScope': 'public rendered pixels sampled at nominal60fps; native simulation ticks unknown',
            'nativeInputs': report['inputs'], 'nativeVideoStartedAtMs': (video or {}).get('startedAtMs'),
            'videoPixelAlignmentRender': alignment,
            'videoInputSynchronization': 'video startup offset unmeasured in v1 captures; use motion-change samples for approximate timing'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifest', type=Path)
    parser.add_argument('--repeat', type=Path)
    args = parser.parse_args()
    measured = analyze(args.manifest)
    if args.repeat:
        other = analyze(args.repeat)
        right = {frame['label']: frame for frame in other['frames']}
        comparisons = []
        for frame in measured['frames']:
            if frame['label'] not in right:
                raise RuntimeError('Repeat capture checkpoint missing')
            a, b = Image.open(frame['path']), Image.open(right[frame['label']]['path'])
            row = {'label': frame['label'], 'fullFrame': diff(a, b)}
            if frame['actor'] and frame['actor']['identified']:
                x, y = frame['actor']['estimatedFootAnchorRender']
                box = (int(x)-40, int(y)-70, int(x)+40, int(y)+10)
                row.update({'actorCropRender': box, 'actorCrop': diff(a.crop(box), b.crop(box))})
            comparisons.append(row)
            a.close();b.close()
        measured['nativeRepeat'] = {'manifest': str(args.repeat), 'comparisons': comparisons,
                                    'frames': other['frames'], 'videoFrames': other['videoFrames']}
    output = args.manifest.with_name('measurements.json')
    output.write_text(json.dumps(measured, indent=2)+'\n')
    print(json.dumps({'measurements': str(output), 'videoFrames': len(measured['videoFrames']),
                      'frameCount': len(measured['frames'])}))


if __name__ == '__main__':
    main()
