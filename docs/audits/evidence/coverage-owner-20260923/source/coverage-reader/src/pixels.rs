#[derive(Clone, Copy)]
pub struct Plane { pub width: usize, pub height: usize, pub stride: usize, pub x: i32, pub y: i32 }

pub fn copy(source: &[u8], output: &mut [u8], src: Plane, dst: Plane, size: usize) -> Result<(), &'static str> {
    if ![4, 8, 16].contains(&size) { return Err("pixel format"); }
    for (plane, len) in [(src, source.len()), (dst, output.len())] {
        if plane.width.checked_mul(size).is_none_or(|row| row > plane.stride) ||
            plane.height.checked_mul(plane.stride).is_none_or(|bytes| bytes > len) {
            return Err("pixel bounds");
        }
    }
    output.fill(0);
    let left = i64::from(src.x).max(i64::from(dst.x));
    let top = i64::from(src.y).max(i64::from(dst.y));
    let right = (i64::from(src.x) + src.width as i64).min(i64::from(dst.x) + dst.width as i64);
    let bottom = (i64::from(src.y) + src.height as i64).min(i64::from(dst.y) + dst.height as i64);
    if right <= left || bottom <= top { return Ok(()); }
    let bytes = (right - left) as usize * size;
    for y in top..bottom {
        let from = (y - i64::from(src.y)) as usize * src.stride + (left - i64::from(src.x)) as usize * size;
        let to = (y - i64::from(dst.y)) as usize * dst.stride + (left - i64::from(dst.x)) as usize * size;
        output[to..to + bytes].copy_from_slice(&source[from..from + bytes]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_words_and_padded_rows_survive_positive_and_negative_origins() {
        for size in [4, 8, 16] {
            let source: Vec<_> = (0..size * 6).map(|i| (i + 1) as u8).collect();
            for origin in [(-10, -20), (10, 20)] {
                let src = Plane { width: 2, height: 2, stride: size * 3, x: origin.0 + 1, y: origin.1 + 1 };
                let dst = Plane { width: 3, height: 3, stride: size * 4, x: origin.0, y: origin.1 };
                let mut output = vec![255; size * 12];
                let mut expected = vec![0; size * 12];
                expected[size * 5..size * 7].copy_from_slice(&source[..size * 2]);
                expected[size * 9..size * 11].copy_from_slice(&source[size * 3..size * 5]);
                copy(&source, &mut output, src, dst, size).unwrap();
                assert_eq!(output, expected);
                assert!(copy(&source[..size], &mut output, src, dst, size).is_err());
            }
        }
    }
    #[test]
    fn clipping_empty_intersection_and_hdr_words_are_exact() {
        let words: [u32; 8] = [0x80000000, 0x7fc01234, 0x40000000, 0xbf000000, 1, 2, 3, 4];
        let source: Vec<_> = words.into_iter().flat_map(u32::to_ne_bytes).collect();
        let src = Plane { width: 2, height: 1, stride: 32, x: -1, y: 0 };
        let dst = Plane { width: 1, height: 1, stride: 16, x: 0, y: 0 };
        let mut output = [255; 16];
        copy(&source, &mut output, src, dst, 16).unwrap();
        assert_eq!(output, source[16..]);
        copy(&source, &mut output, src, Plane { x: -1, ..dst }, 16).unwrap();
        assert_eq!(output, source[..16]);
        copy(&source, &mut output, src, Plane { y: 2, ..dst }, 16).unwrap();
        assert_eq!(output, [0; 16]);
    }
}
