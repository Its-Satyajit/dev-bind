use rcgen::*;
fn main() {
    let mut params: rcgen::CertificateParams = Default::default();
    let kp = KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap();
    params.key_pair = Some(kp);
    let cert = Certificate::from_params(params).unwrap();
    let der = cert.serialize_der().unwrap();
}
