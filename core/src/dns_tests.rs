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

    // ── A record (type 1) ────────────────────────────────────────────────────

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
    fn test_a_query_id_is_mirrored() {
        // Build query with a distinct ID [0xDE, 0xAD]
        let mut query = build_test_query("mirror.test", 1);
        query[0] = 0xDE;
        query[1] = 0xAD;
        let resp = handle_query(&query);
        assert_eq!(resp[0], 0xDE, "Response ID byte 0 must mirror query");
        assert_eq!(resp[1], 0xAD, "Response ID byte 1 must mirror query");
    }

    // ── NXDOMAIN ────────────────────────────────────────────────────────────

    #[test]
    fn test_nxdomain_for_non_test_domain() {
        let query = build_test_query("example.com", 1);
        let resp = handle_query(&query);

        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(flags & 0x000F, 3, "RCODE should be NXDOMAIN");
    }

    #[test]
    fn test_nxdomain_for_org_domain() {
        let query = build_test_query("openai.org", 1);
        let resp = handle_query(&query);
        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(flags & 0x000F, 3, "*.org must be NXDOMAIN");
    }

    #[test]
    fn test_nxdomain_for_io_domain() {
        let query = build_test_query("myapp.io", 1);
        let resp = handle_query(&query);
        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(flags & 0x000F, 3, "*.io must be NXDOMAIN");
    }

    #[test]
    fn test_nxdomain_for_test_lookalike_domain() {
        // "nottest" must not be confused with ".test"
        let query = build_test_query("myapp.nottest", 1);
        let resp = handle_query(&query);
        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(flags & 0x000F, 3, "non-.test TLD must be NXDOMAIN");
    }

    // ── AAAA record (type 28) ────────────────────────────────────────────────

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

    // ── Special .test forms ──────────────────────────────────────────────────

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

    #[test]
    fn test_uppercase_test_domain_resolves() {
        // Label parsing lowercases; uppercase MYAPP.TEST must still resolve
        let query = build_test_query("MYAPP.TEST", 1);
        let resp = handle_query(&query);
        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(
            flags & 0x000F,
            0,
            "uppercase .TEST must resolve (case-insensitive)"
        );
        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 1);
    }

    // ── Unknown query types ──────────────────────────────────────────────────

    #[test]
    fn test_mx_query_for_test_domain_returns_empty_answer() {
        // Type 15 = MX — should get NOERROR with 0 answers
        let query = build_test_query("myapp.test", 15);
        let resp = handle_query(&query);
        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(flags & 0x000F, 0, "MX query must not error");
        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 0, "MX query on .test must return 0 answers");
    }

    #[test]
    fn test_soa_query_for_test_domain_returns_empty_answer() {
        // Type 6 = SOA
        let query = build_test_query("myapp.test", 6);
        let resp = handle_query(&query);
        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        assert_eq!(flags & 0x000F, 0, "SOA query must be NOERROR");
        let ancount = u16::from_be_bytes([resp[6], resp[7]]);
        assert_eq!(ancount, 0, "SOA query on .test must return 0 answers");
    }

    // ── Malformed packet ─────────────────────────────────────────────────────

    #[test]
    fn test_truncated_packet_returns_formerr() {
        // Anything shorter than 12 bytes is filtered in run_dns_server,
        // but handle_query itself should return a FORMERR if question parsing fails.
        // A 12-byte header with no question section will fail parse_question.
        let short_pkt = vec![
            0xCA, 0xFE, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let resp = handle_query(&short_pkt);
        // Must return at least an 8-byte response (ID + flags + counts)
        assert!(resp.len() >= 4, "even FORMERR must have at least ID+flags");
        let flags = u16::from_be_bytes([resp[2], resp[3]]);
        // RCODE should be 1 (FORMERR) when the question is missing
        assert_eq!(
            flags & 0x000F,
            1,
            "truncated question section must return FORMERR"
        );
        // ID must still be mirrored
        assert_eq!(resp[0], 0xCA);
        assert_eq!(resp[1], 0xFE);
    }
