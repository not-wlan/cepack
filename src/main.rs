mod cearchive;
mod errors;

#[macro_use]
extern crate quick_error;
extern crate byteorder;

use crate::cearchive::CEArchive;
use crate::errors::UnpackError;

use pelite::pe32::{Pe, PeFile};
use pelite::resources::*;
use pelite::FileMap;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str;

/// The magic bytes used by CE Trainers
static TRAINER_MAGIC: &str = "CHEAT";
static TRAINER_FILE: &str = "CET_TRAINER.CETRAINER";

/// "ARCHIVE" in 16 Bit wide chars
const ARCHIVE_NAME: &str = "ARCHIVE";
/// "DECOMPRESSOR" in 16 Bit wide chars
const DECOMPRESSSOR_NAME: &str = "DECOMPRESSOR";
/// ID of the resource type RCDATA ( https://docs.microsoft.com/en-us/windows/desktop/menurc/resource-types )
const RT_RCDATA: Name = Name::Id(10);

fn main() {
    if let Some(arg) = env::args().nth(1) {
        match run(&arg) {
            Err(e) => {
                println!("[-] {}", e);
            }
            _ => {
                println!("[+] success!");
            }
        }
    } else {
        println!("usage: cepack <file>");
    }
}

fn run(filename: &str) -> Result<(), UnpackError> {
    let path = Path::new(&filename);
    let map = FileMap::open(path)?;
    let file = PeFile::from_bytes(&map)?;
    let resources = file.resources()?;

    println!("[+] attempting to unpack \"{}\"", filename);

    // If a "DECOMPRESSOR" Resource isn't present the tiny mode was used. That means that the trainer can be unpacked as is.
    let archive_resource = resources.find_resource(RT_RCDATA, Name::from(ARCHIVE_NAME))?;
    let mut data  = match resources.find_resource(RT_RCDATA, Name::from(DECOMPRESSSOR_NAME)) {
        Ok(_) => {
            CEArchive::new(archive_resource)?
                .files
                .iter()
                .find(|f| f.name.as_str() == TRAINER_FILE)
                .ok_or(UnpackError::InvalidArchive)?
                .data
                .clone()
        },
        Err(_) => archive_resource.to_vec(),
    };

    println!("[+] found archive! length: {:#X}", data.len());

    // Cheat Engine's "protection"
    // https://github.com/cheat-engine/cheat-engine/blob/master/Cheat%20Engine/OpenSave.pas#L1406
    for i in 2..data.len() {
        data[i] ^= data[i - 2];
    }
    for i in (0..data.len() - 1).rev() {
        data[i] ^= data[i + 1];
    }
    let mut ckey = 0xCEu8;
    for i in data.iter_mut() {
        *i ^= ckey;
        ckey = ckey.wrapping_add(1);
    }

    // Every new Trainer starts with "CHEAT" as the first 5 bytes
    if &data[0..5] == TRAINER_MAGIC.as_bytes() {
        println!("[+] matched trainer signature!");
    } else {
        return Err(UnpackError::BadMagic);
    }

    let raw_trainer = inflate::inflate_bytes(&data[5..]).or(Err(UnpackError::ZlibError))?;

    // Save the resulting .xml in <trainer name>.xml
    let filename = path
        .file_name()
        .ok_or(UnpackError::BadFilename)?
        .to_str()
        .ok_or(UnpackError::BadFilename)?;
    let out_filename = format!("{}.xml", filename);

    println!("[+] writing result to \"{}\"", out_filename);

    let out_path = Path::new(out_filename.as_str());
    let mut out_file = File::create(&out_path)?;
    // TODO: find out why there's 4 byte of garbage at the beginning
    out_file.write_all(&raw_trainer[4..])?;

    Ok(())
}
