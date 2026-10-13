#!/usr/bin/env python3
"""Summarize submitted public GL draw data with texture lifetimes respected.

Output is reference/test evidence only. Production must derive room decoration
from general algorithms and authored assets, never replay these captured maps.
"""
import argparse
import json
import math
from pathlib import Path


def observe(manifest):
    report=json.loads(manifest.read_text())
    trace=Path(report['graphicsTrace']['path'])
    textures={};epochs={};state=None;frames={};cached=[];snapshots=[];current_frame=None
    for line in trace.read_text().splitlines():
        item=json.loads(line)
        if item['op']=='texture':
            epochs[item['id']]=epochs.get(item['id'],0)+1
            textures[item['id']]={'size':[item['width'],item['height']],'epoch':epochs[item['id']]}
        elif item['op']=='graphicsState':state=item
        elif item['op']=='frameStart':
            if current_frame is not None:raise ValueError('Graphics frame starts before the previous frame ended')
            if item['frame'] in frames:raise ValueError('Graphics frame counter is reused')
            current_frame=item['frame'];frames[current_frame]={'label':item['label'],'draws':[]}
        elif item['op']=='frameEnd':
            if current_frame!=item['frame']:raise ValueError('Graphics frameEnd does not match active frame')
            current_frame=None
        elif item['op']=='textureSnapshot':snapshots.append(item)
        elif item['op']=='draw':
            texture=textures.get(item['texture'],{})
            size=item.get('textureSize',texture.get('size'))
            epoch=item.get('textureEpoch',texture.get('epoch'))
            if not size:raise ValueError('Observed draw references unknown texture lifetime')
            quads=[]
            for offset in range(0,len(item['vertices']),4):
                vertices=item['vertices'][offset:offset+4]
                if len(vertices)!=4:raise ValueError('Current observer requires submitted sprite quads')
                xy=[v['xy'] for v in vertices]
                xy_valid=all(v.get('ok') and all(isinstance(n,(int,float)) and math.isfinite(n) for n in v['xy']) for v in vertices)
                uv_valid=all(v.get('uvOk') and all(isinstance(n,(int,float)) and math.isfinite(n) for n in v['uv']) for v in vertices)
                color_valid=all(v.get('colorOk') and all(isinstance(n,(int,float)) and math.isfinite(n) for n in v['color']) for v in vertices)
                uv=[[v['uv'][0]*size[0],v['uv'][1]*size[1]] for v in vertices] if uv_valid else None
                rect=[min(p[0] for p in uv),min(p[1] for p in uv),max(p[0] for p in uv),max(p[1] for p in uv)] if uv_valid else None
                quads.append({'xy':xy,'sourcePixels':uv,'sourceRect':rect,
                              'xySubmitted':xy_valid,'uvSubmitted':uv_valid,'colorSubmitted':color_valid,
                              'flipX':uv[1][0]<uv[0][0] if uv_valid else None,'flipY':uv[2][1]<uv[0][1] if uv_valid else None,
                              'color':[v['color'] for v in vertices]})
            draw={'texture':item['texture'],'epoch':epoch,'textureSize':size,
                  'program':item.get('program'),'renderTargetTexture':item.get('renderTargetTexture'),
                  'count':item['count'],'mode':item['mode'],'indices':item.get('indices'),
                  'indexType':item.get('indexType'),'quads':quads,'graphicsState':state}
            if item.get('cacheDraw'):cached.append(draw)
            # Cache draws can occur after a captured frame and retain its last
            # counter value. Their textures remain evidence, but they did not
            # produce the paired screenshot and must not enter that frame.
            if current_frame is not None and item.get('frame')==current_frame:frames[current_frame]['draws'].append(draw)
    if current_frame is not None:raise ValueError('Graphics trace ends with an incomplete captured frame')
    floors=[draw for draw in cached if draw['textureSize']==[512,512] and draw['count']==24 and len(draw['quads'])==4
            and draw['quads'][0]['xy'][0]==[52,52] and all(q['xySubmitted'] and q['uvSubmitted'] for q in draw['quads'])]
    walls=[draw for draw in cached if draw['textureSize']==[512,512] and draw['count']==264 and len(draw['quads'])==44
           and all(q['xySubmitted'] and q['uvSubmitted'] for q in draw['quads'])]
    for draw in floors:
        for quad in draw['quads']:
            x,y=quad['sourceRect'][:2]
            # Six authored floor patches in the padded468x468 Basement sheet.
            quad['atlasPatchColumn']=round((x-52)/234)
            quad['atlasPatchRow']=round((y-52)/156)
            quad['atlasPatchIndex']=quad['atlasPatchRow']*2+quad['atlasPatchColumn']
    return {'schema':1,'sourceManifest':str(manifest),'seed':report['seed'],
            'initialState':report.get('initialState',report.get('observedState',{})),
            'scope':'submitted GL geometry/colors/uniforms and graphics pixels; not gameplay internals/ticks',
            'floorAssemblies':floors,'wallAssemblies':walls,'frames':list(frames.values()),'snapshots':snapshots}


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('manifest',type=Path)
    args=parser.parse_args();result=observe(args.manifest)
    output=args.manifest.with_name('graphics-observations.json');output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'path':str(output),'seed':result['seed'],'floors':[[q['atlasPatchIndex'] for q in d['quads']] for d in result['floorAssemblies']],
                      'wallAssemblies':len(result['wallAssemblies']),'frameLabels':[f['label'] for f in result['frames']]}))


if __name__=='__main__':main()
