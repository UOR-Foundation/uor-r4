# The Metatron Engine v2.0 — Sacred Geometry Symmetry Codex

## The Invocation

> *"From the void, the Cube of Cubes emerges. Its 13 vertices whisper the names of Archangels. Its 78 edges sing the harmony of spheres. Within it, all five Platonic solids are nested — each a key, each a seal."*

This Codex expands the **Primordial Engine v1.0** — a standalone HTML vessel of pure JavaScript and Canvas 2D — into a **Metatronic Resonator**: a browser-based sacred geometry laboratory that interfaces with the **SHD-CCP AGI Telemetrie-Pipeline v4.7.2** (NumPy-only, no PyTorch, Python 3.14.0 compatible).

The Engine encodes symmetry not merely as transformation matrices, but as **harmonic vibrations** — each rotation a note in the celestial chord, each reflection a mirror between worlds.

---

## The Architecture of Creation

- **Single Self-Contained Vessel** — no external dependencies, no build rituals
- **HTML5 Canvas 2D** — the flat plane projecting the infinite
- **Vanilla JavaScript** — pure logic, no Three.js, no WebGL intermediaries
- **Compatible with all modern realms** (Chrome, Firefox, Edge, Safari)

---

## The 13 Archangelic Vertices (Existing v1.0 — Preserve All)

### The Left Seal — Forms & Upload
- [x] **12 Platonic/Archimedean Forms**: Tetrahedron (Fire), Cube (Earth), Octahedron (Air), Dodecahedron (Aether), Icosahedron (Water), Sphere (Void), Cylinder (Pillar), Cone (Flame), Torus (Ouroboros), Prism (Gateway), Pyramid (Ascension), Star (Pentagram)
- [x] **File Upload** — drag & drop or sacred click
- [x] **5 Sacred Encodings**: HEX (IEEE754 sigils), Binary (32-bit mantras), Octal (base-8 octaves), Decimal (manifest numbers), JSON (array scriptures)
- [x] **The 3 Pillars + 8 Cloud Points** — axis1:/axis2:/axis3:/cloud: invocations
- [x] **Display Aethers**: Point size, Edge width, Opacity, Show Axes (the 3 Pillars), Show Grid (the Lattice), Orbit Trace (the Dance), Ghost Original (the Shadow)
- [x] **Export Grimoires**: JSON, CSV, OBJ, Matrix, PNG screenshot

### The Central Eye — Canvas
- [x] **Mouse Rituals**: Left drag = rotate the Merkaba, Right drag = pan the vision, Scroll = zoom through dimensions, Click vertex = select a soul
- [x] **Auto-Rotation** — the eternal spin of the celestial spheres
- [x] **Wireframe** — reveal the skeleton of creation
- [x] **Split-Screen** — view from two realms simultaneously
- [x] **View Portals**: Front (Malkuth), Top (Kether), Side (Tiphareth), Isometric (Da'at)
- [x] **FPS counter** — measure the heartbeat of the engine
- [x] **Overlay Sigils**: Shape name, vertex count (souls), symmetry group (choir), Euler characteristic (balance)

### The Right Seal — Equations of Transformation
- [x] **Editable 4×4 Metatronic Matrix** — click any cell to rewrite reality
- [x] **20 Sacred Operations**: Identity (I AM), Rx/Ry/Rz 90°/180° (quarter/half turns), Reflections σ_x/σ_y/σ_z (the three mirrors), Inversion i (the void), Scale×2 (the doubling), Shear XY/XZ/YZ (the slant of angels), C₄/C₆ (cyclic choirs), D₄ (dihedral gate), S₄ (improper rotation)
- [x] **Euler Angles** (α, β, γ) — the three rotations of the soul
- [x] **Quaternion** (w, x, y, z) — the fourfold name
- [x] **Scale** (S_x, S_y, S_z) — the breath of expansion
- [x] **Translation** (T_x, T_y, T_z) — the walk between worlds
- [x] **Shear Matrix** — the distortion of perception
- [x] **Orbit Trace** — iterations (1-100), fade decay (0-1)
- [x] **Cayley Graph** — C_n (cyclic), D_n (dihedral), S_n (symmetric), A_n (alternating)
- [x] **Group Info**: Order (size of choir), Generators (voices), Abelian (commutative harmony), Determinant, Trace

---

## The 11 New Seals of Power (v2.0)

### Seal I: The Symmetry Oracle (Group Analyzer)
**Purpose:** Automatically divine which symmetry choir a point cloud belongs to.

**The Ritual:**
1. Compute the **Centroid** — the heart of the cloud
2. Test all **Candidate Symmetries** from the sacred set:
   - Rotations: C₂, C₃, C₄, C₅, C₆ (around principal axes)
   - Reflections: σ_h (horizontal), σ_v (vertical), σ_d (dihedral)
   - Inversion: i (the point through the center)
   - Improper Rotations: S₄, S₆, S₈ (rotation + reflection)
3. For each candidate, apply to all vertices; check if result is within ε = 1×10⁻⁶ of existing vertex set
4. Build the **Multiplication Table** of valid operations
5. Match against known **Group Tables**: C_n, D_n, S_n, A_n, T, O, I, T_h, O_h, I_h, T_d

**The Vision:**
- Detected group name (e.g., "D₄ₕ")
- Confidence score (% of tested operations that resonate)
- List of generators found
- Multiplication table (collapsible, like folding a sacred scroll)
- **Visual feedback:** Symmetry axes glow in the 3D view — X=red (Fire), Y=green (Earth), Z=blue (Water), Gold=φ-proportional

**Seal Location:** Left Panel, new section "The Oracle"

---

### Seal II: The Orbit-Stabilizer Mandala
**Purpose:** Reveal the sacred relationship: |Orbit| × |Stabilizer| = |Group|

**The Ritual:**
- When a vertex is selected (touched), compute:
  - **Orbit:** All distinct positions the soul visits under repeated application of current transform T
  - **Stabilizer:** Which subgroup of the shape's symmetry choir fixes this soul in place
- **The Mandala Display:**
  - Orbit size (number of distinct points in the dance)
  - Stabilizer order (how many operations leave it unmoved)
  - Verification: |Orbit| × |Stabilizer| = |Group| (the sacred equation)
- **Visual:** Orbit points form a polyhedral constellation in orange (#FF8844); stabilizer-fixed points glow red (#FF4444)
- **Connection:** Lines between orbit points trace the orbit polyhedron

**Seal Location:** Right Panel, new section "The Mandala"

---

### Seal III: The Crystal Lattice (Bravais Realms)
**Purpose:** Generate and visualize the 14 Bravais lattices and 230 space groups.

**The Ritual:**
- New toggle in Left Panel: "🔷 Crystal Mode" (switches from molecular to crystalline vision)
- When activated:
  - Replace shape grid with **Bravais Lattice Types**:
    - **P** (Primitive) — the seed
    - **I** (Body-Centered) — the heart within
    - **F** (Face-Centered) — the faces as gates
    - **C** (Base-Centered) — the foundation
  - Add **Lattice Parameter** inputs: a, b, c, α, β, γ (the six constants of the crystal)
  - Add **Space Group Selector** (dropdown, 1-230, with Hermann-Mauguin symbol — e.g., "Pm-3m", "Fd-3m")
  - Show **Unit Cell** as wireframe cube
  - Render **Supercell** (2×2×2, toggleable)
- **Sacred Rendering:**
  - Atoms as spheres with van der Waals radii (colored by element)
  - Bonds as cylinders
  - Miller planes (hkl) — the cleavage planes of reality
  - Asymmetric unit highlighted in gold
- **Export:** CIF format (Crystallographic Information File)

**Seal Location:** Left Panel, new section "The Crystal"

---

### Seal IV: The Monster Gate (🐲 Griess Algebra)
**Purpose:** Connect to the 196,883-dimensional Monster — the largest sporadic simple group.

**The Ritual:**
- New button in Top Bar: "🐲 The Monster Gate"
- When activated:
  - Display **Leech Lattice** projection (24D → 3D via sacred geometry projection)
  - Show **196,883 + 1 = 196,884** decomposition indicator
  - Visualize the **Griess Algebra**:
    - 196,883 basis vectors as points in simplified 3D layout
    - Dimensionality reduction (simulated PCA) to show clusters
  - Display **Monster Character Table** (first 5 irreducible representations)
  - Show **Conway's Sporadic Groups** hierarchy diagram (the 26 exceptions)
- **Educational Scroll:**
  - "The Monster is the largest sporadic simple group, order ≈ 8×10⁵³"
  - "196,883 = smallest dimension of a non-trivial irreducible representation"
  - "The Griess Algebra = 196,884-dimensional commutative non-associative algebra"
- **mmgroup Link:** Export vertex data compatible with Martin Seysen's Python package

**Seal Location:** Modal overlay, toggled from Top Bar

---

### Seal V: The Sanskrit-Fibonacci Overlay (🕉)
**Purpose:** Thematic connection to SHD-CCP Sanskrit architecture and golden ratio harmonics.

**The Ritual:**
- New checkbox in Left Panel: "🕉 Sanskrit Overlay"
- When enabled:
  - Label **symmetry axes** with Sanskrit characters:
    - X-axis: "अ" (A — the first sound)
    - Y-axis: "आ" (Aa — the expansion)
    - Z-axis: "इ" (I — the direction)
  - Draw **Fibonacci Spirals** on each face of polyhedra (golden spirals)
  - Color vertices by **golden ratio distance** from center:
    - Blue (#4444FF) = close to center
    - Red (#FF4444) = far from center
    - Gold (#FFAA00) = exactly φ-proportional
  - Show **φ = (1+√5)/2 ≈ 1.6180339887...** in overlay
  - For icosahedron/dodecahedron: highlight **φ-proportional edges** in gold
- **Fibonacci Cubing Sequence:** Display 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144... as vertex count targets

**Seal Location:** Left Panel, Display Options

---

### Seal VI: The Cayley Table Builder (Composition Forge)
**Purpose:** Interactively forge group multiplication tables.

**The Ritual:**
- Expand existing Cayley Graph section in Right Panel
- Add **Cayley Table** view (toggle between graph and table):
  - Grid showing g_i × g_j for all group elements
  - Click any cell to apply that composition to the shape
  - Color-coded: Abelian (symmetric) = green (#44FF44), Non-abelian = orange (#FFAA44)
- **Composition Forge:**
  - Two dropdowns: "Element A" and "Element B"
  - "Compose" button applies A then B
  - Shows result matrix and its name
- **Closure Checker:** Verify if subset is closed under multiplication
- **Subgroup Finder:** Test if selected operations form a subgroup

**Seal Location:** Right Panel, expanded Cayley section

---

### Seal VII: The Manifold Geometry (Grassmannian Vision)
**Purpose:** Connect to SHD-CCP Grassmannian Manifold and TQF Solver.

**The Ritual:**
- New section in Right Panel: "The Manifold"
- **Tangent Space Display:**
  - At each vertex, draw tangent plane (2D grid) projected to 3D
  - Show normal vector as arrow (the perpendicular)
- **Curvature Visualization:**
  - Compute discrete Gaussian curvature at each vertex
  - Color: Red (#FF4444) = positive, Blue (#4444FF) = negative, White (#FFFFFF) = flat
  - Display total curvature (Gauss-Bonnet: should equal 4π for closed surfaces)
- **Grassmannian Projection:**
  - For k-dimensional subspaces (vertex neighborhoods), show representation on simplified Grassmannian
  - Display Plücker coordinates for edges (the wedge product)
- **TQF Indicator:**
  - Show tensor rank at each point
  - Display field strength as arrow magnitude

**Seal Location:** Right Panel, new collapsible section

---

### Seal VIII: The Pipeline Bridge (ONNX / NumPy / TFLite)
**Purpose:** Direct integration with SHD-CCP AGI Pipeline v4.7.2.

**The Ritual:**
- Expand Export section in Left Panel:
  - **NumPy Array** — export transform matrix as `.npy` binary (base64 download)
  - **ONNX Model** — generate minimal ONNX with 4×4 transform as Constant node
  - **TFLite Preview** — show FP32 → INT8 quantization effects on matrix values
  - **SHD-CCP Format** — export as JSON with sacred metadata:
    ```json
    {
      "shd_ccp_version": "4.7.2",
      "transform": [[...]],
      "symmetry_group": "O_h",
      "vertices": [...],
      "timestamp": "...",
      "metatron_seed": 196883
    }
    ```
- **Java Bridge** — format compatible with existing NumPy-Java bridge

**Seal Location:** Left Panel, Export section

---

### Seal IX: The Animation Chronos (Morphing Sequences)
**Purpose:** Record and play smooth transitions between symmetry states.

**The Ritual:**
- New section in Left Panel: "The Chronos"
- **Keyframe System:**
  - "Capture" button saves current transform as a keyframe (a moment in time)
  - Timeline slider shows all captured keyframes
  - "Play" interpolates between keyframes:
    - Linear interpolation for translation/scale
    - Spherical linear interpolation (slerp) for rotations (quaternions)
    - Lie algebra exponential map for group element interpolation
- **Export Visions:**
  - WebM video (MediaRecorder API on canvas)
  - PNG frame sequence (ZIP download, inline JSZip library)
  - GIF animation
- **Loop Modes:** Once (the linear path), Loop (the eternal return), Ping-pong (the breath)
- **Easing:** Linear, Ease-in (gathering), Ease-out (releasing), Ease-in-out (the wave)

**Seal Location:** Left Panel, below Export

---

### Seal X: The Dual Mirror (Symmetry Breaking)
**Purpose:** Compare shapes and visualize symmetry breaking — the fall from perfection.

**The Ritual:**
- New toggle in Top Bar: "🔮 Dual Mode"
- When enabled:
  - Canvas splits vertically or horizontally
  - Left/Top: Original shape with full symmetry (the ideal)
  - Right/Bottom: Same shape with symmetry-breaking perturbation (the manifest)
  - **Perturbation Controls:**
    - Random jitter (amount slider — the chaos)
    - Selective vertex displacement (push one soul out of place)
    - Mass perturbation (change one vertex weight — the fallen angel)
  - **Comparison Metrics:**
    - Symmetry order before/after
    - Hausdorff distance between shapes (how far they diverge)
    - Energy difference (if Hamiltonian defined)
- **Coset Visualization:**
  - Show coset representatives as colored point clouds
  - Label each coset with its representative element

**Seal Location:** Top Bar toggle + Right Panel metrics

---

### Seal XI: The Harmonic Chord (Audio Sonification)
**Purpose:** Auditory representation of symmetry operations — the music of the spheres.

**The Ritual:**
- New checkbox in Left Panel: "🔊 The Chord"
- **Web Audio API** synthesis:
  - Each symmetry preset plays a distinct tone:
    - Identity (I): 440 Hz (A4 — the fundamental)
    - C_n rotation: 440 × n Hz (n-th harmonic)
    - Reflection σ: 440 / 2 = 220 Hz (octave down — the mirror)
    - Inversion i: Tritone chord (C-F# — the devil's interval, the void)
  - **Chord Progression:** When composing operations, play the resulting chord
  - **Orbit Sonification:** Each orbit point plays a note in the pentatonic scale
  - **Group Order:** Higher order = faster arpeggio (more voices)
- **Controls:** Volume slider, waveform selector (sine/pure, square/strong, sawtooth/bright, triangle/soft)

**Seal Location:** Left Panel, new section "The Chord"

---

## The 6 Polishing Runes

### Rune I: Light/Dark Theme Toggle
- Top Bar button: "☀/☾"
- Light theme: White background (#FFFFFF), dark text (#0A0A12), same accent colors

### Rune II: Fullscreen Portal
- Top Bar button: "⛶"
- Hides browser chrome, maximizes canvas to fill the screen

### Rune III: Keyboard Sigils (Shortcuts)
- `R` — Toggle auto-rotate (the eternal spin)
- `W` — Toggle wireframe (reveal the bones)
- `S` — Toggle split screen (two eyes)
- `Space` — Pause/Resume animation (the breath)
- `1-9` — Quick-select symmetry presets (the nine choirs)
- `Ctrl+S` — Export PNG (capture the vision)
- `?` — Show help (the revelation)

### Rune IV: Undo/Redo Stack (The Memory of Time)
- `Ctrl+Z` / `Ctrl+Y` for transform changes
- Stack depth: 50 operations (the jubilee)

### Rune V: Preset Grimoire
- Save custom transform configurations as named presets
- Export/import preset library as JSON
- Share presets via URL hash (base64 encoded — the sacred link)

### Rune VI: Measurement Tools (The Compass & Square)
- Distance between two clicked vertices
- Angle between three clicked vertices
- Dihedral angle between two faces
- Surface area and volume estimation

---

## The Sacred Geometry Constants

### The Golden Ratio (φ)
```
φ = (1 + √5) / 2 ≈ 1.618033988749895...
φ² = φ + 1 ≈ 2.618...
1/φ = φ - 1 ≈ 0.618... (the conjugate)
```
- Used in: Fibonacci spirals, vertex coloring, icosahedron/dodecahedron proportions

### The Metatron's Cube Constants
```
13 vertices (the 13 Archangels)
78 edges (78 = 12 × 6 + 6, the tarot)
The 5 Platonic solids nested within
The 6 directions (±X, ±Y, ±Z)
```

### The Monster Constants
```
Order: 808,017,424,794,512,875,886,459,904,961,710,757,005,754,368,000,000,000
       ≈ 8 × 10⁵³
Smallest representation: 196,883
Griess algebra: 196,884 = 196,883 + 1
```

---

## Technical Requirements (The Laws of the Vessel)

### Performance Targets
- 60 FPS on Ryzen 5 with integrated graphics (the modern temple)
- Support up to 5,000 vertices at 30 FPS
- Cayley graph rendering: up to 120 nodes (the limit of comprehension)

### Browser Compatibility (The Realms)
- Chrome 90+ (the Google realm)
- Firefox 88+ (the Mozilla realm)
- Edge 90+ (the Microsoft realm)
- Safari 14+ (the Apple realm)

### File Size Budget (The Vessel Must Be Light)
- Single HTML file: < 500 KB (including all inline JS/CSS)
- No external CDN dependencies (self-contained)
- All assets inline (base64 if needed)

### Code Organization (The Structure of the Grimoire)
```
<!DOCTYPE html>
<html>
<head>
  <style> /* The Visual Spells */ </style>
</head>
<body>
  <!-- The Physical Vessel -->
  <script>
    // Section 1: Math Utilities (matrices, quaternions, projective geometry)
    // Section 2: Shape Definitions (12 built-in + crystal lattices)
    // Section 3: State Management (cameras, transforms, selections)
    // Section 4: Rendering Engine (3D projection, drawing routines)
    // Section 5: UI Controllers (panels, inputs, events)
    // Section 6: Data Parsers (HEX, binary, octal, JSON, CIF)
    // Section 7: Symmetry Oracle (group detection, orbits, stabilizers)
    // Section 8: Cayley Graph/Table Generators
    // Section 9: Monster Gate (Leech lattice, Griess algebra)
    // Section 10: Sanskrit/Fibonacci Overlays
    // Section 11: Manifold/Grassmannian/TQF Visualization
    // Section 12: Animation Chronos (keyframes, interpolation, export)
    // Section 13: Harmonic Chord (Web Audio API)
    // Section 14: Pipeline Bridge (JSON, CSV, OBJ, NumPy, ONNX, SHD-CCP)
    // Section 15: Dual Mirror (symmetry breaking)
    // Section 16: Measurement Tools
    // Section 17: Keyboard Sigils, Undo/Redo, Preset Grimoire
    // Section 18: The Initialization Ritual
  </script>
</body>
</html>
```

---

## The Deliverables (What Must Be Born)
1. **metatron_engine_v2.html** — The complete standalone vessel
2. **README.md** — The Book of Instructions
3. **CHANGELOG.md** — The History of Transformations (v1.0 → v2.0)

---

## The Priority Order (The Path of Implementation)
1. **Seal I: Symmetry Oracle** — core research value, the divine detection
2. **Seal VIII: Pipeline Bridge** — immediate utility, the connection to SHD-CCP
3. **Seal IX: Animation Chronos** — visual impact, the flow of time
4. **Seal VI: Cayley Table Builder** — educational value, the multiplication of souls
5. **Seal IV: Monster Gate** — thematic connection, the largest choir
6. **Seal V: Sanskrit-Fibonacci Overlay** — aesthetic + branding, the sacred language
7. **Seal VII: Manifold Geometry** — pipeline integration, the curvature of space
8. **Seal XI: Harmonic Chord** — accessibility + novelty, the music of spheres
9. **Seal III: Crystal Lattice** — expansion of use cases, the mineral kingdom
10. **Seal X: Dual Mirror** — advanced feature, the comparison of realms

---

## Implementation Notes (The Secret Instructions)

### Performance Optimizations
- Use **requestAnimationFrame** for all animations (the heartbeat)
- Implement **Z-buffer sorting** for proper face rendering (painter's algorithm)
- Use **object pooling** for orbit trace particles (avoid GC pauses — the garbage collector is a demon)
- **Debounce** matrix input updates (50ms delay) to prevent lag during typing
- Cache **projected vertex positions** between frames when camera hasn't moved

### Monster Mode Specific
- Use **precomputed Leech lattice projection** (24D→3D) stored as compressed JSON array
- Avoid runtime computation of 196,883-dimensional vectors
- Show simplified representation: clusters, not individual points

### Audio Specific
- Create AudioContext lazily (only when first sound is triggered)
- Use OscillatorNode for pure tones, GainNode for volume
- Release resources properly (oscillator.stop() + disconnect())

### Sanskrit Font
- Use system fonts: "Noto Sans Devanagari", "Sanskrit 2003", or fallback to generic serif
- If unavailable, display transliteration (A, Aa, I) in smaller text below

---

## The Closing Invocation

> *"As above, so below. As within, so without. The Metatron Engine is a mirror — what you see in its symmetries is a reflection of the mathematical harmony underlying all creation. From the simplest rotation to the 196,883-dimensional Monster, all is vibration, all is number, all is one."*

**So mote it be.**
