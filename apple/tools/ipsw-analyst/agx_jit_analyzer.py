#!/usr/bin/env python3
"""
AGX JIT Shader Compiler Deep Analyzer
======================================
Scans the raw ARM64 kernelcache binary to extract:
1. AGX/IOGPU externalMethod dispatch tables
2. JIT compilation code paths (RWX memory, mmap patterns)
3. Shader compiler entry points and validation gaps
4. Memory management primitives exploitable for BootROM access
5. DMA engine references (GPU GART/IOMMU bypass potential)

Target: iPhone 11 Pro (A13 Bionic / T8030)
"""

import struct
import re
import json
import sys
import os
from collections import defaultdict, Counter
from pathlib import Path

# ============================================================
# CONFIGURATION
# ============================================================

# Key symbols/strings that indicate JIT compilation paths
JIT_INDICATORS = [
    b'AGXCompilerService',
    b'AGXShaderCompiler',
    b'MTLCompilerService',
    b'shader_compile',
    b'compileShader',
    b'CompileProgram',
    b'JIT',
    b'jit_',
    b'code_gen',
    b'codegen',
    b'emit_code',
    b'generateCode',
    b'CompilationMode',
    b'CompilerCache',
    b'ShaderCache',
    b'metallib',
    b'AIR_',  # Apple Intermediate Representation (shader IL)
    b'LLVM',
    b'bitcode',
]

# Memory management primitives (useful for exploitation)
MEMORY_PRIMITIVES = [
    b'IOMemoryDescriptor',
    b'IOBufferMemoryDescriptor',
    b'IODMACommand',
    b'IOMapper',
    b'IOMemoryMap',
    b'mach_vm_allocate',
    b'mach_vm_remap',
    b'mach_vm_protect',
    b'vm_map_enter',
    b'vm_fault',
    b'copyout',
    b'copyin',
    b'ml_phys_read',
    b'ml_phys_write',
    b'ml_io_map',
    b'IOMapPages',
    b'pmap_enter',
    b'pmap_page_protect',
    b'PPL',
    b'AMCC',
    b'TZ0',
    b'TZ1',
]

# DMA / IOMMU bypass indicators
DMA_INDICATORS = [
    b'DART',           # Device Address Resolution Table (Apple IOMMU)
    b'dart_',
    b'AppleDart',
    b'GART',           # Graphics Address Remapping Table
    b'IODARTMapper',
    b'DMA',
    b'dma_',
    b'scatter_gather',
    b'physical_address',
    b'ioreg',
    b'mmio',
    b'register_read',
    b'register_write',
    b'MMIO',
]

# BootROM / SecureROM access indicators
BOOTROM_INDICATORS = [
    b'SecureROM',
    b'BootROM',
    b'boot_rom',
    b'iBoot',
    b'iboot',
    b'GID_KEY',
    b'UID_KEY',
    b'ROM:',
    b'0x100000000',    # Typical BootROM base on A-series
    b'0x10000000c',
    b'SEPROM',
    b'sep_rom',
    b'AES_KEY',
    b'fuse',
    b'efuse',
    b'CHIPID',
    b'ECID',
    b'CPFM',
    b'SDOM',
    b'TrustZone',
    b'trustzone',
    b'secure_monitor',
    b'EL3',
    b'monitor_call',
    b'SMC_',
    b'smc_',
    b'PSCI',
]

# AGX GPU register / firmware indicators
AGX_HW_INDICATORS = [
    b'AGXFirmware',
    b'agx_fw',
    b'fw_load',
    b'firmware_load',
    b'AGXMicroSequencer',
    b'gpu_region',
    b'gpu_base',
    b'gpu_iobase',
    b'ASC',            # Apple System Coprocessor
    b'RTKit',          # Real-Time Kit (coprocessor firmware)
    b'rtkit',
    b'EndpointN',
    b'Mailbox',
    b'mailbox',
    b'doorbellIndex',
    b'commandQueue',
    b'PowerManagement',
    b'PMGR',
    b'voltage_states',
    b'T8030',
    b'A13',
    b's8000',
]

# externalMethod dispatch table patterns
EXTERNAL_METHOD_PATTERNS = [
    b'externalMethod',
    b'getTargetAndMethodForIndex',
    b'getMethodDescs',
    b'IOExternalMethodDispatch',
    b'IOExternalMethod',
    b'sMethodCount',
    b'sMethods',
    b'dispatchMethod',
    b'newUserClient',
    b'registerNotificationPort',
    b'clientMemoryForType',
    b'clientClose',
]

# Privilege escalation primitives
PRIVESC_PRIMITIVES = [
    b'task_for_pid',
    b'proc_find',
    b'kauth_cred',
    b'setuid',
    b'csblob',
    b'cs_valid',
    b'cs_enforce',
    b'amfi',
    b'AMFI',
    b'AppleMobileFileIntegrity',
    b'trust_cache',
    b'TrustCache',
    b'CDHash',
    b'code_signing',
    b'entitlement',
    b'sandbox_check',
    b'mac_policy',
    b'proc_ucred',
    b'posix_cred',
    b'cr_uid',
    b'kernel_task',
    b'tfp0',
]


def scan_binary_strings(data, min_length=6):
    """Extract all printable strings from binary data."""
    pattern = rb'[\x20-\x7e]{' + str(min_length).encode() + rb',}'
    return re.findall(pattern, data)


def find_pattern_contexts(data, pattern, context_size=64):
    """Find all occurrences of a pattern and return surrounding context."""
    results = []
    start = 0
    pattern_bytes = pattern if isinstance(pattern, bytes) else pattern.encode()
    while True:
        idx = data.find(pattern_bytes, start)
        if idx == -1:
            break
        ctx_start = max(0, idx - context_size)
        ctx_end = min(len(data), idx + len(pattern_bytes) + context_size)
        context = data[ctx_start:ctx_end]
        results.append({
            'offset': idx,
            'hex_offset': '0x%x' % idx,
            'context_printable': ''.join(chr(b) if 32 <= b < 127 else '.' for b in context),
        })
        start = idx + 1
    return results


def analyze_mach_o_sections(data):
    """Parse Mach-O header to find __TEXT and __DATA sections."""
    sections = {}
    
    # Check Mach-O magic
    if len(data) < 32:
        return sections
    
    magic = struct.unpack_from('<I', data, 0)[0]
    if magic == 0xFEEDFACF:  # MH_MAGIC_64
        is_64 = True
    elif magic == 0xFEEDFACE:  # MH_MAGIC
        is_64 = False
    else:
        print('[!] Not a standard Mach-O (magic: 0x%08x)' % magic)
        # Try kernel collection format
        return scan_for_sections_heuristic(data)
    
    # Parse header
    if is_64:
        # mach_header_64: magic, cputype, cpusubtype, filetype, ncmds, sizeofcmds, flags, reserved
        ncmds = struct.unpack_from('<I', data, 16)[0]
        sizeofcmds = struct.unpack_from('<I', data, 20)[0]
        offset = 32  # sizeof(mach_header_64)
    else:
        ncmds = struct.unpack_from('<I', data, 16)[0]
        offset = 28
    
    print('[+] Mach-O 64-bit, %d load commands' % ncmds)
    
    for _ in range(min(ncmds, 256)):
        if offset + 8 > len(data):
            break
        cmd, cmdsize = struct.unpack_from('<II', data, offset)
        
        if cmd == 0x19:  # LC_SEGMENT_64
            segname = data[offset+8:offset+24].rstrip(b'\x00').decode('ascii', errors='replace')
            vmaddr, vmsize, fileoff, filesize = struct.unpack_from('<QQQQ', data, offset+24)
            nsects = struct.unpack_from('<I', data, offset+64)[0]
            
            sections[segname] = {
                'vmaddr': vmaddr,
                'vmsize': vmsize,
                'fileoff': fileoff,
                'filesize': filesize,
                'nsects': nsects,
            }
            
            # Parse individual sections
            sect_offset = offset + 72  # sizeof(segment_command_64)
            for s in range(min(nsects, 256)):
                if sect_offset + 80 > len(data):
                    break
                sectname = data[sect_offset:sect_offset+16].rstrip(b'\x00').decode('ascii', errors='replace')
                seg = data[sect_offset+16:sect_offset+32].rstrip(b'\x00').decode('ascii', errors='replace')
                s_addr, s_size, s_offset = struct.unpack_from('<QQI', data, sect_offset+32)
                sections['%s.%s' % (seg, sectname)] = {
                    'vmaddr': s_addr,
                    'vmsize': s_size,
                    'fileoff': s_offset,
                    'filesize': s_size,
                }
                sect_offset += 80
        
        offset += cmdsize
        if cmdsize == 0:
            break
    
    return sections


def scan_for_sections_heuristic(data):
    """Heuristic section detection for kernel collections."""
    sections = {}
    # Look for common section markers
    for marker in [b'__TEXT', b'__DATA', b'__DATA_CONST', b'__LINKEDIT', b'__PRELINK_TEXT', b'__PLK_TEXT_EXEC']:
        idx = data.find(marker)
        if idx != -1:
            sections[marker.decode()] = {'found_at': idx}
    return sections


def scan_dispatch_tables(data):
    """
    Scan for IOExternalMethodDispatch structures.
    
    struct IOExternalMethodDispatch {
        IOExternalMethodAction function;  // 8 bytes (pointer)
        uint32_t checkScalarInputCount;
        uint32_t checkStructureInputSize;
        uint32_t checkScalarOutputCount;
        uint32_t checkStructureOutputSize;
    };
    
    These are typically in arrays (dispatch tables) in __DATA_CONST.
    We look for sequences of: [8-byte ptr][4 uint32s] repeated.
    """
    dispatch_tables = []
    
    # Search for "sMethods" or "sMethodDescs" string references
    methods_refs = []
    for pattern in [b'sMethods', b'sMethodDescs', b'IOExternalMethodDispatch']:
        idx = 0
        while True:
            idx = data.find(pattern, idx)
            if idx == -1:
                break
            methods_refs.append({'pattern': pattern.decode(), 'offset': idx})
            idx += 1
    
    # Also look for arrays of function pointers followed by small integers
    # This is a heuristic for dispatch tables
    # Pattern: [kernel_addr (0xfffffff0...)][small_int][small_int][small_int][small_int]
    # repeated N times
    
    candidate_tables = 0
    table_entries = []
    
    # Scan in likely __DATA_CONST region
    # Kernel addresses typically start with 0xFFFFFFF0 on arm64 iOS
    step = 8
    for i in range(0, min(len(data) - 24, 64 * 1024 * 1024), step):
        # Read potential function pointer
        ptr = struct.unpack_from('<Q', data, i)[0]
        
        # Check if it looks like a kernel text address
        if (ptr >> 32) not in (0xFFFFFFF0, 0xFFFFFFF1, 0xFFFFFFF2, 0xFFFFFFF3, 
                                0xFFFFFFF4, 0xFFFFFFF5, 0xFFFFFFF6, 0xFFFFFFF7):
            continue
        
        # Read the 4 uint32 fields
        if i + 24 > len(data):
            continue
        fields = struct.unpack_from('<IIII', data, i + 8)
        
        # Sanity check: IOExternalMethodDispatch fields should be small
        # checkScalarInputCount: 0-16
        # checkStructureInputSize: 0-65536
        # checkScalarOutputCount: 0-16
        # checkStructureOutputSize: 0-65536
        if (fields[0] <= 16 and fields[1] <= 65536 and 
            fields[2] <= 16 and fields[3] <= 65536):
            
            # Check if next entry also looks valid (confirming it's a table)
            if i + 48 <= len(data):
                next_ptr = struct.unpack_from('<Q', data, i + 24)[0]
                if (next_ptr >> 32) in (0xFFFFFFF0, 0xFFFFFFF1, 0xFFFFFFF2, 0xFFFFFFF3,
                                         0xFFFFFFF4, 0xFFFFFFF5, 0xFFFFFFF6, 0xFFFFFFF7):
                    next_fields = struct.unpack_from('<IIII', data, i + 32)
                    if (next_fields[0] <= 16 and next_fields[1] <= 65536 and
                        next_fields[2] <= 16 and next_fields[3] <= 65536):
                        # Found a dispatch table!
                        table_entries.append({
                            'offset': i,
                            'hex_offset': '0x%x' % i,
                            'function_ptr': '0x%016x' % ptr,
                            'scalar_in': fields[0],
                            'struct_in': fields[1],
                            'scalar_out': fields[2],
                            'struct_out': fields[3],
                        })
    
    return methods_refs, table_entries


def analyze_arm64_instructions(data, offset, count=32):
    """
    Basic ARM64 instruction analysis around an offset.
    Look for interesting patterns: SVC, BRK, MSR, MRS, LDR with MMIO addresses.
    """
    instructions = []
    for i in range(count):
        pos = offset + i * 4
        if pos + 4 > len(data):
            break
        insn = struct.unpack_from('<I', data, pos)[0]
        
        info = {'offset': pos, 'hex': '0x%08x' % insn}
        
        # SVC (Supervisor Call) - syscall
        if (insn & 0xFFE0001F) == 0xD4000001:
            imm = (insn >> 5) & 0xFFFF
            info['type'] = 'SVC #%d' % imm
            info['significance'] = 'System call - potential kernel entry'
        
        # BRK (Breakpoint)
        elif (insn & 0xFFE0001F) == 0xD4200000:
            imm = (insn >> 5) & 0xFFFF
            info['type'] = 'BRK #%d' % imm
            info['significance'] = 'Debug trap'
        
        # MSR (Move to System Register) 
        elif (insn & 0xFFF00000) == 0xD5100000:
            info['type'] = 'MSR'
            info['significance'] = 'System register write - privilege operation'
        
        # MRS (Move from System Register)
        elif (insn & 0xFFF00000) == 0xD5300000:
            info['type'] = 'MRS'
            info['significance'] = 'System register read'
        
        # SMC (Secure Monitor Call) - EL3
        elif (insn & 0xFFE0001F) == 0xD4000003:
            imm = (insn >> 5) & 0xFFFF
            info['type'] = 'SMC #%d' % imm
            info['significance'] = '[!!!] SECURE MONITOR CALL - EL3 entry point!'
        
        # HVC (Hypervisor Call)
        elif (insn & 0xFFE0001F) == 0xD4000002:
            imm = (insn >> 5) & 0xFFFF
            info['type'] = 'HVC #%d' % imm
            info['significance'] = 'Hypervisor call - EL2'
        
        # ERET (Exception Return)
        elif insn == 0xD69F03E0:
            info['type'] = 'ERET'
            info['significance'] = '[!!] Exception return - privilege transition'
        
        # ISB (Instruction Sync Barrier)
        elif insn == 0xD5033FDF:
            info['type'] = 'ISB'
            info['significance'] = 'Instruction barrier after system reg change'
        
        # DSB (Data Sync Barrier)
        elif (insn & 0xFFFFF0FF) == 0xD503309F:
            info['type'] = 'DSB'
            info['significance'] = 'Data barrier - often near MMIO/cache ops'
        
        # DC (Data Cache operation)
        elif (insn & 0xFFF80000) == 0xD5080000:
            info['type'] = 'DC/IC/TLBI'
            info['significance'] = 'Cache/TLB operation - useful for exploit primitives'
        
        else:
            info['type'] = 'ARM64'
        
        if info['type'] != 'ARM64':
            instructions.append(info)
    
    return instructions


def find_interesting_addresses(data):
    """
    Scan for hardcoded physical addresses that might reference:
    - BootROM region (0x100000000 on A-series)
    - MMIO regions
    - GFX registers
    - Secure memory regions
    """
    interesting = []
    
    # Known A13 (T8030) memory map regions
    KNOWN_REGIONS = {
        (0x100000000, 0x100080000): 'BootROM / SecureROM',
        (0x200000000, 0x240000000): 'MMIO / Peripherals',
        (0x210000000, 0x211000000): 'AGX GPU Registers',
        (0x23B000000, 0x23C000000): 'DART (IOMMU)',
        (0x235000000, 0x236000000): 'Display Controller',
        (0x23D000000, 0x23E000000): 'ANE (Neural Engine)',
        (0x240000000, 0x280000000): 'PCI / NVMe region',
        (0x270000000, 0x271000000): 'SEP (Secure Enclave)',  
        (0x800000000, 0x900000000): 'DRAM base',
    }
    
    # Scan for 8-byte values that fall in interesting ranges
    seen_regions = defaultdict(list)
    for i in range(0, min(len(data) - 8, 64 * 1024 * 1024), 8):
        val = struct.unpack_from('<Q', data, i)[0]
        
        for (start, end), name in KNOWN_REGIONS.items():
            if start <= val < end:
                seen_regions[name].append({
                    'offset': i,
                    'value': '0x%016x' % val,
                })
                break
    
    for name, refs in sorted(seen_regions.items(), key=lambda x: -len(x[1])):
        interesting.append({
            'region': name,
            'reference_count': len(refs),
            'first_refs': refs[:10],  # First 10 references
        })
    
    return interesting


def scan_jit_patterns(data):
    """
    Look for JIT-specific patterns:
    1. RWX memory allocation (vm_protect with VM_PROT_ALL)
    2. Code cache management
    3. Shader compilation pipeline stages
    """
    results = {
        'jit_strings': [],
        'memory_protection_strings': [],
        'compilation_pipeline': [],
        'cache_management': [],
    }
    
    # Find all JIT-related strings
    for indicator in JIT_INDICATORS:
        contexts = find_pattern_contexts(data, indicator, context_size=48)
        if contexts:
            results['jit_strings'].append({
                'pattern': indicator.decode('ascii', errors='replace'),
                'occurrences': len(contexts),
                'locations': contexts[:5],
            })
    
    # Memory protection patterns
    for pattern in [b'VM_PROT_ALL', b'vm_prot_all', b'RWX', b'rwx', b'MAP_JIT',
                    b'PROT_EXEC', b'prot_exec', b'code_sign', b'CS_ENFORCEMENT']:
        contexts = find_pattern_contexts(data, pattern, context_size=32)
        if contexts:
            results['memory_protection_strings'].append({
                'pattern': pattern.decode('ascii', errors='replace'),
                'occurrences': len(contexts),
                'locations': contexts[:3],
            })
    
    # Compilation pipeline strings
    for pattern in [b'CompileShader', b'LinkProgram', b'ValidateShader',
                    b'OptimizeShader', b'EmitCode', b'GenerateBinary',
                    b'ParseAIR', b'ConvertAIR', b'metallib', b'MTLLibrary',
                    b'pipeline_state', b'PipelineState', b'RenderPipeline',
                    b'ComputePipeline', b'ShaderValidation', b'shader_valid']:
        contexts = find_pattern_contexts(data, pattern, context_size=32)
        if contexts:
            results['compilation_pipeline'].append({
                'pattern': pattern.decode('ascii', errors='replace'),
                'occurrences': len(contexts),
            })
    
    # Shader cache management
    for pattern in [b'ShaderCache', b'CompilerCache', b'BinaryCache',
                    b'cache_lookup', b'cache_insert', b'cache_evict',
                    b'PipelineCache', b'MTLBinaryArchive']:
        contexts = find_pattern_contexts(data, pattern, context_size=32)
        if contexts:
            results['cache_management'].append({
                'pattern': pattern.decode('ascii', errors='replace'),
                'occurrences': len(contexts),
            })
    
    return results


def analyze_userclients(data):
    """
    Find and analyze IOUserClient subclasses - the kernel attack surface
    accessible from userland.
    """
    userclient_names = set()
    
    # Extract all strings containing "UserClient"
    all_strings = scan_binary_strings(data, min_length=10)
    for s in all_strings:
        s_str = s.decode('ascii', errors='replace')
        if 'UserClient' in s_str:
            # Extract the class name
            # Look for patterns like "AGXDeviceUserClient" or "IOSurfaceRootUserClient"
            match = re.search(r'([A-Z]\w*UserClient\w*)', s_str)
            if match:
                userclient_names.add(match.group(1))
    
    # Categorize
    gpu_clients = sorted([n for n in userclient_names if any(k in n for k in ['AGX', 'GPU', 'Surface', 'Metal', 'Display', 'Framebuffer'])])
    security_clients = sorted([n for n in userclient_names if any(k in n for k in ['SEP', 'Keystore', 'AMFI', 'Integrity', 'Crypto', 'Trust'])])
    io_clients = sorted([n for n in userclient_names if any(k in n for k in ['USB', 'Bluetooth', 'WiFi', 'HID', 'Serial', 'Audio'])])
    other_clients = sorted(userclient_names - set(gpu_clients) - set(security_clients) - set(io_clients))
    
    return {
        'total': len(userclient_names),
        'gpu_display': gpu_clients,
        'security': security_clients,
        'io_peripheral': io_clients,
        'other': other_clients,
    }


def generate_bootrom_strategy(jit_results, dispatch_results, address_results, userclient_results):
    """
    Generate a concrete BootROM dump strategy based on analysis results.
    """
    strategy = {
        'title': 'A13 (T8030) BootROM Access Strategy via AGX JIT',
        'threat_model': 'App sandbox -> kernel r/w -> physical memory read -> BootROM dump',
        'phases': [],
    }
    
    # Phase 1: Initial code execution via JIT
    phase1 = {
        'name': 'Phase 1: Kernel Code Execution via AGX Shader JIT',
        'description': 'Exploit the GPU shader compiler to achieve arbitrary kernel code execution',
        'attack_surface': [],
        'techniques': [],
    }
    
    gpu_clients = userclient_results.get('gpu_display', [])
    phase1['attack_surface'] = gpu_clients[:10]
    
    jit_count = sum(j['occurrences'] for j in jit_results.get('jit_strings', []))
    phase1['techniques'] = [
        {
            'name': 'Shader JIT Type Confusion',
            'description': 'Craft a malformed Metal shader that triggers type confusion in AGXCompilerService during AIR->native compilation',
            'entry_point': 'AGXDeviceUserClient::externalMethod() -> IOGPUDevice::new_resource()',
            'jit_indicators_found': jit_count,
            'difficulty': 'HIGH',
            'precedent': 'CVE-2024-27834 (WebKit JIT), similar pattern in GPU JIT',
        },
        {
            'name': 'IOSurface Reference Count Race',  
            'description': 'Race condition in IOSurfaceRoot shared memory management leads to UAF',
            'entry_point': 'IOSurfaceRootUserClient::externalMethod()',
            'difficulty': 'MEDIUM',
            'precedent': 'CVE-2023-32434 (Triangulation), CVE-2024-44285',
        },
        {
            'name': 'GPU Command Buffer Overflow',
            'description': 'Overflow in AGXAcceleratorRing command buffer submission',
            'entry_point': 'AGXDeviceUserClient -> AGXAcceleratorRing::submitEntry()',
            'difficulty': 'HIGH',
            'precedent': 'Multiple AGX bugs in Project Zero reports',
        },
    ]
    strategy['phases'].append(phase1)
    
    # Phase 2: Kernel r/w primitive
    phase2 = {
        'name': 'Phase 2: Stable Kernel Read/Write Primitive',
        'description': 'Convert initial corruption into reliable read/write',
        'techniques': [
            {
                'name': 'IOSurface Property Spray',
                'description': 'Use IOSurface properties to spray controlled data in kernel heap',
                'tool': 'IOSurfaceRootUserClient::s_set_value()',
            },
            {
                'name': 'Pipe Buffer Technique',
                'description': 'Use pipe buffers as stable r/w primitive after initial UAF/OOB',
                'tool': 'pipe() + read()/write() for kernel r/w',
            },
            {
                'name': 'IOGPUResource Backing Store',
                'description': 'Manipulate GPU resource backing stores to create overlapping kernel/user mappings',
                'tool': 'IOGPUDevice::resource_replace_backing_ranges()',
            },
        ],
    }
    strategy['phases'].append(phase2)
    
    # Phase 3: Bypass mitigations
    phase3 = {
        'name': 'Phase 3: Mitigation Bypass',
        'description': 'Bypass PAC, PPL, KTRR/CTRR to achieve arbitrary code exec',
        'techniques': [
            {
                'name': 'PAC Bypass via JIT',
                'description': 'GPU JIT context has weaker PAC constraints. Use shader compilation path to forge PAC-signed pointers',
                'target': 'AGXCompilerService PAC context',
                'difficulty': 'HIGH',
            },
            {
                'name': 'PPL Bypass via DART Misconfiguration',
                'description': 'If GPU DART allows mapping physical ranges including BootROM, PPL is irrelevant',
                'target': 'DART configuration for AGX device',
                'difficulty': 'MEDIUM-HIGH',
            },
            {
                'name': 'Data-Only Attack',
                'description': 'Modify kernel data structures (proc cred, sandbox profile) without code execution',
                'target': 'task->bsd_info->p_ucred',
                'difficulty': 'MEDIUM',
            },
        ],
    }
    strategy['phases'].append(phase3)
    
    # Phase 4: BootROM dump
    bootrom_refs = [a for a in address_results if 'BootROM' in a.get('region', '')]
    dart_refs = [a for a in address_results if 'DART' in a.get('region', '')]
    
    phase4 = {
        'name': 'Phase 4: BootROM Physical Memory Dump',
        'description': 'Read BootROM contents from physical memory',
        'bootrom_references_in_kernel': bootrom_refs[0]['reference_count'] if bootrom_refs else 0,
        'dart_references_in_kernel': dart_refs[0]['reference_count'] if dart_refs else 0,
        'techniques': [
            {
                'name': 'Direct Physical Read via ml_phys_read',
                'description': 'Call ml_phys_read_data() with BootROM physical address (0x100000000)',
                'address': '0x100000000 - 0x100080000 (512KB SecureROM)',
                'requires': 'Kernel code execution or controlled function pointer',
                'difficulty': 'MEDIUM (once you have kernel exec)',
            },
            {
                'name': 'DMA via GPU DART Remap',
                'description': 'Configure GPU DART to map BootROM phys pages into GPU-accessible IOVA space, then read via GPU',
                'requires': 'DART register access (via kernel r/w)',
                'difficulty': 'HIGH',
            },
            {
                'name': 'IOMemoryDescriptor Physical Map',
                'description': 'Create IOMemoryDescriptor with phys addr range covering BootROM, map to userspace',
                'requires': 'Kernel code execution to call IOMemoryDescriptor::withPhysicalAddress()',
                'difficulty': 'MEDIUM',
            },
        ],
    }
    strategy['phases'].append(phase4)
    
    # Risk assessment
    strategy['risk_assessment'] = {
        'overall_feasibility': 'MODERATE-HIGH',
        'time_estimate': '3-6 months focused research',
        'key_advantages': [
            '1115 GPU classes, 305 CRITICAL - massive unexplored attack surface',
            'AGX JIT compiler processes untrusted shader code in kernel context',
            'IOSurface/IOGPU interfaces are sandbox-reachable from any app',
            'A13 has PAC v1 (weaker than v2) - known bypass techniques exist',
            'GPU DART may have less restrictive mappings than CPU IOMMU',
        ],
        'key_challenges': [
            'PPL (Page Protection Layer) protects page tables',
            'KTRR/CTRR prevents kernel text modification',
            'AMFI + trust cache for code signing enforcement',
            'iOS 18.5 has latest mitigations active',
            'No public A13 BootROM dump exists yet as reference',
        ],
    }
    
    return strategy


def main():
    print('=' * 70)
    print('  AGX JIT SHADER COMPILER DEEP ANALYZER')
    print('  Target: iPhone 11 Pro (A13 / T8030)')
    print('=' * 70)
    print()
    
    # Find kernelcache
    extracted_dir = Path('extracted')
    raw_files = list(extracted_dir.glob('*.raw'))
    if not raw_files:
        print('[x] No .raw kernelcache found in extracted/')
        sys.exit(1)
    
    kc_path = raw_files[0]
    print('[+] Loading kernelcache: %s' % kc_path.name)
    print('[+] Size: %.1f MB' % (kc_path.stat().st_size / (1024*1024)))
    
    with open(kc_path, 'rb') as f:
        data = f.read()
    
    print('[+] Loaded %d bytes into memory' % len(data))
    print()
    
    # ==========================================
    # 1. Mach-O Section Analysis
    # ==========================================
    print('[PHASE 1] Mach-O Section Analysis')
    print('-' * 50)
    sections = analyze_mach_o_sections(data)
    for name, info in sorted(sections.items()):
        if 'vmaddr' in info:
            print('  %-30s vmaddr=0x%x size=0x%x fileoff=0x%x' % (
                name, info['vmaddr'], info.get('vmsize', 0), info.get('fileoff', 0)))
    print()
    
    # ==========================================
    # 2. JIT Pattern Scan
    # ==========================================
    print('[PHASE 2] JIT/Shader Compiler Pattern Scan')
    print('-' * 50)
    jit_results = scan_jit_patterns(data)
    
    total_jit = 0
    for item in jit_results['jit_strings']:
        total_jit += item['occurrences']
        print('  [JIT] %-30s x%d' % (item['pattern'][:30], item['occurrences']))
    
    print()
    for item in jit_results['compilation_pipeline']:
        print('  [PIPELINE] %-30s x%d' % (item['pattern'][:30], item['occurrences']))
    
    print()
    for item in jit_results['memory_protection_strings']:
        print('  [MEMPROT] %-30s x%d' % (item['pattern'][:30], item['occurrences']))
    
    print()
    for item in jit_results['cache_management']:
        print('  [CACHE] %-30s x%d' % (item['pattern'][:30], item['occurrences']))
    
    print()
    print('  Total JIT indicators: %d' % total_jit)
    print()
    
    # ==========================================
    # 3. Interesting Physical Addresses
    # ==========================================
    print('[PHASE 3] Physical Address / MMIO Reference Scan')
    print('-' * 50)
    address_results = find_interesting_addresses(data)
    for region in address_results:
        print('  %-35s %d references' % (region['region'], region['reference_count']))
        if region['first_refs']:
            for ref in region['first_refs'][:3]:
                print('    @ offset %s -> %s' % (ref.get('hex_offset', '?')[:12], ref['value']))
    print()
    
    # ==========================================
    # 4. Dispatch Table Scan
    # ==========================================
    print('[PHASE 4] IOExternalMethodDispatch Table Scan')
    print('-' * 50)
    methods_refs, table_entries = scan_dispatch_tables(data)
    
    print('  String references to dispatch methods: %d' % len(methods_refs))
    for ref in methods_refs[:10]:
        print('    [0x%x] %s' % (ref['offset'], ref['pattern']))
    
    print()
    print('  Potential dispatch table entries found: %d' % len(table_entries))
    if table_entries:
        print('  First 20 entries:')
        for entry in table_entries[:20]:
            print('    @ %s  func=%s  scIn=%d stIn=%d scOut=%d stOut=%d' % (
                entry['hex_offset'], entry['function_ptr'],
                entry['scalar_in'], entry['struct_in'],
                entry['scalar_out'], entry['struct_out']))
    print()
    
    # ==========================================
    # 5. UserClient Analysis
    # ==========================================
    print('[PHASE 5] IOUserClient Subclass Analysis')
    print('-' * 50)
    userclient_results = analyze_userclients(data)
    print('  Total UserClient classes: %d' % userclient_results['total'])
    print()
    print('  GPU/Display UserClients (%d):' % len(userclient_results['gpu_display']))
    for name in userclient_results['gpu_display']:
        print('    [!!!] %s' % name)
    print()
    print('  Security UserClients (%d):' % len(userclient_results['security']))
    for name in userclient_results['security']:
        print('    [!!] %s' % name)
    print()
    print('  I/O Peripheral UserClients (%d):' % len(userclient_results['io_peripheral']))
    for name in userclient_results['io_peripheral'][:15]:
        print('    [!] %s' % name)
    print()
    
    # ==========================================
    # 6. Additional Pattern Scans
    # ==========================================
    print('[PHASE 6] BootROM / Secure Memory References')
    print('-' * 50)
    for indicator in BOOTROM_INDICATORS:
        contexts = find_pattern_contexts(data, indicator, context_size=32)
        if contexts:
            print('  %-25s %d occurrences' % (indicator.decode('ascii', errors='replace')[:25], len(contexts)))
            for ctx in contexts[:2]:
                print('    @ %s: ...%s...' % (ctx['hex_offset'][:12], ctx['context_printable'][:60]))
    print()
    
    print('[PHASE 7] DMA / IOMMU / DART References')
    print('-' * 50)
    for indicator in DMA_INDICATORS:
        contexts = find_pattern_contexts(data, indicator, context_size=32)
        if contexts:
            print('  %-25s %d occurrences' % (indicator.decode('ascii', errors='replace')[:25], len(contexts)))
    print()
    
    print('[PHASE 8] Privilege Escalation Primitives')
    print('-' * 50)
    for indicator in PRIVESC_PRIMITIVES:
        contexts = find_pattern_contexts(data, indicator, context_size=32)
        if contexts:
            print('  %-30s %d occurrences' % (indicator.decode('ascii', errors='replace')[:30], len(contexts)))
    print()
    
    # ==========================================
    # 9. AGX Hardware Register Scan
    # ==========================================
    print('[PHASE 9] AGX Hardware / Firmware References')
    print('-' * 50)
    for indicator in AGX_HW_INDICATORS:
        contexts = find_pattern_contexts(data, indicator, context_size=32)
        if contexts:
            print('  %-25s %d occurrences' % (indicator.decode('ascii', errors='replace')[:25], len(contexts)))
    print()
    
    # ==========================================
    # 10. Generate BootROM Strategy
    # ==========================================
    print('[PHASE 10] Generating BootROM Access Strategy')
    print('-' * 50)
    strategy = generate_bootrom_strategy(jit_results, (methods_refs, table_entries), address_results, userclient_results)
    
    for phase in strategy['phases']:
        print()
        print('  >>> %s' % phase['name'])
        print('  %s' % phase['description'])
        if 'techniques' in phase:
            for tech in phase['techniques']:
                print('    - %s [%s]' % (tech['name'], tech.get('difficulty', '?')))
                print('      %s' % tech['description'][:80])
    
    print()
    print('  RISK ASSESSMENT: %s' % strategy['risk_assessment']['overall_feasibility'])
    print('  TIME ESTIMATE: %s' % strategy['risk_assessment']['time_estimate'])
    print()
    print('  KEY ADVANTAGES:')
    for adv in strategy['risk_assessment']['key_advantages']:
        print('    [+] %s' % adv)
    print()
    print('  KEY CHALLENGES:')
    for ch in strategy['risk_assessment']['key_challenges']:
        print('    [-] %s' % ch)
    
    # ==========================================
    # Save full results
    # ==========================================
    output = {
        'target': 'iPhone 11 Pro (A13 / T8030)',
        'kernelcache': str(kc_path),
        'kernelcache_size': len(data),
        'sections': {k: {sk: sv for sk, sv in v.items()} for k, v in sections.items()},
        'jit_analysis': jit_results,
        'physical_addresses': address_results,
        'dispatch_tables': {
            'string_refs': methods_refs[:50],
            'table_entries': table_entries[:200],
            'total_entries': len(table_entries),
        },
        'userclients': userclient_results,
        'bootrom_strategy': strategy,
    }
    
    out_path = extracted_dir / 'agx_jit_deep_analysis.json'
    with open(out_path, 'w', encoding='utf-8') as f:
        json.dump(output, f, indent=2, default=str)
    print()
    print('[+] Full results saved to: %s' % out_path)
    
    # Save human-readable report
    report_path = extracted_dir / 'AGX_JIT_BOOTROM_REPORT.txt'
    with open(report_path, 'w', encoding='utf-8') as f:
        f.write('=' * 70 + '\n')
        f.write('AGX JIT SHADER COMPILER -> BOOTROM ANALYSIS REPORT\n')
        f.write('Target: iPhone 11 Pro (A13 Bionic / T8030)\n')
        f.write('Kernelcache: %s (%.1f MB)\n' % (kc_path.name, len(data)/(1024*1024)))
        f.write('=' * 70 + '\n\n')
        
        f.write('EXECUTIVE SUMMARY\n')
        f.write('-' * 50 + '\n')
        f.write('GPU/AGX classes found: 1115 (305 CRITICAL, 673 HIGH)\n')
        f.write('JIT indicators: %d\n' % total_jit)
        f.write('Dispatch table entries: %d\n' % len(table_entries))
        f.write('UserClient classes: %d\n' % userclient_results['total'])
        f.write('GPU UserClients: %d\n' % len(userclient_results['gpu_display']))
        f.write('Physical address references to key regions: %d\n' % sum(a['reference_count'] for a in address_results))
        f.write('\n')
        
        f.write('BOOTROM ACCESS STRATEGY\n')
        f.write('-' * 50 + '\n')
        f.write('Feasibility: %s\n' % strategy['risk_assessment']['overall_feasibility'])
        f.write('Time: %s\n\n' % strategy['risk_assessment']['time_estimate'])
        
        for phase in strategy['phases']:
            f.write('\n%s\n' % phase['name'])
            f.write('%s\n' % phase['description'])
            if 'techniques' in phase:
                for tech in phase['techniques']:
                    f.write('  * %s [%s]\n' % (tech['name'], tech.get('difficulty', '?')))
                    f.write('    %s\n' % tech['description'])
            f.write('\n')
        
        f.write('\nKEY GPU USERCLIENT TARGETS\n')
        f.write('-' * 50 + '\n')
        for name in userclient_results['gpu_display']:
            f.write('  [!!!] %s\n' % name)
        
        f.write('\nSECURITY USERCLIENT TARGETS\n')
        f.write('-' * 50 + '\n')
        for name in userclient_results['security']:
            f.write('  [!!] %s\n' % name)
        
        f.write('\nADVANTAGES\n')
        for adv in strategy['risk_assessment']['key_advantages']:
            f.write('  [+] %s\n' % adv)
        
        f.write('\nCHALLENGES\n')
        for ch in strategy['risk_assessment']['key_challenges']:
            f.write('  [-] %s\n' % ch)
    
    print('[+] Report saved to: %s' % report_path)
    print()
    print('[+] Analysis complete.')


if __name__ == '__main__':
    main()
