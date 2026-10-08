//! Reads the figure pictures out of the user's own copy of Skylanders
//! Spyro's Adventure, Giants, SWAP Force, Trap Team or SuperChargers, for
//! Omoio's portal menu. Omoio downloads
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
//! whole, its element symbols, and the four magic items it has pictures of,
//! as it shows them on screen. From SuperChargers, a .wua of the Wii U
//! version, it writes every toy and variant its Collection screen shows,
//! vehicles and trophies among them, its ten element symbols, its badge for
//! a SuperCharger, `class-supercharger.png`, and its Land, Sea and Sky
//! symbols as white shapes, `terrain-<name>.png`. It
//! prints `progress <done> <of>` as it goes and `done <written>` at the end.
//! From Imaginators, a .wua of the Wii U version, it writes every toy the
//! same way, its Senseis and Creation Crystals among them, its eleven
//! element symbols, Kaos's included, and its battle classes' symbols as
//! white shapes, `class-<class>.png`, and its badges for a Sensei and an
//! Imaginator as the game draws them, `class-sensei.png` and
//! `class-imaginator.png`.
//! From Spyro's Adventure on the Wii, a copy of the game's files as Dolphin's
//! own tool writes it (the folder `DATA`, which holds `files`), it writes
//! each Skylander whole as its versus screen shows them, and its element
//! symbols, as from the PS3 version, and every magic item and adventure pack
//! as it shows them on screen.
//! Nothing is downloaded: every picture comes from the user's own files.
//!
//! A third way is for working out a game that isn't known yet:
//!
//!     omoio-portraits survey <game.wua or game folder> [<word>]
//!
//! prints what a reader of its pictures would need to know and writes
//! nothing (see `survey`).

mod dxt5;
mod gx;
mod gx2;
mod igz;
mod names;
mod pak;
mod strm;
mod survey;
mod toys;
mod wua;

use std::collections::{HashMap, HashSet};
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
/// each Skylander whole, 512 pixels square, for its versus screen, the
/// element symbols, `elementicon<element>`, whose shapes are their alpha,
/// and the sprites of four of its magic items (see
/// `names::spyros_adventure_toy`), 64 pixels square. The game says death
/// for Undead and mech for Tech.
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
/// Where a Wii game's own files start in a copy of them: under `files` in
/// the folder of its game partition, which Dolphin's own tool names `DATA`
/// and Omoio reads from, or in the folder above it, where Dolphin also
/// takes the partition's folder named `P0`.
const WII_ROOTS: [&str; 3] = ["files/", "DATA/files/", "P0/files/"];
/// Spyro's Adventure on the Wii keeps the pictures of its versus screen in
/// the archive of its versus mode, each Skylander whole in a file of its own
/// with its alpha (see `igz` and `names`), from 271 to 554 pixels wide and
/// 350 high. Its element symbols are in the screen kept for every level,
/// `.../elementalicons/<element>_128.png`, 128 pixels square, whose shapes
/// are their alpha. Seen in the game's files, 8 October 2026.
const SPYROS_ADVENTURE_WII: &str = "misc/PvP_MainControl.arc";
const WII_SYMBOLS: &str = "permanent/global.bld";
const WII_SYMBOL: &str = "_128";
/// Each magic item and adventure pack has an archive of its own,
/// `item/Item_<name>.bld`, with the toy's model and effects, whose screen
/// holds the sprite the game shows it with, 64 pixels square, with its
/// alpha (see `names::SPYROS_ADVENTURE_TOYS`). The game has no bigger
/// picture of them: its Collection screens have Skylanders, treasures, hats
/// and story scrolls but no toys, and the toys' other pictures are their
/// models' textures. Seen in the game's files, 8 October 2026.
const WII_TOYS: &str = "item/";
/// CMPR, the Wii's DXT1, as the game names it: the FNV-1a hash of
/// "dxt1_tile_big_wii". Every picture read from the Wii is in it.
const CMPR: u32 = 0x79dd_819e;
const PARAM_SFO: &str = "PS3_GAME/PARAM.SFO";
const NOT_KNOWN_GAME: &str = "This game has no figure pictures Omoio can read yet. So far that is Skylanders Spyro's Adventure on the PS3 and the Wii, Giants on the PS3, SWAP Force and Trap Team, and SuperChargers and Imaginators on the Wii U.";
/// Skylanders Imaginators on the Wii U, by the last eight digits of its title
/// ids: 00050000101F4D00 and 00050000101FB100, the USA's and Europe's in
/// WiiUBrew's title database (read 7 October 2026), and 0005000010205E00,
/// which Cemu's graphic packs list beside them in the pack that also patches
/// a demo (read 8 October 2026). Its update and DLC share the last eight
/// digits. Its archives have the names SuperChargers' do, so it is told from
/// SuperChargers by its title id.
const IMAGINATORS: [&str; 3] = ["101f4d00", "101fb100", "10205e00"];
/// Imaginators' symbols, seen in Bertram's copy of the European release
/// with its update, 8 October 2026:
/// - its menus' element symbols, flat and coloured, Kaos's among them, as
///   `!MenuComponents!ElementIcon_<Element>2` in `permanent.pak`, made
///   white like the other games';
/// - its badges for a Sensei and an Imaginator in the same archive,
///   `ClassIcon_Sensei3` and `ClassIcon_Imaginator3`, kept as they are;
/// - a gold symbol for each battle class in `permanent_2016.pak`,
///   `ClassIcon_<weapon>2`, named by the class's weapon, made white. Which
///   weapon is which class is the game's own: its pictures for the classes,
///   `SI_CRM_<Class>Class`, carry the same symbols under the classes' names
///   (checked by eye).
const IMAGINATORS_SYMBOLS: &str = "archives/permanent.pak";
const IMAGINATORS_ELEMENTS: [&str; 11] = ["air", "dark", "earth", "fire", "kaos", "life", "light", "magic", "tech", "undead", "water"];
const IMAGINATORS_BADGES: [(&str, &str); 2] = [("sensei", "!ClassIcon_Sensei3`tga"), ("imaginator", "!ClassIcon_Imaginator3`tga")];
const IMAGINATORS_CLASS_ICONS: &str = "archives/permanent_2016.pak";
const IMAGINATORS_CLASSES: [(&str, &str); 11] = [
    ("knight", "Sword"),
    ("bowslinger", "Archer"),
    ("quickshot", "Pistol"),
    ("ninja", "Thrown"),
    ("brawler", "Fist"),
    ("smasher", "Club"),
    ("sorcerer", "Mage"),
    ("swashbuckler", "Blades"),
    ("sentinel", "Doubler"),
    ("bazooker", "Bazooka"),
    ("kaos", "Kaos"),
];
/// SuperChargers keeps its toy data in `permanent.pak` (see `toys`) and the
/// pictures its Collection screen draws each toy with in
/// `ToyCollectionMaterials.pak`: a material for each,
/// `materialInstances/ToyCollection/<name>.igz`, which names its picture in
/// the namespace `image`, an entry `textures/<name>.igz` of the same
/// archive, 88 pixels square. Seen in the game's files, 7 October 2026.
const COLLECTION: &str = "archives/ToyCollectionMaterials.pak";
const TOY_DATA: &str = "archives/permanent.pak";
const TOY_DATA_FOLDER: &str = "/ToyData/";
const MATERIALS: &str = "/materialInstances/ToyCollection/";
const TEXTURES: &str = "/textures/";
const PICTURE_NAMESPACE: &str = "image";
const IGZ: &str = ".igz";
/// SuperChargers' symbols, each found by the end of its picture's name. Its
/// element symbols, white, are those of its Skystones game: Water's is in
/// `permanent.pak` with the toy data and the other nine in every level's
/// archive, read from the hub's. Its badge for a SuperCharger is
/// the bolt it keeps with its coloured element symbols in `permanent.pak`.
/// Its Land, Sea and Sky symbols are the pale vehicle icons of its race
/// menu, where the game says Air for Sky; they are made white like the
/// element symbols.
const SUPERCHARGERS_HUB: &str = "archives/Academy.pak";
const SUPERCHARGERS_ELEMENTS: [&str; 10] = ["air", "dark", "earth", "fire", "life", "light", "magic", "tech", "undead", "water"];
const SUPERCHARGERS_BADGE: &str = "!ui!ElementalIcons!DN_SuperCharged`tga";
const RACE_MENU: &str = "archives/AP_RaceSelect.pak";
const TERRAINS: [(&str, &str); 3] = [
    ("land", "!RaceMenu!LandVehicle_Icon`tga"),
    ("sea", "!RaceMenu!SeaVehicle_Icon`tga"),
    ("sky", "!RaceMenu!AirVehicle_Icon`tga"),
];
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

const USAGE: &str = "Usage: omoio-portraits title <game.wua or game folder>\n       omoio-portraits pictures <game.wua or game folder> <folder>\n       omoio-portraits survey <game.wua or game folder> [<word>]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["title", game] => title(Path::new(game)).map(|id| println!("title {id}")),
        ["pictures", game, folder] => pictures(Path::new(game), Path::new(folder)).map(|n| println!("done {n}")),
        ["survey", game] => survey::survey(Path::new(game), None).map(|report| print!("{report}")),
        ["survey", game, word] => survey::survey(Path::new(game), Some(word)).map(|report| print!("{report}")),
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

/// One of a Wii game's own files, from a copy of them in a folder (see
/// `WII_ROOTS`).
fn wii_file(game: &Path, inside: &str) -> Option<Vec<u8>> {
    if !game.is_dir() {
        return None;
    }
    WII_ROOTS.iter().find_map(|root| std::fs::read(game.join(root).join(inside)).ok())
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
    let id = meta_value(&String::from_utf8_lossy(&meta), "title_id").ok_or_else(unknown)?;
    Ok(id.to_ascii_lowercase())
}

/// A value of a Wii U game's meta.xml, such as its `title_id`.
fn meta_value(meta: &str, key: &str) -> Option<String> {
    let (_, rest) = meta.split_once(&format!("<{key}"))?;
    let (_, rest) = rest.split_once('>')?;
    let (value, _) = rest.split_once('<')?;
    Some(value.trim().to_string())
}

/// Whether a title id, as `title` gives it, is one of Skylanders Imaginators
/// on the Wii U: the game's, its demo's, its update's or its DLC's.
fn is_imaginators(id: &str) -> bool {
    id.len() == 16 && IMAGINATORS.iter().any(|end| id.ends_with(end))
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
/// copy of it is read the same way: SuperChargers' two title ids,
/// 00050000101BFC00 and 00050000101B8500, alike. Imaginators keeps its
/// archives under SuperChargers' names, so between the two the title id
/// decides. The copy Omoio makes of a Wii game holds its files but not the
/// disc's id, so Spyro's Adventure on the Wii is told by its versus mode's
/// archive.
fn pictures(game: &Path, folder: &Path) -> Result<usize, String> {
    if let Some(bytes) = data_file(game, PORTRAITS)? {
        return swap_force(game, &bytes, folder);
    }
    if let Some(bytes) = data_file(game, COLLECTION)? {
        if title(game).is_ok_and(|id| is_imaginators(&id)) {
            return imaginators(game, &bytes, folder);
        }
        return superchargers(game, &bytes, folder);
    }
    // Trap Team on the PS3 has a Collection screen archive named like
    // Giants', so it is told by its Villain Vault first.
    if data_file(game, VILLAIN_VAULT)?.is_some() {
        return trap_team(game, folder);
    }
    if let Some(bytes) = game_file(game, GIANTS)? {
        return giants(game, &bytes, folder);
    }
    if let Some(bytes) = game_file(game, SPYROS_ADVENTURE)? {
        return spyros_adventure(&bytes, folder);
    }
    match wii_file(game, SPYROS_ADVENTURE_WII) {
        Some(bytes) => spyros_adventure_wii(game, &bytes, folder),
        None => Err(NOT_KNOWN_GAME.to_string()),
    }
}

fn superchargers(game: &Path, collection: &[u8], folder: &Path) -> Result<usize, String> {
    let collection = pak::open(collection)?;
    let toy_data = data_file(game, TOY_DATA)?.ok_or(NOT_KNOWN_GAME)?;
    let toy_data = pak::open(&toy_data)?;
    // The symbols, the badge and the terrains come last, a step each.
    let written = toy_pictures(&collection, &toy_data, &toys::SUPERCHARGERS, folder, 3)?;
    let steps = written.steps;
    let done = steps - 3;
    let mut symbols = 0;
    let hub = data_file(game, SUPERCHARGERS_HUB)?.unwrap_or_default();
    let hub = pak::open(&hub).ok();
    for element in SUPERCHARGERS_ELEMENTS {
        let name = format!("!SkyStonesElementicons_{element}_C`tga");
        let path = folder.join(format!("element-{element}.png"));
        let mut found = write_symbol(&toy_data, &name, &path, make_white)?;
        if let (0, Some(hub)) = (found, &hub) {
            found = write_symbol(hub, &name, &path, make_white)?;
        }
        symbols += found;
    }
    println!("progress {} {steps}", done + 1);
    symbols += write_symbol(&toy_data, SUPERCHARGERS_BADGE, &folder.join("class-supercharger.png"), as_drawn)?;
    println!("progress {} {steps}", done + 2);
    let race_menu = data_file(game, RACE_MENU)?.unwrap_or_default();
    if let Ok(race_menu) = pak::open(&race_menu) {
        for (terrain, name) in TERRAINS {
            symbols += write_symbol(&race_menu, name, &folder.join(format!("terrain-{terrain}.png")), make_white)?;
        }
    }
    println!("progress {steps} {steps}");
    Ok(written.count + symbols)
}

fn imaginators(game: &Path, collection: &[u8], folder: &Path) -> Result<usize, String> {
    let collection = pak::open(collection)?;
    let toy_data = data_file(game, IMAGINATORS_SYMBOLS)?.ok_or(NOT_KNOWN_GAME)?;
    let toy_data = pak::open(&toy_data)?;
    // The element symbols, the badges and the classes come last, a step each.
    let written = toy_pictures(&collection, &toy_data, &toys::IMAGINATORS, folder, 3)?;
    let steps = written.steps;
    let done = steps - 3;
    let mut symbols = 0;
    for element in IMAGINATORS_ELEMENTS {
        let name = format!("!MenuComponents!ElementIcon_{}2`tga", capitalised(element));
        symbols += write_symbol(&toy_data, &name, &folder.join(format!("element-{element}.png")), white_without_glow)?;
    }
    println!("progress {} {steps}", done + 1);
    for (badge, name) in IMAGINATORS_BADGES {
        symbols += write_symbol(&toy_data, name, &folder.join(format!("class-{badge}.png")), as_drawn)?;
    }
    println!("progress {} {steps}", done + 2);
    let classes = data_file(game, IMAGINATORS_CLASS_ICONS)?.unwrap_or_default();
    if let Ok(classes) = pak::open(&classes) {
        for (class, weapon) in IMAGINATORS_CLASSES {
            let name = format!("!ClassIcon_{weapon}2`tga");
            symbols += write_symbol(&classes, &name, &folder.join(format!("class-{class}.png")), make_white)?;
        }
    }
    println!("progress {steps} {steps}");
    Ok(written.count + symbols)
}

/// "air" as the game writes it in a file's name, "Air".
fn capitalised(word: &str) -> String {
    let mut letters = word.chars();
    letters.next().map(|first| first.to_uppercase().chain(letters).collect()).unwrap_or_default()
}

/// What `toy_pictures` wrote: how many pictures, and how many steps of
/// progress the whole reading takes.
struct Written {
    count: usize,
    steps: usize,
}

/// Every toy's Collection picture, as SuperChargers and Imaginators keep
/// them: a material for each toy in `collection`, which names its picture,
/// and the toys themselves in `toy_data`. Progress is printed a toy at a
/// time, with `more` steps left for the caller.
fn toy_pictures(collection: &pak::Pak, toy_data: &pak::Pak, layout: &toys::Layout, folder: &Path, more: usize) -> Result<Written, String> {
    let mut images = HashMap::new();
    for file in collection.files.iter().filter(|file| file.name.contains(TEXTURES)) {
        if let Some(name) = file.name.split(TEXTURES).nth(1).and_then(|name| name.strip_suffix(IGZ)) {
            images.insert(name, file);
        }
    }
    let mut materials = HashMap::new();
    for file in collection.files.iter().filter(|file| file.name.contains(MATERIALS)) {
        let Some(name) = file.name.rsplit('/').next().and_then(|name| name.strip_suffix(IGZ)) else {
            continue;
        };
        let bytes = collection.read(file)?;
        let picture = igz::Objects::read(&bytes).and_then(|objects| objects.outside_in(PICTURE_NAMESPACE).find_map(|picture| images.get(picture).copied()));
        if let Some(picture) = picture {
            materials.insert(name, picture);
        }
    }
    let mut toy_files = Vec::new();
    for file in toy_data.files.iter().filter(|file| file.name.contains(TOY_DATA_FOLDER) && file.name.ends_with(IGZ)) {
        toy_files.push(toy_data.read(file)?);
    }
    let toys = toys::pictures(toy_files.iter().map(Vec::as_slice), layout);
    std::fs::create_dir_all(folder).map_err(|_| "Couldn't make the folder for the pictures.".to_string())?;
    let steps = toys.len() + more;
    let mut written = HashSet::new();
    for (done, toy) in toys.iter().enumerate() {
        println!("progress {done} {steps}");
        let names: Vec<u16> = toy.variants.iter().copied().filter(|&variant| !written.contains(&(toy.id, variant))).collect();
        let Some(&texture) = materials.get(toy.material.as_str()) else {
            continue;
        };
        let Some(picture) = igz::read(&collection.read(texture)?).filter(|picture| dxt5(picture.format)) else {
            continue;
        };
        let rgba = upright(&picture)?;
        for variant in names {
            write(&folder.join(format!("{}-{variant:04x}.png", toy.id)), picture.width, picture.height, &rgba)?;
            written.insert((toy.id, variant));
        }
    }
    println!("progress {} {steps}", toys.len());
    Ok(Written { count: written.len(), steps })
}

/// The picture of an archive whose name holds `name`, written to `path`
/// after `finish` has had it, such as `make_white`. Gives how many were
/// written: 0 when the archive has none.
fn write_symbol(archive: &pak::Pak, name: &str, path: &Path, finish: fn(&mut [u8])) -> Result<usize, String> {
    for file in archive.files.iter().filter(|file| file.name.contains(name)) {
        let Some(picture) = igz::read(&archive.read(file)?).filter(|picture| dxt5(picture.format)) else {
            continue;
        };
        let mut rgba = upright(&picture)?;
        finish(&mut rgba);
        write(path, picture.width, picture.height, &rgba)?;
        return Ok(1);
    }
    Ok(0)
}

/// A badge kept as the game draws it.
fn as_drawn(_: &mut [u8]) {}

/// A symbol as a white shape, for Omoio to colour through its alpha.
fn make_white(rgba: &mut [u8]) {
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[..3].fill(255);
    }
}

/// Imaginators' flat element symbols glow: their edge fades out over some
/// eighteen pixels, and the lines inside them are drawn half see-through
/// (measured on its Fire symbol, 8 October 2026). Made white as they are,
/// the glow would blur the shape, so only what is nearly solid is kept, and
/// the lines inside come out as gaps, as the game draws them.
const SOLID_FROM: u8 = 176;
const SOLID_AT: u8 = 240;

fn white_without_glow(rgba: &mut [u8]) {
    make_white(rgba);
    for pixel in rgba.chunks_exact_mut(4) {
        let over = u32::from(pixel[3].saturating_sub(SOLID_FROM));
        pixel[3] = (over * 255 / u32::from(SOLID_AT - SOLID_FROM)).min(255) as u8;
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
        let toy = names::spyros_adventure_toy(&picture.name);
        let element = picture
            .name
            .strip_prefix(SPYRO_SYMBOL)
            .and_then(|own| SPYRO_ELEMENTS.iter().find(|(game, _)| *game == own))
            .map(|&(_, omoio)| omoio)
            .filter(|element| !symbols_done.contains(element));
        if figure.is_none() && toy.is_none() && element.is_none() {
            continue;
        }
        symbols_done.extend(element);
        let (width, height) = (picture.width, picture.height);
        let blocks = picture.pixels.get(..width * height).ok_or("A picture in the game is shorter than its size says.")?;
        let mut rgba = dxt5::decode(&strm::blocks(blocks), width, height);
        if let Some(id) = figure {
            let small = shrink(&rgba, width, height, PORTRAIT_SIDE);
            write(&folder.join(format!("{id}-0000.png")), PORTRAIT_SIDE, PORTRAIT_SIDE, &small)?;
        } else if let Some(id) = toy {
            without_mark(&mut rgba, width, height);
            let (square, side) = around_shape(&rgba, width, height);
            write(&folder.join(format!("{id}-0000.png")), side, side, &square)?;
        } else if let Some(element) = element {
            make_white(&mut rgba);
            without_mark(&mut rgba, width, height);
            write(&folder.join(format!("element-{element}.png")), width, height, &rgba)?;
        }
        written += 1;
    }
    println!("progress {steps} {steps}");
    Ok(written)
}

/// Spyro's Adventure on the PS3 marks its small pictures, the element
/// symbols and the toys' sprites, with ten pixels in their last 4 x 4
/// corner, well apart from the shape, which would show as a speck. The
/// mark is cleared.
fn without_mark(rgba: &mut [u8], width: usize, height: usize) {
    for y in height.saturating_sub(4)..height {
        for x in width.saturating_sub(4)..width {
            rgba[(y * width + x) * 4 + 3] = 0;
        }
    }
}

/// Spyro's Adventure on the Wii: each Skylander as its versus screen shows
/// them, set in a square and shrunk to the size the PS3 version's are
/// written at, the element symbols, and the magic items and adventure packs.
fn spyros_adventure_wii(game: &Path, bytes: &[u8], folder: &Path) -> Result<usize, String> {
    let archive = pak::open(bytes)?;
    let versus: Vec<_> = archive.files.iter().filter_map(|file| names::spyros_adventure_wii(&file.name).map(|id| (file, id))).collect();
    std::fs::create_dir_all(folder).map_err(|_| "Couldn't make the folder for the pictures.".to_string())?;
    // The element symbols and the toys come last, a step each.
    let steps = versus.len() + 2;
    let mut written = 0;
    for (done, (file, id)) in versus.iter().enumerate() {
        println!("progress {done} {steps}");
        let pictures = igz::screen(&archive.read(file)?).unwrap_or_default();
        let Some((picture, rgba)) = pictures.first().and_then(|picture| Some((picture, wii_upright(picture)?))) else {
            continue;
        };
        let side = picture.width.max(picture.height);
        let small = shrink(&squared(&rgba, picture.width, picture.height), side, side, PORTRAIT_SIDE);
        write(&folder.join(format!("{id}-0000.png")), PORTRAIT_SIDE, PORTRAIT_SIDE, &small)?;
        written += 1;
    }
    println!("progress {} {steps}", versus.len());
    written += wii_symbols(game, folder)?;
    println!("progress {} {steps}", versus.len() + 1);
    written += wii_toys(game, folder)?;
    println!("progress {steps} {steps}");
    Ok(written)
}

/// Spyro's Adventure's magic items and adventure packs on the Wii, each the
/// sprite the game shows it with, read from the toy's own archive (see
/// `names::SPYROS_ADVENTURE_TOYS`) and cut to the square around its shape,
/// so it fills its place as a Skylander does: `<id>-0000.png`. Sky-Iron
/// Shield's archive also has a texture of its model named "shield", without
/// an alpha, so the sprite is the picture of that name with one. A copy
/// without a toy's archive, or with one that doesn't read, just has no
/// picture of it.
fn wii_toys(game: &Path, folder: &Path) -> Result<usize, String> {
    let mut written = 0;
    for &(sprite, archive, id) in names::SPYROS_ADVENTURE_TOYS {
        let Some(bytes) = wii_file(game, &format!("{WII_TOYS}{archive}.bld")) else {
            continue;
        };
        let Ok(pictures) = screen_pictures(&bytes) else {
            continue;
        };
        let found = pictures
            .iter()
            .filter(|picture| picture.source == sprite && picture.alpha.is_some())
            .find_map(|picture| Some((picture, wii_upright(picture)?)));
        let Some((picture, rgba)) = found else {
            continue;
        };
        let (square, side) = around_shape(&rgba, picture.width, picture.height);
        write(&folder.join(format!("{id}-0000.png")), side, side, &square)?;
        written += 1;
    }
    Ok(written)
}

/// Spyro's Adventure's eight element symbols on the Wii, written as white
/// shapes, `element-<name>.png`. A copy without the screen they are in just
/// has none.
fn wii_symbols(game: &Path, folder: &Path) -> Result<usize, String> {
    let Some(bytes) = wii_file(game, WII_SYMBOLS) else {
        return Ok(0);
    };
    let pictures = screen_pictures(&bytes)?;
    let mut written = 0;
    for element in ELEMENTS {
        let name = format!("{element}{WII_SYMBOL}");
        let found = pictures.iter().filter(|picture| picture.source == name).find_map(|picture| Some((picture, wii_upright(picture)?)));
        let Some((picture, shape)) = found else {
            continue;
        };
        let (mut shape, side) = around_shape(&shape, picture.width, picture.height);
        make_white(&mut shape);
        write(&folder.join(format!("element-{element}.png")), side, side, &shape)?;
        written += 1;
    }
    Ok(written)
}

/// A symbol cut to the smallest square around its shape, the shape in the
/// middle, and the square's side. The Wii's symbols leave some twenty of
/// their 128 pixels clear on every side, where the PS3's fill theirs, and
/// Omoio fits a symbol's picture to the place it shows it in.
fn around_shape(rgba: &[u8], width: usize, height: usize) -> (Vec<u8>, usize) {
    let shown = |x: usize, y: usize| rgba[(y * width + x) * 4 + 3] > 0;
    let columns: Vec<usize> = (0..width).filter(|&x| (0..height).any(|y| shown(x, y))).collect();
    let rows: Vec<usize> = (0..height).filter(|&y| (0..width).any(|x| shown(x, y))).collect();
    let (Some(&left), Some(&right), Some(&top), Some(&bottom)) = (columns.first(), columns.last(), rows.first(), rows.last()) else {
        return (squared(rgba, width, height), width.max(height));
    };
    let wide = right + 1 - left;
    let cut: Vec<u8> = (top..=bottom).flat_map(|y| &rgba[(y * width + left) * 4..(y * width + right + 1) * 4]).copied().collect();
    let high = bottom + 1 - top;
    (squared(&cut, wide, high), wide.max(high))
}

/// One of the Wii's pictures as RGBA, top row first, its alpha taken from
/// the picture kept for it: a grey one, whose green has the most bits of
/// the three. `None` for a picture not in CMPR, or shorter than its size
/// says.
fn wii_upright(picture: &igz::Picture) -> Option<Vec<u8>> {
    if picture.format != CMPR {
        return None;
    }
    let (width, height) = (picture.width, picture.height);
    let mut rgba = gx::decode(&picture.pixels, width, height)?;
    if let Some(alpha) = &picture.alpha {
        let alpha = gx::decode(alpha, width, height)?;
        for (pixel, cover) in rgba.chunks_exact_mut(4).zip(alpha.chunks_exact(4)) {
            pixel[3] = cover[1];
        }
    }
    // The game keeps its pictures bottom row first.
    Some(rgba.chunks(width * 4).rev().flatten().copied().collect())
}

/// A picture set in the middle of a clear square as wide as its longer
/// side, so it shrinks without being squeezed.
fn squared(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let side = width.max(height);
    let (left, top) = ((side - width) / 2, (side - height) / 2);
    let mut out = vec![0; side * side * 4];
    for (y, row) in rgba.chunks_exact(width * 4).enumerate() {
        let at = ((top + y) * side + left) * 4;
        out[at..at + width * 4].copy_from_slice(row);
    }
    out
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

/// The pictures of the screen one of Giants' archives holds, or one of
/// Spyro's Adventure's on the Wii.
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
    fn a_wii_u_game_is_told_by_its_meta_xml() {
        let meta = "<menu><title_version type=\"unsignedInt\">16</title_version><title_id type=\"hexBinary\" length=\"8\">00050000101BFC00</title_id></menu>";
        assert_eq!(meta_value(meta, "title_id"), Some("00050000101BFC00".to_string()));
        assert_eq!(meta_value(meta, "title_version"), Some("16".to_string()));
        assert_eq!(meta_value(meta, "longname_en"), None);
    }

    #[test]
    fn imaginators_is_told_by_its_title_ids() {
        assert!(is_imaginators("00050000101f4d00"));
        assert!(is_imaginators("00050000101fb100"));
        assert!(is_imaginators("0005000010205e00")); // the demo
        assert!(is_imaginators("0005000e101fb100")); // its update
        assert!(!is_imaginators("00050000101bfc00")); // SuperChargers
        assert!(!is_imaginators("101fb100"));
        assert!(!is_imaginators("BLES02240"));
    }

    #[test]
    fn a_glowing_symbol_keeps_only_its_solid_part() {
        // Glow, half see-through lines inside, the edge, and the solid shape.
        let mut rgba = [10, 20, 30, 0, 9, 9, 9, 100, 9, 9, 9, 176, 9, 9, 9, 208, 9, 9, 9, 240, 9, 9, 9, 255];
        white_without_glow(&mut rgba);
        let alphas: Vec<u8> = rgba.chunks_exact(4).map(|pixel| pixel[3]).collect();
        assert_eq!(alphas, [0, 0, 0, 127, 255, 255]);
        assert!(rgba.chunks_exact(4).all(|pixel| pixel[..3] == [255, 255, 255]));
    }

    #[test]
    fn imaginators_pictures_come_from_its_toy_data_and_its_menus() {
        use crate::igz::tests::{build_objects, build_picture};
        let output = "Temporary/BuildServer/cafe/Output";
        let texture = |name: &str| format!("{output}/textures/GuiStandard_diffuse,textures@{name}`tga,101.igz");
        let icon = "!ui!Collections!ToyInventoryIcons!2016Characters!S601_Sensei_KingPen";
        let material = build_objects(
            &["igObjectList"],
            &["GuiStandard", "graphics_effect", &format!("GuiStandard_diffuse,textures@{icon}`tga,101"), "image"],
            &[(0, 1), (2, 3)],
            None,
            &[],
            &[],
            None,
        );
        let picture = build_picture(4, &[0; 1024]);
        let collection = pak::tests::build_chunked(
            0x0b,
            &[
                (&format!("{output}/materialInstances/ToyCollection/Collection_KingPen_Normal.igz"), &material),
                (&texture(icon), &picture),
            ],
        );
        // King Pen, with his own variant in Imaginators' year.
        let toy = toys::tests::build_toy_in(&toys::IMAGINATORS, "CFullCharacterToyData", 601, "KingPen", &["Collection_KingPen_Normal", "Collection_KingPen_Normal"], &[(0, 5, 0)]);
        let permanent = pak::tests::build_chunked(
            0x0b,
            &[
                (&format!("{output}/ToyData/KingPen_ToyData.igz"), &toy),
                (&texture("!UI_TFB!MenuComponents!ElementIcon_Kaos2"), &picture),
                (&texture("!UI_TFB!MenuComponents!ElementIcon_Kaos"), &picture),
                (&texture("!UI_TFB!ClassIcon_Sensei3"), &picture),
            ],
        );
        let classes = pak::tests::build_chunked(0x0b, &[(&texture("!UI_TFB!ClassIcon_Sword2"), &picture)]);
        let game = std::env::temp_dir().join(format!("omoio-portraits-{}-imaginators", std::process::id()));
        std::fs::create_dir_all(game.join("meta")).unwrap();
        let meta = "<menu><title_id type=\"hexBinary\" length=\"8\">00050000101FB100</title_id></menu>";
        std::fs::write(game.join("meta").join("meta.xml"), meta).unwrap();
        std::fs::create_dir_all(game.join("content/archives")).unwrap();
        for (name, bytes) in [("ToyCollectionMaterials.pak", &collection), ("permanent.pak", &permanent), ("permanent_2016.pak", &classes)] {
            std::fs::write(game.join("content/archives").join(name), bytes).unwrap();
        }
        let folder = game.join("pictures");
        assert_eq!(pictures(&game, &folder), Ok(6));
        let mut written: Vec<String> = std::fs::read_dir(&folder).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned()).collect();
        written.sort();
        assert_eq!(written, ["601-0000.png", "601-5000.png", "601-5100.png", "class-knight.png", "class-sensei.png", "element-kaos.png"]);
        std::fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn superchargers_pictures_come_from_its_toy_data() {
        use crate::igz::tests::{build_objects, build_picture};
        let output = "Temporary/BuildServer/cafe/Output";
        let texture = |name: &str| format!("{output}/textures/GuiStandard_diffuse,textures@{name}`tga,101.igz");
        let icon = |name: &str| format!("!ui!Collections!ToyInventoryIcons!2015Characters!{name}");
        let material = |name: &str| {
            let picture = format!("GuiStandard_diffuse,textures@{}`tga,101", icon(name));
            build_objects(&["igObjectList"], &["GuiStandard", "graphics_effect", &picture, "image"], &[(0, 1), (2, 3)], None, &[], &[], None)
        };
        // A picture's pixels: whole micro tiles of blocks, all clear.
        let picture = build_picture(4, &[0; 1024]);
        let (normal, legendary) = (material("DriverJetVac"), material("DriverJetVac_Legendary"));
        let collection = pak::tests::build_chunked(
            0x0b,
            &[
                (&format!("{output}/materialInstances/ToyCollection/Collection_DriverJetVac_Normal.igz"), &normal),
                (&format!("{output}/materialInstances/ToyCollection/Collection_DriverJetVac_Legendary.igz"), &legendary),
                (&texture(&icon("DriverJetVac")), &picture),
                (&texture(&icon("DriverJetVac_Legendary")), &picture),
            ],
        );
        let toy = toys::tests::build_toy(
            "CFullCharacterToyData",
            3413,
            "DriverJetVac",
            &["Collection_DriverJetVac_Normal", "Collection_DriverJetVac_Legendary"],
            &[(3, 4, 0x0001_0000)],
        );
        // A toy whose material isn't in the collection gets no picture.
        let unseen = toys::tests::build_toy("CVehicleToyData", 3220, "AirJet", &["Collection_VehicleSparHawk_Normal"], &[]);
        let toy_data = pak::tests::build_chunked(
            0x0b,
            &[
                (&format!("{output}/ToyData/DriverJetVac_ToyData.igz"), &toy),
                (&format!("{output}/ToyData/AirJet_ToyData.igz"), &unseen),
                (&format!("{output}/ToyData/DriverJetVac_ToyData_en.lng"), b"words"),
                (&texture("!ui!ElementalIcons!DN_SuperCharged"), &picture),
                (&texture("!levels!Whiterooms!WR_SkyStonesSmash!SkyStonesElementicons_water_C"), &picture),
            ],
        );
        let race_menu = pak::tests::build_chunked(0x0b, &[(&texture("!ui!ActionPacks!RaceMenu!SeaVehicle_Icon"), &picture)]);
        let hub = pak::tests::build_chunked(0x0b, &[(&texture("!levels!Whiterooms!WR_SkyStonesSmash!SkyStonesElementicons_air_C"), &picture)]);
        let game = std::env::temp_dir().join(format!("omoio-portraits-{}-superchargers", std::process::id()));
        for (name, bytes) in [
            ("ToyCollectionMaterials.pak", &collection),
            ("permanent.pak", &toy_data),
            ("AP_RaceSelect.pak", &race_menu),
            ("Academy.pak", &hub),
        ] {
            std::fs::create_dir_all(game.join("content/archives")).unwrap();
            std::fs::write(game.join("content/archives").join(name), bytes).unwrap();
        }
        let folder = game.join("pictures");
        assert_eq!(pictures(&game, &folder), Ok(7));
        let mut written: Vec<String> = std::fs::read_dir(&folder).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned()).collect();
        written.sort();
        assert_eq!(written, ["3413-0000.png", "3413-4403.png", "3413-4503.png", "class-supercharger.png", "element-air.png", "element-water.png", "terrain-sea.png"]);
        std::fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn a_picture_is_set_in_the_middle_of_a_square() {
        let (red, blue, clear) = ([255, 0, 0, 255], [0, 0, 255, 255], [0; 4]);
        // Square already: as it was.
        assert_eq!(squared(&[red, blue, blue, red].concat(), 2, 2), [red, blue, blue, red].concat());
        // Three high, one wide: it stands in the middle column.
        let square = squared(&[red, blue, red].concat(), 1, 3);
        assert_eq!(square, [clear, red, clear, clear, blue, clear, clear, red, clear].concat());
        // Two wide, one high: the row goes at the top of the two.
        assert_eq!(squared(&[red, blue].concat(), 2, 1), [red, blue, clear, clear].concat());
    }

    #[test]
    fn a_symbol_is_cut_to_the_square_around_its_shape() {
        // Five square, the shape two wide and three high off to the right.
        let (white, faint, clear) = ([255, 255, 255, 255], [255, 255, 255, 1], [0; 4]);
        let rows = [
            [clear, clear, clear, clear, clear],
            [clear, clear, clear, white, faint],
            [clear, clear, clear, white, white],
            [clear, clear, clear, faint, white],
            [clear, clear, clear, clear, clear],
        ];
        let (cut, side) = around_shape(&rows.concat().concat(), 5, 5);
        assert_eq!(side, 3);
        let wanted = [[white, faint, clear], [white, white, clear], [faint, white, clear]];
        assert_eq!(cut, wanted.concat().concat());
        // A picture with nothing in it stays as it is.
        assert_eq!(around_shape(&[0; 16], 2, 2), (vec![0; 16], 2));
    }

    #[test]
    fn spyros_adventure_on_the_wii_is_read_from_dolphins_copy() {
        use crate::igz::tests::build_wii;
        // Pictures of one tile of CMPR, four blocks: each block a colour and
        // black, and a byte for each of its rows saying which.
        let block = |colour: u16, rows: [u8; 4]| [colour.to_be_bytes().to_vec(), vec![0, 0], rows.to_vec()].concat();
        let tile = |colour: u16| vec![block(colour, [0; 4]); 4].concat();
        let (red, white) = (tile(0xf800), tile(0xffff));
        // An alpha whose first pixel as kept is black, so clear.
        let corner = [block(0xffff, [0b01_00_00_00, 0, 0, 0]), block(0xffff, [0; 4]), block(0xffff, [0; 4]), block(0xffff, [0; 4])].concat();
        let versus = build_wii(&[("", 1, 8, 8, &red), ("", 0, 8, 8, &white)]);
        let screen = build_wii(&[
            ("levels/includes/ui_main/sprites/elementalicons/fire_128.png", 1, 8, 8, &white),
            ("levels/includes/ui_main/sprites/elementalicons/fire_128.png_ALPHACHANNEL", 0, 8, 8, &corner),
            ("levels/includes/ui_main/sprites/elementalicons/life_64.png", 1, 8, 8, &white),
        ]);
        // Sky-Iron Shield's sprite, 16 x 8 with its right half clear, after
        // a texture of its model with the same name and no alpha.
        let (blue, black) = (tile(0x001f), tile(0x0000));
        let toy = build_wii(&[
            ("models/objects/powerups_items/item_shield/shield.png", 1, 8, 8, &red),
            ("levels/includes/ui_magicitems/sprites/shield.png", 1, 16, 8, &[blue.clone(), blue].concat()),
            ("levels/includes/ui_magicitems/sprites/shield.png_ALPHACHANNEL", 0, 16, 8, &[white.clone(), black].concat()),
        ]);
        let models = "C:/tfb/build/wii/Models/characters/MinionsMonsters";
        let pvp = pak::tests::build_wii(&[], (&format!("{models}/SpyroJr/Sprite/SpyroLeft_VS.png/0xf848aa48.png.igb.tex.igz"), &versus));
        let global = pak::tests::build_wii(&[("FRENCH.pak", b"words"), ("level.bld", &screen)], ("ENGLISH.pak", b"words"));
        let shield = pak::tests::build_wii(&[("level.bld", &toy)], ("ENGLISH.pak", b"words"));
        let game = std::env::temp_dir().join(format!("omoio-portraits-{}-wii", std::process::id()));
        let files = game.join("DATA").join("files");
        for (inside, name, bytes) in [("misc", "PvP_MainControl.arc", &pvp), ("permanent", "global.bld", &global), ("item", "Item_Shield.bld", &shield)] {
            std::fs::create_dir_all(files.join(inside)).unwrap();
            std::fs::write(files.join(inside).join(name), bytes).unwrap();
        }
        // Omoio gives the folder Dolphin's own tool wrote, DATA.
        let folder = game.join("pictures");
        assert_eq!(pictures(&game.join("DATA"), &folder), Ok(3));
        let mut written: Vec<String> = std::fs::read_dir(&folder).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned()).collect();
        written.sort();
        assert_eq!(written, ["16-0000.png", "205-0000.png", "element-fire.png"]);
        let read = |name: &str| {
            let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(folder.join(name)).unwrap()));
            let mut reader = decoder.read_info().unwrap();
            let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
            let info = reader.next_frame(&mut pixels).unwrap();
            (info.width, info.height, pixels)
        };
        let (width, height, spyro) = read("16-0000.png");
        assert_eq!((width, height), (256, 256));
        assert_eq!(&spyro[..4], [255, 0, 0, 255]);
        // The symbol is clear in its bottom left corner, the game keeping
        // its pictures bottom row first.
        let (width, height, fire) = read("element-fire.png");
        assert_eq!((width, height), (8, 8));
        let alpha = |x: usize, y: usize| fire[(y * 8 + x) * 4 + 3];
        assert_eq!((alpha(0, 0), alpha(0, 7), alpha(1, 7)), (255, 0, 255));
        assert_eq!(&fire[..3], [255, 255, 255]);
        // The shield is its sprite cut to its shape, the half that shows.
        let (width, height, shield) = read("205-0000.png");
        assert_eq!((width, height), (8, 8));
        assert!(shield.chunks_exact(4).all(|pixel| pixel == [0, 0, 255, 255]));
        // The folder above DATA does as well.
        std::fs::remove_dir_all(&folder).unwrap();
        assert_eq!(pictures(&game, &folder), Ok(3));
        std::fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn spyros_adventure_on_the_ps3_gives_four_magic_items() {
        use flate2::write::ZlibEncoder;
        use std::io::Write;
        // DXT5 blocks as the game keeps them, colour before alpha: red, and
        // solid or clear.
        let block = |alpha: u8| [vec![0x00, 0xf8, 0, 0, 0, 0, 0, 0], vec![alpha, alpha, 0, 0, 0, 0, 0, 0]].concat();
        // Healing Elixir's sprite: its top left block shows, the bottom
        // right one is the game's mark, the rest is clear.
        let potion = [block(255), block(0), block(0), block(255)].concat();
        let unpacked = strm::tests::stream(&[
            strm::tests::package("potion", 8, &potion),
            // A hat's picture, named like Anvil Rain's sprite on the Wii.
            strm::tests::package("anvil", 8, &potion),
        ]);
        let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&unpacked).unwrap();
        let file = [b"strm".to_vec(), vec![0; 16], encoder.finish().unwrap()].concat();
        let game = std::env::temp_dir().join(format!("omoio-portraits-{}-spyro-ps3", std::process::id()));
        std::fs::create_dir_all(game.join("PS3_GAME/USRDIR")).unwrap();
        std::fs::write(game.join("PS3_GAME/USRDIR/uigameimage.str"), file).unwrap();
        let folder = game.join("pictures");
        assert_eq!(pictures(&game, &folder), Ok(1));
        let written: Vec<String> = std::fs::read_dir(&folder).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned()).collect();
        assert_eq!(written, ["202-0000.png"]);
        // Without its mark it is cut to the block that shows.
        let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(folder.join("202-0000.png")).unwrap()));
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        assert_eq!((info.width, info.height), (4, 4));
        assert!(pixels.chunks_exact(4).all(|pixel| pixel == [255, 0, 0, 255]));
        std::fs::remove_dir_all(game).unwrap();
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
