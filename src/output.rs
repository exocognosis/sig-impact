use crate::model::{Impact, Scenario};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn integer(value: u128) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn percent(value: f64) -> String {
    format!("{:.2}%", value * 100.0)
}

pub fn table(impacts: &[Impact]) -> String {
    let headers = [
        "Scheme",
        "PK bytes",
        "Sig bytes",
        "Block bytes",
        "Day bytes",
        "Year bytes",
        "vs ECDSA",
        "Saved",
    ];
    let rows: Vec<Vec<String>> = impacts
        .iter()
        .map(|impact| {
            vec![
                impact.scheme.name.to_owned(),
                integer(impact.pk_total),
                integer(impact.sig_total),
                integer(impact.bytes_per_block),
                integer(impact.bytes_per_day),
                integer(impact.bytes_per_year),
                format!("{:.2}x", impact.multiplier_vs_ecdsa),
                percent(impact.savings_vs_none),
            ]
        })
        .collect();
    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(column, header)| {
            rows.iter()
                .map(|row| row[column].len())
                .max()
                .unwrap_or(0)
                .max(header.len())
        })
        .collect();
    let mut out = String::new();
    for (column, header) in headers.iter().enumerate() {
        let _ = write!(
            out,
            "{header:<width$}{}",
            if column + 1 == headers.len() {
                "\n"
            } else {
                "  "
            },
            width = widths[column]
        );
    }
    for (column, width) in widths.iter().enumerate() {
        out.push_str(&"-".repeat(*width));
        out.push_str(if column + 1 == widths.len() {
            "\n"
        } else {
            "  "
        });
    }
    for row in rows {
        for (column, value) in row.iter().enumerate() {
            let _ = write!(
                out,
                "{value:<width$}{}",
                if column + 1 == row.len() { "\n" } else { "  " },
                width = widths[column]
            );
        }
    }
    out
}

pub fn markdown(impacts: &[Impact], scenario: &Scenario) -> String {
    let mut out = format!(
        "# Signature impact report\n\nValidators: {} · Transactions/block: {} · Target TPS: {} · Aggregation: {}\n\n",
        scenario.validators, scenario.txs_per_block, scenario.target_tps, scenario.aggregation
    );
    out.push_str("| Scheme | Public keys | Signatures | Bytes/block | Bytes/day | Bytes/year | vs ECDSA | Savings vs none |\n");
    out.push_str("|---|---:|---:|---:|---:|---:|---:|---:|\n");
    for impact in impacts {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {:.2}x | {} |",
            impact.scheme.name,
            integer(impact.pk_total),
            integer(impact.sig_total),
            integer(impact.bytes_per_block),
            integer(impact.bytes_per_day),
            integer(impact.bytes_per_year),
            impact.multiplier_vs_ecdsa,
            percent(impact.savings_vs_none)
        );
    }
    out.push_str(
        "\n_This is a deterministic size model, not a cryptographic or consensus simulation._\n",
    );
    out
}

pub fn csv(impacts: &[Impact]) -> Result<String, csv::Error> {
    let mut writer = csv::Writer::from_writer(vec![]);
    writer.write_record([
        "scheme_id",
        "scheme",
        "aggregation",
        "pk_bytes_each",
        "sig_bytes_each",
        "public_key_count",
        "signature_count",
        "pk_total_bytes",
        "sig_total_bytes",
        "header_bytes",
        "bytes_per_block",
        "bytes_per_day",
        "bytes_per_year",
        "bytes_per_epoch",
        "multiplier_vs_ecdsa",
        "savings_vs_none_pct",
        "notes",
    ])?;
    for impact in impacts {
        writer.write_record([
            impact.scheme.id.to_owned(),
            impact.scheme.name.to_owned(),
            impact.aggregation.to_string(),
            impact.scheme.pk_bytes.to_string(),
            impact.scheme.sig_bytes.to_string(),
            impact.public_key_count.to_string(),
            impact.signature_count.to_string(),
            impact.pk_total.to_string(),
            impact.sig_total.to_string(),
            impact.header_bytes.to_string(),
            impact.bytes_per_block.to_string(),
            impact.bytes_per_day.to_string(),
            impact.bytes_per_year.to_string(),
            impact
                .bytes_per_epoch
                .map(|v| v.to_string())
                .unwrap_or_default(),
            format!("{:.6}", impact.multiplier_vs_ecdsa),
            format!("{:.4}", impact.savings_vs_none * 100.0),
            impact.scheme.notes.to_owned(),
        ])?;
    }
    let bytes = writer.into_inner().map_err(|err| err.into_error())?;
    Ok(String::from_utf8(bytes).expect("CSV output is UTF-8"))
}

pub fn output_path(base: &Path, extension: &str, multiple_files: bool) -> PathBuf {
    if !multiple_files && base.extension().and_then(|ext| ext.to_str()) == Some(extension) {
        return base.to_owned();
    }
    let stem = if base.extension().is_some() {
        base.with_extension("")
    } else {
        base.to_owned()
    };
    stem.with_extension(extension)
}
