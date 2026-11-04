//! Disney Infinity 1.0 on the PS3: each character as its Hall of Heroes
//! shows them, and each power disc as the game shows it when one is put on
//! the base, written as `<number>.png` by the number on the toy, as Cemu's
//! figure list numbers them (1000001 is Mr. Incredible).
//!
//! The game keeps its files in ordinary .zip archives under
//! `PS3_GAME/USRDIR/ASSETS`, every file in them stored or deflated, read
//! here as PKWARE's APPNOTE.TXT lays the format out (sections 4.3.7, 4.3.12
//! and 4.3.16). A picture is two files of one archive: `<name>.mtb`, which
//! says what the picture is, and its pixels, `textures/<16 hex digits>.tbody`,
//! named in the .mtb. The .mtb is big-endian: the magic `LDNB`, its version
//! (1), its length and how many parts follow (1); the part `BXET`, its
//! version (3), its length and 1; the picture's name led by its length with
//! its NUL, padded to four bytes; the eight bytes that name the pixels'
//! file; 0x28, the length of what follows; the pixels' length, 1 and the
//! length again; and the 24 bytes of the PS3's own description of a texture
//! (see `Texture`). Worked out from the game's files, 4 November 2026: every
//! picture read here has its pixels in the same archive.
//!
//! Which pictures the menus show comes from the game's screens, whose scripts
//! are compiled Lua with their strings readable: the pop-up for a hero, in
//! `PRESENTATION.ZIP`, `hero_unlock_alert.lua`, loads its picture from
//! `ui/hallofheroes/`, and the pop-up for a power disc,
//! `igp_coinpopup.lua`, loads one named `IGP_COIN_<name>` (its own example
//! is `IGP_COIN_CinderellaCoach`). Seen in Bertram's copy of the release for
//! Scandinavia, BLES01844, 4 November 2026.
//!
//! The game keeps no table of the toys' numbers outside its program, which
//! is not read: no file of its archives holds 1000001 or 3000003 in any
//! form. So the tables below were made by name, the characters' by looking
//! at each picture, the power discs' from the game's own English name for
//! each, the string `igp_coin_<name>_name` of `ASSETS/LANG/LANG.ZIP`,
//! matched to the names in Cemu's figure list (`s_listFigures`,
//! src/Cafe/OS/libs/nsyshid/Infinity.cpp, read 4 November 2026) and checked
//! by eye.

use flate2::read::DeflateDecoder;
use std::io::Read;
use std::path::Path;

/// The four releases of Disney Infinity 1.0 on the PS3, as Omoio knows them
/// (src-tauri/src/core/infinity.rs there, read 4 November 2026: each found
/// on Sony's update server with the title Disney Infinity). RPCS3's
/// compatibility list has BLUS30977 and BLES01842 under the same name; the
/// fourth is Bertram's own, the release for Scandinavia.
const TITLE_IDS: [&str; 4] = ["BLUS30977", "BLES01842", "BLES01843", "BLES01844"];

/// The Hall of Heroes' pictures, each character whole, 256 pixels square,
/// and the icons of the game's menus, the power discs among them.
const HALL_OF_HEROES: &str = "PS3_GAME/USRDIR/ASSETS/UI/HALLOFHEROES/HALLOFHEROES.ZIP";
const ICONS: &str = "PS3_GAME/USRDIR/ASSETS/UI/ICONS/ICONS.ZIP";

/// Each character by the name of their picture in the Hall of Heroes,
/// `av_<name>`, which is the name of their model: Bob is Mr. Incredible and
/// Helen Mrs. Incredible. The game has no picture of a crystal figure, which
/// it calls Infinite, or of the glowing Buzz Lightyear, but the
/// heads its HUD shows for them, so they have their character's.
const CHARACTERS: [(&str, u32); 38] = [
    ("bob", 1000001), // Mr. Incredible
    ("sullivan", 1000002), // Sulley
    ("jacksparrow", 1000003), // Jack Sparrow
    ("loneranger", 1000004), // Lone Ranger
    ("tonto", 1000005), // Tonto
    ("mcqueen", 1000006), // Lightning McQueen
    ("holly", 1000007), // Holley Shiftwell
    ("buzz", 1000008), // Buzz Lightyear
    ("jessie", 1000009), // Jessie
    ("mike", 1000010), // Mike
    ("helen", 1000011), // Mrs. Incredible
    ("barbossa", 1000012), // Hector Barbossa
    ("davyjones", 1000013), // Davy Jones
    ("randall", 1000014), // Randy
    ("syndrome", 1000015), // Syndrome
    ("woody", 1000016), // Woody
    ("mater", 1000017), // Mater
    ("dash", 1000018), // Dash
    ("violet", 1000019), // Violet
    ("francesco", 1000020), // Francesco Bernoulli
    ("mickeymouse", 1000021), // Sorcerer's Apprentice Mickey
    ("jackskellington", 1000022), // Jack Skellington
    ("rapunzel", 1000023), // Rapunzel
    ("anna", 1000024), // Anna
    ("elsa", 1000025), // Elsa
    ("phineas", 1000026), // Phineas
    ("perry", 1000027), // Agent P
    ("ralph", 1000028), // Wreck-It Ralph
    ("vanellope", 1000029), // Vanellope
    ("bob", 1000030), // Mr. Incredible (Crystal)
    ("jacksparrow", 1000031), // Jack Sparrow (Crystal)
    ("sullivan", 1000032), // Sulley (Crystal)
    ("mcqueen", 1000033), // Lightning McQueen (Crystal)
    ("loneranger", 1000034), // Lone Ranger (Crystal)
    ("buzz", 1000035), // Buzz Lightyear (Crystal)
    ("perry", 1000036), // Agent P (Crystal)
    ("mickeymouse", 1000037), // Sorcerer's Apprentice Mickey (Crystal)
    ("buzz", 1000038), // Buzz Lightyear (Glowing)
];

/// Each power disc by the name of its picture, `igp_coin_<name>`, the disc
/// whole with its frame, 128 pixels square: round for a power (3000003 on),
/// a hexagon for a toy, a texture or a sky for the Toy Box (4000018 on).
/// The game's English name is given where Cemu's differs. Its icons also
/// have `igp_coin_supercombo`, a disc with only the Infinity mark on it and
/// no name in the game's words, which is none of these.
const DISCS: [(&str, u32); 63] = [
    ("boltstrength", 3000003), // Bolt's Super Strength
    ("wreckitdamage", 3000004), // Ralph's Power of Destruction
    ("churnabogstrength", 3000005), // Chernabog's Power
    ("chromedamageincreaser", 3000006), // C.H.R.O.M.E. Damage Increaser
    ("doofinshmirtzdamage", 3000007), // Dr. Doofenshmirtz's Damage-Inator!
    ("electrocharge", 3000008), // Electro-Charge
    ("felixfixyou", 3000009), // Fix-It Felix's Repair Power
    ("rapunzelhealing", 3000010), // Rapunzel's Healing
    ("chrome_damagecontrol", 3000011), // C.H.R.O.M.E. Armor Shield; the game's "C.H.R.O.M.E.'S Armor Shield"
    ("starcommandshield", 3000012), // Star Command Shield
    ("violetforcefield", 3000013), // Violet's Force Field
    ("piratebooty", 3000014), // Pieces of Eight
    ("scroogedime", 3000015), // Scrooge McDuck's Lucky Dime
    ("usercontrol", 3000016), // User Control
    ("mickeysorcererhat", 3000017), // Sorcerer Mickey's Hat; the game's "Mickey's Sorcerer Hat"
    ("zurgwrath", 3000062), // Emperor Zurg's Wrath
    ("merlinsummon", 3000063), // Merlin's Summon
    ("mickeyjalopy", 4000018), // Mickey's Car
    ("cinderellacoach", 4000019), // Cinderella's Coach
    ("muppetsbus", 4000020), // Electric Mayhem Bus
    ("cruellacar", 4000021), // Cruella De Vil's Car
    ("pizzaplanettruck", 4000022), // Pizza Planet Delivery Truck
    ("mikecar", 4000023), // Mike's New Car
    ("parkinglottram", 4000025), // Parking Lot Tram
    ("hookship", 4000026), // Captain Hook's Ship
    ("dumboride", 4000027), // Dumbo; the game's "Dumbo the Flying Elephant"
    ("calicohelicopter", 4000028), // Calico Helicopter
    ("tangledmaximus", 4000029), // Maximus
    ("braveangus", 4000030), // Angus
    ("abuelephant", 4000031), // Abu the Elephant
    ("sleepyhollowhorse", 4000032), // Headless Horseman's Horse
    ("beautybeastphillippe", 4000033), // Phillipe
    ("khanhorse", 4000034), // Khan; the game's "Kahn"
    ("tarzantantor", 4000035), // Tantor
    ("mulancannon", 4000036), // Dragon Firework Cannon
    ("stitchblaster", 4000037), // Stitch's Blaster
    ("midwaygun", 4000038), // Toy Story Mania Blaster
    ("flamingomallet", 4000039), // Flamingo Croquet Mallet
    ("tennisballcane", 4000040), // Carl Fredricksen's Cane
    ("stitchsurfboard", 4000041), // Hangin' Ten Stitch With Surfboard
    ("condorglider", 4000042), // Condorman Glider; the game's "Condor Wing Glider"
    ("wallefireextinguisher", 4000043), // WALL-E's Fire Extinguisher
    ("trontextures", 4000044), // On the Grid
    ("walletextures", 4000045), // WALL-E's Collection
    ("sugarrushtextures", 4000046), // King Candy's Dessert Toppings
    ("frankenweenietextures", 4000048), // Victor's Experiments
    ("nightmaretextures", 4000049), // Jack's Scary Decorations
    ("frozentextures", 4000051), // Frozen Flourish
    ("tangledtextures", 4000052), // Rapunzel's Kingdom
    ("tronskydome", 4000053), // TRON Interface
    ("walleskydome", 4000054), // Buy N Large Atmosphere
    ("sugarrushskydome", 4000055), // Sugar Rush Sky
    ("frankenweenieskydome", 4000057), // New Holland Skyline
    ("nightmareskydome", 4000058), // Halloween Town Sky
    ("frozenskydome", 4000060), // Chill in the Air
    ("tangledskydome", 4000061), // Rapunzel's Birthday Sky
    ("buzzride", 4000064), // Astro Blasters Space Cruiser; the game's "Buzz Lightyear's Astro Blaster Ride"
    ("nemotextures", 4000065), // Marlin's Reef
    ("nemoskydome", 4000066), // Nemo's Seascape
    ("alicetextures", 4000067), // Alice's Wonderland
    ("aliceskydome", 4000068), // Tulgey Wood
    ("phineastextures", 4000069), // Tri-State Area Terrain
    ("phineasskydome", 4000070), // Danville Sky
];

/// The PS3's texture formats, as RPCS3 numbers them
/// (rpcs3/Emu/RSX/gcm_enums.h, read 4 November 2026): A8R8G8B8, four bytes
/// a pixel, alpha first, and DXT45, which is DXT5. Two bits beside the
/// format say a texture's pixels are in rows rather than swizzled, and how
/// its coordinates are read, which doesn't change its pixels.
const ARGB: u8 = 0x85;
const DXT5: u8 = 0x88;
const LINEAR: u8 = 0x20;
const UNNORMALISED: u8 = 0x40;
/// A texture's remap says where each channel comes from; this one, which
/// every picture read here has, takes each from itself. PSL1GHT's
/// ppu/include/rsx/gcm_sys.h (read 4 November 2026) gives its fields: a
/// type of 2 for each channel from bit 8 up, and its source, A, R, G and B
/// as 0 to 3, from bit 0.
const OWN_CHANNELS: u32 = 0xaae4;

const MTB: &str = ".mtb";
const MTB_MAGIC: &[u8] = b"LDNB";
const TEXTURE_PART: &[u8] = b"BXET";
const NAME_LENGTH_AT: usize = 0x20;
const NAME_AT: usize = 0x28;
/// What follows the name of the pixels' file: the sizes and the texture.
const DESCRIPTION: u32 = 0x28;

pub fn knows(title_id: &str) -> bool {
    TITLE_IDS.contains(&title_id)
}

/// Every picture into `out`, telling `progress` how many of how many are
/// done as it goes. Gives how many were written.
pub fn pictures(game: &Path, out: &Path, progress: &mut dyn FnMut(usize, usize)) -> Result<usize, String> {
    let missing = || "This copy of Disney Infinity is missing the files its pictures are in.".to_string();
    let heroes = super::game_file(game, HALL_OF_HEROES)?.ok_or_else(missing)?;
    let icons = super::game_file(game, ICONS)?.ok_or_else(missing)?;
    let unreadable = || "Couldn't read the archives Disney Infinity keeps its pictures in.".to_string();
    let heroes = Zip::open(&heroes).ok_or_else(unreadable)?;
    let icons = Zip::open(&icons).ok_or_else(unreadable)?;
    std::fs::create_dir_all(out).map_err(|_| "Couldn't make the folder for the pictures.".to_string())?;
    let wanted: Vec<(&Zip, String, u32)> = CHARACTERS
        .iter()
        .map(|&(name, number)| (&heroes, format!("av_{name}"), number))
        .chain(DISCS.iter().map(|&(name, number)| (&icons, format!("igp_coin_{name}"), number)))
        .collect();
    let steps = wanted.len();
    let mut written = 0;
    for (done, (archive, name, number)) in wanted.iter().enumerate() {
        progress(done, steps);
        let Some((rgba, width, height)) = picture(archive, name) else {
            continue;
        };
        let (square, side) = super::around_shape(&rgba, width, height);
        super::write(&out.join(format!("{number}.png")), side, side, &square)?;
        written += 1;
    }
    progress(steps, steps);
    Ok(written)
}

/// The picture `<name>.mtb` of an archive as RGBA, top row first, with its
/// width and height. `None` when the archive hasn't it, or has it in a form
/// not read here.
fn picture(archive: &Zip, name: &str) -> Option<(Vec<u8>, usize, usize)> {
    let texture = Texture::read(&archive.read(&format!("{name}{MTB}"))?)?;
    let pixels = archive.read(&texture.pixels)?;
    Some((texture.rgba(&pixels)?, texture.width, texture.height))
}

/// What a .mtb says of its picture: where its pixels are, and the parts of
/// the PS3's own description of a texture that matter here. That
/// description is PSL1GHT's `gcmTexture` (ppu/include/rsx/gcm_sys.h, read
/// 4 November 2026): format, mipmaps, dimension and cube map a byte each,
/// the remap, width, height and depth, location and a byte of padding, the
/// pitch and the offset.
#[derive(Debug, PartialEq)]
struct Texture {
    pixels: String,
    format: u8,
    remap: u32,
    width: usize,
    height: usize,
    pitch: usize,
}

impl Texture {
    fn read(mtb: &[u8]) -> Option<Texture> {
        if mtb.get(..4)? != MTB_MAGIC || mtb.get(0x10..0x14)? != TEXTURE_PART {
            return None;
        }
        let name_ends = NAME_AT + (be32(mtb, NAME_LENGTH_AT)? as usize).next_multiple_of(4);
        let pixels: String = mtb.get(name_ends..name_ends + 8)?.iter().map(|byte| format!("{byte:02x}")).collect();
        let sizes = name_ends + 8;
        if be32(mtb, sizes)? != DESCRIPTION {
            return None;
        }
        let texture = sizes + 16;
        Some(Texture {
            pixels: format!("textures/{pixels}.tbody"),
            format: *mtb.get(texture)?,
            remap: be32(mtb, texture + 4)?,
            width: usize::from(be16(mtb, texture + 8)?),
            height: usize::from(be16(mtb, texture + 10)?),
            pitch: be32(mtb, texture + 16)? as usize,
        })
    }

    /// The pixels as RGBA, top row first, as the PS3 keeps them. Only the
    /// first mipmap is read; it comes first.
    fn rgba(&self, pixels: &[u8]) -> Option<Vec<u8>> {
        let (width, height) = (self.width, self.height);
        if self.remap != OWN_CHANNELS || width == 0 || height == 0 {
            return None;
        }
        match self.format & !(LINEAR | UNNORMALISED) {
            // DXT5's blocks are in rows, as on a PC, swizzled or not.
            DXT5 if width % 4 == 0 && height % 4 == 0 => Some(super::dxt5::decode(pixels.get(..width * height)?, width, height)),
            ARGB => {
                let in_rows = self.format & LINEAR != 0;
                let row = if in_rows && self.pitch != 0 { self.pitch } else { width * 4 };
                if row < width * 4 || (!in_rows && !(width.is_power_of_two() && height.is_power_of_two())) {
                    return None;
                }
                let pixels = pixels.get(..row * height)?;
                let mut rgba = Vec::with_capacity(width * height * 4);
                for y in 0..height {
                    for x in 0..width {
                        let at = if in_rows { y * row + x * 4 } else { swizzled(x, y, width, height) * 4 };
                        let pixel = &pixels[at..at + 4];
                        rgba.extend_from_slice(&[pixel[1], pixel[2], pixel[3], pixel[0]]);
                    }
                }
                Some(rgba)
            }
            _ => None,
        }
    }
}

/// Where pixel (x, y) of a swizzled texture is kept: the bits of x and y
/// taken in turn from the lowest, x first, for as many bits as the shorter
/// side has, then the rest of the longer side's above them. As RPCS3's
/// `convert_linear_swizzle` (rpcs3/Emu/RSX/Utils/image_utils.hpp, read
/// 4 November 2026) reads them; both sides are powers of two.
fn swizzled(x: usize, y: usize, width: usize, height: usize) -> usize {
    let shared = width.min(height).trailing_zeros();
    let mut index = 0;
    for bit in 0..shared {
        index |= (x >> bit & 1) << (2 * bit) | (y >> bit & 1) << (2 * bit + 1);
    }
    let rest = if width > height { x >> shared } else { y >> shared };
    index | rest << (2 * shared)
}

fn be16(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

fn be32(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn le16(bytes: &[u8], at: usize) -> Option<usize> {
    Some(usize::from(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?)))
}

fn le32(bytes: &[u8], at: usize) -> Option<usize> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize)
}

/// One of the game's .zip archives, its files found through the central
/// directory at its end. Everything in a .zip is little-endian.
struct Zip<'a> {
    bytes: &'a [u8],
    files: Vec<ZipFile>,
}

struct ZipFile {
    name: String,
    method: usize,
    packed: usize,
    size: usize,
    header: usize,
}

/// The records of APPNOTE.TXT: the end of the central directory (4.3.16),
/// an entry of it (4.3.12) and the header before each file's bytes (4.3.7).
const END: &[u8] = b"PK\x05\x06";
const ENTRY: &[u8] = b"PK\x01\x02";
const HEADER: &[u8] = b"PK\x03\x04";
const END_LENGTH: usize = 22;
const ENTRY_LENGTH: usize = 46;
const HEADER_LENGTH: usize = 30;
/// The longest comment a .zip may end with, after its last record.
const LONGEST_COMMENT: usize = 0xffff;
const STORED: usize = 0;
const DEFLATED: usize = 8;

impl<'a> Zip<'a> {
    fn open(bytes: &'a [u8]) -> Option<Zip<'a>> {
        let last = bytes.len().checked_sub(END_LENGTH)?;
        let end = (last.saturating_sub(LONGEST_COMMENT)..=last).rev().find(|&at| &bytes[at..at + 4] == END)?;
        let count = le16(bytes, end + 10)?;
        let mut at = le32(bytes, end + 16)?;
        let mut files = Vec::with_capacity(count);
        for _ in 0..count {
            if bytes.get(at..at + 4)? != ENTRY {
                return None;
            }
            let name_length = le16(bytes, at + 28)?;
            let name = bytes.get(at + ENTRY_LENGTH..at + ENTRY_LENGTH + name_length)?;
            files.push(ZipFile {
                name: String::from_utf8_lossy(name).into_owned(),
                method: le16(bytes, at + 10)?,
                packed: le32(bytes, at + 20)?,
                size: le32(bytes, at + 24)?,
                header: le32(bytes, at + 42)?,
            });
            at += ENTRY_LENGTH + name_length + le16(bytes, at + 30)? + le16(bytes, at + 32)?;
        }
        Some(Zip { bytes, files })
    }

    /// A file of the archive by its name, unpacked. `None` when it hasn't
    /// one, or it doesn't unpack to its size.
    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let file = self.files.iter().find(|file| file.name == name)?;
        let header = file.header;
        if self.bytes.get(header..header + 4)? != HEADER {
            return None;
        }
        let start = header + HEADER_LENGTH + le16(self.bytes, header + 26)? + le16(self.bytes, header + 28)?;
        let packed = self.bytes.get(start..start + file.packed)?;
        let bytes = match file.method {
            STORED => packed.to_vec(),
            DEFLATED => {
                let mut out = Vec::with_capacity(file.size);
                DeflateDecoder::new(packed).read_to_end(&mut out).ok()?;
                out
            }
            _ => return None,
        };
        (bytes.len() == file.size).then_some(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::DeflateEncoder;
    use std::io::Write;

    /// A .zip holding `files`, each deflated or stored as it says, laid out
    /// as the game's are: the files, then the central directory.
    fn build_zip(files: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut directory = Vec::new();
        for &(name, bytes, deflate) in files {
            let packed = if deflate {
                let mut encoder = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(bytes).unwrap();
                encoder.finish().unwrap()
            } else {
                bytes.to_vec()
            };
            let method: u16 = if deflate { 8 } else { 0 };
            let (packed_length, size, name_length) = (packed.len() as u32, bytes.len() as u32, name.len() as u16);
            let header = out.len() as u32;
            out.extend_from_slice(b"PK\x03\x04\x14\x00\x00\x00");
            out.extend_from_slice(&method.to_le_bytes());
            out.extend_from_slice(&[0; 8]);
            out.extend_from_slice(&packed_length.to_le_bytes());
            out.extend_from_slice(&size.to_le_bytes());
            out.extend_from_slice(&name_length.to_le_bytes());
            // An extra field, as the game's have, which the reader steps over.
            out.extend_from_slice(&4u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(&[0xaa; 4]);
            out.extend_from_slice(&packed);
            directory.extend_from_slice(b"PK\x01\x02\x14\x00\x14\x00\x00\x00");
            directory.extend_from_slice(&method.to_le_bytes());
            directory.extend_from_slice(&[0; 8]);
            directory.extend_from_slice(&packed_length.to_le_bytes());
            directory.extend_from_slice(&size.to_le_bytes());
            directory.extend_from_slice(&name_length.to_le_bytes());
            directory.extend_from_slice(&[0; 12]);
            directory.extend_from_slice(&header.to_le_bytes());
            directory.extend_from_slice(name.as_bytes());
        }
        let (directory_at, directory_length, count) = (out.len() as u32, directory.len() as u32, files.len() as u16);
        out.extend_from_slice(&directory);
        out.extend_from_slice(b"PK\x05\x06\x00\x00\x00\x00");
        out.extend_from_slice(&count.to_le_bytes());
        out.extend_from_slice(&count.to_le_bytes());
        out.extend_from_slice(&directory_length.to_le_bytes());
        out.extend_from_slice(&directory_at.to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out
    }

    /// A .mtb laid out as the game's, for a picture of one mipmap whose
    /// pixels are in `textures/<pixels as hex>.tbody`.
    fn build_mtb(name: &str, pixels: [u8; 8], format: u8, width: u16, height: u16, length: u32) -> Vec<u8> {
        let named = (name.len() + 1).next_multiple_of(4);
        let mut out = Vec::new();
        out.extend_from_slice(b"LDNB\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x01");
        out.extend_from_slice(b"BXET\x00\x00\x00\x03\x00\x00\x00\x00\x00\x00\x00\x01");
        out.extend_from_slice(&(name.len() as u32 + 1).to_be_bytes());
        out.extend_from_slice(&0x28u32.to_be_bytes());
        out.extend_from_slice(name.as_bytes());
        out.resize(NAME_AT + named, 0);
        out.extend_from_slice(&pixels);
        for word in [0x28, length, 1, length] {
            out.extend_from_slice(&u32::to_be_bytes(word));
        }
        out.extend_from_slice(&[format, 1, 2, 0]);
        out.extend_from_slice(&OWN_CHANNELS.to_be_bytes());
        out.extend_from_slice(&width.to_be_bytes());
        out.extend_from_slice(&height.to_be_bytes());
        out.extend_from_slice(&[0, 1, 0, 0]);
        out.extend_from_slice(&[0; 8]);
        out.extend_from_slice(&[1, 1, 1, 0]);
        out.extend_from_slice(&[0; 12]);
        let length = out.len() as u32;
        out[8..12].copy_from_slice(&length.to_be_bytes());
        out[0x18..0x1c].copy_from_slice(&(length - 0x10).to_be_bytes());
        out
    }

    #[test]
    fn a_zip_gives_its_files_stored_or_deflated() {
        let zip = build_zip(&[("a.mtb", b"stored", false), ("textures/b.tbody", &[7; 300], true)]);
        let zip = Zip::open(&zip).unwrap();
        assert_eq!(zip.read("a.mtb").as_deref(), Some(&b"stored"[..]));
        assert_eq!(zip.read("textures/b.tbody"), Some(vec![7; 300]));
        assert_eq!(zip.read("c.mtb"), None);
        assert!(Zip::open(b"PK\x03\x04 not a whole zip").is_none());
    }

    #[test]
    fn a_mtb_names_its_pixels_and_their_texture() {
        // Mr. Incredible's, as the game has it: its name padded from 7 to 8.
        let mtb = build_mtb("av_bob", [0x56, 0x8f, 0x0d, 0xd0, 0x8a, 0x2f, 0xa3, 0x28], DXT5, 256, 256, 0x10000);
        assert_eq!(mtb.len(), 112);
        let texture = Texture::read(&mtb).unwrap();
        assert_eq!(
            texture,
            Texture { pixels: "textures/568f0dd08a2fa328.tbody".to_string(), format: DXT5, remap: OWN_CHANNELS, width: 256, height: 256, pitch: 0 }
        );
        // A name of 19 bytes with its NUL is padded to 20.
        let texture = Texture::read(&build_mtb("av_jackskellington", [1; 8], ARGB, 128, 64, 0x8000)).unwrap();
        assert_eq!((texture.pixels.as_str(), texture.width, texture.height), ("textures/0101010101010101.tbody", 128, 64));
        assert_eq!(Texture::read(b"LDNB but nothing more"), None);
    }

    #[test]
    fn swizzled_pixels_take_x_and_y_bits_in_turn() {
        // Square: x's bits at the even places, y's at the odd ones.
        assert_eq!(swizzled(1, 0, 4, 4), 1);
        assert_eq!(swizzled(0, 1, 4, 4), 2);
        assert_eq!(swizzled(2, 0, 4, 4), 4);
        assert_eq!(swizzled(3, 3, 4, 4), 15);
        // Wider than high: past the shorter side, the rest of x goes above.
        assert_eq!(swizzled(2, 0, 4, 2), 4);
        assert_eq!(swizzled(3, 1, 4, 2), 7);
        // Higher than wide: the rest of y does.
        assert_eq!(swizzled(1, 2, 2, 4), 5);
        assert_eq!(swizzled(1, 3, 2, 4), 7);
    }

    #[test]
    fn argb_pixels_come_out_as_rgba_in_rows() {
        let argb = |a: u8, r: u8| [a, r, 0, 0];
        // Two by two, swizzled: (0,0), (1,0), (0,1), (1,1).
        let pixels = [argb(255, 1), argb(255, 2), argb(128, 3), argb(0, 4)].concat();
        let texture = Texture { pixels: String::new(), format: ARGB, remap: OWN_CHANNELS, width: 2, height: 2, pitch: 0 };
        assert_eq!(texture.rgba(&pixels).unwrap(), [[1, 0, 0, 255], [2, 0, 0, 255], [3, 0, 0, 128], [4, 0, 0, 0]].concat());
        // Four by two in rows, which the swizzle would have put apart.
        let pixels: Vec<u8> = (1..=8).flat_map(|red| argb(255, red)).collect();
        let texture = Texture { pixels: String::new(), format: ARGB | LINEAR, remap: OWN_CHANNELS, width: 4, height: 2, pitch: 16 };
        let reds: Vec<u8> = texture.rgba(&pixels).unwrap().chunks(4).map(|pixel| pixel[0]).collect();
        assert_eq!(reds, [1, 2, 3, 4, 5, 6, 7, 8]);
        // Channels moved about, or too few pixels, aren't read.
        let moved = Texture { remap: 0xaa1b, ..texture };
        assert_eq!(moved.rgba(&pixels), None);
        let texture = Texture { pixels: String::new(), format: ARGB, remap: OWN_CHANNELS, width: 4, height: 4, pitch: 0 };
        assert_eq!(texture.rgba(&pixels), None);
    }

    #[test]
    fn the_four_releases_are_known() {
        for id in ["BLUS30977", "BLES01842", "BLES01843", "BLES01844"] {
            assert!(knows(id));
        }
        assert!(!knows("BLES02105")); // LEGO Dimensions
        assert!(!knows("BLUS31418")); // Disney Infinity 2.0
    }

    #[test]
    fn every_number_has_one_picture() {
        let mut numbers: Vec<u32> = CHARACTERS.iter().chain(DISCS.iter()).map(|&(_, number)| number).collect();
        numbers.sort();
        numbers.dedup();
        assert_eq!(numbers.len(), CHARACTERS.len() + DISCS.len());
    }

    #[test]
    fn pictures_are_named_by_the_number_on_the_toy() {
        // Mr. Incredible as one DXT5 block, solid red, and Bolt's disc as four
        // pixels of ARGB, swizzled: blue and clear above, green below.
        let red = [255, 255, 0, 0, 0, 0, 0, 0, 0x00, 0xf8, 0x00, 0xf8, 0, 0, 0, 0];
        let bolt = [[255, 0, 0, 255], [0, 0, 0, 0], [255, 0, 255, 0], [255, 0, 255, 0]].concat();
        let heroes = build_zip(&[
            ("av_bob.mtb", &build_mtb("av_bob", [0x56; 8], DXT5, 4, 4, 16), true),
            ("textures/5656565656565656.tbody", &red, true),
        ]);
        let icons = build_zip(&[
            ("igp_coin_boltstrength.mtb", &build_mtb("igp_coin_boltstrength", [0x0b; 8], ARGB, 2, 2, 16), true),
            ("textures/0b0b0b0b0b0b0b0b.tbody", &bolt, false),
        ]);
        let game = std::env::temp_dir().join(format!("omoio-portraits-{}-infinity", std::process::id()));
        for (inside, bytes) in [(HALL_OF_HEROES, &heroes), (ICONS, &icons)] {
            let path = game.join(inside);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, bytes).unwrap();
        }
        let folder = game.join("pictures");
        let mut steps = Vec::new();
        assert_eq!(pictures(&game, &folder, &mut |done, of| steps.push((done, of))), Ok(3));
        let mut written: Vec<String> = std::fs::read_dir(&folder).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned()).collect();
        written.sort();
        // The crystal Mr. Incredible has his picture too.
        assert_eq!(written, ["1000001.png", "1000030.png", "3000003.png"]);
        assert_eq!(steps.first(), Some(&(0, 101)));
        assert_eq!(steps.last(), Some(&(101, 101)));
        let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(folder.join("3000003.png")).unwrap()));
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        assert_eq!((info.width, info.height), (2, 2));
        assert_eq!(pixels, [[0, 0, 255, 255], [0, 0, 0, 0], [0, 255, 0, 255], [0, 255, 0, 255]].concat());
        std::fs::remove_dir_all(game).unwrap();
    }

    /// Reads the copy of the game named by OMOIO_INFINITY_PS3, the folder
    /// that holds PS3_GAME, so it never runs where there is none:
    /// `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn a_real_copy_gives_every_picture() {
        let game = std::env::var_os("OMOIO_INFINITY_PS3").expect("OMOIO_INFINITY_PS3 names no copy of the game");
        let folder = std::env::temp_dir().join(format!("omoio-portraits-{}-infinity-real", std::process::id()));
        assert_eq!(pictures(Path::new(&game), &folder, &mut |_, _| ()), Ok(CHARACTERS.len() + DISCS.len()));
        std::fs::remove_dir_all(folder).unwrap();
    }
}
