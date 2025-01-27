use redox::*;

use super::from_bytes::FromBytes;

const MZAP_ENT_LEN: usize = 64;
const MZAP_NAME_LEN: usize = MZAP_ENT_LEN - 8 - 4 - 2;

#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum ZapObjectType {
    Micro = (1 << 63) + 3,
    Header = (1 << 63) + 1,
    Leaf = 1 << 63,
}

/// Microzap
#[repr(packed)]
pub struct MZapPhys {
    pub block_type: ZapObjectType, // ZapObjectType::Micro
    pub salt: u64,
    pub norm_flags: u64,
    pad: [u64; 5],
    pub chunk: [MZapEntPhys; 1],
    // actually variable size depending on block size
}

impl FromBytes for MZapPhys {
    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() >= mem::size_of::<MZapPhys>() {
            let mzap_phys = unsafe { ptr::read(data.as_ptr() as *const MZapPhys) };
            Some(mzap_phys)
        } else {
            Option::None
        }
    }
}

impl fmt::Debug for MZapPhys {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "MZapPhys {{\nblock_type: {:?},\nsalt: {:X},\nnorm_flags: {:X},\nchunk: [\n",
                    self.block_type, self.salt, self.norm_flags));
        for chunk in &self.chunk {
            try!(write!(f, "{:?}", chunk));
        }
        try!(write!(f, "] }}\n"));
        Ok(())
    }
}

#[repr(packed)]
pub struct MZapEntPhys{
    pub value: u64,
    pub cd: u32,
    pub pad: u16,
    pub name: [u8; MZAP_NAME_LEN],
}

impl fmt::Debug for MZapEntPhys {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "MZapEntPhys {{\nvalue: {:X},\ncd: {:X},\nname: ",
                    self.value, self.cd));
        for i in 0..MZAP_NAME_LEN {
            if self.name[i] == 0 {
                break;
            }
            try!(write!(f, "{}", self.name[i] as char));
        }
        try!(write!(f, "\n}}\n"));
        Ok(())
    }
}

/// Fatzap
#[repr(packed)]
pub struct ZapPhys {
    pub block_type: ZapObjectType, // ZapObjectType::Header
    pub magic: u64,
    pub ptr_table: ZapTablePhys,
    pub free_block: u64,
    pub num_leafs: u64,
    pub num_entries: u64,
    pub salt: u64,
    pub pad: [u64; 8181],
    pub leafs: [u64; 8192],
}

#[repr(packed)]
pub struct ZapTablePhys {
     pub block: u64,
     pub num_blocks: u64,
     pub shift: u64,
     pub next_block: u64,
     pub block_copied: u64,
}

const ZAP_LEAF_MAGIC: u32 = 0x2AB1EAF;
const ZAP_LEAF_CHUNKSIZE: usize = 24;

// The amount of space within the chunk available for the array is:
// chunk size - space for type (1) - space for next pointer (2)
const ZAP_LEAF_ARRAY_BYTES: usize = ZAP_LEAF_CHUNKSIZE - 3;

/*pub struct ZapLeafPhys {
    pub header: ZapLeafHeader,
    hash: [u16; ZAP_LEAF_HASH_NUMENTRIES],
    union zap_leaf_chunk {
        entry,
        array,
        free,
    } chunks[ZapLeafChunk; ZAP_LEAF_NUMCHUNKS],
}*/

#[repr(packed)]
pub struct ZapLeafHeader {
    pub block_type: ZapObjectType, // ZapObjectType::Leaf
    pub next: u64,
    pub prefix: u64,
    pub magic: u32,
    pub n_free: u16,
    pub n_entries: u16,
    pub prefix_len: u16,
    pub free_list: u16,
    pad2: [u8; 12],
}

#[repr(packed)]
struct ZapLeafEntry {
    leaf_type: u8,
    int_size: u8,
    next: u16,
    name_chunk: u16,
    name_length: u16,
    value_chunk: u16,
    value_length: u16,
    cd: u16,
    pad: [u8; 2],
    hash: u64,
}

#[repr(packed)]
struct ZapLeafArray {
    leaf_type: u8,
    array: [u8; ZAP_LEAF_ARRAY_BYTES],
    next: u16,
}

#[repr(packed)]
struct ZapLeafFree{
    free_type: u8,
    pad: [u8; ZAP_LEAF_ARRAY_BYTES],
    next: u16,
}
