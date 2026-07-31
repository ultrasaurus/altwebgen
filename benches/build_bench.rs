//! Baseline performance benchmarks.
//!
//! Run with: cargo bench
//! Save a named baseline before a change:  cargo bench -- --save-baseline before
//! Compare against it after the change:    cargo bench -- --baseline before
//!
//! Targets `samples/media`, since it's the sample app that exercises the
//! `ref/` (quote + audio + transcript) pipeline end to end.

use altwebgen::config::{Config, Mode, Transcript};
use altwebgen::setup;
use altwebgen::web::Ref;
use criterion::{criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

fn media_config() -> Config {
    Config::new(
        "samples/media/.dist",
        "samples/media/source",
        "samples/media/template",
        "",
        Mode::Build,
        Transcript::Off,
    )
}

fn bench_full_build(c: &mut Criterion) {
    let config = media_config();
    c.bench_function("full_build_samples_media", |b| {
        b.iter(|| {
            setup::init_and_build(&config).expect("build should succeed");
        });
    });
}

fn bench_ref_processing(c: &mut Criterion) {
    let config = media_config();
    let ref_source_dir = PathBuf::from("samples/media/ref");
    let ref_dest_dir = PathBuf::from("target/bench-out/ref");
    std::fs::create_dir_all(&ref_dest_dir).expect("failed to create bench output dir");

    c.bench_function("ref_process_markdown_samples_media", |b| {
        b.iter(|| {
            Ref::process_markdown(&config, &ref_source_dir, &ref_dest_dir)
                .expect("ref processing should succeed");
        });
    });
}

criterion_group!(benches, bench_full_build, bench_ref_processing);
criterion_main!(benches);
