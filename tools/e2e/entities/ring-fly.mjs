import { f32 } from '../../foundation/entity-compare.mjs';

export default {
  type: 222, variant: 0,
  about: 'Ring Fly rooms: the shared ring around a leader walking to the player, its rotation and breathing radius, damage, death and room clear',
  contrived: ['Y81EVEQ7:2', 'M9V60SQ4:1'],
  count: 2,
  required: ['admission'],
  fields: (native, actor) => [
    ['ringSlotRadius', native.npcWanderVelocity?.map(f32), actor.wanderVelocity && [f32(actor.wanderVelocity.x), f32(actor.wanderVelocity.y)]],
    ['ringPoint', native.npcGridFeedbackVelocity?.map(f32), actor.ringFly && [f32(actor.ringFly.point.x), f32(actor.ringFly.point.y)]],
    ['ringGrowing', native.npcAux1, actor.ringFly?.growing],
    ['ringHold', native.npcAux2, actor.ringFly?.holdDistance],
  ],
};
