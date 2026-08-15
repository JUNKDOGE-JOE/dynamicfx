//! Gradient values and LUT baking (ADR-0031 §4/§5, ADR-0033).
//!
//! This module is pure value logic. It does **not** persist anything: since
//! [ADR-0033] a gradient's stops are ordinary AE parameters, so After Effects
//! owns saving, undo, copy/paste and keyframes. A `Gradient` here is just the
//! snapshot assembled from those parameters for one frame, so it can be
//! sampled and baked into the LUT the shader reads.
//!
//! A snapshot that violates the rules **fails closed** (`E54`) rather than
//! being repaired by guesswork — per-stop keyframes make a non-monotone order
//! representable, so the read has to check.
//!
//! [ADR-0033]: ../../docs/adr/0033-gradient-stops-are-ordinary-parameters.md

/// ADR-0031 §3. Raising this is an append-compatible format change; lowering
/// it is not.
pub const MAX_STOPS: usize = 8;

/// ADR-0031 §5: 256 samples is past visible banding for a ramp at 8-bpc and
/// costs 4 KB at float precision.
pub const LUT_WIDTH: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stop {
    pub position: f32,
    /// Straight sRGB, `0..1` per channel — the same encoding `hint:color`
    /// already delivers to shaders (ADR-0026), so the two agree.
    pub rgba: [f32; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub stops: Vec<Stop>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientError {
    Empty,
    TooManyStops(usize),
    PositionOutOfRange,
    NotSorted,
}

impl Default for Gradient {
    /// A neutral black→white ramp. Chosen over a single-stop flat value so a
    /// freshly added control reads as a *gradient* on sight.
    fn default() -> Self {
        Self {
            stops: vec![
                Stop { position: 0.0, rgba: [0.0, 0.0, 0.0, 1.0] },
                Stop { position: 1.0, rgba: [1.0, 1.0, 1.0, 1.0] },
            ],
        }
    }
}

impl Gradient {
    /// Assemble one frame's snapshot from the live stop parameters
    /// (ADR-0033 §1). `count` is the gradient's `Stops` value; `stops` supplies
    /// every declared slot, live or not, so the caller need not slice.
    pub fn from_parameters(count: usize, stops: &[Stop]) -> Self {
        let live = count.min(stops.len());
        Self { stops: stops[..live].to_vec() }
    }

    /// ADR-0033 §5 validation, re-scoped from a decoded blob to a live read:
    /// per-stop keyframes make a non-monotone order representable, so the read
    /// checks. Never repairs — a silently reordered ramp would render a
    /// picture the user never authored.
    pub fn validate(&self) -> Result<(), GradientError> {
        if self.stops.is_empty() {
            return Err(GradientError::Empty);
        }
        if self.stops.len() > MAX_STOPS {
            return Err(GradientError::TooManyStops(self.stops.len()));
        }
        let mut previous = f32::NEG_INFINITY;
        for stop in &self.stops {
            if !stop.position.is_finite() || !(0.0..=1.0).contains(&stop.position) {
                return Err(GradientError::PositionOutOfRange);
            }
            if stop.position < previous {
                return Err(GradientError::NotSorted);
            }
            previous = stop.position;
        }
        Ok(())
    }

    /// Colour at `t`, linearly interpolated in straight sRGB (ADR-0031 §4).
    /// Outside the first/last stop the nearest stop's colour is held — a ramp
    /// whose stops do not reach the ends must not fade to transparent.
    pub fn sample(&self, t: f32) -> [f32; 4] {
        let Some(first) = self.stops.first() else { return [0.0; 4] };
        let t = t.clamp(0.0, 1.0);
        if t <= first.position {
            return first.rgba;
        }
        let last = self.stops.last().expect("non-empty");
        if t >= last.position {
            return last.rgba;
        }
        for pair in self.stops.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if t >= a.position && t <= b.position {
                let span = b.position - a.position;
                // Coincident stops are a legal hard edge: take the later one.
                if span <= f32::EPSILON {
                    return b.rgba;
                }
                let k = (t - a.position) / span;
                let mut out = [0.0f32; 4];
                for c in 0..4 {
                    out[c] = a.rgba[c] + (b.rgba[c] - a.rgba[c]) * k;
                }
                return out;
            }
        }
        last.rgba
    }

    /// Bake the `LUT_WIDTH x 1` strip the shader samples. Texel centres, so
    /// texel 0 is `t = 0.5/W` rather than 0 — sampling with `vec2(t, 0.5)`
    /// and linear filtering then reproduces `sample()` across the whole ramp.
    pub fn bake_lut(&self) -> Vec<[f32; 4]> {
        (0..LUT_WIDTH)
            .map(|i| self.sample((i as f32 + 0.5) / LUT_WIDTH as f32))
            .collect()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn stop(position: f32, v: f32) -> Stop {
        Stop { position, rgba: [v, v, v, 1.0] }
    }

    #[test]
    fn default_is_a_valid_black_to_white_ramp() {
        let g = Gradient::default();
        assert!(g.validate().is_ok());
        assert_eq!(g.sample(0.0), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(g.sample(1.0), [1.0, 1.0, 1.0, 1.0]);
        assert!((g.sample(0.5)[0] - 0.5).abs() < 1e-6);
    }

    /// Every malformed shape is rejected, never repaired (ADR-0031 §3).
    #[test]
    fn malformed_values_fail_closed() {
        assert_eq!(Gradient { stops: vec![] }.validate(), Err(GradientError::Empty));

        let too_many = Gradient {
            stops: (0..MAX_STOPS + 1)
                .map(|i| stop(i as f32 / (MAX_STOPS as f32 + 1.0), 0.0))
                .collect(),
        };
        assert_eq!(too_many.validate(), Err(GradientError::TooManyStops(MAX_STOPS + 1)));

        let out_of_range = Gradient { stops: vec![stop(0.0, 0.0), stop(1.5, 1.0)] };
        assert_eq!(out_of_range.validate(), Err(GradientError::PositionOutOfRange));

        let nan = Gradient { stops: vec![stop(f32::NAN, 0.0)] };
        assert_eq!(nan.validate(), Err(GradientError::PositionOutOfRange));

        let unsorted = Gradient { stops: vec![stop(0.8, 0.0), stop(0.2, 1.0)] };
        assert_eq!(unsorted.validate(), Err(GradientError::NotSorted));

        // A maximum-length value is legal — the cap is inclusive.
        let exactly_max = Gradient {
            stops: (0..MAX_STOPS).map(|i| stop(i as f32 / (MAX_STOPS - 1) as f32, 0.0)).collect(),
        };
        assert!(exactly_max.validate().is_ok());
    }

    #[test]
    fn stops_outside_the_ends_hold_rather_than_fade() {
        let g = Gradient { stops: vec![stop(0.25, 0.2), stop(0.75, 0.8)] };
        assert_eq!(g.sample(0.0), g.stops[0].rgba, "below the first stop holds");
        assert_eq!(g.sample(1.0), g.stops[1].rgba, "above the last stop holds");
    }

    #[test]
    fn coincident_stops_make_a_hard_edge() {
        let g = Gradient { stops: vec![stop(0.0, 0.0), stop(0.5, 0.0), stop(0.5, 1.0)] };
        assert_eq!(g.sample(0.49)[0], 0.0);
        assert_eq!(g.sample(0.5)[0], 1.0);
    }

    /// ADR-0033 §1: the snapshot assembled from live parameters must produce
    /// exactly the LUT the equivalent literal gradient does. Declared slots
    /// past the live count are ignored, not blended in.
    #[test]
    fn parameter_snapshot_matches_the_literal_gradient() {
        let declared = vec![
            stop(0.0, 0.0),
            stop(1.0, 1.0),
            // Slots 3..8 exist in the topology but are not live; if the read
            // ever included them the ramp would fold back on itself.
            stop(0.4, 0.9),
            stop(0.2, 0.1),
        ];
        let from_params = Gradient::from_parameters(2, &declared);
        let literal = Gradient { stops: vec![stop(0.0, 0.0), stop(1.0, 1.0)] };
        assert_eq!(from_params, literal);
        assert_eq!(from_params.bake_lut(), literal.bake_lut());
        assert!(from_params.validate().is_ok());
    }

    /// A count beyond what the caller supplied must clamp rather than panic —
    /// the count is a keyframeable parameter and can outrun reality.
    #[test]
    fn snapshot_clamps_a_count_past_the_declared_slots() {
        let declared = vec![stop(0.0, 0.0), stop(1.0, 1.0)];
        let g = Gradient::from_parameters(99, &declared);
        assert_eq!(g.stops.len(), 2);
        assert!(g.validate().is_ok());
    }

    /// Per-stop keyframes make a non-monotone order representable, so the read
    /// has to reject it (ADR-0033 §5) rather than silently render a ramp the
    /// user never authored.
    #[test]
    fn animated_stops_out_of_order_are_rejected_not_repaired() {
        let declared = vec![stop(0.8, 0.0), stop(0.2, 1.0)];
        let g = Gradient::from_parameters(2, &declared);
        assert_eq!(g.validate(), Err(GradientError::NotSorted));
        // And the value is untouched — no quiet sort behind the user's back.
        assert_eq!(g.stops[0].position, 0.8);
    }

    #[test]
    fn lut_uses_texel_centres_and_covers_the_ramp() {
        let lut = Gradient::default().bake_lut();
        assert_eq!(lut.len(), LUT_WIDTH);
        // Monotone black -> white, endpoints near but not exactly 0/1 because
        // the samples sit at texel centres.
        assert!(lut[0][0] > 0.0 && lut[0][0] < 0.01);
        assert!(lut[LUT_WIDTH - 1][0] > 0.99 && lut[LUT_WIDTH - 1][0] < 1.0);
        for pair in lut.windows(2) {
            assert!(pair[1][0] >= pair[0][0], "ramp must not go backwards");
        }
    }


}
