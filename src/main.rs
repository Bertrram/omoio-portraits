//! Reads the figure pictures out of the user's own copy of Skylanders
//! Spyro's Adventure, Giants, SWAP Force or Trap Team, for Omoio's portal
//! menu. Omoio downloads
//! this program only when the user asks for the pictures, and runs it two
//! ways:
//!
//!     omoio-portraits title <game.wua or game folder>
//!     omoio-portraits pictures <game.wua or game folder> <folder>
//!
//! `title` prints `title <title id>`, so Omoio can tell which game a .wua
//! or a PS3 game folder is. `pictures` writes one PNG per figure into the
//! folder, the right way up, named `<id>-<variant>.png` with the variant as
//! four hex digits, the game's eight element symbols as white shapes,
//! `element-<name>.png`, for Omoio to colour, and the badges of the eight
//! ways a swapper moves, `movement-<name>.png`. From Trap Team it writes
//! every figure and trap as its Collection screen shows them, with the same
//! names, and each villain in a trap and out of one, `villain-<number>.png`
//! and `villain-<number>-loose.png`. From Giants, a PS3 game folder, it
//! writes every figure, magic item and sidekick its Collection screen shows,
//! its element symbols, and its badge for a Giant, `class-giant.png`.
//! SWAP Force and Trap Team give the same pictures from a .wua of the Wii U
//! version as from a folder of the PS3 one. From Spyro's Adventure, a PS3
//! game folder, it writes each Skylander as its versus screen shows them,
//! whole, and its element symbols. It
//! prints `progress <done> <of>` as it goes and `done <written>` at the end.
//! Nothing is downloaded: every picture comes from the user's own files.

mod dxt5;
mod gx2;
mod igz;
mod names;
mod pak;
mod strm;
mod wua;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Where the game's own files start: on the Wii U in `content`, on the PS3
/// in `PS3_GAME/USRDIR`. SWAP Force and Trap Team keep the same archives
/// under the same names on both.
const ROOTS: [&str; 2] = ["content/", "PS3_GAME/USRDIR/"];
/// The archive in SWAP Force that holds every figure's portrait.
const PORTRAITS: &str = "archives/characterillustrations.pak";
/// Trap Team's pictures, each in the archive of the screen that shows them:
/// the Collection screen's figures, its traps, and the Villain Vault.
const TRAP_TEAM: [&str; 3] = [
    "misc/UI_Collection_Champions.arc",
    "misc/ui_collection_traps.arc",
    VILLAIN_VAULT,
];
const VILLAIN_VAULT: &str = "misc/ui_villainvault_stream.arc";
/// Giants keeps each of its screens in an archive of its own: the Collection
/// screen's has a picture of every figure, and the card game's the element
/// symbols. The screen itself is the archive's `level.bld`; its other files
/// are its words in each language.
const GIANTS: &str = "PS3_GAME/USRDIR/misc/ui_collection_champions.bld";
const GIANTS_SYMBOLS: &str = "PS3_GAME/USRDIR/misc/ui_cardgame_handselect_streaming.bld";
/// Giants' badge for a Giant, a horned head on a disc, is in none of its
/// menu screens but in the screens of its levels, so it is read from the
/// hub's level, or failing that from any level that has it.
const GIANTS_LEVELS: &str = "PS3_GAME/USRDIR/level";
const GIANTS_HUB: &str = "level_008_islandtown.bld";
const GIANT_BADGE: &str = "icon_giant";
/// The badge is 512 pixels square; Omoio shows it far smaller than that.
const BADGE_SIDE: usize = 128;
const SCREEN: &str = "level.bld";
/// Giants' element symbols are named `<element>_glass256`.
const GLASS_SYMBOL: &str = "_glass256";
const ELEMENTS: [&str; 8] = ["air", "earth", "fire", "life", "magic", "tech", "undead", "water"];
/// Giants' symbols are glass, a see-through shape with a faint glow around
/// it, and Omoio paints a symbol through its alpha. So the glow, which stays
/// under 64, is cleared, and the glass, whose flat inside is 176, made
/// solid, keeping the shape's soft edge and the lines cut into it.
const GLOW: u32 = 64;
const GLASS: u32 = 176;
/// Spyro's Adventure keeps its menus' pictures in one file (see `strm`):
/// each Skylander whole, 512 pixels square, for its versus screen, and the
/// element symbols, `elementicon<element>`, whose shapes are their alpha.
/// The game says death for Undead and mech for Tech.
const SPYROS_ADVENTURE: &str = "PS3_GAME/USRDIR/uigameimage.str";
const SPYRO_SYMBOL: &str = "elementicon";
const SPYRO_ELEMENTS: [(&str, &str); 8] = [
    ("air", "air"),
    ("death", "undead"),
    ("earth", "earth"),
    ("fire", "fire"),
    ("life", "life"),
    ("magic", "magic"),
    ("mech", "tech"),
    ("water", "water"),
];
/// The size SWAP Force's portraits are, which Omoio shows them at.
const PORTRAIT_SIDE: usize = 256;
const PARAM_SFO: &str = "PS3_GAME/PARAM.SFO";
const NOT_KNOWN_GAME: &str = "This game has no figure pictures Omoio can read yet. So far that is Skylanders Spyro's Adventure and Giants on the PS3, SWAP Force and Trap Team.";
/// The archive with the town's elemental stones, whose texture has the eight
/// element symbols as white shapes, four across and two down, in this order
/// (checked by eye).
const SYMBOLS: &str = "archives/town3_elementalstones.pak";
const SYMBOLS_PICTURE: &str = "_elementIcons_D";
const SYMBOL_ORDER: [&str; 8] = ["tech", "water", "air", "undead", "magic", "life", "fire", "earth"];
/// The archive of the pictures a level's list of Swap Zones shows: the
/// zone's badge, a hexagon standing on a point, on a piece of the level.
const ZONES: &str = "archives/collectibleicons.pak";
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
/// DXT5 as Giants keeps it on the PS3, by the hashes of "dxt5_tile_big_ps3"
/// and "dxt5_big_ps3": in both the blocks are in rows, as on a PC.
const DXT5_PS3: [u32; 2] = [0x942d_575f, 0xf8bb_b422];

fn dxt5(format: u32) -> bool {
    format == DXT5_TILED || DXT5_PS3.contains(&format)
}

const USAGE: &str = "Usage: omoio-portraits title <game.wua or game folder>\n       omoio-portraits pictures <game.wua or game folder> <folder>";

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
        return Ok(std::fs::read(game_folder(game).join(inside)).ok());
    }
    let mut archive = wua::Archive::open(game)?;
    match archive.files().into_iter().find(|file| file.path.ends_with(inside)) {
        Some(file) => archive.read(&file).map(Some),
        None => Ok(None),
    }
}

/// One of the game's own files, by its path under `content` on the Wii U or
/// `PS3_GAME/USRDIR` on the PS3.
fn data_file(game: &Path, inside: &str) -> Result<Option<Vec<u8>>, String> {
    for root in ROOTS {
        if let Some(bytes) = game_file(game, &format!("{root}{inside}"))? {
            return Ok(Some(bytes));
        }
    }
    Ok(None)
}

/// The folder a game's paths start from. A PS3 game may be given as the
/// folder that holds PS3_GAME, or as PS3_GAME or its USRDIR.
fn game_folder(game: &Path) -> &Path {
    let named = |path: &Path, name: &str| path.file_name().is_some_and(|own| own.eq_ignore_ascii_case(name));
    match game.parent() {
        Some(parent) if named(game, "USRDIR") && named(parent, "PS3_GAME") => parent.parent().unwrap_or(parent),
        Some(parent) if named(game, "PS3_GAME") => parent,
        _ => game,
    }
}

/// The game's title id: a PS3 game's from its PARAM.SFO, a Wii U game's
/// from its meta.xml.
fn title(game: &Path) -> Result<String, String> {
    let unknown = || "Couldn't tell which game this is.".to_string();
    if let Some(sfo) = game_file(game, PARAM_SFO)? {
        return sfo_text(&sfo, "TITLE_ID").ok_or_else(unknown);
    }
    let meta = game_file(game, META)?.ok_or_else(unknown)?;
    let meta = String::from_utf8_lossy(&meta);
    let (_, rest) = meta.split_once("<title_id").ok_or_else(unknown)?;
    let (_, rest) = rest.split_once('>').ok_or_else(unknown)?;
    let (id, _) = rest.split_once('<').ok_or_else(unknown)?;
    Ok(id.trim().to_ascii_lowercase())
}

/// A text value of a PS3 game's PARAM.SFO. After the magic and a version
/// come where the keys and the values start and how many there are, then
/// 16 bytes for each: the key's offset, its format, the value's length, the
/// room kept for it and its offset. Everything is little-endian.
fn sfo_text(sfo: &[u8], wanted: &str) -> Option<String> {
    let word = |at: usize| sfo.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize);
    if sfo.get(..4)? != b"\0PSF" {
        return None;
    }
    let (keys, values) = (word(8)?, word(12)?);
    (0..word(16)?).find_map(|index| {
        let entry = 20 + index * 16;
        let key_at = keys + sfo.get(entry..entry + 2).map(|b| usize::from(u16::from_le_bytes([b[0], b[1]])))?;
        if sfo.get(key_at..)?.split(|&b| b == 0).next()? != wanted.as_bytes() {
            return None;
        }
        let value_at = values + word(entry + 12)?;
        let value = sfo.get(value_at..value_at + word(entry + 4)?)?;
        Some(String::from_utf8_lossy(value).trim_end_matches('\0').to_string())
    })
}

/// An archive's textures; its other files are models, sounds and scripts.
fn textures<'a>(archive: &'a pak::Pak) -> Vec<&'a pak::PakFile> {
    archive.files.iter().filter(|file| file.name.starts_with("textures\\")).collect()
}

/// Tells the game by its files rather than its title id, so every region's
/// copy of it is read the same way.
fn pictures(game: &Path, folder: &Path) -> Result<usize, String> {
    if let Some(bytes) = data_file(game, PORTRAITS)? {
        return swap_force(game, &bytes, folder);
    }
    // Trap Team on the PS3 has a Collection screen archive named like
    // Giants', so it is told by its Villain Vault first.
    if data_file(game, VILLAIN_VAULT)?.is_some() {
        return trap_team(game, folder);
    }
    if let Some(bytes) = game_file(game, GIANTS)? {
        return giants(game, &bytes, folder);
    }
    match game_file(game, SPYROS_ADVENTURE)? {
        Some(bytes) => spyros_adventure(&bytes, folder),
        None => Err(NOT_KNOWN_GAME.to_string()),
    }
}

fn spyros_adventure(bytes: &[u8], folder: &Path) -> Result<usize, String> {
    let unpacked = strm::unpack(bytes)?;
    let pictures = strm::pictures(&unpacked)?;
    std::fs::create_dir_all(folder).map_err(|_| "Couldn't make the folder for the pictures.".to_string())?;
    let steps = pictures.len();
    let mut written = 0;
    // The symbols are in the file twice, once for each screen that shows them.
    let mut symbols_done = Vec::new();
    for (done, picture) in pictures.iter().enumerate() {
        println!("progress {done} {steps}");
        if picture.format != strm::DXT5 {
            continue;
        }
        let figure = names::spyros_adventure(&picture.name);
        let element = picture
            .name
            .strip_prefix(SPYRO_SYMBOL)
            .and_then(|own| SPYRO_ELEMENTS.iter().find(|(game, _)| *game == own))
            .map(|&(_, omoio)| omoio)
            .filter(|element| !symbols_done.contains(element));
        if figure.is_none() && element.is_none() {
            continue;
        }
        symbols_done.extend(element);
        let (width, height) = (picture.width, picture.height);
        let blocks = picture.pixels.get(..width * height).ok_or("A picture in the game is shorter than its size says.")?;
        let mut rgba = dxt5::decode(&strm::blocks(blocks), width, height);
        if let Some(id) = figure {
            let small = shrink(&rgba, width, height, PORTRAIT_SIDE);
            write(&folder.join(format!("{id}-0000.png")), PORTRAIT_SIDE, PORTRAIT_SIDE, &small)?;
        } else if let Some(element) = element {
            for pixel in rgba.chunks_exact_mut(4) {
                pixel[..3].fill(255);
            }
            // Every symbol has a mark of ten pixels in its last 4 x 4 corner,
            // well apart from the shape, which would show as a speck.
            for y in height - 4..height {
                for x in width - 4..width {
                    rgba[(y * width + x) * 4 + 3] = 0;
                }
            }
            write(&folder.join(format!("element-{element}.png")), width, height, &rgba)?;
        }
        written += 1;
    }
    println!("progress {steps} {steps}");
    Ok(written)
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
        if !dxt5(picture.format) {
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
        archives.push(data_file(game, inside)?.ok_or(NOT_KNOWN_GAME)?);
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
        if !dxt5(picture.format) {
            continue;
        }
        write(&folder.join(format!("{name}.png")), picture.width, picture.height, &upright(&picture)?)?;
        written += 1;
    }
    println!("progress {steps} {steps}");
    Ok(written)
}

fn giants(game: &Path, bytes: &[u8], folder: &Path) -> Result<usize, String> {
    let pictures = screen_pictures(bytes)?;
    std::fs::create_dir_all(folder).map_err(|_| "Couldn't make the folder for the pictures.".to_string())?;
    // The element symbols and the Giant badge come last, a step each.
    let steps = pictures.len() + 2;
    let mut written = 0;
    for (done, picture) in pictures.iter().enumerate() {
        println!("progress {done} {steps}");
        let Some((id, variant)) = names::giants(&picture.source) else {
            continue;
        };
        if !DXT5_PS3.contains(&picture.format) {
            continue;
        }
        write(&folder.join(format!("{id}-{variant:04x}.png")), picture.width, picture.height, &upright(picture)?)?;
        written += 1;
    }
    println!("progress {} {steps}", pictures.len());
    written += giants_symbols(game, folder)?;
    println!("progress {} {steps}", pictures.len() + 1);
    written += giant_badge(game, folder)?;
    println!("progress {steps} {steps}");
    Ok(written)
}

/// The pictures of the screen one of Giants' archives holds.
fn screen_pictures(bytes: &[u8]) -> Result<Vec<igz::Picture>, String> {
    let archive = pak::open(bytes)?;
    let screen = archive.files.iter().find(|file| file.name == SCREEN).ok_or(NOT_KNOWN_GAME)?;
    igz::screen(&archive.read(screen)?).ok_or_else(|| NOT_KNOWN_GAME.to_string())
}

/// Giants' eight element symbols, which its card game shows, written as
/// `element-<name>.png`. A copy without the card game just has none.
fn giants_symbols(game: &Path, folder: &Path) -> Result<usize, String> {
    let Some(bytes) = game_file(game, GIANTS_SYMBOLS)? else {
        return Ok(0);
    };
    let mut written = 0;
    for picture in screen_pictures(&bytes)? {
        let Some(element) = picture.source.strip_suffix(GLASS_SYMBOL) else {
            continue;
        };
        if !ELEMENTS.contains(&element) || !DXT5_PS3.contains(&picture.format) {
            continue;
        }
        let mut shape = upright(&picture)?;
        solid(&mut shape);
        write(&folder.join(format!("element-{element}.png")), picture.width, picture.height, &shape)?;
        written += 1;
    }
    Ok(written)
}

/// A glass symbol as a white shape: its glow cleared and its glass solid.
fn solid(rgba: &mut [u8]) {
    for pixel in rgba.chunks_exact_mut(4) {
        let alpha = (u32::from(pixel[3]).saturating_sub(GLOW) * 255 / (GLASS - GLOW)).min(255);
        pixel.copy_from_slice(&[255, 255, 255, alpha as u8]);
    }
}

/// Giants' badge for a Giant, written as `class-giant.png`, 128 pixels
/// square. A copy without it just has none.
fn giant_badge(game: &Path, folder: &Path) -> Result<usize, String> {
    let levels = game_folder(game).join(GIANTS_LEVELS);
    let mut others: Vec<PathBuf> = std::fs::read_dir(&levels)
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default();
    others.retain(|path| {
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        name.to_ascii_lowercase().ends_with(".bld") && !name.eq_ignore_ascii_case(GIANTS_HUB)
    });
    others.sort();
    for archive in std::iter::once(levels.join(GIANTS_HUB)).chain(others) {
        let Ok(bytes) = std::fs::read(&archive) else {
            continue;
        };
        // A level whose screen doesn't read is passed over like one without
        // the badge; another level may still have it.
        let Ok(pictures) = screen_pictures(&bytes) else {
            continue;
        };
        let found = pictures.iter().find(|picture| picture.source == GIANT_BADGE && DXT5_PS3.contains(&picture.format));
        let Some(badge) = found else {
            continue;
        };
        let small = shrink(&upright(badge)?, badge.width, badge.height, BADGE_SIDE);
        write(&folder.join("class-giant.png"), BADGE_SIDE, BADGE_SIDE, &small)?;
        return Ok(1);
    }
    Ok(0)
}

/// A picture shrunk to `side` pixels square, each pixel the average of the
/// ones it covers. Colour is weighted by alpha, so the clear pixels around
/// a shape don't darken its edge.
fn shrink(rgba: &[u8], width: usize, height: usize, side: usize) -> Vec<u8> {
    let span = |at: usize, whole: usize| {
        let start = at * whole / side;
        start..((at + 1) * whole / side).max(start + 1)
    };
    let mut out = Vec::with_capacity(side * side * 4);
    for y in 0..side {
        for x in 0..side {
            let (rows, columns) = (span(y, height), span(x, width));
            let count = (rows.len() * columns.len()) as u64;
            let mut sum = [0u64; 4];
            for row in rows {
                for pixel in rgba[(row * width + columns.start) * 4..(row * width + columns.end) * 4].chunks_exact(4) {
                    let alpha = u64::from(pixel[3]);
                    for channel in 0..3 {
                        sum[channel] += u64::from(pixel[channel]) * alpha;
                    }
                    sum[3] += alpha;
                }
            }
            let colour = |channel: usize| (sum[channel] + sum[3] / 2).checked_div(sum[3]).unwrap_or(0) as u8;
            out.extend_from_slice(&[colour(0), colour(1), colour(2), ((sum[3] + count / 2) / count) as u8]);
        }
    }
    out
}

/// The eight element symbols, written as `element-<name>.png`. A game
/// without the town's stones just has none.
fn symbols(game: &Path, folder: &Path) -> Result<usize, String> {
    let Some(bytes) = data_file(game, SYMBOLS)? else {
        return Ok(0);
    };
    let archive = pak::open(&bytes)?;
    for file in textures(&archive) {
        let Some(picture) = igz::read(&archive.read(file)?) else {
            continue;
        };
        if picture.source != SYMBOLS_PICTURE || !dxt5(picture.format) {
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
    let Some(bytes) = data_file(game, ZONES)? else {
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
        if done[index] || !dxt5(picture.format) || (picture.width, picture.height) != (ZONE_SIZE, ZONE_SIZE) {
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
    let blocks = if picture.format == DXT5_TILED {
        gx2::untile(&picture.pixels, wide, high)
    } else {
        picture.pixels.get(..wide * high * 16).map(<[u8]>::to_vec)
    }
    .ok_or("A picture in the game is shorter than its size says.")?;
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

    #[test]
    fn a_glass_symbol_becomes_a_solid_white_shape() {
        let mut pixels = [9, 9, 9, 0, 200, 100, 50, 64, 30, 30, 30, 120, 255, 255, 255, 176, 1, 2, 3, 255];
        solid(&mut pixels);
        assert_eq!(pixels, [255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 127, 255, 255, 255, 255, 255, 255, 255, 255]);
    }

    #[test]
    fn a_shrunk_picture_averages_what_each_pixel_covers() {
        // Four square to two square: the top left pixel covers a red pixel,
        // a darker red one and two clear ones, which add cover but no colour.
        let red = [200, 0, 0, 255];
        let dark = [100, 0, 0, 255];
        let clear = [9, 9, 9, 0];
        let green = [0, 255, 0, 255];
        let rows = [
            [red, dark, green, green],
            [clear, clear, green, green],
            [clear, clear, clear, clear],
            [clear, clear, clear, clear],
        ]
        .concat()
        .concat();
        let top = [150, 0, 0, 128, 0, 255, 0, 255];
        assert_eq!(shrink(&rows, 4, 4, 2), [top, [0; 8]].concat());
    }

    /// A PARAM.SFO holding `entries`, laid out as a PS3 game's.
    fn build_sfo(entries: &[(&str, &str)]) -> Vec<u8> {
        let keys_at = 20 + entries.len() * 16;
        let keys: Vec<u8> = entries.iter().flat_map(|(key, _)| [key.as_bytes(), b"\0"].concat()).collect();
        let values_at = (keys_at + keys.len()).next_multiple_of(4);
        let mut out = vec![0; values_at];
        out[0..4].copy_from_slice(b"\0PSF");
        out[4..8].copy_from_slice(&0x101u32.to_le_bytes());
        out[8..12].copy_from_slice(&(keys_at as u32).to_le_bytes());
        out[12..16].copy_from_slice(&(values_at as u32).to_le_bytes());
        out[16..20].copy_from_slice(&(entries.len() as u32).to_le_bytes());
        out[keys_at..keys_at + keys.len()].copy_from_slice(&keys);
        let mut key_offset = 0;
        for (index, (key, value)) in entries.iter().enumerate() {
            let entry = 20 + index * 16;
            let value_offset = out.len() - values_at;
            out[entry..entry + 2].copy_from_slice(&(key_offset as u16).to_le_bytes());
            out[entry + 2..entry + 4].copy_from_slice(&0x0204u16.to_le_bytes());
            out[entry + 4..entry + 8].copy_from_slice(&(value.len() as u32 + 1).to_le_bytes());
            out[entry + 8..entry + 12].copy_from_slice(&32u32.to_le_bytes());
            out[entry + 12..entry + 16].copy_from_slice(&(value_offset as u32).to_le_bytes());
            out.extend_from_slice(value.as_bytes());
            out.resize(out.len() + 32 - value.len(), 0);
            key_offset += key.len() + 1;
        }
        out
    }

    #[test]
    fn a_ps3_game_is_told_by_its_param_sfo() {
        let sfo = build_sfo(&[("CATEGORY", "DG"), ("TITLE", "Skylanders Giants"), ("TITLE_ID", "BLES01689")]);
        assert_eq!(sfo_text(&sfo, "TITLE_ID"), Some("BLES01689".to_string()));
        assert_eq!(sfo_text(&sfo, "TITLE"), Some("Skylanders Giants".to_string()));
        assert_eq!(sfo_text(&sfo, "APP_VER"), None);
        assert_eq!(sfo_text(b"PK\x03\x04", "TITLE_ID"), None);
    }

    #[test]
    fn a_ps3_game_is_read_from_the_folder_that_holds_ps3_game() {
        let disc = Path::new("D:\\Games\\Skylanders Giants");
        assert_eq!(game_folder(disc), disc);
        assert_eq!(game_folder(&disc.join("PS3_GAME")), disc);
        assert_eq!(game_folder(&disc.join("PS3_GAME").join("USRDIR")), disc);
        assert_eq!(game_folder(&disc.join("ps3_game").join("usrdir")), disc);
        assert_eq!(game_folder(Path::new("D:\\Games\\USRDIR")), Path::new("D:\\Games\\USRDIR"));
    }
}
