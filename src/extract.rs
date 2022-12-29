//! This function is designed to extract the component parts of a GOG Linux installer script. It is almost an exact port of the implementation found [here](https://github.com/Yepoleb/gogextract), and the ideas belong to him. Thanks for the original implementation.
use crate::error::*;
use regex::*;
use std::fs;
use std::fs::*;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Read;
use std::io::SeekFrom::*;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::*;
pub struct ToExtract {
    pub unpacker: bool,
    pub mojosetup: bool,
    pub data: bool,
}
#[derive(Debug)]
pub enum EOCDOffset {
    Offset(usize),
    Offset64(usize),
}
#[derive(Debug)]
pub struct CentralDirectory {
    pub header: u32,
    pub disk: u16,
    pub cd_start_disk: u16,
    pub cd_records: u16,
    pub total_cd_records: u16,
    pub cd_size: u32,
    pub cd_start_offset: u32,
    pub comment_length: u16,
    pub comment: String,
}
impl CentralDirectory {
    pub fn from_reader<R: Read>(reader: &mut BufReader<R>) -> Self {
        let header = read_32(reader);
        let disk = read_16(reader);
        let cd_start_disk = read_16(reader);
        let cd_records = read_16(reader);
        let total_cd_records = read_16(reader);
        let cd_size = read_32(reader);
        let cd_start_offset = read_32(reader);
        let comment_length = read_16(reader);
        let mut comment = String::new();
        if comment_length > 0 {
            let mut buffer = vec![0; comment_length as usize];
            reader
                .take(comment_length as u64)
                .read_to_end(&mut buffer)
                .unwrap();
            comment = String::from_utf8(buffer.to_vec()).unwrap();
        }
        CentralDirectory {
            header,
            disk,
            cd_start_disk,
            cd_records,
            total_cd_records,
            cd_size,
            cd_start_offset,
            comment_length,
            comment,
        }
    }
}
#[derive(Debug)]
pub struct CentralDirectory64 {
    pub header: u32,
    pub directory_record_size: u64,
    pub version_made_by: u16,
    pub version_needed: u16,
    pub cd: u32,
    pub cd_start: u32,
    pub cd_total_disk: u64,
    pub cd_total: u64,
    pub cd_size: u64,
    pub cd_offset: u64,
}

impl CentralDirectory64 {
    pub fn from_reader<R: Read>(reader: &mut BufReader<R>) -> Self {
        let header = read_32(reader);
        let directory_record_size = read_64(reader);
        let version_made_by = read_16(reader);
        let version_needed = read_16(reader);
        let cd = read_32(reader);
        let cd_start = read_32(reader);
        let cd_total_disk = read_64(reader);
        let cd_total = read_64(reader);
        let cd_size = read_64(reader);
        let cd_offset = read_64(reader);
        CentralDirectory64 {
            header: header,
            directory_record_size: directory_record_size,
            version_made_by: version_made_by,
            version_needed: version_needed,
            cd: cd,
            cd_start: cd_start,
            cd_total_disk: cd_total_disk,
            cd_total: cd_total,
            cd_size: cd_size,
            cd_offset: cd_offset,
        }
    }
}
#[derive(Debug)]
pub struct CDEntry {
    pub header: u32,
    pub version_made_by: Option<u16>,
    pub version_needed: u16,
    pub flag: u16,
    pub compression_method: u16,
    pub mod_date: u16,
    pub mod_time: u16,
    pub crc32: u32,
    pub comp_size: u64,
    pub uncomp_size: u64,
    pub filename_length: u16,
    pub extra_length: u16,
    pub comment_length: Option<u16>,
    pub disk_num: Option<u32>,
    pub internal_file_attr: Option<u16>,
    pub external_file_attr: Option<u32>,
    pub disk_offset: Option<u64>,
    pub filename: String,
    pub comment: String,
    pub end_offset: u64,
    pub start_offset: u64,
}
impl CDEntry {
    pub fn from_reader<R: Read>(reader: &mut BufReader<R>) -> Self {
        let header = read_32(reader);
        let version_made_by = Some(read_16(reader));
        let version_needed = read_16(reader);
        let flag = read_16(reader);
        let compression_method = read_16(reader);
        let mod_time = read_16(reader);
        let mod_date = read_16(reader);
        let crc32 = read_32(reader);
        let comp_size = read_32(reader) as u64;
        let uncomp_size = read_32(reader) as u64;
        let filename_length = read_16(reader);
        let extra_length = read_16(reader);
        let comment_length = Some(read_16(reader));
        let disk_num = Some(read_16(reader) as u32);
        let internal_file_attr = Some(read_16(reader));
        let external_file_attr = Some(read_32(reader));
        let disk_offset = Some(read_32(reader) as u64);
        let mut filename = String::new();
        if filename_length > 0 {
            let mut buffer = vec![0; filename_length as usize];
            reader.read_exact(&mut buffer).unwrap();
            filename = String::from_utf8_lossy(&buffer.to_vec()).to_string();
        }
        if extra_length > 0 {
            let buffer = vec![0; extra_length as usize];
            reader.read_exact(&mut buffer.to_vec()).unwrap();
        }
        let mut comment = String::new();
        if let Some(co_length) = comment_length {
            if co_length > 0 {
                let mut buffer = vec![0; co_length as usize];
                reader.read_exact(&mut buffer).unwrap();
                comment = String::from_utf8_lossy(&buffer.to_vec()).to_string();
            }
        }
        CDEntry {
            header: header,
            version_made_by: version_made_by,
            version_needed: version_needed,
            flag: flag,
            compression_method: compression_method,
            mod_date: mod_date,
            mod_time: mod_time,
            crc32: crc32,
            comp_size: comp_size,
            uncomp_size: uncomp_size,
            filename_length: filename_length,
            extra_length: extra_length,
            comment_length: comment_length,
            disk_num: disk_num,
            internal_file_attr: internal_file_attr,
            external_file_attr: external_file_attr,
            disk_offset: disk_offset,
            filename: filename,
            comment: comment,
            end_offset: 0,
            start_offset: 0,
        }
    }
}

fn read_16<R: Read>(reader: &mut BufReader<R>) -> u16 {
    let mut buffer = [0; 2];
    reader.read_exact(&mut buffer).unwrap();
    u16::from_le_bytes(buffer)
}
fn read_32<R: Read>(reader: &mut BufReader<R>) -> u32 {
    let mut buffer = [0; 4];
    reader.read_exact(&mut buffer).unwrap();
    u32::from_le_bytes(buffer)
}
fn read_64<R: Read>(reader: &mut BufReader<R>) -> u64 {
    let mut buffer = [0; 8];
    reader.read_exact(&mut buffer).unwrap();
    u64::from_le_bytes(buffer)
}
#[derive(Debug)]
pub struct ZipData {
    pub url: String,
    pub files: Vec<CDEntry>,
    pub sizes: (usize, usize),
    pub cd: Option<CentralDirectory>,
}
/// When given a file descriptor for a GOG Game installer and output directory, extracts out the different parts of it as unpacker.sh, mojosetup.tar.gz, and data.zip.
pub fn extract<N>(in_file: &mut File, out_string: N, extract: ToExtract) -> Result<()>
where
    N: Into<String>,
{
    let out_string = out_string.into();
    let out_dir = Path::new(&out_string);
    let filesize_reg = Regex::new(r#"filesizes="(\d+)"#).unwrap();
    let offset_reg = Regex::new(r"offset=`head -n (\d+)").unwrap();
    fs::create_dir_all(out_dir)?;
    let mut beg_buf: [u8; 10240] = [0; 10240];
    in_file.read_exact(&mut beg_buf)?;
    let vec_beg: Vec<u8> = beg_buf.into();
    let mut buf_in_file = BufReader::new(in_file);
    buf_in_file.seek(Start(0))?;
    let beg_string = String::from_utf8(vec_beg).unwrap();
    let offset_string = offset_reg.captures(&beg_string).unwrap()[1].to_string();
    let lines: i64 = offset_string.parse().unwrap();
    let mut dump = String::new();
    let mut script_size: u64 = 0;
    for _ in 0..lines {
        script_size += buf_in_file.read_line(&mut dump)? as u64;
    }
    buf_in_file.seek(Start(0))?;
    let mut script_bin = vec![0u8; script_size as usize];
    buf_in_file.read_exact(&mut script_bin)?;
    let unpacker_name = out_dir.join("unpacker.sh");
    if extract.unpacker {
        let mut unpacker_fd = File::create(unpacker_name)?;
        #[cfg(unix)]
        {
            let mut perms = unpacker_fd.metadata()?.permissions();
            perms.set_mode(0o744);
            unpacker_fd.set_permissions(perms)?;
        }
        unpacker_fd.write_all(&script_bin)?;
    }
    let script = String::from_utf8(script_bin).unwrap();
    let filesize = filesize_reg.captures(&script).unwrap()[1]
        .to_string()
        .parse()
        .unwrap();
    let mut mojo = vec![0u8; filesize];
    buf_in_file.seek(Start(script_size))?;
    buf_in_file.read_exact(&mut mojo)?;
    let archive_name = out_dir.join("mojosetup.tar.gz");
    if extract.mojosetup {
        let mut archive_fd = File::create(archive_name)?;
        archive_fd.write_all(&mojo)?;
    }
    let dataoffset = script_size + filesize as u64;
    buf_in_file.seek(Start(dataoffset))?;
    let mut data: Vec<u8> = Vec::new();
    buf_in_file.read_to_end(&mut data)?;
    let data_name = out_dir.join("data.zip").as_path().to_owned();
    if extract.data {
        let mut data_fd = File::create(data_name)?;
        data_fd.write_all(&data)?;
    }
    Ok(())
}
