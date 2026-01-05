
// use nom::{bytes::complete::take, combinator::{self, rest, success, value}, multi::{length_data, length_value}, number::complete::{le_u16, le_u32}};

// use nom_derive::*;
use alloc::vec::Vec;
// use zerocopy_derive::*;  
// use zerocopy::{FromBytes, KnownLayout, Immutable};  
use zerocopy::*;
// use nom::{IResult, Parser, bytes::complete::{tag, take}, number::complete::{le_u16, le_u32, u8} };

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
//     // #[nom(Count="num_chunks")]
//     // pub chunks: Vec<ASEChunk<'a>>,
//     pub chunks: [ASEChunk<'a>; 200],
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


fn parse_header<'a>(input: &'a [u8]) -> Result<(&'a ASEHeader, &'a [u8]), CastError<&'a[u8], ASEHeader>> {
    let (header, rest) = ASEHeader::ref_from_prefix(&input)?;

    Ok((header, rest))
}

fn parse_frames<'a>(header: &'a ASEHeader, input: &'a [u8]) -> Result<(&'a ASEFrame, &'a [u8]), CastError<&'a [u8], ASEFrame>> {
    // let (frames, rest) = ASEFrame::ref_from_prefix(&input)?;
    let (frame, rest) = ASEFrame::ref_from_prefix(&input)?;

    Ok((frame, rest))
}

pub fn testparse(input: &[u8]) -> u16 {
    let (header, rest) = parse_header(input).unwrap();
    let (frame, rest) = parse_frames(header, rest).unwrap();
    frame._magic

}

// pub fn parse_header(input: &'_ [u8]) -> nom::IResult<&'_ [u8], ASEHeader<'_>> {
//     ASEHeader::parse(input)
// }

// pub fn parse_aseprite(input: &[u8]) -> nom::IResult<&[u8], ASE> {
//     let mut a = (
//         ASEHeader::parse,
//     );
//     let b = a.parse(input);




//     ASE::parse(input)
// }


#[cfg(test)]
mod test {
    use compression::prelude::{EncodeExt, ZlibEncoder};
    use nom::error::dbg_dmp;

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

        let (frame , rest) = parse_frames(header, rest).unwrap();

        // let (rest, ase) = ASE::parse(&a).unwrap();
        // println!("{:#x?}, rest: {}", ase, rest.len());
        println!("{:#x?}", frame);


    }
}
