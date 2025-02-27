use std::fmt::{Display, Formatter};
use std::fmt;

/// Enum for defining the type of each packet for the file transfer protocol.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilePacketType {

    Unknown(u8),
    Metadata,
    Chunk,
    LastChunk,

}


impl From<u8> for FilePacketType {

    fn from(item: u8) -> Self{
        match item{
            1 => FilePacketType::Metadata,
            2 => FilePacketType::Chunk,
            3 => FilePacketType::LastChunk,
            _ => FilePacketType::Unknown(item),
        }
    }

}

impl From<FilePacketType> for u8{

    fn from(item: FilePacketType) -> Self {

        match item{

            FilePacketType::Metadata => 1,
            FilePacketType::Chunk => 2,
            FilePacketType::LastChunk => 3,
            FilePacketType::Unknown(num) => num,

        }

    }

}

impl Display for FilePacketType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FilePacketType::Unknown(code) => write!(f, "Unknown({})", code),
            FilePacketType::Metadata => write!(f, "Metadata"),
            FilePacketType::Chunk => write!(f, "Chunk"),
            FilePacketType::LastChunk => write!(f, "LastChunk"),
        }
    }
}



#[cfg(test)]

pub mod tests{
    use super::*;

    #[test]
    fn test_file_packet_type1(){

        let packet_type = FilePacketType::from(1);
        assert_eq!(packet_type,FilePacketType::Metadata);

        let packet_type = FilePacketType::from(2);
        assert_eq!(packet_type,FilePacketType::Chunk);

        let packet_type = FilePacketType::from(3);
        assert_eq!(packet_type,FilePacketType::LastChunk);

        let packet_type = FilePacketType::from(255);
        assert_eq!(packet_type,FilePacketType::Unknown(255));

    }

    #[test]
    fn test_file_packet_type2(){

        let packet_type: FilePacketType = 1u8.into();
        assert_eq!(packet_type,FilePacketType::Metadata);

        let packet_type: FilePacketType = 2u8.into();
        assert_eq!(packet_type,FilePacketType::Chunk);

        let packet_type: FilePacketType = 3u8.into();
        assert_eq!(packet_type,FilePacketType::LastChunk);

        let packet_type: FilePacketType = 255u8.into();
        assert_eq!(packet_type,FilePacketType::Unknown(255));

    }

    #[test]
    fn test_file_packet_type3(){

        let num = u8::from(FilePacketType::Metadata);
        assert_eq!(num,1);
        let num = u8::from(FilePacketType::Chunk);
        assert_eq!(num,2);
        let num = u8::from(FilePacketType::LastChunk);
        assert_eq!(num,3);
        let num = u8::from(FilePacketType::Unknown(255));
        assert_eq!(num,255);

    }


}
