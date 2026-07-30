mod model;
mod output;
mod sizes;

use clap::{parser::ValueSource, CommandFactory, FromArgMatches, Parser, ValueEnum};
use model::{AggregationMode, Scenario};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ValueEnum)]
enum Format {
    Table,
    Csv,
    Md,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Preset {
    SmallL1,
    EthereumScale,
    HeavyValidators,
}

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Model signature byte impact in validator/blockchain scenarios"
)]
struct Cli {
    /// Validator public keys represented in state/block data.
    #[arg(long, default_value_t = 100)]
    validators: u64,
    /// Transactions (and unaggregated signatures) per block.
    #[arg(long = "txs", default_value_t = 100)]
    txs_per_block: u64,
    /// Target transactions per second.
    #[arg(long, default_value_t = 50.0)]
    tps: f64,
    /// Comma-separated schemes (use "all" for every scheme).
    #[arg(long, value_delimiter = ',', default_value = "mldsa65,ecdsa")]
    schemes: Vec<String>,
    /// none: all keys+sigs; threshold: one key+sig; batch: all keys+one sig.
    #[arg(long = "agg", value_enum, default_value_t = AggregationMode::None)]
    aggregation: AggregationMode,
    /// Comma-separated output formats.
    #[arg(long, value_enum, value_delimiter = ',', default_value = "table")]
    format: Vec<Format>,
    /// File path/stem for CSV and Markdown output; tables print to stdout.
    #[arg(long)]
    out: Option<PathBuf>,
    /// Optional epoch length in blocks.
    #[arg(long)]
    epoch_blocks: Option<u64>,
    /// Fixed bytes added to every modeled block.
    #[arg(long, default_value_t = 0)]
    header_bytes: u64,
    /// Apply a scenario preset; explicit numeric flags override preset values.
    #[arg(long, value_enum)]
    preset: Option<Preset>,
}

fn preset_values(preset: Preset) -> (u64, u64, f64) {
    match preset {
        Preset::SmallL1 => (100, 100, 50.0),
        Preset::EthereumScale => (1_000_000, 200, 20.0),
        Preset::HeavyValidators => (10_000, 1_000, 1_000.0),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Cli::command().get_matches();
    let mut cli = Cli::from_arg_matches(&matches)?;
    if let Some(preset) = cli.preset {
        let (validators, txs, tps) = preset_values(preset);
        if matches.value_source("validators") == Some(ValueSource::DefaultValue) {
            cli.validators = validators;
        }
        if matches.value_source("txs_per_block") == Some(ValueSource::DefaultValue) {
            cli.txs_per_block = txs;
        }
        if matches.value_source("tps") == Some(ValueSource::DefaultValue) {
            cli.tps = tps;
        }
    }
    if cli.validators == 0 || cli.txs_per_block == 0 {
        return Err("--validators and --txs must be greater than zero".into());
    }
    if !cli.tps.is_finite() || cli.tps <= 0.0 {
        return Err("--tps must be a finite number greater than zero".into());
    }

    let selected = if cli.schemes.iter().any(|id| id.eq_ignore_ascii_case("all")) {
        sizes::SCHEMES.to_vec()
    } else {
        cli.schemes
            .iter()
            .map(|id| {
                sizes::find(id).ok_or_else(|| {
                    format!(
                        "unknown scheme '{id}'; valid IDs: {}",
                        sizes::SCHEMES
                            .iter()
                            .map(|s| s.id)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let scenario = Scenario {
        validators: cli.validators,
        txs_per_block: cli.txs_per_block,
        target_tps: cli.tps,
        epoch_blocks: cli.epoch_blocks,
        aggregation: cli.aggregation,
        header_bytes: cli.header_bytes,
    };
    let ecdsa = sizes::find("ecdsa").expect("ECDSA baseline exists");
    let impacts: Vec<_> = selected
        .into_iter()
        .map(|scheme| model::calculate(scheme, &scenario, ecdsa))
        .collect();
    let multiple_files = cli
        .format
        .iter()
        .filter(|format| **format != Format::Table)
        .count()
        > 1;

    for format in cli.format {
        match format {
            Format::Table => print!("{}", output::table(&impacts)),
            Format::Csv => {
                let content = output::csv(&impacts)?;
                if let Some(base) = &cli.out {
                    let path = output::output_path(base, "csv", multiple_files);
                    std::fs::write(&path, content)?;
                    eprintln!("wrote {}", path.display());
                } else {
                    print!("{content}");
                }
            }
            Format::Md => {
                let content = output::markdown(&impacts, &scenario);
                if let Some(base) = &cli.out {
                    let path = output::output_path(base, "md", multiple_files);
                    std::fs::write(&path, content)?;
                    eprintln!("wrote {}", path.display());
                } else {
                    print!("{content}");
                }
            }
        }
    }
    Ok(())
}
