//! One picture in the game's own format (.igz, version 7), read only as far
//! as a portrait needs: the name of the picture it was made from, its pixel
//! format, its size, and its pixels. The file lists its sections from 0x18
//! (offset, size, alignment); the first holds tables that name things, the
//! third is the picture's object, and the last holds the pixels after
//! padding as long as its alignment. Everything is big-endian.
//!
//! Skylanders Trap Team writes version 8: the same section list, but three
//! sections, the object being the second, with the size at 0xC0 in it, and
//! no table of strings. Its pictures are named by the archive instead.
//!
//! Skylanders Giants on the PS3 writes version 6, which keeps a whole screen
//! in one file: its objects in one section, the pixels of all its pictures
//! in another, and its strings in the last. The sections are listed from
//! 0x10, the first holding numbered tables. A pointer gives the section in
//! its top byte, counting from the objects', and the offset in the rest.
//! Table 0 names the kinds of object, 2 lists outside things by a pair of
//! hashes, the name's first, 5 says where each object starts, and 10 lists
//! blocks of memory, a size and a pointer each. A picture, `igImage2`, keeps
//! its name at 0x08, its size at 0x30, its pixel format at 0x3C, as a number
//! in table 2, and its pixels at 0x48, as a number in table 10. Worked out
//! from the game's files.

const MAGIC: u32 = 0x4947_5a01;
const GIANTS: u32 = 6;
const SWAP_FORCE: u32 = 7;
const TRAP_TEAM: u32 = 8;
const SECTIONS: usize = 0x18;
const GIANTS_SECTIONS: usize = 0x10;
const KINDS: u32 = 0;
const OUTSIDE_THINGS: u32 = 2;
const OBJECTS: u32 = 5;
const MEMORY: u32 = 10;
const PICTURE: &[u8] = b"igImage2";
const PICTURE_NAME: usize = 0x08;
const PICTURE_SIZE: usize = 0x30;
const PICTURE_FORMAT: usize = 0x3c;
const PICTURE_PIXELS: usize = 0x48;
/// The table names are four letters stored back to front: "TSTR" is the
/// table of strings, "EXID" the list of outside things, the pixel format
/// first.
const STRINGS: u32 = u32::from_le_bytes(*b"TSTR");
const OUTSIDE: u32 = u32::from_le_bytes(*b"EXID");
/// Which section is the picture's object, and where its width and height sit
/// in it, one after the other as 16-bit numbers, for each version.
const SWAP_FORCE_OBJECT: (usize, usize) = (2, 0x40);
const TRAP_TEAM_OBJECT: (usize, usize) = (1, 0xc0);

pub struct Picture {
    /// The file it was made from, such as `Spyro2012_WiiPortrait`, or
    /// `airdragon` in Giants. Empty for Trap Team, which doesn't keep it.
    pub source: String,
    pub format: u32,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

struct Section {
    offset: usize,
    size: usize,
    align: usize,
}

fn be16(bytes: &[u8], at: usize) -> Option<usize> {
    bytes.get(at..at + 2).map(|b| usize::from(u16::from_be_bytes([b[0], b[1]])))
}

fn be32(bytes: &[u8], at: usize) -> Option<u32> {
    bytes.get(at..at + 4).map(|b| u32::from_be_bytes(b.try_into().unwrap()))
}

/// The file's version, known here or not. `None` when it isn't in the
/// game's own format.
pub fn version(bytes: &[u8]) -> Option<u32> {
    if be32(bytes, 0)? != MAGIC {
        return None;
    }
    be32(bytes, 4)
}

/// Whether files of this version are read here, as single pictures or as
/// Giants' screens.
pub fn known(version: u32) -> bool {
    matches!(version, GIANTS | SWAP_FORCE | TRAP_TEAM)
}

pub fn read(bytes: &[u8]) -> Option<Picture> {
    let (object_section, size_in_object) = match version(bytes)? {
        SWAP_FORCE => SWAP_FORCE_OBJECT,
        TRAP_TEAM => TRAP_TEAM_OBJECT,
        _ => return None,
    };
    let mut sections = Vec::new();
    let mut at = SECTIONS;
    while be32(bytes, at)? != 0 {
        sections.push(Section {
            offset: be32(bytes, at)? as usize,
            size: be32(bytes, at + 4)? as usize,
            align: be32(bytes, at + 8)? as usize,
        });
        at += 16;
    }
    // The pixels come after the object, in a section of their own.
    if sections.len() < object_section + 2 {
        return None;
    }
    let (tables, object, pixels) = (&sections[0], &sections[object_section], &sections[sections.len() - 1]);

    let mut source = None;
    let mut format = None;
    let mut at = tables.offset;
    while at < tables.offset + tables.size {
        let (kind, length, start) = (be32(bytes, at)?, be32(bytes, at + 8)? as usize, be32(bytes, at + 12)? as usize);
        if kind == STRINGS {
            let text = bytes.get(at + start..)?;
            let end = text.iter().position(|&b| b == 0)?;
            source = Some(String::from_utf8_lossy(&text[..end]).into_owned());
        } else if kind == OUTSIDE {
            format = be32(bytes, at + start);
        }
        if length == 0 {
            break;
        }
        at += length;
    }
    // The strings table gives the whole path the picture was made from.
    let path = source.unwrap_or_default();

    Some(Picture {
        source: stem(&path).to_string(),
        format: format?,
        width: be16(bytes, object.offset + size_in_object)?,
        height: be16(bytes, object.offset + size_in_object + 2)?,
        pixels: bytes.get(pixels.offset + pixels.align..pixels.offset + pixels.size)?.to_vec(),
    })
}

/// A path's file name without its folders and extension.
fn stem(path: &str) -> &str {
    let name = path.rsplit(['\\', '/']).next().unwrap_or_default();
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

/// The text a pointer leads to, up to its end.
fn text(bytes: &[u8], at: usize) -> Option<&str> {
    let text = bytes.get(at..)?;
    std::str::from_utf8(&text[..text.iter().position(|&b| b == 0)?]).ok()
}

/// Where each object starts in its section, from Giants' table 5. Each start
/// is the one before plus four, plus four times a number written three bits
/// to a nibble, low nibble first, with a nibble's top bit saying another
/// follows. Counting from 0, the first object starts at 4 or later, as a
/// pointer of 0 means none.
fn object_starts(packed: &[u8], count: usize) -> Option<Vec<usize>> {
    let mut nibbles = packed.iter().flat_map(|&byte| [byte & 15, byte >> 4]);
    let mut at = 0;
    (0..count)
        .map(|_| {
            let (mut fours, mut shift) = (0, 0);
            loop {
                let nibble = nibbles.next()?;
                fours |= usize::from(nibble & 7) << shift;
                shift += 3;
                if nibble & 8 == 0 {
                    break;
                }
            }
            at += fours * 4 + 4;
            Some(at)
        })
        .collect()
}

/// Every picture in one of Giants' screens, which the game keeps whole in
/// one file. `None` when the file isn't one; pictures that don't read are
/// left out.
pub fn screen(bytes: &[u8]) -> Option<Vec<Picture>> {
    if version(bytes)? != GIANTS {
        return None;
    }
    let mut sections = Vec::new();
    let mut at = GIANTS_SECTIONS;
    while be32(bytes, at)? != 0 {
        sections.push(be32(bytes, at)? as usize);
        at += 16;
    }
    let (&tables, pools) = sections.split_first()?;
    let pointer = |value: u32| Some(pools.get(value as usize >> 24)? + (value as usize & 0xff_ffff));
    // Each table: its number, two words, how many entries it has, its
    // length, and where in it they start.
    let table = |number: u32| {
        let mut at = tables + be32(bytes, tables + 0x14)? as usize;
        for _ in 0..be32(bytes, tables + 0x10)? {
            if be32(bytes, at)? == number {
                return Some((at + be32(bytes, at + 20)? as usize, be32(bytes, at + 12)? as usize));
            }
            at += be32(bytes, at + 16)? as usize;
        }
        None
    };

    let (names, kinds) = table(KINDS)?;
    let mut at = names;
    let mut picture_kind = None;
    for kind in 0..kinds as u32 {
        let name = bytes.get(at..)?.split(|&b| b == 0).next()?;
        if name == PICTURE {
            picture_kind = Some(kind);
            break;
        }
        at += name.len() + 1;
    }
    let Some(picture_kind) = picture_kind else {
        return Some(Vec::new());
    };
    let (packed, count) = table(OBJECTS)?;
    let starts = object_starts(bytes.get(packed..)?, count)?;
    let (outside, _) = table(OUTSIDE_THINGS)?;
    let (memory, _) = table(MEMORY)?;
    let objects = *pools.first()?;
    let picture = |object: usize| {
        let name = text(bytes, pointer(be32(bytes, object + PICTURE_NAME)?)?)?;
        let format = be32(bytes, outside + (be32(bytes, object + PICTURE_FORMAT)? as usize & 0xff_ffff) * 8)?;
        let block = memory + be32(bytes, object + PICTURE_PIXELS)? as usize * 8;
        let start = pointer(be32(bytes, block + 4)?)?;
        let length = be32(bytes, block)? as usize & 0xff_ffff;
        Some(Picture {
            source: stem(name).to_string(),
            format,
            width: be16(bytes, object + PICTURE_SIZE)?,
            height: be16(bytes, object + PICTURE_SIZE + 2)?,
            pixels: bytes.get(start..start + length)?.to_vec(),
        })
    };
    Some(
        starts
            .into_iter()
            .map(|start| objects + start)
            .filter(|&object| be32(bytes, object) == Some(picture_kind))
            .filter_map(picture)
            .collect(),
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A picture file laid out the way the game's are, with `pixels` for
    /// its pixel data. Trap Team's have no strings and no list section, so
    /// `path` is left out for them.
    pub(crate) fn build(version: u32, path: Option<&str>, format: u32, width: u16, height: u16, pixels: &[u8]) -> Vec<u8> {
        let mut out = vec![0; 0x800];
        out[0..4].copy_from_slice(&MAGIC.to_be_bytes());
        out[4..8].copy_from_slice(&version.to_be_bytes());

        let tables = out.len();
        let mut entries = vec![(OUTSIDE, [format.to_be_bytes(), [0; 4]].concat())];
        if let Some(path) = path {
            let mut strings = path.as_bytes().to_vec();
            strings.push(0);
            while strings.len() % 4 != 0 {
                strings.push(0);
            }
            entries.insert(0, (STRINGS, strings));
        }
        for (kind, body) in entries {
            let length = 16 + body.len();
            out.extend_from_slice(&kind.to_be_bytes());
            out.extend_from_slice(&1u32.to_be_bytes());
            out.extend_from_slice(&(length as u32).to_be_bytes());
            out.extend_from_slice(&16u32.to_be_bytes());
            out.extend_from_slice(&body);
        }
        let tables_size = out.len() - tables;
        let mut sections = vec![(tables, tables_size, 0)];
        if version == SWAP_FORCE {
            sections.push((out.len(), 0x40, 4));
            out.extend_from_slice(&[0; 0x40]);
        }
        let size_at = if version == SWAP_FORCE { SWAP_FORCE_OBJECT.1 } else { TRAP_TEAM_OBJECT.1 };
        let object = out.len();
        out.extend_from_slice(&[0; 0x100]);
        out[object + size_at..object + size_at + 2].copy_from_slice(&width.to_be_bytes());
        out[object + size_at + 2..object + size_at + 4].copy_from_slice(&height.to_be_bytes());
        sections.push((object, 0x100, 16));
        let align = 0x200;
        sections.push((out.len(), align + pixels.len(), align));
        out.extend_from_slice(&vec![0; align]);
        out.extend_from_slice(pixels);

        for (index, (offset, size, alignment)) in sections.into_iter().enumerate() {
            let at = SECTIONS + index * 16;
            out[at..at + 4].copy_from_slice(&(offset as u32).to_be_bytes());
            out[at + 4..at + 8].copy_from_slice(&(size as u32).to_be_bytes());
            out[at + 8..at + 12].copy_from_slice(&(alignment as u32).to_be_bytes());
        }
        out
    }

    #[test]
    fn a_picture_gives_its_name_format_size_and_pixels() {
        let pixels: Vec<u8> = (0..64).collect();
        let path = "U:\\data\\textures\\ui\\Spyro2012_WiiPortrait.tga";
        let bytes = build(SWAP_FORCE, Some(path), 0x98cb_2a65, 256, 128, &pixels);
        let picture = read(&bytes).unwrap();
        assert_eq!(picture.source, "Spyro2012_WiiPortrait");
        assert_eq!(picture.format, 0x98cb_2a65);
        assert_eq!((picture.width, picture.height), (256, 128));
        assert_eq!(picture.pixels, pixels);
    }

    #[test]
    fn a_trap_team_picture_gives_its_format_size_and_pixels() {
        let pixels: Vec<u8> = (0..96).collect();
        let bytes = build(TRAP_TEAM, None, 0x98cb_2a65, 171, 171, &pixels);
        let picture = read(&bytes).unwrap();
        assert_eq!(picture.source, "");
        assert_eq!(picture.format, 0x98cb_2a65);
        assert_eq!((picture.width, picture.height), (171, 171));
        assert_eq!(picture.pixels, pixels);
    }

    #[test]
    fn a_file_tells_its_version_even_when_not_known() {
        assert_eq!(version(&build(TRAP_TEAM, None, 0, 4, 4, &[0; 16])), Some(TRAP_TEAM));
        assert_eq!(version(b"IGZ\x01\x00\x00\x00\x09"), Some(9));
        assert!(known(GIANTS) && known(TRAP_TEAM) && !known(9));
        assert!(read(b"IGZ\x01\x00\x00\x00\x09 and a version not known").is_none());
        assert_eq!(version(b"IGA\x1a"), None);
    }

    #[test]
    fn other_files_are_not_pictures() {
        assert!(read(b"IGA\x1a nothing like a picture").is_none());
        assert!(read(&[0; 16]).is_none());
        assert!(screen(b"IGA\x1a nothing like a screen").is_none());
    }

    #[test]
    fn objects_start_where_giants_says() {
        // The first bytes of the list in the game's Collection screen, whose
        // first objects are a list at 4 and others at 0x3F4, 0x410 and 0x434.
        assert_eq!(object_starts(&[0xb0, 0x3f, 0x86, 0x61], 4), Some(vec![4, 0x3f4, 0x410, 0x434]));
        assert_eq!(object_starts(&[0x08], 2), None);
    }

    /// One of Giants' screens laid out as the game's are, with an object of
    /// another kind and then a picture named by `path`, its pixels `pixels`.
    fn build_screen(path: &str, format: u32, width: u16, height: u16, pixels: &[u8]) -> Vec<u8> {
        let word = |out: &mut Vec<u8>, value: u32| out.extend_from_slice(&value.to_be_bytes());
        let mut out = vec![0; 0x800];
        out[0..4].copy_from_slice(&MAGIC.to_be_bytes());
        out[4..8].copy_from_slice(&GIANTS.to_be_bytes());

        let tables = out.len();
        let entries: [(u32, u32, Vec<u8>); 4] = [
            (KINDS, 2, b"igObjectList\0igImage2\0\0\0".to_vec()),
            (OUTSIDE_THINGS, 1, [format.to_be_bytes(), 0x843c_d0c2u32.to_be_bytes()].concat()),
            // Objects at 4 and 0x1C: steps of no fours, then five.
            (OBJECTS, 2, vec![0x50, 0, 0, 0]),
            // The pixels: 0x80 into the pixels' section, as the game's start.
            (MEMORY, 1, [(0x2800_0000 | pixels.len() as u32).to_be_bytes(), 0x0100_0080u32.to_be_bytes()].concat()),
        ];
        out.extend_from_slice(&[0; 0x1c]);
        out[tables + 0x10..tables + 0x14].copy_from_slice(&(entries.len() as u32).to_be_bytes());
        out[tables + 0x14..tables + 0x18].copy_from_slice(&0x1cu32.to_be_bytes());
        for (number, count, body) in entries {
            word(&mut out, number);
            word(&mut out, 0);
            word(&mut out, 0);
            word(&mut out, count);
            word(&mut out, 24 + body.len() as u32);
            word(&mut out, 24);
            out.extend_from_slice(&body);
        }

        // The object at 4 is left as zeros, which makes it an igObjectList.
        let objects = out.len();
        out.extend_from_slice(&[0; 0x1c + 0x60]);
        let picture = objects + 0x1c;
        out[picture..picture + 4].copy_from_slice(&1u32.to_be_bytes());
        // Its name starts the strings' section, number 2 counting from the
        // objects'.
        out[picture + PICTURE_NAME..picture + PICTURE_NAME + 4].copy_from_slice(&0x0200_0000u32.to_be_bytes());
        out[picture + PICTURE_SIZE..picture + PICTURE_SIZE + 2].copy_from_slice(&width.to_be_bytes());
        out[picture + PICTURE_SIZE + 2..picture + PICTURE_SIZE + 4].copy_from_slice(&height.to_be_bytes());
        out[picture + PICTURE_FORMAT..picture + PICTURE_FORMAT + 4].copy_from_slice(&0x8000_0000u32.to_be_bytes());

        let pixels_at = out.len();
        out.extend_from_slice(&[0; 0x80]);
        out.extend_from_slice(pixels);
        let strings = out.len();
        out.extend_from_slice(path.as_bytes());
        out.push(0);

        for (index, offset) in [tables, objects, pixels_at, strings].into_iter().enumerate() {
            let at = GIANTS_SECTIONS + index * 16;
            out[at..at + 4].copy_from_slice(&(offset as u32).to_be_bytes());
        }
        out
    }

    #[test]
    fn a_giants_screen_gives_each_picture_with_its_name() {
        let pixels: Vec<u8> = (0..=255).collect();
        let path = "levels/includes/ui_main/sprites/collections/sky2/portraits/airdragon.png";
        let pictures = screen(&build_screen(path, 0x942d_575f, 16, 16, &pixels)).unwrap();
        assert_eq!(pictures.len(), 1);
        assert_eq!(pictures[0].source, "airdragon");
        assert_eq!(pictures[0].format, 0x942d_575f);
        assert_eq!((pictures[0].width, pictures[0].height), (16, 16));
        assert_eq!(pictures[0].pixels, pixels);
    }
}
