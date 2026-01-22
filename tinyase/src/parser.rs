use thiserror::Error;
use zerocopy::*;
use core::iter::Iterator;

use crate::parser::frame::FrameListIterator;

use self::chunk::*;

pub mod chunk;
pub mod frame;


pub struct HeaderReader<'a> {
    data: &'a[u8]
}

impl<'a> HeaderReader<'a> {
    pub fn new(data: &'a[u8]) -> Self {
        HeaderReader { data }
    }

    pub fn header(&self) -> &'a ASEHeader {
        let (header, next) = parse_header(self.data).unwrap();
        header
    }
    
    pub fn frames(&self) -> FrameListIterator<'a> {
        let (header, next) = parse_header(self.data).unwrap();
        FrameListIterator {
            rest: next,
            remaining: header.frames,
        }
    }
}



#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(packed)]
pub struct ASEHeader {
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

pub fn parse_header<'a>(input: &'a [u8]) -> Result<(&'a ASEHeader, &'a [u8]), HeaderParseError> {
    let (header, rest) = ASEHeader::ref_from_prefix(&input).map_err(|_| HeaderParseError::CastError)?;

    Ok((header, rest))
}



#[cfg(test)]
mod test {
    use super::*;

    pub fn get_chunk_17<'a>() -> ASEChunkReader<'a>  {
        let a = std::fs::read("tests/anim_idle.ase").unwrap();
        let (header, rest) = parse_header(a.leak()).unwrap();

        // let (fc, rest) = parse_frame(rest).unwrap();
        // let w: ChunkIterator<'a> = fc.chunks();
        // // let cptr = Box::leak(Box::new(w));
        // let cptr = w;

        // let chunk: ASEChunkContainer<'a> = cptr.get(17).unwrap();
        // chunk
        todo!()
    }


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
