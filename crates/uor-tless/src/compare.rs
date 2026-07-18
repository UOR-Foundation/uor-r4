//! The COMPARATOR: measured, same-machine, same-source throughput
//! comparison, quality attached to every row. What it times in-process:
//! the shipped runtime's plain path (word-popcount form of the kernel
//! functions, equality witnessed by `certify`), the fully kernel-counted
//! path (accounting overhead bound), and the in-crate teacher. External
//! classical runtimes are recorded with versions and reproduction commands
//! in docs/COMPARISON.md.

use crate::compiler::{self, Compiled};
use crate::runtime::{build_store, code_plain, derive_rotations, predict_plain, Runtime};
use crate::teacher::TeacherOracle;

pub fn compare(oracle: &mut dyn TeacherOracle) {
    let c = compiler::load_corpus().expect("corpus incomplete: run `uor-tless gen` first");
    let art: Compiled = match compiler::load_artifacts() {
        Some(a) => {
            println!("compiled artifacts loaded from {}", compiler::ART_PATH);
            a
        }
        None => {
            println!("no saved artifacts; compiling (once, offline)…");
            let a = compiler::compile(oracle, &c);
            compiler::save_artifacts(&a);
            a
        }
    };
    let cut = (c.stories as f64 * 0.8) as u32;
    let (store, _codes) = build_store(&art, &c);
    let store_bytes: usize = store
        .iter()
        .flat_map(|l| l.iter())
        .map(|(k, v)| k.len() + v.len() * 6)
        .sum();
    let runtime_bytes = art.token_codes.len()
        + art.stage_books.iter().map(|b| b.len()).sum::<usize>()
        + art.thresholds.len() * 8
        + art.class_sigs.iter().map(|s| s.len()).sum::<usize>();

    let test: Vec<usize> = (0..c.n).filter(|&i| c.story[i] >= cut).collect();
    let rot = derive_rotations();

    // 1. shipped runtime (plain path; equality with the kernel path is
    // witnessed by `certify`): full per-token path, live agreement.
    let timed = test.len().min(4000);
    let t0 = std::time::Instant::now();
    let mut agree = 0u64;
    for &i in test.iter().take(timed) {
        let code = code_plain(&art, &rot, &c, i);
        if predict_plain(&store, &code) == c.t_argmax[i] {
            agree += 1;
        }
    }
    let mulfree_tps = timed as f64 / t0.elapsed().as_secs_f64();
    let mulfree_agree = 100.0 * agree as f64 / timed as f64;

    // 2. kernel-counted runtime: every operation dispatched and counted.
    let mut rt = Runtime::new(&art);
    let itimed = 200usize;
    let t0 = std::time::Instant::now();
    for &i in test.iter().take(itimed) {
        let code = rt.assign(&c, i);
        let _ = rt.predict(&store, &code);
    }
    let inst_tps = itimed as f64 / t0.elapsed().as_secs_f64();

    // 3. in-crate teacher: greedy generation, single thread, through the
    // same oracle surface the compiler consumed.
    let vocab = oracle.vocab();
    let mut logits = vec![0f32; vocab];
    oracle.reset();
    let mut token = 1usize;
    let steps = 256usize;
    let t0 = std::time::Instant::now();
    for pos in 0..steps {
        oracle.step(token, pos, &mut logits);
        let mut best = 0usize;
        for i in 1..vocab {
            if logits[i] > logits[best] {
                best = i;
            }
        }
        token = best;
    }
    let teacher_tps = steps as f64 / t0.elapsed().as_secs_f64();

    println!();
    println!("measured on this machine, single thread, same κ-pinned source:");
    println!("| runtime | tok/s | agreement w/ teacher argmax | multiplies/token |");
    println!("|---|---|---|---|");
    println!(
        "| transformerless mul-free (shipped path, decode-on-demand) | {:.0} | {:.1}% (live, n={}) | 0 |",
        mulfree_tps, mulfree_agree, timed
    );
    println!(
        "| transformerless, every op census-counted | {:.0} | same function (witnessed by certify) | 0 |",
        inst_tps
    );
    println!(
        "| in-crate teacher (exactness-witnessed transformer) | {:.0} | 100% (it is the reference) | ~15M mul-adds |",
        teacher_tps
    );
    println!();
    println!(
        "artifact footprint (compressed, as shipped): runtime tables {:.2} MB (codes {:.0} KB + books {:.0} KB + thresholds {:.1} KB + signatures {:.0} KB) + store ≈ {:.1} MB ({} keys) = {:.2} MB total",
        runtime_bytes as f64 / 1e6,
        art.token_codes.len() as f64 / 1e3,
        art.stage_books.iter().map(|b| b.len()).sum::<usize>() as f64 / 1e3,
        art.thresholds.len() as f64 * 8.0 / 1e3,
        art.class_sigs.iter().map(|s| s.len()).sum::<usize>() as f64 / 1e3,
        store_bytes as f64 / 1e6,
        store.iter().map(|l| l.len()).sum::<usize>(),
        (runtime_bytes + store_bytes) as f64 / 1e6
    );
    println!();
    println!(
        "external classical runtimes (llama.cpp f32/q8_0, run.c, OpenMP APE) are\nrecorded with versions and reproduction commands in docs/COMPARISON.md."
    );
}
