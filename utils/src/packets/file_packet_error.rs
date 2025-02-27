use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::error::Error;
use std::ops::Deref;
use crate::packets::chunk_type::FilePacketType;

#[derive(Debug)]
pub enum FilePacketError{
    // Used by the metadata packet
    NotMetadata,
    // Used by the metadata packet
    FileExists(PathBuf),
    // Used by the chunk packets
    FileNotRegistered,
    // Used by chunk packets
    InvalidPacketType(FilePacketType),
}

impl Display for FilePacketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self{
            FilePacketError::NotMetadata => write!(f, "Not a metadata file packet"),
            FilePacketError::FileExists(path) => write!(f, "File already downloaded or being downloaded: {}",path.display()),
            FilePacketError::FileNotRegistered => write!(f, "Packet not recognized"),
            FilePacketError::InvalidPacketType(packet_type) => write!(f, "Invalid packet type: {packet_type}"),
        }
    }
}

impl Error for FilePacketError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self{
            FilePacketError::NotMetadata => None,
            FilePacketError::FileExists(_) => None,
            FilePacketError::FileNotRegistered => None,
            FilePacketError::InvalidPacketType(_) => None,
        }

    }

}