#[derive(Clone, Copy, Debug)]
pub struct Scheme {
    pub id: &'static str,
    pub name: &'static str,
    pub pk_bytes: u64,
    pub sig_bytes: u64,
    pub notes: &'static str,
}

pub const SCHEMES: &[Scheme] = &[
    Scheme {
        id: "ecdsa",
        name: "ECDSA-secp256k1",
        pk_bytes: 33,
        sig_bytes: 64,
        notes: "compressed public key; compact signature",
    },
    Scheme {
        id: "bls",
        name: "BLS12-381",
        pk_bytes: 96,
        sig_bytes: 48,
        notes: "minimal-signature-size variant",
    },
    Scheme {
        id: "mldsa44",
        name: "ML-DSA-44",
        pk_bytes: 1_312,
        sig_bytes: 2_420,
        notes: "NIST FIPS 204",
    },
    Scheme {
        id: "mldsa65",
        name: "ML-DSA-65",
        pk_bytes: 1_952,
        sig_bytes: 3_309,
        notes: "NIST FIPS 204",
    },
    Scheme {
        id: "mldsa87",
        name: "ML-DSA-87",
        pk_bytes: 2_592,
        sig_bytes: 4_627,
        notes: "NIST FIPS 204",
    },
    Scheme {
        id: "slhdsa128f",
        name: "SLH-DSA-128f",
        pk_bytes: 32,
        sig_bytes: 17_088,
        notes: "SHA2/SHAKE sizes; NIST FIPS 205",
    },
    Scheme {
        id: "slhdsa128s",
        name: "SLH-DSA-128s",
        pk_bytes: 32,
        sig_bytes: 7_856,
        notes: "SHA2/SHAKE sizes; NIST FIPS 205",
    },
    Scheme {
        id: "falcon512",
        name: "Falcon-512",
        pk_bytes: 897,
        sig_bytes: 666,
        notes: "padded signature baseline; not a final NIST standard",
    },
    Scheme {
        id: "falcon1024",
        name: "Falcon-1024",
        pk_bytes: 1_793,
        sig_bytes: 1_280,
        notes: "padded signature baseline; not a final NIST standard",
    },
];

pub fn find(id: &str) -> Option<Scheme> {
    let normalized = id.to_ascii_lowercase().replace(['-', '_'], "");
    SCHEMES.iter().copied().find(|scheme| {
        scheme.id.replace(['-', '_'], "") == normalized
            || scheme.name.to_ascii_lowercase().replace(['-', '_'], "") == normalized
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standardized_sizes_are_exact() {
        let expected = [
            ("mldsa44", 1_312, 2_420),
            ("mldsa65", 1_952, 3_309),
            ("mldsa87", 2_592, 4_627),
            ("slhdsa128f", 32, 17_088),
            ("slhdsa128s", 32, 7_856),
        ];
        for (id, pk, sig) in expected {
            let scheme = find(id).unwrap();
            assert_eq!((scheme.pk_bytes, scheme.sig_bytes), (pk, sig));
        }
    }
}
