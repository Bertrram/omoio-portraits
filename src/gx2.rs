//! The Wii U's graphics chip keeps a texture in tiles, not row by row. This
//! puts the blocks of a DXT5 picture (128 bits each) back in rows. The chip
//! belongs to AMD's R600 family, set up with 2 pipes, 4 banks and a 256-byte
//! pipe interleave. A texture is 2D tiled, padded out to whole macro tiles
//! (32 x 16 blocks), or 1D tiled, padded to whole micro tiles (8 x 8); the
//! game's own pictures show which by their size, a 120-pixel icon being 2D
//! and a 160-pixel one 1D. Written from the family's addressing rules and
//! checked by eye against the game's own pictures.
//!
//! The same rules put other elements back in rows: a DXT1 or DXT3 block (64
//! and 128 bits) or a pixel of 32 bits (`untile_elements`). Only where an
//! element sits among the 64 of its micro tile depends on its size; the
//! pipes, banks and padding are the same for all three. Read in AMD's own
//! address library for the R600 family as decaf-emu keeps it for the Wii U
//! (github.com/decaf-emu/addrlib, MIT, commit 194162c, read 14 October
//! 2026): `ComputePixelIndexWithinMicroTile` in src/core/addrlib.cpp, lines
//! 643 to 760, its cases for each size from line 691; the element
//! sizes of BC1 to BC3 in src/core/addrelemlib.cpp, lines 246 to 269; the
//! padding in src/r600/r600addrlib.cpp, lines 714 to 870, which comes to 8
//! and 32 elements across for all three sizes; and the addresses in the
//! same file, lines 1407 to 1450 and 1582 on. DXT5 is the only size checked
//! against the game's own pictures so far (14 October 2026).

const BLOCK_BYTES: usize = 16;
const MICRO: usize = 8;
const PIPES: usize = 2;
const BANKS: usize = 4;
const PIPE_BITS: usize = 1;
const BANK_BITS: usize = 2;
const INTERLEAVE_BITS: usize = 8;
const MACRO_WIDE: usize = MICRO * BANKS;
const MACRO_HIGH: usize = MICRO * PIPES;

/// Where an element of `bytes` sits among the 64 of its 8 x 8 micro tile.
/// A DXT5 block, 16 bytes, is the case the game's pictures first showed.
fn micro_index(x: usize, y: usize, bytes: usize) -> usize {
    let bit = |value: usize, at: usize| value >> at & 1;
    let order = match bytes {
        16 => [bit(y, 0), bit(x, 0), bit(x, 1), bit(x, 2), bit(y, 1), bit(y, 2)],
        8 => [bit(x, 0), bit(y, 0), bit(x, 1), bit(x, 2), bit(y, 1), bit(y, 2)],
        _ => [bit(x, 0), bit(x, 1), bit(y, 0), bit(x, 2), bit(y, 1), bit(y, 2)],
    };
    order.iter().enumerate().fold(0, |index, (at, &value)| index | value << at)
}

/// The byte where element (x, y), of `bytes`, is kept in a 2D tiled surface
/// `pitch` elements wide. Each micro tile belongs to one pipe and bank; a
/// macro tile's bytes are spread over all of them, 256 at a time.
fn macro_tiled(x: usize, y: usize, pitch: usize, bytes: usize) -> usize {
    let pipe = ((y >> 3) ^ (x >> 3)) & 1;
    let bank = ((y / (16 * PIPES)) ^ (x >> 3)) & 1 | (((y / (8 * PIPES)) ^ (x >> 4)) & 1) << 1;
    let macro_bytes = MACRO_WIDE * MACRO_HIGH * bytes;
    let macro_offset = (x / MACRO_WIDE + pitch / MACRO_WIDE * (y / MACRO_HIGH)) * macro_bytes;
    let total = micro_index(x, y, bytes) * bytes + (macro_offset >> (PIPE_BITS + BANK_BITS));
    let interleave = (1 << INTERLEAVE_BITS) - 1;
    (total & !interleave) << (PIPE_BITS + BANK_BITS)
        | bank << (INTERLEAVE_BITS + PIPE_BITS)
        | pipe << INTERLEAVE_BITS
        | total & interleave
}

/// The byte where element (x, y), of `bytes`, is kept in a 1D tiled surface:
/// whole micro tiles, row after row.
fn micro_tiled(x: usize, y: usize, pitch: usize, bytes: usize) -> usize {
    ((y / MICRO) * (pitch / MICRO) + x / MICRO) * MICRO * MICRO * bytes + micro_index(x, y, bytes) * bytes
}

/// The blocks of a tiled DXT5 surface `wide` x `high` blocks, in rows.
/// `None` when the data is too short for that size.
pub fn untile(tiled: &[u8], wide: usize, high: usize) -> Option<Vec<u8>> {
    untile_elements(tiled, wide, high, BLOCK_BYTES)
}

/// The elements of a tiled surface `wide` x `high` elements of `bytes`
/// each (8 for a DXT1 block, 16 for a DXT3 or DXT5 one, 4 for a pixel of
/// 32 bits), in rows. `None` when the data is too short for that size, or
/// for an element of another size, which the chip lays out otherwise.
pub fn untile_elements(tiled: &[u8], wide: usize, high: usize, bytes: usize) -> Option<Vec<u8>> {
    if ![4, 8, 16].contains(&bytes) {
        return None;
    }
    let padded_2d = wide.next_multiple_of(MACRO_WIDE) * high.next_multiple_of(MACRO_HIGH) * bytes;
    let two_d = tiled.len() >= padded_2d;
    let pitch = wide.next_multiple_of(if two_d { MACRO_WIDE } else { MICRO });
    let mut rows = vec![0; wide * high * bytes];
    for y in 0..high {
        for x in 0..wide {
            let from = if two_d { macro_tiled(x, y, pitch, bytes) } else { micro_tiled(x, y, pitch, bytes) };
            let to = (y * wide + x) * bytes;
            rows[to..to + bytes].copy_from_slice(tiled.get(from..from + bytes)?);
        }
    }
    Some(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_block_somewhere_else(address: impl Fn(usize, usize) -> usize, wide: usize, high: usize, bytes: usize) {
        let mut taken = vec![false; wide * high];
        for y in 0..high {
            for x in 0..wide {
                let at = address(x, y);
                assert_eq!(at % bytes, 0);
                assert!(at / bytes < taken.len(), "({x}, {y}) lands outside the surface");
                assert!(!taken[at / bytes], "({x}, {y}) lands on another block");
                taken[at / bytes] = true;
            }
        }
    }

    #[test]
    fn each_block_has_its_own_place() {
        for bytes in [4, 8, 16] {
            every_block_somewhere_else(|x, y| macro_tiled(x, y, 64, bytes), 64, 64, bytes);
            every_block_somewhere_else(|x, y| macro_tiled(x, y, 128, bytes), 128, 128, bytes);
            every_block_somewhere_else(|x, y| micro_tiled(x, y, 16, bytes), 16, 16, bytes);
        }
    }

    #[test]
    fn blocks_land_where_the_game_put_them() {
        // Worked out by hand from the rules, and matching Spyro's portrait.
        assert_eq!(macro_tiled(0, 0, 64, 16), 0);
        assert_eq!(macro_tiled(0, 1, 64, 16), 16);
        assert_eq!(macro_tiled(1, 0, 64, 16), 32);
        assert_eq!(macro_tiled(0, 8, 64, 16), 256);
        assert_eq!(macro_tiled(8, 0, 64, 16), 768);
        assert_eq!(micro_tiled(8, 0, 16, 16), 1024);
    }

    #[test]
    fn where_an_element_sits_in_its_micro_tile_depends_on_its_size() {
        // The first bits of the index, from AMD's ComputePixelIndexWithinMicroTile:
        // 128 bits take y0 first, 64 bits x0 then y0, 32 bits x0, x1, then y0.
        assert_eq!([micro_index(1, 0, 16), micro_index(0, 1, 16), micro_index(2, 0, 16)], [2, 1, 4]);
        assert_eq!([micro_index(1, 0, 8), micro_index(0, 1, 8), micro_index(2, 0, 8)], [1, 2, 4]);
        assert_eq!([micro_index(1, 0, 4), micro_index(0, 1, 4), micro_index(2, 0, 4)], [1, 4, 2]);
        // The rest is the same for every size: x2, then y1 and y2.
        for bytes in [4, 8, 16] {
            assert_eq!([micro_index(4, 0, bytes), micro_index(0, 2, bytes), micro_index(0, 4, bytes)], [8, 16, 32]);
        }
        // So DXT1's second block across is 8 bytes on, and the one below 16.
        assert_eq!([macro_tiled(1, 0, 64, 8), macro_tiled(0, 1, 64, 8)], [8, 16]);
        // Pipes and banks don't depend on the size.
        assert_eq!([macro_tiled(0, 8, 64, 8), macro_tiled(8, 0, 64, 8)], [256, 768]);
        assert_eq!(micro_tiled(8, 0, 16, 8), 512);
    }

    #[test]
    fn a_dxt1_surface_comes_back_in_rows() {
        // Each block marked with its own number, put where the rules keep it.
        let (wide, high): (usize, usize) = (40, 20);
        for (pitch, two_d) in [(wide.next_multiple_of(32), true), (wide.next_multiple_of(8), false)] {
            let size = if two_d { 64 * 32 * 8 } else { pitch * high.next_multiple_of(8) * 8 };
            let mut tiled = vec![0u8; size];
            for y in 0..high {
                for x in 0..wide {
                    let at = if two_d { macro_tiled(x, y, pitch, 8) } else { micro_tiled(x, y, pitch, 8) };
                    tiled[at..at + 8].copy_from_slice(&((y * wide + x) as u64).to_le_bytes());
                }
            }
            let rows = untile_elements(&tiled, wide, high, 8).unwrap();
            let numbers: Vec<u64> = rows.chunks(8).map(|block| u64::from_le_bytes(block.try_into().unwrap())).collect();
            assert_eq!(numbers, (0..(wide * high) as u64).collect::<Vec<_>>(), "2D: {two_d}");
        }
        assert!(untile_elements(&[0; 64 * 64 * 2], 8, 8, 2).is_none());
    }

    #[test]
    fn too_little_data_is_refused() {
        assert!(untile(&[0; 100], 64, 64).is_none());
        assert_eq!(untile(&vec![0; 64 * 64 * 16], 64, 64).map(|rows| rows.len()), Some(64 * 64 * 16));
    }

    #[test]
    fn the_size_of_the_data_tells_the_layout() {
        // A 120-pixel icon fills whole macro tiles once padded: 2D. A
        // 160-pixel one is only big enough for micro tiles: 1D. The block at
        // (8, 0) is kept at a different byte in each layout, so marking it
        // there shows which one was read.
        let marked = |at: usize, size: usize| {
            let mut data = vec![0u8; size];
            data[at..at + BLOCK_BYTES].fill(7);
            data
        };
        assert_ne!(macro_tiled(8, 0, 32, 16), micro_tiled(8, 0, 32, 16));
        let two_d = untile(&marked(macro_tiled(8, 0, 32, 16), 32 * 32 * 16), 30, 30).unwrap();
        assert_eq!(two_d[8 * BLOCK_BYTES], 7);
        let one_d = untile(&marked(micro_tiled(8, 0, 40, 16), 40 * 40 * 16), 40, 40).unwrap();
        assert_eq!(one_d[8 * BLOCK_BYTES], 7);
    }
}
