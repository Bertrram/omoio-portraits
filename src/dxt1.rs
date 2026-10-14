//! DXT1 (BC1) and DXT3 (BC2) pixels, for Trap Team's pictures that aren't
//! DXT5. A DXT1 block is 8 bytes for 4 x 4 pixels: two RGB565 ends and a
//! 2-bit choice per pixel. When the first end is the larger number the other
//! two colours lie a third and two thirds of the way from it to the second;
//! otherwise the third lies half way and the fourth is black and clear. A
//! DXT3 block is 16 bytes: a 4-bit alpha per pixel, then a DXT1 block whose
//! four colours are always the opaque ones, whichever end is larger. The
//! numbers are little-endian, the first pixel in the lowest bits.
//!
//! Written from the S3TC specification, EXT_texture_compression_s3tc
//! (github.com/KhronosGroup/OpenGL-Registry, extensions/EXT/, commit
//! 6af574a, read 14 October 2026): DXT1 in lines 675 to 732, DXT3 in lines
//! 743 to 772, and issue 6, lines 128 to 135, for DXT3's colours. Ends are
//! widened to 8 bits and blended as `dxt5` does, so the two agree.

const DXT1_BYTES: usize = 8;
const DXT3_BYTES: usize = 16;

fn expand(value: u16, bits: u32) -> u32 {
    let max = (1u32 << bits) - 1;
    (u32::from(value) * 255 + max / 2) / max
}

fn rgb565(colour: u16) -> [u32; 3] {
    [expand(colour >> 11, 5), expand(colour >> 5 & 63, 6), expand(colour & 31, 5)]
}

/// The four RGBA colours a block's pixels choose from. `opaque` keeps the
/// four opaque ones whichever end is larger, as DXT3 does.
fn palette(block: &[u8], opaque: bool) -> [[u8; 4]; 4] {
    let (first, second) = (u16::from_le_bytes([block[0], block[1]]), u16::from_le_bytes([block[2], block[3]]));
    let (a, b) = (rgb565(first), rgb565(second));
    let solid = |[r, g, b]: [u32; 3]| [r as u8, g as u8, b as u8, 255];
    if opaque || first > second {
        [solid(a), solid(b), solid([0, 1, 2].map(|c| (2 * a[c] + b[c]) / 3)), solid([0, 1, 2].map(|c| (a[c] + 2 * b[c]) / 3))]
    } else {
        [solid(a), solid(b), solid([0, 1, 2].map(|c| (a[c] + b[c]) / 2)), [0; 4]]
    }
}

/// Blocks of `bytes` in rows, as RGBA pixels in rows, each block's pixels
/// given by `pixel(block, index)`.
fn decode_with(blocks: &[u8], width: usize, height: usize, bytes: usize, pixel: impl Fn(&[u8], usize) -> [u8; 4]) -> Vec<u8> {
    let mut pixels = vec![0; width * height * 4];
    let blocks_wide = width / 4;
    for (index, block) in blocks.chunks_exact(bytes).take(blocks_wide * (height / 4)).enumerate() {
        let (left, top) = (index % blocks_wide * 4, index / blocks_wide * 4);
        for at in 0..16 {
            let (x, y) = (left + at % 4, top + at / 4);
            let to = (y * width + x) * 4;
            pixels[to..to + 4].copy_from_slice(&pixel(block, at));
        }
    }
    pixels
}

/// A pixel's 2-bit choice of colour in an 8-byte colour block.
fn choice(block: &[u8], at: usize) -> usize {
    (u32::from_le_bytes([block[4], block[5], block[6], block[7]]) >> (2 * at) & 3) as usize
}

/// DXT1 blocks, in rows of blocks, as RGBA pixels in rows.
pub fn decode(blocks: &[u8], width: usize, height: usize) -> Vec<u8> {
    decode_with(blocks, width, height, DXT1_BYTES, |block, at| palette(block, false)[choice(block, at)])
}

/// DXT3 blocks, in rows of blocks, as RGBA pixels in rows.
pub fn decode_dxt3(blocks: &[u8], width: usize, height: usize) -> Vec<u8> {
    decode_with(blocks, width, height, DXT3_BYTES, |block, at| {
        let alpha = u64::from_le_bytes(block[..8].try_into().unwrap()) >> (4 * at) & 15;
        let [r, g, b, _] = palette(&block[8..], true)[choice(&block[8..], at)];
        // alpha / 15 of the way to opaque: 17 steps of 255 each.
        [r, g, b, alpha as u8 * 17]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: u16 = 0xf800;
    const BLUE: u16 = 0x001f;
    const WHITE: u16 = 0xffff;

    fn block(first: u16, second: u16, choices: [u8; 16]) -> Vec<u8> {
        let bits = choices.iter().enumerate().fold(0u32, |bits, (at, &choice)| bits | u32::from(choice) << (2 * at));
        [first.to_le_bytes().to_vec(), second.to_le_bytes().to_vec(), bits.to_le_bytes().to_vec()].concat()
    }

    #[test]
    fn a_block_with_the_larger_end_first_has_four_opaque_colours() {
        // Red (0xF800) to blue (0x001F): the thirds are 170 and 85 of red.
        let pixels = decode(&block(RED, BLUE, [0, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), 4, 4);
        let row: Vec<&[u8]> = pixels[..16].chunks(4).collect();
        assert_eq!(row, [&[255, 0, 0, 255][..], &[0, 0, 255, 255], &[170, 0, 85, 255], &[85, 0, 170, 255]]);
    }

    #[test]
    fn a_block_with_the_smaller_end_first_has_a_half_way_colour_and_a_clear_one() {
        let pixels = decode(&block(BLUE, RED, [0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]), 4, 4);
        let row: Vec<&[u8]> = pixels[..16].chunks(4).collect();
        assert_eq!(row, [&[0, 0, 255, 255][..], &[255, 0, 0, 255], &[127, 0, 127, 255], &[0, 0, 0, 0]]);
        // Equal ends count as the smaller first.
        assert_eq!(&decode(&block(WHITE, WHITE, [3; 16]), 4, 4)[..4], [0, 0, 0, 0]);
    }

    #[test]
    fn the_first_pixel_is_in_the_lowest_bits_and_rows_go_down() {
        let mut choices = [0; 16];
        choices[1] = 1; // second pixel of the top row
        choices[4] = 1; // first pixel of the second row
        let pixels = decode(&block(RED, BLUE, choices), 4, 4);
        let at = |x: usize, y: usize| &pixels[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4];
        assert_eq!([at(0, 0), at(1, 0), at(0, 1), at(1, 1)], [[255, 0, 0, 255], [0, 0, 255, 255], [0, 0, 255, 255], [255, 0, 0, 255]]);
    }

    #[test]
    fn blocks_go_in_rows_of_blocks() {
        // 8 x 8: four blocks, each one colour, red, blue, white, then black.
        let blocks = [block(RED, 0, [0; 16]), block(BLUE, 0, [0; 16]), block(WHITE, 0, [0; 16]), block(1, 0, [1; 16])].concat();
        let pixels = decode(&blocks, 8, 8);
        let at = |x: usize, y: usize| &pixels[(y * 8 + x) * 4..(y * 8 + x) * 4 + 4];
        assert_eq!([at(0, 0), at(7, 0), at(0, 7), at(7, 7)], [[255, 0, 0, 255], [0, 0, 255, 255], [255, 255, 255, 255], [0, 0, 0, 255]]);
    }

    #[test]
    fn dxt3_gives_each_pixel_its_own_alpha_and_never_a_clear_colour() {
        // Alphas 0, 1, 15 and 8 for the first four pixels; the colour block
        // has the smaller end first, which DXT3 still reads as four opaque
        // colours, so choice 3 is two thirds of the way, not clear.
        let alpha: u64 = 0x8f10;
        let colour = block(BLUE, RED, [3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let pixels = decode_dxt3(&[alpha.to_le_bytes().to_vec(), colour].concat(), 4, 4);
        let row: Vec<&[u8]> = pixels[..16].chunks(4).collect();
        assert_eq!(row, [&[170, 0, 85, 0][..], &[85, 0, 170, 17], &[255, 0, 0, 255], &[0, 0, 255, 136]]);
    }
}
