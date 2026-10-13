#!/usr/bin/env python3
"""Contract checks for complete graphics frames and absent client attributes."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('observe',Path(__file__).with_name('foundation-gl-observe.py'))
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)


def draw(cache=False, valid=True):
    vertices=[{'id':n,'ok':True,'xy':[n%2,n//2],
               'uv':[n%2,n//2] if valid else [None,None],
               'uvOk':valid,'color':[1,1,1,1],'colorOk':True} for n in range(4)]
    return {'op':'draw','frame':1,'texture':1,'textureSize':[512,512],
            'count':6,'mode':4,'vertices':vertices,'cacheDraw':cache}


class GraphicsFrameContract(unittest.TestCase):
    def observe(self, events):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);trace=root/'commands.jsonl';manifest=root/'manifest.json'
            trace.write_text(''.join(json.dumps(event)+'\n' for event in events))
            manifest.write_text(json.dumps({'seed':'fixture','observedState':{'stage':1},
                'graphicsTrace':{'path':str(trace)}}))
            return module.observe(manifest)

    def test_cache_after_end_is_not_in_paired_frame(self):
        result=self.observe([{'op':'frameStart','frame':1,'label':'main'},draw(),
            {'op':'frameEnd','frame':1},draw(cache=True)])
        self.assertEqual(len(result['frames'][0]['draws']),1)

    def test_cache_inside_frame_remains_in_actual_submission(self):
        result=self.observe([{'op':'frameStart','frame':1,'label':'main'},draw(cache=True),
            {'op':'frameEnd','frame':1}])
        self.assertEqual(len(result['frames'][0]['draws']),1)

    def test_disabled_or_nonfinite_uv_does_not_become_a_zero_patch(self):
        result=self.observe([{'op':'frameStart','frame':1,'label':'main'},draw(valid=False),
            {'op':'frameEnd','frame':1}])
        quad=result['frames'][0]['draws'][0]['quads'][0]
        self.assertFalse(quad['uvSubmitted'])
        self.assertIsNone(quad['sourceRect'])

    def test_partial_frame_is_rejected(self):
        with self.assertRaisesRegex(ValueError,'incomplete'):
            self.observe([{'op':'frameStart','frame':1,'label':'main'},draw()])

    def test_mismatched_end_is_rejected(self):
        with self.assertRaisesRegex(ValueError,'does not match'):
            self.observe([{'op':'frameStart','frame':1,'label':'main'},draw(),{'op':'frameEnd','frame':2}])

    def test_nested_start_is_rejected(self):
        with self.assertRaisesRegex(ValueError,'before'):
            self.observe([{'op':'frameStart','frame':1,'label':'main'},{'op':'frameStart','frame':2,'label':'title'}])

    def test_reused_counter_is_rejected(self):
        with self.assertRaisesRegex(ValueError,'reused'):
            self.observe([{'op':'frameStart','frame':1,'label':'main'}, {'op':'frameEnd','frame':1},
                          {'op':'frameStart','frame':1,'label':'title'}])


if __name__=='__main__':unittest.main()
