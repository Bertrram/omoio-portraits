//! The game's own archives (.pak), "IGA" version 10 as Skylanders SWAP Force
//! writes them on the Wii U: a big-endian header, a checksum and a
//! descriptor for each file, the file names, and each file stored as it is
//! or as raw deflate in 32 KB chunks. A chunk is led by its compressed
//! length, little-endian unlike everything else, and starts on the archive's
//! alignment; a chunk that didn't shrink is stored whole, with no length.
//! The layout follows the community's notes on the format and was checked
//! against the game's files.
//!
//! Skylanders Trap Team (.arc and .bld) writes version 11 with its header
//! little-endian: the descriptor's offset moves to its first word and the
//! names' offset to 0x28. Every picture in it is stored whole; its packed
//! files use another scheme, which nothing here needs.

use flate2::read::DeflateDecoder;
use std::io::Read;

const MAGIC: u32 = 0x1a41_4749;
const SWAP_FORCE: u32 = 0x0a;
const TRAP_TEAM: u32 = 0x0b;
const CHUNK: usize = 0x8000;
const DESCRIPTORS: usize = 0x38;
const STORED: u32 = 0xff;
const DEFLATED: [u32; 2] = [0x00, 0x10];

pub struct Pak<'a> {
    bytes: &'a [u8],
    align: usize,
    pub files: Vec<PakFile>,
}

pub struct PakFile {
    pub name: String,
    offset: usize,
    size: usize,
    mode: u32,
}

fn be32(bytes: &[u8], at: usize) -> Option<usize> {
    bytes.get(at..at + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap()) as usize)
}

fn le32(bytes: &[u8], at: usize) -> Option<usize> {
    bytes.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize)
}

fn not_known() -> String {
    "The game's picture archive isn't in a form Omoio knows.".to_string()
}

pub fn open(bytes: &[u8]) -> Result<Pak<'_>, String> {
    // Which way round the numbers are, where the names start, and where in a
    // descriptor the file's offset is.
    let (word, names_at, offset_at): (fn(&[u8], usize) -> Option<usize>, usize, usize) =
        if be32(bytes, 0) == Some(MAGIC as usize) && be32(bytes, 4) == Some(SWAP_FORCE as usize) {
            (be32, 0x2c, 4)
        } else if le32(bytes, 0) == Some(MAGIC as usize) && le32(bytes, 4) == Some(TRAP_TEAM as usize) {
            (le32, 0x28, 0)
        } else {
            return Err(not_known());
        };
    let count = word(bytes, 0x0c).ok_or_else(not_known)?;
    let align = word(bytes, 0x10).ok_or_else(not_known)?.max(1);
    let names = word(bytes, names_at).ok_or_else(not_known)?;
    // Each file has a four-byte checksum before the descriptors start.
    let descriptors = DESCRIPTORS + count * 4;
    let files = (0..count)
        .map(|index| {
            let at = descriptors + index * 16;
            let name_at = names + word(bytes, names + index * 4)?;
            let length = bytes.get(name_at..)?.iter().position(|&b| b == 0)?;
            Some(PakFile {
                name: String::from_utf8_lossy(&bytes[name_at..name_at + length]).into_owned(),
                offset: word(bytes, at + offset_at)?,
                size: word(bytes, at + 8)?,
                mode: word(bytes, at + 12)? as u32,
            })
        })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(not_known)?;
    Ok(Pak { bytes, align, files })
}

impl Pak<'_> {
    pub fn read(&self, file: &PakFile) -> Result<Vec<u8>, String> {
        let kind = file.mode >> 24;
        if kind == STORED {
            return self
                .bytes
                .get(file.offset..file.offset + file.size)
                .map(<[u8]>::to_vec)
                .ok_or_else(not_known);
        }
        if !DEFLATED.contains(&kind) {
            return Err(not_known());
        }
        let mut out = Vec::with_capacity(file.size);
        let mut at = file.offset;
        while out.len() < file.size {
            let length = self
                .bytes
                .get(at..at + 2)
                .map(|b| usize::from(u16::from_le_bytes([b[0], b[1]])))
                .ok_or_else(not_known)?;
            let packed = self.bytes.get(at + 2..at + 2 + length).unwrap_or(&[]);
            let mut chunk = Vec::with_capacity(CHUNK);
            if !packed.is_empty() && DeflateDecoder::new(packed).read_to_end(&mut chunk).is_ok() {
                at += 2 + length;
            } else {
                let end = (at + CHUNK).min(self.bytes.len());
                chunk = self.bytes[at..end].to_vec();
                at = end;
            }
            if chunk.is_empty() {
                return Err(not_known());
            }
            out.extend_from_slice(&chunk);
            at = at.div_ceil(self.align) * self.align;
        }
        out.truncate(file.size);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;

    /// An archive of one file, `data`, deflated in chunks as the game does.
    fn build(name: &str, data: &[u8]) -> Vec<u8> {
        let align = 0x800;
        let mut out = vec![0; DESCRIPTORS + 4 + 16];
        out[0..4].copy_from_slice(&MAGIC.to_be_bytes());
        out[4..8].copy_from_slice(&SWAP_FORCE.to_be_bytes());
        out[0x0c..0x10].copy_from_slice(&1u32.to_be_bytes());
        out[0x10..0x14].copy_from_slice(&(align as u32).to_be_bytes());
        let offset = out.len().div_ceil(align) * align;
        out.resize(offset, 0);
        for chunk in data.chunks(CHUNK) {
            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(chunk).unwrap();
            let packed = encoder.finish().unwrap();
            out.extend_from_slice(&(packed.len() as u16).to_le_bytes());
            out.extend_from_slice(&packed);
            out.resize(out.len().div_ceil(align) * align, 0);
        }
        let names = out.len();
        out.extend_from_slice(&4u32.to_be_bytes());
        out.extend_from_slice(name.as_bytes());
        out.push(0);
        out[0x2c..0x30].copy_from_slice(&(names as u32).to_be_bytes());
        let at = DESCRIPTORS + 4;
        out[at + 4..at + 8].copy_from_slice(&(offset as u32).to_be_bytes());
        out[at + 8..at + 12].copy_from_slice(&(data.len() as u32).to_be_bytes());
        out[at + 12..at + 16].copy_from_slice(&0x1000_0000u32.to_be_bytes());
        out
    }

    #[test]
    fn a_deflated_file_reads_back_across_chunks() {
        let data: Vec<u8> = (0..CHUNK * 2 + 300).map(|i| (i % 97) as u8).collect();
        let bytes = build("textures\\a.igz", &data);
        let pak = open(&bytes).unwrap();
        assert_eq!(pak.files.len(), 1);
        assert_eq!(pak.files[0].name, "textures\\a.igz");
        assert_eq!(pak.read(&pak.files[0]).unwrap(), data);
    }

    #[test]
    fn other_archives_are_refused() {
        assert!(open(b"PK\x03\x04 not an archive of the game's").is_err());
    }

    /// A Trap Team archive of files stored whole, laid out as the
    /// game's are: little-endian, offset first in each descriptor, and the
    /// names' offset at 0x28.
    fn build_trap_team(files: &[(&str, &[u8])]) -> Vec<u8> {
        let count = files.len();
        let mut out = vec![0; DESCRIPTORS + count * 20];
        out[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        out[4..8].copy_from_slice(&TRAP_TEAM.to_le_bytes());
        out[0x0c..0x10].copy_from_slice(&(count as u32).to_le_bytes());
        out[0x10..0x14].copy_from_slice(&0x800u32.to_le_bytes());
        let mut places = Vec::new();
        for (_, data) in files {
            out.resize(out.len().div_ceil(0x800) * 0x800, 0);
            places.push(out.len());
            out.extend_from_slice(data);
        }
        let names = out.len();
        out[0x28..0x2c].copy_from_slice(&(names as u32).to_le_bytes());
        let mut text = Vec::new();
        for (name, _) in files {
            out.extend_from_slice(&((count * 4 + text.len()) as u32).to_le_bytes());
            // Each name is followed by a second string, a hash, as in the game.
            text.extend_from_slice(name.as_bytes());
            text.extend_from_slice(b"\x000x322c7612\x00");
        }
        out.extend_from_slice(&text);
        for (index, ((_, data), place)) in files.iter().zip(places).enumerate() {
            let at = DESCRIPTORS + count * 4 + index * 16;
            out[at..at + 4].copy_from_slice(&(place as u32).to_le_bytes());
            out[at + 8..at + 12].copy_from_slice(&(data.len() as u32).to_le_bytes());
            out[at + 12..at + 16].copy_from_slice(&u32::MAX.to_le_bytes());
        }
        out
    }

    #[test]
    fn a_trap_team_archive_gives_its_files_whole() {
        let first: Vec<u8> = (0..3000).map(|i| (i % 251) as u8).collect();
        let bytes = build_trap_team(&[("ui/villains/1001_villaincaptured.png/0x1.igz", &first), ("b.igz", b"IGZ\x01")]);
        let pak = open(&bytes).unwrap();
        assert_eq!(pak.files.len(), 2);
        assert_eq!(pak.files[0].name, "ui/villains/1001_villaincaptured.png/0x1.igz");
        assert_eq!(pak.read(&pak.files[0]).unwrap(), first);
        assert_eq!(pak.read(&pak.files[1]).unwrap(), b"IGZ\x01");
    }
}
