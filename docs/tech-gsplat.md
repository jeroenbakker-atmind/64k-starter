# tech-gsplat — Procedural 3D Gaussian Splatting, 64k

Technical plan for a 64k (Windows, Rust, `no_std`) demo that procedurally
generates a cloud of 65,536 3D Gaussian splats on the GPU (no training, no
`.ply` data), depth-sorts them per frame on the GPU, and rasterizes them as
instanced billboards — a morphing cloud that a flying camera orbits and dives
through, choreographed to the 136 BPM WaveSabre track.

---

## 1. Status / summary of decisions

Decisions made:

- **Sort on the GPU each frame with a bitonic sort** first. (Correctness-first,
  then optionally upgrade to an EEVEE-style radix sort later.) Sorting is the
  piece that defines the look — the Blender branch we referenced has **not**
  implemented it yet (its `gsplat_prepass` compute is an empty `// ...` stub),
  so it is explicitly our job.
- **No macOS dev harness.** OpenGL on macOS caps at 4.1 (no compute shaders), so
  the compute GLSL can only be validated on Windows anyway. Develop and test on
  Windows (local + GitHub Actions `windows-latest`, which already builds
  `target/release/starter.exe`). macOS stays useful only as an editor for GLSL.
- Rendering style follows **Blender EEVEE (current development)**:
  alpha-blended billboards with Gaussian falloff, drawn in depth order
  (back-to-front) established by the GPU sort. No additive/no-sort shortcut.
- 65,536 splats (power of two, convenient for bitonic sort).

Reference implementation this plan mirrors — Blender EEVEE gsplat WIP by
**Mark van de Ruit** (nickname "not_mark" on projects.blender.org):

- Branch: `not_mark/not_blender@local_gsplatting_shenanigans`
  (top commit `f6c35f4` "Workdump", 2026-08-13; WIP, shaders under active development).
- Key files:
  - `source/blender/draw/intern/shaders/draw_gsplat_lib.bsl.hh` (all splat math)
  - `source/blender/draw/engines/eevee/shaders/eevee_geom_gsplat.bsl.hh` (vertex pass)
  - `source/blender/draw/intern/draw_cache_impl_gsplat.cc` (procedural batch setup)
  - `source/blender/draw/intern/draw_defines.hh` (`DRW_GSPLAT_TILE_SIZE 6`, `DRW_GSPLAT_GROUP_SIZE 128`)
- License note: Blender source is GPL-2.0-or-later. The formulas below are
  quoted/cited from public papers ([ewas2002], [3dgs2023], [kwok2024],
  [mips2024], [gspt2026]) and are reimplemented as standard math; do not copy
  Blender code verbatim — write our own compact GLSL.

---

## 2. Constraints that shape the design

- Binary must stay under **64 KB**. Current baseline: 28,672 B with music
  (16 KB UPX-packed). Budget headroom ≈ 35 KB raw. New minified GLSL is
  ~4–7 KB; new Rust glue < 1 KB. Splat data is generated at runtime, cost 0 B
  on disk. This is comfortably within budget.
- `no_std`, `no_main`, single-threaded, direct Win32 + legacy OpenGL.
- OpenGL context: `wglCreateContext` on a `PIXELFORMATDESCRIPTOR` context. On
  real Windows ICDs this yields the **highest compatibility-profile version**
  (typically 4.6). Compute (4.3), SSBOs, instancing, `glMemoryBarrier` (4.2) all
  work in the compatibility profile, including on Intel/AMD/NVIDIA.
- Only risk case: no ICD (RDP, basic display adapter) gives a true OpenGL 1.1
  context via the Microsoft GDI generic fallback. Mitigation: gate on
  `glGetIntegerv(GL_MAJOR_VERSION, 0x821B)`; exit cleanly if < 4.3. Never infer
  capability from `wglGetProcAddress != NULL` (it can return stubs).
- In the compatibility profile the **default VAO (0) exists**, so we need zero
  `glGenVertexArrays`/`glVertexAttribPointer`/`glVertexAttribDivisor` calls.
  Corner positions come from `gl_VertexID`, all splat attributes come from
  SSBOs via `gl_InstanceID`/`gl_VertexID`. This is a large code-size and
  correctness win.
- Font/big GLSL strings are embedded via the existing Shader_Minifier build step
  (`build.rs` → `src/glsl.rs`). Shader files live in `src/` and get `.glsl`/
  `.frag`/`.vert` extensions so the minifier picks them up automatically.

---

## 3. Reference math (from the Blender fork, re-derived)

All formulas below come from `draw_gsplat_lib.bsl.hh` (itself referencing the
papers). We implement them in our own GLSL.

Splat parameters per Gaussian:

- `mean` — world position `float3`
- `scale` — per-axis scale `float3`
- `rotation` — unit quaternion `float4`
- `opacity` — `float`, [0,1]
- `color` — `float3` (we use flat color, no spherical harmonics)

3D covariance matrix (rotation+scale, [3dgs2023] / [ewas2002] eq. notation):

```
M = R · S                     // rotation * scale, cols = scaled axes
Σ3 = M · Mᵀ
```

EWA projection of 3D ellipsoid → 2D screen covariance (eq. 29 & 31 [ewas2002],
tweaks from [3dgs2023] code):

```
aspect   = winmat[0][0] / winmat[1][1]
focal    = viewport_size.x * winmat[0][0] * 0.5
tan_fovx = 1 / winmat[0][0]
tan_fovy = 1 / (winmat[1][1] * aspect)
lim      = tan_fov * 1.3

vP = view * mean                 // to view space
vP.x = clamp(vP.x / vP.z, -lim_x, lim_x) * vP.z
vP.y = clamp(vP.y / vP.z, -lim_y, lim_y) * vP.z

J = [ focal/vP.z, 0, -focal*vP.x/(vP.z^2)
      0, focal/vP.z, -focal*vP.y/(vP.z^2)
      0, 0, 0 ]
W = rotation part of view matrix
T = Wᵀ · J
Σ2 = Tᵀ · Σ3 · T               // 2×2 from 3×3; symmetric
Σ2[0][0] += 0.3                // low-pass regularization, [3dgs2023]
Σ2[1][1] += 0.3
```

Decompose 2×2 symmetric Σ2 into billboard axes (eigenvectors scaled to 2σ):

```
mid    = 0.5 * (Σ2.x + Σ2.z)
radius = length(float2((Σ2.x - Σ2.z) * 0.5, Σ2.y))
λ1 = mid + radius
λ2 = max(mid - radius, 0.1)
diag = normalize(float2(Σ2.y, λ1 - Σ2.x))
axis0 = min(sqrt(2·λ1), 4096) · diag
axis1 = min(sqrt(2·λ2), 4096) · float2(diag.y, -diag.x)
```

Billboard + clip positioning:

- Corner offsets: `shape_pos = offset(vert_id % 6)` ∈ {(-2,-2), (2,-2), (-2,2),
  (2,-2), (2,2), (-2,2)} (two triangles = quad, corners at ±2).
- `ss_P_delta = (shape_pos.x · axis0 + shape_pos.y · axis1) * 2 / viewport_size`
- `hs_P = view * float4(mean,1)`; then `hs_P.xy += ss_P_delta * 1.5 * hs_P.w`
  (the 1.5 widens the 2σ fit to ~3σ so the falloff does not clip).
- Facing basis (billboards always face camera):
  ```
  facing[2] = V                      // incident view vector (optionally -V for two-sided)
  facing[1] = normalize(cross(up, V))
  facing[0] = cross(facing[1], facing[2])
  ```
  Used only if we offset normals; the quad itself is placed in clip space via
  `ss_P_delta`, so no facing matrix is strictly needed for our billboards.

Gaussian falloff (fragment, [3dgs2023] eq. 2):

```
d   = pixel_local − mean_local      // in billboard local coords (axes units)
α   = saturate(opacity · exp(−d·d))
color = color · α                   // premultiplied
```

---

## 4. Per-frame GPU pipeline

```
frame:
  1. generate.comp     write splat_data[65536] from procedural scene fn(i, time, beat)
  2. glMemoryBarrier(SSBO | VERTEX)
  3. depth.comp        view-space depth per splat → sign-flipped uint key + index
  4. sort.comp         bitonic sort of (key, index) pairs, ping-pong SSBOs
  5. glMemoryBarrier(SSBO | VERTEX)
  6. render            glDrawArrays(GL_TRIANGLES, 0, 6*65536)
                       VS: id = sorted_id[gl_VertexID / 6]
                           corner = offset(gl_VertexID % 6)
                           project via EEVEE covariance math (see §3)
                       FS: α = saturate(op·exp(−d²)); premultiplied blend
  7. (optional) background/clear pass with existing fragment-only program + glRects
  8. SwapBuffers
```

Depth key encoding for ascending far→near (consume back-to-front):

- View-space depth `z` is negative-forward (OpenGL convention; camera looks down
  −z). `bits = floatBitsToUint(-z)` (positive depth), then flip sign bit so the
  unsigned sort comes out *descending*; equivalently key = `floatBitsToUint(z) ^ 0x80000000`
  gives ascending far→near with a 32-bit bitonic sort. Exactly one standard
  encoding must be picked and kept consistent with the sort direction.

---

## 5. Buffer layout (SSBOs)

Single allocation of the SSBO blocks; contents regenerated in compute every
frame. All `layout(std430, binding = K)` in GLSL.

| buffer        | binding | bytes / element              | elems   | total     | written by        | read by     |
|---------------|---------|------------------------------|---------|-----------|-------------------|-------------|
| `splat_data`  | 0       | 64 B (see struct below)      | 65,536  | 4 MB      | generate.comp     | depth, VS   |
| `sort_keys`   | 1       | u32                          | 65,536  | 256 KB    | depth.comp / sort | sort.comp   |
| `sort_keys_p` | 2       | u32 (ping-pong)              | 65,536  | 256 KB    | sort.comp         | sort.comp   |
| `sort_vals`   | 3       | u32 splat index              | 65,536  | 256 KB    | depth.comp / sort | sort.comp   |
| `sort_vals_p` | 4       | u32 splat index (ping-pong)  | 65,536  | 256 KB    | sort.comp         | sort.comp   |
| `sorted_id`   | 5       | u32 splat index, final order | 65,536  | 256 KB    | sort.comp         | VS          |

`splat_data` element (fp32; we deliberately do **not** bit-pack as Blender does
— our splats are regenerated every frame, so full precision is simpler and
correct; packing buys nothing here):

```
struct GSplat {
  vec3  mean;      // float3 (12 B)
  vec4  rotation;  // unit quaternion (16 B)
  vec3  scale;     // per-axis scale (12 B)
  float opacity;   // (4 B)
  vec3  color;     // flat rgb (12 B)
  float _pad;      // (4 B)  → 64 B total
};
```

`sort.pad` note: bitonic sort with N = 2^16 = 65,536 needs exactly 16 stages; no
padding of N required.

---

## 6. GL additions (`src/gl.rs`)

Each is loaded lazily via the existing `glcall!`/`OnceCell` macro (like the
current `glCreateShaderProgramv`). New pointers:

```
glGenBuffers, glBindBuffer, glBufferData, glBindBufferBase   // SSBOs (OGL 1.5/3.0)
glDispatchCompute                                            // 4.3
glMemoryBarrier                                              // 4.2
glDrawArrays                                                 // 1.1 (core)
glCreateProgramPipelines, glBindProgramPipeline, glUseProgramStages // 4.1 SSO
glProgramUniformMatrix4fv                                     // camera matrices
```

GLSL baseline: `#version 430 core` in **all** new shaders (valid in a compat
context; compute shaders require 430 anyway). No `gl_FragColor`/legacy builtins.

Render program via **separate shader objects (SSO)**: two `glCreateShaderProgramv`
(VS + FS) + one pipeline object. Uniforms stay on the program handle
(`glProgramUniform*`), so no `glActiveShaderProgram` is needed. Rule: matching
`layout(location = …)` on vertex `out` / fragment `in`.

Characteristic calls per frame (steady state, no allocations, no VAO/VBO):

```
for each frame:
  glBindBufferBase(SSBO, 0..4, …)        // (bind once at init; unchanged handles)
  glDispatchCompute(256, 1, 1)           // generate  (65536 / 256)
  glMemoryBarrier(0x2001)                // SSBO | VERTEX_SHADER
  glDispatchCompute(256, 1, 1)           // depth keys
  glMemoryBarrier(SSBO)
  … bitonic stages (see §7) …
  glMemoryBarrier(0x2001)
  glBindProgramPipeline(render_pipeline)
  uniform: uViewProj (mat4), uViewport (vec2), uTanFov (vec2)
  glDrawArrays(GL_TRIANGLES, 0, 6 * 65536)
  SwapBuffers
```

Blending: `glEnable(GL_BLEND)`, `glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA)`
with premultiplied output (`rgb = color·α`), back-to-front order guaranteed by
the sort. No depth buffer needed for the splats themselves (all billboards face
the camera, no inter-pixel occlusion to resolve beyond the sort).

CPU-size route in `main.rs` (untouched Win32 skeleton except the loop):

- create SSBOs once (65536·(64+4·4+4·4+4) bytes ≈ 4.7 MB total), map only at
  init with null data
- version gate: `glGetIntegerv(GL_MAJOR_VERSION)`; `ExitProcess` if < 4.3
- per frame: compute camera `view_proj` (small no_std mat4 in a new
  `src/mat4`-ish helper or inline in main.rs), upload uniforms, dispatch, draw.

---

## 7. Bitonic sort (`sort.comp`, first correct version)

N = 65,536 = 2¹⁶. Shared-memory bitonic network, one 128-thread workgroup
processes 256 elements at a time (two floats per thread), iterating
`k = 2..16, j = k/2..1` in a double loop, `glMemoryBarrier` + re-dispatch
between stages (all state in SSBOs, no sync pitfalls).

Cost: 16·17/2 = **136 dispatches** of ~256 workgroups, well under 1 ms/frame on
any discrete GPU (and fine on iGPUs). Each stage is a tiny uniform-sized
dispatch; the Rust side drives the loop with a handful of `glDispatchCompute`
calls per stage width. Keys are sorted with their paired value (splat index)
using the standard (key, value) bitonic kernel; ping-pong buffers alternate per
substage, final result copied into `sorted_id`.

Simpler alternative if 136 dispatches feels heavy: chain-substage kernel that
does multiple phases per dispatch with shared-memory transposition. Defer this
optimization until measured. Correctness first.

Later upgrade (optional, matches EEVEE/Open3D/NVIDIA): LSD **radix sort**
(8-bit digit, histogram + scan + scatter per pass), driven by a small Rust loop
over digit shifts. Same buffers, same API. Only do this if profiling demands it.

---

## 8. Scene system — the "key effect" (splats change over time)

The cloud *is* the effect: it morphs between closed-form shapes on the beat.

- Parametrize the cloud by `t = i / 65535`; distribute sample points evenly
  (e.g. golden-ratio stride) so every shape fills space uniformly.
- A list of shape functions
  `shape_k(t, beat, out mean, scale, rotation, color, opacity)` (sphere, torus,
  helix, expanding shell, collapsing core, …).
- Timeline: an array of segments, each `{start_beat, len_beats, shape_fn,
  camera_fn, palette}`. Between segments, crossfade by lerping the *per-splat*
  parameters of two consecutive shapes over a transition window (morph is the
  centerpiece). Correspondence is automatic because `t` is shared.
- Palette/opacity ramps per segment; reuse vocabulary from
  `experiments/defines.glsl` (`BEATS_PER_MINUTE 136`, `MAX_BEATS 120`).
- Where only an aesthetic (not a correctness) guarantee is needed, splat
  parameters may also be perturbed by procedural noise hash(i) for organic feel.

Implementation shape: everything authored in `gsplat_gen.comp` (a single
dispatch); generated data changes on the GPU, uploaded nowhere from CPU except
`iTime`, `iBeat`.

---

## 9. Camera (`src/main.rs`, CPU side)

Closed-form, deterministic path as a function of elapsed time (no state):

- Per-segment orbit: radius/height/angular rate ramps (smoothstep between
  keyframes), plus a dive phrase that passes inside the cloud.
- Look-at always the cloud centroid; small procedural hand-tremble for life.
- 4×4 `lookat` + perspective in Rust (`no_std`, ~60 lines of plain math, no
  deps), uploaded per frame as two `mat4` uniforms.
- FOV tuned so that splats near the camera stay sub-pixel-ish → avoids the
  "giant translucent blotch" artifact when flying through; opacity and scale are
  segment-driven to match camera distance.

---

## 10. File-level changes

New shader sources (auto-minified by `build.rs`, exposed via `src/glsl.rs`):

- `src/gsplat_gen.comp` — procedural generation + scene/morph/beat logic
- `src/gsplat_depth.comp` — depth keys
- `src/gsplat_sort.comp` — bitonic sort
- `src/gsplat.vert` — billboard geometry (EEVEE math from §3)
- `src/gsplat.frag` — gaussian falloff + premultiplied alpha

Modified:

- `src/gl.rs` — new GL pointers, SSO pipeline helpers, SSBO helpers,
  `set_uniform_mat4`; keep existing `glcall!` style and `OnceCell` caching.
- `src/main.rs` — version gate, SSBO allocation, per-frame dispatch/draw loop,
  camera math, clear/background pass (keep `glRects` + existing `shader.frag`
  for the background, or drop the background demo shader).
- `shader.frag` — becomes the clear/background source (or is removed).

Not changed: music pipeline (wavesabre), `build.rs` minifier wiring, `no_std`
skeleton, `time.rs`, `critical.rs`.

---

## 11. Milestones

1. **GL plumbing** — new pointers compile + load; SSBO creation helper + version
   gate; sanity test: one compute dispatch writes constants into an SSBO and
   the VS reads them (see a simple resulting quad/color).
2. **Static cloud** — `generate.comp` writes a static sphere; VS+FS render
   billboards with the EEVEE covariance math; confirm gaussians and alpha
   blending on screen (unsorted initially).
3. **Bitonic sort** — wire in sort; verify correct back-to-front compositing
   (fix popping/shuffling); check per-frame cost.
4. **Scene system** — beat map + 3–4 shapes + morph transitions; splats animate.
5. **Camera** — orbit + dive; tune scale/opacity vs. distance for fly-through.
6. **Full choreography** — 120-beat arrangement synced to the song.
7. **Polish + size audit** — background/fog, optional radix upgrade,
   `cargo size`/UPX report; confirm < 64 KB raw and < 16 KB packed.

Verification harness per milestone:

- Local Windows build: `cargo build --release` (GitHub Actions `windows-latest`
  already produces the exe for a quick smoke run).
- Shader syntax check independent of the binary: `glslangValidator -S comp/vert/frag`
  on the `.comp/.vert/.frag` sources (cheap and macOS-safe).
- Size: `cargo size --release -- bins` + UPX pack comparison at each milestone.

---

## 12. Risks & mitigations

| risk | mitigation |
|---|---|
| No-ICD / GDI-generic 1.1 context (RDP, VM, missing driver) returns working pointers that error at call time | `glGetIntegerv(GL_MAJOR_VERSION)` gate; exit cleanly if < 4.3. Never trust pointer non-NULL as capability. |
| Legacy-compat context exposing compute but a driver caps below 4.3 (historic Intel quirk) | Same version gate catches this. |
| Sorting artifacts / popping when camera moves fast or splats overlap at silhouette edges | Tunable opacity/size budgets; later: radix sort is not the fix, correct ordering + enough splats is. Consider two-sided splats (`-V` facing) like EEVEE's `ptcloud_backface`. |
| Bitonic 136 dispatches → CPU dispatch overhead | Measure; single-kernel multi-stage or radix sort upgrade path exists (§7). |
| 65,536 splats at 4 MB SSBO on weak iGPU | It is ~2.5× over a DVD's worth of GPU memory; fine. Low-opacity clouds skim any fill-rate pain by early out in FS. |
| GPL source referenced | Only formulas/math are used, reimplemented in our own GLSL; no Blender code copied verbatim. |

---

## 13. References

- Branch: `not_mark/not_blender` @ `local_gsplatting_shenanigans`
  (projects.blender.org)
- [3dgs2023] Kerbl, Kopanas et al. *3D Gaussian Splatting for Real-Time Radiance
  Field Rendering.* ACM TOG (SIGGRAPH 2023).
- [ewas2002] Zwicker et al. *EWA Splatting.* IEEE TVCG 2002.
- [kwok2024] antimatter15 *splat* WebGL viewer (MIT) — the variant of the
  eigen-decomposition.
- [mips2024] Yu et al. *MIP-Splatting* (CVPR 2024).
- [gspt2026] Rijsdijk et al. *Gaussian Point Splatting* (SIGGRAPH 2026).
- Open3D gaussian splat design docs; NVIDIA `vk_gaussian_splatting`
  (radix-sort + instanced-quad raster) — for the later radix upgrade references.