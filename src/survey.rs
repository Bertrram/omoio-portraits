//! `survey` prints what a reader of a game's pictures would need to know
//! about a copy of it, and writes nothing: its title id, its files by kind,
//! its archives with the version each is and whether it opens here, and,
//! inside them, the pictures that look like a menu's, with their size and
//! pixel format. It was written on 7 October 2026 for Skylanders
//! SuperChargers, whose files haven't been seen here, so it looks for what
//! the other games' readers use and for what SuperChargers adds: vehicles,
//! trophies, a SuperCharger badge and the Land, Sea and Sky symbols. Long
//! lists are cut short, saying how many were left out. Given a word as
//! well, it prints only the entries whose names hold that word, all of
//! them.
//!
//! On 8 October 2026, for Skylanders Imaginators, whose files haven't been
//! seen here either, it came to look for Senseis and Creation Crystals as
//! well; to tell a file of version 9 that isn't a picture by the kinds of
//! object it holds, as SuperChargers' toy data was found, and, given a word,
//! to print the outside names each gives, as its materials and the pictures
//! they name; and to read a file of a version it doesn't know as version 9
//! lays one out, saying so, so a newer version that kept that layout still
//! shows its pictures.
//!
//! An archive keeps its names at its end, as every archive of the PS3
//! copies of Giants, SWAP Force and Trap Team does, so it is listed from its
//! first bytes and its last ones, and an entry stored as it is is read on
//! its own. Only an archive with a packed entry worth looking at is read
//! whole. That kept a survey of each of those copies to between two and five
//! seconds (7 October 2026).

use crate::{igz, pak, wua};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// The folders of a game folder that hold the game's own files: a Wii U
/// game's code, content and meta, and a PS3 game's PS3_GAME. Anything else
/// in the folder isn't the game's and is left alone.
const GAME_FOLDERS: [&str; 4] = ["code", "content", "meta", "PS3_GAME"];
const META: &str = "meta/meta.xml";
const PARAM_SFO: &str = "PS3_GAME/PARAM.SFO";
/// Enough of an archive's first bytes to read its head, and to show the
/// start of a file whose version isn't known.
const HEAD: u64 = 0x40;
/// Enough of a file's first bytes to tell an archive or a picture.
const MAGIC: u64 = 8;
/// The game's pictures are entries named `.igz`, as all of SWAP Force's
/// and Trap Team's are.
const PICTURE_ENDING: &str = ".igz";

/// Words that mark a picture worth printing, anywhere in its name: what the
/// other games' readers look for (SWAP Force's `Spyro_WiiPortrait`,
/// `Body_Aviator_Combo` and `_elementIcons_D`, Trap Team's
/// `collection_images/.../462_12288_snapshot.png`), what SuperChargers
/// adds, and what Imaginators adds: Senseis, whose battle classes are
/// covered by "class", and Creation Crystals.
const PICTURE_WORDS: [&str; 16] = [
    "portrait",
    "illustration",
    "snapshot",
    "collection",
    "checklist",
    "vehicle",
    "troph",
    "supercharg",
    "element",
    "badge",
    "terrain",
    "class",
    "combo",
    "sensei",
    "crystal",
    "creation",
];
/// Short words that count only on their own, not inside a longer one:
/// SuperChargers' three terrains, Trap Team's `skylander_toys`, SWAP
/// Force's Swap Zones (`SZ_Tuto_Dig`) and Spyro's Adventure's versus screen
/// (`eruptor_vs`).
const WHOLE_WORDS: [&str; 7] = ["land", "sea", "sky", "toy", "toys", "sz", "vs"];
/// Words that mark a whole archive as a menu's, worth looking through, as
/// SWAP Force's `characterillustrations.pak` and `collectibleicons.pak`
/// are. Trap Team's and Giants' menus are archives whose names start with
/// `ui`, which counts too.
const MENU_WORDS: [&str; 10] = [
    "collection",
    "portrait",
    "illustration",
    "checklist",
    "icon",
    "vehicle",
    "troph",
    "supercharg",
    "terrain",
    "badge",
];
const MENU_START: &str = "ui";

/// The game names a picture's pixel format by the FNV-1a hash of the
/// format's name: 0x98cb2a65 is "dxt5_tile_cafe", as the readers' own
/// formats show. So a format is named by trying these names with these
/// endings, and one that matches none is printed as its number.
const FORMATS: [&str; 16] = [
    "dxt1", "dxt3", "dxt5", "a8r8g8b8", "r8g8b8a8", "b8g8r8a8", "x8r8g8b8", "r8g8b8", "r5g6b5", "a1r5g5b5", "a4r4g4b4",
    "a8", "l8", "l8a8", "a8l8", "ati2",
];
const FORMAT_ENDINGS: [&str; 12] = [
    "",
    "_big",
    "_tile",
    "_tile_big",
    "_cafe",
    "_big_cafe",
    "_tile_cafe",
    "_tile_big_cafe",
    "_ps3",
    "_big_ps3",
    "_tile_ps3",
    "_tile_big_ps3",
];

/// How many of each list are printed before the rest are only counted.
/// Given a word, every entry that holds it is printed.
const SHOWN_EXTENSIONS: usize = 12;
const SHOWN_FOLDERS: usize = 12;
const SHOWN_ARCHIVES: usize = 5;
const SHOWN_INSIDE: usize = 15;
const SHOWN_GROUPS: usize = 10;
const SHOWN_ALIKE: usize = 3;
const SHOWN_ENTRIES: usize = 4;
/// How many of the kinds of object a file holds are named, the first first.
const SHOWN_HELD: usize = 6;

/// The game's files, from a .wua or from a game folder.
struct Copy {
    wua: Option<(wua::Archive, Vec<wua::Entry>)>,
    folder: PathBuf,
    /// Each file's path, with `/` between folders, and its size.
    files: Vec<(String, u64)>,
}

impl Copy {
    fn open(game: &Path) -> Result<Self, String> {
        if game.is_dir() {
            let folder = super::game_folder(game).to_path_buf();
            let mut files = Vec::new();
            for inside in GAME_FOLDERS {
                walk(&folder, inside, &mut files, 0);
            }
            if files.is_empty() {
                return Err("Couldn't find a game's files in this folder.".to_string());
            }
            files.sort();
            return Ok(Copy { wua: None, folder, files });
        }
        let archive = wua::Archive::open(game)?;
        let mut entries = archive.files();
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        let files = entries.iter().map(|entry| (entry.path.clone(), entry.size)).collect();
        Ok(Copy { wua: Some((archive, entries)), folder: PathBuf::new(), files })
    }

    /// `length` bytes of file `index` from `from`, fewer where it ends.
    fn read(&mut self, index: usize, from: u64, length: u64) -> Result<Vec<u8>, String> {
        if let Some((archive, entries)) = &mut self.wua {
            return archive.read_range(&entries[index], from, length);
        }
        let unreadable = |_| "Couldn't read one of the game's files.".to_string();
        let mut file = File::open(self.folder.join(&self.files[index].0)).map_err(unreadable)?;
        file.seek(SeekFrom::Start(from)).map_err(unreadable)?;
        let mut bytes = Vec::new();
        file.take(length).read_to_end(&mut bytes).map_err(unreadable)?;
        Ok(bytes)
    }

    fn whole(&mut self, index: usize) -> Result<Vec<u8>, String> {
        self.read(index, 0, self.files[index].1)
    }
}

/// Every file under `inside`, a folder of `root`. Links are passed over, so
/// a folder that leads back into itself can't keep the walk going.
fn walk(root: &Path, inside: &str, files: &mut Vec<(String, u64)>, depth: usize) {
    if depth > 32 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(root.join(inside)) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = format!("{inside}/{}", entry.file_name().to_string_lossy());
        if kind.is_dir() {
            walk(root, &path, files, depth + 1);
        } else if kind.is_file() {
            files.push((path, entry.metadata().map_or(0, |data| data.len())));
        }
    }
}

/// The survey's text, a line at a time.
#[derive(Default)]
struct Report(String);

impl Report {
    fn line(&mut self, text: impl AsRef<str>) {
        self.0.push_str(text.as_ref());
        self.0.push('\n');
    }

    /// Every one of `lines`, each indented by `indent`.
    fn all(&mut self, indent: &str, lines: impl IntoIterator<Item = String>) {
        for text in lines {
            self.line(format!("{indent}{text}"));
        }
    }

    /// Up to `shown` of `lines`, each indented by `indent`, then how many
    /// more there were, as `what` names one and more of them.
    fn list(&mut self, indent: &str, lines: impl IntoIterator<Item = String>, shown: usize, what: [&str; 2]) {
        let lines: Vec<String> = lines.into_iter().collect();
        for text in lines.iter().take(shown) {
            self.line(format!("{indent}{text}"));
        }
        if lines.len() > shown {
            self.line(format!("{indent}and {}", more(lines.len() - shown, what)));
        }
    }
}

/// The word for `count` of something, from its word for one and for more.
fn noun<'a>(count: usize, [one, more]: [&'a str; 2]) -> &'a str {
    if count == 1 {
        one
    } else {
        more
    }
}

/// A count and what it counts: `1 file`, `2 files`.
fn many(count: usize, what: [&str; 2]) -> String {
    format!("{count} {}", noun(count, what))
}

/// How many more there are than were shown: `1 more file`, `2 more files`.
fn more(count: usize, what: [&str; 2]) -> String {
    format!("{count} more {}", noun(count, what))
}

const FILES: [&str; 2] = ["file", "files"];
const PICTURES: [&str; 2] = ["picture", "pictures"];
const ARCHIVES: [&str; 2] = ["archive", "archives"];
const FOLDERS: [&str; 2] = ["folder", "folders"];
const KINDS: [&str; 2] = ["kind", "kinds"];

/// An entry that is a picture, or may be one.
struct Item {
    name: String,
    /// What it is, which alike entries share: `igz 8, dxt5_tile_cafe, 256 x 256`.
    kind: String,
    /// Its size, how it is kept, and the name it was made from where the
    /// game keeps one.
    more: String,
}

/// An archive, or the files outside them, with what was found in it.
struct Inside {
    path: String,
    files: usize,
    /// The entries that are pictures or may be, by the folder each is in.
    groups: BTreeMap<String, Vec<Item>>,
    /// The others, by what they are, each with its name and the outside
    /// names it gives, such as the materials a toy is drawn with.
    others: BTreeMap<String, Vec<(String, Vec<String>)>>,
}

impl Inside {
    fn new(path: String, files: usize) -> Self {
        Inside { path, files, groups: BTreeMap::new(), others: BTreeMap::new() }
    }

    fn pictures(&self) -> usize {
        self.groups.values().map(Vec::len).sum()
    }

    fn add(&mut self, name: &str, kind: String, more: String) {
        let (folder, picture) = split_entry(name);
        self.groups.entry(folder.to_string()).or_default().push(Item { name: picture.to_string(), kind, more });
    }

    fn add_other(&mut self, name: &str, kind: String, naming: Vec<String>) {
        let (_, own) = split_entry(name);
        self.others.entry(kind).or_default().push((own.to_string(), naming));
    }
}

/// The start of the first picture file seen of each version not read here.
type Unknown = BTreeMap<u32, (String, Vec<u8>)>;

pub fn survey(game: &Path, word: Option<&str>) -> Result<String, String> {
    let mut copy = Copy::open(game)?;
    let mut report = Report::default();
    let kind = if copy.wua.is_some() { "a .wua" } else { "a game folder" };
    report.line(format!("omoio-portraits {}, survey of {kind}", env!("CARGO_PKG_VERSION")));
    match super::title(game) {
        Ok(id) => report.line(format!("title {id}")),
        Err(_) => report.line("title unknown"),
    }
    titles(&mut copy, &mut report);

    if word.is_none() {
        extensions(&copy, &mut report);
    }
    let mut archives = Vec::new();
    let mut loose = Vec::new();
    let mut others = 0;
    for index in 0..copy.files.len() {
        let start = copy.read(index, 0, MAGIC).unwrap_or_default();
        if let Some(version) = pak::version(&start) {
            archives.push((index, version));
        } else if igz::version(&start).is_some() {
            loose.push(index);
        } else {
            others += 1;
        }
    }
    if word.is_none() {
        report.line(format!(
            "by their first bytes: {} (IGA), {} outside them (IGZ) and {}",
            many(archives.len(), ARCHIVES),
            many(loose.len(), PICTURES),
            many(others, ["other file", "other files"])
        ));
    }

    let mut unknown = Unknown::new();
    let mut listed = Vec::new();
    let mut insides = Vec::new();
    for (done, &(index, version)) in archives.iter().enumerate() {
        eprint!("\rLooking through archive {} of {}", done + 1, archives.len());
        let start = copy.read(index, 0, HEAD).unwrap_or_default();
        let files = list(&mut copy, index, &start);
        let path = copy.files[index].0.clone();
        let mut wanted = 0;
        if let Some(files) = &files {
            let menu = archive_looks_wanted(&path);
            let chosen: Vec<usize> = (0..files.len())
                .filter(|&at| match word {
                    Some(word) => holds(&path, word) || holds(&files[at].name, word),
                    None => menu || entry_looks_wanted(&files[at].name),
                })
                .collect();
            wanted = chosen.len();
            if !chosen.is_empty() {
                let mut inside = Inside::new(path.clone(), files.len());
                look_inside(&mut copy, index, files, &chosen, &mut inside, &mut unknown);
                insides.push(inside);
            }
        }
        listed.push(Listed { index, version, files: files.map(|files| files.len()), start, wanted });
    }
    if !archives.is_empty() {
        eprintln!();
    }

    let mut outside = Inside::new("files outside the archives".to_string(), loose.len());
    for index in loose {
        let path = copy.files[index].0.clone();
        let chosen = match word {
            Some(word) => holds(&path, word),
            None => entry_looks_wanted(&path),
        };
        if chosen {
            let bytes = copy.whole(index).unwrap_or_default();
            look_at(&mut outside, "", &path, &bytes, size(bytes.len() as u64), &mut unknown);
        }
    }
    if !outside.groups.is_empty() || !outside.others.is_empty() {
        insides.push(outside);
    }

    if word.is_none() {
        archive_list(&copy, &listed, &mut report);
    }
    entries(&mut insides, word, &mut report);
    for (version, (place, start)) in unknown {
        report.line(format!("the first igz {version}, {place}, starts:"));
        report.all("  ", hex_rows(&start));
    }
    Ok(report.0)
}

/// Looks at the `chosen` entries of an archive: each stored as it is on its
/// own, and the rest from the archive read whole, once.
fn look_inside(copy: &mut Copy, index: usize, files: &[pak::PakFile], chosen: &[usize], inside: &mut Inside, unknown: &mut Unknown) {
    let path = copy.files[index].0.clone();
    let more = |file: &pak::PakFile| format!("{}, {}", size(file.size as u64), file.packing());
    let mut packed = Vec::new();
    for &at in chosen {
        let file = &files[at];
        match file.stored_at() {
            Some((offset, length)) => match copy.read(index, offset as u64, length as u64) {
                Ok(bytes) => look_at(inside, &path, &file.name, &bytes, more(file), unknown),
                Err(_) => inside.add(&file.name, "not read here".to_string(), more(file)),
            },
            None => packed.push(at),
        }
    }
    if packed.is_empty() {
        return;
    }
    let bytes = copy.whole(index).unwrap_or_default();
    let archive = pak::open(&bytes);
    for at in packed {
        let file = &files[at];
        match archive.as_ref().map(|archive| archive.read(&archive.files[at])) {
            Ok(Ok(bytes)) => look_at(inside, &path, &file.name, &bytes, more(file), unknown),
            _ => inside.add(&file.name, "not unpacked here".to_string(), more(file)),
        }
    }
}

/// The title of each game in the copy: a .wua from Cemu may hold the game
/// with its update and its DLC, each in a folder of its own.
fn titles(copy: &mut Copy, report: &mut Report) {
    let mut lines = Vec::new();
    for index in 0..copy.files.len() {
        let path = copy.files[index].0.clone();
        if let Some(folder) = path.strip_suffix(META) {
            let Ok(bytes) = copy.whole(index) else {
                continue;
            };
            let meta = String::from_utf8_lossy(&bytes);
            let value = |key: &str| super::meta_value(&meta, key).unwrap_or_default();
            let name = value("longname_en").split_whitespace().collect::<Vec<_>>().join(" ");
            lines.push(format!("{}, version {}, {name}{}", value("title_id"), value("title_version"), place(folder)));
        } else if let Some(folder) = path.strip_suffix(PARAM_SFO) {
            let Ok(sfo) = copy.whole(index) else {
                continue;
            };
            let value = |key: &str| super::sfo_text(&sfo, key).unwrap_or_default();
            lines.push(format!("{}, version {}, {}{}", value("TITLE_ID"), value("APP_VER"), value("TITLE"), place(folder)));
        }
    }
    if !lines.is_empty() {
        report.line("in the copy:");
        report.all("  ", lines);
    }
}

fn place(folder: &str) -> String {
    match folder.trim_end_matches('/') {
        "" => String::new(),
        folder => format!(", in {folder}"),
    }
}

/// The copy's files counted by their extension, the biggest share first.
fn extensions(copy: &Copy, report: &mut Report) {
    let mut kinds: BTreeMap<String, (usize, u64)> = BTreeMap::new();
    for (path, bytes) in &copy.files {
        let (_, name) = cut_last(path);
        let extension = match name.rsplit_once('.') {
            Some((_, extension)) if !extension.is_empty() && extension.len() <= 8 => format!(".{}", extension.to_ascii_lowercase()),
            _ => "no extension".to_string(),
        };
        let kind = kinds.entry(extension).or_default();
        kind.0 += 1;
        kind.1 += bytes;
    }
    let mut kinds: Vec<_> = kinds.into_iter().collect();
    kinds.sort_by(|a, b| b.1 .1.cmp(&a.1 .1).then(a.0.cmp(&b.0)));
    let total: u64 = copy.files.iter().map(|(_, bytes)| bytes).sum();
    report.line(format!("files {}, {}", copy.files.len(), size(total)));
    let lines = kinds.iter().map(|(extension, (count, bytes))| format!("{extension}: {count}, {}", size(*bytes)));
    report.list("  ", lines, SHOWN_EXTENSIONS, ["kind of file", "kinds of file"]);
}

/// What the survey found out about one archive.
struct Listed {
    index: usize,
    version: (u32, bool),
    /// How many files it has, or `None` when it doesn't open here.
    files: Option<usize>,
    start: Vec<u8>,
    wanted: usize,
}

/// The archive's files, listed from its first bytes and its names, which
/// are at its end. `None` when it doesn't open here.
fn list(copy: &mut Copy, index: usize, start: &[u8]) -> Option<Vec<pak::PakFile>> {
    let head = pak::Head::read(start).ok()?;
    let length = copy.files[index].1;
    if head.length() as u64 > length {
        return None;
    }
    let start = copy.read(index, 0, head.length() as u64).ok()?;
    // An archive with no files may end before where its names would start.
    let names = match length.checked_sub(head.names as u64) {
        Some(rest) => copy.read(index, head.names as u64, rest).ok()?,
        None => Vec::new(),
    };
    head.files(&start, &names).ok()
}

fn endian(little: bool) -> &'static str {
    if little {
        "little-endian"
    } else {
        "big-endian"
    }
}

/// Every archive: each version found and how many of it open here, then
/// the archives folder by folder, those with the most worth looking at
/// first, then the biggest.
fn archive_list(copy: &Copy, listed: &[Listed], report: &mut Report) {
    if listed.is_empty() {
        return;
    }
    report.line("archives:");
    let mut versions: BTreeMap<(u32, bool), Vec<&Listed>> = BTreeMap::new();
    for archive in listed {
        versions.entry(archive.version).or_default().push(archive);
    }
    for ((version, little), all) in versions {
        let open = all.iter().filter(|archive| archive.files.is_some()).count();
        let opens = match open {
            _ if open == all.len() => "all open here".to_string(),
            0 => "none open here".to_string(),
            _ => format!("{open} open here"),
        };
        report.line(format!("  IGA {version}, {}: {}, {opens}", endian(little), all.len()));
        if let Some(first) = all.iter().find(|archive| archive.files.is_none()) {
            report.line(format!("    the first that doesn't, {}, starts:", copy.files[first.index].0));
            report.all("      ", hex_rows(&first.start));
        }
    }
    let mut folders: BTreeMap<&str, Vec<&Listed>> = BTreeMap::new();
    for archive in listed {
        let (folder, _) = cut_last(&copy.files[archive.index].0);
        folders.entry(folder).or_default().push(archive);
    }
    let mut folders: Vec<_> = folders.into_iter().collect();
    folders.sort_by_key(|(_, all)| std::cmp::Reverse(all.iter().map(|archive| archive.wanted).sum::<usize>()));
    let total = folders.len();
    for (folder, mut all) in folders.into_iter().take(SHOWN_FOLDERS) {
        let bytes: u64 = all.iter().map(|archive| copy.files[archive.index].1).sum();
        report.line(format!("  in {folder}: {}, {}", all.len(), size(bytes)));
        all.sort_by_key(|archive| (std::cmp::Reverse(archive.wanted), std::cmp::Reverse(copy.files[archive.index].1)));
        let lines = all.iter().map(|archive| {
            let (path, bytes) = &copy.files[archive.index];
            let (_, name) = cut_last(path);
            let (version, little) = archive.version;
            let what = match archive.files {
                Some(files) if archive.wanted > 0 => format!("{}, {} looked at", many(files, FILES), archive.wanted),
                Some(files) => many(files, FILES),
                None => "doesn't open here".to_string(),
            };
            format!("{name}, {}, IGA {version} {}, {what}", size(*bytes), endian(little))
        });
        report.list("    ", lines, SHOWN_ARCHIVES, ARCHIVES);
    }
    if total > SHOWN_FOLDERS {
        report.line(format!("  and {}", more(total - SHOWN_FOLDERS, FOLDERS)));
    }
}

/// What was found, archive by archive, the one with the most pictures
/// first. In each folder the kinds of picture are counted, then a few are
/// named; the entries that aren't pictures are counted by what they are.
fn entries(insides: &mut [Inside], word: Option<&str>, report: &mut Report) {
    let cap = |count: usize| if word.is_some() { usize::MAX } else { count };
    match word {
        Some(word) => report.line(format!("entries whose names hold \"{word}\":")),
        None => report.line("pictures that look like a menu's:"),
    }
    if insides.is_empty() {
        report.line("  none");
        return;
    }
    insides.sort_by_key(|inside| std::cmp::Reverse((inside.pictures(), inside.others.values().map(Vec::len).sum::<usize>())));
    for inside in insides.iter().take(cap(SHOWN_INSIDE)) {
        report.line(format!("{}: {}, {}", inside.path, many(inside.files, FILES), many(inside.pictures(), PICTURES)));
        let mut groups: Vec<_> = inside.groups.iter().collect();
        groups.sort_by_key(|(_, items)| std::cmp::Reverse(items.len()));
        for (folder, items) in groups.iter().take(cap(SHOWN_GROUPS)) {
            let shown = if folder.is_empty() { "(no folder)" } else { folder.as_str() };
            report.line(format!("  {shown}: {}", items.len()));
            let mut alike: BTreeMap<&str, usize> = BTreeMap::new();
            for item in items.iter() {
                *alike.entry(&item.kind).or_default() += 1;
            }
            let mut alike: Vec<_> = alike.into_iter().collect();
            alike.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            let kinds = alike.iter().map(|(kind, count)| format!("{count} x {kind}"));
            report.list("    ", kinds, cap(SHOWN_ALIKE), KINDS);
            let mut named: Vec<_> = items.iter().collect();
            named.sort_by(|a, b| a.name.cmp(&b.name));
            let lines = named.iter().map(|item| format!("{}: {}, {}", item.name, item.kind, item.more));
            report.list("    ", lines, cap(SHOWN_ENTRIES), ["entry", "entries"]);
        }
        if groups.len() > cap(SHOWN_GROUPS) {
            let left: usize = groups.iter().skip(SHOWN_GROUPS).map(|(_, items)| items.len()).sum();
            report.line(format!("  and {}, with {}", more(groups.len() - SHOWN_GROUPS, FOLDERS), many(left, PICTURES)));
        }
        if !inside.others.is_empty() {
            let mut others: Vec<_> = inside.others.iter().collect();
            others.sort_by_key(|(_, entries)| std::cmp::Reverse(entries.len()));
            report.line("  not pictures:");
            // Given a word, each entry is named on a line of its own with the
            // outside names it gives.
            if word.is_some() {
                for (kind, entries) in others {
                    report.line(format!("    {} x {kind}:", entries.len()));
                    let lines = entries.iter().map(|(name, naming)| match naming.as_slice() {
                        [] => name.clone(),
                        _ => format!("{name}, naming {}", naming.join(", ")),
                    });
                    report.all("      ", lines);
                }
            } else {
                let lines = others.iter().map(|(kind, entries)| format!("{} x {kind}, such as {}", entries.len(), entries[0].0));
                report.list("    ", lines, SHOWN_ALIKE, KINDS);
            }
        }
    }
    if insides.len() > cap(SHOWN_INSIDE) {
        let rest = insides.iter().skip(SHOWN_INSIDE).map(|inside| format!("{} ({})", inside.path, inside.pictures()));
        report.line(format!("and {}, with this many pictures:", more(insides.len() - SHOWN_INSIDE, ARCHIVES)));
        report.list("  ", rest, SHOWN_INSIDE, ARCHIVES);
    }
}

/// Looks at one entry, `bytes` once unpacked: a picture, a screen of
/// pictures as Giants keeps them, or something else. A file of version 9
/// that isn't a picture is told by the kinds of object it holds, and keeps
/// the outside names it gives, such as the materials a toy's data names. A
/// file of a version not read here is read as version 9 lays one out, in
/// case a newer version kept that layout, and is said to be read so.
fn look_at(inside: &mut Inside, archive: &str, name: &str, bytes: &[u8], more: String, unknown: &mut Unknown) {
    let Some(version) = igz::version(bytes) else {
        inside.add_other(name, first_bytes(bytes), Vec::new());
        return;
    };
    let known = igz::known(version);
    if !known {
        unknown.entry(version).or_insert_with(|| {
            let place = if archive.is_empty() { name.to_string() } else { format!("{name} in {archive}") };
            (place, bytes[..bytes.len().min(HEAD as usize)].to_vec())
        });
    }
    let objects = if known { igz::Objects::read(bytes) } else { igz::Objects::read_as_version_9(bytes) };
    let label = match (known, &objects) {
        (true, _) => format!("igz {version}"),
        (false, Some(_)) => format!("igz {version} read as igz 9"),
        (false, None) => {
            inside.add(name, format!("igz {version}, a version not read here"), more);
            return;
        }
    };
    let mut pictures = match (igz::screen(bytes), &objects) {
        (Some(pictures), _) => pictures,
        (None, Some(objects)) => objects.picture().into_iter().collect(),
        (None, None) => igz::read(bytes).into_iter().collect(),
    };
    pictures.retain(plausible);
    let describe = |picture: &igz::Picture| format!("{label}, {}, {} x {}", format_name(picture.format), picture.width, picture.height);
    match pictures.as_slice() {
        [] => match &objects {
            Some(objects) => {
                let naming = objects.outside_names().map(|(name, space)| format!("{name} ({space})")).collect();
                inside.add_other(name, format!("{label}, not a picture, {}", held(objects)), naming);
            }
            None => inside.add_other(name, format!("{label}, not a picture"), Vec::new()),
        },
        [picture] => {
            let (_, own) = split_entry(name);
            let more = match picture.source.as_str() {
                "" => more,
                source if own.starts_with(source) => more,
                source => format!("{more}, made from {source}"),
            };
            inside.add(name, describe(picture), more);
        }
        // Giants keeps a whole screen in one file; its pictures are listed
        // together, by the names they were made from.
        _ => {
            let (folder, own) = split_entry(name);
            let group = if folder.is_empty() { own.to_string() } else { format!("{folder}/{own}") };
            let items = inside.groups.entry(format!("{group}, a screen")).or_default();
            for picture in &pictures {
                let more = format!("{} of pixels", size(picture.pixels.len() as u64));
                items.push(Item { name: picture.source.clone(), kind: describe(picture), more });
            }
        }
    }
}

/// The kinds of object a file holds, the first few, as in `holds
/// igObjectList, CFullCharacterToyData and 2 more kinds`.
fn held(objects: &igz::Objects) -> String {
    let kinds = objects.kinds_held();
    match kinds.len() {
        0 => "holds no objects".to_string(),
        count if count > SHOWN_HELD => format!("holds {} and {}", kinds[..SHOWN_HELD].join(", "), more(count - SHOWN_HELD, KINDS)),
        _ => format!("holds {}", kinds.join(", ")),
    }
}

/// Whether what was read is a picture. `igz::read` takes any file of a
/// known version for one, and a model's numbers make no sense as a size: a
/// picture is at most 8192 pixels a side and has at least half a byte of
/// pixels for each, as DXT1, the smallest format, keeps them.
fn plausible(picture: &igz::Picture) -> bool {
    let sides = 1..=8192;
    sides.contains(&picture.width) && sides.contains(&picture.height) && picture.pixels.len() >= picture.width * picture.height / 2
}

/// An entry's folder and the name of the picture it holds. Trap Team keeps
/// each picture in a file named by a hash, in a folder named like the
/// picture (`.../519_12288_triggersnappy.png/0x659c8c69.png.igb.tex.igz`),
/// so there that folder is the picture.
fn split_entry(name: &str) -> (&str, &str) {
    let (folder, last) = cut_last(name);
    match cut_last(folder) {
        (above, picture) if picture.contains('.') => (above, picture),
        _ => (folder, last),
    }
}

/// A path's folder and its last part.
fn cut_last(path: &str) -> (&str, &str) {
    path.rfind(['/', '\\']).map_or(("", path), |at| (&path[..at], &path[at + 1..]))
}

/// The words of a name, small: its runs of letters and digits, split again
/// where a small letter meets a capital (`WiiPortrait`).
fn words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut after_small = false;
    for c in name.chars() {
        let breaks = !c.is_ascii_alphanumeric() || (after_small && c.is_ascii_uppercase());
        if breaks && !word.is_empty() {
            words.push(std::mem::take(&mut word));
        }
        if c.is_ascii_alphanumeric() {
            word.push(c.to_ascii_lowercase());
        }
        after_small = c.is_ascii_lowercase();
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

/// Whether an archive's own name, not its folders', says it holds a menu.
fn archive_looks_wanted(path: &str) -> bool {
    let (_, name) = cut_last(path);
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
    let small = stem.to_ascii_lowercase();
    words(stem).first().is_some_and(|first| first == MENU_START) || MENU_WORDS.iter().any(|word| small.contains(word))
}

/// Whether an entry looks like a menu picture: a picture file with one of
/// the words in its name, or named, as Trap Team's are, by a figure's id
/// and variant.
fn entry_looks_wanted(name: &str) -> bool {
    let small = name.to_ascii_lowercase();
    if !small.ends_with(PICTURE_ENDING) {
        return false;
    }
    let (_, picture) = split_entry(name);
    let mut numbers = picture.split(|c: char| !c.is_ascii_alphanumeric());
    let numbered = matches!((numbers.next(), numbers.next()), (Some(id), Some(variant)) if id.parse::<u16>().is_ok() && variant.parse::<u16>().is_ok());
    numbered
        || PICTURE_WORDS.iter().any(|word| small.contains(word))
        || words(name).iter().any(|word| WHOLE_WORDS.contains(&word.as_str()))
}

fn holds(name: &str, word: &str) -> bool {
    name.to_ascii_lowercase().contains(&word.to_ascii_lowercase())
}

fn fnv1a(text: &str) -> u32 {
    text.bytes().fold(0x811c_9dc5, |hash, byte| (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193))
}

fn format_name(format: u32) -> String {
    FORMATS
        .iter()
        .flat_map(|name| FORMAT_ENDINGS.iter().map(move |ending| format!("{name}{ending}")))
        .find(|name| fnv1a(name) == format)
        .unwrap_or_else(|| format!("format 0x{format:08x}"))
}

fn size(bytes: u64) -> String {
    const UNITS: [&str; 3] = ["KB", "MB", "GB"];
    if bytes < 1024 {
        return format!("{bytes} bytes");
    }
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

/// What a file that isn't a picture starts with: its first four bytes, as
/// letters when they all are.
fn first_bytes(bytes: &[u8]) -> String {
    let start = &bytes[..bytes.len().min(4)];
    if start.is_empty() {
        "empty".to_string()
    } else if start.iter().all(u8::is_ascii_graphic) {
        format!("starts \"{}\"", String::from_utf8_lossy(start))
    } else {
        format!("starts {}", hex_rows(start).concat())
    }
}

fn hex_rows(bytes: &[u8]) -> Vec<String> {
    bytes
        .chunks(16)
        .map(|row| row.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(" "))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_format_is_named_by_its_hash() {
        assert_eq!(fnv1a("dxt5_tile_cafe"), crate::DXT5_TILED);
        assert_eq!(format_name(crate::DXT5_TILED), "dxt5_tile_cafe");
        assert_eq!(format_name(crate::DXT5_PS3[0]), "dxt5_tile_big_ps3");
        assert_eq!(format_name(crate::DXT5_PS3[1]), "dxt5_big_ps3");
        assert_eq!(format_name(fnv1a("dxt1_tile_cafe")), "dxt1_tile_cafe");
        assert_eq!(format_name(0x1234_5678), "format 0x12345678");
    }

    #[test]
    fn an_entry_is_split_into_its_folder_and_picture() {
        let trap_team = "C:/tfb/build/ps3/ui_sky4/collection_images/skylander_toys/2014/2014_skylanders_regular/519_12288_triggersnappy.png/0x659c8c69.png.igb.tex.igz";
        assert_eq!(
            split_entry(trap_team),
            ("C:/tfb/build/ps3/ui_sky4/collection_images/skylander_toys/2014/2014_skylanders_regular", "519_12288_triggersnappy.png")
        );
        assert_eq!(split_entry("textures\\juicestandard_body_firework_1528228744.igz"), ("textures", "juicestandard_body_firework_1528228744.igz"));
        assert_eq!(split_entry("level.bld"), ("", "level.bld"));
    }

    #[test]
    fn names_are_cut_into_words() {
        assert_eq!(words("CatGryphon_WiiPortrait"), ["cat", "gryphon", "wii", "portrait"]);
        assert_eq!(words("ui_sky4/SZ_Tuto_Dig.png"), ["ui", "sky4", "sz", "tuto", "dig", "png"]);
    }

    #[test]
    fn menu_pictures_are_told_by_their_names() {
        // What the readers of the other games use.
        assert!(entry_looks_wanted("textures\\Spyro2012_WiiPortrait.igz"));
        assert!(entry_looks_wanted("textures\\juicestandard_belt_freeze_combo_10.igz"));
        assert!(entry_looks_wanted("x/skylander_toys/2011/2011_skylanders/419_0_legendarytriggerhappy.png/0x298d7392.png.igb.tex.igz"));
        assert!(entry_looks_wanted("x/traps/square/217_12291.png/0x3.igz"));
        assert!(entry_looks_wanted("textures\\collectibles_SZ_Tuto_Dig.igz"));
        // What SuperChargers might name its own.
        assert!(entry_looks_wanted("ui/icons/terrain_land.png/0x1.igz"));
        assert!(entry_looks_wanted("ui/VehicleIconSea.png/0x2.igz"));
        assert!(entry_looks_wanted("ui/trophies/3500_0_sky.png/0x3.igz"));
        // What Imaginators might name its own.
        assert!(entry_looks_wanted("textures\\!ui!Senseis!KnightSensei`tga.igz"));
        assert!(entry_looks_wanted("ui/battleclass/icon_bowslinger.png/0x4.igz"));
        assert!(entry_looks_wanted("textures\\!ui!CreationCrystals!FireCrystal`tga.igz"));
        // Not menu pictures: a short word inside a longer one, sounds, a
        // screen whose archive isn't a menu's.
        assert!(!entry_looks_wanted("textures\\island_rock_d.igz"));
        assert!(!entry_looks_wanted("textures\\destroyable_crate.igz"));
        assert!(!entry_looks_wanted("c:/tfb/build/ps3/levels/champion_audio_sky4/sea_saw/vo_sea_saw_battlecry_02032.wav"));
        assert!(!entry_looks_wanted("c:/tfb/build/wiiu/levels/criminal_audio_sky4/x.wav/0x8.wav.hz.wav.enc"));
        assert!(!entry_looks_wanted("level.bld"));
    }

    #[test]
    fn menu_archives_are_told_by_their_own_names() {
        for wanted in [
            "PS3_GAME/USRDIR/misc/UI_Collection_Champions.arc",
            "PS3_GAME/USRDIR/misc/ui_villainvault_stream.arc",
            "PS3_GAME/USRDIR/misc/ui_cardgame_handselect_streaming.bld",
            "content/archives/characterillustrations.pak",
            "content/archives/collectibleicons.pak",
        ] {
            assert!(archive_looks_wanted(wanted), "{wanted}");
        }
        for not_wanted in [
            "PS3_GAME/USRDIR/level/level_008_islandtown.bld",
            "PS3_GAME/USRDIR/level/level_320_elementoflight.arc",
            "PS3_GAME/USRDIR/character/1006_lightvillain.arc",
            "PS3_GAME/USRDIR/character/101_bugmangiant.arc",
            "PS3_GAME/USRDIR/misc/build_info.arc",
            "content/ui_collection/level_008.arc",
        ] {
            assert!(!archive_looks_wanted(not_wanted), "{not_wanted}");
        }
    }

    #[test]
    fn sizes_read_plainly() {
        assert_eq!(size(512), "512 bytes");
        assert_eq!(size(5 * 1024 * 1024 + 700 * 1024), "5.7 MB");
        assert_eq!(size(20 * 1024 * 1024 * 1024), "20.0 GB");
        assert_eq!(first_bytes(b"IGA\x1a"), "starts 49 47 41 1a");
        assert_eq!(first_bytes(b"strm and more"), "starts \"strm\"");
    }

    /// A Wii U game folder in the temporary folder, with `files` in it.
    fn game_folder(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("omoio-portraits-{}-{name}", std::process::id()));
        for (path, bytes) in files {
            let path = folder.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, bytes).unwrap();
        }
        folder
    }

    fn meta(id: &str, name: &str) -> Vec<u8> {
        format!("<menu><title_id type=\"hexBinary\" length=\"8\">{id}</title_id><title_version type=\"unsignedInt\" length=\"4\">0</title_version><longname_en type=\"string\" length=\"512\">{name}</longname_en></menu>").into_bytes()
    }

    #[test]
    fn a_survey_lists_the_archives_and_the_pictures_in_them() {
        let picture = crate::igz::tests::build(8, None, crate::DXT5_TILED, 16, 16, &[0; 256]);
        // A model read as a picture gives a size that makes no sense.
        let model = crate::igz::tests::build(8, None, 0x55118a24, 17292, 5772, &[0; 64]);
        let newer = b"IGZ\x01\x00\x00\x00\x0a a version not read here";
        let garage = crate::pak::tests::build_trap_team(&[
            ("C:/tfb/build/wiiu/ui/vehicles/3220_0_jetstream.png/0x1.png.igb.tex.igz", &picture),
            ("C:/tfb/build/wiiu/ui/vehicles/3221_0_stealthstinger.png/0x2.png.igb.tex.igz", &picture),
            ("C:/tfb/build/wiiu/ui/badges/badge_supercharger.png/0x3.png.igb.tex.igz", newer),
            ("C:/tfb/build/wiiu/ui/sounds/click.wav/0x4.wav", b"FSB5 and a sound"),
            ("C:/tfb/build/wiiu/models/garage_door.igz", &model),
        ]);
        let level = crate::pak::tests::build_trap_team(&[
            ("C:/tfb/build/wiiu/levels/rocks.png/0x5.igz", &picture),
            ("C:/tfb/build/wiiu/levels/vehicle_shadow.png/0x6.igz", &picture),
        ]);
        let folder = game_folder(
            "survey",
            &[
                ("meta/meta.xml", &meta("00050000101BFC00", "Skylanders\nSuperChargers")),
                ("content/misc/ui_garage.arc", &garage),
                ("content/levels/level_01.arc", &level),
                ("content/misc/newer.arc", b"IGA\x1a\x0c\x00\x00\x00 and a version not known here"),
                ("content/movies/intro.bik", b"BIKi"),
                ("download notes.txt", b"not the game's"),
            ],
        );
        let report = survey(&folder, None).unwrap();
        for wanted in [
            "title 00050000101bfc00",
            "  00050000101BFC00, version 0, Skylanders SuperChargers",
            "files 5,",
            "by their first bytes: 3 archives (IGA), 0 pictures outside them (IGZ) and 2 other files",
            "  IGA 11, little-endian: 2, all open here",
            "  IGA 12, little-endian: 1, none open here",
            "    the first that doesn't, content/misc/newer.arc, starts:",
            "    ui_garage.arc, ",
            "IGA 11 little-endian, 5 files, 5 looked at",
            "IGA 11 little-endian, 2 files, 1 looked at",
            "content/misc/ui_garage.arc: 5 files, 3 pictures",
            "  C:/tfb/build/wiiu/ui/vehicles: 2",
            "    2 x igz 8, dxt5_tile_cafe, 16 x 16",
            "    3220_0_jetstream.png: igz 8, dxt5_tile_cafe, 16 x 16, ",
            "    badge_supercharger.png: igz 10, a version not read here, ",
            "    1 x starts \"FSB5\", such as click.wav",
            "    1 x igz 8, not a picture, such as garage_door.igz",
            "content/levels/level_01.arc: 2 files, 1 picture",
            "    vehicle_shadow.png: igz 8",
            "the first igz 10, C:/tfb/build/wiiu/ui/badges/badge_supercharger.png/0x3.png.igb.tex.igz in content/misc/ui_garage.arc, starts:",
        ] {
            assert!(report.contains(wanted), "{wanted:?} is missing from:\n{report}");
        }
        // Only the game's own folders are looked through, and only the
        // level's entries that look like menu pictures.
        assert!(!report.contains("txt"), "{report}");
        assert!(!report.contains("rocks"), "{report}");

        let report = survey(&folder, Some("Stealth")).unwrap();
        assert!(report.contains("entries whose names hold \"Stealth\":"), "{report}");
        assert!(report.contains("3221_0_stealthstinger.png: igz 8"), "{report}");
        assert!(!report.contains("3220_0_jetstream"), "{report}");
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn a_file_is_told_by_the_first_kinds_it_holds() {
        use crate::igz::tests::{build_objects, object};
        let kinds = ["igObjectList", "B", "C", "D", "E", "F", "G", "H"];
        let objects: Vec<Vec<u8>> = (0..kinds.len() as u32).map(|kind| object(kind, &[])).collect();
        let file = build_objects(&kinds, &[], &[], None, &objects, &[], None);
        assert_eq!(held(&igz::Objects::read(&file).unwrap()), "holds igObjectList, B, C, D, E, F and 2 more kinds");
        let file = build_objects(&kinds[..2], &[], &[], None, &objects[..2], &[], None);
        assert_eq!(held(&igz::Objects::read(&file).unwrap()), "holds igObjectList, B");
    }

    #[test]
    fn a_survey_tells_what_a_file_holds_and_reads_a_newer_one_as_version_9() {
        use crate::igz::tests::{build_objects, build_picture};
        let output = "Temporary/BuildServer/cafe/Output";
        let picture = build_picture(16, &[0; 1024]);
        let mut newer = picture.clone();
        newer[4..8].copy_from_slice(&10u32.to_be_bytes());
        let toy = crate::toys::tests::build_toy("CFullCharacterToyData", 601, "Example", &["Collection_Example_Normal"], &[]);
        let material = build_objects(&["igObjectList"], &["GuiStandard", "graphics_effect", "Example_Icon", "image"], &[(0, 1), (2, 3)], None, &[], &[], None);
        let data = crate::pak::tests::build_chunked(
            0x0b,
            &[
                (&format!("{output}/ToyData/Example_ToyData.igz"), &toy),
                (&format!("{output}/textures/!ui!Senseis!Example_Sensei`tga.igz"), &newer),
                (&format!("{output}/textures/!ui!Crystals!Example_Crystal`tga.igz"), &picture),
                (&format!("{output}/textures/!levels!rocks`tga.igz"), &picture),
            ],
        );
        let collection = crate::pak::tests::build_chunked(0x0b, &[(&format!("{output}/materialInstances/ToyCollection/Collection_Example_Normal.igz"), &material)]);
        let folder = game_folder(
            "survey-holds",
            &[
                ("meta/meta.xml", &meta("00050000101FB100", "Skylanders Imaginators")),
                ("content/archives/permanent.pak", &data),
                ("content/archives/ToyCollectionMaterials.pak", &collection),
            ],
        );
        let report = survey(&folder, None).unwrap();
        for wanted in [
            "title 00050000101fb100",
            "    1 x igz 9, not a picture, holds CFullCharacterToyData, CVariantIdentifierList, such as Example_ToyData.igz",
            "    1 x igz 9, not a picture, holds no objects, such as Collection_Example_Normal.igz",
            "    !ui!Senseis!Example_Sensei`tga.igz: igz 10 read as igz 9, dxt5_tile_cafe, 16 x 16, ",
            "    !ui!Crystals!Example_Crystal`tga.igz: igz 9, dxt5_tile_cafe, 16 x 16, ",
            "the first igz 10, Temporary/BuildServer/cafe/Output/textures/!ui!Senseis!Example_Sensei`tga.igz in content/archives/permanent.pak, starts:",
        ] {
            assert!(report.contains(wanted), "{wanted:?} is missing from:\n{report}");
        }
        assert!(!report.contains("rocks"), "{report}");
        // The outside names are printed only for a word, each entry on a line
        // of its own.
        assert!(!report.contains("naming"), "{report}");

        let report = survey(&folder, Some("ToyData")).unwrap();
        assert!(report.contains("    1 x igz 9, not a picture, holds CFullCharacterToyData, CVariantIdentifierList:\n      Example_ToyData.igz, naming Collection_Example_Normal (Collection_Example_Normal)\n"), "{report}");
        let report = survey(&folder, Some("Collection")).unwrap();
        assert!(report.contains("      Collection_Example_Normal.igz, naming GuiStandard (graphics_effect), Example_Icon (image)\n"), "{report}");
        assert!(!report.contains("Example_ToyData"), "{report}");
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn a_wua_is_surveyed_the_same_way() {
        let picture = crate::igz::tests::build(8, None, crate::DXT5_TILED, 16, 16, &[0; 256]);
        let archive = crate::pak::tests::build_trap_team(&[("ui/vehicles/3220_0_jetstream.png/0x1.igz", &picture)]);
        let path = std::env::temp_dir().join(format!("omoio-portraits-{}-survey.wua", std::process::id()));
        std::fs::write(&path, crate::wua::tests::build(&archive)).unwrap();
        let report = survey(&path, None).unwrap();
        for wanted in [
            "survey of a .wua",
            "title unknown",
            "by their first bytes: 1 archive (IGA), 0 pictures outside them (IGZ) and 0 other files",
            "a.bin, ",
            "IGA 11 little-endian, 1 file, 1 looked at",
            "content/a.bin: 1 file, 1 picture",
            "    3220_0_jetstream.png: igz 8, dxt5_tile_cafe, 16 x 16, ",
        ] {
            assert!(report.contains(wanted), "{wanted:?} is missing from:\n{report}");
        }
        std::fs::remove_file(path).unwrap();
    }
}
