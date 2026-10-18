export default {
  type: 51, variant: 0,
  about: 'Envy miniboss rooms (6.2050): wall-bouncing heads at constant speed and the split into smaller heads on each death',
  // OS-random discoveries of the miniboss room next to the start room
  // (catalog 371); RW0KKXS2 is the playtest floor whose top door it blocked.
  contrived: ['QXM1LDQT:1', 'QR7Z9WC9:0'],
  count: 1,
  // The shooting sweep rarely hits a wall-bouncing head; the port's recorded
  // fight leads the nearest head until every split child is dead (the death
  // marker matches 51.0 only, so the split children's deaths are graded as
  // room state and identities, not as a marker).
  fight: 9000,
  required: ['admission'],
};
