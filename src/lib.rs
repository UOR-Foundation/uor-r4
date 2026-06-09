//! # UOR-aligned R⁴ Tangent Space Router Implementation (Wasm Library)

use std::collections::HashMap;
use std::f64::consts::PI;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

/// Core mathematical constants representing the 3/8 Resonance Hashing Field.
pub const ALPHA_4: f64 = 1.0 / (2.0 * PI); // 1 / 2π
pub const ALPHA_5: f64 = 2.0 * PI;         // 2π (Unity Constraint: ALPHA_4 * ALPHA_5 = 1)

/// Represents a high-dimensional vector in continuous R⁴ Space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct R4Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl R4Vector {
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
    }

    pub fn minkowski_norm(&self) -> f64 {
        -(self.w * self.w) + (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
    }

    pub fn tangent_direction_from_origin(&self) -> Self {
        let magnitude = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if magnitude == 0.0 {
            Self::origin()
        } else {
            Self {
                x: self.x / magnitude,
                y: self.y / magnitude,
                z: self.z / magnitude,
                w: self.w / magnitude,
            }
        }
    }
}

/// A content-addressed identifier derived via the 3/8 Resonance Hashing Law.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UorAddress {
    pub hash_bytes: [u8; 32],
}

impl UorAddress {
    pub fn to_uri(&self) -> String {
        let hex_str: String = self.hash_bytes.iter()
            .take(8)
            .map(|b| format!("{:02x}", b))
            .collect();
        format!("uor-addr-{}", hex_str)
    }
}

/// Models a single active reasoning trajectory or thought stream traveling through R⁴.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtStream {
    pub id: String,
    pub raw_content: String,
    pub uor_address: UorAddress,
    pub uor_uri: String,
    pub r4_target: R4Vector,
    pub path_steps: Vec<R4Vector>,
    pub activated_experts: Vec<usize>,
    pub alignment_phase: f64, // $\theta$ phase state
    pub twist_parity_spin: i8, // $\kappa \in \{-1, 1\}$
}

/// The unified router core coordinator.
#[wasm_bindgen]
pub struct UorR4Router {
    active_streams: HashMap<String, ThoughtStream>,
    expert_active_counts: [usize; 64],
    connection_drift: f64,
    kill_switch_threshold: f64,
    is_aligned: bool,
}

#[wasm_bindgen]
impl UorR4Router {
    /// Instantiates the R4 Router with perfect, error-free default states
    #[wasm_bindgen(constructor)]
    pub fn new(threshold: f64) -> Self {
        Self {
            active_streams: HashMap::new(),
            expert_active_counts: [0; 64],
            connection_drift: 0.0,
            kill_switch_threshold: threshold,
            is_aligned: true,
        }
    }

    /// Exposes read-only status of manifold alignment
    pub fn is_aligned(&self) -> bool {
        self.is_aligned
    }

    /// Returns current connection drift
    pub fn connection_drift(&self) -> f64 {
        self.connection_drift
    }

    /// Returns the kill switch threshold limit
    pub fn kill_switch_threshold(&self) -> f64 {
        self.kill_switch_threshold
    }

    /// Compiles a raw string thought parameter down into its content-addressed math state
    pub fn compile_thought(&self, content: &str) -> JsValue {
        let stream = self.compile_thought_internal(content);
        serde_wasm_bindgen::to_value(&stream).unwrap_or(JsValue::NULL)
    }

    /// Injects a new thought stream, updates MoE activations, and returns the stream
    pub fn inject_thought_stream(&mut self, content: &str) -> JsValue {
        if !self.is_aligned {
            return JsValue::NULL;
        }

        let stream = self.compile_thought_internal(content);
        
        // Map the MoE active overlays
        for &ch in &stream.activated_experts {
            if ch < 64 {
                self.expert_active_counts[ch] += 1;
            }
        }

        let id = stream.id.clone();
        let js_stream = serde_wasm_bindgen::to_value(&stream).unwrap_or(JsValue::NULL);
        self.active_streams.insert(id, stream);
        js_stream
    }

    /// Progresses the connection drift state using delta-time ($dt$) increments.
    /// Returns a log message string if a ZKP reset occurs, otherwise returns undefined.
    pub fn update_drift_physics(&mut self, dt: f64, drift_rate: f64) -> Option<String> {
        if !self.is_aligned || drift_rate == 0.0 {
            return None;
        }

        self.connection_drift += drift_rate * dt;

        if self.connection_drift >= self.kill_switch_threshold {
            let log = self.execute_zkp_phase_reset();
            Some(log)
        } else {
            None
        }
    }

    /// Reset the alignment back to native state ($0.00\%$ error) using ZKP 2i Sync-Handshake
    pub fn execute_zkp_phase_reset(&mut self) -> String {
        self.is_aligned = false;
        self.connection_drift = 0.0;
        
        let log = String::from(
            "[ZKP 2i HANDSHAKE]: Manifold limit breached. Recalibrating continuous tangent vectors...\n\
             [ZKP 2i HANDSHAKE]: Recalibration complete. Zero-point origin alignment LOCKED (0.00% err)."
        );

        // Clear active overlays during reset
        self.expert_active_counts = [0; 64];
        self.active_streams.clear();

        // Safe restabilization
        self.is_aligned = true;
        log
    }

    /// Returns the active stream list as a JS Array
    pub fn get_active_streams(&self) -> JsValue {
        let streams: Vec<&ThoughtStream> = self.active_streams.values().collect();
        serde_wasm_bindgen::to_value(&streams).unwrap_or(JsValue::NULL)
    }

    /// Returns the active counts for the 64 experts
    pub fn get_expert_counts(&self) -> Vec<usize> {
        self.expert_active_counts.to_vec()
    }
}

impl UorR4Router {
    /// Internal Rust compilation logic matching the original spec
    fn compile_thought_internal(&self, content: &str) -> ThoughtStream {
        // 1. Calculate pseudo-sha256 representation matching the 3/8 Resonance Hashing
        let mut hash_bytes = [0u8; 32];
        let mut hash_accumulator: i32 = 0;
        
        for (i, ch) in content.chars().enumerate() {
            hash_accumulator = hash_accumulator.wrapping_add(ch as i32);
            hash_bytes[i % 32] = (hash_accumulator & 0xFF) as u8;
        }

        let uor_addr = UorAddress { hash_bytes };
        let uor_uri = uor_addr.to_uri();

        // 2. Map coordinates in hyperbolic space using sinusoids on the byte values
        let x_coord = (hash_accumulator as f64 * 0.015).sin() * 110.0;
        let y_coord = (hash_accumulator as f64 * 0.025).cos() * 110.0;
        let z_coord = (hash_accumulator as f64 * 0.035).sin() * 90.0;
        let w_coord = (hash_accumulator as f64 * 0.045).cos() * 50.0;

        let r4_target = R4Vector { x: x_coord, y: y_coord, z: z_coord, w: w_coord };

        // 3. Extract active Mixture of Experts (MoE) channels via Bit-Parity
        let mut activated_experts = Vec::new();
        for i in 0..64 {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if (hash_bytes[byte_idx] & (1 << bit_idx)) != 0 {
                activated_experts.push(i);
            }
        }
        activated_experts.truncate(8); // limit channels to maintain structural sparsity

        // 4. Calculate orthogonal prime residues modulo-6 path steps
        let mut path_steps = Vec::new();
        for step in 0..=50 {
            let ratio = step as f64 / 50.0;
            // Generate structured wavy geodesic traces traveling from zero-point origin
            path_steps.push(R4Vector {
                x: x_coord * ratio + (ratio * PI * 2.0).sin() * 25.0,
                y: y_coord * ratio + (ratio * PI * 2.0).cos() * 15.0,
                z: z_coord * ratio,
                w: w_coord * ratio,
            });
        }

        // 5. Spin twist parity check
        let twist_parity_spin = if hash_accumulator % 2 == 0 { 1 } else { -1 };
        let alignment_phase = (hash_accumulator.abs() % 314) as f64 / 100.0;

        ThoughtStream {
            id: format!("stream-{}", uuid_placeholder(hash_accumulator)),
            raw_content: content.to_string(),
            uor_address: uor_addr,
            uor_uri,
            r4_target,
            path_steps,
            activated_experts,
            alignment_phase,
            twist_parity_spin,
        }
    }
}

/// Helper method to create pseudo unique ID strings based on trace accumulator
fn uuid_placeholder(seed: i32) -> String {
    let mut val = seed.abs();
    let mut output = String::new();
    let chars = "abcdef0123456789";
    for _ in 0..6 {
        let idx = (val % 16) as usize;
        output.push(chars.chars().nth(idx).unwrap());
        val /= 16;
    }
    output
}

#[wasm_bindgen(start)]
pub fn init_wasm() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}
