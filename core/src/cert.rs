use anyhow::{Context, Result};
use rcgen::generate_simple_self_signed;
use rustls::crypto::aws_lc_rs::sign::any_supported_type;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::sign::CertifiedKey;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

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

    fn get_root_ca(&self) -> Result<rcgen::Certificate> {
        let ca_cert_path = self.certs_dir.join("devbind-rootCA.crt");
        let ca_key_path = self.certs_dir.join("devbind-rootCA.key");

        if ca_cert_path.exists() && ca_key_path.exists() {
            let key_pem = fs::read_to_string(&ca_key_path)?;
            let key_pair = rcgen::KeyPair::from_pem(&key_pem)
                .map_err(|e| anyhow::anyhow!("Failed parsing root key: {}", e))?;

            let cert_pem = fs::read_to_string(&ca_cert_path)?;
            let params = rcgen::CertificateParams::from_ca_cert_pem(&cert_pem, key_pair)
                .map_err(|e| anyhow::anyhow!("Failed parsing root cert: {}", e))?;

            let cert = rcgen::Certificate::from_params(params)
                .map_err(|e| anyhow::anyhow!("Failed restoring root cert: {}", e))?;

            return Ok(cert);
        }

        // Generate a new Root CA
        let mut params: rcgen::CertificateParams = Default::default();
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.distinguished_name = rcgen::DistinguishedName::new();
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "DevBind Root CA");
        params
            .distinguished_name
            .push(rcgen::DnType::OrganizationName, "DevBind Proxy");

        let cert = rcgen::Certificate::from_params(params)
            .map_err(|e| anyhow::anyhow!("Failed generating root cert: {}", e))?;

        let cert_pem = cert
            .serialize_pem()
            .map_err(|e| anyhow::anyhow!("Failed signing root cert: {}", e))?;

        let key_pem = cert.serialize_private_key_pem();

        fs::write(&ca_cert_path, cert_pem)?;
        fs::write(&ca_key_path, key_pem)?;

        Ok(cert)
    }

    pub fn get_or_generate_cert(&self, domain: &str) -> Result<Arc<CertifiedKey>> {
        let cert_path = self.certs_dir.join(format!("{}.crt", domain));
        let key_path = self.certs_dir.join(format!("{}.key", domain));

        let (cert_der, key_der) = if cert_path.exists() && key_path.exists() {
            let cert_bytes = fs::read(&cert_path).context("Failed to read cert")?;
            let key_bytes = fs::read(&key_path).context("Failed to read key")?;
            (cert_bytes, key_bytes)
        } else {
            // Load the Root CA explicitly to sign this
            let ca_cert = self.get_root_ca()?;

            let mut params: rcgen::CertificateParams = Default::default();
            params.subject_alt_names = vec![rcgen::SanType::DnsName(domain.to_string())];
            params.distinguished_name = rcgen::DistinguishedName::new();
            params
                .distinguished_name
                .push(rcgen::DnType::CommonName, domain);

            let child_cert = rcgen::Certificate::from_params(params)
                .map_err(|e| anyhow::anyhow!("Failed to build child certificate: {}", e))?;

            let cert_der = child_cert
                .serialize_der_with_signer(&ca_cert)
                .map_err(|e| anyhow::anyhow!("Cert CA serialize error: {}", e))?;

            let key_der = child_cert.serialize_private_key_der();

            fs::write(&cert_path, &cert_der)?;
            fs::write(&key_path, &key_der)?;

            (cert_der, key_der)
        };

        let cert = CertificateDer::from(cert_der);
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));

        let signing_key =
            any_supported_type(&key).map_err(|_| anyhow::anyhow!("Invalid key format"))?;

        Ok(Arc::new(CertifiedKey::new(vec![cert], signing_key)))
    }
}
