use libp2p_identity::PeerId as libp2p_PeerId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PeerId {
    id: libp2p_PeerId,
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id.to_base58())
    }
}

impl From<libp2p_PeerId> for PeerId {
    fn from(id: libp2p_PeerId) -> Self {
        Self { id }
    }
}

impl Serialize for PeerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id.to_bytes().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PeerId {
    fn deserialize<D>(deserializer: D) -> Result<PeerId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <[u8; 32]>::deserialize(deserializer)?;
        let id = libp2p_PeerId::from_bytes(&bytes).map_err(serde::de::Error::custom)?;
        Ok(PeerId { id })
    }
}

impl FromStr for PeerId {
    type Err = libp2p_identity::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = libp2p_PeerId::from_str(s)?;
        Ok(Self { id })
    }
}
