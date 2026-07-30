# sig-impact

[![CI](https://github.com/exocognosis/sig-impact/actions/workflows/ci.yml/badge.svg)](https://github.com/exocognosis/sig-impact/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

A tiny, offline CLI that models the bandwidth and state impact of replacing
classical ECDSA and BLS signatures with NIST-standardized ML-DSA (FIPS 204) and
SLH-DSA (FIPS 205), alongside Falcon/FN-DSA comparison baselines, in blockchain
and validator environments—including simplified aggregation scenarios.

Install it in one line:

```sh
cargo install --git https://github.com/exocognosis/sig-impact
```

It answers questions like:

- How many extra bytes does ML-DSA-65 add at 1,000 validators?
- How much does threshold aggregation save compared with no aggregation?
- What does the change mean per block, day, year, or epoch?

`sig-impact` performs deterministic byte arithmetic locally and returns results
in under a second. It never performs cryptographic operations, contacts a
network service, or estimates verification time.

## Why I built it

During post-quantum signature and threshold research, I kept needing the same
numbers:

> “How many extra bytes does ML-DSA-65 actually add at this validator count?”
>
> “What does threshold aggregation buy me versus pure BLS?”

I got tired of recalculating them by hand, so I wrote this.

## Quick start

You need a current stable [Rust toolchain](https://rustup.rs/). Then:

```sh
git clone https://github.com/exocognosis/sig-impact.git
cd sig-impact
cargo run --release --
```

The default scenario compares ML-DSA-65 with ECDSA using 100 validators,
100 transactions per block, 50 TPS, and no aggregation:

```text
Scheme           vs ECDSA  PK bytes  Sig bytes  Block bytes  Day bytes
ML-DSA-65        54.24x    195,200   330,900    526,100      22,727,520,000
ECDSA-secp256k1  1.00x     3,300     6,400      9,700        419,040,000
```

Try a custom comparison:

```sh
cargo run --release -- \
  --validators 1000 \
  --txs 200 \
  --tps 50 \
  --schemes mldsa65,ecdsa,bls \
  --agg threshold
```

### A more dramatic comparison

The `heavy-validators` preset models 10,000 validators, 1,000 transactions per
block, and 1,000 TPS. With no aggregation:

```text
Scheme           vs ECDSA  PK bytes    Sig bytes  Block bytes  Saved
ML-DSA-65        57.94x    19,520,000  3,309,000  22,829,000   0.00%
ECDSA-secp256k1  1.00x     330,000     64,000     394,000      0.00%
BLS12-381        2.56x     960,000     48,000     1,008,000    0.00%
```

Under the simplified threshold model, the block representation falls to one
group key and one signature:

```text
Scheme           vs ECDSA  PK bytes  Sig bytes  Block bytes  Saved
ML-DSA-65        54.24x    1,952     3,309      5,261        99.98%
ECDSA-secp256k1  1.00x     33        64         97           99.98%
BLS12-381        1.48x     96        48         144          99.99%
```

Reproduce both tables:

```sh
sig-impact --preset heavy-validators --schemes mldsa65,ecdsa,bls --agg none
sig-impact --preset heavy-validators --schemes mldsa65,ecdsa,bls --agg threshold
```

## Install

Install directly from GitHub:

```sh
cargo install --git https://github.com/exocognosis/sig-impact
sig-impact --help
```

Or install from a local checkout:

```sh
cargo install --path .
```

The compiled executable is self-contained and does not need configuration
files or network access. Cargo downloads the `clap` and `csv` crates when
building for the first time.

## Export reports

Generate a human-readable table and reusable CSV and Markdown reports:

```sh
sig-impact \
  --validators 1000 \
  --txs 200 \
  --tps 50 \
  --schemes mldsa65,ecdsa,bls \
  --agg threshold \
  --format table,csv,md \
  --out report
```

The table prints to the terminal; `report.csv` and `report.md` are written to
the current directory.

## CLI

```text
Usage: sig-impact [OPTIONS]

Options:
      --validators <VALIDATORS>      Validator public keys represented [default: 100]
      --txs <TXS_PER_BLOCK>          Transactions/signatures per block [default: 100]
      --tps <TPS>                    Target transactions per second [default: 50]
      --schemes <SCHEMES>            Comma-separated scheme IDs [default: mldsa65,ecdsa]
      --agg <AGGREGATION>            none, threshold, or batch [default: none]
      --format <FORMAT>              table, csv, or md [default: table]
      --out <OUT>                    Output path or stem for CSV/Markdown
      --epoch-blocks <EPOCH_BLOCKS>  Optional epoch length in blocks
      --header-bytes <HEADER_BYTES>  Fixed bytes added to every block [default: 0]
      --preset <PRESET>              small-l1, ethereum-scale, or heavy-validators
  -h, --help                         Print help
  -V, --version                      Print version
```

Use `--schemes all` to include every built-in scheme. Explicit numeric flags
override values supplied by a preset.

## Presets

| Preset | Validators | Transactions/block | TPS | Intended use |
|---|---:|---:|---:|---|
| `small-l1` | 100 | 100 | 50 | Compact validator network |
| `ethereum-scale` | 1,000,000 | 200 | 20 | Very large validator registry |
| `heavy-validators` | 10,000 | 1,000 | 1,000 | High-throughput validator stress case |

Preset values are also shown by `sig-impact --help`. Supplying
`--validators`, `--txs`, or `--tps` overrides the corresponding preset value.

## Aggregation model

| Mode | Public-key count | Signature count | Interpretation |
|---|---:|---:|---|
| `none` | validators | transactions/block | Every validator key remains represented; every transaction has a signature |
| `batch` | validators | 1 | The validator registry remains; signatures are represented by one optimistic batch proof |
| `threshold` | 1 | 1 | One group key and one threshold signature represent the block |

These are intentionally simple size scenarios, not claims that every listed
scheme natively supports aggregation.

```text
bytes/block =
    public-key count × public-key bytes
  + signature count × signature bytes
  + optional header bytes

blocks/day = TPS × 86,400 ÷ transactions/block
```

Savings compare the selected aggregation mode with `none` for the same scheme.
The multiplier compares each result with ECDSA under the same aggregation mode.
A year is modeled as 365 days.

## Built-in schemes

| ID | Public key | Signature | Basis |
|---|---:|---:|---|
| `ecdsa` | 33 B | 64 B | compressed secp256k1 key; compact signature |
| `bls` | 96 B | 48 B | BLS12-381 minimal-signature-size variant |
| `mldsa44` | 1,312 B | 2,420 B | ML-DSA-44, NIST FIPS 204 |
| `mldsa65` | 1,952 B | 3,309 B | ML-DSA-65, NIST FIPS 204 |
| `mldsa87` | 2,592 B | 4,627 B | ML-DSA-87, NIST FIPS 204 |
| `slhdsa128f` | 32 B | 17,088 B | SLH-DSA-128f, NIST FIPS 205 |
| `slhdsa128s` | 32 B | 7,856 B | SLH-DSA-128s, NIST FIPS 205 |
| `falcon512` | 897 B | 666 B | Falcon-512 padded-signature baseline |
| `falcon1024` | 1,793 B | 1,280 B | Falcon-1024 padded-signature baseline |

The SHA2 and SHAKE variants of each listed SLH-DSA parameter set have identical
key and signature sizes. Falcon is included as a research comparison baseline;
it is not currently a final NIST FIPS signature standard.

Primary references:

- [NIST FIPS 204: Module-Lattice-Based Digital Signature Standard](https://doi.org/10.6028/NIST.FIPS.204)
- [NIST FIPS 205: Stateless Hash-Based Digital Signature Standard](https://doi.org/10.6028/NIST.FIPS.205)
- [Falcon specification](https://falcon-sign.info/falcon.pdf)

## Development

Everything contributors need is available through Cargo:

```sh
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Run the complete local verification suite with:

```sh
make check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the development workflow and
[SECURITY.md](SECURITY.md) for reporting security issues.

## Scope

This project deliberately does not:

- perform real signing or verification;
- benchmark cryptographic operations;
- simulate consensus or network topology;
- claim production aggregation support for any scheme.

Keeping those boundaries explicit makes the output fast, reproducible, and
easy to cite.

## License

Licensed under the [MIT License](LICENSE).
