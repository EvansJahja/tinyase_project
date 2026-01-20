use thiserror::Error;
use zerocopy::*;
use core::iter::Iterator;

use crate::parser::chunk::ChunkIterator;


pub struct FrameListIterator<'a> {
    pub(crate) rest: &'a [u8],
    pub(crate) remaining: u16,
}

impl<'a> Iterator for FrameListIterator<'a> {
    type Item = FrameReader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining <= 0 {
            return None
        }
        let Some(fr) = FrameReader::new(self.rest).ok() else {
            return None
        };
        let frame_size = fr.size() as usize;
        self.rest = &self.rest[frame_size..];
        self.remaining = self.remaining - 1;
        
        Some(fr)
    }
}

pub struct FrameReader<'a> {
    pub frame: &'a ASEFrameHeader,
    pub rest: &'a[u8],
}

impl<'a> FrameReader<'a> {
    fn new(data: &'a[u8]) -> Result<FrameReader<'a>, FrameParseError> {
        parse_frame(data).map(|(frame, rest)| FrameReader{frame: frame, rest})
    }

    fn size(&self) -> u32 {
        self.frame.num_bytes
    }

    pub fn chunks(&self) -> ChunkIterator<'a> {
        ChunkIterator {
            ptr: self.rest,
            count: self.frame.num_chunks as usize,
        }
    }
}

#[derive(Debug, FromBytes, KnownLayout, Immutable)]
#[repr(packed)]
pub struct ASEFrameHeader {
    num_bytes: u32,
    _magic: u16,
    old_unused: u16,
    duration: u16,
    _reserved: [u8; 2],
    num_chunks: u32,
}

#[derive(Error, Debug)]
pub enum FrameParseError {
    #[error("Cast error")]
    CastError,
    #[error("Invalid magic number: {0}")]
    InvalidMagic(u16),
}

pub fn parse_frame<'a>(input: &'a [u8]) -> Result<(&'a ASEFrameHeader, &'a [u8]), FrameParseError> {
    let (frame, rest) = ASEFrameHeader::ref_from_prefix(&input)
        .map_err(|_| FrameParseError::CastError)?;
    if frame._magic != 0xF1FA {
        return Err(FrameParseError::InvalidMagic(frame._magic));
    }

    Ok((frame, rest))
}

#[cfg(test)]
mod test {
    use crate::parser::{HeaderReader, chunk::ASEChunk};

    use super::*;
    #[test]
    fn test_reader_2() {
        let v = std::fs::read("tests/anim_idle.ase").unwrap();
        let data: &[u8] = &v;

        let r = HeaderReader::new(data);
        let mut frames = r.frames();
        let frame_1 = frames.next().unwrap();
        let frame_2 = frames.next().unwrap();
        let frame_3 = frames.next().unwrap();
        let frame_4 = frames.next().unwrap();
        let frame_5 = frames.next().unwrap();
        let frame_6 = frames.next().unwrap();
        let frame_7 = frames.next().unwrap();
        let frame_8 = frames.next().unwrap();
        let frame_9 = frames.next().unwrap();
        let mut chunks = frame_8.chunks();
        for chunk in chunks {
            if let ASEChunk::Cel(cel) = chunk {
                let cel_data = cel.get();
                println!("{:?}", cel_data);
            } else {
                println!("{:x?}", chunk);
            }
        }
    }
}