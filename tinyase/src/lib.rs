#![cfg_attr(not(test), no_std)]

#![warn(
    // missing_docs,
    non_camel_case_types,
    non_snake_case,
    path_statements,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_allocation,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    while_true,
    clippy::panic,
    clippy::print_stdout,
    clippy::todo,
    //clippy::unwrap_used, // not yet in stable
    clippy::wrong_pub_self_convention
)]

pub mod parser;
pub use parser::ASEHeader;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

//     #[test]
//     fn test_decompress() {
//         use compression::prelude::*;
//             //         let data: [u8; 0x33] = [
//             //     0x78, 0x9C, 0x5D, 0x8C, 0x31, 0x12, 0x00, 0x30, 0x08, 0xC2, 0x08, 0xFF, 0x7F, 0x74, 0x39, 0xD4, 
//             //     0xA5, 0x0C, 0x8A, 0x68, 0x14, 0xA0, 0x0A, 0x6C, 0xD4, 0x29, 0x99, 0x77, 0x4A, 0x1D, 0xB5, 0xCF, 
//             //     0x0E, 0xCD, 0x99, 0x6A, 0x0E, 0x0F, 0xC1, 0x3E, 0xBA, 0x30, 0xB4, 0x7F, 0xB7, 0xC4, 0x03, 0x15, 
//             //     0x89, 0x00, 0x54, 
//             // ];
//             let data: [u8; 0x11] = [
//     0x78, 0x9C, 0x63, 0x60, 0x18, 0x05, 0xA3, 0x60, 0x14, 0x8C, 0x54, 0x00, 0x00, 0x04, 0x00, 0x00, 
//     0x01, 
// ];

//             // let compressed = b"aabbaabbaabbaabb\n"
//             // let decompressed = no_inflate::inflate_zlib(&data).unwrap();
            
//             // let decompressed = data
//             //     .into_iter()
//             //     .decode(&mut ZlibDecoder::new())
//             //     .collect::<Result<Vec<_>, _>>()
//             //     .unwrap();

//         println!("{:?}", decompressed);

//     }
}
