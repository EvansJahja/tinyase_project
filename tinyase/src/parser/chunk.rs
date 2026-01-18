use zerocopy::*;

#[derive(Debug)]
pub enum ASEChunk<'a> {
    Unknown(&'a[u8]),
}

impl<'a> ASEChunk<'a> {
    pub(super) fn new(chunk_type: u16, ptr: &'a[u8]) -> Self {
        match chunk_type {
            // 0x2005 => chunk_cel(ptr),
            _ => ASEChunk::Unknown(ptr)
        }
    }
}

#[derive(Debug, Unaligned, TryFromBytes, KnownLayout, Immutable)]
#[repr(packed)]
pub struct CelHeader {
    layer_index: u16,
    point_x: i16,
    point_y: i16,
    opacity: u8,
    cel_type: u16,
    z_index: i16,
    _unused_1: [u8; 5],
}

fn chunk_cel<'a>(ptr: &'a[u8]) -> CelContainer<'a>{
    let (h, p) = CelHeader::try_ref_from_prefix(ptr).unwrap();
    CelContainer { cel_header: h, ptr: p }
}

struct CelContainer<'a> {
    cel_header: &'a CelHeader,
    ptr: &'a [u8],
}

#[derive(Debug)]
enum CelData<'a> {
    Raw(RawImageDataContainer<'a>),
    Linked(u32),
    
}

impl<'a> CelContainer<'a> {
    fn get(&self) -> CelData<'a> {
        let header = self.cel_header;
        let cel_type = header.cel_type;
        match cel_type {
            0 => {
                let a = RawImageHeader::ref_from_prefix(self.ptr).unwrap();
                let b = RawImageDataContainer{header: a.0, ptr: a.1};
                CelData::Raw(b)
            },
            1 => {
                let (z, _) = u32::ref_from_prefix(self.ptr).unwrap();
                CelData::Linked(*z)
            }

            _ => panic!("Unsupported cel_type: {}", cel_type)
        }
    }
}

#[derive(Debug, FromBytes, Immutable, KnownLayout)]
#[repr(packed)]
struct RawImageHeader {
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct RawImageDataContainer<'a> {
    header: &'a RawImageHeader,
    ptr: &'a [u8],
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
        
        println!("{:#x?}", cd);

    }
}