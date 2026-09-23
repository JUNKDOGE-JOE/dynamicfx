# ADR-0059: Licensed upstream Liquid Glass material

- Status: Accepted
- Authority: user selection of the proposed GitHub implementation and request to port it.
- Extends [ADR-0036](0036-single-repository-record.md) publication provenance.

Adapt the publicly MIT-licensed shader from `iyinchao/liquid-glass-studio`,
commit `f7b28c36305a862f5cffed3ddd51511cf1204f56`. Retain source attribution,
license texts and a modification record, including its credited color library.
This explicitly authorized open-source reuse is distinct from the unpublished
third-party product reverse-engineering excluded by ADR-0036.

Use native coverage for geometry. A bounded raster distance transform supplies
distance and normals to the upstream optical/light formulas. Original alpha is
not thresholded for compositing; AE still applies it once. Keep field precision
through packed float intermediates at every bit depth. Adapt coordinates,
parameters and premultiplied-alpha handling to the existing shader ABI.

This changes the material only: no native parameter, persistence, geometry
resource or compatibility contract is added. Verify distance against a CPU
reference and verify real AE output before treating the new material as accepted.
