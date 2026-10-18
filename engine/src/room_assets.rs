//! Complete structural base-Rebirth STB decoding of authored room assets.
//! Entity and grid selectors remain verbatim; decoding does not instantiate
//! unsupported entities or infer their behavior.
use crate::room_templates::AuthoredTemplate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpawnEntry {
    #[serde(rename = "type")]
    pub entity_type: u16,
    pub variant: u16,
    pub subtype: u16,
    pub weight: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SpawnGroup {
    pub x: i16,
    pub y: i16,
    pub entries: Vec<SpawnEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AuthoredDoor {
    pub x: i16,
    pub y: i16,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoomAsset {
    pub template: AuthoredTemplate,
    pub doors: Vec<AuthoredDoor>,
    pub spawns: Vec<SpawnGroup>,
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(count).ok_or("STB cursor overflow")?;
        let value = self.bytes.get(self.at..end).ok_or("truncated STB")?;
        self.at = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn word(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn signed(&mut self) -> Result<i16, String> {
        Ok(self.word()? as i16)
    }
    fn dword(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn weight(&mut self) -> Result<f32, String> {
        let value = f32::from_bits(self.dword()?);
        if !value.is_finite() || value < 0.0 {
            return Err("invalid STB weight".into());
        }
        Ok(value)
    }
}

pub fn door_slot(x: i16, y: i16) -> Result<u8, String> {
    // Loader 5d4dec..5d514c recognizes these authored interior coordinates.
    match (x, y) {
        (-1, 3) => Ok(0),
        (6, -1) => Ok(1),
        (13 | 26, 3) => Ok(2),
        (6, 7 | 14) => Ok(3),
        (-1, 10) => Ok(4),
        (19, -1) => Ok(5),
        (13 | 26, 10) => Ok(6),
        (19, 7 | 14) => Ok(7),
        _ => Err("unrecognized authored STB door coordinate".into()),
    }
}

pub fn decode(bytes: &[u8]) -> Result<Vec<RoomAsset>, String> {
    if bytes.len() > 32 * 1024 * 1024 {
        return Err("STB exceeds 32 MiB".into());
    }
    let mut reader = Reader { bytes, at: 0 };
    let count = reader.dword()? as usize;
    if count == 0 || count > 100_000 {
        return Err("STB room count outside 1..100000".into());
    }
    let mut rooms = Vec::with_capacity(count);
    for _ in 0..count {
        let room_type = reader.dword()?;
        let variant = reader.dword()?;
        let difficulty = reader.byte()?;
        let name_length = reader.word()? as usize;
        let name_bytes = reader.take(name_length)?;
        if !name_bytes.is_ascii() {
            return Err("non-ASCII base room name".into());
        }
        let name = String::from_utf8(name_bytes.to_vec()).unwrap();
        let weight = reader.weight()?;
        let width = reader.byte()?;
        let height = reader.byte()?;
        let door_count = reader.byte()?;
        let spawn_count = reader.word()?;
        let mut door_mask = 0;
        let mut doors = Vec::with_capacity(usize::from(door_count));
        for _ in 0..door_count {
            let x = reader.signed()?;
            let y = reader.signed()?;
            let enabled = match reader.byte()? {
                0 => false,
                1 => true,
                _ => return Err("invalid STB door enabled flag".into()),
            };
            if enabled {
                door_mask |= 1 << door_slot(x, y)?;
            }
            doors.push(AuthoredDoor { x, y, enabled });
        }
        let mut spawns = Vec::with_capacity(usize::from(spawn_count));
        for _ in 0..spawn_count {
            let x = reader.signed()?;
            let y = reader.signed()?;
            let entry_count = reader.byte()?;
            let mut entries = Vec::with_capacity(usize::from(entry_count));
            for _ in 0..entry_count {
                entries.push(SpawnEntry {
                    entity_type: reader.word()?,
                    variant: reader.word()?,
                    subtype: reader.word()?,
                    weight: reader.weight()?,
                });
            }
            spawns.push(SpawnGroup { x, y, entries });
        }
        rooms.push(RoomAsset {
            template: AuthoredTemplate {
                room_type,
                variant,
                difficulty,
                name,
                weight,
                width,
                height,
                door_mask,
            },
            doors,
            spawns,
        });
    }
    if reader.at != bytes.len() {
        return Err("trailing STB bytes".into());
    }
    Ok(rooms)
}

pub fn special() -> Vec<RoomAsset> {
    decode(include_bytes!(
        "../../references/isaac/rebirth/resources/rooms/00.special rooms.stb"
    ))
    .expect("checked-in authored Special Rooms STB")
}

pub fn basement() -> Vec<RoomAsset> {
    decode(include_bytes!(
        "../../references/isaac/rebirth/resources/rooms/01.basement.stb"
    ))
    .expect("checked-in authored Basement STB")
}

/// stages.xml room set 3, the Caves I/II catalogue (Level stages 3 and 4).
pub fn caves() -> Vec<RoomAsset> {
    decode(include_bytes!(
        "../../references/isaac/rebirth/resources/rooms/03.caves.stb"
    ))
    .expect("checked-in authored Caves STB")
}

/// Resolve complete authored groups through the descriptor's original catalogue
/// index, with an identity check. Selected weights live separately in Floor.
pub fn for_descriptor(descriptor: &crate::floor::Descriptor) -> Result<RoomAsset, String> {
    for_descriptor_with_special(descriptor, None)
}

///One decoded per-boot catalogue supplies both selector metadata and Room
///admission. The transport accepts real STB records, not alternate actor state.
pub(crate) fn for_descriptor_with_special(
    descriptor: &crate::floor::Descriptor,
    source: Option<&[RoomAsset]>,
) -> Result<RoomAsset, String> {
    let mut catalog = match descriptor.asset_stage {
        0 => source.map_or_else(special, <[RoomAsset]>::to_vec),
        1 => basement(),
        3 => caves(),
        _ => return Err("unimplemented authored stage catalogue".into()),
    };
    if descriptor.catalog_index >= catalog.len() {
        return Err("descriptor authored index outside its stage catalogue".into());
    }
    let asset = catalog.swap_remove(descriptor.catalog_index);
    if asset.template != descriptor.template {
        return Err("descriptor identity differs from original authored catalogue".into());
    }
    Ok(asset)
}

///Serialize the immutable shared resource in a complete Game continuation.
///State snapshots never contain this catalogue. Rebinding an Arc after load
///preserves the same decoded authored input for later Room admissions.
pub(crate) mod shared_catalogue {
    use super::RoomAsset;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;

    pub fn serialize<S: Serializer>(
        value: &Option<Arc<Vec<RoomAsset>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.as_deref().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Arc<Vec<RoomAsset>>>, D::Error> {
        Option::<Vec<RoomAsset>>::deserialize(deserializer).map(|value| value.map(Arc::new))
    }
}
