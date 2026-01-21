use core::{fmt::{Debug, Formatter, Result}, marker::PhantomData, ops::Deref};

use thiserror::Error;
use zerocopy::*;
use bitflags::bitflags;

#[derive(Debug)]
pub struct Layer<'a> {
    pub header: &'a LayerHeader,
    pub name: &'a str,
    rest: &'a [u8],
}

bitflags! {
    #[derive(Debug)]
    pub struct LayerFlag : u16 {
        const VISIBLE = 0x1;
        const EDITABLE = 0x2;
        const LOCK_MOVEMENT = 0x4;
        const BACKGROUND = 0x8;
        const PREFER_LINKED_CELS = 0x10;
        const DISPLAY_COLLAPSED = 0x20;
        const REFERENCE = 0x40;
    }
}

#[derive(FromBytes, Immutable, Clone, Unaligned)]
#[repr(transparent)]
struct LayerFlags([u8; 2]);
impl From<&LayerFlags> for LayerFlag {
    fn from(value: &LayerFlags) -> Self {
        LayerFlag::from_bits_truncate(u16::from_le_bytes(value.0))
    }
}

impl Debug for LayerFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let bits: LayerFlag = self.into();
        write!(f, "{:?}", bits)
    }
}

#[repr(u16)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable)]
pub enum LayerType {
    Normal = 0,
    Group = 1,
    Tilemap = 2,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, TryFromBytes, KnownLayout, Immutable)]
pub enum BlendMode {
    Normal = 0,
    Multiply = 1,
    Screen = 2,
    Overlay = 3,
    Darken = 4,
    Lighten = 5,
    ColorDodge = 6,
    ColorBurn = 7,
    HardLight = 8,
    SoftLight = 9,
    Difference = 10,
    Exclusion = 11,
    Hue = 12,
    Saturation = 13,
    Color = 14,
    Luminosity = 15,
    Addition = 16,
    Subtract = 17,
    Divide = 18,
}

#[derive(TryFromBytes, KnownLayout, Immutable, Unaligned)]  
#[repr(transparent)]  
struct RawBlend([u8; 2]);

// TODO: use try into
impl From<&RawBlend> for BlendMode {
    fn from(value: &RawBlend) -> Self {
        Self::try_read_from_bytes(&value.0).unwrap()
    }
}

impl Debug for RawBlend {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let blend_mode: BlendMode = self.into();
        write!(f, "{:?}", blend_mode)
    }
}


#[derive(Debug, TryFromBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
pub struct LayerHeader {
    flags: LayerFlags,
    layer_type: U16<LE>,
    child_level: U16<LE>,
    _ignored_width: U16<LE>,
    _ignored_height: U16<LE>,
    blend_mode: RawBlend,
    opacity: u8,
    _reserved: [u8; 3],
}

impl<'a> Layer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let (layer, rest) = LayerHeader::try_ref_from_prefix(data).unwrap();
        let (name_len, rest) = U16::<LE>::ref_from_prefix(rest).unwrap();
        let name_len: usize = name_len.get() as _;
        let layer_name = str::from_utf8(&rest[..name_len]).unwrap();
        Layer { header: layer, name: layer_name, rest: &rest[name_len..] }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::{HeaderReader, chunk::ASEChunk};

    #[test]
    fn layer() {
        let v = std::fs::read("tests/anim_idle.ase").unwrap();
        let data: &[u8] = &v;

        let r = HeaderReader::new(data);
        let mut frames = r.frames();
        let frame_1 = frames.next().unwrap();

        for chunk in frame_1.chunks() {
            println!("{:?}", chunk);
        }
        let ASEChunk::Layer(layer) = frame_1.chunks().find(|c| 
            matches!(c, ASEChunk::Layer(_))
        ).unwrap() else {panic!("No layer chunk found")};

        let bl: BlendMode = (&layer.header.blend_mode).into();
        println!("{:?}", bl);
    }
}