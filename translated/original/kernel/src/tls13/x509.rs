//! Minimal X.509 Certificate Parsing
//!
//! Basic ASN.1/DER parsing for X.509 certificates.
//! Note: This is a minimal implementation for TLS 1.3 compatibility.

use alloc::vec::Vec;
use alloc::string::String;

/// ASN.1 tag types
mod asn1_tags {
    pub const BOOLEAN: u8 = 0x01;
    pub const INTEGER: u8 = 0x02;
    pub const BIT_STRING: u8 = 0x03;
    pub const OCTET_STRING: u8 = 0x04;
    pub const NULL: u8 = 0x05;
    pub const OID: u8 = 0x06;
    pub const UTF8_STRING: u8 = 0x0C;
    pub const PRINTABLE_STRING: u8 = 0x13;
    pub const IA5_STRING: u8 = 0x16;
    pub const UTC_TIME: u8 = 0x17;
    pub const GENERALIZED_TIME: u8 = 0x18;
    pub const SEQUENCE: u8 = 0x30;
    pub const SET: u8 = 0x31;
    pub const CONTEXT_0: u8 = 0xA0;
    pub const CONTEXT_1: u8 = 0xA1;
    pub const CONTEXT_2: u8 = 0xA2;
    pub const CONTEXT_3: u8 = 0xA3;
}

/// Parse ASN.1 length
fn parse_length(data: &[u8], pos: &mut usize) -> Option<usize> {
    if *pos >= data.len() {
        return None;
    }
    
    let first = data[*pos];
    *pos += 1;
    
    if first < 0x80 {
        Some(first as usize)
    } else if first == 0x81 {
        if *pos >= data.len() {
            return None;
        }
        let len = data[*pos] as usize;
        *pos += 1;
        Some(len)
    } else if first == 0x82 {
        if *pos + 1 >= data.len() {
            return None;
        }
        let len = ((data[*pos] as usize) << 8) | (data[*pos + 1] as usize);
        *pos += 2;
        Some(len)
    } else if first == 0x83 {
        if *pos + 2 >= data.len() {
            return None;
        }
        let len = ((data[*pos] as usize) << 16) 
            | ((data[*pos + 1] as usize) << 8) 
            | (data[*pos + 2] as usize);
        *pos += 3;
        Some(len)
    } else {
        None
    }
}

/// Parse an ASN.1 element
fn parse_element<'a>(data: &'a [u8], pos: &mut usize) -> Option<(u8, &'a [u8])> {
    if *pos >= data.len() {
        return None;
    }
    
    let tag = data[*pos];
    *pos += 1;
    
    let len = parse_length(data, pos)?;
    
    if *pos + len > data.len() {
        return None;
    }
    
    let value = &data[*pos..*pos + len];
    *pos += len;
    
    Some((tag, value))
}

/// X.509 Certificate (minimal representation)
#[derive(Debug)]
pub struct Certificate {
    /// Raw DER-encoded certificate
    pub raw: Vec<u8>,
    /// Subject Common Name
    pub subject_cn: Option<String>,
    /// Issuer Common Name
    pub issuer_cn: Option<String>,
    /// Not Before (as string)
    pub not_before: Option<String>,
    /// Not After (as string)
    pub not_after: Option<String>,
    /// Subject Alternative Names
    pub san: Vec<String>,
    /// Public key algorithm OID
    pub pubkey_algo: Vec<u8>,
    /// Public key data
    pub pubkey: Vec<u8>,
}

impl Certificate {
    /// Parse a DER-encoded X.509 certificate
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut pos = 0;
        
        // Certificate SEQUENCE
        let (tag, cert_data) = parse_element(data, &mut pos)?;
        if tag != asn1_tags::SEQUENCE {
            return None;
        }
        
        let mut cert_pos = 0;
        
        // TBSCertificate SEQUENCE
        let (tag, tbs_data) = parse_element(cert_data, &mut cert_pos)?;
        if tag != asn1_tags::SEQUENCE {
            return None;
        }
        
        // Parse TBSCertificate
        let mut tbs_pos = 0;
        
        // Version (optional, context-specific [0])
        if tbs_pos < tbs_data.len() && tbs_data[tbs_pos] == asn1_tags::CONTEXT_0 {
            let _ = parse_element(tbs_data, &mut tbs_pos)?;
        }
        
        // Serial number
        let _ = parse_element(tbs_data, &mut tbs_pos)?;
        
        // Signature algorithm
        let _ = parse_element(tbs_data, &mut tbs_pos)?;
        
        // Issuer
        let (tag, issuer_data) = parse_element(tbs_data, &mut tbs_pos)?;
        let issuer_cn = if tag == asn1_tags::SEQUENCE {
            extract_cn(issuer_data)
        } else {
            None
        };
        
        // Validity
        let (tag, validity_data) = parse_element(tbs_data, &mut tbs_pos)?;
        let (not_before, not_after) = if tag == asn1_tags::SEQUENCE {
            parse_validity(validity_data)
        } else {
            (None, None)
        };
        
        // Subject
        let (tag, subject_data) = parse_element(tbs_data, &mut tbs_pos)?;
        let subject_cn = if tag == asn1_tags::SEQUENCE {
            extract_cn(subject_data)
        } else {
            None
        };
        
        // Subject Public Key Info
        let (tag, spki_data) = parse_element(tbs_data, &mut tbs_pos)?;
        let (pubkey_algo, pubkey) = if tag == asn1_tags::SEQUENCE {
            parse_spki(spki_data)
        } else {
            (Vec::new(), Vec::new())
        };
        
        // Extensions (optional, context-specific [3])
        let mut san = Vec::new();
        while tbs_pos < tbs_data.len() {
            if let Some((tag, ext_wrapper)) = parse_element(tbs_data, &mut tbs_pos) {
                if tag == asn1_tags::CONTEXT_3 {
                    // Extensions SEQUENCE
                    let mut ext_pos = 0;
                    if let Some((_, extensions)) = parse_element(ext_wrapper, &mut ext_pos) {
                        san = parse_san_extension(extensions);
                    }
                }
            }
        }
        
        Some(Certificate {
            raw: data.to_vec(),
            subject_cn,
            issuer_cn,
            not_before,
            not_after,
            san,
            pubkey_algo,
            pubkey,
        })
    }
    
    /// Check if the certificate is valid for a given hostname
    pub fn valid_for_hostname(&self, hostname: &str) -> bool {
        // Check SAN first
        for name in &self.san {
            if matches_hostname(name, hostname) {
                return true;
            }
        }
        
        // Fall back to CN
        if let Some(cn) = &self.subject_cn {
            if matches_hostname(cn, hostname) {
                return true;
            }
        }
        
        false
    }
}

/// Extract Common Name from a Name SEQUENCE
fn extract_cn(data: &[u8]) -> Option<String> {
    // OID for commonName: 2.5.4.3 = 55 04 03
    let cn_oid = [0x55, 0x04, 0x03];
    
    let mut pos = 0;
    while pos < data.len() {
        if let Some((tag, rdn_set)) = parse_element(data, &mut pos) {
            if tag == asn1_tags::SET {
                let mut set_pos = 0;
                if let Some((_, rdn_seq)) = parse_element(rdn_set, &mut set_pos) {
                    let mut seq_pos = 0;
                    if let Some((_, oid)) = parse_element(rdn_seq, &mut seq_pos) {
                        if oid == cn_oid {
                            if let Some((_, value)) = parse_element(rdn_seq, &mut seq_pos) {
                                return Some(String::from_utf8_lossy(value).into_owned());
                            }
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    
    None
}

/// Parse Validity SEQUENCE
fn parse_validity(data: &[u8]) -> (Option<String>, Option<String>) {
    let mut pos = 0;
    
    let not_before = if let Some((tag, value)) = parse_element(data, &mut pos) {
        if tag == asn1_tags::UTC_TIME || tag == asn1_tags::GENERALIZED_TIME {
            Some(String::from_utf8_lossy(value).into_owned())
        } else {
            None
        }
    } else {
        None
    };
    
    let not_after = if let Some((tag, value)) = parse_element(data, &mut pos) {
        if tag == asn1_tags::UTC_TIME || tag == asn1_tags::GENERALIZED_TIME {
            Some(String::from_utf8_lossy(value).into_owned())
        } else {
            None
        }
    } else {
        None
    };
    
    (not_before, not_after)
}

/// Parse Subject Public Key Info
fn parse_spki(data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut pos = 0;
    
    // Algorithm
    let algo = if let Some((tag, algo_seq)) = parse_element(data, &mut pos) {
        if tag == asn1_tags::SEQUENCE {
            let mut algo_pos = 0;
            if let Some((_, oid)) = parse_element(algo_seq, &mut algo_pos) {
                oid.to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    // Public key (BIT STRING)
    let pubkey = if let Some((tag, value)) = parse_element(data, &mut pos) {
        if tag == asn1_tags::BIT_STRING && !value.is_empty() {
            // Skip the unused bits count
            value[1..].to_vec()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    (algo, pubkey)
}

/// Parse Subject Alternative Name extension
fn parse_san_extension(extensions: &[u8]) -> Vec<String> {
    // OID for subjectAltName: 2.5.29.17 = 55 1D 11
    let san_oid = [0x55, 0x1D, 0x11];
    let mut result = Vec::new();
    
    let mut pos = 0;
    while pos < extensions.len() {
        if let Some((tag, ext_seq)) = parse_element(extensions, &mut pos) {
            if tag == asn1_tags::SEQUENCE {
                let mut ext_pos = 0;
                if let Some((_, oid)) = parse_element(ext_seq, &mut ext_pos) {
                    if oid == san_oid {
                        // Skip critical (if present)
                        if ext_pos < ext_seq.len() && ext_seq[ext_pos] == asn1_tags::BOOLEAN {
                            let _ = parse_element(ext_seq, &mut ext_pos);
                        }
                        
                        // Extension value (OCTET STRING containing SEQUENCE)
                        if let Some((_, octet)) = parse_element(ext_seq, &mut ext_pos) {
                            let mut san_pos = 0;
                            if let Some((_, san_seq)) = parse_element(octet, &mut san_pos) {
                                result = parse_general_names(san_seq);
                            }
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    
    result
}

/// Parse GeneralNames sequence
fn parse_general_names(data: &[u8]) -> Vec<String> {
    let mut result = Vec::new();
    let mut pos = 0;
    
    while pos < data.len() {
        let tag = data[pos];
        pos += 1;
        
        if let Some(len) = parse_length(data, &mut pos) {
            if pos + len <= data.len() {
                // Context-specific [2] = dNSName
                if tag == 0x82 {
                    let name = String::from_utf8_lossy(&data[pos..pos + len]).into_owned();
                    result.push(name);
                }
                pos += len;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    result
}

/// Check if a certificate name matches a hostname
fn matches_hostname(pattern: &str, hostname: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let hostname = hostname.to_lowercase();
    
    if pattern.starts_with("*.") {
        // Wildcard match
        let suffix = &pattern[2..];
        if let Some(dot_pos) = hostname.find('.') {
            return &hostname[dot_pos + 1..] == suffix;
        }
        false
    } else {
        pattern == hostname
    }
}

/// Parse a certificate chain from TLS Certificate message
pub fn parse_certificate_chain(data: &[u8]) -> Vec<Certificate> {
    let mut result = Vec::new();
    
    if data.len() < 7 {
        return result;
    }
    
    // Skip handshake header (1 + 3 bytes)
    let mut pos = 4;
    
    // Certificate request context (1 byte length + data)
    if pos >= data.len() {
        return result;
    }
    let ctx_len = data[pos] as usize;
    pos += 1 + ctx_len;
    
    // Certificate list length (3 bytes)
    if pos + 3 > data.len() {
        return result;
    }
    let list_len = ((data[pos] as usize) << 16)
        | ((data[pos + 1] as usize) << 8)
        | (data[pos + 2] as usize);
    pos += 3;
    
    let list_end = pos + list_len;
    
    while pos + 3 <= list_end {
        // Certificate entry length (3 bytes)
        let cert_len = ((data[pos] as usize) << 16)
            | ((data[pos + 1] as usize) << 8)
            | (data[pos + 2] as usize);
        pos += 3;
        
        if pos + cert_len > data.len() {
            break;
        }
        
        // Parse certificate
        if let Some(cert) = Certificate::parse(&data[pos..pos + cert_len]) {
            result.push(cert);
        }
        pos += cert_len;
        
        // Skip extensions (2 bytes length + data)
        if pos + 2 <= list_end {
            let ext_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2 + ext_len;
        }
    }
    
    result
}
