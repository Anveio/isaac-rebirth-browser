// Independent phase checks on live original and production snapshots. Native
// fields never enter production; differing shared-RNG values stay diagnostic.
import assert from 'node:assert/strict';
const f=Math.fround;
const add=(a,b)=>f(f(a)+f(b));
const mul=(a,b)=>f(f(a)*f(b));
const offset=h=>add(add(mul(h,.5),-15),mul(Math.max(add(h,8.5),0),Math.max(add(h,8.5),0)));
// serde emits round-trippable short f32 decimals. Expand them back to the
// represented binary32 before comparing native Python's expanded decimals.
const pair=p=>[f(p.x),f(p.y)];
const uid=t=>`${t.referenceAddress}:${t.opaqueBaseBytes.slice(16,24)}`;

export class TearContracts {
  constructor(){this.previousNative=new Map();this.previousBrowser=new Map();this.birthIds=new Map();this.summary={nativePhaseChecks:0,browserPhaseChecks:0,nativeRemovals:0,browserRemovals:0,nativeRangeDeaths:0,browserRangeDeaths:0,nativeGridDeaths:0,browserGridDeaths:0,numericMismatches:[]};}
  step(state,observed,index){
    const full=observed.systemPhaseCounter%2===1;
    const currentNative=new Map();
    const currentBrowser=new Map();
    for(const t of observed.tears){
      const id=uid(t),previous=this.previousNative.get(id);
      if(!t.active){
        if(previous){assert.ok(full&&previous.dead,'Native removal requires an earlier dying phase');this.summary.nativeRemovals++;}
        continue;
      }
      assert.equal(t.collisionRadius,8.164994239807129,'Fresh native tear collision radius');
      const actual=full?t.position:t.previousPosition;
      this.check({actual,velocity:t.velocity,height:t.height,speed:t.fallingSpeed,offset:t.renderOffset[1],dead:t.dead,grid:t.gridContact},previous,full,'native');
      currentNative.set(id,{actual,velocity:t.velocity,height:t.height,speed:t.fallingSpeed,offset:t.renderOffset[1],dead:t.dead,grid:t.gridContact});
      if(!this.birthIds.has(id))this.birthIds.set(id,this.birthIds.size+1);
      const browser=state.tears.find(tear=>tear.id===this.birthIds.get(id));
      if(browser){
        assert.deepEqual(pair(browser.motion.velocity),t.velocity,'Paired stored native velocity');
        for(const [field,original,production] of [['height',t.height,f(browser.motion.height)],['fallingSpeed',t.fallingSpeed,f(browser.motion.fallingSpeed)],['ground',actual,pair(browser.motion.position)]]){
          if(JSON.stringify(original)!==JSON.stringify(production)&&!this.summary.numericMismatches.some(row=>row.tearId===browser.id&&row.field===field))this.summary.numericMismatches.push({index,tearId:browser.id,field,original,production,reason:'Shared post-Title/run/ambient/effect consumers remain incomplete; neither source values nor cursor adjustments are supplied to production'});
        }
      }
      this.summary.nativePhaseChecks++;
    }
    for(const t of state.tears){
      const m=t.motion,previous=this.previousBrowser.get(t.id);
      assert.equal(f(t.collisionRadius),8.164994239807129,'Fresh production tear collision radius');
      const now={actual:pair(m.position),velocity:pair(m.velocity),height:f(m.height),speed:f(m.fallingSpeed),offset:f(m.renderOffsetY),dead:m.lifetime==='dying',grid:t.gridContact};
      this.check(now,previous,full,'browser');
      const presented=full?now.actual:now.actual.map((value,axis)=>add(value,mul(now.velocity[axis],.5)));
      assert.deepEqual(pair(m.presentedPosition),presented,'Browser ground presentation');
      currentBrowser.set(t.id,now);this.summary.browserPhaseChecks++;
    }
    for(const [id,previous] of this.previousBrowser)if(!currentBrowser.has(id)){assert.ok(full&&previous.dead,'Browser removal requires an earlier dying phase');this.summary.browserRemovals++;}
    this.previousNative=currentNative;this.previousBrowser=currentBrowser;
  }
  check(now,previous,full,side){
    if(!previous){assert.equal(now.height,add(-23.75,now.speed),`${side}:source birth height`);assert.ok(now.speed>=f(.1)&&now.speed<=f(.28),`${side}:source uniform birth falling range`);}
    else if(full){
      assert.equal(previous.dead,false,`${side}:dying tear must retire at next full phase`);
      assert.deepEqual(now.actual,previous.actual.map((v,i)=>add(v,previous.velocity[i])),`${side}:full ground integration`);
      const dead=previous.height>-5||previous.grid;
      assert.equal(now.dead,dead,`${side}:PRE height/contact death`);
      if(dead){
        assert.equal(now.height,previous.height);assert.equal(now.speed,previous.speed);
        this.summary[`${side}${previous.height>-5?'Range':'Grid'}Deaths`]++;
      }else{const speed=add(mul(previous.speed,.9),.1);assert.equal(now.speed,speed,`${side}:falling recurrence`);assert.equal(now.height,add(previous.height,speed),`${side}:height recurrence`);}
    }else{
      assert.deepEqual(now.actual,previous.actual,`${side}:intermediate ground freeze`);
      assert.equal(now.height,previous.height);assert.equal(now.speed,previous.speed);assert.equal(now.dead,previous.dead);
    }
    if(!now.dead)assert.equal(now.offset,offset(full?now.height:add(now.height,mul(now.speed,.5))),`${side}:height render transform`);
  }
}
