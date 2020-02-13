
use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;

use uf2::Block as Uf2Block;

#[allow(unused_imports)]
use itm_logger::*;

const BLOCK_SIZE: usize = 512;
const NUM_FAT_BLOCKS: u32 = 8000;
const RESERVED_SECTORS: u32 = 1;
const ROOT_DIR_SECTORS: u32 = 4;
const SECTORS_PER_FAT: u32 = (NUM_FAT_BLOCKS * 2 + 511) / 512;
const START_FAT0: u32 = RESERVED_SECTORS;
const START_FAT1: u32 = START_FAT0 + SECTORS_PER_FAT;
const START_ROOTDIR: u32 = START_FAT1 + SECTORS_PER_FAT;
const START_CLUSTERS: u32 = START_ROOTDIR + ROOT_DIR_SECTORS;
const UF2_SIZE: u32 = 0x10000 * 2;
const UF2_SECTORS: u32 = UF2_SIZE / (BLOCK_SIZE as u32);



#[derive(Clone, Copy, PackedStruct, Default)]
#[packed_struct(endian="lsb")]
pub struct DirectoryEntry {
    pub name: [u8; 11],
    /*
        pub name: [u8; 8],
        pub ext: [u8; 3],
    */
    pub attrs: u8,
    _reserved: u8,
    pub create_time_fine: u8,
    pub create_time: u16,
    pub create_date: u16,
    pub last_access_date: u16,
    pub high_start_cluster: u16,
    pub update_time: u16,
    pub update_date: u16,
    pub start_cluster: u16,
    pub size: u32,
}


#[derive(Clone, Copy, PackedStruct)]
#[packed_struct(endian="lsb")]
pub struct FatFile {
    pub name: [u8; 11],
    pub content: [u8; 255],
}

pub fn fat_files() -> [FatFile; 2] {
    let mut info = FatFile { name: [0x20; 11], content: [0x20; 255] };
    let mut index = FatFile { name: [0x20; 11], content: [0x20; 255] };

    info.name.copy_from_slice("INFO_UF2TXT".as_bytes());
    info.content[..60].copy_from_slice("UF2 Bootloader 1.2.3\r\nModel: BluePill\r\nBoard-ID: spunk_123\r\n".as_bytes());
    
    index.name.copy_from_slice("INDEX   HTM".as_bytes());
    index.content[..90].copy_from_slice("<!doctype html>\n<html><body><script>\nlocation.replace(INDEX_URL);\n</script></body></html>\n".as_bytes());

    [info, index]
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="lsb")]
pub struct FatBootBlock {
    pub jump_instruction: [u8; 3],
    pub oem_info: [u8; 8],
    pub sector_size: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fat_copies: u8,
    pub root_directory_entries: u16,
    pub total_sectors16: u16,
    pub media_descriptor: u8,
    pub sectors_per_fat: u16,
    pub sectors_per_track: u16,
    pub heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors32: u32,
    pub physical_drive_num: u8,
    _reserved: u8,
    pub extended_boot_sig: u8,
    pub volume_serial_number: u32,
    pub volume_label: [u8; 11],
    pub filesystem_identifier: [u8; 8],
}

pub fn fat_boot_block() -> FatBootBlock {
    const RESERVED_SECTORS: u16 = 1;
    const ROOT_DIR_SECTORS: u16 = 4;
    const NUM_FAT_BLOCKS: u16 = 8000;
    const SECTORS_PER_FAT: u16 = (NUM_FAT_BLOCKS * 2 + 511) / 512;
    let mut fat = FatBootBlock {
        jump_instruction: [0xEB, 0x3C, 0x90],
        oem_info: [0x20; 8],
        sector_size: 512,
        sectors_per_cluster: 1,
        reserved_sectors: RESERVED_SECTORS,
        fat_copies: 2,
        root_directory_entries: (ROOT_DIR_SECTORS * 512 / 32),
        total_sectors16: NUM_FAT_BLOCKS - 2,
        media_descriptor: 0xF8,
        sectors_per_fat: SECTORS_PER_FAT,
        sectors_per_track: 1,
        heads: 1,
        hidden_sectors: 0,
        total_sectors32: NUM_FAT_BLOCKS as u32 - 1,
        physical_drive_num: 0,
        _reserved: 0,
        extended_boot_sig: 0x29,
        volume_serial_number: 0x00420042,
        volume_label: [0x20; 11],
        filesystem_identifier: [0x20; 8],
    };
    fat.oem_info[..7].copy_from_slice("UF2 UF2".as_bytes());
    fat.volume_label[..8].copy_from_slice("BLUEPILL".as_bytes());
    fat.filesystem_identifier[..5].copy_from_slice("FAT16".as_bytes());

    fat
}


/// # Dummy fat implementation that provides a [UF2 bootloader](https://github.com/microsoft/uf2)
pub struct GhostFat {
    fat_boot_block: FatBootBlock,
    fat_files: [FatFile; 2],
}

impl GhostFat {
    pub fn new() -> GhostFat {
        GhostFat {
            fat_boot_block: fat_boot_block(),
            fat_files: fat_files(),
        }
    }

    pub fn read_block(&self, lba: u32, block: &mut [u8]) {
        assert_eq!(block.len(), BLOCK_SIZE);

        info!("GhostFAT reading block: 0x{:X?}", lba);

        // Clear the buffer since we're sending all of it
        for b in block.iter_mut() { *b = 0 }
   
        if lba == 0 {
            // Block 0 is the fat boot block
            let packed = self.fat_boot_block.pack();
            block[..packed.len()].copy_from_slice(&packed);
            block[510] = 0x55;
            block[511] = 0xAA;
        } else if lba < START_ROOTDIR {
            let mut section_index = lba - START_FAT0;

            if section_index >= SECTORS_PER_FAT {
                section_index -= SECTORS_PER_FAT;
            }

            if section_index == 0 {
                block[0] = 0xF0;
                for i in 1..(self.fat_files.len() * 2 + 4) {
                    block[i] = 0xFF;
                }
            }
         
            let uf2_first_sector = self.fat_files.len() + 1;
            let uf2_last_sector = uf2_first_sector + UF2_SECTORS as usize - 1;
            
            for i in 0..256_usize {
                let v = section_index as usize * 256 + i;
                let j = 2 * i;
                if v >= uf2_first_sector && v < uf2_last_sector {
                    block[j+0] = (((v + 1) >> 0) & 0xFF) as u8;
                    block[j+1] = (((v + 1) >> 8) & 0xFF) as u8;
                } else if v == uf2_last_sector {
                    block[j+0] = 0xFF;
                    block[j+1] = 0xFF;
                }
            }            
        } else if lba < START_CLUSTERS {
            let section_index = lba - START_ROOTDIR;            
            if section_index == 0 {
                let mut dir = DirectoryEntry::default();
                dir.name.copy_from_slice(&self.fat_boot_block.volume_label);
                dir.attrs = 0x28;
                let packed = dir.pack();
                let len = packed.len();
                block[..len].copy_from_slice(&packed);
                dir.attrs = 0;
                for (i, info) in self.fat_files.iter().enumerate() {
                    dir.size = info.content.len() as u32;
                    dir.start_cluster = i as u16 + 2;
                    dir.name.copy_from_slice(&info.name);
                    let start = (i+1) * len;
                    block[start..(start + len)].copy_from_slice(&dir.pack());
                }
            }
        } else {
            let section_index = (lba - START_CLUSTERS) as usize;
            if section_index < self.fat_files.len() {
                let info = self.fat_files[section_index];
                block[..info.content.len()].copy_from_slice(&info.content);
            } else {
                //UF2
            }
        }
    }

    pub fn write_block(&mut self, lba: u32, block: &[u8]) {
        info!("GhostFAT writing block: 0x{:X?}", lba);

        if lba < (START_CLUSTERS + self.fat_files.len() as u32) {
            info!("    GhostFAT skipping non-UF2 area");
            return;
        }

        if block.len() < Uf2Block::BYTES {
            warn!("    GhostFAT attempt to write to UF2 area with < 512 byte block");
            return;
        }
        assert_eq!(block.len(), Uf2Block::BYTES);
        
        let uf2 = if let Ok(uf2) = Uf2Block::parse(block) {
            uf2
        } else {
            warn!("   GhostFAT failed to parse as UF2");
            return;
        };

        if !uf2_family_is_correct(&uf2) {
            warn!("   GhostFAT UF2 family id was wrong");
            return;
        }

        info!("   GhostFAT writing {} bytes of UF2 block at 0x{:X?}", uf2.payload_size, lba);          
    }
}

fn uf2_family_is_correct(_uf2: &Uf2Block) -> bool {
    true
}