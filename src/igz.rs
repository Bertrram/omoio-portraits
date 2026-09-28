//! One picture in the game's own format (.igz, version 7), read only as far
//! as a portrait needs: the name of the picture it was made from, its pixel
//! format, its size, and its pixels. The file lists its sections from 0x18
//! (offset, size, alignment); the first holds tables that name things, the
//! third is the picture's object, and the last holds the pixels after
//! padding as long as its alignment. Everything is big-endian.

const MAGIC: u32 = 0x4947_5a01;
const VERSION: u32 = 7;
const SECTIONS: usize = 0x18;
/// The table names are four letters stored back to front: "TSTR" is the
/// table of strings, "EXID" the list of outside things, the pixel format
/// first.
const STRINGS: u32 = u32::from_le_bytes(*b"TSTR");
const OUTSIDE: u32 = u32::from_le_bytes(*b"EXID");
/// Where the width and height sit in the picture's object, one after the
/// other as 16-bit numbers.
const SIZE_IN_OBJECT: usize = 0x40;

pub struct Picture {
    /// The file it was made from, such as `Spyro2012_WiiPortrait`.
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
    if be32(bytes, 0)? != MAGIC || be32(bytes, 4)? != VERSION {
        return None;
    }
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
    if sections.len() < 4 {
        return None;
    }
    let (tables, object, pixels) = (&sections[0], &sections[2], &sections[sections.len() - 1]);

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
    let path = source?;
    let name = path.rsplit('\\').next()?;
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);

    Some(Picture {
        source: stem.to_string(),
        format: format?,
        width: be16(bytes, object.offset + SIZE_IN_OBJECT)?,
        height: be16(bytes, object.offset + SIZE_IN_OBJECT + 2)?,
        pixels: bytes.get(pixels.offset + pixels.align..pixels.offset + pixels.size)?.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture file laid out the way the game's are, with `pixels` for
    /// its pixel data.
    fn build(path: &str, format: u32, width: u16, height: u16, pixels: &[u8]) -> Vec<u8> {
        let mut out = vec![0; 0x800];
        out[0..4].copy_from_slice(&MAGIC.to_be_bytes());
        out[4..8].copy_from_slice(&VERSION.to_be_bytes());

        let tables = out.len();
        let mut strings = path.as_bytes().to_vec();
        strings.push(0);
        while strings.len() % 4 != 0 {
            strings.push(0);
        }
        for (kind, body) in [(STRINGS, strings), (OUTSIDE, [format.to_be_bytes(), [0; 4]].concat())] {
            let length = 16 + body.len();
            out.extend_from_slice(&kind.to_be_bytes());
            out.extend_from_slice(&1u32.to_be_bytes());
            out.extend_from_slice(&(length as u32).to_be_bytes());
            out.extend_from_slice(&16u32.to_be_bytes());
            out.extend_from_slice(&body);
        }
        let tables_size = out.len() - tables;
        let list = out.len();
        out.extend_from_slice(&[0; 0x40]);
        let object = out.len();
        out.extend_from_slice(&[0; 0x70]);
        out[object + SIZE_IN_OBJECT..object + SIZE_IN_OBJECT + 2].copy_from_slice(&width.to_be_bytes());
        out[object + SIZE_IN_OBJECT + 2..object + SIZE_IN_OBJECT + 4].copy_from_slice(&height.to_be_bytes());
        let align = 0x200;
        let data = out.len();
        out.extend_from_slice(&vec![0; align]);
        out.extend_from_slice(pixels);

        for (index, (offset, size, alignment)) in [
            (tables, tables_size, 0),
            (list, 0x40, 4),
            (object, 0x70, 16),
            (data, align + pixels.len(), align),
        ]
        .into_iter()
        .enumerate()
        {
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
        let bytes = build("U:\\data\\textures\\ui\\Spyro2012_WiiPortrait.tga", 0x98cb_2a65, 256, 128, &pixels);
        let picture = read(&bytes).unwrap();
        assert_eq!(picture.source, "Spyro2012_WiiPortrait");
        assert_eq!(picture.format, 0x98cb_2a65);
        assert_eq!((picture.width, picture.height), (256, 128));
        assert_eq!(picture.pixels, pixels);
    }

    #[test]
    fn other_files_are_not_pictures() {
        assert!(read(b"IGA\x1a nothing like a picture").is_none());
        assert!(read(&[0; 16]).is_none());
    }
}
