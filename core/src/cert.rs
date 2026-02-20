use rcgen::generate_simple_self_signed;
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::sync::Arc;
use rustls::sign::CertifiedKey;
use rustls::crypto::aws_lc_rs::sign::any_supported_type;

pub struct CertManager {
    certs_dir: PathBuf,
}

impl CertManager {
    pub fn new(config_dir: &std::path::Path) -> Self {
        let mut certs_dir = config_dir.to_path_buf();
        certs_dir.push("certs");
        if !certs_dir.exists() {
            let _ = fs::create_dir_all(&certs_dir);
        }
        Self { certs_dir }
    }

    pub fn get_or_generate_cert(&self, domain: &str) -> Result<Arc<CertifiedKey>> {
        let cert_path = self.certs_dir.join(format!("{}.crt", domain));
        let key_path = self.certs_dir.join(format!("{}.key", domain));

        let (cert_der, key_der) = if cert_path.exists() && key_path.exists() {
            let cert_bytes = fs::read(&cert_path).context("Failed to read cert")?;
            let key_bytes = fs::read(&key_path).context("Failed to read key")?;
            (cert_bytes, key_bytes)
        } else {
            let rcgen_cert = generate_simple_self_signed(vec![domain.to_string()])
                .map_err(|e| anyhow::anyhow!("Cert gen error: {}", e))?;

            let cert_der = rcgen_cert.serialize_der()
                .map_err(|e| anyhow::anyhow!("Cert serialize error: {}", e))?;

            let key_der = rcgen_cert.serialize_private_key_der();

            fs::write(&cert_path, &cert_der)?;
            fs::write(&key_path, &key_der)?;

            (cert_der, key_der)
        };

        let cert = CertificateDer::from(cert_der);
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));

        let signing_key = any_supported_type(&key)
            .map_err(|_| anyhow::anyhow!("Invalid key format"))?;

        Ok(Arc::new(CertifiedKey::new(
            vec![cert],
            signing_key,
        )))
    }
}
