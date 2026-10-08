//! The Wii's own compressed pixel format, CMPR, which Skylanders Spyro's
//! Adventure keeps its pictures in ("dxt1_tile_big_wii", as the game names
//! it). It is DXT1 laid out for the Wii's graphics chip: blocks of 4 x 4
//! pixels, four to a tile of 8 x 8 (top left, top right, bottom left, bottom
//! right), the tiles in rows, and a picture kept as whole tiles. Each block
//! is two RGB565 colours, big-endian, then a byte for each row of pixels,
//! the leftmost pixel in its top two bits. When the first colour is the
//! larger number the other two lie a third and two thirds of the way from it
//! to the second; otherwise the third lies half way and the fourth is clear.
//! Written from the format as it is widely described, and checked by eye
//! against the game's own pictures, 8 October 2026.

const TILE: usize = 8;
const BLOCK: usize = 4;
const BLOCK_BYTES: usize = 8;
const BLOCKS_IN_TILE: usize = (TILE / BLOCK) * (TILE / BLOCK);

/// How many bytes a CMPR picture `width` x `height` takes: whole tiles.
pub fn size(width: usize, height: usize) -> usize {
    width.div_ceil(TILE) * height.div_ceil(TILE) * BLOCKS_IN_TILE * BLOCK_BYTES
}

fn expand(value: u16, bits: u32) -> u8 {
    let max = (1u32 << bits) - 1;
    ((u32::from(value) * 255 + max / 2) / max) as u8
}

fn rgb565(colour: u16) -> [u8; 3] {
    [expand(colour >> 11, 5), expand(colour >> 5 & 63, 6), expand(colour & 31, 5)]
}

/// The four colours a block's pixels choose from, as RGBA.
fn palette(block: &[u8]) -> [[u8; 4]; 4] {
    let (first, second) = (u16::from_be_bytes([block[0], block[1]]), u16::from_be_bytes([block[2], block[3]]));
    let (a, b) = (rgb565(first), rgb565(second));
    let mix = |towards: u16, of: u16| {
        let [r, g, b] = [0, 1, 2].map(|c| ((u16::from(a[c]) * (of - towards) + u16::from(b[c]) * towards) / of) as u8);
        [r, g, b, 255]
    };
    let solid = |[r, g, b]: [u8; 3]| [r, g, b, 255];
    if first > second {
        [solid(a), solid(b), mix(1, 3), mix(2, 3)]
    } else {
        [solid(a), solid(b), mix(1, 2), [0; 4]]
    }
}

/// A CMPR picture `width` x `height` as RGBA pixels, row after row in the
/// order they are kept. `None` when there are fewer bytes than its tiles
/// take.
pub fn decode(pixels: &[u8], width: usize, height: usize) -> Option<Vec<u8>> {
    let tiles_wide = width.div_ceil(TILE);
    let blocks = pixels.get(..size(width, height))?;
    let mut out = vec![0; width * height * 4];
    for (index, block) in blocks.chunks_exact(BLOCK_BYTES).enumerate() {
        let (tile, quarter) = (index / BLOCKS_IN_TILE, index % BLOCKS_IN_TILE);
        let left = tile % tiles_wide * TILE + quarter % 2 * BLOCK;
        let top = tile / tiles_wide * TILE + quarter / 2 * BLOCK;
        let colours = palette(block);
        for y in 0..BLOCK {
            for x in 0..BLOCK {
                let (column, row) = (left + x, top + y);
                if column >= width || row >= height {
                    continue;
                }
                let choice = block[BLOCK + y] >> (6 - 2 * x) & 3;
                let at = (row * width + column) * 4;
                out[at..at + 4].copy_from_slice(&colours[usize::from(choice)]);
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pixels made of RGB565 colours, two to a block, written as CMPR the
    /// way the game's are: each block its larger colour first, so its four
    /// colours are solid, and each pixel choosing one of the two.
    fn encode(rgb565s: &[u16], width: usize, height: usize) -> Vec<u8> {
        let mut out = Vec::new();
        for tile_top in (0..height).step_by(TILE) {
            for tile_left in (0..width).step_by(TILE) {
                for quarter in 0..BLOCKS_IN_TILE {
                    let (left, top) = (tile_left + quarter % 2 * BLOCK, tile_top + quarter / 2 * BLOCK);
                    let at = |x: usize, y: usize| rgb565s[(top + y) * width + left + x];
                    let mut both = vec![at(0, 0)];
                    for y in 0..BLOCK {
                        for x in 0..BLOCK {
                            if !both.contains(&at(x, y)) {
                                both.push(at(x, y));
                            }
                        }
                    }
                    assert!(both.len() <= 2, "a block of the test has more than two colours");
                    let (high, low) = (both.iter().copied().max().unwrap(), both.iter().copied().min().unwrap());
                    // Equal colours would make the block's fourth colour clear.
                    let low = if high == low { low.wrapping_sub(1) } else { low };
                    out.extend_from_slice(&high.to_be_bytes());
                    out.extend_from_slice(&low.to_be_bytes());
                    for y in 0..BLOCK {
                        let row = (0..BLOCK).fold(0u8, |row, x| row | u8::from(at(x, y) != high) << (6 - 2 * x));
                        out.push(row);
                    }
                }
            }
        }
        out
    }

    fn rgba(colour: u16) -> [u8; 4] {
        let [r, g, b] = rgb565(colour);
        [r, g, b, 255]
    }

    #[test]
    fn a_picture_comes_back_as_it_was_written() {
        // 16 x 16, two tiles across and two down, with a different pair of
        // colours in every block and the pixels choosing between them.
        let (width, height) = (16, 16);
        let colours: Vec<u16> = (0..width * height)
            .map(|at| {
                let (x, y) = (at % width, at / width);
                let block = (y / BLOCK * (width / BLOCK) + x / BLOCK) as u16;
                if (x * 3 + y * 5) % 7 < 3 { 0xf000 | block * 97 } else { 0x0100 + block * 13 }
            })
            .collect();
        let pixels = encode(&colours, width, height);
        assert_eq!(pixels.len(), size(width, height));
        let decoded = decode(&pixels, width, height).unwrap();
        let wanted: Vec<u8> = colours.iter().flat_map(|&colour| rgba(colour)).collect();
        assert_eq!(decoded, wanted);
    }

    #[test]
    fn blocks_fill_their_tile_in_quarters_and_tiles_go_in_rows() {
        // Two tiles side by side, each of its eight blocks one colour.
        let mut pixels = Vec::new();
        for block in 0..8u16 {
            pixels.extend_from_slice(&(0x1000 * (block + 1)).to_be_bytes());
            pixels.extend_from_slice(&0u16.to_be_bytes());
            pixels.extend_from_slice(&[0; 4]);
        }
        let decoded = decode(&pixels, 16, 8).unwrap();
        let colour = |x: usize, y: usize| &decoded[(y * 16 + x) * 4..(y * 16 + x) * 4 + 4];
        let block = |n: u16| rgba(0x1000 * (n + 1));
        assert_eq!(colour(0, 0), block(0));
        assert_eq!(colour(7, 0), block(1));
        assert_eq!(colour(0, 7), block(2));
        assert_eq!(colour(7, 7), block(3));
        assert_eq!(colour(8, 0), block(4));
        assert_eq!(colour(15, 7), block(7));
    }

    #[test]
    fn a_rows_leftmost_pixel_is_in_its_top_bits() {
        let mut block = [0xff, 0xff, 0x00, 0x00, 0b00_01_10_11, 0, 0, 0].to_vec();
        block.extend_from_slice(&[0; 24]);
        let decoded = decode(&block, 8, 8).unwrap();
        let firsts: Vec<&[u8]> = decoded[..16].chunks(4).collect();
        assert_eq!(firsts, [&[255, 255, 255, 255][..], &[0, 0, 0, 255], &[170, 170, 170, 255], &[85, 85, 85, 255]]);
    }

    #[test]
    fn a_block_whose_first_colour_is_not_larger_has_a_clear_pixel() {
        let mut block = [0x00, 0x00, 0xff, 0xff, 0b10_11_00_00, 0, 0, 0].to_vec();
        block.extend_from_slice(&[0; 24]);
        let decoded = decode(&block, 8, 8).unwrap();
        assert_eq!(&decoded[..8], [127, 127, 127, 255, 0, 0, 0, 0]);
    }

    #[test]
    fn a_picture_is_kept_as_whole_tiles() {
        // 10 x 3 takes two tiles; what lies past its edge is cut off.
        assert_eq!(size(10, 3), 2 * 32);
        let pixels = encode(&[0xffff; 16 * 8], 16, 8);
        assert_eq!(decode(&pixels, 10, 3).unwrap(), [255; 10 * 3 * 4].to_vec());
        assert!(decode(&pixels[..40], 10, 3).is_none());
    }
}
