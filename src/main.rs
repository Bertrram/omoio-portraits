//! Reads the figure pictures out of the user's own copy of Skylanders SWAP
//! Force, for Omoio's portal menu. Omoio downloads this program only when
//! the user asks for the pictures, and runs it two ways:
//!
//!     omoio-portraits title <game.wua>
//!     omoio-portraits pictures <game.wua or unpacked game folder> <folder>
//!
//! `title` prints `title <title id>`, so Omoio can tell which game a .wua
//! is. `pictures` writes one PNG per figure into the folder, the right way
//! up, named `<id>-<variant>.png` with the variant as four hex digits, and
//! prints `progress <done> <of>` as it goes and `done <written>` at the end.
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

fn pictures(game: &Path, folder: &Path) -> Result<usize, String> {
    let bytes = game_file(game, PORTRAITS)?
        .ok_or("This game has no figure pictures Omoio can read yet. So far that is Skylanders SWAP Force.")?;
    let archive = pak::open(&bytes)?;
    std::fs::create_dir_all(folder).map_err(|_| "Couldn't make the folder for the pictures.".to_string())?;
    let textures: Vec<&pak::PakFile> = archive.files.iter().filter(|file| file.name.starts_with("textures\\")).collect();
    let mut written = 0;
    for (done, file) in textures.iter().enumerate() {
        println!("progress {done} {}", textures.len());
        let Some(picture) = igz::read(&archive.read(file)?) else {
            continue;
        };
        if picture.format != DXT5_TILED {
            continue;
        }
        if let Some((id, variant)) = names::figure(&picture.source) {
            write(&picture, &folder.join(format!("{id}-{variant:04x}.png")))?;
            written += 1;
        }
    }
    println!("progress {0} {0}", textures.len());
    Ok(written)
}

fn write(picture: &igz::Picture, path: &Path) -> Result<(), String> {
    let failed = |_| "Couldn't write a picture into the folder.".to_string();
    let (width, height) = (picture.width, picture.height);
    let blocks = gx2::untile(&picture.pixels, width / 4, height / 4).ok_or("A picture in the game is shorter than its size says.")?;
    let rows = dxt5::decode(&blocks, width, height);
    // The game keeps its pictures bottom row first.
    let upright: Vec<u8> = rows.chunks(width * 4).rev().flatten().copied().collect();
    let file = std::fs::File::create(path).map_err(|_| "Couldn't write a picture into the folder.".to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(failed)?;
    writer.write_image_data(&upright).map_err(failed)?;
    writer.finish().map_err(failed)
}
