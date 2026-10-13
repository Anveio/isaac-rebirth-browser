#!/usr/bin/env python3
"""Recorded spike route and unfiltered fresh public-seed death interactions."""
import argparse,importlib.util,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('death_paired_scenarios',ROOT/'tools/original/foundation-paired-scenarios.py')
paired=importlib.util.module_from_spec(spec);spec.loader.exec_module(paired)
def key(name,down):return {'type':'key','key':name,'down':down}
def dense(count,prefix):return [action for n in range(count) for action in ({'type':'step','frames':1},{'type':'capture','label':f'{prefix}-{n+1}'})]
def tail(delay,action):
 result=dense(delay,'phase')+[key('s',True)]+dense(17,'depart')+[key('s',False)]+dense(101,'entry')
 result += [key('s',True)]+dense(25,'spikes')+[key('s',False)]+dense(120,'hurt')
 result += [key('w',True)]+dense(17,'coast')+[key('w',False)]+dense(370,'death')
 result += [{'type':'capture','label':'death-settled'},key(action,True)]+dense(1,'request')+[key(action,False)]+dense(80,'dispatch')
 return result
def build(identifier,seed,delay,action,required):
 case=paired.build('paired-room-combat',seed)
 case['scenario']={'id':identifier,'description':'Public spike movement then deferred death popup action','systems':['player-death','damage-pipeline','input-clock','audio','menus'],'randomized':not required}
 for name in ['tape','pairedTape']:
  at=next(i for i,a in enumerate(case[name]) if a.get('label')=='gameplay-start')
  case[name]=case[name][:at+1]+tail(delay,action)
 case['deathRequired']=required;case['deathAction']=action
 case['deathScope']='Normal no-revival Isaac. Original death/finish/request execution is mandatory for contrived cases; every random outcome is retained. No observed source state is supplied to the browser.'
 paired.runner.validate_tape(case['tape']);return case
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--count',type=int,default=2);a=p.parse_args();assert 0<=a.count<=16
 cases=[build('death-spikes-restart','22PFCSFS',0,'Return',True),build('death-spikes-exit','22PFCSFS',1,'Escape',True)]
 for n in range(a.count):cases.append(build(f'death-unfiltered-{n}',None,n%2,'Return' if n%2==0 else 'Escape',False))
 print(json.dumps({'schema':1,'cases':cases}))
