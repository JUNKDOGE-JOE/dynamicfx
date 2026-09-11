# Windows 0.1.1 archive refresh

PASS, 2026-09-09. The user explicitly requested replacing the GitHub v0.1.1
Windows AEX package while keeping the version. The final project-open repair
from TR-OPEN-001 is now in that existing release. No source commit, push or tag
rewrite was performed. Mac and Sample asset IDs, bytes, hashes and metadata
were preserved.

The package contains the exact previously installed and host-verified AEX,
SHA-256 `c91db8c0435ccf90e0fac33f5473825c29bbd959cfeef8cc5fcebe80092b0d0a`.
Its 33 frozen source-input hashes still match. AMD64, PE32+ DLL, DynamicFxMain,
EffectMain and PluginDataEntryFunction2 exports were rechecked. Existing
230/230 default/editor results and AE 2026 project-open/native-render acceptance
apply to these same bytes; they were not rerun for packaging. Other AE years,
macOS acceptance of this repair and Source Undo remain outside this result.

Only three ZIP members changed: DynamicFx.aex, INSTALL.txt and
build/source-identity.json. The other 270 members, including the exact Cargo
lock and dependency notices, are byte-identical. The build identity records
the working-tree repair beyond the unchanged source tag rather than claiming
the binary is built from that tag alone. No production project, shader,
private raw evidence or local archive history was uploaded.

The release ZIP and SHA256SUMS.txt were replaced with `gh release upload
v0.1.1 ... --clobber`. The existing release notes were corrected for the refreshed
Windows artifact with `gh release edit ... --notes-file`; release name, tag,
publication time and flags were preserved. Only the Windows ZIP and Windows
AEX checksum lines changed. Both uploaded files were downloaded again with
`gh release download`, compared byte-for-byte with staging, checked against
GitHub asset digests and unpacked for AEX hash, install hash, manifest and ZIP
CRC verification. An initial strict release-note byte comparison rejected
GitHub's CRLF normalization; content comparison after normalizing CRLF passed.
No upload retry was needed.

[Machine-readable verification](published-verification.json) records all
final hashes and the unchanged tag object. Original downloaded Windows ZIP
and checksum files are retained in the task workspace for rollback.

Next action: no release work remains in this scope. Further source publication,
host acceptance or Source Undo work requires separately authorized scope.
