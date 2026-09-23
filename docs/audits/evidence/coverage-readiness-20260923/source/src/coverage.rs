use crate::{diagnostics::Diag, render::Depth};

pub fn readiness(state: f64, certificate: u64) -> Result<(), Diag> {
    if state == 2.0 { return Err(Diag::CoverageUnsupported); }
    if (4..=(1 << 48) - 1).contains(&certificate) && state == certificate as f64 { return Ok(()); }
    Err(Diag::CoverageUnavailable)
}

pub struct Plane<'a> {
    pub pixels: &'a [u8],
    pub stride: usize,
    pub width: usize,
    pub height: usize,
    pub origin: (i32, i32),
}

pub fn encode(plane: Plane<'_>, depth: Depth, canvas: (i32, i32, usize, usize)) -> Result<Vec<u8>, &'static str> {
    let input_bpp = match depth { Depth::U8 => 4, Depth::U15 => 8, Depth::F32 => 16 };
    let row = plane.width.checked_mul(input_bpp).ok_or("coverage row overflow")?;
    if plane.stride < row || plane.stride.checked_mul(plane.height).is_none_or(|n| n > plane.pixels.len()) {
        return Err("coverage buffer bounds");
    }
    let (cx, cy, width, height) = canvas;
    let bytes = width.checked_mul(height).and_then(|n| n.checked_mul(depth.bpp())).ok_or("coverage canvas overflow")?;
    let mut result = Vec::new();
    result.try_reserve_exact(bytes).map_err(|_| "coverage allocation failed")?;
    result.resize(bytes, 0);
    let left = i64::from(cx).max(i64::from(plane.origin.0));
    let top = i64::from(cy).max(i64::from(plane.origin.1));
    let right = (i64::from(cx) + width as i64).min(i64::from(plane.origin.0) + plane.width as i64);
    let bottom = (i64::from(cy) + height as i64).min(i64::from(plane.origin.1) + plane.height as i64);
    for y in top..bottom {
        for x in left..right {
            let src = (y - i64::from(plane.origin.1)) as usize * plane.stride
                + (x - i64::from(plane.origin.0)) as usize * input_bpp;
            let dst = ((y - i64::from(cy)) as usize * width + (x - i64::from(cx)) as usize) * depth.bpp();
            match depth {
                Depth::U8 => result[dst..dst + 4].fill(plane.pixels[src]),
                Depth::U15 => {
                    let a = u16::from_ne_bytes([plane.pixels[src], plane.pixels[src + 1]]) as f32 / 32768.0;
                    for channel in result[dst..dst + 16].chunks_exact_mut(4) { channel.copy_from_slice(&a.to_ne_bytes()); }
                }
                Depth::F32 => {
                    for channel in result[dst..dst + 16].chunks_exact_mut(4) { channel.copy_from_slice(&plane.pixels[src..src + 4]); }
                }
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unready_unknown_and_nonfinite_states_never_authorize_coverage() {
        assert_eq!(readiness(42.0, 42), Ok(()));
        assert_eq!(readiness(42.0, 43), Err(Diag::CoverageUnavailable));
        assert_eq!(readiness(42.0, 0), Err(Diag::CoverageUnavailable));
        assert_eq!(readiness(2.0, 42), Err(Diag::CoverageUnsupported));
        for state in [0.0, 1.0, 3.0, -1.0, 1.5, f64::NAN, f64::INFINITY] {
            assert_eq!(readiness(state, 42), Err(Diag::CoverageUnavailable));
        }
    }
    #[test]
    fn crop_origin_padding_and_stride_are_not_stretched() {
        let pixels = [11, 1, 2, 3, 22, 4, 5, 6, 255, 255, 255, 255,
                      33, 1, 2, 3, 44, 4, 5, 6, 255, 255, 255, 255];
        let result = encode(Plane { pixels: &pixels, width: 2, height: 2, stride: 12, origin: (1, 1) }, Depth::U8, (0, 0, 4, 3)).unwrap();
        let expected = [0, 0, 0, 0, 0, 11, 22, 0, 0, 33, 44, 0];
        for (pixel, value) in result.chunks_exact(4).zip(expected) { assert_eq!(pixel, [value; 4]); }
        let clipped = encode(Plane { pixels: &pixels, width: 2, height: 2, stride: 12, origin: (-1, -1) }, Depth::U8, (0, 0, 2, 2)).unwrap();
        assert_eq!(&clipped[..4], &[44; 4]);
        assert!(clipped[4..].iter().all(|b| *b == 0));
    }
    #[test]
    fn every_u15_alpha_is_exact_in_float32() {
        let pixels: Vec<u8> = (0..=32768u16).flat_map(|a| [a.to_ne_bytes(), [0; 2], [0; 2], [0; 2]].concat()).collect();
        let result = encode(Plane { pixels: &pixels, width: 32769, height: 1, stride: pixels.len(), origin: (0, 0) }, Depth::U15, (0, 0, 32769, 1)).unwrap();
        for (a, pixel) in result.chunks_exact(16).enumerate() {
            for channel in pixel.chunks_exact(4) {
                assert_eq!(f32::from_ne_bytes(channel.try_into().unwrap()) * 32768.0, a as f32);
            }
        }
    }
    #[test]
    fn float_words_are_copied_without_canonicalizing_or_clamping() {
        let words: [u32; 5] = [0x80000000, 0x3f000001, 0x40000000, 0xbf000000, 0x7fc01234];
        let pixels: Vec<u8> = words.iter().flat_map(|w| [w.to_ne_bytes(), [1; 4], [2; 4], [3; 4]].concat()).collect();
        let result = encode(Plane { pixels: &pixels, width: 5, height: 1, stride: pixels.len(), origin: (5, 9) }, Depth::F32, (5, 9, 5, 1)).unwrap();
        for (pixel, word) in result.chunks_exact(16).zip(words) {
            for channel in pixel.chunks_exact(4) { assert_eq!(channel, word.to_ne_bytes()); }
        }
    }
    #[test]
    fn invalid_rows_and_truncated_buffers_are_rejected() {
        for (pixels, stride) in [(&[0u8; 4][..], 8usize), (&[0u8; 8][..], 4)] {
            assert!(encode(Plane { pixels, stride, width: 2, height: 1, origin: (0, 0) }, Depth::U8, (0, 0, 2, 1)).is_err());
        }
    }
}
