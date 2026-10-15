//! Local Certificate Authority and SAN SSL certificate generator using rcgen.

use crate::error::{CoreError, Result};
use crate::persistence::AppPaths;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair, KeyUsagePurpose,
    SanType,
};
use std::fs;
use std::path::PathBuf;

pub struct LocalCertManager;

impl LocalCertManager {
    /// Return the SSL storage directory (%LOCALAPPDATA%\Dotnet\ssl).
    pub fn ssl_dir() -> PathBuf {
        AppPaths::ssl_dir()
    }

    /// Root CA certificate path.
    pub fn root_ca_cert_path() -> PathBuf {
        Self::ssl_dir().join("rootCA.crt")
    }

    /// Root CA private key path.
    pub fn root_ca_key_path() -> PathBuf {
        Self::ssl_dir().join("rootCA.key")
    }

    fn make_ca_params() -> CertificateParams {
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Dotnet Development CA");
        dn.push(DnType::OrganizationName, "Dotnet Local Dev");
        dn.push(DnType::OrganizationalUnitName, "Development");
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        params
    }

    /// Retrieve or generate Root CA certificate and key PEM.
    pub fn get_or_create_root_ca() -> Result<(String, String)> {
        let ssl_dir = Self::ssl_dir();
        fs::create_dir_all(&ssl_dir)?;

        let cert_path = Self::root_ca_cert_path();
        let key_path = Self::root_ca_key_path();

        if cert_path.exists() && key_path.exists() {
            let cert_pem = fs::read_to_string(&cert_path)?;
            let key_pem = fs::read_to_string(&key_path)?;
            return Ok((cert_pem, key_pem));
        }

        // Generate new Root CA Key and Certificate
        let key_pair = KeyPair::generate()
            .map_err(|e| CoreError::Service(format!("Failed to generate RSA key pair: {e}")))?;

        let params = Self::make_ca_params();
        let cert = params
            .self_signed(&key_pair)
            .map_err(|e| CoreError::Service(format!("Failed to generate self-signed CA: {e}")))?;

        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        fs::write(&cert_path, &cert_pem)?;
        fs::write(&key_path, &key_pem)?;

        tracing::info!("Root CA generated at: {}", cert_path.display());
        Ok((cert_pem, key_pem))
    }

    /// Generate SAN SSL certificate for a local domain (`domain` and `*.domain`).
    pub fn generate_domain_cert(domain: &str) -> Result<(PathBuf, PathBuf)> {
        let ssl_dir = Self::ssl_dir();
        fs::create_dir_all(&ssl_dir)?;

        let cert_path = ssl_dir.join(format!("{}.crt", domain));
        let key_path = ssl_dir.join(format!("{}.key", domain));

        if cert_path.exists() && key_path.exists() {
            return Ok((cert_path, key_path));
        }

        let (_ca_cert_pem, ca_key_pem) = Self::get_or_create_root_ca()?;
        let ca_key = KeyPair::from_pem(&ca_key_pem)
            .map_err(|e| CoreError::Service(format!("Failed to parse CA key: {e}")))?;

        let ca_params = Self::make_ca_params();
        let ca_cert = ca_params
            .self_signed(&ca_key)
            .map_err(|e| CoreError::Service(format!("Failed to rebuild CA cert: {e}")))?;

        // End-entity certificate params
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, domain);
        params.distinguished_name = dn;
        params.subject_alt_names =
            vec![
                SanType::DnsName(domain.to_string().try_into().map_err(|e| {
                    CoreError::Service(format!("Invalid DNS name '{domain}': {e}"))
                })?),
                SanType::DnsName(
                    format!("*.{domain}").try_into().map_err(|e| {
                        CoreError::Service(format!("Invalid wildcard DNS name: {e}"))
                    })?,
                ),
            ];
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];

        let domain_key = KeyPair::generate()
            .map_err(|e| CoreError::Service(format!("Failed to generate domain key pair: {e}")))?;

        let cert = params
            .signed_by(&domain_key, &ca_cert, &ca_key)
            .map_err(|e| CoreError::Service(format!("Failed to sign domain cert: {e}")))?;

        fs::write(&cert_path, cert.pem())?;
        fs::write(&key_path, domain_key.serialize_pem())?;

        tracing::info!(
            "Domain certificate generated for {domain} at {}",
            cert_path.display()
        );
        Ok((cert_path, key_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_ca_and_domain_cert_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("DOTNET_DATA_DIR", temp_dir.path().to_str().unwrap());

        let (ca_cert, ca_key) = LocalCertManager::get_or_create_root_ca().unwrap();
        assert!(ca_cert.contains("BEGIN CERTIFICATE"));
        assert!(ca_key.contains("PRIVATE KEY"));

        let (domain_cert_path, domain_key_path) =
            LocalCertManager::generate_domain_cert("myproject.test").unwrap();

        assert!(domain_cert_path.exists());
        assert!(domain_key_path.exists());

        let domain_cert_content = fs::read_to_string(&domain_cert_path).unwrap();
        assert!(domain_cert_content.contains("BEGIN CERTIFICATE"));
    }
}
