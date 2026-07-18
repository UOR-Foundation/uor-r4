//! transformerless — cross-compilation of a transformer LM into a
//! multiplication-free, table-native, certifiable inference artifact.
//!
//! Read docs/TRANSFORMERLESS.md (the extrapolation) and docs/PROOF.md (the
//! proof structure and the measured certificate) alongside this code.
//!
//! # Commands
//!
//!   setup            print the external prerequisite commands
//!   gen [secs] [target]   generate/extend the teacher-labeled corpus
//!                         (resumable; whole-story chunking keeps the
//!                         stream deterministic under any chunking)
//!   certify          compile the source, build the store, and print the
//!                    full equivalence certificate and op census
//!
//! # The claim, precisely
//!
//! COMPILER (offline, once): multiplication permitted; every output frozen
//! and blake3-κ-pinned. RUNTIME (per token): every arithmetic operation
//! goes through `OpKernel`, whose complete method set is add / shift / xor /
//! compare / table-read — multiplication is absent from the interface, and
//! the census printed by `certify` measures the ops actually used. The
//! CERTIFIER is instrumentation and may use anything; it never runs at
//! inference. The source-architecture interface is two surfaces (embedding
//! table + next-token oracle); this crate ships the llama-family adapter,
//! and qwen/phi-class sources differ only in that adapter.





use uor_tless::{certify, compare, compiler, runtime, scenarios, teacher::LlamaOracle};

const DEFAULT_CHECKPOINT: &str = "/tmp/ref/out/model.bin";
const STORE_PATH: &str = "/tmp/tless_store.bin";

fn setup() {
    println!(
        "\
external prerequisites (network domains: github.com, raw.githubusercontent.com):

# source checkpoint AND tokenizer: stories15M inside the APE zip payload of
# the trholding/llama2.c release asset (source κ must pin to blake3:0ae73395…;
# tokenizer.bin is required by `scenarios`)
curl -sL -o /tmp/run.com https://github.com/trholding/llama2.c/releases/download/experimental/run.com
cd /tmp && unzip -o run.com out/model.bin tokenizer.bin -d ref

# real out-of-domain text for the scenario suite (public domain)
curl -sL https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt -o /tmp/corpus.txt

pipeline:
  uor-tless gen 1500 150000    # repeat until 'done=1'
  uor-tless certify            # compile + store + certificate + census
  uor-tless compare            # runtime comparison (docs/COMPARISON.md)
  uor-tless scenarios          # scenario suite (needs tokenizer + corpus.txt)"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("setup") => setup(),
        Some("gen") => {
            let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(300);
            let target: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(150_000);
            let mut oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            compiler::generate(&mut oracle, secs, target);
        }
        Some("certify") => {
            let oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            certify::certify(&oracle);
        }
        Some("compile") => {
            let c = compiler::load_corpus().expect("corpus incomplete: run gen first");
            let oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            let art = compiler::compile(&oracle, &c);
            compiler::save_artifacts(&art);
        }
        Some("store") => {
            let c = compiler::load_corpus().expect("corpus incomplete: run `uor-tless gen` first");
            let art = compiler::load_artifacts().expect("run `uor-tless compile` first");
            let (store, _) = runtime::build_store(&art, &c);
            let bytes = runtime::store_bytes(&store);
            std::fs::write(STORE_PATH, &bytes).unwrap();
            println!(
                "store saved: {} ({} bytes, κ {})",
                STORE_PATH,
                bytes.len(),
                runtime::store_kappa(&store)
            );
        }
        Some("compare") => {
            let mut oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            compare::compare(&mut oracle);
        }
        Some("scenarios") => {
            let mut oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            scenarios::scenarios(&mut oracle);
        }
        Some("teacher-kappa") => match std::fs::read(DEFAULT_CHECKPOINT) {
            Ok(b) => println!("source κ: blake3:{} ({} bytes)", blake3::hash(&b).to_hex(), b.len()),
            Err(_) => println!("source checkpoint not found; see `setup`"),
        },
        _ => {
            println!(
                "uor-tless — cross-compile a transformer into a mul-free table artifact\n\
                 commands: setup | gen [secs] [target] | compile | store | certify | compare | scenarios | teacher-kappa\n\
                 docs: docs/TRANSFORMERLESS.md (extrapolation), docs/PROOF.md (proof + certificate)"
            );
        }
    }
}
