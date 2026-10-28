use zerocopy::{FromBytes, Immutable, KnownLayout, TryFromBytes, Unaligned, little_endian::U16};

use crate::parser::prefix_string;


#[derive(TryFromBytes, KnownLayout, Immutable, Debug, Unaligned)]
#[repr(C)]
pub struct TagInner {
    pub from: U16,
    pub to: U16,
    pub loop_dir: u8,
    pub repeat_times: U16,
    _res_1: [u8; 6],
    pub tag_color: [u8;3],
    _res_2: u8,
}


#[derive(Debug)]
pub struct Tag<'a> {
    pub tag_inner: &'a TagInner,
    pub name: &'a str,
}

#[derive(Debug)]
pub struct TagIterator<'a> {
    ptr: &'a[u8],
    remaining: u16,
}

impl<'a> TagIterator<'a> {
    pub fn new(ptr: &'a [u8]) -> Self {
        let (count, ptr) = U16::ref_from_prefix(ptr).unwrap();
        let (_reserved, ptr) = <[u8;8]>::ref_from_prefix(ptr).unwrap();

        let count = count.get();

        TagIterator {
            ptr,
            remaining: count,
        }
    }
}

impl<'a> Iterator for TagIterator<'a> {
    type Item = Tag<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None
        }

        self.remaining = self.remaining - 1;

        let ptr = self.ptr;

        let (tag_inner, ptr) = TagInner::try_ref_from_prefix(ptr).unwrap();
        let (tag_name, ptr) =  prefix_string(ptr);
        let tag = Tag {tag_inner, name: tag_name};    
        self.ptr = ptr;


        Some(tag)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining as _, Some(self.remaining as _))
    }
}


#[cfg(test)]
mod test {
    use std::println;

use crate::parser::{AsepriteFile, chunk::{ASEChunk, ASEChunkReader}};

    #[test]
    pub fn read_tags() {
        let bytes = std::fs::read("tests/anim_idle.ase").unwrap();
        let header_reader = AsepriteFile::new(&bytes);
        let frame_1 = header_reader.frames().next().unwrap();
        let chunk_iter = frame_1.chunks();
        for c in chunk_iter {
            if let ASEChunk::Tags(mut tags) = c {
                let tag0 = tags.nth(1).unwrap();
                println!("{:?}", tag0);
            }
        }
}
}