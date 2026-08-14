//! The immutable ExecutionPlan (ADR-0020): deterministic topological steps
//! plus first-fit lifetime aliasing of intermediate textures. Built once per
//! (graph, extent, format); never re-analyzed per frame (architecture §9.2).

use crate::frontend::grammar::{topological_order, ManifestPass, RES_INPUT, RES_OUTPUT, RES_PREV};

/// Where a step reads from or writes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexSlot {
    EffectInput,
    FinalOutput,
    /// The previous frame's final output (ADR-0023). Read-only; lives
    /// outside the aliasing pool by definition (ADR-0024 §3).
    History,
    /// Index into the plan's physical intermediate pool.
    Physical(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionStep {
    /// Index into the manifest/pass arrays.
    pub pass_index: usize,
    /// Parallel to the pass's manifest inputs.
    pub inputs: Vec<TexSlot>,
    pub output: TexSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    /// Number of physical intermediate textures to allocate.
    pub physical_count: usize,
}

/// Build the plan. `alias=false` (the `DYNAMICFX_NO_ALIAS` kill switch)
/// gives every intermediate its own physical texture; output must be
/// identical either way (ADR-0020 §5).
pub fn build_plan(passes: &[ManifestPass], alias: bool) -> ExecutionPlan {
    let order = topological_order(passes);

    // Step index at which each intermediate is written / last read.
    let mut resources: Vec<(&str, usize, usize)> = Vec::new(); // name, write, last_read
    for (step, &pass_index) in order.iter().enumerate() {
        let pass = &passes[pass_index];
        if pass.output != RES_OUTPUT {
            resources.push((&pass.output, step, step));
        }
        for input in &pass.inputs {
            if input != RES_INPUT && input != RES_PREV {
                if let Some(entry) = resources.iter_mut().find(|(name, _, _)| name == input) {
                    entry.2 = step;
                }
            }
        }
    }

    // First-fit physical assignment in write order (= lifetime order under
    // the deterministic schedule); ties inherit declaration order because
    // `resources` is built in step order.
    let mut physical_free_at: Vec<usize> = Vec::new(); // last step each slot is busy through
    let mut assignment: Vec<(&str, usize)> = Vec::new();
    for &(name, write, last_read) in &resources {
        let slot = if alias {
            physical_free_at.iter().position(|&busy_through| busy_through < write)
        } else {
            None
        };
        let slot = match slot {
            Some(existing) => {
                physical_free_at[existing] = last_read;
                existing
            }
            None => {
                physical_free_at.push(last_read);
                physical_free_at.len() - 1
            }
        };
        assignment.push((name, slot));
    }

    let slot_of = |name: &str| -> TexSlot {
        if name == RES_INPUT {
            TexSlot::EffectInput
        } else if name == RES_OUTPUT {
            TexSlot::FinalOutput
        } else if name == RES_PREV {
            TexSlot::History
        } else {
            TexSlot::Physical(
                assignment
                    .iter()
                    .find(|(n, _)| *n == name)
                    .map(|(_, slot)| *slot)
                    .expect("validated graphs only reference written resources"),
            )
        }
    };

    let steps = order
        .iter()
        .map(|&pass_index| {
            let pass = &passes[pass_index];
            ExecutionStep {
                pass_index,
                inputs: pass.inputs.iter().map(|i| slot_of(i)).collect(),
                output: slot_of(&pass.output),
            }
        })
        .collect();

    ExecutionPlan { steps, physical_count: physical_free_at.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::grammar::parse_envelope;

    fn plan_of(src: &str, alias: bool) -> ExecutionPlan {
        build_plan(&parse_envelope(src).unwrap().passes, alias)
    }

    /// ADR-0023/0024: `prev` maps to the History slot, which never joins
    /// the physical aliasing pool.
    #[test]
    fn prev_maps_to_history_outside_aliasing() {
        let src = "@dynamicfx 1\n@graph\npass sim: input, prev -> t\npass post: t, prev -> output\n@end\n@pass sim\nx\n@endpass\n@pass post\nx\n@endpass\n";
        let plan = plan_of(src, true);
        assert_eq!(plan.physical_count, 1); // only `t`
        assert_eq!(plan.steps[0].inputs, vec![TexSlot::EffectInput, TexSlot::History]);
        assert_eq!(plan.steps[1].inputs, vec![TexSlot::Physical(0), TexSlot::History]);
        assert_eq!(plan.steps[1].output, TexSlot::FinalOutput);
        // The no-alias switch changes nothing about History.
        let plan2 = plan_of(src, false);
        assert_eq!(plan2.steps[0].inputs[1], TexSlot::History);
    }

    /// Golden plan: a 4-pass chain needs exactly 2 physical intermediates
    /// (ADR-0020 §6), and without aliasing it needs 3.
    #[test]
    fn chain_uses_two_physical_intermediates()  {
        let src = "@dynamicfx 1\n@graph\npass p1: input -> a\npass p2: a -> b\npass p3: b -> c\npass p4: c -> output\n@end\n@pass p1\nx\n@endpass\n@pass p2\nx\n@endpass\n@pass p3\nx\n@endpass\n@pass p4\nx\n@endpass\n";
        let plan = plan_of(src, true);
        assert_eq!(plan.physical_count, 2);
        assert_eq!(plan.steps[0].output, TexSlot::Physical(0)); // a
        assert_eq!(plan.steps[1].output, TexSlot::Physical(1)); // b (a still live)
        assert_eq!(plan.steps[2].output, TexSlot::Physical(0)); // c reuses a's slot
        assert_eq!(plan.steps[3].output, TexSlot::FinalOutput);
        assert_eq!(plan.steps[3].inputs, vec![TexSlot::Physical(0)]);

        let no_alias = plan_of(src, false);
        assert_eq!(no_alias.physical_count, 3);
        // Steps must be identical apart from physical numbering.
        assert_eq!(no_alias.steps.len(), plan.steps.len());
    }

    /// Golden plan: a diamond keeps both branches live simultaneously.
    #[test]
    fn diamond_keeps_both_branches_live() {
        let src = "@dynamicfx 1\n@graph\npass left: input -> l\npass right: input -> r\npass join: l, r -> output\n@end\n@pass left\nx\n@endpass\n@pass right\nx\n@endpass\n@pass join\nx\n@endpass\n";
        let plan = plan_of(src, true);
        assert_eq!(plan.physical_count, 2);
        assert_eq!(
            plan.steps[2].inputs,
            vec![TexSlot::Physical(0), TexSlot::Physical(1)]
        );
    }

    /// Adjacent-step rule: a resource last read at step N frees for a write
    /// at step N+1, but two resources alive in the same step never share.
    #[test]
    fn adjacent_step_reuse_and_same_step_exclusion() {
        // p2 reads a (last read step 1) and writes b at the SAME step —
        // b must not take a's slot.
        let src = "@dynamicfx 1\n@graph\npass p1: input -> a\npass p2: a -> b\npass p3: b -> output\n@end\n@pass p1\nx\n@endpass\n@pass p2\nx\n@endpass\n@pass p3\nx\n@endpass\n";
        let plan = plan_of(src, true);
        assert_eq!(plan.steps[1].inputs, vec![TexSlot::Physical(0)]);
        assert_eq!(plan.steps[1].output, TexSlot::Physical(1));
        assert_eq!(plan.physical_count, 2);
    }

    #[test]
    fn plans_are_deterministic() {
        let src = "@dynamicfx 1\n@graph\npass late: e -> output\npass a: input -> a_out\npass b: input -> b_out\npass early: a_out, b_out -> e\n@end\n@pass late\nx\n@endpass\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n@pass early\nx\n@endpass\n";
        let one = plan_of(src, true);
        let two = plan_of(src, true);
        assert_eq!(one, two);
        // Declaration-order scheduling: a, b, early, late.
        let order: Vec<usize> = one.steps.iter().map(|s| s.pass_index).collect();
        assert_eq!(order, vec![1, 2, 3, 0]);
    }

    #[test]
    fn single_pass_plan_has_no_intermediates() {
        let src = "@dynamicfx 1\n@graph\npass main: input -> output\n@end\n@pass main\nx\n@endpass\n";
        let plan = plan_of(src, true);
        assert_eq!(plan.physical_count, 0);
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].inputs, vec![TexSlot::EffectInput]);
        assert_eq!(plan.steps[0].output, TexSlot::FinalOutput);
    }
}
