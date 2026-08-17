# ADR-0036: Single-repository record — the public repository is the whole record

- Status: Accepted
- Date: 2026-08-17
- Owners: DynamicFX project
- Supersedes: the publication clause of [ADR-0027](0027-0.0.1-prerelease-scope.md) (its 0.0.1 host-scope decision stands)
- Related architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)
- Related implementation: this repository; [../../scripts/check_governance.py](../../scripts/check_governance.py)
- Related tests/audits: [TR-M0-001 governance check](../audits/00-governance-check.txt)

## Context

ADR-0027 split the project across two repositories: a curated public tree
(code, README, MIT licence) and a private development repository holding the
governance corpus — ADRs, audits, `TEST_MATRIX.md`, evidence artifacts. It
asserted that "divergence is prevented by releasing from private-repo state
only."

That assertion did not hold. By 2026-08-17 the public tree carried content the
private tree never had (`skills/dynamicfx-shaders/`, a reordered README from
commit `8735b28`), so a one-paragraph README edit had to be written twice, by
hand, in two places — and a wholesale copy from the private tree would have
silently reverted the public-only work.

On 2026-08-17 the user closed the private repository. `dynamicfx-dev` was
archived (private, read-only) after its default branch was repointed from
`main` — stranded on `fde0c8a`, a 2026-08-05 prototype-era merge — to
`codex/stabilize-programmatic-flow`, which carries the entire rewrite record
through `d07755c`.

That left the governance corpus with no writable remote, while
[CLAUDE.md](../../CLAUDE.md) requires every session to read *and update* it.
A record that cannot be written is not a record.

## Decision

1. **One repository.** `github.com/JUNKDOGE-JOE/dynamicfx` is the sole
   writable project record: runtime code, user-facing documentation, and the
   full governance corpus (ADRs, audits, test matrix, implementation status,
   roadmap, evidence artifacts, verification harnesses).

2. **The archive is the historical record, not a working repository.**
   `dynamicfx-dev` is frozen at `d07755c` on branch
   `codex/stabilize-programmatic-flow`. It holds the unredacted history,
   including every commit that ever contained the withheld document in §3.
   Its history is therefore **never** merged into this repository; this
   repository keeps its own fresh history and receives curated content only.

3. **One document is withheld from publication:** the static competitor
   analysis (one file under `docs/`, retained only in the archived record —
   its filename names the vendor, so it is not reproduced here). It
   reproduces a third party's product internals obtained by static analysis
   of a shipping binary. The
   standing instruction is not to publish another vendor's product detail.
   The adopt/defer/reject boundaries that study produced are *decisions of
   this project* and remain published, in `ARCHITECTURE.md` and the ADRs.

4. **Redactions are listed, never silent.** Accepted ADRs are immutable, so
   where a published document cited that study, the public copy drops the
   product's identity and any reproduced internals, keeps the reasoning, and
   carries a visible marker. This is the exhaustive list:

   | Document | What was removed | What remains |
   |---|---|---|
   | [ADR-0013 §3](0013-paramid-grammar-and-pools.md) | vendor name | "the studied competitor's actual pool capacity is not statically determinable" |
   | [ADR-0023 §3](0023-temporal-seek-reset.md) | vendor name and version, decoded PIPL `OutFlags2` value, verbatim flag constants | the structural finding (a shipping competitor's feedback is a per-frame loop-carry construct with no cross-frame state) and the conclusion drawn from it |
   | [audit 00](../audits/00-architecture-contract.md) | evidence-table link to the withheld file | the row, pointing here |
   | [audit 07](../audits/07-performance-mfr.md) | vendor name in one parenthetical | "(competitor study)" |
   | [ARCHITECTURE.md](../ARCHITECTURE.md) | link to the withheld file | the statement that the boundaries live in the architecture and ADRs |
   | [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) | link to the withheld file | same |

   ADR-0014 §"alternatives" and ADR-0025 §Context already said "the reference
   competitor" without naming anyone and are published unchanged.

5. **Publication boundary.** Nothing in the following may be committed to this
   repository: the withheld document or any other reproduction of a third
   party's product internals; credentials, tokens, or auth material; machine-
   local paths or host identifiers beyond what evidence records require. A
   leak scan for these terms precedes every push that adds documents.

6. **Evidence artifacts are published as-is.** The PNG/PSD/log artifacts that
   `TEST_MATRIX.md` cites (~3.4 MB) ship with the record. A `PASS` whose
   artifact is unreachable is a claim, not evidence — publishing the matrix
   without them would degrade every result to `CLAIMED_UNVERIFIED`.

## Alternatives considered

- **Keep the two-repository split.** Rejected: it is the arrangement that
  produced the divergence this ADR corrects, and one half is now archived.
- **Publish everything, including the competitor analysis.** Rejected: it
  contradicts a standing instruction and publishes another vendor's internals.
- **Withhold every ADR that cites the study.** Rejected: ADRs cross-reference
  densely; removing four of them breaks the numbered record, the index, and
  the link check, and deletes decisions that are ours, not the competitor's.
- **Silently reword the citing passages.** Rejected: CLAUDE.md forbids editing
  an Accepted ADR to make history look consistent. Hence §4's visible markers.
- **Unarchive `dynamicfx-dev` and continue as before.** Rejected by the user;
  it also leaves the divergence unfixed.

## Consequences

### Benefits

- One tree to read, edit, and verify. `docs/` updates in place; no manual
  double-apply, no drift between two records.
- `scripts/check_governance.py` now covers the whole published surface.
- A future session satisfies CLAUDE.md's required reading order from one
  clone, with no access to a private repository.

### Costs and risks

- **The published ADR text is no longer byte-identical to the historical
  record.** The six redactions in §4 are marked, but the unredacted originals
  live in a *private* archive: an outside reader cannot verify them. This is
  a deliberate trade of external verifiability for a third party's privacy.
- Verification harnesses (`scripts/m1`–`m7`, `scripts/f003`, `scripts/spike`)
  and ~3.4 MB of evidence binaries become public surface. They were written
  as internal tooling and are not documented for outside use.
- Internal governance — including recorded failures, blockers, and residual
  risks — is now visible to anyone. That is the intended cost of a public
  record; it is not a reason to soften future audits.
- The local working copy at the old private path still points at the archived
  remote and still contains the withheld document in its history. It must not
  be pushed here.

## Revisit conditions

- A licensing, legal, or vendor request that changes what may be published.
- Evidence artifacts growing to a size where the repository becomes
  impractical to clone (revisit as artifact storage, not as secrecy).
- A decision to accept outside contributions, which would need a contribution
  policy this ADR does not define.

## Verification obligations

- `python scripts/check_governance.py` returns `RESULT=PASS` in this tree,
  with every link resolving after the withheld file's removal.
- A leak scan over the tree for the withheld vendor's name and for token or
  credential patterns returns nothing, recorded before the publishing commit.
