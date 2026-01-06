
// use nom::{bytes::complete::take, combinator::{self, rest, success, value}, multi::{length_data, length_value}, number::complete::{le_u16, le_u32}};

// use nom_derive::*;
use alloc::vec::Vec;
use thiserror::Error;
// use zerocopy_derive::*;  
// use zerocopy::{FromBytes, KnownLayout, Immutable};  
use zerocopy::*;
// use nom::{IResult, Parser, bytes::complete::{tag, take}, number::complete::{le_u16, le_u32, u8} };
use core::convert::Infallible;
use core::iter::Iterator;
use core::ops::Index;
use core::marker::PhantomData;
use core::ops::Deref;

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

// fn parse_header(input: &[u8]) -> IResult<&[u8], MyAwesomeStruct> {
//     let (input, 
//             ( header
//             , _magic
//             , frames
//             , width
//             , height
//             , depth
//             , flags
//         )) =
//         ( le_u32
//         , tag(&b"\xE0\xA5"[..])
//         , le_u16
//         , le_u16
//         , le_u16
//         , le_u16
//         , le_u32
//     ).parse_complete(input)?;

//     Ok((input, MyAwesomeStruct {
//         header: header,
//         frames: frames,
//         width: width,
//         height: height,
//         depth: depth,
//         flags: flags,
//     }))
// }

// #[derive(NomLE, Debug)]
// pub struct ASE<'a> {
//     pub header: ASEHeader<'a>,
//     #[nom(Count="header.frames as usize")]
//     pub frames: Vec<ASEFrame<'a>>,
// }

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

// #[derive(NomLE, Debug)]
// pub struct ASEChunk<'a> {
//     pub size: u32,
//     pub chunk_type: u16,
//     // #[nom(Count="size as usize - 6")]
//     // data: Vec<u8>
//     #[nom(Parse = "(|i| parse_chunk_data(i, chunk_type, size))")]  
//     chunk_data: ChunkData<'a>,

// }

// #[derive(Debug)]
// enum ChunkData<'a> {
//     Unknown(&'a[u8]),
// }


// fn parse_chunk_data(input: &[u8], chunk_type: u16, size: u32) -> nom::IResult<&[u8], ChunkData> {
//     match chunk_type {
//         _ => {
//             // let (input, data) = combinator::rest(input)?;
//             // Ok((input, ChunkData::Unknown(data.to_vec())))
//             // take (size as usize - 6usize)(input)
//             let (input, data) = take(size as usize - 6usize)(input)?;
//             Ok((input, ChunkData::Unknown(data)))
//         }
//     }
// }

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

pub trait NextResult<'a> {
    type Output;
    type Error;
    fn next(&'a self) -> Result<(Self::Output, &'a [u8]), Self::Error>;
}

// impl<'a> NextResult<'a> for Result<(&'a ASEChunkHeader, &'a [u8]), ChunkHeaderParseError> {
//     type Output = &'a ASEChunkHeader;
//     type Error = ChunkHeaderParseError;
//     // We need to check current size, and return a new result with updated slice
//     fn next(&'a self) -> Result<(Self::Output, &'a [u8]), Self::Error> {
//         match self {
//             Ok((chunk, rest)) => {
//                 let ofs : usize = chunk.size() - 6;
//                 let next_chunk_start = &rest[ofs..];
//                 Self::parse_chunk_header(next_chunk_start)
//             },
//             Err(e) => Err(e.clone()),
//         }
//     }
// }

trait HasSize {
    fn size(&self) -> usize;
}


#[derive(Debug, Clone)]
pub struct ChunkPtr<'a> {
    ptr: &'a [u8],
    count: usize,
}

impl ChunkPtr<'_> {
    fn len(&self) -> usize {
        self.count
    }

    fn get<'a>(&'a self, index: usize) -> Option<ChunkPtr<'a>> {
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
        w.next()


        // w.next().map(|(a ,b)| {ChunkPtr {
        //     ptr: b,
        //     count: a.size as usize,
        // }})

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



    
impl<'a> Deref for ChunkPtr<'a> {
    type Target = ASEChunkHeader;
    
    fn deref (&self) -> &'a Self::Target {
        const SIZE: usize = 4;
        let chunk_ptr =&self.ptr[SIZE..];
        let (chunk_header, rest) = ASEChunkHeader::ref_from_prefix(&chunk_ptr).unwrap();
        ChunkCel::
        

        // UnparsedChunk::r
    }
}


// impl<'a> Mwahaha<'a, &'a ASEChunkHeader, ChunkHeaderParseError> for ChunkPtr<'a, &'a ASEChunkHeader, ChunkHeaderParseError> {
//     fn parse_chunk_header(input: &'a [u8]) -> Result<(&'a ASEChunkHeader, &'a [u8]), ChunkHeaderParseError> {
//         let (chunk, rest) = ASEChunkHeader::ref_from_prefix(&input)
//             .map_err(|_| ChunkHeaderParseError::CastError)?;

//         Ok((chunk, rest))
//     }
// }
    

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
    // use compression::prelude::{EncodeExt, ZlibEncoder};

    use super::*;
    // #[test]
    // fn test_parse_u32() {
    //     let data = [0x12, 0x34, 0x56, 0x78, 0x9A];
    //     let (remaining, value) = parse_header(&data).unwrap();

    //     println!("{:?}", value);
    // }

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
        let w = c.get(2).unwrap();
        let x = &w[0..10];
        println!("{:#x?}", w.chunk);


    }
}
