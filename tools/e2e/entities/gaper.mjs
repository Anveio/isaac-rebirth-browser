export default {
  type: 10, variant: 1,
  about: 'Gaper rooms: FindGridPath chase, groans on sight, and the dead-AI Morph into Gusher 11.0 or Pacer 11.1',
  // OS-random discoveries next to the start room (catalog 110 and 553);
  // 0F8NB23G door 2 rolls a green champion Gaper, whose champion hook leaves
  // Creep (Green) 1000.23 on every seventh game update.
  contrived: ['2E4HDLMF:0', 'SD8SQGXA:3', '0F8NB23G:2'],
  count: 1,
  required: ['admission'],
  greenCreepContract: { requiredRoute: { seed: '0F8NB23G', slot: 2 } },
};
