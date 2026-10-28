use thiserror::Error;
use zerocopy::{FromBytes, Immutable, KnownLayout, little_endian::U16};


use crate::parser::frame::FrameListIterator;

pub mod chunk;
pub mod frame;


pub struct AsepriteFile<'a> {
    data: &'a[u8]
}

impl<'a> AsepriteFile<'a> {
    pub fn new(data: &'a[u8]) -> Self {
        AsepriteFile { data }
    }

    pub fn header(&self) -> &'a Header {
        let (header, _next) = Self::parse_header(self.data).unwrap();
        header
    }
    
    pub fn frames(&self) -> FrameListIterator<'a> {
        let (header, next) = Self::parse_header(self.data).unwrap();
        FrameListIterator {
            ptr: next,
            remaining: header.frames,
        }
    }
    fn parse_header(input: &'a [u8]) -> Result<(&'a Header, &'a [u8]), HeaderParseError> {
        let (header, rest) = Header::ref_from_prefix(&input).map_err(|_| HeaderParseError::CastError)?;

        Ok((header, rest))
    }
}



#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(packed)]
pub struct Header {
    pub filesize: u32,
    _magic: u16,
    pub frames: u16,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub flags: u32,
    _deprecated_speed: u16,
    _reserved_11: u32,
    _reserved_12: u32,
    pub pallet_transparent_idx: u8,
    _reserved2: [u8; 3],
    pub num_colors: u16,
    pub pixel_width: u8,
    pub pixel_height: u8,
    pub xgrid: u16,
    pub ygrid: u16,
    pub grid_width: u16,
    pub grid_height: u16,
    _reserved3: [u8; 84],
}



#[derive(Error, Debug)]
pub enum HeaderParseError {
    #[error("Cast error")]
    CastError,
}






pub fn prefix_string(ptr: &[u8]) -> (&str, &[u8]) {
    let (name_len, rest) = U16::ref_from_prefix(ptr).unwrap();
    let name_len: usize = name_len.get() as _;
    let layer_name = str::from_utf8(&rest[..name_len]).unwrap();
    let rest = &rest[name_len..];

    (layer_name, rest)
}




#[cfg(test)]
mod test {
    use crate::parser::chunk::ASEChunkReader;

    use super::*;



    // #[test]
    // fn test_read() {
    //     let a = std::fs::read("tests/anim_idle.ase").unwrap();
    //     let (header, rest) = parse_header(&a).unwrap();

    //     let ASEFrameContainer(frame , rest) = parse_frame(rest).unwrap();

    //     let c: ChunkIterator = ChunkIterator { ptr: rest, count: frame.num_chunks as usize};
    //     let chunk = c.get(17).unwrap();
    //     if let ASEChunk::Unknown(contents) = chunk.get_chunk() {
    //         println!("{:#x?}", &contents[..10]);
    //     }

    // }
}
