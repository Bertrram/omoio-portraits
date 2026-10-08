//! The game's own archives (.pak), "IGA" version 10 as Skylanders SWAP Force
//! writes them on the Wii U: a big-endian header, a checksum and a
//! descriptor for each file, the file names, and each file stored as it is
//! or as raw deflate in 32 KB chunks. A chunk is led by its compressed
//! length, little-endian unlike everything else, and starts on the archive's
//! alignment; a chunk that didn't shrink is stored whole, with no length.
//! The layout follows the community's notes on the format and was checked
//! against the game's files.
//!
//! Which chunks are packed, and where each starts, is in three tables after
//! the descriptors, whose lengths the header gives at 0x1C, 0x20 and 0x24:
//! 32-bit, 16-bit and 8-bit numbers. Each is where a chunk starts, in
//! sectors of the archive's alignment from the file's own start, with its
//! top bit set when the chunk is packed, and each file has one for every
//! chunk and one more for its end. A file of up to 0x7F sectors uses the
//! 8-bit table, one of up to 0x7FFF the 16-bit table and a longer one the
//! 32-bit table; the low 28 bits of its mode say where its numbers start.
//! This follows LG-RZ's igArchiveLib (MIT, read 7 October 2026) and was
//! checked against every archive of SWAP Force on the PS3 and of
//! SuperChargers on the Wii U the same day: each table is used exactly
//! once by the files that point into it, every file starts at sector 0 and
//! ends on a number without the top bit. Before, a chunk stored whole was
//! told by its not inflating, which mistook one whose first bytes happen to
//! inflate (SuperChargers has four).
//!
//! Skylanders Trap Team (.arc and .bld) writes version 11 with its header
//! little-endian: the descriptor's offset moves to its first word and the
//! names' offset to 0x28. Every picture in it is stored whole; its packed
//! files use another scheme, which nothing here needs.
//!
//! Skylanders Giants on the PS3 (.arc and .bld) writes version 8, also
//! little-endian, with the names' offset at 0x1C, the checksums from 0x34
//! and descriptors of 12 bytes: offset, size and mode. Its .bld archives
//! pack their files with LZMA, in chunks laid out as SWAP Force's are; a
//! packed chunk's stream starts with the coder's five bytes of settings,
//! which its length leaves out.
//!
//! Skylanders SuperChargers on the Wii U writes version 11 as Trap Team
//! does, but big-endian and laid out as SWAP Force's version 10 in every
//! other way: the names' offset at 0x2C, the descriptor's offset in its
//! second word, the chunk tables, and its files deflated in the same
//! chunks. NefariousTechSupport's igArchiveExtractor (GPL-3.0, read
//! 7 October 2026) lists the same offsets for it. Checked against all 6237
//! archives of the game the same day: every packed file of all 270,576
//! inflates to its size. Some, which hold no files, end before where their
//! names would start.
//!
//! Skylanders Spyro's Adventure on the Wii (.arc and .bld) writes version 4,
//! its header little-endian: the names' offset at 0x18, the checksums from
//! 0x30, descriptors of 12 bytes as Giants' (offset, size and mode), and the
//! three chunk tables after them, little-endian too, their lengths at 0x20,
//! 0x24 and 0x28. Its header keeps no alignment (0x10 holds something
//! else); every file and chunk starts on 0x800. A packed file's mode starts
//! 0x10, and its chunks are LZMA as Giants' are, but each led by its length
//! big-endian. Worked out from the game's files, 8 October 2026, and checked
//! against every archive of its `misc` and `permanent` folders.

use flate2::read::DeflateDecoder;
use lzma_rust2::LzmaReader;
use std::io::Read;

const MAGIC: u32 = 0x1a41_4749;
const SPYROS_ADVENTURE_WII: u32 = 0x04;
const GIANTS: u32 = 0x08;
const SWAP_FORCE: u32 = 0x0a;
const TRAP_TEAM: u32 = 0x0b;
const SUPERCHARGERS: u32 = 0x0b;
const CHUNK: usize = 0x8000;
const DESCRIPTORS: usize = 0x38;
const GIANTS_DESCRIPTORS: usize = 0x34;
const WII_DESCRIPTORS: usize = 0x30;
/// Where the lengths of the chunk tables are kept: version 4 has them one
/// word on from the later versions.
const TABLES: usize = 0x1c;
const WII_TABLES: usize = 0x20;
/// Version 4's alignment, which its header doesn't keep.
const WII_ALIGN: usize = 0x800;
const STORED: u32 = 0xff;
const DEFLATED: [u32; 2] = [0x00, 0x10];
const LZMA: u32 = 0x20;
const WII_LZMA: u32 = 0x10;
/// The part of a packed file's mode that says where its chunk numbers start.
const BLOCK_INDEX: u32 = 0x0fff_ffff;
/// The longest files, in sectors, the 8-bit and the 16-bit tables are for.
const SMALL_SECTORS: usize = 0x7f;
const MEDIUM_SECTORS: usize = 0x7fff;
/// The settings Giants packs every chunk with: lc 3, lp 0 and pb 2, and a
/// dictionary as big as a chunk.
const LZMA_SETTINGS: [u8; 5] = [0x5d, 0x00, 0x80, 0x00, 0x00];

pub struct Pak<'a> {
    bytes: &'a [u8],
    align: usize,
    /// Where the 32-bit, 16-bit and 8-bit chunk tables start, in the
    /// versions that have them.
    tables: Option<[usize; 3]>,
    /// Version 4's tables are little-endian and its chunks' lengths
    /// big-endian, the other way round from the later versions.
    wii: bool,
    pub files: Vec<PakFile>,
}

pub struct PakFile {
    pub name: String,
    offset: usize,
    /// Its size once unpacked.
    pub size: usize,
    mode: u32,
    packing: Packing,
}

/// How a file is kept, from the top byte of its mode, which version 4
/// numbers its own way.
#[derive(Clone, Copy, PartialEq)]
enum Packing {
    Stored,
    Deflate,
    Lzma,
    Other(u32),
}

fn packing(version: u32, mode: u32) -> Packing {
    match (version, mode >> 24) {
        (_, STORED) => Packing::Stored,
        (SPYROS_ADVENTURE_WII, WII_LZMA) => Packing::Lzma,
        (SPYROS_ADVENTURE_WII, kind) => Packing::Other(kind),
        (_, kind) if DEFLATED.contains(&kind) => Packing::Deflate,
        (_, LZMA) => Packing::Lzma,
        (_, kind) => Packing::Other(kind),
    }
}

type Word = fn(&[u8], usize) -> Option<usize>;

/// What an archive's first bytes say about it: which way round its numbers
/// are, where its descriptors start and how long each is, where in one the
/// file's offset, size and mode are, how many files it has, its alignment,
/// and where its names start. In every version seen the names are at the
/// end, after the files.
pub struct Head {
    version: u32,
    word: Word,
    descriptors: usize,
    descriptor: usize,
    fields: [usize; 3],
    count: usize,
    align: usize,
    /// How many numbers each chunk table holds, in the versions that have
    /// them.
    tables: Option<[usize; 3]>,
    pub names: usize,
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

/// The archive's version and whether its header is little-endian, known
/// here or not. `None` when it isn't one of the game's archives.
pub fn version(bytes: &[u8]) -> Option<(u32, bool)> {
    if be32(bytes, 0)? == MAGIC as usize {
        Some((be32(bytes, 4)? as u32, false))
    } else if le32(bytes, 0)? == MAGIC as usize {
        Some((le32(bytes, 4)? as u32, true))
    } else {
        None
    }
}

impl Head {
    /// Reads the head from the archive's first 0x40 bytes or more.
    pub fn read(bytes: &[u8]) -> Result<Head, String> {
        // Where the names' offset is kept, where the checksums start, the
        // descriptors' layout, and where the lengths of the chunk tables
        // that follow them are kept, for each version.
        let (word, names_at, checksums, descriptor, fields, chunked): (Word, usize, usize, usize, [usize; 3], Option<usize>) =
            match version(bytes) {
                Some((SWAP_FORCE | SUPERCHARGERS, false)) => (be32, 0x2c, DESCRIPTORS, 16, [4, 8, 12], Some(TABLES)),
                Some((TRAP_TEAM, true)) => (le32, 0x28, DESCRIPTORS, 16, [0, 8, 12], None),
                Some((GIANTS, true)) => (le32, 0x1c, GIANTS_DESCRIPTORS, 12, [0, 4, 8], None),
                Some((SPYROS_ADVENTURE_WII, true)) => (le32, 0x18, WII_DESCRIPTORS, 12, [0, 4, 8], Some(WII_TABLES)),
                _ => return Err(not_known()),
            };
        let at = |offset: usize| word(bytes, offset).ok_or_else(not_known);
        let count = at(0x0c)?;
        let version = version(bytes).map_or(0, |(version, _)| version);
        Ok(Head {
            version,
            word,
            // Each file has a four-byte checksum before the descriptors start.
            descriptors: checksums + count * 4,
            descriptor,
            fields,
            count,
            align: if version == SPYROS_ADVENTURE_WII { WII_ALIGN } else { at(0x10)?.max(1) },
            tables: match chunked {
                Some(lengths) => Some([at(lengths)?, at(lengths + 4)?, at(lengths + 8)?]),
                None => None,
            },
            names: at(names_at)?,
        })
    }

    /// How many of the archive's first bytes hold the head, the checksums
    /// and the descriptors.
    pub fn length(&self) -> usize {
        self.descriptors + self.count * self.descriptor
    }

    /// The files, from the archive's first `length()` bytes or more, and its
    /// bytes from where the names start.
    pub fn files(&self, start: &[u8], names: &[u8]) -> Result<Vec<PakFile>, String> {
        let word = self.word;
        let [offset_at, size_at, mode_at] = self.fields;
        (0..self.count)
            .map(|index| {
                let at = self.descriptors + index * self.descriptor;
                let name_at = word(names, index * 4)?;
                let length = names.get(name_at..)?.iter().position(|&b| b == 0)?;
                let mode = word(start, at + mode_at)? as u32;
                Some(PakFile {
                    name: String::from_utf8_lossy(&names[name_at..name_at + length]).into_owned(),
                    offset: word(start, at + offset_at)?,
                    size: word(start, at + size_at)?,
                    mode,
                    packing: packing(self.version, mode),
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(not_known)
    }
}

pub fn open(bytes: &[u8]) -> Result<Pak<'_>, String> {
    let head = Head::read(bytes)?;
    let files = head.files(bytes, bytes.get(head.names..).ok_or_else(not_known)?)?;
    // The tables follow the descriptors, the 32-bit one first.
    let tables = head.tables.map(|[large, medium, _]| {
        let start = head.length();
        [start, start + large * 4, start + large * 4 + medium * 2]
    });
    Ok(Pak { bytes, align: head.align, tables, wii: head.version == SPYROS_ADVENTURE_WII, files })
}

impl PakFile {
    /// Where the file's bytes are in the archive and how many, when it is
    /// stored as it is, so it can be read without the rest.
    pub fn stored_at(&self) -> Option<(usize, usize)> {
        (self.packing == Packing::Stored).then_some((self.offset, self.size))
    }

    /// How the file is kept: as it is, deflated, packed with LZMA, or some
    /// other way, given by its number, which nothing here unpacks.
    pub fn packing(&self) -> String {
        match self.packing {
            Packing::Stored => "stored".to_string(),
            Packing::Deflate => "deflate".to_string(),
            Packing::Lzma => "lzma".to_string(),
            Packing::Other(kind) => format!("packing 0x{kind:02x}"),
        }
    }
}

impl Pak<'_> {
    pub fn read(&self, file: &PakFile) -> Result<Vec<u8>, String> {
        let unpack = match file.packing {
            Packing::Stored => {
                return self
                    .bytes
                    .get(file.offset..file.offset + file.size)
                    .map(<[u8]>::to_vec)
                    .ok_or_else(not_known);
            }
            Packing::Deflate => inflate,
            Packing::Lzma => unlzma,
            Packing::Other(_) => return Err(not_known()),
        };
        match self.tables {
            Some(tables) => self.read_by_tables(file, tables, unpack),
            None => self.read_trying(file, unpack),
        }
    }

    /// A packed file chunk by chunk, where the chunk tables say each starts
    /// and whether it is packed.
    fn read_by_tables(&self, file: &PakFile, [large, medium, small]: [usize; 3], unpack: Unpack) -> Result<Vec<u8>, String> {
        let first = (file.mode & BLOCK_INDEX) as usize;
        // Version 4 keeps its tables little-endian and its chunks' lengths
        // big-endian, the later versions the other way round.
        let half = |at: usize, little: bool| {
            let b = self.bytes.get(at..at + 2)?;
            Some(if little { u16::from_le_bytes([b[0], b[1]]) } else { u16::from_be_bytes([b[0], b[1]]) })
        };
        let start = |chunk: usize| -> Option<(bool, usize)> {
            let at = first + chunk;
            if file.size <= SMALL_SECTORS * self.align {
                let number = *self.bytes.get(small + at)?;
                Some((number & 0x80 != 0, usize::from(number & 0x7f)))
            } else if file.size <= MEDIUM_SECTORS * self.align {
                let number = half(medium + at * 2, self.wii)?;
                Some((number & 0x8000 != 0, usize::from(number & 0x7fff)))
            } else {
                let number = if self.wii { le32(self.bytes, large + at * 4)? } else { be32(self.bytes, large + at * 4)? };
                Some((number & 0x8000_0000 != 0, number & 0x7fff_ffff))
            }
        };
        let mut out = Vec::with_capacity(file.size);
        for chunk in 0..file.size.div_ceil(CHUNK) {
            let (packed, sector) = start(chunk).ok_or_else(not_known)?;
            let at = file.offset + sector * self.align;
            let size = (file.size - out.len()).min(CHUNK);
            let bytes = if packed {
                let length = half(at, !self.wii).map(usize::from);
                length.and_then(|length| unpack(self.bytes.get(at + 2..)?, length, size)).map(|(bytes, _)| bytes)
            } else {
                self.bytes.get(at..at + size).map(<[u8]>::to_vec)
            };
            match bytes {
                Some(bytes) if bytes.len() == size => out.extend_from_slice(&bytes),
                _ => return Err(not_known()),
            }
        }
        Ok(out)
    }

    /// A packed file of an archive without chunk tables, Giants', chunk by
    /// chunk, each one that doesn't unpack taken as stored whole.
    fn read_trying(&self, file: &PakFile, unpack: Unpack) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(file.size);
        let mut at = file.offset;
        while out.len() < file.size {
            let length = self
                .bytes
                .get(at..at + 2)
                .map(|b| usize::from(u16::from_le_bytes([b[0], b[1]])))
                .ok_or_else(not_known)?;
            let size = (file.size - out.len()).min(CHUNK);
            let chunk = match unpack(self.bytes.get(at + 2..).unwrap_or(&[]), length, size) {
                Some((chunk, used)) => {
                    at += 2 + used;
                    chunk
                }
                None => {
                    let end = (at + CHUNK).min(self.bytes.len());
                    let chunk = self.bytes[at..end].to_vec();
                    at = end;
                    chunk
                }
            };
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

/// Unpacks one chunk: the bytes after its length, its length, and its size
/// once unpacked. Gives the chunk and how many bytes it took.
type Unpack = fn(&[u8], usize, usize) -> Option<(Vec<u8>, usize)>;

/// The deflated chunk at the start of `packed`, `length` long, and how many
/// bytes it took. `None` when it isn't one.
fn inflate(packed: &[u8], length: usize, size: usize) -> Option<(Vec<u8>, usize)> {
    let packed = packed.get(..length).filter(|packed| !packed.is_empty())?;
    let mut chunk = Vec::with_capacity(size);
    DeflateDecoder::new(packed).read_to_end(&mut chunk).ok()?;
    Some((chunk, length))
}

/// The chunk packed with LZMA at the start of `packed`, `size` bytes once
/// unpacked: the settings, then a stream `length` long. Returns it and how
/// many bytes it took, or `None` when no settings lead it.
fn unlzma(packed: &[u8], length: usize, size: usize) -> Option<(Vec<u8>, usize)> {
    let stream = packed.strip_prefix(&LZMA_SETTINGS)?.get(..length)?;
    let dictionary = u32::from_le_bytes(LZMA_SETTINGS[1..].try_into().unwrap());
    let mut reader = LzmaReader::new_with_props(stream, size as u64, LZMA_SETTINGS[0], dictionary, None).ok()?;
    let mut chunk = vec![0; size];
    reader.read_exact(&mut chunk).ok()?;
    Some((chunk, LZMA_SETTINGS.len() + length))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use lzma_rust2::{LzmaOptions, LzmaWriter};
    use std::io::Write;

    /// An archive laid out as SWAP Force's and SuperChargers' are: big-endian,
    /// of `version`, each file deflated in chunks, a chunk that doesn't
    /// shrink stored whole, and the chunk tables saying which is which. A
    /// file of more than 0x7F sectors goes in the 16-bit table.
    pub(crate) fn build_chunked(version: u32, files: &[(&str, &[u8])]) -> Vec<u8> {
        let align = 0x800;
        let count = files.len();
        // The files first, from 0, with where each chunk starts in sectors
        // and whether it is packed, then where the file ends.
        let mut body = Vec::new();
        let mut placed = Vec::new();
        for (_, data) in files {
            let start = body.len();
            let mut numbers = Vec::new();
            for chunk in data.chunks(CHUNK) {
                numbers.push((body.len() - start) / align);
                let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(chunk).unwrap();
                let packed = encoder.finish().unwrap();
                if packed.len() + 2 < chunk.len() {
                    *numbers.last_mut().unwrap() |= 1 << 31;
                    body.extend_from_slice(&(packed.len() as u16).to_le_bytes());
                    body.extend_from_slice(&packed);
                } else {
                    body.extend_from_slice(chunk);
                }
                body.resize(body.len().div_ceil(align) * align, 0);
            }
            numbers.push((body.len() - start) / align);
            placed.push((start, numbers));
        }
        let (mut medium, mut small, mut modes) = (Vec::new(), Vec::new(), Vec::new());
        for ((_, data), (_, numbers)) in files.iter().zip(&placed) {
            let packed = |number: usize| (number >> 31 == 1, number & 0x7fff_ffff);
            if data.len() <= SMALL_SECTORS * align {
                modes.push(0x1000_0000 | small.len() as u32);
                small.extend(numbers.iter().map(|&n| packed(n)).map(|(on, sector)| sector as u8 | u8::from(on) << 7));
            } else {
                modes.push(0x1000_0000 | medium.len() as u32);
                medium.extend(numbers.iter().map(|&n| packed(n)).map(|(on, sector)| sector as u16 | u16::from(on) << 15));
            }
        }
        let tables = DESCRIPTORS + count * 20;
        let first = (tables + medium.len() * 2 + small.len()).div_ceil(align) * align;
        let mut out = vec![0; first];
        let word = |out: &mut Vec<u8>, at: usize, value: usize| out[at..at + 4].copy_from_slice(&(value as u32).to_be_bytes());
        word(&mut out, 0, MAGIC as usize);
        word(&mut out, 4, version as usize);
        word(&mut out, 0x0c, count);
        word(&mut out, 0x10, align);
        word(&mut out, 0x20, medium.len());
        word(&mut out, 0x24, small.len());
        for (index, ((_, data), (start, _))) in files.iter().zip(&placed).enumerate() {
            let at = DESCRIPTORS + count * 4 + index * 16;
            word(&mut out, at + 4, first + start);
            word(&mut out, at + 8, data.len());
            word(&mut out, at + 12, modes[index] as usize);
        }
        for (k, number) in medium.iter().enumerate() {
            out[tables + k * 2..tables + k * 2 + 2].copy_from_slice(&u16::to_be_bytes(*number));
        }
        out[tables + medium.len() * 2..tables + medium.len() * 2 + small.len()].copy_from_slice(&small);
        out.extend_from_slice(&body);
        let names = out.len();
        word(&mut out, 0x2c, names);
        let mut text = Vec::new();
        for (name, _) in files {
            out.extend_from_slice(&((count * 4 + text.len()) as u32).to_be_bytes());
            text.extend_from_slice(name.as_bytes());
            text.push(0);
        }
        out.extend_from_slice(&text);
        out
    }

    /// Bytes that don't shrink, from a fixed seed.
    fn noise(length: usize) -> Vec<u8> {
        let mut seed = 0x2545_f491_u32;
        (0..length)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                seed as u8
            })
            .collect()
    }

    #[test]
    fn a_deflated_file_reads_back_across_chunks() {
        // Its second chunk doesn't shrink, so it is stored whole, and starts
        // as a chunk of two packed bytes that inflate to nothing would: as
        // one of SuperChargers' pictures does, taken for a packed chunk when
        // the tables were not read.
        let mut data: Vec<u8> = (0..CHUNK).map(|i| (i % 97) as u8).collect();
        data.extend_from_slice(b"\x02\x00\x03\x00");
        data.extend(noise(CHUNK - 4));
        data.extend((0..300).map(|i| (i % 13) as u8));
        let bytes = build_chunked(SWAP_FORCE, &[("textures\\a.igz", &data)]);
        let pak = open(&bytes).unwrap();
        assert_eq!(pak.files.len(), 1);
        assert_eq!(pak.files[0].name, "textures\\a.igz");
        assert_eq!(pak.files[0].packing(), "deflate");
        assert_eq!(pak.files[0].stored_at(), None);
        assert_eq!(pak.read(&pak.files[0]).unwrap(), data);
        assert!(pak.read_trying(&pak.files[0], inflate).is_err());
    }

    #[test]
    fn a_long_file_is_found_in_the_16_bit_table() {
        let long: Vec<u8> = (0..SMALL_SECTORS * 0x800 + 5000).map(|i| (i % 251) as u8).collect();
        let short = noise(5000);
        let bytes = build_chunked(SUPERCHARGERS, &[("short.igz", &short), ("long.igz", &long), ("last.igz", b"IGZ\x01")]);
        let pak = open(&bytes).unwrap();
        let read: Vec<Vec<u8>> = pak.files.iter().map(|file| pak.read(file).unwrap()).collect();
        assert_eq!(read, [short, long, b"IGZ\x01".to_vec()]);
    }

    #[test]
    fn other_archives_are_refused() {
        assert!(open(b"PK\x03\x04 not an archive of the game's").is_err());
    }

    /// A Trap Team archive of files stored whole, laid out as the
    /// game's are: little-endian, offset first in each descriptor, and the
    /// names' offset at 0x28.
    pub(crate) fn build_trap_team(files: &[(&str, &[u8])]) -> Vec<u8> {
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

    /// A Giants archive laid out as the game's are, with `packed` packed in
    /// LZMA chunks, all but its second, which is left whole as a chunk that
    /// didn't shrink is, and `whole` stored as it is.
    fn build_giants(packed: &[u8], whole: &[u8]) -> Vec<u8> {
        let align = 0x800;
        let mut out = vec![0; GIANTS_DESCRIPTORS + 2 * 16];
        out[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        out[4..8].copy_from_slice(&GIANTS.to_le_bytes());
        out[0x0c..0x10].copy_from_slice(&2u32.to_le_bytes());
        out[0x10..0x14].copy_from_slice(&(align as u32).to_le_bytes());
        out.resize(out.len().div_ceil(align) * align, 0);
        let first = out.len();
        let mut options = LzmaOptions::with_preset(6);
        options.dict_size = CHUNK as u32;
        for (index, chunk) in packed.chunks(CHUNK).enumerate() {
            if index == 1 {
                out.extend_from_slice(chunk);
            } else {
                let mut writer = LzmaWriter::new_no_header(Vec::new(), &options, false).unwrap();
                writer.write_all(chunk).unwrap();
                let stream = writer.finish().unwrap();
                out.extend_from_slice(&(stream.len() as u16).to_le_bytes());
                out.extend_from_slice(&LZMA_SETTINGS);
                out.extend_from_slice(&stream);
            }
            out.resize(out.len().div_ceil(align) * align, 0);
        }
        let second = out.len();
        out.extend_from_slice(whole);
        let names = out.len();
        out[0x1c..0x20].copy_from_slice(&(names as u32).to_le_bytes());
        // As in the game, each name is followed by four more bytes, which
        // nothing here reads.
        out.extend_from_slice(&8u32.to_le_bytes());
        out.extend_from_slice(&22u32.to_le_bytes());
        out.extend_from_slice(b"level.bld\0\x86\x9a\xb7\x17ENGLISH.pak\0\x86\x9a\xb7\x17");
        let files = [(first, packed.len(), 0x2000_0000), (second, whole.len(), u32::MAX)];
        for (index, (offset, size, mode)) in files.into_iter().enumerate() {
            let at = GIANTS_DESCRIPTORS + 2 * 4 + index * 12;
            out[at..at + 4].copy_from_slice(&(offset as u32).to_le_bytes());
            out[at + 4..at + 8].copy_from_slice(&(size as u32).to_le_bytes());
            out[at + 8..at + 12].copy_from_slice(&mode.to_le_bytes());
        }
        out
    }

    /// An archive laid out as Spyro's Adventure's on the Wii: version 4,
    /// little-endian, each of `packed` packed in LZMA chunks, those that
    /// don't shrink left whole, the chunk tables saying which, and `whole`
    /// stored as it is last. A file of more than 0x7F sectors goes in the
    /// 16-bit table.
    pub(crate) fn build_wii(packed: &[(&str, &[u8])], whole: (&str, &[u8])) -> Vec<u8> {
        let align = WII_ALIGN;
        let mut options = LzmaOptions::with_preset(6);
        options.dict_size = CHUNK as u32;
        let mut body = Vec::new();
        let mut placed = Vec::new();
        for (_, data) in packed {
            let start = body.len();
            let mut numbers = Vec::new();
            for chunk in data.chunks(CHUNK) {
                let mut writer = LzmaWriter::new_no_header(Vec::new(), &options, false).unwrap();
                writer.write_all(chunk).unwrap();
                let stream = writer.finish().unwrap();
                let sector = (body.len() - start) / align;
                if stream.len() + 7 < chunk.len() {
                    numbers.push((true, sector));
                    body.extend_from_slice(&(stream.len() as u16).to_be_bytes());
                    body.extend_from_slice(&LZMA_SETTINGS);
                    body.extend_from_slice(&stream);
                } else {
                    numbers.push((false, sector));
                    body.extend_from_slice(chunk);
                }
                body.resize(body.len().div_ceil(align) * align, 0);
            }
            numbers.push((false, (body.len() - start) / align));
            placed.push((start, numbers));
        }
        let (mut medium, mut small, mut modes) = (Vec::new(), Vec::new(), Vec::new());
        for ((_, data), (_, numbers)) in packed.iter().zip(&placed) {
            if data.len() <= SMALL_SECTORS * align {
                modes.push(0x1000_0000 | small.len() as u32);
                small.extend(numbers.iter().map(|&(on, sector)| sector as u8 | u8::from(on) << 7));
            } else {
                modes.push(0x1000_0000 | medium.len() as u32);
                medium.extend(numbers.iter().map(|&(on, sector)| sector as u16 | u16::from(on) << 15));
            }
        }
        let count = packed.len() + 1;
        let tables = WII_DESCRIPTORS + count * 16;
        let first = (tables + medium.len() * 2 + small.len()).div_ceil(align) * align;
        let mut out = vec![0; first];
        let word = |out: &mut Vec<u8>, at: usize, value: usize| out[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
        word(&mut out, 0, MAGIC as usize);
        word(&mut out, 4, SPYROS_ADVENTURE_WII as usize);
        word(&mut out, 0x0c, count);
        // Not the alignment, which version 4 doesn't keep.
        word(&mut out, 0x10, 0x2449_9224);
        word(&mut out, 0x24, medium.len());
        word(&mut out, 0x28, small.len());
        for (index, ((_, data), (start, _))) in packed.iter().zip(&placed).enumerate() {
            let at = WII_DESCRIPTORS + count * 4 + index * 12;
            word(&mut out, at, first + start);
            word(&mut out, at + 4, data.len());
            word(&mut out, at + 8, modes[index] as usize);
        }
        for (k, number) in medium.iter().enumerate() {
            out[tables + k * 2..tables + k * 2 + 2].copy_from_slice(&number.to_le_bytes());
        }
        out[tables + medium.len() * 2..tables + medium.len() * 2 + small.len()].copy_from_slice(&small);
        out.extend_from_slice(&body);
        let (at, offset) = (WII_DESCRIPTORS + count * 4 + packed.len() * 12, out.len());
        word(&mut out, at, offset);
        word(&mut out, at + 4, whole.1.len());
        word(&mut out, at + 8, u32::MAX as usize);
        out.extend_from_slice(whole.1);
        out.resize(out.len().div_ceil(align) * align, 0);
        let names = out.len();
        word(&mut out, 0x18, names);
        let mut text = Vec::new();
        for (name, _) in packed.iter().chain([&whole]) {
            out.extend_from_slice(&((count * 4 + text.len()) as u32).to_le_bytes());
            text.extend_from_slice(name.as_bytes());
            text.push(0);
        }
        out.extend_from_slice(&text);
        out
    }

    #[test]
    fn a_wii_archive_unpacks_its_lzma_chunks() {
        // The long file's second chunk doesn't shrink, so it is left whole.
        let mut long: Vec<u8> = (0..CHUNK).map(|i| (i % 97) as u8).collect();
        long.extend(noise(CHUNK));
        long.extend((0..SMALL_SECTORS * WII_ALIGN).map(|i| (i % 13) as u8));
        let short: Vec<u8> = (0..3000).map(|i| (i % 7) as u8).collect();
        let bytes = build_wii(&[("FRENCH.pak", &short), ("level.bld", &long)], ("x.igz", b"IGZ\x01 stored whole"));
        assert_eq!(version(&bytes), Some((SPYROS_ADVENTURE_WII, true)));
        let pak = open(&bytes).unwrap();
        let names: Vec<_> = pak.files.iter().map(|file| (file.name.as_str(), file.packing())).collect();
        assert_eq!(names, [("FRENCH.pak", "lzma".to_string()), ("level.bld", "lzma".to_string()), ("x.igz", "stored".to_string())]);
        assert_eq!(pak.read(&pak.files[0]).unwrap(), short);
        assert_eq!(pak.read(&pak.files[1]).unwrap(), long);
        assert_eq!(pak.read(&pak.files[2]).unwrap(), b"IGZ\x01 stored whole");
        assert!(pak.files[1].stored_at().is_none());
    }

    #[test]
    fn a_giants_archive_unpacks_its_lzma_chunks() {
        let mut seed = 0x2545_f491_u32;
        let mut packed: Vec<u8> = (0..CHUNK).map(|i| (i % 97) as u8).collect();
        packed.extend((0..CHUNK).map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as u8
        }));
        packed.extend((0..5000).map(|i| (i % 13) as u8));
        let bytes = build_giants(&packed, b"IGZ\x01 stored whole");
        let pak = open(&bytes).unwrap();
        assert_eq!(pak.files.len(), 2);
        assert_eq!(pak.files[0].name, "level.bld");
        assert_eq!(pak.files[1].name, "ENGLISH.pak");
        assert_eq!(pak.files[0].packing(), "lzma");
        assert_eq!(pak.read(&pak.files[0]).unwrap(), packed);
        assert_eq!(pak.read(&pak.files[1]).unwrap(), b"IGZ\x01 stored whole");
    }

    #[test]
    fn an_archive_is_listed_from_its_start_and_its_names() {
        let bytes = build_trap_team(&[("a.igz", &[1; 3000]), ("ui/b.png/0x2.igz", b"IGZ\x01")]);
        let head = Head::read(&bytes[..0x40]).unwrap();
        let files = head.files(&bytes[..head.length()], &bytes[head.names..]).unwrap();
        let names: Vec<_> = files.iter().map(|file| (file.name.as_str(), file.size, file.packing())).collect();
        assert_eq!(names, [("a.igz", 3000, "stored".to_string()), ("ui/b.png/0x2.igz", 4, "stored".to_string())]);
        let (offset, size) = files[1].stored_at().unwrap();
        assert_eq!(&bytes[offset..offset + size], b"IGZ\x01");
        // The descriptors end well before the files, so the start alone holds them.
        assert!(head.length() < 0x800);
    }

    #[test]
    fn an_archive_tells_its_version_even_when_not_known() {
        assert_eq!(version(&build_chunked(SWAP_FORCE, &[("a", b"x")])), Some((SWAP_FORCE, false)));
        assert_eq!(version(&build_chunked(SUPERCHARGERS, &[("a", b"x")])), Some((SUPERCHARGERS, false)));
        assert_eq!(version(&build_trap_team(&[("a", b"x")])), Some((TRAP_TEAM, true)));
        assert_eq!(version(b"IGA\x1a\x0c\x00\x00\x00"), Some((12, true)));
        assert!(Head::read(b"IGA\x1a\x0c\x00\x00\x00 and a version not known").is_err());
        assert_eq!(version(b"PK\x03\x04 not one"), None);
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
