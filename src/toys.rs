//! Which figure each of Skylanders SuperChargers' collection pictures shows,
//! from the game's own toy data: a file for each toy in the `ToyData` folder
//! of `permanent.pak`, read as objects (see `igz::Objects`). Worked out from
//! the game's files on 7 October 2026 and checked against Cemu 2.6's figure
//! list the same day.
//!
//! A toy is an object whose kind's name ends `ToyData` (CFullCharacterToyData,
//! CVehicleToyData, CTrapToyData and so on). It keeps its id at 0x08, a list
//! of its variants at 0x1C, at 0x20 a handle to the material the Collection
//! screen draws it with, and its code name at 0x2C, such as `JetVac`. Each
//! variant, a CVariantIdentifier, keeps its deco at 0x08, its year at 0x0C
//! (all ones for any year), four bytes saying LightCore, alternate deco and
//! new pose at 0x10, and its own material at 0x1C. Built into a variant
//! number as the figures carry it, year in the top four bits, then new pose
//! 0x800, alternate deco 0x400, LightCore 0x200 and the deco, 192 of the
//! game's 252 variants are on Cemu's list as they are, among them Dark Hot
//! Streak 0x4402 and LightCore Jet-Vac 0x1206.
//!
//! Fifteen more are on it with 0x100 set as well, every SuperChargers
//! character's variant among them (Legendary Hurricane Jet Vac is 0x4503,
//! where the game's own fields give 0x4403), as on Dolphin's list, which
//! also gives the plain SuperChargers 0x4100 and the plain vehicles 0x4000.
//! The game's variants have no field for that bit, so it reads a figure the
//! same with or without it, and each picture is named under both numbers.
//! A plain toy is named `<id>-0000`, the name Omoio looks for when a
//! figure's own variant has no picture, so it serves a figure made with
//! 0x0000, as Cemu makes them, and with 0x4000 or 0x4100 alike.
//!
//! A SWAP Force swapper's two halves are toys of their own, `Body_Cowboy`
//! and `Legs_Cowboy`, and only the top has a material: the game shows the
//! swapper by its top. So a bottom is given its top's pictures, variant by
//! variant.

use crate::igz::Objects;
use std::collections::HashMap;

const TOY: &str = "ToyData";
const ID: usize = 0x08;
const VARIANTS: usize = 0x1c;
const MATERIAL: usize = 0x20;
const CODE_NAME: usize = 0x2c;
const DECO: usize = 0x08;
const YEAR: usize = 0x0c;
const FLAGS: usize = 0x10;
const VARIANT_MATERIAL: usize = 0x1c;
const ANY_YEAR: u32 = u32::MAX;
/// The years of the figures SuperChargers reads, its own the last: a
/// variant the game takes in any year is named in each.
const YEARS: std::ops::RangeInclusive<u32> = 0..=4;
const UNRECORDED: u16 = 0x100;
const TOP: &str = "Body_";
const BOTTOM: &str = "Legs_";

/// One picture of a toy: the material it is drawn with, and the figures,
/// by id and variant, it shows.
#[derive(Debug, PartialEq)]
pub struct Picture {
    pub material: String,
    pub id: u16,
    pub variants: Vec<u16>,
}

/// A toy as its data gives it: its plain material and each variant's
/// numbers and material, where it has them.
struct Toy {
    id: u16,
    code_name: String,
    material: Option<String>,
    variants: Vec<(Vec<u16>, Option<String>)>,
}

/// The pictures of every toy in the game's files of toy data.
pub fn pictures<'a>(files: impl IntoIterator<Item = &'a [u8]>) -> Vec<Picture> {
    let toys: Vec<Toy> = files.into_iter().flat_map(toys).collect();
    let tops: HashMap<&str, &Toy> = toys.iter().filter_map(|toy| Some((toy.code_name.strip_prefix(TOP)?, toy))).collect();
    let mut pictures = Vec::new();
    for toy in &toys {
        let top = toy.code_name.strip_prefix(BOTTOM).and_then(|name| tops.get(name));
        if let Some(material) = toy.material.clone().or_else(|| top?.material.clone()) {
            pictures.push(Picture { material, id: toy.id, variants: vec![0] });
        }
        for (numbers, material) in &toy.variants {
            let theirs = || top?.variants.iter().find(|(their_numbers, _)| their_numbers == numbers)?.1.clone();
            if let Some(material) = material.clone().or_else(theirs) {
                pictures.push(Picture { material, id: toy.id, variants: numbers.clone() });
            }
        }
    }
    pictures
}

/// The toys in one file of toy data.
fn toys(bytes: &[u8]) -> Vec<Toy> {
    let Some(objects) = Objects::read(bytes) else {
        return Vec::new();
    };
    objects
        .of_kinds_ending(TOY)
        .filter_map(|toy| {
            let id = u16::try_from(objects.word(toy, ID)?).ok()?;
            let variants = objects.object(toy, VARIANTS).and_then(|list| objects.list(list)).unwrap_or_default();
            Some(Toy {
                id,
                code_name: objects.text(toy, CODE_NAME).unwrap_or_default().to_string(),
                material: objects.outside_name(toy, MATERIAL).map(str::to_string),
                variants: variants
                    .into_iter()
                    .filter_map(|variant| Some((numbers(&objects, variant)?, objects.outside_name(variant, VARIANT_MATERIAL).map(str::to_string))))
                    .collect(),
            })
        })
        .collect()
}

/// The variant numbers one of the game's variants stands for.
fn numbers(objects: &Objects, variant: usize) -> Option<Vec<u16>> {
    let deco = u16::try_from(objects.word(variant, DECO)?).ok().filter(|&deco| deco <= 0xff)?;
    let [light_core, alternate, posed, _] = objects.word(variant, FLAGS)?.to_be_bytes();
    let base = u16::from(posed != 0) << 11 | u16::from(alternate != 0) << 10 | u16::from(light_core != 0) << 9 | deco;
    let years: Vec<u32> = match objects.word(variant, YEAR)? {
        ANY_YEAR => YEARS.collect(),
        year if year <= 0xf => vec![year],
        _ => return None,
    };
    let numbers = years.into_iter().flat_map(|year| {
        let number = (year as u16) << 12 | base;
        [number, number | UNRECORDED]
    });
    Some(numbers.collect())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::igz::tests::{build_objects, object};

    const HANDLE: u32 = 0x8000_0000;
    /// A pointer to the start of the section after the objects.
    const FIRST_TEXT: u32 = 1 << 27;

    /// Toy data as SuperChargers keeps it: a toy, `id`, called `code_name`,
    /// drawn with `materials[0]`, with a variant drawn with each of the
    /// others, given as its deco, year and flags. An empty material is
    /// none.
    pub(crate) fn build_toy(kind: &str, id: u32, code_name: &str, materials: &[&str], variants: &[(u32, u32, u32)]) -> Vec<u8> {
        // The toy at 0, its list of variants at 0x40, which points to them
        // from 0x58, and the variants from 0x80, 0x40 apart.
        let handle = |index: usize| if materials[index].is_empty() { 0 } else { HANDLE | index as u32 };
        let mut list = vec![(0x08, variants.len() as u32), (0x14, 0x58)];
        list.extend((0..variants.len()).map(|index| (0x18 + index * 4, 0x80 + index as u32 * 0x40)));
        let toy = object(2, &[(ID, id), (VARIANTS, if variants.is_empty() { 0 } else { 0x40 }), (MATERIAL, handle(0)), (CODE_NAME, FIRST_TEXT)]);
        let mut objects = vec![toy, object(3, &list)];
        for (index, &(deco, year, flags)) in variants.iter().enumerate() {
            objects.push(object(1, &[(DECO, deco), (YEAR, year), (FLAGS, flags), (VARIANT_MATERIAL, handle(index + 1))]));
        }
        let outside: Vec<(u32, u32)> = (0..materials.len() as u32).map(|index| (index, index)).collect();
        let kinds = ["igObjectList", "CVariantIdentifier", kind, "CVariantIdentifierList"];
        build_objects(&kinds, materials, &outside, None, &objects, &[code_name.as_bytes(), b"\0"].concat(), None)
    }

    #[test]
    fn a_toy_and_its_variants_are_numbered_as_figures_carry_them() {
        // Hurricane Jet Vac and its Legendary deco: deco 3, the year of
        // SuperChargers, an alternate deco.
        let file = build_toy(
            "CFullCharacterToyData",
            3413,
            "DriverJetVac",
            &["Collection_DriverJetVac_Normal", "Collection_DriverJetVac_Legendary"],
            &[(3, 4, 0x0001_0000)],
        );
        assert_eq!(
            pictures([file.as_slice()]),
            [
                Picture { material: "Collection_DriverJetVac_Normal".to_string(), id: 3413, variants: vec![0x0000] },
                Picture { material: "Collection_DriverJetVac_Legendary".to_string(), id: 3413, variants: vec![0x4403, 0x4503] },
            ]
        );
    }

    #[test]
    fn every_flag_and_any_year_count() {
        // LightCore, alternate deco and new pose, and a variant taken in
        // any year, as Dive Bomber's Spring Ahead deco is.
        let file = build_toy(
            "CVehicleToyData",
            3231,
            "WaterTorpedoSub",
            &["Collection_A", "Collection_B", "Collection_C", "Collection_D"],
            &[(6, 1, 0x0100_0000), (5, 2, 0x0000_0100), (2, ANY_YEAR, 0x0001_0000)],
        );
        let read = pictures([file.as_slice()]);
        assert_eq!(read[1].variants, [0x1206, 0x1306]);
        assert_eq!(read[2].variants, [0x2805, 0x2905]);
        assert_eq!(read[3].variants, [0x0402, 0x0502, 0x1402, 0x1502, 0x2402, 0x2502, 0x3402, 0x3502, 0x4402, 0x4502]);
    }

    #[test]
    fn a_swappers_bottom_is_shown_by_its_top() {
        let top = build_toy("CShapeshifterPartToyData", 2013, "Body_Cowboy", &["Collection_Body_Cowboy_Normal", "Collection_Body_Cowboy_AltDeco"], &[(2, 2, 0x0001_0000)]);
        let bottom = build_toy("CShapeshifterPartToyData", 1013, "Legs_Cowboy", &["", ""], &[(2, 2, 0x0001_0000)]);
        let read = pictures([bottom.as_slice(), top.as_slice()]);
        let bottom: Vec<(&str, &[u16])> = read.iter().filter(|picture| picture.id == 1013).map(|picture| (picture.material.as_str(), picture.variants.as_slice())).collect();
        assert_eq!(bottom, [("Collection_Body_Cowboy_Normal", &[0x0000][..]), ("Collection_Body_Cowboy_AltDeco", &[0x2402, 0x2502][..])]);
    }

    #[test]
    fn a_toy_without_a_picture_gives_none() {
        // As UFO Hat, whose toy data names no material.
        let file = build_toy("CMagicItemToyData", 3204, "MagicItem_HatPromo", &[""], &[]);
        assert_eq!(pictures([file.as_slice()]), []);
        assert_eq!(pictures([b"IGZ\x01 not toy data".as_slice()]), []);
    }
}
