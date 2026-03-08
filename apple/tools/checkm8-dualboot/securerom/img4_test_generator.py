#!/usr/bin/env python3
"""
T8020 B1 SecureROM — IMG4/DER Malformed Payload Generator
============================================================
Generates structurally-targeted malformed IMG4/DER payloads designed to
trigger specific vulnerability patterns identified in the parser audit.

Each test case targets a specific parser code path:
  1. DER length overflow (integer wrap via long-form length)
  2. Deep ASN.1 nesting (stack exhaustion)
  3. Tag type confusion (unexpected tags at expected positions)
  4. Truncated structures (premature EOF in various positions)
  5. IM4P component fuzzing (malformed inner structures)
  6. IMG4 manifest size boundary (exactly at various comparison limits)
  7. OCTET STRING size mismatch (claimed vs actual)
  8. Negative / zero length edge cases
  9. Maximum-size fields (0x7F, 0x80 boundary)
  10. Cross-structure pointer confusion

Output: Binary .bin files ready for DFU DNLOAD testing
        + JSON manifest describing each test case
"""

import struct, os, json, hashlib
from pathlib import Path

OUTPUT_DIR = Path(__file__).parent / "test_payloads"
OUTPUT_DIR.mkdir(exist_ok=True)

class DERBuilder:
    """Helper to construct DER/ASN.1 structures."""
    
    @staticmethod
    def tag(tag_byte):
        """Single-byte tag."""
        return bytes([tag_byte])
    
    @staticmethod
    def length(size):
        """Standard DER length encoding."""
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
    def length_raw(length_bytes):
        """Raw length bytes (for crafting malformed lengths)."""
        return bytes(length_bytes)
    
    @staticmethod
    def sequence(contents):
        """SEQUENCE (0x30) wrapper."""
        data = b''.join(contents) if isinstance(contents, (list, tuple)) else contents
        return b'\x30' + DERBuilder.length(len(data)) + data
    
    @staticmethod
    def set_of(contents):
        """SET (0x31) wrapper."""
        data = b''.join(contents) if isinstance(contents, (list, tuple)) else contents
        return b'\x31' + DERBuilder.length(len(data)) + data
    
    @staticmethod
    def octet_string(data):
        """OCTET STRING (0x04)."""
        return b'\x04' + DERBuilder.length(len(data)) + data
    
    @staticmethod
    def ia5_string(text):
        """IA5String (0x16)."""
        data = text.encode('ascii') if isinstance(text, str) else text
        return b'\x16' + DERBuilder.length(len(data)) + data
    
    @staticmethod
    def integer(value, width=None):
        """INTEGER (0x02)."""
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
        return b'\x02' + DERBuilder.length(len(data)) + data
    
    @staticmethod
    def boolean(val):
        """BOOLEAN (0x01)."""
        return b'\x01\x01' + (b'\xFF' if val else b'\x00')
    
    @staticmethod
    def context_tag(num, contents, constructed=True):
        """Context-specific tag [num]."""
        cls = 0xA0 if constructed else 0x80
        data = b''.join(contents) if isinstance(contents, (list, tuple)) else contents
        return bytes([cls | (num & 0x1F)]) + DERBuilder.length(len(data)) + data
    
    @staticmethod
    def raw_tlv(tag_byte, length_bytes, value):
        """Build a TLV with raw (potentially malformed) length bytes."""
        return bytes([tag_byte]) + bytes(length_bytes) + value


D = DERBuilder

def build_img4_container(payload_data, manifest_data=None):
    """Build a minimal IMG4 container wrapping the payload."""
    # IMG4 is: SEQUENCE { IA5 "IMG4", payload, [0] manifest }
    parts = [D.ia5_string("IMG4"), payload_data]
    if manifest_data:
        parts.append(D.context_tag(0, manifest_data))
    return D.sequence(parts)

def build_im4p(component_tag=b"illb", description=b"iBoot", data=b"\x00" * 16):
    """Build a minimal IM4P structure."""
    # IM4P is: SEQUENCE { IA5 "IM4P", IA5 component, IA5 desc, OCTET data }
    return D.sequence([
        D.ia5_string("IM4P"),
        D.ia5_string(component_tag),
        D.ia5_string(description),
        D.octet_string(data),
    ])

def build_im4m(properties=None):
    """Build a minimal IM4M (manifest)."""
    if properties is None:
        properties = D.sequence([D.integer(0)])
    return D.sequence([
        D.ia5_string("IM4M"),
        D.integer(0),  # version
        D.set_of(properties),
    ])


# ============================================================
# TEST CASE GENERATORS
# ============================================================
test_cases = []

def register_case(name, payload, description, target_func, severity, vuln_type):
    """Register a test case."""
    fname = f"tc_{len(test_cases):03d}_{name}.bin"
    fpath = OUTPUT_DIR / fname
    fpath.write_bytes(payload)
    tc = {
        "id": len(test_cases),
        "name": name,
        "file": fname,
        "size": len(payload),
        "sha256": hashlib.sha256(payload).hexdigest(),
        "description": description,
        "target_function": target_func,
        "severity": severity,
        "vuln_type": vuln_type,
        "hex_preview": payload[:64].hex(),
    }
    test_cases.append(tc)
    print(f"  [{tc['id']:3d}] {severity:8s} {name:40s} ({len(payload):5d} bytes) -> {target_func}")
    return tc

# ────────────────────────────────────────────────────────────
# Category 1: DER Length Field Overflow
# ────────────────────────────────────────────────────────────
print("\n=== Category 1: DER Length Field Overflow ===\n")

# TC 1.1: Long-form length with 4 bytes claiming 0xFFFFFFFF
payload = b'\x30' + bytes([0x84, 0xFF, 0xFF, 0xFF, 0xFF]) + b'\x00' * 100
register_case("der_len_overflow_4byte_max", payload,
    "SEQUENCE with 4-byte long-form length = 0xFFFFFFFF. Parser must handle "
    "this as > remaining buffer without integer overflow in size calculation.",
    "0x10000D000+", "CRITICAL", "INTEGER_OVERFLOW")

# TC 1.2: Long-form length with 8 bytes (count=0x88)
# length byte = 0x88 → 8 subsequent bytes specify the length
# This creates a 64-bit length value that wraps to small when truncated to 32-bit
payload = b'\x30' + bytes([0x88, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x10]) + b'\x00' * 100
register_case("der_len_overflow_8byte", payload,
    "SEQUENCE with 8-byte long-form length (0x88). If parser only reads 4 bytes, "
    "remaining bytes become content → tag confusion. If it reads all 8, "
    "the 64-bit value could truncate to 0x10 in 32-bit comparison.",
    "0x10000D000+", "CRITICAL", "INTEGER_OVERFLOW")

# TC 1.3: Length byte = 0x85 (5-byte length, exceeds 32-bit)
payload = b'\x30' + bytes([0x85, 0x01, 0x00, 0x00, 0x00, 0x20]) + b'\x00' * 40
register_case("der_len_5byte_overflow", payload,
    "5-byte long-form DER length (0x85). Value = 0x0100000020 = 4GB+32. "
    "If truncated to 32-bit → only 0x20 (32 bytes). Parser reads 32 bytes "
    "but thinks structure is 4GB → subsequent parsing OOB.",
    "0x10000D000+", "CRITICAL", "INTEGER_OVERFLOW")

# TC 1.4: Long-form with count=0 (0x80 = indefinite form, invalid in DER but...)
payload = b'\x30\x80' + b'\x00' * 100 + b'\x00\x00'  # BER indefinite form
register_case("der_len_indefinite_form", payload,
    "DER indefinite form (0x80). Illegal in DER but parsers may handle it. "
    "Could cause unbounded read until 0x00 0x00 end-of-contents marker.",
    "0x10000D000+", "HIGH", "MISSING_VALIDATION")

# TC 1.5: Claimed length exactly = buffer remaining (off-by-one)
inner = b'\x02\x01\x00'  # INTEGER 0
claimed_len = 0x7F0  # Almost fills entire 0x800 DFU buffer
pad = b'\x00' * (claimed_len - len(inner))
payload = b'\x30' + D.length(claimed_len) + inner + pad
register_case("der_len_exact_boundary", payload[:0x800],
    "SEQUENCE with length exactly at DFU buffer boundary. Tests parser "
    "behavior when claimed structure size = available data.",
    "0x10000D000+", "MEDIUM", "BOUNDARY_CONDITION")

# TC 1.6: Nested structures with lengths that sum > parent
inner_a = D.octet_string(b'\x41' * 0x100)
inner_b = D.octet_string(b'\x42' * 0x100)
# Outer claims 0x50 bytes but inner_a + inner_b = 0x208
outer = b'\x30' + D.length(0x50) + inner_a + inner_b
payload = outer + b'\x00' * 0x100
register_case("der_children_exceed_parent", payload,
    "SEQUENCE claims 0x50 bytes but children total 0x208 bytes. "
    "Tests if parser validates sum(child_lengths) <= parent_length.",
    "0x10000D000+", "HIGH", "LENGTH_MISMATCH")


# ────────────────────────────────────────────────────────────
# Category 2: Deep ASN.1 Nesting (Stack Overflow)
# ────────────────────────────────────────────────────────────
print("\n=== Category 2: Deep ASN.1 Nesting ===\n")

# TC 2.1: 200 nested SEQUENCEs
def make_nested_seq(depth, leaf=b'\x02\x01\x00'):
    data = leaf
    for _ in range(depth):
        data = D.sequence(data)
    return data

for depth in [50, 100, 200, 400]:
    try:
        payload = make_nested_seq(depth)
        if len(payload) < 0x800:
            register_case(f"asn1_nesting_depth_{depth}", payload,
                f"{depth} nested SEQUENCE tags. SecureROM stack is limited. "
                f"Each recursive parser call uses ~0x30-0x60 stack bytes → "
                f"at depth {depth}: ~{depth * 0x40:#x} bytes of stack consumed.",
                "0x10000D000+ recursive", "CRITICAL" if depth >= 200 else "HIGH",
                "STACK_OVERFLOW")
    except RecursionError:
        pass

# TC 2.2: Mixed nesting (SEQUENCE + SET + context tags)
def make_mixed_nesting(depth):
    data = D.integer(0xDEAD)
    for i in range(depth):
        if i % 3 == 0:
            data = D.sequence(data)
        elif i % 3 == 1:
            data = D.set_of(data)
        else:
            data = D.context_tag(i % 4, data)
    return data

payload = make_mixed_nesting(150)
if len(payload) < 0x800:
    register_case("asn1_mixed_nesting_150", payload,
        "150 levels of mixed SEQUENCE/SET/context tags. Each tag type may "
        "use different parser dispatch, increasing stack usage.",
        "0x10000D000+", "HIGH", "STACK_OVERFLOW")


# ────────────────────────────────────────────────────────────
# Category 3: Tag Type Confusion
# ────────────────────────────────────────────────────────────
print("\n=== Category 3: Tag Type Confusion ===\n")

# TC 3.1: IMG4 with wrong inner type (BOOLEAN instead of IA5STRING for tag)
bad_img4 = D.sequence([
    D.boolean(True),  # Should be IA5STRING "IMG4"
    build_im4p(),
])
register_case("img4_tag_type_confusion", bad_img4,
    "IMG4 container with BOOLEAN where IA5STRING 'IMG4' expected. "
    "Tests if parser checks tag type before processing value.",
    "0x100005480", "MEDIUM", "TYPE_CONFUSION")

# TC 3.2: IM4P with INTEGER where OCTET STRING expected (payload field)
bad_im4p = D.sequence([
    D.ia5_string("IM4P"),
    D.ia5_string("illb"),
    D.ia5_string("iBoot"),
    D.integer(0x4141414141414141),  # INTEGER instead of OCTET STRING
])
bad_container = build_img4_container(bad_im4p)
register_case("im4p_payload_type_confusion", bad_container,
    "IM4P with INTEGER (0x02) where OCTET STRING (0x04) expected for payload. "
    "If parser reads length without checking tag → type confusion.",
    "0x100004800+", "HIGH", "TYPE_CONFUSION")

# TC 3.3: Context tag with impossible number
bad_ctx = D.sequence([
    D.ia5_string("IMG4"),
    build_im4p(),
    bytes([0xBF, 0x1F]) + D.length(10) + b'\x00' * 10,  # context tag [31] long form
])
register_case("img4_context_tag_31", bad_ctx,
    "IMG4 with context tag [31] (long-form tag encoding, 0xBF 0x1F). "
    "If dispatch table is indexed by tag number without bounds → OOB!",
    "0x1000211A0", "CRITICAL", "DISPATCH_OOB")

# TC 3.4: All 256 possible tag values at IMG4 payload position
for tag_val in [0x00, 0x01, 0x03, 0x05, 0x06, 0x0C, 0x13, 0x17, 0x18, 
                0x1E, 0x1F, 0x30, 0x31, 0x40, 0x80, 0xA0, 0xC0, 0xE0, 0xFF]:
    data = bytes([tag_val]) + D.length(4) + b'\x41' * 4
    container = D.sequence([D.ia5_string("IMG4"), data])
    register_case(f"img4_tag_value_0x{tag_val:02x}", container,
        f"IMG4 with tag byte 0x{tag_val:02X} at payload position. "
        f"Tests parser tag dispatch for unusual/reserved tag values.",
        "0x10000D000+", "MEDIUM", "TAG_FUZZ")


# ────────────────────────────────────────────────────────────
# Category 4: Truncated Structures
# ────────────────────────────────────────────────────────────
print("\n=== Category 4: Truncated Structures ===\n")

# Build a valid IMG4 and truncate at various offsets
valid_img4 = build_img4_container(build_im4p(data=b'\x00' * 256), build_im4m())
for trunc_off in [1, 2, 3, 4, 8, 16, 32, 64, len(valid_img4) // 2, len(valid_img4) - 1]:
    payload = valid_img4[:trunc_off]
    register_case(f"img4_truncated_at_{trunc_off}", payload,
        f"Valid IMG4 container truncated after {trunc_off} bytes. "
        f"Tests every parser path for premature EOF handling.",
        "0x10000A704", "HIGH", "TRUNCATION")

# TC 4.1: Tag present but length missing
payload = b'\x30'  # Just the SEQUENCE tag, no length
register_case("der_tag_only_no_length", payload,
    "Bare SEQUENCE tag (0x30) with no length byte. "
    "Parser must not read beyond buffer.",
    "0x10000D000+", "HIGH", "TRUNCATION")

# TC 4.2: Tag + length but no value
payload = b'\x30\x82\x01\x00'  # SEQUENCE, length=256, but no data
register_case("der_tag_length_no_value", payload,
    "SEQUENCE with length=256 but no value bytes. "
    "Tests if parser validates available data before reading.",
    "0x10000D000+", "HIGH", "TRUNCATION")


# ────────────────────────────────────────────────────────────
# Category 5: IM4P Component Fuzzing
# ────────────────────────────────────────────────────────────
print("\n=== Category 5: IM4P Component Fuzzing ===\n")

# TC 5.1: IM4P with maximum-size OCTET STRING
big_payload = b'\xCC' * 0x700  # Fill most of DFU buffer
im4p_big = build_im4p(data=big_payload)
register_case("im4p_large_payload", im4p_big[:0x800],
    "IM4P with 0x700-byte payload filling most of the 0x800 DFU buffer.",
    "0x100004800+", "MEDIUM", "BOUNDARY")

# TC 5.2: IM4P with compression tag (LZSS/LZFSE magic)
# If the ROM checks for compression, it might try to decompress
compressed_marker = b'\x62\x76\x78\x32'  # bvx2 = LZFSE
im4p_lzfse = D.sequence([
    D.ia5_string("IM4P"),
    D.ia5_string("illb"),
    D.ia5_string("iBoot"),
    D.octet_string(compressed_marker + b'\xFF' * 100),
])
register_case("im4p_fake_lzfse", im4p_lzfse,
    "IM4P with LZFSE magic (bvx2) in payload. If ROM tries to decompress, "
    "malformed LZFSE data could cause heap overflow in decompressor.",
    "0x100004800+", "HIGH", "DECOMPRESSION")

# LZSS variant
im4p_lzss = D.sequence([
    D.ia5_string("IM4P"),
    D.ia5_string("illb"),
    D.ia5_string("iBoot"),
    D.octet_string(b'complzss' + struct.pack('<I', 0x10000) + b'\xFF' * 100),
])
register_case("im4p_fake_lzss", im4p_lzss,
    "IM4P with LZSS magic and claimed decompressed size 0x10000. "
    "Decompressor heap overflow if output buffer allocated from claimed size.",
    "0x100004800+", "HIGH", "DECOMPRESSION")

# TC 5.3: IM4P with empty component tag
im4p_empty_tag = D.sequence([
    D.ia5_string("IM4P"),
    D.ia5_string(""),  # Empty component tag
    D.ia5_string(""),
    D.octet_string(b'\x00' * 16),
])
register_case("im4p_empty_component", build_img4_container(im4p_empty_tag),
    "IM4P with zero-length component and description strings.",
    "0x100004800+", "MEDIUM", "EDGE_CASE")

# TC 5.4: IM4P with KBAG (keybag) present
im4p_kbag = D.sequence([
    D.ia5_string("IM4P"),
    D.ia5_string("illb"),
    D.ia5_string("iBoot"),
    D.octet_string(b'\x00' * 16),
    D.octet_string(b'\x01' + b'\x00' * 15 + b'\xAA' * 48),  # Fake KBAG
])
register_case("im4p_with_kbag", build_img4_container(im4p_kbag),
    "IM4P with 5th element (KBAG/keybag). Tests if parser handles extra "
    "elements and if AES key unwrap code has exploitable paths.",
    "0x100004800+", "HIGH", "KEY_HANDLING")


# ────────────────────────────────────────────────────────────
# Category 6: IMG4 Manifest Boundary Cases
# ────────────────────────────────────────────────────────────
print("\n=== Category 6: IMG4 Manifest Boundary Cases ===\n")

# TC 6.1: Manifest with exactly 16 bytes (the manifest check at 0x10000E540)
# ROM checks: received >= 16 bytes before processing manifest
for size in [0, 1, 15, 16, 17, 32]:
    payload = b'\x00' * size
    register_case(f"manifest_size_{size}_bytes", payload,
        f"Raw {size}-byte payload sent to manifest path. "
        f"ROM's check at 0x10000E540: cmp w9, #0xf (>= 16 required). "
        f"Tests exact boundary condition.",
        "0x10000E540", "HIGH" if size in (15, 16) else "MEDIUM", "BOUNDARY")

# TC 6.2: Full DFU block count boundary (multiple blocks)
# Total payload assembled from multiple 2048-byte DNLOAD blocks
full_block = b'\x41' * 2048
register_case("full_2048_block", full_block,
    "Exactly 2048 bytes (one full DFU block). Tests wLength=0x800 path.",
    "0x10000E494", "MEDIUM", "BOUNDARY")

# TC 6.3: Various sizes near 0x801 (the DFU length check)
for size in [0x7FF, 0x800, 0x801]:
    payload = b'\x42' * min(size, 0x800)
    register_case(f"dfu_size_0x{size:03x}", payload,
        f"Payload size 0x{size:X} — testing around the cmp #0x801 check "
        f"at 0x10000E4AC.",
        "0x10000E4AC", "HIGH" if size == 0x801 else "MEDIUM", "BOUNDARY")


# ────────────────────────────────────────────────────────────
# Category 7: OCTET STRING Size Mismatch
# ────────────────────────────────────────────────────────────
print("\n=== Category 7: OCTET STRING Size Mismatch ===\n")

# TC 7.1: OCTET STRING claims 0x400 bytes but only 0x10 follow
payload = b'\x04' + D.length(0x400) + b'\x00' * 0x10
container = D.sequence([D.ia5_string("IMG4"), D.sequence([
    D.ia5_string("IM4P"), D.ia5_string("illb"), D.ia5_string("x"),
    payload  # Malformed OCTET STRING
])])
register_case("octet_string_oversize_claim", container,
    "OCTET STRING claims 0x400 bytes but only 0x10 provided. "
    "Parser may read 0x400 bytes into heap → OOB read.",
    "0x10000D000+", "CRITICAL", "OOB_READ")

# TC 7.2: OCTET STRING claims 0 bytes but has content
payload = b'\x04\x00' + b'HIDDEN_DATA_NOT_PARSED'
container = D.sequence([D.ia5_string("IMG4"), D.sequence([
    D.ia5_string("IM4P"), D.ia5_string("illb"), D.ia5_string("x"),
    payload
])])
register_case("octet_string_zero_with_trailing", container,
    "OCTET STRING with length=0 followed by hidden data. "
    "Tests if parser advances past the 0-length field correctly.",
    "0x10000D000+", "MEDIUM", "EDGE_CASE")


# ────────────────────────────────────────────────────────────
# Category 8: Negative / Zero Length Edge Cases
# ────────────────────────────────────────────────────────────
print("\n=== Category 8: Negative/Zero Length Edge Cases ===\n")

# TC 8.1: Long-form length with value 0
payload = b'\x30\x81\x00'  # SEQUENCE, 1-byte long form, value = 0
register_case("der_longform_zero_length", payload,
    "SEQUENCE with long-form length encoding for value 0 (0x81 0x00). "
    "Should be short-form. Some parsers mishandle this.",
    "0x10000D000+", "MEDIUM", "EDGE_CASE")

# TC 8.2: Long-form length that would be negative if treated as signed
payload = b'\x30\x82\xFF\xFF' + b'\x00' * 100  # length = 0xFFFF (65535)
register_case("der_len_signedness", payload,
    "SEQUENCE with length 0xFFFF. If cast to signed 16-bit = -1. "
    "If extended to 32-bit signed = 0xFFFFFFFF → massive OOB.",
    "0x10000D000+", "HIGH", "SIGNEDNESS")

# TC 8.3: Length field 0x80 exactly (reserved value in DER)
payload = b'\x30\x80\x00\x00'  # indefinite length marker
register_case("der_len_0x80_reserved", payload,
    "Length byte 0x80 is reserved (indefinite form) in DER. "
    "BER allows it. Parser behavior undefined → potential OOB.",
    "0x10000D000+", "HIGH", "RESERVED_VALUE")


# ────────────────────────────────────────────────────────────
# Category 9: Specific ROM Address Targets
# ────────────────────────────────────────────────────────────
print("\n=== Category 9: Targeted ROM Code Path Triggers ===\n")

# TC 9.1: Target img4_verify with Memz magic (0x4D656D7A)
# From rom_img4_trace: cmp w1, w8 where w8 = 0x4D656D7A ("Memz")
memz_header = struct.pack('>I', 0x4D656D7A)  # "Memz" magic
payload = D.sequence([
    D.ia5_string("IMG4"),
    D.sequence([
        D.ia5_string("IM4P"),
        D.ia5_string("illb"),
        D.ia5_string("iBoot"),
        D.octet_string(memz_header + b'\x00' * 100),
    ]),
])
register_case("img4_memz_magic", payload,
    "IM4P payload starts with 'Memz' (0x4D656D7A) magic. "
    "img4_verify checks for this at 0x10000A7D4. Takes different code path "
    "than standard img4 magic → less tested → more likely buggy.",
    "0x10000A7D4", "HIGH", "ALTERNATE_PATH")

# TC 9.2: Target the dispatch at 0x10000A810 (bl 0x100005480)
# img4_verify_internal is the main parser — craft input that goes deep
deep_im4p = D.sequence([
    D.ia5_string("IM4P"),
    D.ia5_string("illb"),
    D.ia5_string("iBoot"),
    D.octet_string(
        # Craft data that looks like an inner DER structure
        D.sequence([
            D.sequence([D.integer(1), D.octet_string(b'\x00' * 32)]),
            D.sequence([D.integer(2), D.octet_string(b'\xBB' * 64)]),
            D.set_of([D.boolean(True), D.integer(0xCAFE)]),
        ])
    ),
])
container = build_img4_container(deep_im4p, build_im4m())
register_case("img4_deep_inner_der", container[:0x800],
    "IMG4 with IM4P payload containing nested DER structures. "
    "If parser recursively parses payload content, inner structures "
    "trigger additional parser code paths.",
    "0x100005480", "HIGH", "RECURSIVE_PARSE")

# TC 9.3: Target specific comparison values from the ROM
# The ROM has constants: 0x8020 (CPID), 0x5AC (Apple VID)
for magic in [(0x8020, "cpid"), (0x05AC, "vid"), (0x30000, "event30k"),
              (0x40030000, "event40m"), (0x696D3470, "img4"), (0x494D3450, "IM4P_be")]:
    val, name = magic
    payload = struct.pack('<I', val) + b'\x00' * 60
    register_case(f"raw_magic_{name}", payload,
        f"Raw payload starting with 0x{val:X} ({name}). "
        f"Tests if parser finds false magic matches in attacker data.",
        "various", "LOW", "MAGIC_CONFUSION")

# TC 9.4: Trigger the manifest boot path with minimal valid structure
# Need: >=16 bytes, image buffer != NULL
manifest_trigger = D.sequence([
    D.ia5_string("IMG4"),
    D.sequence([
        D.ia5_string("IM4P"),
        D.ia5_string("illb"),
        D.ia5_string("iBoot"),
        D.octet_string(b'\x00' * 256),  # Enough payload
    ]),
    D.context_tag(0, [
        D.sequence([
            D.ia5_string("IM4M"),
            D.integer(0),
            D.set_of([
                D.sequence([
                    D.ia5_string("MANP"),
                    D.set_of([
                        D.sequence([D.ia5_string("CHIP"), D.integer(0x8020)]),
                        D.sequence([D.ia5_string("ECID"), D.integer(0)]),
                    ]),
                ]),
            ]),
        ]),
    ]),
])
register_case("img4_full_manifest_trigger", manifest_trigger[:0x800],
    "Full IMG4 with IM4P + IM4M manifest containing MANP properties. "
    "This should trigger the full manifest verification path including "
    "property iteration at 0x100004CB8 (dispatch through tag table).",
    "0x100004CB8", "HIGH", "MANIFEST_PARSE")


# ────────────────────────────────────────────────────────────
# Category 10: Cross-Structure Pointer Confusion
# ────────────────────────────────────────────────────────────
print("\n=== Category 10: Cross-Structure Confusion ===\n")

# TC 10.1: Overlapping structures (child extends past parent boundary)
inner = D.octet_string(b'\xDD' * 200)
# Manually craft outer with wrong length
outer_content = D.ia5_string("IMG4") + inner
outer = b'\x30' + D.length(len(outer_content) - 100) + outer_content  # 100 bytes short
register_case("img4_child_exceeds_parent", outer,
    "IMG4 outer SEQUENCE claims 100 bytes less than actual content. "
    "If parser trusts parent length → child parsing reads wrong offsets. "
    "If parser trusts child length → reads past parent boundary.",
    "0x10000D000+", "HIGH", "BOUNDARY_CONFUSION")

# TC 10.2: Two IM4P structures (parser may only expect one)
double_im4p = D.sequence([
    D.ia5_string("IMG4"),
    build_im4p(b"illb", b"first", b'\x00' * 16),
    build_im4p(b"ibss", b"second", b'\xAA' * 16),
])
register_case("img4_double_im4p", double_im4p,
    "IMG4 with TWO IM4P elements. Parser may only process first, "
    "second, or attempt both → state confusion if structures share buffers.",
    "0x100004800+", "MEDIUM", "STRUCTURE_CONFUSION")


# ────────────────────────────────────────────────────────────
# Category 11: Special Padding and Alignment
# ────────────────────────────────────────────────────────────
print("\n=== Category 11: Padding and Alignment ===\n")

# TC 11.1: Payload filled with DER tag bytes (could trigger false structure detection)
false_tags = bytes([0x30, 0x82, 0x01, 0x00] * 128)
register_case("payload_false_tags", false_tags[:0x800],
    "Raw payload filled with repeating SEQUENCE headers. If parser scans "
    "forward looking for structure → matches at wrong positions.",
    "0x10000D000+", "MEDIUM", "FALSE_SYNC")

# TC 11.2: All 0xFF payload (every byte is a valid tag class)
register_case("payload_all_ff", b'\xFF' * 0x400,
    "All-0xFF payload. Tag 0xFF = application constructed [31], long form. "
    "Stresses tag parsing limits.",
    "0x10000D000+", "MEDIUM", "EDGE_CASE")

# TC 11.3: Payload that looks like SRAM addresses
sram_spray = b''
for i in range(0, 0x200, 8):
    # Spray SRAM addresses that overlap with interesting globals
    sram_spray += struct.pack('<Q', 0x19C010BE0 + (i % 0x100))
register_case("payload_sram_address_spray", sram_spray,
    "Payload containing SRAM addresses (0x19C010Bxx). If parser stores "
    "pointers from parsed data, these values could corrupt DFU globals.",
    "0x10000D000+", "HIGH", "POINTER_INJECTION")


# ────────────────────────────────────────────────────────────
# Summary
# ────────────────────────────────────────────────────────────
print(f"\n{'=' * 80}")
print(f"  TOTAL TEST CASES GENERATED: {len(test_cases)}")
print(f"  Output directory: {OUTPUT_DIR}")
print(f"{'=' * 80}\n")

sev_counts = {}
for tc in test_cases:
    sev_counts[tc['severity']] = sev_counts.get(tc['severity'], 0) + 1
for sev in ['CRITICAL', 'HIGH', 'MEDIUM', 'LOW']:
    if sev in sev_counts:
        print(f"  {sev:10s}: {sev_counts[sev]}")

# Export manifest
manifest = {
    "generator": "img4_test_generator.py",
    "date": "2026-03-04",
    "target": "T8020 B1 SecureROM (iBoot-3865.0.0.4.7)",
    "total_cases": len(test_cases),
    "severity_summary": sev_counts,
    "cases": test_cases,
    "usage": {
        "dfu_test": "Send each .bin via DFU DNLOAD → zero-length DNLOAD → GETSTATUS",
        "crash_indicator": "Device disappears from USB = crash (potential vuln)",
        "timing_indicator": "Manifest time differs significantly from baseline = different code path",
        "error_indicator": "dfuERROR state = parser rejected (expected for most cases)",
    }
}

manifest_path = OUTPUT_DIR / "test_manifest.json"
manifest_path.write_text(json.dumps(manifest, indent=2))
print(f"\n  Manifest written to: {manifest_path}")

# Also generate a quick-reference loader script
loader_script = '''#!/usr/bin/env python3
"""Quick loader to send test payloads to DFU device."""
import sys, os, json, time
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import usb.core, usb.util, libusb_package, usb.backend.libusb1
    HAS_USB = True
except:
    HAS_USB = False
    print("WARNING: pyusb/libusb not available - dry run only")

APPLE_VID = 0x05AC; DFU_PID = 0x1227

def connect():
    if not HAS_USB:
        return None
    backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
    dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend)
    if dev:
        try: dev.set_configuration()
        except: pass
    return dev

def get_status(dev):
    try:
        data = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        return {"bStatus": data[0], "bState": data[4]}
    except:
        return None

def send_test(dev, payload, block=0):
    try:
        dev.ctrl_transfer(0x21, 1, block, 0, payload, timeout=5000)
        return True
    except:
        return False

def run_test_case(tc_file, dry_run=False):
    with open(tc_file, 'rb') as f:
        payload = f.read()
    print(f"  Payload: {tc_file} ({len(payload)} bytes)")
    print(f"  Preview: {payload[:32].hex()}")
    
    if dry_run or not HAS_USB:
        print("  [DRY RUN] Would send to DFU")
        return
    
    dev = connect()
    if not dev:
        print("  [ERROR] No DFU device found")
        return
    
    # Reset to dfuIDLE
    for _ in range(10):
        st = get_status(dev)
        if st and st['bState'] == 2: break
        if st and st['bState'] == 10:
            dev.ctrl_transfer(0x21, 4, 0, 0, 0)  # CLR_STATUS
        time.sleep(0.05)
    
    # Send payload in chunks
    CHUNK = 2048
    for i in range(0, len(payload), CHUNK):
        chunk = payload[i:i+CHUNK]
        ok = send_test(dev, chunk, block=i//CHUNK)
        if not ok:
            print(f"  [FAIL] DNLOAD block {i//CHUNK} failed")
            return
    
    # Send zero-length to trigger manifest
    send_test(dev, b'', block=(len(payload)//CHUNK)+1)
    
    # Poll status
    t0 = time.time()
    for _ in range(100):
        time.sleep(0.1)
        st = get_status(dev)
        if st:
            print(f"  bState={st['bState']} bStatus={st['bStatus']} t={time.time()-t0:.1f}s")
            if st['bState'] in (2, 10):  # IDLE or ERROR
                break
        else:
            print(f"  [CRASH?] Device not responding at t={time.time()-t0:.1f}s")
            break

if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("test_file", nargs="?", help="Specific .bin to test")
    ap.add_argument("--all", action="store_true", help="Run all test cases")
    ap.add_argument("--dry", action="store_true", help="Dry run (no USB)")
    ap.add_argument("--category", type=int, help="Run specific category (1-11)")
    args = ap.parse_args()
    
    manifest = json.loads(open("test_manifest.json").read())
    
    if args.test_file:
        run_test_case(args.test_file, dry_run=args.dry)
    elif args.all:
        for tc in manifest["cases"]:
            print(f"\\n--- TC {tc['id']}: {tc['name']} ({tc['severity']}) ---")
            run_test_case(tc["file"], dry_run=args.dry)
    elif args.category:
        cat_names = {
            1: "der_len", 2: "asn1_nesting", 3: "tag", 4: "truncat",
            5: "im4p", 6: "manifest", 7: "octet", 8: "negative",
            9: "target", 10: "cross", 11: "pad"
        }
        prefix = cat_names.get(args.category, "")
        for tc in manifest["cases"]:
            if prefix and prefix in tc["name"]:
                print(f"\\n--- TC {tc['id']}: {tc['name']} ({tc['severity']}) ---")
                run_test_case(tc["file"], dry_run=args.dry)
    else:
        # Default: run CRITICAL cases only
        for tc in manifest["cases"]:
            if tc["severity"] == "CRITICAL":
                print(f"\\n--- TC {tc['id']}: {tc['name']} ---")
                run_test_case(tc["file"], dry_run=args.dry)
'''
loader_path = OUTPUT_DIR / "run_tests.py"
loader_path.write_text(loader_script)
print(f"  Test runner written to: {loader_path}")
