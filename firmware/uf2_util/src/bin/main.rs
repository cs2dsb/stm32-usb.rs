use std::{
    path::PathBuf,
    io::Write,
    fs::{ self, File },
    env,
};
use structopt::StructOpt;
use clap::arg_enum;
use failure::Fallible;
use env_logger;
use log::*;
use uf2_util::{ convert_elf, convert_bin };

arg_enum! {
    #[derive(Debug, PartialEq)]
    enum InputType {
        Bin,
        Elf,
    }
}

fn parse_hex_32(input: &str) -> Result<u32, std::num::ParseIntError> {
    if input.starts_with("0x") {
        u32::from_str_radix(&input[2..], 16)
    } else {
        input.parse::<u32>()
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "uf2_util", about = "A utility for converting to & from UF2")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(short, long, default_value = "bin")]
    input_type: InputType,

    #[structopt(short, long, parse(try_from_str = parse_hex_32))]
    base_address: Option<u32>,

    #[structopt(short, long, default_value = "256")]
    page_size: u16,
}

fn main() -> Fallible<()> {
    if env::var(env_logger::DEFAULT_FILTER_ENV).is_err() {
        // Set the default logging verbosity
        env::set_var(
            env_logger::DEFAULT_FILTER_ENV, 
            "info",
        );
    }
    env_logger::init();

    let opt = Opt::from_args();

    let page_size = opt.page_size;

    if opt.input_type == InputType::Bin && opt.base_address.is_none() {
        panic!("base_address must be provided if input_type is bin");
    }

    let out_path = opt.output.unwrap_or(opt.input.with_extension("uf2"));
    let data = fs::read(opt.input)?;

    info!("Type: {:?}", opt.input_type);
    info!("Output: {:?}", out_path);
    debug!("Base address: {:?}", opt.base_address);

    let mut out = File::create(out_path)?;

    let bytes = match opt.input_type {
        InputType::Elf => convert_elf(&data, page_size)?,
        InputType::Bin => convert_bin(&data, page_size, opt.base_address.unwrap())?,
    };

    out.write(&bytes)?;

    Ok(())
}