# ADR-0060: Color default arity and percent display

- Status: Accepted
- Authority: user request to resolve public issues #10 and #11.
- Refines ADR-0026 default decoding; existing vec4 six-digit defaults stay opaque.

Six-digit hex defaults retain three RGB components until uniform reflection.
A vec3 consumes RGB; a vec4 supplies implicit alpha 1. Eight-digit defaults
retain explicit alpha and are rejected for vec3, avoiding silent alpha loss.
The declared GLSL/WGSL uniform type determines the type, not literal length.

Add `hint:percent` for float/f32 members. It sets AE's percent display flag on
the existing Float pool slot. Values, defaults, ranges, expressions and uploaded
uniforms stay raw: 25 means 25%, not 0.25. Authors use min:0 max:100 when desired.
Other types reject the hint. Removing it or reusing the slot clears the flag.
Percent is presentation metadata, rederived from Source after reopen. No new
parameter index, binding kind, persistent field or upload encoding is introduced.

Verify both language frontends, old vec4 defaults, explicit-alpha rejection,
slot/keyframe preservation, flag removal and native save/reopen/render behavior.
