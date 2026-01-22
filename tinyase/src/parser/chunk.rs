use thiserror::Error;
use zerocopy::*;

pub mod layer;

// ptr Points to start of chunk header, which is <u32 size><u16 type>
#[derive(Debug, Clone)]
pub struct ChunkIterator<'a> {
    pub ptr: &'a [u8],
    pub remaining: usize,
}

impl<'a> Iterator for ChunkIterator<'a> 
{
    type Item = ASEChunk<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining <= 0 {
            return None
        }
        if let Some((chunk,rest)) = ASEChunkHeader::ref_from_prefix(&self.ptr)
            .map_err(|_| ChunkHeaderParseError::CastError)
            .ok() {
            // #[cfg(test)] {
            //     let chunk_type = chunk.chunk_type;
            //     let size = chunk.size;
            //     println!("Chunk type: {:#x}, size: {}", chunk_type, size);
            // }
            let my_resp = ASEChunkReader(chunk, rest);
            self.ptr = &rest[(chunk.size as usize - 6)..];
            self.remaining = self.remaining - 1;
            Some(my_resp.get_chunk())
        } else {
            return None;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

pub struct ASEChunkReader<'a> (pub &'a ASEChunkHeader, pub &'a [u8]);
impl<'a> ASEChunkReader<'a> {
    pub fn get_chunk(&self) -> ASEChunk<'a> {
        let chunk_size = self.0.size;
        let data = &self.1[..(chunk_size as usize - 6)];
        ASEChunk::new(self.0.chunk_type, data)
    }
}

#[derive(Debug)]
pub enum ASEChunk<'a> {
    Unknown(u16, &'a[u8]),
    Cel(CelContainer<'a>),
    Layer(Layer<'a>),
}

#[cfg(test)]
    use std::fmt::Display;

use crate::parser::chunk::layer::Layer;

#[cfg(test)]
    impl Display for ASEChunk<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ASEChunk::Unknown(t, _) => write!(f, "Unknown Chunk Type: {:#x}", t),
                ASEChunk::Cel(_) => write!(f, "Cel Chunk"),
                ASEChunk::Layer(_) => write!(f, "Layer Chunk"),
            }
        }
    }

impl<'a> ASEChunk<'a> {
    pub(super) fn new(chunk_type: u16, data: &'a[u8]) -> Self {
        match chunk_type {
            0x2004 => ASEChunk::Layer(Layer::new(data)),
            0x2005 => ASEChunk::Cel(chunk_cel(data)),
            _ => ASEChunk::Unknown(chunk_type, data),
        }
    }
}

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(packed)]
pub struct ASEChunkHeader {
    pub size: u32,
    pub chunk_type: u16,
}

#[derive(Error, Debug, Clone)]
pub enum ChunkHeaderParseError {
    #[error("Cast error")]
    CastError,
}

pub trait NextResult<'a> {
    type Output;
    type Error;
    fn next(&'a self) -> Result<(Self::Output, &'a [u8]), Self::Error>;
}


#[derive(Debug, Unaligned, TryFromBytes, KnownLayout, Immutable)]
#[repr(packed)]
pub struct CelHeader {
    pub layer_index: u16,
    pub point_x: i16,
    pub point_y: i16,
    pub opacity: u8,
    pub cel_type: u16,
    pub z_index: i16,
    _unused_1: [u8; 5],
}

fn chunk_cel<'a>(ptr: &'a[u8]) -> CelContainer<'a>{
    let (h, p) = CelHeader::try_ref_from_prefix(ptr).unwrap();
    CelContainer { cel_header: h, ptr: p }
}

#[derive(Debug)]
pub struct CelContainer<'a> {
    pub cel_header: &'a CelHeader,
    ptr: &'a [u8],
}

#[derive(Debug)]
pub enum CelData<'a> {
    Raw(RawImageDataContainer<'a>),
    Linked(u16),
    
}

impl<'a> CelContainer<'a> {
    pub fn get(&self) -> CelData<'a> {
        let header = self.cel_header;
        let cel_type = header.cel_type;
        match cel_type {
            0 => {
                let a = RawImageHeader::ref_from_prefix(self.ptr).unwrap();
                let b = RawImageDataContainer{header: a.0, ptr: a.1};
                CelData::Raw(b)
            },
            1 => {
                let (z, _) = U16::<LittleEndian>::ref_from_prefix(self.ptr).unwrap();
                CelData::Linked(z.get())
            }
            _ => panic!("Unsupported cel_type: {}", cel_type)
        }
    }
}

#[derive(Debug, FromBytes, Immutable, KnownLayout)]
#[repr(packed)]
pub struct RawImageHeader {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
pub struct RawImageDataContainer<'a> {
    pub header: &'a RawImageHeader,
    pub ptr: &'a [u8],
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cel() {
        let cont = crate::parser::test::get_chunk_17();
        // assume this is a cel

        let chunk_ptr = cont.1;
        let ase_chunk = chunk_cel(chunk_ptr);
        let cd = ase_chunk.get();
        if let CelData::Raw(r) = cd {
            println!("{:?}", r.header)

        } else {
            panic!("unexpected cel type")
        };

        // println!("{:#x?}", cd);

    }
}