//! The imported glibc `rand()` (TYPE_3 additive feedback) with its default
//! initial state: no `srand` is imported, so the process-global generator
//! starts from seed 1. It is separate from the run xorshift and the shared MT.
//! Its game callers are the nightmare list shuffle 5b8940 at System init and
//! the Curse of the Maze room swap 58deb0/58d700 in Level::ChangeRoom.
//! External library consumption before initialization still needs live
//! readback.
use serde::{Deserialize, Serialize};

const WORDS: usize = 31;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibcRand {
    words: [u32; WORDS],
    front: usize,
    rear: usize,
}

impl Default for LibcRand {
    /// The generator as System init leaves it: seeded, then the nightmare
    /// shuffle's draws.
    fn default() -> Self {
        let mut rng = Self::seeded();
        crate::nightmares::shuffled_order(&mut rng);
        rng
    }
}

impl LibcRand {
    pub fn seeded() -> Self {
        let mut words = [0; WORDS];
        words[0] = 1;
        for index in 1..WORDS {
            words[index] = ((u64::from(words[index - 1]) * 16_807) % 2_147_483_647) as u32;
        }
        let mut rng = Self {
            words,
            front: 3,
            rear: 0,
        };
        for _ in 0..310 {
            rng.next();
        }
        rng
    }

    pub fn next(&mut self) -> u32 {
        let value = self.words[self.front].wrapping_add(self.words[self.rear]);
        self.words[self.front] = value;
        self.front = (self.front + 1) % WORDS;
        self.rear = (self.rear + 1) % WORDS;
        value >> 1
    }

    /// libstdc++ `std::random_shuffle(first, last)`: for each later position,
    /// swap with `rand() % (i + 1)`.
    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in 1..values.len() {
            values.swap(index, self.next() as usize % (index + 1));
        }
    }
}
