//! The pixel formats a Trap Team picture may come in besides the DXT5 every
//! other game's pictures here are read as. The game names a format by the
//! FNV-1a hash of its name (see `survey`), and the engine names its formats
//! after a plain one: the plain name, `_`, then the platform, or `_tile_`
//! then the platform for a texture the console keeps in its own order. The
//! platforms are `big_ps3` and `cafe`, the Wii U. So igRewrite8 builds them
//! (github.com/NefariousTechSupport/igRewrite8, Apache 2.0, commit 2dd2f1a,
//! read 14 October 2026), in igLibrary/Gfx/igGfx.cs, lines 587, 597, 727
//! and 733, from the plain names in lines 157 to 230.
//!
//! How each is laid out:
//!
//! - DXT1, DXT3 and DXT5, `_srgb` or not (the same bytes, meant as sRGB,
//!   which a PNG is anyway), on the PS3, tiled or not, are blocks in rows, as
//!   on a PC. RPCS3 (github.com/RPCS3/rpcs3, GPL, read and not copied, commit
//!   6d7020d, rpcs3/Emu/RSX/Common/TextureUtils.cpp, lines 1065 to 1101 and
//!   1126 to 1141) copies a flat compressed texture as it is, saying "on PS3
//!   we only have a linear one", and Giants' DXT5 pictures here bear it out.
//! - The same on the Wii U are blocks in rows when not tiled and in the chip's
//!   tiles when tiled (`gx2`). igRewrite8 untiles only `_tile_cafe`
//!   (igLibrary/Gfx/igMetaImage.cs, lines 211 to 216), and STTUMM, a mod
//!   manager for Trap Team on the Wii U (github.com/GlitchedDeveloper/STTUMM,
//!   GPL, read and not copied, commit 7cdef68, IGAE/TextureHelper.cs, lines
//!   47 to 49 and 178), untiles DXT1 there as the chip's BC1.
//! - `r8g8b8a8_tile_cafe`: a pixel's red, green, blue and alpha bytes in that
//!   order, in the chip's tiles of 32-bit elements, and `r8g8b8a8_cafe` the
//!   same in rows. `b8g8r8a8_big_ps3`: alpha, red, green, blue, in rows.
//!   igRewrite8 writes them so (igCauldron3/Conversion/TextureConversion.cs,
//!   lines 149 to 153); it leaves the PS3's tiled kind alone, so it is not
//!   read here.
//!
//! Which of these Trap Team's figures and traps use is not known from the
//! game's own files yet (14 October 2026); `trap_team` says which formats it
//! passes over, so the next run on the game shows it.

use crate::survey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pixels {
    Dxt1,
    Dxt3,
    Dxt5,
    /// Four bytes a pixel: red, green, blue, alpha.
    Rgba,
    /// Four bytes a pixel: alpha, red, green, blue.
    Argb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Format {
    pub pixels: Pixels,
    /// Kept in the Wii U chip's tiles rather than in rows.
    pub tiled: bool,
}

impl Pixels {
    /// How many pixels across and down one element covers: a block of 4 x 4
    /// for DXT, one pixel otherwise.
    pub fn side(self) -> usize {
        match self {
            Pixels::Dxt1 | Pixels::Dxt3 | Pixels::Dxt5 => 4,
            Pixels::Rgba | Pixels::Argb => 1,
        }
    }

    /// How many bytes one element takes.
    pub fn bytes(self) -> usize {
        match self {
            Pixels::Dxt1 => 8,
            Pixels::Dxt3 | Pixels::Dxt5 => 16,
            Pixels::Rgba | Pixels::Argb => 4,
        }
    }
}

/// Each format read here, by name.
fn known() -> Vec<(String, Format)> {
    let mut known = Vec::new();
    let mut add = |name: &str, pixels: Pixels, endings: &[(&str, bool)]| {
        for &(ending, tiled) in endings {
            known.push((format!("{name}{ending}"), Format { pixels, tiled }));
        }
    };
    let blocks = [("_big_ps3", false), ("_tile_big_ps3", false), ("_cafe", false), ("_tile_cafe", true)];
    for (name, pixels) in [("dxt1", Pixels::Dxt1), ("dxt3", Pixels::Dxt3), ("dxt5", Pixels::Dxt5)] {
        add(name, pixels, &blocks);
        add(&format!("{name}_srgb"), pixels, &blocks);
    }
    for name in ["r8g8b8a8", "r8g8b8a8_srgb"] {
        add(name, Pixels::Rgba, &[("_cafe", false), ("_tile_cafe", true)]);
    }
    for name in ["b8g8r8a8", "b8g8r8a8_srgb"] {
        add(name, Pixels::Argb, &[("_big_ps3", false)]);
    }
    known
}

/// The format a picture's format hash names, when it is read here.
pub fn of(hash: u32) -> Option<Format> {
    known().into_iter().find(|(name, _)| survey::fnv1a(name) == hash).map(|(_, format)| format)
}

/// A format for a message: its name, when it is one known here or to
/// `survey`, and its number.
pub fn describe(hash: u32) -> String {
    let name = match known().into_iter().find(|(name, _)| survey::fnv1a(name) == hash) {
        Some((name, _)) => name,
        None => survey::format_name(hash),
    };
    if name.starts_with("format ") {
        format!("0x{hash:08x}")
    } else {
        format!("{name}, 0x{hash:08x}")
    }
}

/// Four-byte pixels as RGBA.
pub fn to_rgba(pixels: Pixels, bytes: &[u8]) -> Vec<u8> {
    match pixels {
        Pixels::Argb => bytes.chunks_exact(4).flat_map(|pixel| [pixel[1], pixel[2], pixel[3], pixel[0]]).collect(),
        _ => bytes.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_are_told_by_the_hash_of_their_names() {
        let hash = survey::fnv1a;
        assert_eq!(of(hash("dxt1_tile_cafe")), Some(Format { pixels: Pixels::Dxt1, tiled: true }));
        assert_eq!(of(hash("dxt1_cafe")), Some(Format { pixels: Pixels::Dxt1, tiled: false }));
        assert_eq!(of(hash("dxt1_tile_big_ps3")), Some(Format { pixels: Pixels::Dxt1, tiled: false }));
        assert_eq!(of(hash("dxt3_srgb_big_ps3")), Some(Format { pixels: Pixels::Dxt3, tiled: false }));
        assert_eq!(of(hash("r8g8b8a8_tile_cafe")), Some(Format { pixels: Pixels::Rgba, tiled: true }));
        assert_eq!(of(hash("b8g8r8a8_big_ps3")), Some(Format { pixels: Pixels::Argb, tiled: false }));
        // The PS3's own tiles of whole pixels aren't read here.
        assert_eq!(of(hash("b8g8r8a8_tile_big_ps3")), None);
        assert_eq!(of(hash("dxt1_tile_big_wii")), None);
        // The DXT5 the other games' pictures are read as is among them.
        assert_eq!(of(crate::DXT5_TILED), Some(Format { pixels: Pixels::Dxt5, tiled: true }));
        assert!(crate::DXT5_PS3.iter().all(|&format| of(format) == Some(Format { pixels: Pixels::Dxt5, tiled: false })));
    }

    #[test]
    fn a_format_is_named_for_a_message() {
        let named = |name: &str| describe(survey::fnv1a(name));
        assert_eq!(named("dxt1_srgb_tile_cafe"), format!("dxt1_srgb_tile_cafe, 0x{:08x}", survey::fnv1a("dxt1_srgb_tile_cafe")));
        assert!(named("a8_tile_cafe").starts_with("a8_tile_cafe, 0x"));
        assert_eq!(describe(0x1234_5678), "0x12345678");
    }

    #[test]
    fn whole_pixels_come_out_as_rgba() {
        assert_eq!(to_rgba(Pixels::Rgba, &[1, 2, 3, 4]), [1, 2, 3, 4]);
        assert_eq!(to_rgba(Pixels::Argb, &[4, 1, 2, 3, 40, 10, 20, 30]), [1, 2, 3, 4, 10, 20, 30, 40]);
    }
}
