//! Minimal embedded DNS server for DevBind.
//!
//! Resolves **all** `*.test` A queries to `127.0.0.1` and AAAA to `::1`.
//! Returns NXDOMAIN for anything outside `.test`.
//!
//! Wire format reference: RFC 1035 §4.

use std::net::Ipv4Addr;
use tokio::net::UdpSocket;
use tracing::{error, info, trace};

/// Default listen address for the DevBind DNS server.
pub const DNS_LISTEN_ADDR: &str = "127.0.2.1:53";

/// Start the DNS server. This function runs forever (call via `tokio::spawn`).
pub async fn run_dns_server(listen_addr: &str) {
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

        let query = &buf[..len];
        let response = handle_query(query);

        if let Err(e) = socket.send_to(&response, src).await {
            error!("DNS send error: {}", e);
        }
    }
}

/// Parse a DNS query and build a response.
///
/// We only care about:
/// - A (type 1) queries for `*.test` → respond with 127.0.0.1
/// - AAAA (type 28) queries for `*.test` → respond with ::1
/// - Everything else → NXDOMAIN
fn handle_query(query: &[u8]) -> Vec<u8> {
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

    if !is_test_domain {
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
mod tests {
    use super::*;

    /// Build a minimal DNS query packet for testing.
    fn build_test_query(domain: &str, qtype: u16) -> Vec<u8> {
        let mut pkt = Vec::new();

        // Header
        pkt.extend_from_slice(&[0xAA, 0xBB]); // ID
        pkt.extend_from_slice(&0x0100u16.to_be_bytes()); // Standard query, RD=1
        pkt.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT = 1
        pkt.extend_from_slice(&0u16.to_be_bytes()); // ANCOUNT
        pkt.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
        pkt.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT

        // Question: encode labels
        for label in domain.split('.') {
            pkt.push(label.len() as u8);
            pkt.extend_from_slice(label.as_bytes());
        }
        pkt.push(0); // End of name

        pkt.extend_from_slice(&qtype.to_be_bytes()); // QTYPE
        pkt.extend_from_slice(&1u16.to_be_bytes()); // QCLASS = IN

        pkt
    }

    #[test]
    fn test_a_query_for_test_domain() {
        let query = build_test_query("myapp.test", 1);
        let resp = handle_query(&query);

        // Check header
        assert_eq!(resp[0], 0xAA); // ID preserved
        assert_eq!(resp[1], 0xBB);

        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(flags & 0x000F, 0, "RCODE should be NOERROR");

        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 1, "Should have 1 answer");

        // Last 4 bytes should be 127.0.0.1
        let ip_bytes = &resp[resp.len() - 4..];
        assert_eq!(ip_bytes, &[127, 0, 0, 1]);
    }

    #[test]
    fn test_nxdomain_for_non_test_domain() {
        let query = build_test_query("example.com", 1);
        let resp = handle_query(&query);

        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(flags & 0x000F, 3, "RCODE should be NXDOMAIN");
    }

    #[test]
    fn test_aaaa_query_for_test_domain() {
        let query = build_test_query("myapp.test", 28);
        let resp = handle_query(&query);

        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 1, "Should have 1 AAAA answer");

        // Last 16 bytes should be ::1
        let ip6_bytes = &resp[resp.len() - 16..];
        let mut expected = [0u8; 16];
        expected[15] = 1;
        assert_eq!(ip6_bytes, &expected);
    }

    #[test]
    fn test_bare_test_domain() {
        let query = build_test_query("test", 1);
        let resp = handle_query(&query);

        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(flags & 0x000F, 0, "bare .test should resolve");

        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 1);
    }

    #[test]
    fn test_deeply_nested_test_subdomain() {
        let query = build_test_query("a.b.c.d.test", 1);
        let resp = handle_query(&query);

        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 1, "deeply nested .test should resolve");
    }
}
