#!/usr/bin/env python3
"""Public input tapes for paired original/browser regular-weapon scenarios."""
import argparse
import importlib.util
import json
from pathlib import Path
spec=importlib.util.spec_from_file_location('clock_capture',Path(__file__).with_name('foundation-clock-run.py'))
capture=importlib.util.module_from_spec(spec);spec.loader.exec_module(capture)
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--seed',required=True)
parser.add_argument('--direction',choices=['Right','Left','Up','Down'],required=True)
parser.add_argument('--delay',type=int,choices=[0,1],required=True)
parser.add_argument('--frames',type=int,default=54)
parser.add_argument('--release-frames',type=int,default=26)
args=parser.parse_args()
if not 1<=args.frames<=120:raise ValueError('Scenario length must be 1..120 presentations')
if not 1<=args.release_frames<=120:raise ValueError('Release length must be 1..120 presentations')
tape=capture.default_tape(args.seed)
stop=next(i for i,action in enumerate(tape) if action.get('label')=='gameplay-start')
tape=tape[:stop+1]
if args.delay:tape.append({'type':'step','frames':1})
tape += [{'type':'key','key':args.direction,'down':True}] + [{'type':'step','frames':1} for _ in range(args.frames)]
tape += [{'type':'key','key':args.direction,'down':False}] + [{'type':'step','frames':1} for _ in range(args.release_frames)]
tape += [{'type':'capture','label':'weapons-end'}]
print(json.dumps(tape))
