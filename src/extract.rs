//! This function is designed to extract the component parts of a GOG Linux installer script. It is almost an exact port of the implementation found [here](https://github.com/Yepoleb/gogextract), and the ideas belong to him. Thanks for the original implementation.
use std::fs::*;
use std::fs;
use std::path::*;
use crate::error::*;
use std::io::Read;
use std::iter::FromIterator;
use regex::*;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::io::SeekFrom::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
pub struct ToExtract {
    pub unpacker: bool,
    pub mojosetup: bool,
    pub data: bool
}
/// When given a file descriptor for a GOG Game installer and output directory, extracts out the different parts of it as unpacker.sh, mojosetup.tar.gz, and data.zip.
pub fn extract(in_file: &mut File, out_string: String, extract: ToExtract) -> Result<()> {
    let out_dir = Path::new(&out_string);
    let filesize_reg = Regex::new(r#"filesizes="(\d+)"#).unwrap();
    let offset_reg = Regex::new(r"offset=`head -n (\d+)").unwrap();
    fs::create_dir_all(out_dir);
    let mut beg_buf : [u8; 10240] = [0; 10240];
    let beginning = in_file.read_exact(&mut beg_buf)?;
    let vec_beg : Vec<u8> = Vec::from_iter(beg_buf.iter().map(|x| {
        *x
    }));
    let mut buf_in_file = BufReader::new(in_file);
    buf_in_file.seek(Start(0))?;
    let beg_string = String::from_utf8(vec_beg).unwrap();
    let offset_string = offset_reg.captures(&beg_string).unwrap()[1].to_string();
    println!("{}", offset_string);
    let lines : i64 = offset_string.parse().unwrap();
    println!("{}", lines);
    let mut dump = String::new();
    let mut script_size : u64 = 0;
    for i in 0..lines {
        script_size += buf_in_file.read_line(&mut dump)? as u64;
    }
    println!("Script size: {}", script_size);
    buf_in_file.seek(Start(0))?;
    let mut script_bin = vec![0u8; script_size as usize];
    buf_in_file.read_exact(&mut script_bin)?;
    let unpacker_name = out_dir.clone().join("unpacker.sh").as_path().to_owned();
    if extract.unpacker {
        let mut unpacker_fd = File::create(unpacker_name)?;
    }
    let mut perms = unpacker_fd.metadata()?.permissions();
    perms.set_mode(0o744);
    unpacker_fd.set_permissions(perms)?;
    unpacker_fd.write_all(&script_bin)?;
    let script = String::from_utf8(script_bin).unwrap();
    let filesize  = filesize_reg.captures(&script).unwrap()[1].to_string().parse().unwrap();
    println!("MojoSetup size: {}", filesize);
    let mut mojo = vec![0u8; filesize];
    buf_in_file.seek(Start(script_size))?;
    buf_in_file.read_exact(&mut mojo)?;
    let archive_name = out_dir.clone().join("mojosetup.tar.gz").as_path().to_owned();
    if extract.mojosetup {
        let mut archive_fd = File::create(archive_name)?;
    }
    archive_fd.write_all(&mojo)?;
    let dataoffset = script_size + filesize as u64;
    buf_in_file.seek(Start(dataoffset as u64))?;
    let mut data : Vec<u8> = Vec::new();
    buf_in_file.read_to_end(&mut data)?;
    let data_name = out_dir.join("data.zip").as_path().to_owned();
    if extract.data {
        let mut data_fd = File::create(data_name)?;
    }
    data_fd.write_all(&data)?;
    Ok(())
}
