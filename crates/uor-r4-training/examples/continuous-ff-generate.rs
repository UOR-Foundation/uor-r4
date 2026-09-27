//! Continuous Floating-Point (FF) Parent Inference Runner
//!
//! Executes greedy autoregressive token generation with RAYON_NUM_THREADS=1
//! on the continuous floating-point recurrent parent checkpoint.

use candle_core::Device;
use std::path::PathBuf;
use std::time::Instant;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::joint_evaluation;
use uor_r4_training::joint_model::{JointModel, ReadMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut checkpoint_dir = PathBuf::from(
        "/Users/casey.allard/uor-r4-investigations/joint-recurrent-20260924/fit256-quaternion-3/checkpoint-final",
    );
    let mut tokenizer_path = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1/tokenizer.json",
    );
    let mut prompt = String::from("Once upon a time there was a little girl who");
    let mut total_tokens = 128usize;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--checkpoint" => {
                idx += 1;
                checkpoint_dir = PathBuf::from(&args[idx]);
            }
            "--tokenizer" => {
                idx += 1;
                tokenizer_path = PathBuf::from(&args[idx]);
            }
            "-p" | "--prompt" => {
                idx += 1;
                prompt = args[idx].clone();
            }
            "-n" | "--tokens" => {
                idx += 1;
                total_tokens = args[idx].parse()?;
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                std::process::exit(1);
            }
        }
        idx += 1;
    }

    let model = JointModel::load(&checkpoint_dir, &Device::Cpu)?;
    let tokenizer_bytes = std::fs::read(&tokenizer_path)?;
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| std::io::Error::other("failed to load tokenizer"))?;

    let start_time = Instant::now();
    let mut generated_so_far = 0;
    let block_size = 128.min(total_tokens);
    let num_blocks = (total_tokens + block_size - 1) / block_size;

    for _ in 0..num_blocks {
        let n_tokens = (total_tokens - generated_so_far).min(block_size);
        let block_start = Instant::now();
        let generation = joint_evaluation::generate(
            &model,
            &tokenizer,
            &prompt,
            ReadMode::Enabled,
            None, // greedy
            n_tokens,
        )?;
        let block_elapsed = block_start.elapsed().as_secs_f64();
        let tok_count = generation.generated_token_ids.len();
        generated_so_far += tok_count;
        println!("{}", generation.response_text);
        let tok_per_sec = if block_elapsed > 0.0 {
            tok_count as f64 / block_elapsed
        } else {
            0.0
        };
        let ms_per_tok = if tok_count > 0 {
            (block_elapsed * 1000.0) / tok_count as f64
        } else {
            0.0
        };
        println!(
            "[telemetry] Generated {} tokens in {:.2}s ({:.1} tok/s, {:.3} ms/tok); stop={:?}",
            tok_count, block_elapsed, tok_per_sec, ms_per_tok, generation.stop
        );
    }

    let total_elapsed = start_time.elapsed().as_secs_f64();
    let total_tok_per_sec = if total_elapsed > 0.0 {
        generated_so_far as f64 / total_elapsed
    } else {
        0.0
    };
    let total_ms_per_tok = if generated_so_far > 0 {
        (total_elapsed * 1000.0) / generated_so_far as f64
    } else {
        0.0
    };
    eprintln!(
        "[summary] Total generated: {} tokens in {:.2}s ({:.1} tok/s, {:.3} ms/tok)",
        generated_so_far, total_elapsed, total_tok_per_sec, total_ms_per_tok
    );

    Ok(())
}
