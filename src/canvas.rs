//! ADR-0039 canvas geometry.
//!
//! The canvas is the rect the shader renders over, in the effect layer's
//! pixel space at the current (possibly downsampled) resolution. Authority
//! (§1): the shader's declared expansion parameter when present — the layer
//! frame grown by that many pixels per side — else the layer frame unioned
//! with the input's upstream extent, which is what an upstream Grow Bounds
//! enlarges. The per-render ROI request never shapes the canvas.
//!
//! Everything here is pure math so `SmartPreRender` can resolve the canvas
//! once and stash it; the render consumes the stash and cannot disagree.

/// Half-open pixel rect `[left, right) x [top, bottom)` in layer space.
/// The layer frame itself is `(0, 0, w, h)`; expansion makes origins negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn frame(width: i32, height: i32) -> Self {
        Rect { left: 0, top: 0, right: width.max(0), bottom: height.max(0) }
    }

    pub fn width(&self) -> i32 {
        (self.right - self.left).max(0)
    }

    pub fn height(&self) -> i32 {
        (self.bottom - self.top).max(0)
    }

    pub fn is_degenerate(&self) -> bool {
        self.width() == 0 || self.height() == 0
    }

    pub fn union(&self, other: &Rect) -> Rect {
        Rect {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    pub fn intersect(&self, other: &Rect) -> Rect {
        Rect {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }

    pub fn contains(&self, other: &Rect) -> bool {
        other.is_degenerate()
            || (self.left <= other.left
                && self.top <= other.top
                && self.right >= other.right
                && self.bottom >= other.bottom)
    }
}

pub struct Resolved {
    pub canvas: Rect,
    /// The expansion was refused because a canvas dimension would exceed the
    /// device texture limit; `canvas` fell back to the layer frame. The
    /// caller logs E57 (`Diag::CanvasTooLarge`) — behaviour degrades to the
    /// released contract, never a crash (ADR-0039 §6).
    pub limited: bool,
}

/// Resolve the canvas per ADR-0039 §1.
///
/// `declared_margin` is the declared expansion in *physical* pixels per axis
/// (already through [`margin_physical`]); `upstream` is the input's
/// `PF_CheckoutResult::max_result_rect`. A declaration replaces the upstream
/// signal; an undeclared source takes `frame ∪ upstream` so an upstream that
/// only shrinks (a cropping matte) can never shrink the canvas below the
/// released contract.
pub fn resolve(
    layer_w: i32,
    layer_h: i32,
    upstream: Option<Rect>,
    declared_margin: Option<(i32, i32)>,
    max_dim: i32,
) -> Resolved {
    let frame = Rect::frame(layer_w, layer_h);
    let canvas = match declared_margin {
        Some((mx, my)) => {
            let mx = mx.max(0);
            let my = my.max(0);
            Rect {
                left: frame.left - mx,
                top: frame.top - my,
                right: frame.right + mx,
                bottom: frame.bottom + my,
            }
        }
        None => match upstream {
            Some(u) if !u.is_degenerate() => frame.union(&u),
            _ => frame,
        },
    };
    if canvas != frame && (canvas.width() > max_dim || canvas.height() > max_dim) {
        return Resolved { canvas: frame, limited: true };
    }
    Resolved { canvas, limited: false }
}

/// Declared expansion: logical pixels (ADR-0029 units) to physical pixels on
/// one axis under the render's downsample factor. `logical_size` maps
/// physical → logical by `* den / num`; this is its inverse, rounded up so
/// the declared reach is never under-covered by a downsampled render.
pub fn margin_physical(logical: f32, num: i32, den: u32) -> i32 {
    if !logical.is_finite() || logical <= 0.0 {
        return 0;
    }
    if num <= 0 || den == 0 {
        return logical.ceil() as i32;
    }
    (logical * num as f32 / den as f32).ceil() as i32
}

/// Where `inner` content lands inside `outer`, both in layer space:
/// `(src_x, src_y, dst_x, dst_y, w, h)` with src offsets inner-local and dst
/// offsets outer-local. `None` when they do not overlap.
pub fn place(inner: &Rect, outer: &Rect) -> Option<(usize, usize, usize, usize, usize, usize)> {
    let ov = inner.intersect(outer);
    if ov.is_degenerate() {
        return None;
    }
    Some((
        (ov.left - inner.left) as usize,
        (ov.top - inner.top) as usize,
        (ov.left - outer.left) as usize,
        (ov.top - outer.top) as usize,
        ov.width() as usize,
        ov.height() as usize,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undeclared_with_plain_upstream_is_the_layer_frame() {
        // The released contract byte-for-byte: no declaration, upstream
        // extent equal to the frame (or absent, or degenerate).
        let frame = Rect::frame(512, 512);
        for upstream in [Some(frame), None, Some(Rect { left: 3, top: 3, right: 3, bottom: 9 })] {
            let r = resolve(512, 512, upstream, None, 16384);
            assert_eq!(r.canvas, frame);
            assert!(!r.limited);
        }
    }

    #[test]
    fn undeclared_grows_to_the_upstream_extent_but_never_shrinks() {
        // Grow Bounds 256 around a 512 layer.
        let grown = Rect { left: -256, top: -256, right: 768, bottom: 768 };
        let r = resolve(512, 512, Some(grown), None, 16384);
        assert_eq!(r.canvas, grown);

        // A cropping upstream (matte) must not shrink the canvas below the
        // frame: union, not adoption.
        let cropped = Rect { left: 100, top: 100, right: 300, bottom: 300 };
        let r = resolve(512, 512, Some(cropped), None, 16384);
        assert_eq!(r.canvas, Rect::frame(512, 512));

        // A partially-outside upstream unions.
        let shifted = Rect { left: -64, top: 0, right: 400, bottom: 512 };
        let r = resolve(512, 512, Some(shifted), None, 16384);
        assert_eq!(r.canvas, Rect { left: -64, top: 0, right: 512, bottom: 512 });
    }

    #[test]
    fn declared_margin_replaces_the_upstream_signal() {
        // Declared 64 under Grow Bounds 256: the author's boundary is law
        // (ADR-0039 §1) — the canvas is frame+64, not frame+256.
        let grown = Rect { left: -256, top: -256, right: 768, bottom: 768 };
        let r = resolve(512, 512, Some(grown), Some((64, 64)), 16384);
        assert_eq!(r.canvas, Rect { left: -64, top: -64, right: 576, bottom: 576 });

        // Declared 0 is exactly the frame even under a grown upstream.
        let r = resolve(512, 512, Some(grown), Some((0, 0)), 16384);
        assert_eq!(r.canvas, Rect::frame(512, 512));
    }

    #[test]
    fn oversize_expansion_falls_back_to_the_frame_and_flags_it() {
        let r = resolve(512, 512, None, Some((9000, 9000)), 16384);
        assert!(r.limited);
        assert_eq!(r.canvas, Rect::frame(512, 512));

        let grown = Rect { left: -20000, top: 0, right: 512, bottom: 512 };
        let r = resolve(512, 512, Some(grown), None, 16384);
        assert!(r.limited);
        assert_eq!(r.canvas, Rect::frame(512, 512));

        // An oversize *layer* is not this guard's business: the canvas equals
        // the frame, and whatever the host/GPU path did before still applies.
        let r = resolve(20000, 512, None, None, 16384);
        assert!(!r.limited);
        assert_eq!(r.canvas, Rect::frame(20000, 512));
    }

    #[test]
    fn margin_physical_inverts_logical_size_and_rounds_up() {
        // Full resolution: identity.
        assert_eq!(margin_physical(256.0, 1, 1), 256);
        // Half resolution: 256 logical = 128 physical.
        assert_eq!(margin_physical(256.0, 1, 2), 128);
        // Third resolution rounds UP so the reach is covered.
        assert_eq!(margin_physical(256.0, 1, 3), 86);
        // Degenerate factors pass the value through.
        assert_eq!(margin_physical(64.0, 0, 0), 64);
        // Negative and non-finite clamp to zero.
        assert_eq!(margin_physical(-5.0, 1, 1), 0);
        assert_eq!(margin_physical(f32::NAN, 1, 1), 0);
    }

    #[test]
    fn place_maps_inner_content_into_outer_coordinates() {
        // A 512 world inside a canvas expanded 256 on every side.
        let world = Rect::frame(512, 512);
        let canvas = Rect { left: -256, top: -256, right: 768, bottom: 768 };
        assert_eq!(place(&world, &canvas), Some((0, 0, 256, 256, 512, 512)));

        // The degenerate identity: canvas == world.
        assert_eq!(place(&world, &world), Some((0, 0, 0, 0, 512, 512)));

        // A world partly outside the canvas (declared crop) clips.
        let wide = Rect { left: -100, top: 0, right: 612, bottom: 512 };
        let tight = Rect { left: -64, top: -64, right: 576, bottom: 576 };
        assert_eq!(place(&wide, &tight), Some((36, 0, 0, 64, 640, 512)));

        // Disjoint rects place nothing.
        let off = Rect { left: 1000, top: 1000, right: 1100, bottom: 1100 };
        assert_eq!(place(&off, &world), None);
    }
}
