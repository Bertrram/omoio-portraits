//! DXT5 (BC3) pixels: each 4 x 4 block is 16 bytes, eight of alpha (two ends
//! and a 3-bit choice per pixel) then eight of colour (two RGB565 ends and a
//! 2-bit choice per pixel). The format is the same on every platform, with
//! its numbers little-endian.

const BLOCK_BYTES: usize = 16;

fn expand(value: u16, bits: u32) -> u32 {
    let max = (1u32 << bits) - 1;
    (u32::from(value) * 255 + max / 2) / max
}

fn rgb565(colour: u16) -> [u32; 3] {
    [expand(colour >> 11, 5), expand(colour >> 5 & 63, 6), expand(colour & 31, 5)]
}

/// The blocks, in rows of blocks, as RGBA pixels in rows.
pub fn decode(blocks: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut pixels = vec![0; width * height * 4];
    let blocks_wide = width / 4;
    for (index, block) in blocks.chunks_exact(BLOCK_BYTES).take(blocks_wide * (height / 4)).enumerate() {
        let (left, top) = (index % blocks_wide * 4, index / blocks_wide * 4);

        let (a0, a1) = (u32::from(block[0]), u32::from(block[1]));
        let mut alphas = [a0, a1, 0, 0, 0, 0, 0, 255];
        if a0 > a1 {
            for i in 1..7 {
                alphas[i + 1] = ((7 - i as u32) * a0 + i as u32 * a1) / 7;
            }
        } else {
            for i in 1..5 {
                alphas[i + 1] = ((5 - i as u32) * a0 + i as u32 * a1) / 5;
            }
        }
        let alpha_bits = block[2..8].iter().rev().fold(0u64, |bits, &byte| bits << 8 | u64::from(byte));

        let (c0, c1) = (
            rgb565(u16::from_le_bytes([block[8], block[9]])),
            rgb565(u16::from_le_bytes([block[10], block[11]])),
        );
        let colours = [
            c0,
            c1,
            [0, 1, 2].map(|c| (2 * c0[c] + c1[c]) / 3),
            [0, 1, 2].map(|c| (c0[c] + 2 * c1[c]) / 3),
        ];
        let colour_bits = u32::from_le_bytes([block[12], block[13], block[14], block[15]]);

        for pixel in 0..16 {
            let (x, y) = (left + pixel % 4, top + pixel / 4);
            let colour = colours[(colour_bits >> (2 * pixel) & 3) as usize];
            let alpha = alphas[(alpha_bits >> (3 * pixel) & 7) as usize];
            let at = (y * width + x) * 4;
            pixels[at..at + 4].copy_from_slice(&[colour[0] as u8, colour[1] as u8, colour[2] as u8, alpha as u8]);
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_solid_block_is_one_colour() {
        // Alpha 255 at both ends, pure red at both colour ends, every choice 0.
        let block = [255, 255, 0, 0, 0, 0, 0, 0, 0x00, 0xf8, 0x00, 0xf8, 0, 0, 0, 0];
        let pixels = decode(&block, 4, 4);
        for pixel in pixels.chunks(4) {
            assert_eq!(pixel, [255, 0, 0, 255]);
        }
    }

    #[test]
    fn choices_pick_the_blend_and_the_clear_end() {
        // White to black; the first pixel takes the colour a third of the
        // way, and with a0 <= a1 alpha choice 6 is fully clear.
        let mut block = [0u8; 16];
        block[0] = 0;
        block[1] = 255;
        block[2] = 6;
        block[8..10].copy_from_slice(&0xffffu16.to_le_bytes());
        block[10..12].copy_from_slice(&0x0000u16.to_le_bytes());
        block[12] = 2;
        let pixels = decode(&block, 4, 4);
        assert_eq!(&pixels[0..4], [170, 170, 170, 0]);
    }
}
