//! Minimal blake3 file hasher for corpus/provenance manifests.
//! Usage: cargo run -q -p uor-r4-core --example b3sum -- <path>...

fn main() {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).expect("read");
        println!("blake3:{}  {}", blake3::hash(&bytes).to_hex(), path);
    }
}
