# Benchmarks

Performance benchmarks using [criterion](https://github.com/bheisler/criterion.rs),
run against `samples/media` since it's the sample app that exercises the `ref/`
(quote + audio + transcript) pipeline end to end.

Two benchmarks:
- `full_build_samples_media` — a full `setup::init_and_build`, i.e. the whole build pipeline.
- `ref_process_markdown_samples_media` — just `Ref::process_markdown`, i.e. the `ref/` grouping
  and rendering logic in isolation.

## Running

Run all benchmarks:
```sh
cargo bench --bench build_bench
```

(`cargo bench` alone also tries to run the crate's lib tests through the benchmark
harness and will error on `--save-baseline`/`--baseline` flags — use `--bench build_bench`
to target just this file.)

## Viewing results

Each run writes an HTML report to `target/criterion/` (gitignored, local only):
```sh
open target/criterion/report/index.html
```

That's a combined index linking to each benchmark's own report
(`target/criterion/<benchmark-name>/report/index.html`), with charts of the
timing distribution across samples.

## Tracking performance over time

Results aren't committed to git (they live under `target/`), so to compare "before"
vs. "after" a change, save a named baseline and diff against it explicitly:

```sh
# snapshot current performance
cargo bench --bench build_bench -- --save-baseline my-baseline

# ...make your changes...

# compare against the snapshot
cargo bench --bench build_bench -- --baseline my-baseline
```

Criterion will report percentage change per benchmark (e.g. "Performance has regressed 12%").

The current baseline, saved before the `ref/` grouping rewrite, is named `pre-ref-rewrite`.
`cargo clean` wipes `target/`, including saved baselines — rerun the save command above
to recreate it if that happens.
