//! Path parameters: AE mask vertices as a texture (ADR-0035).
//!
//! Everything here is pure. The AE-side checkout lives in `lib.rs`; this module
//! only turns the vertices it produces into the ABI's texel layout, so the
//! encoding contract is testable without a host.
//!
//! ADR-0035 §3 fixes the layout as `N x 2` `Rgba32Float`:
//!
//! - row 0, texel `i`: `(x, y, tan_out_x, tan_out_y)`
//! - row 1, texel `i`: `(tan_in_x, tan_in_y, 0, 1)`
//!
//! so a shader that only wants positions reads row 0 and ignores row 1. Every
//! coordinate is normalized to the frame, the same convention Point 2D uses
//! (§3), which is what lets a path vertex and a `hint:point` parameter mean the
//! same thing in one shader.

/// One vertex as `PF_PathVertex` reports it, in layer pixels.
///
/// Whether PF reports the tangents as offsets *from* the vertex or as absolute
/// handle positions is not established from the SDK header — it is not vendored
/// with the crate. This module divides them by the frame size either way, so
/// the delivered value is "what PF reported, normalized"; which of the two
/// readings applies is a host-verification item, not something asserted here.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub tan_in_x: f32,
    pub tan_in_y: f32,
    pub tan_out_x: f32,
    pub tan_out_y: f32,
}

/// Rows in the encoded texture (ADR-0035 §3). A persistent contract the moment
/// a shader reads it: row 0 is positions + outgoing tangents, row 1 is incoming
/// tangents.
pub const ROWS: usize = 2;

/// Upper bound on delivered vertices.
///
/// AE puts no limit on mask vertices, but `max_texture_dimension_2d` does — the
/// wgpu default is 8192, and a wider texture fails creation rather than
/// degrading. Clamping keeps a pathological mask renderable; the caller logs
/// what was dropped, because a silent cap reads as "we delivered everything".
pub const MAX_VERTICES: usize = 8192;

/// Encode vertices into the ADR-0035 §3 texel layout.
///
/// Returns `(width, samples)` where `width` is the delivered vertex count — the
/// value a shader reads back as `textureSize(u_path, 0).x` (§4) — and `samples`
/// is `width * ROWS` texels in row-major order.
///
/// An empty path yields the documented `1 x 2` all-zero texture (§5) rather
/// than an error, so a shader reading an unset selector still renders.
pub fn encode(vertices: &[Vertex], width_px: f32, height_px: f32) -> (usize, Vec<[f32; 4]>) {
    // Guard the divisors rather than the inputs: a zero-sized frame is not a
    // path problem, and NaN in the output would poison the whole texture.
    let sx = if width_px.is_finite() && width_px > 0.0 { width_px } else { 1.0 };
    let sy = if height_px.is_finite() && height_px > 0.0 { height_px } else { 1.0 };

    if vertices.is_empty() {
        return (1, vec![[0.0; 4]; ROWS]);
    }
    let count = vertices.len().min(MAX_VERTICES);
    let mut samples = vec![[0.0f32; 4]; count * ROWS];
    for (i, v) in vertices.iter().take(count).enumerate() {
        samples[i] = [v.x / sx, v.y / sy, v.tan_out_x / sx, v.tan_out_y / sy];
        // The `0, 1` tail is fixed by §3: it leaves room for a later meaning
        // without moving what a shader already reads, and an alpha of 1 keeps
        // the row from reading as "transparent" to anything that samples it
        // as a colour by accident.
        samples[count + i] = [v.tan_in_x / sx, v.tan_in_y / sy, 0.0, 1.0];
    }
    (count, samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(n: f32) -> Vertex {
        Vertex {
            x: n,
            y: n + 1.0,
            tan_in_x: n + 2.0,
            tan_in_y: n + 3.0,
            tan_out_x: n + 4.0,
            tan_out_y: n + 5.0,
        }
    }

    /// ADR-0035 §3/§4: the round trip a shader performs — read the width as the
    /// count, texel `i` of row 0 as position + outgoing tangent, texel `i` of
    /// row 1 as the incoming tangent — with everything divided by the frame.
    #[test]
    fn vertices_round_trip_through_the_texel_layout() {
        let vertices: Vec<Vertex> = (0..4).map(|i| vertex(i as f32 * 10.0)).collect();
        let (count, samples) = encode(&vertices, 200.0, 100.0);
        assert_eq!(count, vertices.len(), "the width IS the vertex count");
        assert_eq!(samples.len(), count * ROWS);
        for (i, v) in vertices.iter().enumerate() {
            assert_eq!(
                samples[i],
                [v.x / 200.0, v.y / 100.0, v.tan_out_x / 200.0, v.tan_out_y / 100.0]
            );
            assert_eq!(
                samples[count + i],
                [v.tan_in_x / 200.0, v.tan_in_y / 100.0, 0.0, 1.0]
            );
        }
    }

    /// §5: an unassigned selector, a deleted mask, or a path with no segments
    /// binds a 1x2 all-zero texture — never an error, never a zero-width
    /// texture (which would fail creation and take the render with it).
    #[test]
    fn an_empty_path_is_the_documented_zero_texture() {
        let (count, samples) = encode(&[], 200.0, 100.0);
        assert_eq!(count, 1);
        assert_eq!(samples, vec![[0.0; 4]; ROWS]);
    }

    /// A frame size the host has no business reporting must not produce NaN in
    /// a texture every pass then samples.
    #[test]
    fn a_degenerate_frame_size_never_yields_nan() {
        for (w, h) in [(0.0, 0.0), (-1.0, 100.0), (f32::NAN, 100.0), (200.0, f32::INFINITY)] {
            let (_, samples) = encode(&[vertex(3.0)], w, h);
            for texel in &samples {
                assert!(texel.iter().all(|c| c.is_finite()), "{w}x{h} produced {texel:?}");
            }
        }
    }

    /// The cap is enforced, not exceeded — a mask wider than the texture limit
    /// would otherwise fail texture creation instead of rendering.
    #[test]
    fn the_vertex_cap_is_enforced() {
        let vertices = vec![vertex(1.0); MAX_VERTICES + 100];
        let (count, samples) = encode(&vertices, 200.0, 100.0);
        assert_eq!(count, MAX_VERTICES);
        assert_eq!(samples.len(), MAX_VERTICES * ROWS);
    }
}
