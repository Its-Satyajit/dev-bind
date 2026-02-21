use anyhow::{Context, Result};
use dashmap::DashMap;
use rustls::crypto::aws_lc_rs::sign::any_supported_type;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::sign::CertifiedKey;
use std::path::PathBuf;
use std::sync::Arc;

pub struct CertManager {
    certs_dir: PathBuf,
    /// In-memory cache: domain → certified key.
    ///
    /// After the first TLS handshake for a domain, subsequent handshakes
    /// never touch disk — eliminating blocking I/O from the hot path.
    cache: DashMap<String, Arc<CertifiedKey>>,
}

impl CertManager {
    pub fn new(config_dir: &std::path::Path) -> Self {
        let certs_dir = config_dir.join("certs");
        if !certs_dir.exists() {
            let _ = std::fs::create_dir_all(&certs_dir);
        }
        Self {
            certs_dir,
            cache: DashMap::new(),
        }
    }

    fn get_root_ca(&self) -> Result<(rcgen::KeyPair, String)> {
        let ca_cert_path = self.certs_dir.join("devbind-rootCA.crt");
        let ca_key_path = self.certs_dir.join("devbind-rootCA.key");

        if ca_cert_path.exists() && ca_key_path.exists() {
            let key_pem = std::fs::read_to_string(&ca_key_path)?;
            let key_pair = rcgen::KeyPair::from_pem(&key_pem)
                .map_err(|e| anyhow::anyhow!("Failed parsing root key: {}", e))?;

            let cert_pem = std::fs::read_to_string(&ca_cert_path)?;
            return Ok((key_pair, cert_pem));
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

        let key_pair = rcgen::KeyPair::generate()?;
        let cert = params
            .self_signed(&key_pair)
            .map_err(|e| anyhow::anyhow!("Failed generating root cert: {}", e))?;

        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        std::fs::write(&ca_cert_path, &cert_pem)?;
        write_private_key(&ca_key_path, key_pem.as_bytes())?;

        Ok((key_pair, cert_pem))
    }

    /// Returns a `CertifiedKey` for the given domain.
    ///
    /// **First call**: reads from disk (or generates) and stores in the in-memory cache.
    /// **Subsequent calls**: served directly from the cache — **zero disk I/O**.
    ///
    /// `SniResolver::resolve` is a synchronous trait method, so this stays synchronous.
    /// The in-memory cache eliminates blocking I/O from the TLS hot path after warm-up.
    pub fn get_or_generate_cert(&self, domain: &str) -> Result<Arc<CertifiedKey>> {
        // Fast path: cache hit — no disk I/O at all.
        if let Some(cached) = self.cache.get(domain) {
            return Ok(cached.clone());
        }

        // Slow path: load from disk or generate, then populate cache.
        let cert_path = self.certs_dir.join(format!("{}.crt", domain));
        let key_path = self.certs_dir.join(format!("{}.key", domain));

        let (cert_der, key_der) = if cert_path.exists() && key_path.exists() {
            let cert_bytes = std::fs::read(&cert_path).context("Failed to read cert")?;
            let key_bytes = std::fs::read(&key_path).context("Failed to read key")?;
            (cert_bytes, key_bytes)
        } else {
            let (ca_key, ca_cert_pem) = self.get_root_ca()?;
            let ca_issuer = rcgen::Issuer::from_ca_cert_pem(&ca_cert_pem, &ca_key)
                .map_err(|e| anyhow::anyhow!("Failed to create CA issuer: {}", e))?;

            let mut params: rcgen::CertificateParams = Default::default();
            params.subject_alt_names = vec![rcgen::SanType::DnsName(
                domain
                    .to_string()
                    .try_into()
                    .map_err(|e| anyhow::anyhow!("Invalid DNS name: {}", e))?,
            )];
            params.distinguished_name = rcgen::DistinguishedName::new();
            params
                .distinguished_name
                .push(rcgen::DnType::CommonName, domain);

            let key_pair = rcgen::KeyPair::generate()?;
            let child_cert = params
                .signed_by(&key_pair, &ca_issuer)
                .map_err(|e| anyhow::anyhow!("Failed to build child certificate: {}", e))?;

            let cert_der = child_cert.der().to_vec();
            let key_der = key_pair.serialize_der();

            std::fs::write(&cert_path, &cert_der)?;
            write_private_key(&key_path, &key_der)?;

            (cert_der, key_der)
        };

        let cert = CertificateDer::from(cert_der);
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));
        let signing_key =
            any_supported_type(&key).map_err(|_| anyhow::anyhow!("Invalid key format"))?;

        let certified_key = Arc::new(CertifiedKey::new(vec![cert], signing_key));

        // Store in cache so future handshakes for this domain skip disk entirely.
        self.cache.insert(domain.to_string(), certified_key.clone());

        Ok(certified_key)
    }
}

/// Write `data` to `path` with file permissions set to `0600` (owner read/write only).
fn write_private_key(path: &std::path::Path, data: &[u8]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .with_context(|| format!("Failed to create key file at {:?}", path))?;

        file.write_all(data)
            .with_context(|| format!("Failed to write key to {:?}", path))?;
    }

    #[cfg(not(unix))]
    {
        std::fs::write(path, data).with_context(|| format!("Failed to write key to {:?}", path))?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "cert_tests.rs"]
mod cert_tests;
