use std::cmp::{min};
use std::error::Error;
use std::ptr;

type BytePacketError = Box<dyn Error>;
type Result<T> = std::result::Result<T, BytePacketError>;

/// Abstraction for building byte packets.
pub struct BytePacket{

    buffer: Vec<u8>,
    buffer_size: usize,
    position: usize,

}

impl BytePacket{

    /// Creates a new packet having the given size.
    pub fn new(buffer_size: usize) -> Self{

        assert!(buffer_size > 0);

        Self{
            buffer: vec![0;buffer_size],
            buffer_size,
            position: 0,
        }

    }

    /// Creates a packet from an already existing buffer.
    pub fn from(buffer: &[u8]) -> Self{

        let buffer_size = buffer.len();

        Self{

            buffer: Vec::from(buffer),
            buffer_size,
            position: 0,

        }

    }

    /// Moves the position to a new one or to the max position if the new position exceeds the buffer size.
    pub fn seek(&mut self,position: usize){
        self.position = min(position, self.buffer_size);
    }

    /// Advances the position by a number of steps. Will be set to max if the new position exceeds the buffer size.
    pub fn step(&mut self, steps: usize){
        self.position = min(self.position + steps,self.buffer_size);
    }

    /// Reads a byte and advances the position by 1 step. Will return an error if the position is at the end of the buffer.
    pub fn read(&mut self) -> Result<u8>{

        if self.position >= self.buffer_size{
            return Err("End of buffer".into());
        }
        self.position += 1;

        Ok(self.buffer[self.position - 1])

    }

    /// Reads 2 bytes and advances the position by a maximum of 2 steps. Will return an error if the position reaches the end of the buffer.
    pub fn read_u16(&mut self) -> Result<u16>{

        let result: u16 = ((self.read()? as u16)  << 8) |
                          ((self.read()? as u16) << 0);
        Ok(result)

    }

    /// Reads 4 bytes and advances the position by a maximum of 4 steps. Will return an error if the position reaches the end of the buffer.
    pub fn read_u32(&mut self) -> Result<u32>{

        let result: u32 = ((self.read()? as u32) << 24)|
                          ((self.read()? as u32) << 16)|
                          ((self.read()? as u32) << 8)|
                          ((self.read()? as u32)<< 0);

        Ok(result)

    }

    /// Reads 8 bytes and advances the position by a maximum of 8 steps. Will return an error if the position reaches the end of the buffer.
    pub fn read_u64(&mut self) -> Result<u64>{

        let result: u64 = ((self.read()? as u64) << 56)|
                          ((self.read()? as u64) << 48)|
                          ((self.read()? as u64) << 40)|
                          ((self.read()? as u64) << 32)|
                          ((self.read()? as u64) << 24)|
                          ((self.read()? as u64) << 16)|
                          ((self.read()? as u64) << 8)|
                          ((self.read()? as u64) << 0);

        Ok(result)

    }

    /// Reads a single byte at a given position, without advancing the position. Will return an error if the position exceeds the size of the buffer.
    pub fn get_byte(&mut self, position: usize) -> Result<u8>{

        if position >= self.buffer_size{
            return Err("End of buffer".into());
        }

        Ok(self.buffer[position])

    }

    /// Writes a single byte. Will return an error if the position reaches the end of the buffer.
    pub fn write(&mut self,value: u8) -> Result<()>{

        if self.position >= self.buffer_size{
            return Err("End of buffer".into());
        }

        self.position += 1;
        self.buffer[self.position-1] = value;

        Ok(())

    }

    /// Writes a maximum of 2 bytes. Will return an error if the position reaches the end of the buffer.
    pub fn write_u16(&mut self,value: u16) -> Result<()>{

        self.write(((value >> 8) & 0xFF) as u8)?;
        self.write(((value >> 0) & 0xFF) as u8)?;

        Ok(())

    }

    /// Writes a maximum of 4 bytes. Will return an error if the position reaches the end of the buffer.
    pub fn write_u32(&mut self,value: u32) -> Result<()>{

        self.write(((value >> 24) & 0xFF) as u8)?;
        self.write(((value >> 16) & 0xFF) as u8)?;
        self.write(((value >> 8) & 0xFF) as u8)?;
        self.write(((value >> 0) & 0xFF) as u8)?;

        Ok(())

    }

    /// Writes a maximum of 8 bytes. Will return an error if the position reaches the end of the buffer.
    pub fn write_u64(&mut self,value: u64) -> Result<()>{

        self.write(((value >> 56) & 0xFF) as u8)?;
        self.write(((value >> 48) & 0xFF) as u8)?;
        self.write(((value >> 40) & 0xFF) as u8)?;
        self.write(((value >> 32) & 0xFF) as u8)?;
        self.write(((value >> 24) & 0xFF) as u8)?;
        self.write(((value >> 16) & 0xFF) as u8)?;
        self.write(((value >> 8) & 0xFF) as u8)?;
        self.write(((value >> 0) & 0xFF) as u8)?;

        Ok(())

    }

    /// Writes an usize value into the buffer. Will return an error if the value cannot be written.
    /// This method checks the length of the usize type to determine how many bytes to write.
    pub fn write_usize(&mut self,value: usize) -> Result<()>{

        let pointer_size = size_of::<usize>();

        match pointer_size{
            8 => self.write_u64(value as u64),
            4 => self.write_u32(value as u32),
            _ => Err("Unknown pointer size".into()),
        }

    }


    /// Attempts to write a non overlapping buffer to the current position.
    /// UNSAFE as the buffer of the packet can be retrieved.
    pub unsafe fn write_buffer(&mut self,buffer: &[u8]) -> Result<()>{

        let new_buffer_size = buffer.len();

        if self.position + new_buffer_size >= self.buffer_size{
           return Err("End of buffer".into());
        }

        unsafe{
            ptr::copy_nonoverlapping(buffer.as_ptr(),self.buffer.as_mut_ptr().add(self.position),new_buffer_size)
        }

        Ok(())

    }

    /// Returns the packet buffer.
    pub fn get_buffer(&self) -> &[u8]{
        self.buffer.as_slice()
    }

}

#[cfg(test)]

mod tests{
    use super::*;

    #[test]
    fn test_read(){

        let buffer = [1,2,3,4];
        let mut packet = BytePacket::from(&buffer);

        assert_eq!(packet.read().unwrap(),1);
        assert_eq!(packet.read().unwrap(),2);
        assert_eq!(packet.read().unwrap(),3);
        assert_eq!(packet.read().unwrap(),4);
        assert_eq!(packet.read().unwrap_or(0),0);

    }

    #[test]
    fn test_read_u16(){

        let buffer = [1,0,0,1];
        let mut packet = BytePacket::from(&buffer);

        assert_eq!(packet.read_u16().unwrap(),1u16 << 8);
        assert_eq!(packet.read_u16().unwrap(),1);
        assert_eq!(packet.read_u16().unwrap_or(0),0);

    }

    fn test_read_u32(){

        let buffer = [1,0,0,1];
        let mut packet = BytePacket::from(&buffer);

        assert_eq!(packet.read_u32().unwrap(),(1u32 << 8) + 1);
        assert_eq!(packet.read_u32().unwrap_or(0),0);
    }

    #[test]
    fn test_seek(){

        let buffer = [1,0,0,1];
        let mut packet = BytePacket::from(&buffer);

        packet.seek(3);

        assert_eq!(packet.position, 3);
    }

    #[test]
    #[should_panic]
    fn test_step(){
        let buffer = [1,0,0,1];
        let mut packet = BytePacket::from(&buffer);

        packet.step(1);
        assert_eq!(packet.position, 1);
        packet.step(2);
        assert_eq!(packet.position, 3);
        packet.step(2);
        assert_eq!(packet.position, 3);

    }

    #[test]
    #[should_panic]
    fn test_get_byte(){
        let buffer = [1,0,0,1];
        let mut packet = BytePacket::from(&buffer);

        assert_eq!(packet.get_byte(0).unwrap(), 1);
        assert_eq!(packet.get_byte(1).unwrap(), 0);
        packet.get_byte(10).unwrap();

    }

    #[test]
    #[should_panic]
    fn test_write(){
        let buffer = [0;4];
        let mut packet = BytePacket::from(&buffer);

        packet.write(1).unwrap();
        packet.write(2).unwrap();
        packet.write(3).unwrap();
        packet.write(4).unwrap();

        assert_eq!(packet.buffer,[1;4]);

        packet.write(5).unwrap();

    }

    #[test]
    #[should_panic]
    fn test_write_u16(){
        let buffer = [0;4];
        let mut packet = BytePacket::from(&buffer);

        packet.write_u16(257).unwrap();
        packet.write_u16(257).unwrap();

        assert_eq!(packet.buffer,[1;4]);

        packet.write_u16(5).unwrap();

    }

    #[test]
    #[should_panic]
    fn test_write_u32(){
        let buffer = [0;4];
        let mut packet = BytePacket::from(&buffer);

        packet.write_u32(1u32<<31).unwrap();

        assert_eq!(packet.buffer,[255;4]);

        packet.write_u16(5).unwrap();

    }

    #[test]
    #[should_panic]
    fn test_write_u64(){
        let buffer = [0;8];
        let mut packet = BytePacket::from(&buffer);
        packet.write_u64(1u64<<63).unwrap();

        assert_eq!(packet.buffer,[255;8]);

        packet.write_u64(5).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_write_usize(){
        let buffer = [0;8];
        let mut packet = BytePacket::from(&buffer);

        let pointer_size = size_of::<usize>();

        match pointer_size{

            8 =>{
                packet.write_usize((1usize<<32) + 1).unwrap();
                assert_eq!(packet.buffer,[0,0,0,1,0,0,0,1]);
                packet.write_usize(1).unwrap();
            }

            4 =>{
                packet.write_usize((1usize<<15) + 1).unwrap();
                assert_eq!(packet.buffer,[0,0,0,0,1,0,0,1]);
                packet.write_usize(1).unwrap();
            }

            _ => panic!("Should not happen")

        }

    }

    #[test]
    #[should_panic]
    fn test_write_buffer(){

        let buffer  = [0;8];
        let mut packet = BytePacket::from(&buffer);

        let new_buffer = [1,1,1,1,0,0,0,1];

        unsafe{
            packet.write_buffer(&new_buffer).unwrap();
            assert_eq!(packet.buffer,new_buffer);
            packet.write_buffer(&new_buffer).unwrap();
        }


    }



}