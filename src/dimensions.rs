//! LEGO Dimensions on the PS3: the picture of each minifigure and vehicle
//! the game shows on its character wheel, read from the user's own copy of
//! the game, the folder that holds `PS3_GAME`.
//!
//! The game keeps its files in archives, `GAME.DAT` and `GAME0.DAT` to
//! `GAME5.DAT` in `PS3_GAME/USRDIR`, each listed by a `.HDR` of the same
//! name. The list is laid out as Luigi Auriemma's ttgames.bms 0.9.11a reads
//! one of versions -5 to -9 (read 4 November 2026); this game's are all -7.
//! After a word that isn't needed come the version and the number of files,
//! little-endian like the rest, then 16 bytes for each file: where it starts
//! in the .DAT, less its low byte, how long it is there and once unpacked,
//! three bytes that aren't 0 when it is packed, and the low byte of where it
//! starts. Then the names, as a tree of 12 bytes a node, the length of the
//! text of the names and the text, and after it a hash of each file's whole
//! path, in the files' order: FNV-1a over the path in capitals with
//! backslashes, with 0x199933 as its prime instead of FNV's own. So a file
//! is found by its path's hash, without walking the tree. Every one of the
//! 19,000 paths of `GAME.HDR` hashes to its file's place, checked
//! 4 November 2026.
//!
//! A packed file is a run of chunks, each "DFLT", its packed and its
//! unpacked length, and the chunk: kept whole when the two lengths are the
//! same, otherwise raw deflate (RFC 1951) of at most 32 KB in a single
//! block, whose type has stored and dynamic swapped: the two bits RFC 1951
//! gives 2 for a block with its own Huffman codes read 0 here. With that put
//! back, every deflated chunk of the GAME archives, 270,129 in 22,817 files,
//! inflates to its unpacked length; none was 2. Worked out from the game's
//! files, 4 November 2026. The INSTALL0_ archives beside them hold a cache
//! of each character's own files, nothing needed here, and aren't read;
//! where their lists put a packed file there is no chunk, so they keep
//! where their files start some other way.
//!
//! Which picture is which toy is the game's own: `chars\collection.txt`
//! gives each character and vehicle the number on its toy tag,
//! `Collect "Batman" portal_model_id 1`, the numbers Cemu's and RPCS3's
//! figure lists use, and its picture on the character wheel is
//! `icons\characters\<name>_nxg.tex`, a DDS 128 pixels square in DXT1 with
//! one bit of alpha. A vehicle has a number and a picture for each of its
//! three builds. Seen in Bertram's copy of the European release, 4 November
//! 2026: the table numbers 44 minifigures (1 to 44) and 144 vehicle builds,
//! and every one has its picture; its one other number, 950, is for a
//! screenshot and has none. The game draws the same pictures from a texture
//! page it packs them into, `charactericontpage_nxg.tsh`; this reads each
//! from its own file, whole.

use flate2::read::DeflateDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// LEGO Dimensions on the PS3: the European release and the American one.
const TITLE_IDS: [&str; 2] = ["BLES02105", "BLUS31473"];
const USRDIR: &str = "PS3_GAME/USRDIR";
/// The archives the game's own files are in, `GAME.HDR`, `GAME0.HDR` and on.
const ARCHIVES: &str = "GAME";
const LIST: &str = ".HDR";
const DATA: &str = ".DAT";
const COLLECTION: &str = "chars\\collection.txt";
const COLLECT: &str = "collect";
const NUMBER: &str = "portal_model_id";
const COMMENT: &str = "//";
const ICONS: &str = "icons\\characters\\";
const ICON_ENDING: &str = "_nxg.tex";
/// Pictures this copy has of figures its table doesn't number yet, matched
/// to Cemu's figure list by name: Harry Potter and Hermione Granger, whose
/// pictures came on the 2015 disc named for the game's second year,
/// "zeus2", and Lord Vortech, whose number Cemu's list keeps for a figure
/// that was never sold. Seen in Bertram's copy of the European release,
/// 4 November 2026. Where a copy's table numbers one of them itself, the
/// table's picture is written.
const NAMED_ONLY: [(&str, u32); 3] = [("zeus2_harrypotter", 52), ("zeus2_hermionegranger", 67), ("lordvortech", 73)];

const VERSIONS: std::ops::RangeInclusive<i32> = -9..=-5;
const ENTRY: usize = 16;
const NAME_NODE: usize = 12;
const FNV_OFFSET: u32 = 0x811c_9dc5;
const TT_PRIME: u32 = 0x0019_9933;
const CHUNK: &[u8; 4] = b"DFLT";
const CHUNK_HEAD: usize = 12;
/// The bits of a deflate stream's first byte that give its first block's
/// type (RFC 1951, 3.2.3), and the two types the game swaps: RFC 1951's
/// dynamic block, which the game numbers 0, its stored block.
const BLOCK_TYPE: u8 = 0b110;
const DYNAMIC: u8 = 2;
const STORED_HERE: u8 = 0;

/// A DDS file is "DDS ", a header of 124 bytes and the pixels; in the
/// header the height is at 12 and the width at 16 of the file, and the
/// pixel format's FourCC at 84. As Microsoft documents DDS_HEADER and
/// DDS_PIXELFORMAT (read 4 November 2026).
const DDS: &[u8; 4] = b"DDS ";
const DDS_HEADER: usize = 124;
const DDS_PIXELS: usize = 128;
const DXT1: &[u8; 4] = b"DXT1";
const DXT5: &[u8; 4] = b"DXT5";
const DXT1_BLOCK: usize = 8;
const DXT5_BLOCK: usize = 16;

/// Whether a PS3 title id, as its PARAM.SFO gives it, is LEGO Dimensions.
pub fn knows(title_id: &str) -> bool {
    TITLE_IDS.iter().any(|id| id.eq_ignore_ascii_case(title_id))
}

/// Writes the picture of each toy the game numbers, `<number>.png`, cut to
/// the square around its shape with its alpha. Gives how many were written.
pub fn pictures(game: &Path, out: &Path, progress: &mut dyn FnMut(usize, usize)) -> Result<usize, String> {
    let archives = archives(&crate::game_folder(game).join(USRDIR));
    if archives.is_empty() {
        return Err("Couldn't find the game's archives in this folder.".to_string());
    }
    let table = find(&archives, COLLECTION).ok_or("This copy of LEGO Dimensions has no list of its toys Omoio can read.")?;
    let numbered = collection(&String::from_utf8_lossy(&table));
    std::fs::create_dir_all(out).map_err(|_| "Couldn't make the folder for the pictures.".to_string())?;
    let toys: Vec<(&str, u32)> = numbered.iter().map(|(name, number)| (name.as_str(), *number)).chain(NAMED_ONLY).collect();
    let steps = toys.len();
    let mut written = Vec::new();
    for (done, &(name, number)) in toys.iter().enumerate() {
        progress(done, steps);
        if written.contains(&number) {
            continue;
        }
        let Some((width, height, rgba)) = find(&archives, &format!("{ICONS}{name}{ICON_ENDING}")).and_then(|bytes| dds(&bytes)) else {
            continue;
        };
        let (square, side) = crate::around_shape(&rgba, width, height);
        crate::write(&out.join(format!("{number}.png")), side, side, &square)?;
        written.push(number);
    }
    progress(steps, steps);
    Ok(written.len())
}

/// Each name the table numbers, with its number, in the table's order.
/// Lines without a number, the characters that aren't toys, are left out.
fn collection(text: &str) -> Vec<(String, u32)> {
    let mut numbered = Vec::new();
    for line in text.lines() {
        let line = line.split(COMMENT).next().unwrap_or_default().trim();
        let Some((word, rest)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let Some((name, rest)) = rest.trim_start().strip_prefix('"').and_then(|rest| rest.split_once('"')) else {
            continue;
        };
        if !word.eq_ignore_ascii_case(COLLECT) {
            continue;
        }
        let mut words = rest.split_whitespace();
        let number = words.by_ref().find(|word| word.eq_ignore_ascii_case(NUMBER)).and_then(|_| words.next()?.parse().ok());
        if let Some(number) = number {
            numbered.push((name.to_ascii_lowercase(), number));
        }
    }
    numbered
}

struct Archive {
    data: PathBuf,
    files: HashMap<u32, Entry>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Entry {
    offset: u64,
    stored: usize,
    size: usize,
    packed: bool,
}

/// The game's archives that read, in the order of their names.
fn archives(usrdir: &Path) -> Vec<Archive> {
    let Ok(folder) = std::fs::read_dir(usrdir) else {
        return Vec::new();
    };
    let mut lists: Vec<PathBuf> = folder
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default().to_ascii_uppercase();
            name.starts_with(ARCHIVES) && name.ends_with(LIST)
        })
        .collect();
    lists.sort();
    lists
        .into_iter()
        .filter_map(|list| {
            let files = index(&std::fs::read(&list).ok()?)?;
            let stem = list.file_stem()?.to_str()?;
            let data = list.with_file_name(format!("{stem}{DATA}"));
            data.is_file().then_some(Archive { data, files })
        })
        .collect()
}

/// An archive's files by the hash of their paths.
fn index(list: &[u8]) -> Option<HashMap<u32, Entry>> {
    if !VERSIONS.contains(&(le32(list, 4)? as i32)) {
        return None;
    }
    let count = le32(list, 8)? as usize;
    let names_at = 12 + count * ENTRY;
    let text_at = names_at + 4 + le32(list, names_at)? as usize * NAME_NODE;
    let hashes_at = text_at + 4 + le32(list, text_at)? as usize;
    list.get(hashes_at..hashes_at + count * 4)?;
    let mut files = HashMap::with_capacity(count);
    for index in 0..count {
        let at = 12 + index * ENTRY;
        let entry = list.get(at..at + ENTRY)?;
        let offset = u64::from(le32(entry, 0)?) << 8 | u64::from(entry[15]);
        let packed = entry[12..15] != [0, 0, 0];
        let entry = Entry { offset, stored: le32(entry, 4)? as usize, size: le32(entry, 8)? as usize, packed };
        files.insert(le32(list, hashes_at + index * 4)?, entry);
    }
    Some(files)
}

/// A file's bytes, unpacked, from the first archive that has it. `None`
/// when none has it or it doesn't read.
fn find(archives: &[Archive], path: &str) -> Option<Vec<u8>> {
    let key = hash(path);
    archives.iter().find_map(|archive| Some((archive, *archive.files.get(&key)?))).and_then(|(archive, entry)| read(archive, entry))
}

fn read(archive: &Archive, entry: Entry) -> Option<Vec<u8>> {
    let mut file = File::open(&archive.data).ok()?;
    file.seek(SeekFrom::Start(entry.offset)).ok()?;
    let mut bytes = Vec::new();
    file.take(entry.stored as u64).read_to_end(&mut bytes).ok()?;
    if bytes.len() != entry.stored {
        return None;
    }
    if entry.packed {
        unpack(&bytes, entry.size)
    } else {
        (entry.stored == entry.size).then_some(bytes)
    }
}

fn hash(path: &str) -> u32 {
    path.bytes().fold(FNV_OFFSET, |hash, byte| {
        let byte = if byte == b'/' { b'\\' } else { byte.to_ascii_uppercase() };
        (hash ^ u32::from(byte)).wrapping_mul(TT_PRIME)
    })
}

/// A packed file's chunks, unpacked, when they come to its size.
fn unpack(bytes: &[u8], size: usize) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes.get(at..at + 4)? != CHUNK {
            return None;
        }
        let (packed, unpacked) = (le32(bytes, at + 4)? as usize, le32(bytes, at + 8)? as usize);
        let chunk = bytes.get(at + CHUNK_HEAD..at + CHUNK_HEAD + packed)?;
        if packed == unpacked {
            out.extend_from_slice(chunk);
        } else {
            out.extend(inflate(chunk, unpacked)?);
        }
        at += CHUNK_HEAD + packed;
    }
    (out.len() == size).then_some(out)
}

/// A deflated chunk, its block type put back as RFC 1951 numbers it.
fn inflate(chunk: &[u8], size: usize) -> Option<Vec<u8>> {
    let mut chunk = chunk.to_vec();
    let first = chunk.first_mut()?;
    let kind = match (*first & BLOCK_TYPE) >> 1 {
        STORED_HERE => DYNAMIC,
        DYNAMIC => STORED_HERE,
        kind => kind,
    };
    *first = *first & !BLOCK_TYPE | kind << 1;
    let mut out = Vec::new();
    DeflateDecoder::new(chunk.as_slice()).take(size as u64 + 1).read_to_end(&mut out).ok()?;
    (out.len() == size).then_some(out)
}

/// A DDS picture in DXT1 or DXT5, the two the game's interface uses, as
/// its width, height and RGBA pixels, top row first.
fn dds(bytes: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    if bytes.get(..4)? != DDS || le32(bytes, 4)? as usize != DDS_HEADER {
        return None;
    }
    let (height, width) = (le32(bytes, 12)? as usize, le32(bytes, 16)? as usize);
    let (wide, high) = (width.div_ceil(4), height.div_ceil(4));
    let pixels = bytes.get(DDS_PIXELS..)?;
    let format = bytes.get(84..88)?;
    let rows = if format == DXT1 {
        dxt1(pixels.get(..wide * high * DXT1_BLOCK)?, wide * 4, high * 4)
    } else if format == DXT5 {
        crate::dxt5::decode(pixels.get(..wide * high * DXT5_BLOCK)?, wide * 4, high * 4)
    } else {
        return None;
    };
    let rgba = rows.chunks(wide * 16).take(height).flat_map(|row| &row[..width * 4]).copied().collect();
    Some((width, height, rgba))
}

/// DXT1 (BC1): each 4 x 4 block is 8 bytes, two RGB565 ends and a 2-bit
/// choice per pixel, little-endian. When the first end is the larger number
/// the two choices between them are blends at a third and two thirds;
/// otherwise the third is the halfway blend and the fourth is clear, which
/// is how the game's icons keep their one bit of alpha. As Microsoft
/// documents BC1 (read 4 November 2026).
fn dxt1(blocks: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut pixels = vec![0; width * height * 4];
    let blocks_wide = width / 4;
    for (index, block) in blocks.chunks_exact(DXT1_BLOCK).take(blocks_wide * (height / 4)).enumerate() {
        let (left, top) = (index % blocks_wide * 4, index / blocks_wide * 4);
        let (end0, end1) = (u16::from_le_bytes([block[0], block[1]]), u16::from_le_bytes([block[2], block[3]]));
        let (c0, c1) = (rgb565(end0), rgb565(end1));
        let blend = |a: u32, b: u32, of: u32| [0, 1, 2].map(|c| (a * c0[c] + b * c1[c]) / of);
        let colours = if end0 > end1 {
            [[c0[0], c0[1], c0[2], 255], [c1[0], c1[1], c1[2], 255], with_alpha(blend(2, 1, 3), 255), with_alpha(blend(1, 2, 3), 255)]
        } else {
            [[c0[0], c0[1], c0[2], 255], [c1[0], c1[1], c1[2], 255], with_alpha(blend(1, 1, 2), 255), [0, 0, 0, 0]]
        };
        let choices = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
        for pixel in 0..16 {
            let (x, y) = (left + pixel % 4, top + pixel / 4);
            let colour = colours[(choices >> (2 * pixel) & 3) as usize];
            let at = (y * width + x) * 4;
            pixels[at..at + 4].copy_from_slice(&colour.map(|channel| channel as u8));
        }
    }
    pixels
}

fn with_alpha([r, g, b]: [u32; 3], alpha: u32) -> [u32; 4] {
    [r, g, b, alpha]
}

fn rgb565(colour: u16) -> [u32; 3] {
    let expand = |value: u16, bits: u32| {
        let max = (1u32 << bits) - 1;
        (u32::from(value) * 255 + max / 2) / max
    };
    [expand(colour >> 11, 5), expand(colour >> 5 & 63, 6), expand(colour & 31, 5)]
}

fn le32(bytes: &[u8], at: usize) -> Option<u32> {
    bytes.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn knows_both_releases() {
        assert!(knows("BLES02105"));
        assert!(knows("BLUS31473"));
        assert!(!knows("BLES01844"));
    }

    #[test]
    fn a_path_hashes_as_the_archive_lists_it() {
        // As GAME.HDR of the European release lists its toy table.
        assert_eq!(hash("chars\\collection.txt"), 0x874c_a1b4);
        assert_eq!(hash("CHARS/COLLECTION.TXT"), 0x874c_a1b4);
    }

    #[test]
    fn a_list_finds_each_file_by_its_hash() {
        let mut list = vec![0, 0, 0, 0];
        list.extend_from_slice(&(-7i32).to_le_bytes());
        list.extend_from_slice(&2u32.to_le_bytes());
        for (offset, low, stored, size, packed) in [(0x12_3456u32, 0x78u8, 10u32, 10u32, 0u8), (0x100, 0, 30, 70, 3)] {
            list.extend_from_slice(&offset.to_le_bytes());
            list.extend_from_slice(&stored.to_le_bytes());
            list.extend_from_slice(&size.to_le_bytes());
            list.extend_from_slice(&[packed, 0, 0, low]);
        }
        // One node of the names' tree, and their text.
        list.extend_from_slice(&1u32.to_le_bytes());
        list.extend_from_slice(&[0; NAME_NODE]);
        list.extend_from_slice(&4u32.to_le_bytes());
        list.extend_from_slice(b"abc\0");
        list.extend_from_slice(&hash("a\\b").to_le_bytes());
        list.extend_from_slice(&hash("a\\c").to_le_bytes());
        let files = index(&list).unwrap();
        assert_eq!(files[&hash("a/b")], Entry { offset: 0x1234_5678, stored: 10, size: 10, packed: false });
        assert_eq!(files[&hash("A\\C")], Entry { offset: 0x1_0000, stored: 30, size: 70, packed: true });
        list[4..8].copy_from_slice(&(-11i32).to_le_bytes());
        assert!(index(&list).is_none());
    }

    #[test]
    fn chunks_unpack_with_their_block_type_put_back() {
        let text: Vec<u8> = b"Batman, Gandalf and Wyldstyle. ".iter().copied().cycle().take(5000).collect();
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&text).unwrap();
        let mut deflated = encoder.finish().unwrap();
        // The encoder writes a dynamic block, which the game numbers 0.
        assert_eq!((deflated[0] & BLOCK_TYPE) >> 1, DYNAMIC);
        deflated[0] &= !BLOCK_TYPE;
        let whole = b"kept whole";
        let mut file = Vec::new();
        for (chunk, size) in [(deflated.as_slice(), text.len()), (whole.as_slice(), whole.len())] {
            file.extend_from_slice(CHUNK);
            file.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
            file.extend_from_slice(&(size as u32).to_le_bytes());
            file.extend_from_slice(chunk);
        }
        let mut expected = text.clone();
        expected.extend_from_slice(whole);
        assert_eq!(unpack(&file, expected.len()).unwrap(), expected);
        assert!(unpack(&file, expected.len() + 1).is_none());
        // Read as RFC 1951 numbers it, the chunk is a stored block, and wrong.
        assert!(DeflateDecoder::new(deflated.as_slice()).read_to_end(&mut Vec::new()).map_or(true, |read| read != text.len()));
    }

    #[test]
    fn a_dxt1_block_keeps_its_one_bit_of_alpha() {
        // Red then white: the first end is the smaller number, so the
        // fourth choice is clear. Pixel 0 takes choice 3, pixel 1 choice 2.
        let block = [0x00, 0xf8, 0xff, 0xff, 0b1011, 0, 0, 0];
        let pixels = dxt1(&block, 4, 4);
        assert_eq!(&pixels[0..4], [0, 0, 0, 0]);
        assert_eq!(&pixels[4..8], [255, 127, 127, 255]);
        assert_eq!(&pixels[8..12], [255, 0, 0, 255]);
        // White then red: four solid colours, choice 3 two thirds of the way to red.
        let block = [0xff, 0xff, 0x00, 0xf8, 0b11, 0, 0, 0];
        assert_eq!(&dxt1(&block, 4, 4)[0..4], [255, 85, 85, 255]);
    }

    #[test]
    fn the_collection_numbers_each_toy() {
        let text = "PermLoadCollectionIcons\r\n\
            // Collect \"Hidden\" portal_model_id 99\r\n\
            Collect \"Batman\" \t\t\tportal_model_id 1 \tcharacter_tag\t\t///< DC\r\n\
            collect \"ti_badCopCar_2\"\t\t\tportal_model_id 1001\t\t\tcan_be_upgraded_by BadCop\r\n\
            collect \"Doctor1st\"\t\t\tportal_alternate_model\r\n\
            collect \"Hub_Balrog\"\t\t\tDo_Not_Show no_progress\r\n";
        assert_eq!(collection(text), [("batman".to_string(), 1), ("ti_badcopcar_2".to_string(), 1001)]);
    }

    /// Reads the copy of the game named by OMOIO_LEGO_DIMENSIONS, the
    /// folder that holds PS3_GAME, into a folder of its own in the system's
    /// temporary folder. Never runs without being asked for.
    #[test]
    #[ignore]
    fn reads_a_real_copy() {
        let game = std::env::var("OMOIO_LEGO_DIMENSIONS").expect("Set OMOIO_LEGO_DIMENSIONS to a copy of the game.");
        let out = std::env::temp_dir().join("omoio-portraits-dimensions");
        let mut steps = Vec::new();
        let written = pictures(Path::new(&game), &out, &mut |done, of| steps.push((done, of))).unwrap();
        assert!(written >= 188, "wrote {written}");
        assert_eq!(steps.last(), Some(&(steps[0].1, steps[0].1)));
        for number in [1, 44, 1000, 1169] {
            assert!(out.join(format!("{number}.png")).is_file(), "no {number}.png");
        }
    }
}
