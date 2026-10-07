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
//!
//! Skylanders SuperChargers writes version 9, whose files are read here as
//! the objects they hold (`Objects`), as its toy data needs that too. The
//! header gives the number of fixup tables at 0x10, and from 0x14 each
//! section's pool name, offset, size and alignment, the fixups first; a
//! picture's sections after them are Default, ImageObject, String and
//! Image. The fixup tables are as in version 7. TMET names the kinds of
//! object and TSTR holds strings, each padded to an even length; EXNM gives
//! each outside name as the number of its string with the top bit set; and
//! RVTB says where each object starts, as numbers packed three bits to a
//! nibble, a nibble's top bit saying another follows, each object four
//! times its number on from the one before. A pointer gives the section in
//! its top five bits, counting from the first after the fixups, and the
//! offset in the rest; a handle to an outside name has its top bit set and
//! the name's number in the rest. A picture, `igImage2`, keeps its width
//! and height at 0x34 and its pixels are the whole last section. Worked out
//! from the game's files, 7 October 2026, with the pool names, the TMET and
//! TSTR tables and an RVTB that sums to the ImageObject section's start in
//! every picture of the game.

const MAGIC: u32 = 0x4947_5a01;
const GIANTS: u32 = 6;
const SWAP_FORCE: u32 = 7;
const TRAP_TEAM: u32 = 8;
const SUPERCHARGERS: u32 = 9;
const SECTIONS: usize = 0x18;
const GIANTS_SECTIONS: usize = 0x10;
const FIXUP_COUNT: usize = 0x10;
const KINDS: u32 = 0;
const OUTSIDE_THINGS: u32 = 2;
const OBJECTS: u32 = 5;
const MEMORY: u32 = 10;
const PICTURE: &str = "igImage2";
/// Where version 9's picture keeps its width and height.
const IMAGE_SIZE: usize = 0x34;
/// Version 9's fixup tables by name, stored back to front as the others.
const TYPES: u32 = u32::from_le_bytes(*b"TMET");
const OUTSIDE_NAMES: u32 = u32::from_le_bytes(*b"EXNM");
const OBJECT_STARTS: u32 = u32::from_le_bytes(*b"RVTB");
const POINTER_SECTION: u32 = 27;
const POINTER_OFFSET: u32 = (1 << POINTER_SECTION) - 1;
const HANDLE: u32 = 0x8000_0000;
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
    matches!(version, GIANTS | SWAP_FORCE | TRAP_TEAM | SUPERCHARGERS)
}

pub fn read(bytes: &[u8]) -> Option<Picture> {
    let (object_section, size_in_object) = match version(bytes)? {
        SWAP_FORCE => SWAP_FORCE_OBJECT,
        TRAP_TEAM => TRAP_TEAM_OBJECT,
        SUPERCHARGERS => return read_objects(bytes),
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

/// A picture of version 9, named by the archive as Trap Team's are.
fn read_objects(bytes: &[u8]) -> Option<Picture> {
    let objects = Objects::read(bytes)?;
    let image = objects.of_kind(PICTURE).next()?;
    let &(pixels, size) = objects.sections.last()?;
    Some(Picture {
        source: String::new(),
        format: objects.format?,
        width: usize::from(objects.half(image, IMAGE_SIZE)?),
        height: usize::from(objects.half(image, IMAGE_SIZE + 2)?),
        pixels: bytes.get(pixels..pixels + size)?.to_vec(),
    })
}

/// A file of version 9 as the objects it holds.
pub struct Objects<'a> {
    bytes: &'a [u8],
    /// Each section after the fixups: where it starts and how long it is.
    sections: Vec<(usize, usize)>,
    kinds: Vec<String>,
    strings: Vec<String>,
    /// For each outside name, the numbers of its string and of its
    /// namespace's.
    outside: Vec<(usize, usize)>,
    /// The first outside thing, a picture's pixel format.
    format: Option<u32>,
    /// Where each object starts in the file.
    starts: Vec<usize>,
}

impl<'a> Objects<'a> {
    pub fn read(bytes: &'a [u8]) -> Option<Self> {
        if version(bytes)? != SUPERCHARGERS {
            return None;
        }
        let mut sections = Vec::new();
        let mut at = SECTIONS;
        while be32(bytes, at)? != 0 {
            sections.push((be32(bytes, at)? as usize, be32(bytes, at + 4)? as usize));
            at += 16;
        }
        let (&(fixups, _), data) = sections.split_first()?;
        let mut objects = Objects {
            bytes,
            sections: data.to_vec(),
            kinds: Vec::new(),
            strings: Vec::new(),
            outside: Vec::new(),
            format: None,
            starts: Vec::new(),
        };
        let mut at = fixups;
        let mut starts = Vec::new();
        for _ in 0..be32(bytes, FIXUP_COUNT)? {
            let (tag, count) = (be32(bytes, at)?, be32(bytes, at + 4)? as usize);
            let (length, start) = (be32(bytes, at + 8)? as usize, be32(bytes, at + 12)? as usize);
            let body = bytes.get(at + start..at + length)?;
            match tag {
                TYPES => objects.kinds = padded_strings(body, count)?,
                STRINGS => objects.strings = padded_strings(body, count)?,
                OUTSIDE_NAMES => {
                    objects.outside = (0..count)
                        .map(|index| Some(((be32(body, index * 8)? & !HANDLE) as usize, be32(body, index * 8 + 4)? as usize)))
                        .collect::<Option<_>>()?;
                }
                OUTSIDE => objects.format = be32(body, 0),
                OBJECT_STARTS => starts = packed_numbers(body, count)?,
                _ => {}
            }
            at += length;
        }
        let mut pointer = 0u32;
        for fours in starts {
            pointer = pointer.checked_add(u32::try_from(fours).ok()?.checked_mul(4)?)?;
            let start = objects.at(pointer)?;
            objects.starts.push(start);
        }
        Some(objects)
    }

    /// Where a pointer leads in the file.
    fn at(&self, pointer: u32) -> Option<usize> {
        let &(start, size) = self.sections.get((pointer >> POINTER_SECTION) as usize)?;
        let offset = (pointer & POINTER_OFFSET) as usize;
        (offset < size).then_some(start + offset)
    }

    /// The objects of one kind, in the order the file lists them.
    pub fn of_kind<'b>(&'b self, kind: &'b str) -> impl Iterator<Item = usize> + 'b {
        self.starts.iter().copied().filter(move |&object| self.kind(object) == Some(kind))
    }

    /// The objects whose kind's name ends so, such as every `...ToyData`.
    pub fn of_kinds_ending<'b>(&'b self, ending: &'b str) -> impl Iterator<Item = usize> + 'b {
        self.starts.iter().copied().filter(move |&object| self.kind(object).is_some_and(|kind| kind.ends_with(ending)))
    }

    /// The name of an object's kind, such as `igImage2`.
    pub fn kind(&self, object: usize) -> Option<&str> {
        self.kinds.get(be32(self.bytes, object)? as usize).map(String::as_str)
    }

    pub fn word(&self, object: usize, field: usize) -> Option<u32> {
        be32(self.bytes, object + field)
    }

    pub fn half(&self, object: usize, field: usize) -> Option<u16> {
        be16(self.bytes, object + field).map(|half| half as u16)
    }

    /// The object a field points to; `None` for a field of 0, which points
    /// nowhere.
    pub fn object(&self, object: usize, field: usize) -> Option<usize> {
        match self.word(object, field)? {
            0 => None,
            pointer => self.at(pointer),
        }
    }

    /// The text a field points to, such as a toy's code name.
    pub fn text(&self, object: usize, field: usize) -> Option<&str> {
        text(self.bytes, self.object(object, field)?)
    }

    /// The objects a list holds: their count at 0x08 and a pointer to them
    /// at 0x14, as `igObjectList` keeps them. An empty list points nowhere.
    pub fn list(&self, list: usize) -> Option<Vec<usize>> {
        let count = self.word(list, 0x08)? as usize;
        if count == 0 {
            return Some(Vec::new());
        }
        let items = self.object(list, 0x14)?;
        (0..count).map(|index| self.at(be32(self.bytes, items + index * 4)?)).collect()
    }

    /// The outside name a field's handle gives, such as a material's.
    pub fn outside_name(&self, object: usize, field: usize) -> Option<&str> {
        let handle = self.word(object, field)?;
        if handle & HANDLE == 0 {
            return None;
        }
        let &(string, _) = self.outside.get((handle & !HANDLE) as usize)?;
        self.strings.get(string).map(String::as_str)
    }

    /// The outside names in one namespace, such as a material's pictures,
    /// in `image`.
    pub fn outside_in<'b>(&'b self, namespace: &'b str) -> impl Iterator<Item = &'b str> + 'b {
        self.outside.iter().filter_map(move |&(name, space)| {
            (self.strings.get(space)? == namespace).then(|| self.strings.get(name).map(String::as_str))?
        })
    }
}

/// `count` strings, each ended by a zero and padded to an even length.
fn padded_strings(body: &[u8], count: usize) -> Option<Vec<String>> {
    let mut at = 0;
    (0..count)
        .map(|_| {
            let text = text(body, at)?.to_string();
            at = (at + text.len() + 1).next_multiple_of(2);
            Some(text)
        })
        .collect()
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
/// is the one before plus four, plus four times a packed number. Counting
/// from 0, the first object starts at 4 or later, as a pointer of 0 means
/// none.
fn object_starts(packed: &[u8], count: usize) -> Option<Vec<usize>> {
    let mut at = 0;
    Some(packed_numbers(packed, count)?.into_iter().map(|fours| {
        at += fours * 4 + 4;
        at
    }).collect())
}

/// `count` numbers written three bits to a nibble, low nibble first, with a
/// nibble's top bit saying another follows.
fn packed_numbers(packed: &[u8], count: usize) -> Option<Vec<usize>> {
    let mut nibbles = packed.iter().flat_map(|&byte| [byte & 15, byte >> 4]);
    (0..count)
        .map(|_| {
            let (mut number, mut shift) = (0usize, 0);
            loop {
                let nibble = nibbles.next()?;
                number |= usize::from(nibble & 7).checked_shl(shift)?;
                shift += 3;
                if nibble & 8 == 0 {
                    return Some(number);
                }
            }
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
        if name == PICTURE.as_bytes() {
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

    /// A file of version 9 laid out as SuperChargers' are: the fixups at
    /// 0x800, `objects` one after another in the section after them, so a
    /// pointer to one is its offset there, `text` in the next, and `pixels`,
    /// when given, in a last section of their own. `outside` gives each
    /// outside name's string and namespace by number.
    pub(crate) fn build_objects(
        kinds: &[&str],
        strings: &[&str],
        outside: &[(u32, u32)],
        format: Option<u32>,
        objects: &[Vec<u8>],
        text: &[u8],
        pixels: Option<&[u8]>,
    ) -> Vec<u8> {
        let padded = |texts: &[&str]| -> Vec<u8> {
            let mut body = Vec::new();
            for text in texts {
                body.extend_from_slice(text.as_bytes());
                body.push(0);
                body.resize(body.len().next_multiple_of(2), 0);
            }
            body
        };
        let mut starts = Vec::new();
        let mut at = 0;
        for object in objects {
            starts.push(at / 4);
            at = (at + object.len()).next_multiple_of(4);
        }
        let steps: Vec<usize> = starts.iter().scan(0, |previous, &start| Some(start - std::mem::replace(previous, start))).collect();
        let mut nibbles = Vec::new();
        for mut number in steps {
            loop {
                let more = number > 7;
                nibbles.push((number & 7) as u8 | if more { 8 } else { 0 });
                number >>= 3;
                if !more {
                    break;
                }
            }
        }
        let packed: Vec<u8> = nibbles.chunks(2).map(|pair| pair[0] | pair.get(1).copied().unwrap_or(0) << 4).collect();
        let mut tables = vec![
            (TYPES, kinds.len(), padded(kinds)),
            (STRINGS, strings.len(), padded(strings)),
            (OUTSIDE_NAMES, outside.len(), outside.iter().flat_map(|&(name, space)| [(name | HANDLE).to_be_bytes(), space.to_be_bytes()]).flatten().collect()),
            (OBJECT_STARTS, objects.len(), packed),
        ];
        if let Some(format) = format {
            tables.push((OUTSIDE, 1, [format.to_be_bytes(), [0; 4]].concat()));
        }
        let mut out = vec![0; 0x800];
        out[0..4].copy_from_slice(&MAGIC.to_be_bytes());
        out[4..8].copy_from_slice(&SUPERCHARGERS.to_be_bytes());
        out[FIXUP_COUNT..FIXUP_COUNT + 4].copy_from_slice(&(tables.len() as u32).to_be_bytes());
        for (tag, count, mut body) in tables {
            body.resize(body.len().next_multiple_of(4), 0);
            out.extend_from_slice(&tag.to_be_bytes());
            out.extend_from_slice(&(count as u32).to_be_bytes());
            out.extend_from_slice(&(16 + body.len() as u32).to_be_bytes());
            out.extend_from_slice(&16u32.to_be_bytes());
            out.extend_from_slice(&body);
        }
        let mut sections = vec![(0x800, out.len() - 0x800)];
        let default = out.len();
        for object in objects {
            out.extend_from_slice(object);
            out.resize(out.len().next_multiple_of(4), 0);
        }
        sections.push((default, (out.len() - default).max(4)));
        out.resize(default + sections[1].1, 0);
        if !text.is_empty() {
            sections.push((out.len(), text.len()));
            out.extend_from_slice(text);
        }
        if let Some(pixels) = pixels {
            sections.push((out.len(), pixels.len()));
            out.extend_from_slice(pixels);
        }
        for (index, (offset, size)) in sections.into_iter().enumerate() {
            let at = SECTIONS + index * 16;
            out[at..at + 4].copy_from_slice(&(offset as u32).to_be_bytes());
            out[at + 4..at + 8].copy_from_slice(&(size as u32).to_be_bytes());
        }
        out
    }

    /// An object of the kind numbered `kind`, 0x40 long, with `fields`
    /// written at their offsets.
    pub(crate) fn object(kind: u32, fields: &[(usize, u32)]) -> Vec<u8> {
        let mut object = vec![0; 0x40];
        object[..4].copy_from_slice(&kind.to_be_bytes());
        for &(at, value) in fields {
            object[at..at + 4].copy_from_slice(&value.to_be_bytes());
        }
        object
    }

    /// A picture of SuperChargers, `side` pixels square, with `pixels`.
    pub(crate) fn build_picture(side: u16, pixels: &[u8]) -> Vec<u8> {
        let size = u32::from(side) << 16 | u32::from(side);
        let image = object(1, &[(IMAGE_SIZE, size)]);
        build_objects(&["igObjectList", PICTURE], &[], &[], Some(0x98cb_2a65), &[object(0, &[]), image], &[], Some(pixels))
    }

    #[test]
    fn a_superchargers_picture_gives_its_format_size_and_pixels() {
        let pixels: Vec<u8> = (0..=255).cycle().take(1024).collect();
        let picture = read(&build_picture(88, &pixels)).unwrap();
        assert_eq!((picture.format, picture.width, picture.height), (0x98cb_2a65, 88, 88));
        assert_eq!(picture.pixels, pixels);
        assert_eq!(picture.source, "");
    }

    #[test]
    fn a_superchargers_file_gives_its_objects() {
        // A list at 0 of two objects, at 0x40 and at 0x80, the second
        // naming an outside thing; the third object stands far on, its
        // start written in several nibbles.
        let list = object(0, &[(0x08, 2), (0x14, 0x18), (0x18, 0x40), (0x1c, 0x80)]);
        let first = object(1, &[(0x08, 3413)]);
        let second = object(2, &[(0x08, 7), (0x20, HANDLE | 1)]);
        let mut far = vec![0; 0x400];
        far[..0x40].copy_from_slice(&object(1, &[(0x08, 3220)]));
        let file = build_objects(
            &["igObjectList", "CFullCharacterToyData", "CVariantIdentifier"],
            &["Collection_DriverJetVac_Normal", "Collection_DriverJetVac_Legendary", "image"],
            &[(0, 0), (1, 2)],
            None,
            &[list, first, second, vec![0; 0x380], far],
            &[],
            None,
        );
        let objects = Objects::read(&file).unwrap();
        let toys: Vec<usize> = objects.of_kind("CFullCharacterToyData").collect();
        assert_eq!(toys.iter().map(|&toy| objects.word(toy, 0x08)).collect::<Vec<_>>(), [Some(3413), Some(3220)]);
        assert_eq!(objects.of_kinds_ending("ToyData").count(), 2);
        let list = objects.of_kind("igObjectList").next().unwrap();
        let items = objects.list(list).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(objects.kind(items[1]), Some("CVariantIdentifier"));
        assert_eq!(objects.outside_name(items[1], 0x20), Some("Collection_DriverJetVac_Legendary"));
        assert_eq!(objects.outside_name(items[1], 0x08), None);
        assert_eq!(objects.outside_in("image").collect::<Vec<_>>(), ["Collection_DriverJetVac_Legendary"]);
        assert_eq!(objects.object(items[0], 0x1c), None);
    }

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
        assert_eq!(version(b"IGZ\x01\x00\x00\x00\x0a"), Some(10));
        assert!(known(GIANTS) && known(TRAP_TEAM) && known(SUPERCHARGERS) && !known(10));
        assert!(read(b"IGZ\x01\x00\x00\x00\x0a and a version not known").is_none());
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
