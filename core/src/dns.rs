//! Minimal embedded DNS server for DevBind.
//!
//! Resolves **all** `*.test` A queries to `127.0.0.1` and AAAA to `::1`.
//! Returns NXDOMAIN for anything outside `.test`.
//!
//! Wire format reference: RFC 1035 §4.

use crate::config::DevBindConfig;
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{error, info, trace};

/// Default listen address for the DevBind DNS server.
pub const DNS_LISTEN_ADDR: &str = "127.0.2.1:53";

/// Start the DNS server. This function runs forever (call via `tokio::spawn`).
pub async fn run_dns_server(listen_addr: &str, config_dir: PathBuf) {
    let socket = match UdpSocket::bind(listen_addr).await {
        Ok(s) => {
            info!("DevBind DNS server listening on {}", listen_addr);
            s
        }
        Err(e) => {
            error!("Failed to bind DNS server to {}: {}", listen_addr, e);
            return;
        }
    };

    let config_path = config_dir.join("config.toml");
    let initial_domains = load_allowed_domains(&config_path);
    let allowed_domains: Arc<RwLock<(HashSet<String>, Instant)>> =
        Arc::new(RwLock::new((initial_domains, Instant::now())));

    const CACHE_TTL: Duration = Duration::from_secs(5);

    let mut buf = [0u8; 512];
    loop {
        let (len, src) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                error!("DNS recv error: {}", e);
                continue;
            }
        };

        if len < 12 {
            continue; // Too short for a DNS header
        }

        // Refresh config only when TTL has expired — avoids per-query disk I/O.
        let domains = {
            let needs_refresh = {
                let guard = allowed_domains.read().await;
                guard.1.elapsed() > CACHE_TTL
            };

            if needs_refresh {
                let mut guard = allowed_domains.write().await;
                if guard.1.elapsed() > CACHE_TTL {
                    guard.0 = load_allowed_domains(&config_path);
                    guard.1 = Instant::now();
                }
                guard.0.clone()
            } else {
                allowed_domains.read().await.0.clone()
            }
        };

        let query = &buf[..len];
        let response = handle_query(query, &domains);

        if let Err(e) = socket.send_to(&response, src).await {
            error!("DNS send error: {}", e);
        }
    }
}

/// Helper to load explicit allowed `.test` domains from the config.
fn load_allowed_domains(config_path: &std::path::Path) -> HashSet<String> {
    let mut domains = HashSet::new();
    if let Ok(config) = DevBindConfig::load(config_path) {
        for route in config.routes {
            domains.insert(route.domain.to_lowercase());
        }
    }
    domains
}

/// Parse a DNS query and build a response.
///
/// We only care about:
/// - A (type 1) queries for explicitly registered `*.test` domains → respond with 127.0.0.1
/// - AAAA (type 28) queries for explicitly registered `*.test` domains → respond with ::1
/// - Everything else (including unregistered `.test` domains) → NXDOMAIN
fn handle_query(query: &[u8], allowed_domains: &HashSet<String>) -> Vec<u8> {
    // --- Parse header (12 bytes) ---
    let id = &query[0..2];
    let flags = u16::from_be_bytes([query[2], query[3]]);
    let qdcount = u16::from_be_bytes([query[4], query[5]]);
    let _opcode = (flags >> 11) & 0xF;

    // Parse the question section to extract the domain name and query type
    let (domain, qtype, question_end) = match parse_question(&query[12..]) {
        Some(v) => v,
        None => return build_error_response(id, 1), // FORMERR
    };

    trace!("DNS query: {} type={} from question", domain, qtype);

    let is_test_domain = domain.ends_with(".test") || domain == "test";

    // Only resolve explicitly registered .test domains.
    // This prevents Chrome search domain leaks (e.g. www.shadcn.io.test)
    // from incorrectly resolving to 127.0.0.1.
    let is_allowed = is_test_domain && allowed_domains.contains(&domain);

    if !is_allowed {
        return build_nxdomain_response(id, query, question_end);
    }

    // Build positive response
    match qtype {
        1 => build_a_response(id, query, question_end, qdcount, Ipv4Addr::LOCALHOST),
        28 => build_aaaa_response(id, query, question_end, qdcount),
        _ => build_empty_response(id, query, question_end, qdcount), // SOA, MX, etc. → empty answer
    }
}

/// Parse a DNS question section. Returns (domain_name, qtype, bytes_consumed).
fn parse_question(data: &[u8]) -> Option<(String, u16, usize)> {
    let mut pos = 0;
    let mut labels = Vec::new();

    loop {
        if pos >= data.len() {
            return None;
        }
        let label_len = data[pos] as usize;
        if label_len == 0 {
            pos += 1; // null terminator
            break;
        }
        if pos + 1 + label_len > data.len() {
            return None;
        }
        let label = std::str::from_utf8(&data[pos + 1..pos + 1 + label_len]).ok()?;
        labels.push(label.to_lowercase());
        pos += 1 + label_len;
    }

    if pos + 4 > data.len() {
        return None;
    }

    let qtype = u16::from_be_bytes([data[pos], data[pos + 1]]);
    // qclass at pos+2..pos+4 (we ignore it, assume IN)
    pos += 4;

    Some((labels.join("."), qtype, 12 + pos)) // 12 for header offset
}

/// Build an A record response (127.0.0.1).
fn build_a_response(
    id: &[u8],
    query: &[u8],
    question_end: usize,
    qdcount: u16,
    ip: Ipv4Addr,
) -> Vec<u8> {
    let mut resp = Vec::with_capacity(question_end + 16);

    // Header
    resp.extend_from_slice(id);
    resp.extend_from_slice(&0x8180u16.to_be_bytes()); // QR=1, AA=1, RD=1, RA=1
    resp.extend_from_slice(&qdcount.to_be_bytes()); // QDCOUNT
    resp.extend_from_slice(&1u16.to_be_bytes()); // ANCOUNT = 1
    resp.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    resp.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT

    // Copy question section
    resp.extend_from_slice(&query[12..question_end]);

    // Answer: pointer to question name (0xC00C), type A, class IN, TTL 60, rdlength 4
    resp.extend_from_slice(&0xC00Cu16.to_be_bytes()); // Name pointer
    resp.extend_from_slice(&1u16.to_be_bytes()); // Type A
    resp.extend_from_slice(&1u16.to_be_bytes()); // Class IN
    resp.extend_from_slice(&60u32.to_be_bytes()); // TTL 60s
    resp.extend_from_slice(&4u16.to_be_bytes()); // RDLENGTH
    resp.extend_from_slice(&ip.octets()); // 127.0.0.1

    resp
}

/// Build an AAAA record response (::1).
fn build_aaaa_response(id: &[u8], query: &[u8], question_end: usize, qdcount: u16) -> Vec<u8> {
    let mut resp = Vec::with_capacity(question_end + 28);

    resp.extend_from_slice(id);
    resp.extend_from_slice(&0x8180u16.to_be_bytes());
    resp.extend_from_slice(&qdcount.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());

    resp.extend_from_slice(&query[12..question_end]);

    resp.extend_from_slice(&0xC00Cu16.to_be_bytes()); // Name pointer
    resp.extend_from_slice(&28u16.to_be_bytes()); // Type AAAA
    resp.extend_from_slice(&1u16.to_be_bytes()); // Class IN
    resp.extend_from_slice(&60u32.to_be_bytes()); // TTL 60s
    resp.extend_from_slice(&16u16.to_be_bytes()); // RDLENGTH
    resp.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]); // ::1

    resp
}

/// Build an empty response (no answers, but NOERROR).
fn build_empty_response(id: &[u8], query: &[u8], question_end: usize, qdcount: u16) -> Vec<u8> {
    let mut resp = Vec::with_capacity(question_end);

    resp.extend_from_slice(id);
    resp.extend_from_slice(&0x8180u16.to_be_bytes());
    resp.extend_from_slice(&qdcount.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT = 0
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());

    resp.extend_from_slice(&query[12..question_end]);

    resp
}

/// Build an NXDOMAIN response.
fn build_nxdomain_response(id: &[u8], query: &[u8], question_end: usize) -> Vec<u8> {
    let mut resp = Vec::with_capacity(question_end);

    resp.extend_from_slice(id);
    resp.extend_from_slice(&0x8183u16.to_be_bytes()); // QR=1, AA=1, RD=1, RA=1, RCODE=3 (NXDOMAIN)
    resp.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT
    resp.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT
    resp.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    resp.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT

    resp.extend_from_slice(&query[12..question_end]);

    resp
}

/// Build a FORMERR response (format error).
fn build_error_response(id: &[u8], rcode: u8) -> Vec<u8> {
    let mut resp = Vec::with_capacity(12);

    resp.extend_from_slice(id);
    let flags = 0x8100u16 | (rcode as u16);
    resp.extend_from_slice(&flags.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());
    resp.extend_from_slice(&0u16.to_be_bytes());

    resp
}

#[cfg(test)]
#[path = "dns_tests.rs"]
mod tests;
