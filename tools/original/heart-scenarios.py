#!/usr/bin/env python3
"""Public natural Heart inputs: contrived source fixtures and unfiltered OS draws."""
import argparse,importlib.util,json,secrets
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('heart_paired',Path(__file__).with_name('foundation-paired-scenarios.py'))
paired=importlib.util.module_from_spec(spec);spec.loader.exec_module(paired)
def case(identifier,code,delay,randomized):
 result=paired.build('paired-room-combat',code)
 result['scenario']={'id':identifier,'description':'Natural Heart admission, public blast damage and collection','systems':['pickups','health-inventory'],'randomized':randomized}
 result['heartRequired']=[] if randomized else ['admission','damage','collection']
 result['entropy']='OS secrets; one draw, no discovery filter or grading reroll' if randomized else 'Deterministic contrived public Room227 fixture, recorded before native/browser grading'
 tail=[];number=0
 # Room227 contains an actual central heart surrounded by four natural poops.
 # The same public blast clears the approach and attempts ordinary self-damage.
 for key,frames in [('s',17),(None,80+delay),('s',35),(None,20),('e',1),(None,125),('s',50),(None,30)]:
  if key:tail.append({'type':'key','key':key,'down':True})
  for _ in range(frames):
   number+=1;tail.extend([{'type':'step','frames':1},{'type':'capture','label':f'heart-{number}'}])
  if key:tail.append({'type':'key','key':key,'down':False})
 for name in ('tape','pairedTape'):
  end=next(i for i,a in enumerate(result[name]) if a.get('label')=='gameplay-start')
  result[name]=result[name][:end+1]+tail;paired.runner.validate_tape(result[name])
 return result
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--count',type=int,default=2);args=parser.parse_args()
 if not 1<=args.count<=4:raise ValueError('Unfiltered OS count must be1..4')
 cases=[case(f'paired-heart-{kind}-{phase}',code,phase,False) for kind,code in [('red','B9118AJA'),('soul','B9119YPW')] for phase in (0,1)]
 # Record every OS draw before either game or a selector grade runs.
 draws=[secrets.randbelow(0xffffffff)+1 for _ in range(args.count)]
 for index,seed in enumerate(draws):cases.append(case(f'paired-heart-os-{index}',paired.runner.codec.encode_seed(seed),index%2,True))
 print(json.dumps({'schema':1,'cases':cases,'unfilteredOsSeedValues':draws}))
if __name__=='__main__':main()
