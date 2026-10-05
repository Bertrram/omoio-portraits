//! Reads the figure pictures out of the user's own copy of Skylanders SWAP
//! Force or Trap Team, for Omoio's portal menu. Omoio downloads this program
//! only when the user asks for the pictures, and runs it two ways:
//!
//!     omoio-portraits title <game.wua>
//!     omoio-portraits pictures <game.wua or unpacked game folder> <folder>
//!
//! `title` prints `title <title id>`, so Omoio can tell which game a .wua
//! is. `pictures` writes one PNG per figure into the folder, the right way
//! up, named `<id>-<variant>.png` with the variant as four hex digits, the
//! game's eight element symbols as white shapes, `element-<name>.png`, for
//! Omoio to colour, and the badges of the eight ways a swapper moves,
//! `movement-<name>.png`. From Trap Team it writes every figure and trap as
//! its Collection screen shows them, with the same names, and each villain
//! in a trap and out of one, `villain-<number>.png` and
//! `villain-<number>-loose.png`. It prints `progress <done> <of>` as it goes
//! and `done <written>` at the end.
//! Nothing is downloaded: every picture comes from the user's own files.

mod dxt5;
mod gx2;
mod igz;
mod names;
mod pak;
mod wua;

use std::path::Path;
use std::process::ExitCode;

/// The archive in SWAP Force that holds every figure's portrait.
const PORTRAITS: &str = "content/archives/characterillustrations.pak";
/// Trap Team's pictures, each in the archive of the screen that shows them:
/// the Collection screen's figures, its traps, and the Villain Vault.
const TRAP_TEAM: [&str; 3] = [
    "content/misc/UI_Collection_Champions.arc",
    "content/misc/ui_collection_traps.arc",
    "content/misc/ui_villainvault_stream.arc",
];
const NOT_KNOWN_GAME: &str =
    "This game has no figure pictures Omoio can read yet. So far that is Skylanders SWAP Force and Trap Team.";
/// The archive with the town's elemental stones, whose texture has the eight
/// element symbols as white shapes, four across and two down, in this order
/// (checked by eye).
const SYMBOLS: &str = "content/archives/town3_elementalstones.pak";
const SYMBOLS_PICTURE: &str = "_elementIcons_D";
const SYMBOL_ORDER: [&str; 8] = ["tech", "water", "air", "undead", "magic", "life", "fire", "earth"];
/// The archive of the pictures a level's list of Swap Zones shows: the
/// zone's badge, a hexagon standing on a point, on a piece of the level.
const ZONES: &str = "content/archives/collectibleicons.pak";
/// The eight ways a swapper moves, as Omoio names them, and how the game
/// ends the names of their Swap Zone pictures ("SZ_Tuto_Dig"). The game
/// calls sneaking stealth there.
const MOVEMENTS: [(&str, &str); 8] = [
    ("bounce", "_bounce"),
    ("climb", "_climb"),
    ("dig", "_dig"),
    ("rocket", "_rocket"),
    ("sneak", "_stealth"),
    ("speed", "_speed"),
    ("spin", "_spin"),
    ("teleport", "_teleport"),
];
/// Where the badge sits in every Swap Zone picture, all of them 200 pixels
/// square: its middle, and how far its corners are from it. Measured where
/// pictures of one zone with different levels behind agree.
const ZONE_SIZE: usize = 200;
const BADGE_MIDDLE: (f32, f32) = (100.5, 80.5);
const BADGE_RADIUS: f32 = 61.0;
const META: &str = "meta/meta.xml";
/// DXT5 kept tiled for the Wii U's chip, as the game's pictures name it
/// (a hash of "dxt5_tile_cafe").
const DXT5_TILED: u32 = 0x98cb_2a65;

const USAGE: &str = "Usage: omoio-portraits title <game.wua>\n       omoio-portraits pictures <game.wua or game folder> <folder>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["title", game] => title(Path::new(game)).map(|id| println!("title {id}")),
        ["pictures", game, folder] => pictures(Path::new(game), Path::new(folder)).map(|n| println!("done {n}")),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// One file of the game, from a .wua or from an unpacked game folder.
fn game_file(game: &Path, inside: &str) -> Result<Option<Vec<u8>>, String> {
    if game.is_dir() {
        return Ok(std::fs::read(game.join(inside)).ok());
    }
    let mut archive = wua::Archive::open(game)?;
    match archive.files().into_iter().find(|file| file.path.ends_with(inside)) {
        Some(file) => archive.read(&file).map(Some),
        None => Ok(None),
    }
}

/// The game's title id, from its meta.xml.
fn title(game: &Path) -> Result<String, String> {
    let unknown = || "Couldn't tell which game this is.".to_string();
    let meta = game_file(game, META)?.ok_or_else(unknown)?;
    let meta = String::from_utf8_lossy(&meta);
    let (_, rest) = meta.split_once("<title_id").ok_or_else(unknown)?;
    let (_, rest) = rest.split_once('>').ok_or_else(unknown)?;
    let (id, _) = rest.split_once('<').ok_or_else(unknown)?;
    Ok(id.trim().to_ascii_lowercase())
}

/// An archive's textures; its other files are models, sounds and scripts.
fn textures<'a>(archive: &'a pak::Pak) -> Vec<&'a pak::PakFile> {
    archive.files.iter().filter(|file| file.name.starts_with("textures\\")).collect()
}

/// Tells the game by its files rather than its title id, so every region's
/// copy of it is read the same way.
fn pictures(game: &Path, folder: &Path) -> Result<usize, String> {
    match game_file(game, PORTRAITS)? {
        Some(bytes) => swap_force(game, &bytes, folder),
        None => trap_team(game, folder),
    }
}

fn swap_force(game: &Path, bytes: &[u8], folder: &Path) -> Result<usize, String> {
    let archive = pak::open(bytes)?;
    std::fs::create_dir_all(folder).map_err(|_| "Couldn't make the folder for the pictures.".to_string())?;
    let portraits = textures(&archive);
    // The element symbols and the movement badges come last, a step each.
    let steps = portraits.len() + 2;
    let mut written = 0;
    for (done, file) in portraits.iter().enumerate() {
        println!("progress {done} {steps}");
        let Some(picture) = igz::read(&archive.read(file)?) else {
            continue;
        };
        if picture.format != DXT5_TILED {
            continue;
        }
        if let Some((id, variant)) = names::figure(&picture.source) {
            let path = folder.join(format!("{id}-{variant:04x}.png"));
            write(&path, picture.width, picture.height, &upright(&picture)?)?;
            written += 1;
        }
    }
    println!("progress {} {steps}", portraits.len());
    written += symbols(game, folder)?;
    println!("progress {} {steps}", portraits.len() + 1);
    written += movements(game, folder)?;
    println!("progress {steps} {steps}");
    Ok(written)
}

fn trap_team(game: &Path, folder: &Path) -> Result<usize, String> {
    let mut archives = Vec::new();
    for inside in TRAP_TEAM {
        archives.push(game_file(game, inside)?.ok_or(NOT_KNOWN_GAME)?);
    }
    // Each archive borrows the bytes read above, so those stay alive in
    // `archives` while the pictures are read out of them.
    let archives = archives.iter().map(|bytes| pak::open(bytes)).collect::<Result<Vec<_>, _>>()?;
    let wanted: Vec<_> = archives
        .iter()
        .flat_map(|archive| {
            archive.files.iter().filter_map(move |file| names::trap_team(&file.name).map(|name| (archive, file, name)))
        })
        .collect();
    std::fs::create_dir_all(folder).map_err(|_| "Couldn't make the folder for the pictures.".to_string())?;
    let steps = wanted.len();
    let mut written = 0;
    for (done, (archive, file, name)) in wanted.iter().enumerate() {
        println!("progress {done} {steps}");
        let Some(picture) = igz::read(&archive.read(file)?) else {
            continue;
        };
        if picture.format != DXT5_TILED {
            continue;
        }
        write(&folder.join(format!("{name}.png")), picture.width, picture.height, &upright(&picture)?)?;
        written += 1;
    }
    println!("progress {steps} {steps}");
    Ok(written)
}

/// The eight element symbols, written as `element-<name>.png`. A game
/// without the town's stones just has none.
fn symbols(game: &Path, folder: &Path) -> Result<usize, String> {
    let Some(bytes) = game_file(game, SYMBOLS)? else {
        return Ok(0);
    };
    let archive = pak::open(&bytes)?;
    for file in textures(&archive) {
        let Some(picture) = igz::read(&archive.read(file)?) else {
            continue;
        };
        if picture.source != SYMBOLS_PICTURE || picture.format != DXT5_TILED {
            continue;
        }
        let rows = upright(&picture)?;
        let (wide, high) = (picture.width / 4, picture.height / 2);
        for (index, name) in SYMBOL_ORDER.iter().enumerate() {
            let (left, top) = (index % 4 * wide, index / 4 * high);
            let cell: Vec<u8> = (top..top + high)
                .flat_map(|y| {
                    let at = (y * picture.width + left) * 4;
                    rows[at..at + wide * 4].iter().copied()
                })
                .collect();
            write(&folder.join(format!("element-{name}.png")), wide, high, &cell)?;
        }
        return Ok(SYMBOL_ORDER.len());
    }
    Ok(0)
}

/// The badge of each way a swapper moves, cut out of one of its Swap Zone
/// pictures and written as `movement-<name>.png`. A game without Swap Zones
/// just has none.
fn movements(game: &Path, folder: &Path) -> Result<usize, String> {
    let Some(bytes) = game_file(game, ZONES)? else {
        return Ok(0);
    };
    let archive = pak::open(&bytes)?;
    let mut done = [false; MOVEMENTS.len()];
    // The archive holds every level's other collectibles too; only the
    // zones' pictures are worth unpacking.
    for file in textures(&archive).into_iter().filter(|file| file.name.to_ascii_lowercase().contains("_sz_")) {
        let Some(picture) = igz::read(&archive.read(file)?) else {
            continue;
        };
        let source = picture.source.to_ascii_lowercase();
        let Some(index) = MOVEMENTS.iter().position(|(_, ending)| source.ends_with(ending)) else {
            continue;
        };
        if done[index] || picture.format != DXT5_TILED || (picture.width, picture.height) != (ZONE_SIZE, ZONE_SIZE) {
            continue;
        }
        let rows = upright(&picture)?;
        let (left, top, wide, high) = badge_box();
        let mut badge = vec![0; wide * high * 4];
        for y in 0..high {
            for x in 0..wide {
                let from = ((top + y) * ZONE_SIZE + left + x) * 4;
                let to = (y * wide + x) * 4;
                badge[to..to + 3].copy_from_slice(&rows[from..from + 3]);
                badge[to + 3] = (u32::from(rows[from + 3]) * badge_cover(left + x, top + y) / 16) as u8;
            }
        }
        write(&folder.join(format!("movement-{}.png", MOVEMENTS[index].0)), wide, high, &badge)?;
        done[index] = true;
    }
    Ok(done.iter().filter(|&&written| written).count())
}

/// The part of a Swap Zone picture the badge fills: left, top, width and
/// height.
fn badge_box() -> (usize, usize, usize, usize) {
    let (x, y) = BADGE_MIDDLE;
    let half_wide = BADGE_RADIUS * 3f32.sqrt() / 2.0;
    let (left, top) = ((x - half_wide).floor(), (y - BADGE_RADIUS).floor());
    let (right, bottom) = ((x + half_wide).ceil(), (y + BADGE_RADIUS).ceil());
    (left as usize, top as usize, (right - left) as usize, (bottom - top) as usize)
}

/// How much of pixel (x, y) of a Swap Zone picture is badge, in sixteenths:
/// the hexagon tried at 4 x 4 points in the pixel, which smooths its edge.
fn badge_cover(x: usize, y: usize) -> u32 {
    let (middle_x, middle_y) = BADGE_MIDDLE;
    let inside = |at_x: f32, at_y: f32| {
        let (across, down) = ((at_x - middle_x).abs(), (at_y - middle_y).abs());
        across <= BADGE_RADIUS * 3f32.sqrt() / 2.0 && down + across / 3f32.sqrt() <= BADGE_RADIUS
    };
    let point = |n: u32| (n as f32 + 0.5) / 4.0;
    (0..16).filter(|n| inside(x as f32 + point(n % 4), y as f32 + point(n / 4))).count() as u32
}

/// A picture's pixels as RGBA, top row first.
fn upright(picture: &igz::Picture) -> Result<Vec<u8>, String> {
    let (width, height) = (picture.width, picture.height);
    // A side that isn't a multiple of 4 is stored as whole blocks; the
    // pixels past it are cut off once decoded. Trap Team's villains are 171.
    let (wide, high) = (width.div_ceil(4), height.div_ceil(4));
    let blocks = gx2::untile(&picture.pixels, wide, high).ok_or("A picture in the game is shorter than its size says.")?;
    let rows = dxt5::decode(&blocks, wide * 4, high * 4);
    // The game keeps its pictures bottom row first.
    Ok(rows.chunks(wide * 16).take(height).rev().flat_map(|row| &row[..width * 4]).copied().collect())
}

fn write(path: &Path, width: usize, height: usize, rgba: &[u8]) -> Result<(), String> {
    let failed = |_| "Couldn't write a picture into the folder.".to_string();
    let file = std::fs::File::create(path).map_err(|_| "Couldn't write a picture into the folder.".to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(failed)?;
    writer.write_image_data(rgba).map_err(failed)?;
    writer.finish().map_err(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_badge_is_cut_along_its_hexagon() {
        assert_eq!(badge_box(), (47, 19, 107, 123));
        assert_eq!(badge_cover(100, 80), 16); // the middle
        assert_eq!(badge_cover(100, 10), 0); // above its top corner
        assert_eq!(badge_cover(20, 80), 0); // left of it
        assert_eq!(badge_cover(150, 25), 0); // past its top right side
        // Its left side runs down x = 47.67, so the pixel at 47 is a quarter badge.
        assert_eq!(badge_cover(47, 80), 4);
    }
}
