//! Machine-checkable Certificate schema for empirical and structural claims
//! emitted by the R4 holographic graph compiler.

use blake3::Hasher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimKind {
    Structural,
    Empirical,
    Performance,
    Safety,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmpiricalClaim {
    pub name: String,
    pub sample_size: u64,
    pub metric_value: f64,
    pub confidence_interval_95: (f64, f64),
    pub slice_label: String,
    pub claim_kind: ClaimKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProtocolAttestation {
    pub deterministic_canonical_mode: bool,
    pub zero_allocation_verified: bool,
    pub no_multiply_verified: bool,
    pub theorem_7_reverse_index_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Certificate {
    pub version: u32,
    pub certificate_cid: String,
    pub source_cid: String,
    pub corpus_cid: String,
    pub graph_cid: String,
    pub metric_cid: String,
    pub op_cid: String,
    pub benchmark_cid: String,
    pub claims: Vec<EmpiricalClaim>,
    pub attestation: ProtocolAttestation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateError {
    SerializationError(String),
    DeserializationError(String),
    CidMismatch { expected: String, actual: String },
    AttestationFailed(String),
}

impl Certificate {
    pub fn new(
        source_cid: impl Into<String>,
        corpus_cid: impl Into<String>,
        graph_cid: impl Into<String>,
        metric_cid: impl Into<String>,
        op_cid: impl Into<String>,
        benchmark_cid: impl Into<String>,
        claims: Vec<EmpiricalClaim>,
        attestation: ProtocolAttestation,
    ) -> Self {
        let mut cert = Certificate {
            version: 1,
            certificate_cid: String::new(),
            source_cid: source_cid.into(),
            corpus_cid: corpus_cid.into(),
            graph_cid: graph_cid.into(),
            metric_cid: metric_cid.into(),
            op_cid: op_cid.into(),
            benchmark_cid: benchmark_cid.into(),
            claims,
            attestation,
        };
        cert.certificate_cid = cert.compute_cid();
        cert
    }

    /// Compute self-referential BLAKE3 CID (hex format) over certificate content.
    pub fn compute_cid(&self) -> String {
        let mut clone = self.clone();
        clone.certificate_cid.clear();
        let bytes = serde_json::to_vec(&clone)
            .expect("Certificate must serialize to JSON for CID computation");
        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        format!("kappa:blake3:{}", hasher.finalize().to_hex())

    /// Verify self-referential BLAKE3 CID.
    pub fn verify_cid(&self) -> bool {
        let computed = self.compute_cid();
        self.certificate_cid == computed
    }

    /// Validate structural attestation requirements (Gate K).
    pub fn verify_attestation(&self) -> Result<(), CertificateError> {
        if !self.verify_cid() {
            return Err(CertificateError::CidMismatch {
                expected: self.compute_cid(),
                actual: self.certificate_cid.clone(),
            });
        }
        if !self.attestation.zero_allocation_verified {
            return Err(CertificateError::AttestationFailed(
                "zero allocation check failed".to_string(),
            ));
        }
        if !self.attestation.no_multiply_verified {
            return Err(CertificateError::AttestationFailed(
                "no multiply check failed".to_string(),
            ));
        }
        if !self.attestation.theorem_7_reverse_index_verified {
            return Err(CertificateError::AttestationFailed(
                "Theorem 7 reverse index check failed".to_string(),
            ));
        }
        Ok(())
    }

    /// Serialize certificate to CBOR bytes.
    pub fn to_cbor_bytes(&self) -> Result<Vec<u8>, CertificateError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .map_err(|e| CertificateError::SerializationError(e.to_string()))?;
        Ok(buf)
    }

    /// Deserialize certificate from CBOR bytes.
    pub fn from_cbor_bytes(bytes: &[u8]) -> Result<Self, CertificateError> {
        let cert: Certificate = ciborium::from_reader(bytes)
            .map_err(|e| CertificateError::DeserializationError(e.to_string()))?;
        if !cert.verify_cid() {
            return Err(CertificateError::CidMismatch {
                expected: cert.compute_cid(),
                actual: cert.certificate_cid,
            });
        }
        Ok(cert)
    }
}
