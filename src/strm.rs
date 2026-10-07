//! Skylanders Spyro's Adventure, made with an older engine than the later
//! games, keeps the pictures of its menus in `uigameimage.str`: "strm" and
//! four words, then one zlib stream. Unpacked, that is the word 0xb053e597
//! and a count, then for each picture the same word, a hash, a zero, the
//! size of what follows, and an "APKF" package holding the picture: its name
//! at the offset kept at 0x44, the size of its pixels at 0x48, an RSX
//! texture description after the name (format, mipmap count, dimension, cube
//! map, the remap 0x0000aae4, width, height), and its pixels at 0x108.
//! Everything is big-endian. All 737 pictures in the PS3 game are laid out
//! so.

use flate2::read::ZlibDecoder;
use std::io::Read;

const STREAM: &[u8] = b"strm";
const STREAM_HEADER: usize = 20;
const RECORD: u32 = 0xb053_e597;
const RECORD_HEADER: usize = 16;
const PACKAGE: &[u8] = b"APKF";
const NAME_AT: usize = 0x44;
const PIXELS_SIZE_AT: usize = 0x48;
const PIXELS_AT: usize = 0x108;
const REMAP: [u8; 4] = [0, 0, 0xaa, 0xe4];
/// The RSX's own number for DXT5, "COMPRESSED_DXT45".
pub const DXT5: u8 = 0x88;
const BLOCK: usize = 16;
const UNREADABLE: &str = "Couldn't read the pictures in this game's files.";

pub struct Picture<'a> {
    pub name: String,
    pub format: u8,
    pub width: usize,
    pub height: usize,
    pub pixels: &'a [u8],
}

/// The file's stream, unpacked.
pub fn unpack(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.get(..STREAM.len()) != Some(STREAM) {
        return Err(UNREADABLE.to_string());
    }
    let mut unpacked = Vec::new();
    ZlibDecoder::new(bytes.get(STREAM_HEADER..).ok_or(UNREADABLE)?)
        .read_to_end(&mut unpacked)
        .map_err(|_| UNREADABLE.to_string())?;
    Ok(unpacked)
}

fn word(bytes: &[u8], at: usize) -> Option<usize> {
    bytes.get(at..at + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap()) as usize)
}

/// Every picture in an unpacked stream.
pub fn pictures(unpacked: &[u8]) -> Result<Vec<Picture<'_>>, String> {
    let count = match (word(unpacked, 0), word(unpacked, 4)) {
        (Some(magic), Some(count)) if magic == RECORD as usize => count,
        _ => return Err(UNREADABLE.to_string()),
    };
    let mut at = 8;
    let mut pictures = Vec::with_capacity(count);
    for _ in 0..count {
        let size = word(unpacked, at + 12).ok_or(UNREADABLE)?;
        let package = unpacked.get(at + RECORD_HEADER..at + RECORD_HEADER + size).ok_or(UNREADABLE)?;
        if word(unpacked, at) != Some(RECORD as usize) || package.get(..PACKAGE.len()) != Some(PACKAGE) {
            return Err(UNREADABLE.to_string());
        }
        pictures.push(picture(package).ok_or(UNREADABLE)?);
        at += RECORD_HEADER + size;
    }
    Ok(pictures)
}

fn picture(package: &[u8]) -> Option<Picture<'_>> {
    let name_at = word(package, NAME_AT)?;
    let name_end = name_at + package.get(name_at..)?.iter().position(|&b| b == 0)?;
    let name = String::from_utf8_lossy(&package[name_at..name_end]).into_owned();
    // The description sits on a word boundary after the name, known by the
    // remap that follows its first word.
    let first = name_end.next_multiple_of(4);
    let texture = (first..PIXELS_AT).step_by(4).find(|&at| package.get(at + 4..at + 8) == Some(&REMAP[..]))?;
    let half = |at: usize| package.get(at..at + 2).map(|b| usize::from(u16::from_be_bytes([b[0], b[1]])));
    let size = word(package, PIXELS_SIZE_AT)?;
    Some(Picture {
        name,
        format: package[texture],
        width: half(texture + 8)?,
        height: half(texture + 10)?,
        pixels: package.get(PIXELS_AT..PIXELS_AT + size)?,
    })
}

/// DXT5 blocks in the usual order. This game keeps each block's eight bytes
/// of colour before its eight of alpha, the other way round.
pub fn blocks(pixels: &[u8]) -> Vec<u8> {
    pixels
        .chunks_exact(BLOCK)
        .flat_map(|block| block[BLOCK / 2..].iter().chain(&block[..BLOCK / 2]))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str, width: u16, pixels: &[u8]) -> Vec<u8> {
        let mut package = vec![0; PIXELS_AT];
        package[..4].copy_from_slice(PACKAGE);
        package[NAME_AT..NAME_AT + 4].copy_from_slice(&0x4cu32.to_be_bytes());
        package[PIXELS_SIZE_AT..PIXELS_SIZE_AT + 4].copy_from_slice(&(pixels.len() as u32).to_be_bytes());
        package[0x4c..0x4c + name.len()].copy_from_slice(name.as_bytes());
        let texture = (0x4c + name.len() + 1).next_multiple_of(16) + 8;
        package[texture] = DXT5;
        package[texture + 4..texture + 8].copy_from_slice(&REMAP);
        package[texture + 8..texture + 10].copy_from_slice(&width.to_be_bytes());
        package[texture + 10..texture + 12].copy_from_slice(&width.to_be_bytes());
        package.extend_from_slice(pixels);
        package.extend_from_slice(&[0; 4]);
        package
    }

    fn stream(packages: &[Vec<u8>]) -> Vec<u8> {
        let mut unpacked = RECORD.to_be_bytes().to_vec();
        unpacked.extend_from_slice(&(packages.len() as u32).to_be_bytes());
        for package in packages {
            unpacked.extend_from_slice(&RECORD.to_be_bytes());
            unpacked.extend_from_slice(&[0; 8]);
            unpacked.extend_from_slice(&(package.len() as u32).to_be_bytes());
            unpacked.extend_from_slice(package);
        }
        unpacked
    }

    #[test]
    fn reads_each_picture_by_its_name() {
        let first = package("eruptor_vs", 4, &[7; 16]);
        let second = package("elementiconfire", 8, &[9; 64]);
        let unpacked = stream(&[first, second]);
        let read = pictures(&unpacked).unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!((read[0].name.as_str(), read[0].format, read[0].width, read[0].height), ("eruptor_vs", DXT5, 4, 4));
        assert_eq!(read[0].pixels, &[7; 16]);
        assert_eq!((read[1].name.as_str(), read[1].width, read[1].pixels.len()), ("elementiconfire", 8, 64));
    }

    #[test]
    fn unpacks_the_stream() {
        use flate2::write::ZlibEncoder;
        use std::io::Write;
        let unpacked = stream(&[package("yeti_vs", 4, &[1; 16])]);
        let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&unpacked).unwrap();
        let mut file = b"strm".to_vec();
        file.extend_from_slice(&[0; 16]);
        file.extend_from_slice(&encoder.finish().unwrap());
        assert_eq!(unpack(&file).unwrap(), unpacked);
        assert!(unpack(b"not a stream at all").is_err());
    }

    #[test]
    fn something_else_is_not_read() {
        assert!(pictures(&[0; 16]).is_err());
        let mut short = stream(&[package("yeti_vs", 4, &[1; 16])]);
        short.truncate(short.len() - 10);
        assert!(pictures(&short).is_err());
    }

    #[test]
    fn colour_comes_back_after_alpha() {
        let block: Vec<u8> = (0..16).collect();
        assert_eq!(blocks(&block), [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7]);
    }
}
