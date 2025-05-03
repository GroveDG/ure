use std::{
    collections::{HashMap, HashSet},
    u64,
};

use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD as B64};
use bimap::BiHashMap;
use rand::{RngCore, rngs::ThreadRng};
use serde::{Deserialize, Serialize, de::Visitor};

pub mod sdl;
pub mod tf;
pub mod tree;
pub mod edit;
pub mod rhai;
pub mod assets;

pub type Components<C> = HashMap<UID, C>;
pub type BiComponents<C> = BiHashMap<UID, C>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct UID(u64);
impl Serialize for UID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let b64 = B64.encode(self.0.to_be_bytes());
            serializer.serialize_str(&b64)
        } else {
            serializer.serialize_u64(self.0)
        }
    }
}
impl<'de> Deserialize<'de> for UID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UIDVisitor;
        impl<'de> Visitor<'de> for UIDVisitor {
            type Value = UID;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("u64 UID in base64 or big endian bytes")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut bytes = [0u8; 8];
                B64.decode_slice(v, &mut bytes).map_err(|e| E::custom(e))?;
                Ok(UID(u64::from_be_bytes(bytes)))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(UID(v))
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(UIDVisitor)
        } else {
            deserializer.deserialize_u64(UIDVisitor)
        }
    }
}

#[derive(Debug, Clone)]
pub struct UIDs {
    entities: HashSet<UID>,
    rng: ThreadRng,
}
impl UIDs {
    pub fn new() -> Result<Self, rand::distr::uniform::Error> {
        Ok(Self {
            entities: Default::default(),
            rng: rand::rng(),
        })
    }

    pub fn add(&mut self) -> UID {
        let uid = UID({
            self.rng.next_u64()
            // All UIDs start with "u" (that character is not fully used anyway)
                & 0b0000001111111111111111111111111111111111111111111111111111111111u64
                | 0b1011100000000000000000000000000000000000000000000000000000000000u64
        });
        while !self.entities.insert(uid) {}
        uid
    }

    pub fn delete(&mut self, uid: &UID) {
        self.entities.remove(uid);
    }
}
