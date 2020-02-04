use std::{
    path::PathBuf,
    io::Write,
    fs::{ self, File },
    env,
};
use structopt::StructOpt;
use clap::arg_enum;
use goblin::elf::{
    Elf,
    program_header::PT_LOAD,
};
use failure::Fallible;
use uf2::DATA_LENGTH;
use env_logger;
use log::*;
use uf2::{ Block, Error };
use log::trace;

fn blockify(base_address: u32, block_size: usize, data: &[u8]) -> Fallible<Vec<Block>> {
    let res: Result<Vec<_>, _> = data
        .chunks(block_size)
        .enumerate()
        .map(|(i, chunk)| Block::new(
                base_address + (i * block_size) as u32,
                chunk,
            )
        )
        .collect();
    res
}

fn finalize(blocks: Vec<Block>) -> Vec<u8> {
    let n = blocks.len() as u32;
    blocks.into_iter().enumerate().flat_map(|(i, mut b)| {
        b.block_number = i as u32;
        b.number_of_blocks = n;
        trace!("{}/{}: 0x{:X?} {}", i, n, b.target_address, b.payload_size);
        b.pack().to_vec()
    }).collect()
}

fn block_size(page_size: u16) -> Fallible<usize> {
    let page_size = page_size as usize;
    if page_size > DATA_LENGTH {
        Err(Error::DataTooLong)?
    }
    Ok((DATA_LENGTH / page_size) * page_size)
}

/// Parses provided bytes as an ELF file and converts contained PT_LOAD segments
/// into UF2 blocks. Will fail if bytes aren't a valid ELF file
pub fn convert_elf(data: &[u8], page_size: u16) -> Fallible<Vec<u8>> {
    let block_size = block_size(page_size)?;
    let mut blocks = Vec::new();
    for header in Elf::parse(&data)?.program_headers {
        let length = header.p_filesz as usize;
        let start = header.p_offset as usize;
        if header.p_type == PT_LOAD && length > 0 {
            blocks.extend(blockify(
                header.p_paddr as u32,
                block_size,
                &data[start..(start+length)],
            )?);
        }
    }
    Ok(finalize(blocks))
}

/// Converts provided bytes into UF2 blocks assuming bytes are a BIN file
/// No checking on the data is performed
pub fn convert_bin(data: &[u8], page_size: u16, base_address: u32) -> Fallible<Vec<u8>> {
    let block_size = block_size(page_size)?;
    let blocks = blockify(
        base_address,
        block_size,
        &data,
    )?;
    Ok(finalize(blocks))
}