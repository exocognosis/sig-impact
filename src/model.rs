use crate::sizes::Scheme;
use clap::ValueEnum;

pub const SECONDS_PER_DAY: f64 = 86_400.0;
pub const DAYS_PER_YEAR: f64 = 365.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum AggregationMode {
    None,
    Threshold,
    Batch,
}

impl std::fmt::Display for AggregationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "none",
                Self::Threshold => "threshold",
                Self::Batch => "batch",
            }
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Scenario {
    pub validators: u64,
    pub txs_per_block: u64,
    pub target_tps: f64,
    pub epoch_blocks: Option<u64>,
    pub aggregation: AggregationMode,
    pub header_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct Impact {
    pub scheme: Scheme,
    pub aggregation: AggregationMode,
    pub public_key_count: u64,
    pub signature_count: u64,
    pub pk_total: u128,
    pub sig_total: u128,
    pub header_bytes: u64,
    pub bytes_per_block: u128,
    pub bytes_per_day: u128,
    pub bytes_per_year: u128,
    pub bytes_per_epoch: Option<u128>,
    pub multiplier_vs_ecdsa: f64,
    pub savings_vs_none: f64,
}

fn counts(scenario: &Scenario) -> (u64, u64) {
    match scenario.aggregation {
        AggregationMode::None => (scenario.validators, scenario.txs_per_block),
        AggregationMode::Threshold => (1, 1),
        AggregationMode::Batch => (scenario.validators, 1),
    }
}

fn block_bytes(scheme: Scheme, scenario: &Scenario, mode: AggregationMode) -> u128 {
    let mut adjusted = *scenario;
    adjusted.aggregation = mode;
    let (pk_count, sig_count) = counts(&adjusted);
    u128::from(pk_count) * u128::from(scheme.pk_bytes)
        + u128::from(sig_count) * u128::from(scheme.sig_bytes)
        + u128::from(scenario.header_bytes)
}

pub fn calculate(scheme: Scheme, scenario: &Scenario, ecdsa: Scheme) -> Impact {
    let (public_key_count, signature_count) = counts(scenario);
    let pk_total = u128::from(public_key_count) * u128::from(scheme.pk_bytes);
    let sig_total = u128::from(signature_count) * u128::from(scheme.sig_bytes);
    let bytes_per_block = pk_total + sig_total + u128::from(scenario.header_bytes);
    let blocks_per_day = scenario.target_tps * SECONDS_PER_DAY / scenario.txs_per_block as f64;
    let bytes_per_day = (bytes_per_block as f64 * blocks_per_day).round() as u128;
    let bytes_per_year = (bytes_per_day as f64 * DAYS_PER_YEAR).round() as u128;
    let bytes_per_epoch = scenario
        .epoch_blocks
        .map(|blocks| bytes_per_block * u128::from(blocks));
    let baseline = block_bytes(ecdsa, scenario, scenario.aggregation);
    let unaggregated = block_bytes(scheme, scenario, AggregationMode::None);

    Impact {
        scheme,
        aggregation: scenario.aggregation,
        public_key_count,
        signature_count,
        pk_total,
        sig_total,
        header_bytes: scenario.header_bytes,
        bytes_per_block,
        bytes_per_day,
        bytes_per_year,
        bytes_per_epoch,
        multiplier_vs_ecdsa: bytes_per_block as f64 / baseline as f64,
        savings_vs_none: 1.0 - bytes_per_block as f64 / unaggregated as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sizes::find;

    fn scenario(aggregation: AggregationMode) -> Scenario {
        Scenario {
            validators: 100,
            txs_per_block: 100,
            target_tps: 50.0,
            epoch_blocks: None,
            aggregation,
            header_bytes: 0,
        }
    }

    #[test]
    fn none_counts_every_key_and_transaction_signature() {
        let impact = calculate(
            find("mldsa65").unwrap(),
            &scenario(AggregationMode::None),
            find("ecdsa").unwrap(),
        );
        assert_eq!(impact.pk_total, 195_200);
        assert_eq!(impact.sig_total, 330_900);
        assert_eq!(impact.bytes_per_block, 526_100);
        assert_eq!(impact.bytes_per_day, 22_727_520_000);
    }

    #[test]
    fn threshold_reduces_both_counts_to_one() {
        let impact = calculate(
            find("mldsa65").unwrap(),
            &scenario(AggregationMode::Threshold),
            find("ecdsa").unwrap(),
        );
        assert_eq!(impact.bytes_per_block, 5_261);
        assert_eq!((impact.public_key_count, impact.signature_count), (1, 1));
        assert!(impact.savings_vs_none > 0.98);
    }

    #[test]
    fn batch_retains_validator_registry_and_bundles_signatures() {
        let impact = calculate(
            find("mldsa65").unwrap(),
            &scenario(AggregationMode::Batch),
            find("ecdsa").unwrap(),
        );
        assert_eq!(impact.bytes_per_block, 198_509);
        assert_eq!((impact.public_key_count, impact.signature_count), (100, 1));
    }
}
