use thiserror::Error;
use zerocopy::*;
use core::iter::Iterator;

use self::chunk::*;

pub mod chunk;


struct HeaderReader<'a> {
    data: &'a[u8]
}

impl<'a> HeaderReader<'a> {
    pub fn new(data: &'a[u8]) -> Self {
        HeaderReader { data }
    }

    fn header(&self) -> &'a ASEHeader {
        let (header, next) = parse_header(self.data).unwrap();
        header
    }
    
    fn frames(&self) -> FrameListIterator<'a> {
        let (header, next) = parse_header(self.data).unwrap();
        FrameListIterator {
            rest: next,
            remaining: header.frames,
        }
    }
}

struct FrameListIterator<'a> {
    rest: &'a [u8],
    remaining: u16,
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

struct FrameReader<'a> {
    frame: &'a ASEFrameHeader,
    rest: &'a[u8],
}

impl<'a> FrameReader<'a> {
    fn new(data: &'a[u8]) -> Result<FrameReader<'a>, FrameParseError> {
        parse_frame(data).map(|(frame, rest)| FrameReader{frame: frame, rest})
    }

    fn size(&self) -> u32 {
        self.frame.num_bytes
    }

    fn chunks(&self) -> ChunkIterator<'a> {
        ChunkIterator {
            ptr: self.rest,
            count: self.frame.num_chunks as usize,
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

#[derive(Clone)]
pub struct ASEFrameContainer<'a> (pub &'a [u8]);

impl<'a, 'b> ASEFrameContainer<'a> {
    fn chunks(&'b self) -> ChunkIterator<'a> {
        todo!()
        // ChunkIterator {
        //     ptr: self.1,
        //     count: self.0.num_chunks as usize,
        // }
    }
}

impl<'a> Iterator for ASEFrameContainer<'a> {
    type Item = ASEFrameContainer<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.0;
        let (frame, rest)= parse_frame(ptr).unwrap();
        let resp = self.clone();

        let advance_bytes = frame.num_bytes as usize - 16;
        self.0 = &rest[advance_bytes..];

        return Some(resp)

        // if let Some((frame, rest)) = ASEFrame::ref_from_prefix(ptr)
        //     .map_err(|_| FrameParseError::CastError)
        //     .ok() {
        //         let resp = self.clone();
        //         let advance_bytes = frame.num_bytes as usize - 16;
        //         #[cfg(test)] {
        //             let fnb = frame.num_bytes;
        //             println!("fb: {}, frame: {:?}", fnb, frame);
        //             println!("Advance bytes {}", advance_bytes);
        //         }
        //         // self.1 = &rest[advance_bytes..];
        //         // self.0 = frame;
        //         Some(resp)
        //     } else {
        //         None
        //     }
    }
}


#[derive(Error, Debug)]
pub enum HeaderParseError {
    #[error("Cast error")]
    CastError,
}

fn parse_header<'a>(input: &'a [u8]) -> Result<(&'a ASEHeader, &'a [u8]), HeaderParseError> {
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
    use super::*;
    use std::boxed::Box;

    pub fn get_chunk_17<'a>() -> ASEChunkReader<'a>  {
        let a = std::fs::read("tests/anim_idle.ase").unwrap();
        let (header, rest) = parse_header(a.leak()).unwrap();

        let (fc, rest) = parse_frame(rest).unwrap();
        // let w: ChunkIterator<'a> = fc.chunks();
        // // let cptr = Box::leak(Box::new(w));
        // let cptr = w;

        // let chunk: ASEChunkContainer<'a> = cptr.get(17).unwrap();
        // chunk
        todo!()
    }

    #[test]
    fn read_frames() {
        let a = std::fs::read("tests/anim_idle.ase").unwrap();
        let (header, rest) = parse_header(&a).unwrap();

        let fc = ASEFrameContainer(rest);
        // let fc = parse_frame(rest).unwrap();

        let mut iter = fc.into_iter();

        // for frame in iter {
        //     println!("{:?}", frame.0)

        // }
        // let frame_1 = iter.next().unwrap().0;

        // let frame_2 = iter.next().unwrap().0;

        // println!("{:?}", frame_2)
        

        
        
    }

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
        let mut chunks = frame_1.chunks();
        for chunk in chunks {
            if let ASEChunk::Cel(cel) = chunk {
                let cel_data = cel.get();
                println!("{:?}", cel_data);
            } else {
                println!("{:x?}", chunk);
            }
        }
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
