# ADR-0046: 0.1.1 Windows patch release boundary

- Status: Accepted
- Date: 2026-09-09
- Authorization: the user requested pushing the completed source updates, publishing 0.1.1, adding IOS27Siri as a repository sample and updating README.
- Qualifies: [ADR-0045](0045-010-source-undo-release-boundary.md) for this explicitly requested patch release only.
- Related: [ADR-0044](0044-wgsl-and-010-release.md), [ADR-0036](0036-single-repository-record.md).

## Decision

Publish the completed Windows compiler-error containment repair and continuous
IOS27Siri sample as 0.1.1. Source-expression Undo remains a disclosed failure;
the user's request to release the current work does not authorize claiming its
separate architectural repair is complete. Carry its warning into README,
sample documentation and release notes. This is a bounded release decision,
not an exemption for future releases or a change to source authority.

Ship a Windows x64 AEX, dependency notices, exact dependency lock and checksums.
No Mac host is available for this patch: keep the existing verified 0.1.0 Mac
asset available under its original tag, and do not relabel it 0.1.1.

The installed repair candidate already passed native AE 2026 valid-source,
source replacement and save/reopen checks. The patch release changes only
Cargo/PE version, PiPL bugversion and compile-only example tests relative to
that runtime. Re-run CPU suites and real-DX12 compiler-failure recovery on the
release sources, and statically verify the new AEX. Document this narrow
runtime-code equivalence; newly versioned bytes are not described as newly
installed or independently aerender-tested. Installing or restarting the
user's active AE is outside this publication request.

## Consequences and verification

No source envelope, Shader ABI, parameter topology, token or persistence
format changes. E58 appends a runtime diagnostic; the optional editor stays
off. The historical native compiler fault-injection block and Source Undo
failure remain visible. Other AE years, exact new-byte native installation,
and 0.1.1 macOS acceptance remain NOT_RUN.

Package only the current sample dependencies. Test project relocation,
embedded source identity, complete native preview frames and original-project
preservation. Publish the necessary failure and recovery results with the source, scan
publication inputs, verify uploaded asset hashes, and record the outcome in [the release audit](../audits/09-windows-011.md).

## Public payload scope

The user requested source, README, a current Sample and a release. Automatic
review rejected publishing the larger local development history with raw
audits, machine paths and archived assets. The release therefore starts from
public main and contains only the requested deliverables and necessary test
logs/results. This narrowly qualifies ADR-0036 for these excluded development
archives: they remain intact on the unpublished local branch; they are neither
claimed as remotely available evidence nor silently deleted. New public logs
mark checkout-path redactions and record original and published hashes. No
third-party internals or credentials are published. This does not create an
unrecorded PASS for an omitted test or exempt future releases from governance.
