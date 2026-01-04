
use nom::number::complete::{le_u16};

use nom_derive::*;
use alloc::vec::Vec;
// use nom::{IResult, Parser, bytes::complete::{tag, take}, number::complete::{le_u16, le_u32, u8} };

#[derive(Debug)]
#[derive(NomLE)]
pub struct ASEHeader<'a> {
    pub header: u32,
    #[nom(Tag(b"\xE0\xA5"))]
    _magic: &'a[u8],
    pub frames: u16,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub flags: u32,
    _deprecated_speed: u16,
    _reserved: u16,
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

#[derive(NomLE, Debug)]
struct ASE<'a> {
    header: ASEHeader<'a>,
    #[nom(Count="header.frames")]
    frames: Vec<ASEFrame>,

}
#[derive(NomLE, Debug)]
struct ASEFrame {
    duration: u16,
    // chunks: Vec<ASEChunk<'a>>,
}


pub fn parse_header(input: &'_ [u8]) -> nom::IResult<&'_ [u8], ASEHeader<'_>> {
    ASEHeader::parse(input)
}

fn parse_aseprite(input: &[u8]) -> nom::IResult<&[u8], ASE> {
    ASE::parse(input)
}


#[cfg(test)]
mod test {
    use compression::prelude::{EncodeExt, ZlibEncoder};

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

        let (_, ase) = ASE::parse(&a).unwrap();
        println!("{:?}", ase);


    }
}
