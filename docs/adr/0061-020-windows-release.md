# ADR-0061: Windows 0.2.0 release scope

- Status: Accepted
- Authority: user explicitly deferred issue #12 and requested release of completed work.
- Qualifies ADR-0046's prior release-only boundary; existing Source Undo limitation remains disclosed.

Release Windows 0.2.0 with original-layer coverage and its paired reader,
Liquid Glass source/FFX/application script, RGB hex defaults and percent display.
Close issues #9–#11 through the merged PR. Keep #12 open and explicitly unresolved;
the absence of a local reproduction is not proof of a fix. It no longer blocks
this release by the user's decision.

Windows AE 26.5x89 is the current native acceptance host. New coverage requires
Windows AE 26.5+. Do not claim new acceptance for other hosts. Keep macOS 0.1.1
under its original version; no untested or relabeled macOS 0.2.0 asset is shipped.
The optional editor remains disabled and the withdrawn Siri sample stays withdrawn.

Bump product version and monotonically advance the main PiPL cache generation.
Remap build-machine home and checkout paths in both compiled artifacts.
No runtime/ABI/persistence changes accompany release preparation. Verify the exact
new artifact pair with unchanged reader runtime in AE, retain CPU/native evidence,
scan outgoing source and package content, merge normally, tag the merged commit,
publish assets and fresh-download/hash/CRC-check them. No administrator bypass
is authorized or required by this release decision.
