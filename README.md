# R⁴ Tangent Space Router (UOR Framework Standard)

A high-dimensional continuous $R^4$ tangent space router, rebased under the official **Universal Object Reference (UOR) Framework** and **Prism Model** standards. This system replaces traditional transformer MoE gating mechanisms with a stable, zero-allocation coordinate reduction pipeline mapped across the first 512 non-trivial Riemann zeta zeroes.

---

## 📖 Overview

Traditional transformers route token inputs using learned parameter gates, which are computationally expensive and act as a black box. The **R⁴ Prime Router** maps natural language queries onto a 512-dimensional prime-factor frequency manifold. By leveraging the **3/8 Resonance Hashing Law**, the router coordinates and synthesizes reasoning trajectories along geometric paths ($R^4$ tangent space vectors), resulting in near-instant local routing.

With this release, the entire coordination engine is rebased onto the **UOR foundation ontology**, converting the routing mechanism into a formally verifiable, type-safe coordinate reduction pipeline.

---

## ⚡ Core Features

- **Algebraic Shape Constraints**: Mapped query contexts and metrics onto formal UOR shapes (`R4RoutingInput` and `R4RoutingOutput`) inside [lib.rs](file:///Users/adminamn/gemini-dev/rust/uor-r4-wasm-router/src/lib.rs) using the `partition_product!` standard.
- **Formal Coordinate Reduction**: Queries are processed through `uor_foundation::pipeline::run_route` by the `UorR4RouterModel` (implementing `PrismModel`), providing formal type-level checking and verification.
- **Real-Time Attestation Witnesses**: Every route execution outputs a `Grounded` witness containing a cryptographic certificate with the following metrics:
  * **UOR Sigma**: The grounding completion ratio ($\sigma \in [0.0, 1.0]$).
  * **UOR $d_\Delta$**: Metric incompatibility between ring distance and Hamming distance.
  * **UOR Euler**: Nerve Euler characteristic ($\chi$) of the constraints.
  * **UOR Free Sites**: The residual free-site rank.
  * **UOR Stratum**: The two-adic valuation stratum coordinate.
- **Wasm-Optimized Zero Allocation**: Borrowed input lifetimes in `R4RoutingInput` pass query buffers on the stack without heap allocation, maximizing execution speed.
- **Interactive 3D Visualizer**: Real-time projection of coordinates onto the $S^2$ base sphere with Hopf fiber rings ($S^1$) and animated trajectory paths.
- **Continuous Manifold Learning**: Learns dynamically during chats by folding prompt-response pairs back into its local JSON database (`manifold_cache_rust.json`).

---

## 🏗️ Architecture

```mermaid
graph TD
    A[User Prompt / Client Query] -->|HTTP POST| B[main.rs Server Endpoint]
    B -->|Stack Allocate [u8; 640]| C[R4RoutingInput]
    C -->|PrismModel::forward| D[run_route Pipeline]
    D -->|Coordinate Reduction Fold| E[R4RouterAxisImpl]
    E -->|Thread-Local ACTIVE_ROUTER| F[route_query_to_manifold_native]
    F -->|Compute Hopf, QIMC & Eigenvalues| G[RoutingData]
    D -->|Mint Witness| H[Grounded Proof Certificate]
    B -->|Replay Witness Derivation| I[UOR Trace Steps]
    B -->|JSON Response Payload| J[index.html Telemetry Dashboard]
```

---

## 🚀 Getting Started

### Prerequisites

Ensure you have the following installed on your machine:
* **Rust** (MSRV 1.65+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
* **Ollama** (Optional, for local LLM synthesis): Download from [ollama.com](https://ollama.com). Pull the model:
  ```bash
  ollama pull gemma:2b
  ```

### Configuration

The project workspace integrates path dependencies to local standards crates in [Cargo.toml](file:///Users/adminamn/gemini-dev/rust/uor-r4-wasm-router/Cargo.toml):
* `uor-foundation` ([UOR-Framework/foundation](file:///Users/adminamn/gemini-dev/rust/uor-r4-wasm-router/uor_standards/UOR-Framework/foundation))
* `uor-prism` ([prism/crates/uor-prism](file:///Users/adminamn/gemini-dev/rust/uor-r4-wasm-router/uor_standards/prism/crates/uor-prism))
* `uor-addr` ([uor-addr/crates/uor-addr](file:///Users/adminamn/gemini-dev/rust/uor-r4-wasm-router/uor_standards/uor-addr/crates/uor-addr))

### Running the Server

Start the local server target:
```bash
cargo run --release --bin server
```
*The server loads the manifold cache from `manifold_cache_rust.json` and starts listening on **`http://127.0.0.1:8000`**.*

---

## 💻 How to Use the App

1. **Access the Dashboard**: Open your browser and go to `http://127.0.0.1:8000/` (loads [index.html](file:///Users/adminamn/gemini-dev/rust/uor-r4-wasm-router/index.html)).
2. **Select Synthesis Engine**: Choose between **Pure Geometric** (local decoding on the manifold coordinates) or **Ollama (Gemma)** (routed prompt steered by prime coordinates and grounded context sentences).
3. **Submit Queries**: Type prompts in the chat box. On submission:
   * The **3D Trajectory visualizer** will project a white pulse path showing the reasoning coordinate evolution.
   * The **QIMC Panel** will display the active prime, the witness validation state (`Verified`), and formal UOR metrics.
   * The **Cascade Trace Logs** console (bottom-right) will output the official step-by-step UOR reduction events in real time.
4. **Index Knowledge**: Paste textual reference materials in the bottom-left text area and click **Index Manifold** to dynamically inject new knowledge coordinates into the local brain.

---

## 📡 API Reference

### 1. Chat Generation
* **Endpoint**: `/api/chat`
* **Method**: `POST`
* **Payload**:
  ```json
  {
    "text": "dry season aquifer depth in the Gambia",
    "identity": "tenant-alpha",
    "engine": "auto",
    "ollama_url": "http://127.0.0.1:11434",
    "ollama_model": "gemma4:e2b"
  }
  ```
* **Response**: Contains `description`, `metrics` (including `uor` validation struct), `trajectory`, and `uor_trace_steps`.

### 2. System Status
* **Endpoint**: `/api/sysinfo`
* **Method**: `GET`
* **Response**: Baseline metrics, uptime, and UOR validation state for initialization.

### 3. Bulk Indexing
* **Endpoint**: `/api/corpus`
* **Method**: `POST`
* **Payload**:
  ```json
  {
    "corpus": "Full text corpus to index into the manifold...",
    "identity": "tenant-alpha"
  }
  ```

### 4. Database Export / Import
* **Endpoint**: `/api/export` (GET) / `/api/import` (POST)
* **Description**: Extracts or restores the complete router vocabulary, prime products, and sentence manifolds in JSON format.

---

## 🧪 Testing

To run the full suite of unit and compilation tests:
```bash
cargo test
```
