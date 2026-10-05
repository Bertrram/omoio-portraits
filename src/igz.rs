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

const MAGIC: u32 = 0x4947_5a01;
const SWAP_FORCE: u32 = 7;
const TRAP_TEAM: u32 = 8;
const SECTIONS: usize = 0x18;
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
    /// The file it was made from, such as `Spyro2012_WiiPortrait`. Empty
    /// for Trap Team, which doesn't keep it.
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

pub fn read(bytes: &[u8]) -> Option<Picture> {
    if be32(bytes, 0)? != MAGIC {
        return None;
    }
    let (object_section, size_in_object) = match be32(bytes, 4)? {
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
    let name = path.rsplit('\\').next().unwrap_or_default();
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);

    Some(Picture {
        source: stem.to_string(),
        format: format?,
        width: be16(bytes, object.offset + size_in_object)?,
        height: be16(bytes, object.offset + size_in_object + 2)?,
        pixels: bytes.get(pixels.offset + pixels.align..pixels.offset + pixels.size)?.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture file laid out the way the game's are, with `pixels` for
    /// its pixel data. Trap Team's have no strings and no list section, so
    /// `path` is left out for them.
    fn build(version: u32, path: Option<&str>, format: u32, width: u16, height: u16, pixels: &[u8]) -> Vec<u8> {
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
    fn other_files_are_not_pictures() {
        assert!(read(b"IGA\x1a nothing like a picture").is_none());
        assert!(read(&[0; 16]).is_none());
    }
}
