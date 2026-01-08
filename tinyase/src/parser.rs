use alloc::vec::Vec;
use thiserror::Error;
use zerocopy::*;
use core::convert::Infallible;
use core::iter::Iterator;
use core::ops::Index;
use core::marker::PhantomData;
use core::ops::Deref;

use self::chunk::*;

pub mod chunk;

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

// #[derive(NomLE, Debug)]
#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(packed)]
pub struct ASEFrame {
    num_bytes: u32,
    _magic: u16,
    old_unused: u16,
    duration: u16,
    _reserved: [u8; 2],
    num_chunks: u32,
}

pub struct ASEFrameContainer<'a> (pub &'a ASEFrame, pub &'a [u8]);

impl<'a> IntoIterator for ASEFrameContainer<'a> {
    type Item = ChunkPtr<'a>;
    type IntoIter = ChunkPtr<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChunkPtr {
            ptr: self.1,
            count: self.0.num_chunks as usize,
        }
    }
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


#[derive(Error, Debug)]
pub enum FrameParseError {
    #[error("Cast error")]
    CastError,
    #[error("Invalid magic number: {0}")]
    InvalidMagic(u16),
}
pub fn parse_frame<'a>(input: &'a [u8]) -> Result<ASEFrameContainer, FrameParseError> {
    let (frame, rest) = ASEFrame::ref_from_prefix(&input)
        .map_err(|_| FrameParseError::CastError)?;
    if frame._magic != 0xF1FA {
        return Err(FrameParseError::InvalidMagic(frame._magic));
    }

    Ok(ASEFrameContainer(frame, rest))
}

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(packed)]
pub struct ASEChunkHeader {
    pub size: u32,
    pub chunk_type: u16,
}

impl HasSize for &'_ ASEChunkHeader {
    fn size(&self) -> usize {
        self.size as usize
    }
}

#[derive(Error, Debug, Clone)]
pub enum ChunkHeaderParseError {
    #[error("Cast error")]
    CastError,
}


pub struct ASEChunkContainer<'a> (pub &'a ASEChunkHeader, pub &'a [u8]);
impl<'a> ASEChunkContainer<'a> {
    fn get_chunk(&self) -> ASEChunk<'a> {
        ASEChunk::new(self.0.chunk_type, self.1)
    }
}

pub trait NextResult<'a> {
    type Output;
    type Error;
    fn next(&'a self) -> Result<(Self::Output, &'a [u8]), Self::Error>;
}

trait HasSize {
    fn size(&self) -> usize;
}


// ptr Points to start of chunk header, which is <u32 size><u16 type>
#[derive(Debug, Clone)]
pub struct ChunkPtr<'a> {
    ptr: &'a [u8],
    count: usize,
}

impl ChunkPtr<'_> {
    fn len(&self) -> usize {
        self.count
    }

    fn get<'a>(&'a self, index: usize) -> Option<ASEChunkContainer<'a>> {
        let mut w = self.clone();
        // let mut w: ChunkPtr<'a>  = self.clone();
        if index >= self.count {
            panic!("Index out of bounds");
        }
        for _ in 0..index-1 {
            if let None = w.next() {
                panic!("Index out of bounds");
            }
        }

        w.next().map(|chunk_ptr|{
            ASEChunkHeader::ref_from_prefix(chunk_ptr.ptr).ok()
            .map(|(h, p)| ASEChunkContainer::<'a>(h, p))
        }).flatten()

    }
}


#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(packed)]
struct ChunkCel {
    layer_index: u16,
    x_pos: i16,
    y_pos: i16,
    opacity: u8,
    cel_type: u16,
    z_index: i16,
    _reserved: [u8; 5],
}

impl<'a> Iterator for ChunkPtr<'a> 
{
    // type Item = (&'a ASEChunkHeader, &'a [u8]);
    type Item = ChunkPtr<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((chunk,rest)) = ASEChunkHeader::ref_from_prefix(&self.ptr)
            .map_err(|_| ChunkHeaderParseError::CastError)
            .ok() {
            // chunk.validate();
            #[cfg(test)] {
                let chunk_type = chunk.chunk_type;
                let size = chunk.size;
                println!("Chunk type: {:#x}, size: {}", chunk_type, size);
            }
            let my_resp = self.clone();
            self.ptr = &rest[(chunk.size as usize - 6)..];
            Some(my_resp)
        } else {
            return None;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}
// struct ASE<'a> {
//     header: &'a ASEHeader,
//     data: &'a [u8],
// }
// impl <'a> ASE<'a> {
//     pub fn frames() -> ASEFrameContainer<'a> {
//         ASEFrameContainer((), ())
//     }
// }



#[cfg(test)]
mod test {
    use super::*;
    use std::boxed::Box;

    pub fn get_chunk_17<'a>() -> ASEChunkContainer<'a>  {
        let a = std::fs::read("tests/anim_idle.ase").unwrap();
        let (header, rest) = parse_header(a.leak()).unwrap();

        let fc: ASEFrameContainer<'a> = parse_frame(rest).unwrap();
        let cptr = Box::leak(Box::new(fc.into_iter()));

        // let c: ChunkPtr = ChunkPtr { ptr: rest, count: frame.num_chunks as usize};
        let chunk = cptr.get(17).unwrap();
        chunk
        // chunk
    }

    #[test]
    fn test_read() {
        let a = std::fs::read("tests/anim_idle.ase").unwrap();
        let (header, rest) = parse_header(&a).unwrap();

        let ASEFrameContainer(frame , rest) = parse_frame(rest).unwrap();

        let c: ChunkPtr = ChunkPtr { ptr: rest, count: frame.num_chunks as usize};

        // let chunk = &c[2];
    
        // let z  = parse_chunk_header(rest);
        // let (chunk , rest) = z.next().unwrap();

        // let (rest, ase) = ASE::parse(&a).unwrap();
        // println!("{:#x?}, rest: {}", ase, rest.len());
        // let w = (c[0]);
        let chunk = c.get(17).unwrap();
        // let x = w;
        // let ct = chunk.0.chunk_type;
        if let ASEChunk::Unknown(contents) = chunk.get_chunk() {
            println!("{:#x?}", &contents[..10]);
        }


    }
}
