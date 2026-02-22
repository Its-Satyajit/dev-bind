use super::*;
use tempfile::TempDir;

#[test]
fn test_cert_manager_creates_certs_dir() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path();

    // Instantiate CertManager
    let manager = CertManager::new(config_dir);

    let certs_dir = config_dir.join("certs");
    assert!(
        certs_dir.exists(),
        "CertManager should create the certs directory if it does not exist"
    );
    assert_eq!(manager.certs_dir, certs_dir);
}

#[test]
fn test_cert_manager_generates_root_ca() {
    let temp_dir = TempDir::new().unwrap();
    let manager = CertManager::new(temp_dir.path());

    // Generate root CA (this should create it on disk)
    let _root_ca = manager
        .get_root_ca()
        .expect("Failed to get or generate root CA");

    // Check that files were created
    let ca_cert_path = manager.certs_dir.join("devbind-rootCA.crt");
    let ca_key_path = manager.certs_dir.join("devbind-rootCA.key");

    assert!(
        ca_cert_path.exists(),
        "Root CA cert should be written to disk"
    );
    assert!(
        ca_key_path.exists(),
        "Root CA key should be written to disk"
    );

    // Calling it again should read from disk and return successfully
    let _root_ca_reloaded = manager
        .get_root_ca()
        .expect("Failed to reload root CA from disk");
}

#[test]
fn test_cert_manager_get_or_generate_cert() {
    let temp_dir = TempDir::new().unwrap();
    let manager = CertManager::new(temp_dir.path());
    let test_domain = "myapp.test";

    // 1st Call - Should generate and cache the cert
    let cert_key = manager
        .get_or_generate_cert(test_domain)
        .expect("Failed to generate cert");

    let cert_path = manager.certs_dir.join(format!("{}.crt", test_domain));
    let key_path = manager.certs_dir.join(format!("{}.key", test_domain));

    assert!(cert_path.exists(), "Domain cert should be written to disk");
    assert!(key_path.exists(), "Domain key should be written to disk");
    assert!(
        manager.cache.contains_key(test_domain),
        "Cert should be cached in memory"
    );

    // 2nd Call - Should return from cache directly
    let cert_key_cached = manager
        .get_or_generate_cert(test_domain)
        .expect("Failed to get cached cert");

    // Since it's an Arc, we can check if they point to the same memory to prove caching
    assert!(
        Arc::ptr_eq(&cert_key, &cert_key_cached),
        "Second call must return the cached Arc"
    );
}

#[test]
fn test_cert_manager_loads_existing_cert_into_cache() {
    let temp_dir = TempDir::new().unwrap();
    let manager1 = CertManager::new(temp_dir.path());
    let test_domain = "persisted.test";

    // Manager 1 generates the cert so it exists on disk
    let _ = manager1.get_or_generate_cert(test_domain).unwrap();

    // Manager 2 is fresh, cache is empty
    let manager2 = CertManager::new(temp_dir.path());
    assert!(
        !manager2.cache.contains_key(test_domain),
        "Fresh manager should have empty cache"
    );

    // Getting cert should read from disk and populate cache
    let _reloaded_cert = manager2.get_or_generate_cert(test_domain).unwrap();
    assert!(
        manager2.cache.contains_key(test_domain),
        "Manager 2 should cache the cert after loading from disk"
    );
}

#[test]
fn test_cert_generation_rejects_non_test_domain() {
    let temp_dir = TempDir::new().unwrap();
    let manager = CertManager::new(temp_dir.path());

    // Non-.test domains must be rejected at the cert level
    let result = manager.get_or_generate_cert("example.com");
    assert!(result.is_err(), "Non-.test domain should be rejected");

    let result = manager.get_or_generate_cert("shadcnstudio.com");
    assert!(result.is_err(), "Non-.test domain should be rejected");

    let result = manager.get_or_generate_cert("nottest");
    assert!(
        result.is_err(),
        "Domain without .test suffix should be rejected"
    );
}

#[test]
fn test_cert_generation_allows_test_domains() {
    let temp_dir = TempDir::new().unwrap();
    let manager = CertManager::new(temp_dir.path());

    // .test domains must still work
    assert!(
        manager.get_or_generate_cert("myapp.test").is_ok(),
        "myapp.test should be allowed"
    );
    assert!(
        manager.get_or_generate_cert("deep.sub.test").is_ok(),
        "deep.sub.test should be allowed"
    );
}
