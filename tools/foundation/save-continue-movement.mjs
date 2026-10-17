// Public live witness law for the declared public D8/coast24 movement.
// Both arguments are actual Browser captures. Native files/values are not inputs.
import assert from 'node:assert/strict';

export function assertPublicRightMovement(before,after,savedFloor,savedRoom) {
  assert.ok(Number.isSafeInteger(savedRoom),'Actual saved room index is required');
  assert.equal(before.currentRoom,savedRoom,'Movement starts in the actual restored room');
  assert.equal(before.screen,'playing');assert.equal(after.screen,'playing');
  assert.ok(Number.isFinite(before.player.x)&&Number.isFinite(before.player.y));
  assert.ok(Number.isFinite(after.player.x)&&Number.isFinite(after.player.y));
  assert.equal(before.player.controlsEnabled,true,'Declared public movement requires actual enabled controls');
  if(after.currentRoom===before.currentRoom){
    //Keep the original same-room rightward-motion obligation in both callers.
    assert.ok(after.player.x>before.player.x,'Public movement must operate on the continued world');
    assert.equal(after.currentRoom,savedRoom);
    return;
  }
  //A right-door exit changes the local coordinate origin. Require the actual
  //reciprocal floor/door/transition owners rather than local-x monotonicity.
  const from=savedFloor.descriptors.filter(row=>row.index===savedRoom);assert.equal(from.length,1);
  const targetIndex=from[0].neighbors[2];assert.ok(Number.isSafeInteger(targetIndex));
  const target=savedFloor.descriptors.filter(row=>row.index===targetIndex);assert.equal(target.length,1);
  assert.equal(target[0].neighbors[0],from[0].index);assert.equal(target[0].gridIndex,from[0].gridIndex+1);
  for(const descriptor of [from[0],target[0]]){assert.equal(descriptor.template.width,13);assert.equal(descriptor.template.height,7);}
  const departure=before.room.appearance.doors.filter(door=>door.slot===2&&door.targetRoom===targetIndex);assert.equal(departure.length,1);assert.equal(departure[0].state,'Opened');
  assert.equal(before.player.x,(departure[0].worldPosition.x-40)*0.65+32);assert.equal(before.player.y,departure[0].worldPosition.y*0.65-46);
  const transition=after.roomTransition;assert.ok(transition,'A different room requires an actual transition owner');
  assert.equal(transition.fromRoom,from[0].index);assert.equal(transition.targetRoom,targetIndex);assert.equal(transition.fromSlot,2);assert.equal(transition.targetSlot,0);assert.equal(transition.phase,'in');assert.ok(Number.isSafeInteger(transition.fullUpdates)&&transition.fullUpdates>=0);
  assert.equal(after.currentRoom,targetIndex);assert.equal(after.room.descriptor.index,targetIndex);assert.equal(after.room.descriptor.gridIndex,target[0].gridIndex);
  const arrival=after.room.appearance.doors.filter(door=>door.slot===0&&door.targetRoom===from[0].index);assert.equal(arrival.length,1);
  //Level.ChangeRoom58DBAA closes ordinary config7 doors on an uncleared
  //destination. Real inbound placement uses that reciprocal Door regardless
  //of its new combat lock; require the actual lock owners rather than waive
  //the Door FSM. This witness admits only the declared normal-room doorway.
  assert.equal(target[0].template.type,1);
  const grid=after.room.grid;assert.ok(grid&&Array.isArray(grid.cells));
  assert.equal(grid.width,15);assert.equal(grid.height,9);assert.equal(grid.cells.length,135);
  const column=(arrival[0].worldPosition.x-40)/40,row=(arrival[0].worldPosition.y-120)/40;
  assert.ok(Number.isInteger(column)&&Number.isInteger(row));
  const door=grid.cells[column+row*grid.width]?.entity;assert.ok(door,'Actual reciprocal grid Door is mandatory');
  assert.equal(door.kind,16);assert.equal(door.variant,7);
  if(after.room.combatClear===false){
    assert.equal(arrival[0].state,'Closed');assert.equal(door.state,1);assert.equal(door.collisionClass,4);
    const slots=after.room.combatShutDoorSlots;assert.ok(Array.isArray(slots));
    assert.ok(slots.every(slot=>Number.isInteger(slot)&&slot>=0&&slot<8));assert.equal(new Set(slots).size,slots.length);assert.ok(slots.includes(arrival[0].slot));
    const counts=after.room.managerCounts;assert.ok(counts);
    assert.ok(Number.isInteger(counts.countableCurrent)&&Number.isInteger(counts.countablePending));
    assert.ok(counts.countableCurrent+counts.countablePending>0,'Closed arrival requires actual positive countable ownership');
    assert.equal(after.room.combatClearCountdown,10);
    assert.ok(Array.isArray(after.room.npcs)&&after.room.npcs.some(npc=>npc.canShutDoors===true&&npc.removed===false&&npc.dead===false&&Number.isFinite(npc.hp)&&npc.hp>0),'Closed arrival requires an actual living can-shut-doors NPC');
    assert.ok(Number.isInteger(after.room.combatShutUpdate)&&Number.isInteger(after.room.initializationGameUpdate));
    assert.ok(after.room.combatShutUpdate>=after.room.initializationGameUpdate&&after.room.combatShutUpdate<=after.gameUpdateCount);
  }else{
    assert.equal(after.room.combatClear,true);assert.equal(arrival[0].state,'Opened');assert.equal(door.state,2);
  }
  assert.equal(after.player.x,(arrival[0].worldPosition.x+40)*0.65+32);assert.equal(after.player.y,arrival[0].worldPosition.y*0.65-46);
  assert.equal(after.player.controlsEnabled,false,'The observed In transition retains its real control lock');
}
