//! The multi-pass envelope grammar v1 (ADR-0018): `@graph` manifest +
//! `@pass` sections inside the reserved `@dynamicfx 1` prefix. Every
//! violation is an `E6 EnvelopeSyntax` with its 1-based source line.

use crate::definition::param::ParamId;

pub const MAX_PASSES: usize = 16;
pub const MAX_INPUTS_PER_PASS: usize = 4;
pub const MAX_INTERMEDIATES: usize = 15;

pub const RES_INPUT: &str = "input";
pub const RES_OUTPUT: &str = "output";
/// ADR-0023 append: the previous frame's final output, readable by any
/// pass, never writable, never a pass or intermediate name.
pub const RES_PREV: &str = "prev";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestPass {
    pub name: String,
    pub inputs: Vec<String>,
    pub output: String,
    /// Manifest line, for diagnostics.
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    /// Manifest order = declaration order (scheduling tie-break, ADR-0020).
    pub passes: Vec<ManifestPass>,
    /// Unescaped pass bodies, parallel to `passes`.
    pub bodies: Vec<String>,
    /// Any pass reads `prev` (ADR-0023): the effect is temporal.
    pub uses_prev: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarError {
    /// 1-based source line.
    pub line: usize,
    pub message: String,
}

fn err(line: usize, message: impl Into<String>) -> GrammarError {
    GrammarError { line, message: message.into() }
}

/// A name valid for passes and resources: the ParamId grammar (ADR-0018 §2).
fn check_name(line: usize, what: &str, name: &str) -> Result<(), GrammarError> {
    ParamId::new(name).map_err(|e| err(line, format!("bad {what} name `{name}`: {e:?}")))?;
    Ok(())
}

/// Parse and validate one complete envelope (the whole committed text,
/// already classified as `@dynamicfx 1` by ADR-0012's classifier).
pub fn parse_envelope(source: &str) -> Result<Envelope, GrammarError> {
    #[derive(PartialEq)]
    enum State {
        BeforeMarker,
        BeforeGraph,
        InGraph,
        Between,
        InPass,
    }

    let mut state = State::BeforeMarker;
    let mut passes: Vec<ManifestPass> = Vec::new();
    let mut sections: Vec<(String, String, usize)> = Vec::new(); // name, body, line
    let mut current_name = String::new();
    let mut current_body = String::new();
    let mut current_line = 0usize;
    let mut saw_graph = false;

    for (index, raw_line) in source.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim_end_matches('\r');
        let trimmed = line.trim();

        if state == State::InPass {
            let stripped = line.trim_start();
            if let Some(after_at) = stripped.strip_prefix('@') {
                if after_at.trim_end() == "endpass" {
                    sections.push((
                        current_name.clone(),
                        std::mem::take(&mut current_body),
                        current_line,
                    ));
                    state = State::Between;
                    continue;
                }
                if after_at.starts_with('@') {
                    // A leading `@@` unescapes to a literal `@` line: keep
                    // the indentation, drop exactly one `@`.
                    let leading_ws = &line[..line.len() - stripped.len()];
                    current_body.push_str(leading_ws);
                    current_body.push_str(after_at);
                    current_body.push('\n');
                    continue;
                }
                return Err(err(
                    line_no,
                    format!(
                        "unknown directive `@{}` inside a pass body (escape a literal line with `@@`)",
                        after_at.split_whitespace().next().unwrap_or("")
                    ),
                ));
            }
            current_body.push_str(line);
            current_body.push('\n');
            continue;
        }

        // Outside pass bodies: blank lines and // comments are ignored.
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        match state {
            State::BeforeMarker => {
                if trimmed.strip_prefix("@dynamicfx").is_some_and(|rest| {
                    rest.trim() == "1"
                }) {
                    state = State::BeforeGraph;
                } else {
                    return Err(err(line_no, "expected the `@dynamicfx 1` marker line"));
                }
            }
            State::BeforeGraph => {
                if trimmed == "@graph" {
                    state = State::InGraph;
                    saw_graph = true;
                } else {
                    return Err(err(line_no, "expected `@graph` before any pass section"));
                }
            }
            State::InGraph => {
                if trimmed == "@end" {
                    state = State::Between;
                    continue;
                }
                let Some(rest) = trimmed.strip_prefix("pass ") else {
                    return Err(err(line_no, "expected `pass <name>: <inputs> -> <out>` or `@end`"));
                };
                let Some((name, io)) = rest.split_once(':') else {
                    return Err(err(line_no, "missing `:` in pass declaration"));
                };
                let Some((ins, out)) = io.split_once("->") else {
                    return Err(err(line_no, "missing `->` in pass declaration"));
                };
                let name = name.trim().to_string();
                check_name(line_no, "pass", &name)?;
                let inputs: Vec<String> =
                    ins.split(',').map(|s| s.trim().to_string()).collect();
                if inputs.iter().any(|i| i.is_empty()) {
                    return Err(err(line_no, "empty input name"));
                }
                if inputs.len() > MAX_INPUTS_PER_PASS {
                    return Err(err(
                        line_no,
                        format!("{} inputs exceed the limit of {MAX_INPUTS_PER_PASS}", inputs.len()),
                    ));
                }
                for input in &inputs {
                    if input != RES_INPUT {
                        check_name(line_no, "resource", input)?;
                    }
                }
                let output = out.trim().to_string();
                if output != RES_OUTPUT {
                    check_name(line_no, "resource", &output)?;
                }
                passes.push(ManifestPass { name, inputs, output, line: line_no });
                if passes.len() > MAX_PASSES {
                    return Err(err(
                        line_no,
                        format!("more than {MAX_PASSES} passes"),
                    ));
                }
            }
            State::Between => {
                let Some(rest) = trimmed.strip_prefix("@pass ") else {
                    return Err(err(line_no, "expected `@pass <name>`"));
                };
                current_name = rest.trim().to_string();
                check_name(line_no, "pass", &current_name)?;
                current_body.clear();
                current_line = line_no;
                state = State::InPass;
            }
            State::InPass => unreachable!(),
        }
    }

    match state {
        State::InPass => return Err(err(source.lines().count(), "unterminated @pass section")),
        State::InGraph => return Err(err(source.lines().count(), "unterminated @graph block")),
        _ => {}
    }
    if !saw_graph {
        return Err(err(1, "missing @graph block"));
    }

    validate_graph(&passes)?;

    // Manifest and section sets must match one-to-one, any order.
    let mut bodies = vec![None; passes.len()];
    for (name, body, line) in sections {
        let Some(position) = passes.iter().position(|p| p.name == name) else {
            return Err(err(line, format!("@pass `{name}` is not declared in @graph")));
        };
        if bodies[position].is_some() {
            return Err(err(line, format!("duplicate @pass section `{name}`")));
        }
        bodies[position] = Some(body);
    }
    for (position, body) in bodies.iter().enumerate() {
        if body.is_none() {
            return Err(err(
                passes[position].line,
                format!("pass `{}` has no @pass section", passes[position].name),
            ));
        }
    }

    let uses_prev = passes.iter().any(|p| p.inputs.iter().any(|i| i == RES_PREV));
    Ok(Envelope { passes, bodies: bodies.into_iter().map(Option::unwrap).collect(), uses_prev })
}

/// The v1 graph rules (ADR-0018 §3).
fn validate_graph(passes: &[ManifestPass]) -> Result<(), GrammarError> {
    if passes.is_empty() {
        return Err(err(1, "the graph declares no passes"));
    }

    // Duplicate pass names.
    for (i, pass) in passes.iter().enumerate() {
        if passes[..i].iter().any(|p| p.name == pass.name) {
            return Err(err(pass.line, format!("duplicate pass `{}`", pass.name)));
        }
    }

    // Reserved-name rules (ADR-0023): `prev` is neither a pass name, an
    // output, nor an intermediate.
    for pass in passes {
        if pass.name == RES_PREV {
            return Err(err(pass.line, "`prev` is reserved and cannot name a pass"));
        }
    }

    // Writers: exactly one per resource; exactly one output writer.
    let mut writers: Vec<(&str, usize)> = Vec::new();
    for pass in passes {
        if pass.output == RES_INPUT {
            return Err(err(pass.line, "`input` is read-only"));
        }
        if pass.output == RES_PREV {
            return Err(err(pass.line, "`prev` cannot be written (it is the previous frame's output)"));
        }
        if writers.iter().any(|(name, _)| *name == pass.output) {
            return Err(err(pass.line, format!("resource `{}` has two writers", pass.output)));
        }
        writers.push((&pass.output, pass.line));
    }
    let output_writers = passes.iter().filter(|p| p.output == RES_OUTPUT).count();
    if output_writers != 1 {
        return Err(err(
            passes[0].line,
            format!("exactly one pass must write `output` (found {output_writers})"),
        ));
    }

    // Reads: output never read; every named input must have a writer;
    // self-reads forbidden.
    for pass in passes {
        for input in &pass.inputs {
            if input == RES_OUTPUT {
                return Err(err(pass.line, "`output` cannot be read"));
            }
            if input == &pass.output {
                return Err(err(pass.line, format!("pass `{}` reads its own output (feedback is M6)", pass.name)));
            }
            if input != RES_INPUT
                && input != RES_PREV
                && !writers.iter().any(|(name, _)| name == input)
            {
                return Err(err(pass.line, format!("input `{input}` has no writer")));
            }
        }
    }

    // Intermediates: count limit; every one read at least once.
    let intermediates: Vec<&(&str, usize)> =
        writers.iter().filter(|(name, _)| *name != RES_OUTPUT).collect();
    if intermediates.len() > MAX_INTERMEDIATES {
        return Err(err(
            passes[0].line,
            format!("more than {MAX_INTERMEDIATES} intermediate resources"),
        ));
    }
    for (name, line) in &writers {
        if *name == RES_OUTPUT {
            continue;
        }
        let read = passes.iter().any(|p| p.inputs.iter().any(|i| i == name));
        if !read {
            return Err(err(*line, format!("intermediate `{name}` is never read")));
        }
    }

    // Acyclicity via Kahn's algorithm; ready passes in declaration order
    // (the ADR-0020 determinism rule shares this order). `prev` is always
    // available: it is last frame's output, not a within-frame dependency.
    let mut scheduled = vec![false; passes.len()];
    let mut available: Vec<&str> = vec![RES_INPUT, RES_PREV];
    for _ in 0..passes.len() {
        let next = passes.iter().position(|p| {
            !scheduled[passes.iter().position(|q| q.name == p.name).unwrap()]
                && p.inputs.iter().all(|i| available.contains(&i.as_str()))
        });
        match next {
            Some(index) => {
                scheduled[index] = true;
                available.push(&passes[index].output);
            }
            None => {
                let stuck = passes
                    .iter()
                    .zip(&scheduled)
                    .find(|(_, done)| !**done)
                    .map(|(p, _)| p)
                    .expect("some pass is unscheduled");
                return Err(err(stuck.line, format!("cycle involving pass `{}`", stuck.name)));
            }
        }
    }

    Ok(())
}

/// Deterministic topological order (ready passes in declaration order),
/// shared by validation and the ExecutionPlan (ADR-0020 §1).
pub fn topological_order(passes: &[ManifestPass]) -> Vec<usize> {
    let mut scheduled = vec![false; passes.len()];
    let mut available: Vec<&str> = vec![RES_INPUT, RES_PREV];
    let mut order = Vec::with_capacity(passes.len());
    while order.len() < passes.len() {
        let next = passes
            .iter()
            .enumerate()
            .position(|(i, p)| {
                !scheduled[i] && p.inputs.iter().all(|r| available.contains(&r.as_str()))
            })
            .expect("validated graphs always schedule");
        scheduled[next] = true;
        available.push(&passes[next].output);
        order.push(next);
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLUR: &str = "@dynamicfx 1\n@graph\npass blur_h: input -> temp\npass blur_v: temp -> output\n@end\n@pass blur_h\nbody one\n@endpass\n@pass blur_v\nbody two\n@endpass\n";

    #[test]
    fn golden_two_pass_parse() {
        let env = parse_envelope(BLUR).unwrap();
        assert_eq!(env.passes.len(), 2);
        assert_eq!(env.passes[0].name, "blur_h");
        assert_eq!(env.passes[0].inputs, vec!["input"]);
        assert_eq!(env.passes[0].output, "temp");
        assert_eq!(env.passes[1].output, "output");
        assert_eq!(env.bodies[0], "body one\n");
        assert_eq!(env.bodies[1], "body two\n");
    }

    #[test]
    fn sections_may_come_in_any_order_and_comments_are_ignored() {
        let src = "@dynamicfx 1\n// header comment\n@graph\n// inside graph\npass a: input -> t\n\npass b: t -> output\n@end\n@pass b\nB\n@endpass\n// between\n@pass a\nA\n@endpass\n";
        let env = parse_envelope(src).unwrap();
        assert_eq!(env.bodies, vec!["A\n".to_string(), "B\n".to_string()]);
    }

    #[test]
    fn escape_rule_round_trips() {
        let src = "@dynamicfx 1\n@graph\npass a: input -> output\n@end\n@pass a\n// @@endpass is literal\n@endpass\n";
        let env = parse_envelope(src).unwrap();
        // Wait: the escape applies to lines whose first non-whitespace is @.
        // `// @@endpass` starts with `/`, so it is plain body text.
        assert_eq!(env.bodies[0], "// @@endpass is literal\n");

        let src2 = "@dynamicfx 1\n@graph\npass a: input -> output\n@end\n@pass a\n  @@endpass literal line\n@endpass\n";
        let env2 = parse_envelope(src2).unwrap();
        assert_eq!(env2.bodies[0], "  @endpass literal line\n");
    }

    fn expect_err(src: &str, needle: &str) -> GrammarError {
        let e = parse_envelope(src).expect_err(needle);
        assert!(e.message.contains(needle), "wanted `{needle}` in {e:?}");
        assert!(e.line >= 1);
        e
    }

    // ADR-0023: `prev` reads mark the envelope temporal; misuse is E6.
    #[test]
    fn prev_input_marks_temporal() {
        let env = parse_envelope(
            "@dynamicfx 1\n@graph\npass sim: input, prev -> output\n@end\n@pass sim\nx\n@endpass\n",
        )
        .unwrap();
        assert!(env.uses_prev);
        assert_eq!(env.passes[0].inputs, vec!["input", "prev"]);

        let plain = parse_envelope(
            "@dynamicfx 1\n@graph\npass a: input -> output\n@end\n@pass a\nx\n@endpass\n",
        )
        .unwrap();
        assert!(!plain.uses_prev);

        // prev may be a pass's only input, and multiple passes may read it.
        let multi = parse_envelope(
            "@dynamicfx 1\n@graph\npass gen: prev -> t\npass mix: t, prev -> output\n@end\n@pass gen\nx\n@endpass\n@pass mix\nx\n@endpass\n",
        )
        .unwrap();
        assert!(multi.uses_prev);
    }

    #[test]
    fn prev_misuse_is_rejected() {
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> prev\npass b: prev -> output\n@end\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n",
            "`prev` cannot be written",
        );
        expect_err(
            "@dynamicfx 1\n@graph\npass prev: input -> output\n@end\n@pass prev\nx\n@endpass\n",
            "reserved",
        );
    }

    #[test]
    fn rule_violations_are_line_numbered_e6_material() {
        // Cycle.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: b_out -> a_out\npass b: a_out -> b_out\npass c: a_out -> output\n@end\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n@pass c\nx\n@endpass\n",
            "cycle",
        );
        // Self-read.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: t -> t\npass b: t -> output\n@end\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n",
            "reads its own output",
        );
        // Two writers.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> t\npass b: input -> t\npass c: t -> output\n@end\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n@pass c\nx\n@endpass\n",
            "two writers",
        );
        // Zero / two output writers.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> t\npass b: t -> t2\n@end\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n",
            "exactly one pass must write",
        );
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> output\npass b: input -> output\n@end\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n",
            "two writers",
        );
        // Output read back.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> output\npass b: output -> t\npass c: t -> out2\n@end\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n@pass c\nx\n@endpass\n",
            "cannot be read",
        );
        // input written.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> input\n@end\n@pass a\nx\n@endpass\n",
            "read-only",
        );
        // Unread intermediate.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> t\npass b: input -> output\n@end\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n",
            "never read",
        );
        // Missing writer.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: ghost -> output\n@end\n@pass a\nx\n@endpass\n",
            "no writer",
        );
        // Manifest/section mismatches, both ways.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> output\n@end\n",
            "has no @pass section",
        );
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> output\n@end\n@pass a\nx\n@endpass\n@pass ghost\nx\n@endpass\n",
            "not declared in @graph",
        );
        // Unknown directive in a body.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> output\n@end\n@pass a\n@weird\n@endpass\n",
            "unknown directive",
        );
        // Unterminated section.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input -> output\n@end\n@pass a\nbody\n",
            "unterminated @pass",
        );
    }

    #[test]
    fn limits_are_enforced_at_the_boundary() {
        // 17 passes.
        let mut src = String::from("@dynamicfx 1\n@graph\n");
        for i in 0..17 {
            let input = if i == 0 { "input".into() } else { format!("t{}", i - 1) };
            let output = if i == 16 { "output".into() } else { format!("t{i}") };
            src.push_str(&format!("pass p{i}: {input} -> {output}\n"));
        }
        src.push_str("@end\n");
        for i in 0..17 {
            src.push_str(&format!("@pass p{i}\nx\n@endpass\n"));
        }
        expect_err(&src, "more than 16 passes");

        // 5 inputs on one pass.
        expect_err(
            "@dynamicfx 1\n@graph\npass a: input, input, input, input, input -> output\n@end\n@pass a\nx\n@endpass\n",
            "exceed the limit",
        );
    }

    #[test]
    fn topological_order_is_declaration_stable() {
        let env = parse_envelope(
            "@dynamicfx 1\n@graph\npass late: early_out -> output\npass a: input -> a_out\npass b: input -> b_out\npass early: a_out, b_out -> early_out\n@end\n@pass late\nx\n@endpass\n@pass a\nx\n@endpass\n@pass b\nx\n@endpass\n@pass early\nx\n@endpass\n",
        )
        .unwrap();
        // Ready passes schedule in declaration order: a, b, early, late.
        assert_eq!(topological_order(&env.passes), vec![1, 2, 3, 0]);
    }
}
