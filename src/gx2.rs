//! The Wii U's graphics chip keeps a texture in tiles, not row by row. This
//! puts the blocks of a DXT5 picture (128 bits each) back in rows. The chip
//! belongs to AMD's R600 family, set up with 2 pipes, 4 banks and a 256-byte
//! pipe interleave. A texture at least one macro tile big (32 x 16 blocks) is
//! 2D tiled, a smaller one 1D tiled. Written from the family's addressing
//! rules and checked by eye against the game's own pictures.

const BLOCK_BYTES: usize = 16;
const MICRO: usize = 8;
const PIPES: usize = 2;
const BANKS: usize = 4;
const PIPE_BITS: usize = 1;
const BANK_BITS: usize = 2;
const INTERLEAVE_BITS: usize = 8;
const MACRO_WIDE: usize = MICRO * BANKS;
const MACRO_HIGH: usize = MICRO * PIPES;

/// Where a block sits among the 64 of its 8 x 8 micro tile.
fn micro_index(x: usize, y: usize) -> usize {
    (y & 1) | (x & 1) << 1 | (x >> 1 & 1) << 2 | (x >> 2 & 1) << 3 | (y >> 1 & 1) << 4 | (y >> 2 & 1) << 5
}

/// The byte where block (x, y) is kept in a 2D tiled surface `pitch` blocks
/// wide. Each micro tile belongs to one pipe and bank; a macro tile's bytes
/// are spread over all of them, 256 at a time.
fn macro_tiled(x: usize, y: usize, pitch: usize) -> usize {
    let pipe = ((y >> 3) ^ (x >> 3)) & 1;
    let bank = ((y / (16 * PIPES)) ^ (x >> 3)) & 1 | (((y / (8 * PIPES)) ^ (x >> 4)) & 1) << 1;
    let macro_bytes = MACRO_WIDE * MACRO_HIGH * BLOCK_BYTES;
    let macro_offset = (x / MACRO_WIDE + pitch / MACRO_WIDE * (y / MACRO_HIGH)) * macro_bytes;
    let total = micro_index(x, y) * BLOCK_BYTES + (macro_offset >> (PIPE_BITS + BANK_BITS));
    let interleave = (1 << INTERLEAVE_BITS) - 1;
    (total & !interleave) << (PIPE_BITS + BANK_BITS)
        | bank << (INTERLEAVE_BITS + PIPE_BITS)
        | pipe << INTERLEAVE_BITS
        | total & interleave
}

/// The byte where block (x, y) is kept in a 1D tiled surface: whole micro
/// tiles, row after row.
fn micro_tiled(x: usize, y: usize, pitch: usize) -> usize {
    ((y / MICRO) * (pitch / MICRO) + x / MICRO) * MICRO * MICRO * BLOCK_BYTES + micro_index(x, y) * BLOCK_BYTES
}

/// The blocks of a tiled surface `wide` x `high` blocks, in rows. `None` when
/// the data is too short for that size.
pub fn untile(tiled: &[u8], wide: usize, high: usize) -> Option<Vec<u8>> {
    let two_d = wide >= MACRO_WIDE && high >= MACRO_HIGH;
    let pitch = wide.next_multiple_of(if two_d { MACRO_WIDE } else { MICRO });
    let mut rows = vec![0; wide * high * BLOCK_BYTES];
    for y in 0..high {
        for x in 0..wide {
            let from = if two_d { macro_tiled(x, y, pitch) } else { micro_tiled(x, y, pitch) };
            let to = (y * wide + x) * BLOCK_BYTES;
            rows[to..to + BLOCK_BYTES].copy_from_slice(tiled.get(from..from + BLOCK_BYTES)?);
        }
    }
    Some(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_block_somewhere_else(address: impl Fn(usize, usize) -> usize, wide: usize, high: usize) {
        let mut taken = vec![false; wide * high];
        for y in 0..high {
            for x in 0..wide {
                let at = address(x, y);
                assert_eq!(at % BLOCK_BYTES, 0);
                assert!(at / BLOCK_BYTES < taken.len(), "({x}, {y}) lands outside the surface");
                assert!(!taken[at / BLOCK_BYTES], "({x}, {y}) lands on another block");
                taken[at / BLOCK_BYTES] = true;
            }
        }
    }

    #[test]
    fn each_block_has_its_own_place() {
        every_block_somewhere_else(|x, y| macro_tiled(x, y, 64), 64, 64);
        every_block_somewhere_else(|x, y| macro_tiled(x, y, 128), 128, 128);
        every_block_somewhere_else(|x, y| micro_tiled(x, y, 16), 16, 16);
    }

    #[test]
    fn blocks_land_where_the_game_put_them() {
        // Worked out by hand from the rules, and matching Spyro's portrait.
        assert_eq!(macro_tiled(0, 0, 64), 0);
        assert_eq!(macro_tiled(0, 1, 64), 16);
        assert_eq!(macro_tiled(1, 0, 64), 32);
        assert_eq!(macro_tiled(0, 8, 64), 256);
        assert_eq!(macro_tiled(8, 0, 64), 768);
        assert_eq!(micro_tiled(8, 0, 16), 1024);
    }

    #[test]
    fn too_little_data_is_refused() {
        assert!(untile(&[0; 100], 64, 64).is_none());
        assert_eq!(untile(&vec![0; 64 * 64 * 16], 64, 64).map(|rows| rows.len()), Some(64 * 64 * 16));
    }
}
