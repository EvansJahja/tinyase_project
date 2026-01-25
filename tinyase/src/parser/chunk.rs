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
    pub fn get(&'a self) -> CelData<'a> {
        let header = self.cel_header;
        let cel_type = header.cel_type;
        match cel_type {
            0 => {
                let a = RawImageHeader::ref_from_prefix(self.ptr).unwrap();
                let b = RawImageDataContainer{parent: self, header: a.0, ptr: a.1};
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

#[derive(Debug, Clone)]
pub struct RawImageDataContainer<'a> {
    pub parent: &'a CelContainer<'a>,
    pub header: &'a RawImageHeader,
    pub ptr: &'a [u8],
}

#[cfg(feature = "embedded_graphics")]  
pub mod embedded_graphics_impl {
    use super::*;
    use embedded_graphics::pixelcolor::raw::RawU2;
    use embedded_graphics::pixelcolor::{BinaryColor, Gray2};
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::Rectangle;

    
    impl From<&RawImageDataContainer<'_>> for Rectangle {
        fn from(value: &RawImageDataContainer) -> Self {
            let x = value.parent.cel_header.point_x as i32;
            let y = value.parent.cel_header.point_y as i32;
            let width = value.header.width as u32;
            let height = value.header.height as u32;
            Rectangle::new(
                Point::new(x, y),
                Size::new(width, height)
            )
        }

    }

    impl<'a> IntoIterator for &'a RawImageDataContainer<'a> {
        type Item = AlphaBinaryColor;
        type IntoIter = PixelIterator<'a>;
        
        fn into_iter(self) -> Self::IntoIter {
            PixelIterator::new(self.clone())
            // PixelIterator {
            //     idc: self.clone(),
            //     remaining: (self.header.width as usize) * (self.header.height as usize),
            // }
        }

    }

    pub struct PixelIterator<'a> {
        idc: RawImageDataContainer<'a>,
        ptr: &'a [u8],
        idx: usize,
    }

    impl<'a> PixelIterator<'a> {
        pub fn new(idc: RawImageDataContainer<'a>) -> Self {
            PixelIterator {
                ptr: idc.ptr,
                idc,
                idx: 0,
            }
        }
    }

    // Used for embedded graphics, starting from top-left
    impl Iterator for PixelIterator<'_> {
        type Item = AlphaBinaryColor;

        fn next(&mut self) -> Option<Self::Item> {
            // todo!();

            if self.idx >= (self.idc.header.width as usize) * (self.idc.header.height as usize) {
                return None;
            }


            // Assume the following colors: transparent, black, white
            let ret = match self.ptr.get(self.idx) {
                Some(x) => Some(AlphaBinaryColor(RawU2::new(*x))),
                _ => return None,
            };
            self.idx += 1;
            ret
        }
    }

    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    pub struct AlphaBinaryColor(RawU2);

    impl PixelColor for AlphaBinaryColor {
        type Raw = RawU2;
    }
    
    impl From<RawU2> for AlphaBinaryColor {
        fn from(data: RawU2) -> Self {
            AlphaBinaryColor(data)
        }
    }

    impl From<AlphaBinaryColor> for RawU2 {
        fn from(color: AlphaBinaryColor) -> Self {
            color.0
        }
    }

        
}





#[cfg(all(test, feature = "embedded_graphics"))]
mod embedded_graphics_test {
    use embedded_graphics::{framebuffer::{Framebuffer, buffer_size}, pixelcolor::{self, Gray2, raw::RawU2}};

    use crate::parser::chunk::embedded_graphics_impl::AlphaBinaryColor;

    use super::*;

    #[test]
    fn test_pixel_iterator() {
        let mut back_buffer: Framebuffer<AlphaBinaryColor, RawU2, BigEndian, 32, 32, {buffer_size::<AlphaBinaryColor>(320, 240)}> = Framebuffer::new();
    }
}