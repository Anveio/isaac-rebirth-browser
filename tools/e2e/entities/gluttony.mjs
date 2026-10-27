import { pickupFields } from './champion-drop.mjs';

export default {
  type: 49, variant: 0,
  about: 'Gluttony miniboss rooms: entry, pathfinder walk, FatAttack burst, laser, damage, death and room clear',
  // 1X37ZP4V start room 6's right door leads to miniboss room 4 (#147).
  contrived: ['1X37ZP4V:2'],
  count: 1,
  required: ['admission'],
  fields: (native, actor) => [
    ['stateFrame', native.npcStateFrame, actor.gluttony?.counter],
    ['laserDirection', native.npcWanderVelocity, actor.gluttony && [actor.gluttony.direction.x, actor.gluttony.direction.y]],
    ['pathTarget', native.npcPathTarget, actor.pathfinder && [actor.pathfinder.target.x, actor.pathfinder.target.y]],
    ['pathDirection', native.npcPathDirection, actor.pathfinder?.direction],
    ['pathStuck', native.npcPathStuck, actor.pathfinder?.stuck],
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
  // The boss callback's drops (5c8430) and the clear award are graded as
  // live pickups at every capture.
  roomFields: pickupFields,
};
