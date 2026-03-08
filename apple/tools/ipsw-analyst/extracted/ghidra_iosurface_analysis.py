# Ghidra Script: IOSurface Deep Analysis for Chain B
# =====================================================
# Target: kernelcache_iPhone12,3_26_3.raw (iOS 26.3 A13/T8030)
# Usage: Open kernelcache in Ghidra, run this script
# 
# This script will:
# 1. Label all IOSurface key offsets
# 2. Find and annotate the externalMethod dispatch table
# 3. Identify selector handler functions
# 4. Mark vulnerability-relevant code paths
# 5. Create bookmarks for manual analysis targets
#
# @category iOS.IOSurface
# @author Chain B Analysis

from ghidra.program.model.symbol import SourceType
from ghidra.program.model.listing import CodeUnit
from ghidra.app.decompiler import DecompileOptions, DecompInterface
from ghidra.util.task import ConsoleTaskMonitor

import re

# ============================================================
# KEY OFFSETS FROM BINARY ANALYSIS
# ============================================================
# These are FILE offsets from our analyzer; Ghidra uses virtual addresses.
# The kernelcache TEXT segment loads at 0xfffffff007004000.
# Adjust base_addr if Ghidra auto-detects differently.

BASE_ADDR = 0xfffffff007004000  # __TEXT segment VA

# IOSurface core string offsets (file offsets from analysis)
IOSURFACE_STRINGS = {
    0x787554: "IOSurfaceRootUserClient",
    0x7876d5: "s_create_surface",
    0x787700: "creating_global_IOSurfaces_accessible_to_any_other_process",
    0x787761: "global_insecure_IOSurface_lookups",
    0x7872cb: "IOSurfaceSharedListEntry",
    0x787537: "CSBufferPitch",
    0x786a71: "IOSurfaceClient_class",
    0x787fe3: "IOSurfaceSendRight",
}

# IOSurface property key strings (Cluster 9 - critical for size calculations)
IOSURFACE_PROPERTIES = {
    0x3be4b6: "IOSurfaceWidth",
    # Heights, bytesPerRow, allocSize, pixelFormat etc. are in this cluster
    # Ghidra will find the rest via proximity
}

# Vulnerability-relevant strings
VULN_STRINGS = {
    0x787537: "VULN_integer_overflow__CSBufferPitch",
    0x7872cb: "VULN_race_condition__IOSurfaceSharedListEntry",
    0x787f52: "VULN_uaf__os_refcnt_resurrection",
    0x7874e2: "VULN_race__bogus_surface_handle",
    0x787464: "VULN_shared_memory__IOSurface_ptr",
}

# VTable-related offsets (Cluster 16 - mangled names)
VTABLE_REGION = {
    "start": 0x8422fb,
    "end": 0x844a43,
    "description": "IOSurfaceRootUserClient_vtable_region",
}


def to_va(file_offset):
    """Convert file offset to virtual address (approximate)."""
    # For kernel collections, the mapping is complex.
    # Ghidra should handle this, but as a rough estimate:
    return BASE_ADDR + file_offset


def label_address(addr, name, comment=None):
    """Create a label at an address."""
    try:
        addr_obj = toAddr(addr)
        if addr_obj is None:
            return False
        
        # Create label
        createLabel(addr_obj, name, True, SourceType.USER_DEFINED)
        
        # Add comment if provided
        if comment:
            listing = currentProgram.getListing()
            cu = listing.getCodeUnitAt(addr_obj)
            if cu:
                cu.setComment(CodeUnit.EOL_COMMENT, comment)
        
        return True
    except Exception as e:
        print("  [!] Failed to label 0x%x (%s): %s" % (addr, name, str(e)))
        return False


def create_bookmark(addr, category, description):
    """Create a bookmark for manual analysis."""
    try:
        addr_obj = toAddr(addr)
        if addr_obj:
            bm = currentProgram.getBookmarkManager()
            bm.setBookmark(addr_obj, "Analysis", category, description)
            return True
    except:
        pass
    return False


def find_string_refs(search_str):
    """Find all addresses referencing a string."""
    results = []
    mem = currentProgram.getMemory()
    addr = mem.getMinAddress()
    
    search_bytes = search_str.encode('utf-8') if isinstance(search_str, str) else search_str
    
    while addr is not None:
        addr = mem.findBytes(addr, search_bytes, None, True, ConsoleTaskMonitor())
        if addr is not None:
            results.append(addr)
            addr = addr.add(1)
    
    return results


def find_xrefs_to(addr):
    """Find all cross-references to an address."""
    refs = []
    ref_mgr = currentProgram.getReferenceManager()
    ref_iter = ref_mgr.getReferencesTo(addr)
    while ref_iter.hasNext():
        refs.append(ref_iter.next())
    return refs


# ============================================================
# PHASE 1: Label IOSurface Strings
# ============================================================
def phase1_label_strings():
    print("=" * 60)
    print("PHASE 1: Labeling IOSurface Key Strings")
    print("=" * 60)
    
    labeled = 0
    
    # Find IOSurfaceRootUserClient string
    addrs = find_string_refs("IOSurfaceRootUserClient")
    for addr in addrs:
        label_address(addr.getOffset(), "str_IOSurfaceRootUserClient",
                      "CHAIN_B: Primary UserClient target")
        create_bookmark(addr.getOffset(), "IOSurface", 
                       "IOSurfaceRootUserClient string - find vtable xref")
        labeled += 1
    
    # Find selector strings  
    selector_strs = [
        ("s_create_surface", "SELECTOR_0__create_surface"),
        ("s_release_surface", "SELECTOR_1__release_surface"),
        ("s_lock_surface", "SELECTOR_2__lock_surface"),
        ("s_unlock_surface", "SELECTOR_3__unlock_surface"),
        ("IOUserClientDefaultLockingSingleThreadExternalMethod",
         "externalMethod_dispatch_key"),
    ]
    
    for search, label in selector_strs:
        addrs = find_string_refs(search)
        for addr in addrs:
            label_address(addr.getOffset(), "str_" + label,
                          "IOSurface dispatch selector string")
            labeled += 1
    
    # Find vulnerability-relevant strings
    vuln_strs = [
        ("CSBufferPitch", "VULN_integer_overflow_target"),
        ("IOSurfaceSharedListEntry", "VULN_race_condition_target"),
        ("os_refcnt", "VULN_uaf_refcount"),
        ("resurrection", "VULN_uaf_resurrection_check"),
        ("global (insecure) IOSurface lookups", "VULN_global_namespace_warning"),
        ("creating global IOSurfaces accessible to any other process",
         "VULN_global_surface_creation"),
        ("Unable to prepare shared entry", "VULN_shared_entry_prepare_fail"),
        ("IOBufferMemoryDescriptor::inTaskWithOptions failed",
         "VULN_buffer_alloc_fail"),
        ("IONewZero failed to alloc IOSurfaceSharedListEntry",
         "VULN_shared_list_alloc_fail"),
        ("bogus surface handle", "VULN_bogus_handle_check"),
        ("no longer mapped", "VULN_stale_mapping_check"),
        ("IOSurface: %s exceeds maximum value", "VULN_size_bounds_check"),
        ("buffer allocation failed", "VULN_buffer_alloc_failed"),
    ]
    
    for search, label in vuln_strs:
        addrs = find_string_refs(search)
        for addr in addrs:
            label_address(addr.getOffset(), "str_" + label,
                          "CHAIN_B: Vulnerability-relevant string")
            create_bookmark(addr.getOffset(), "Vulnerability", 
                           label + " - investigate calling function")
            labeled += 1
    
    # IOSurface property keys (potential size overflow inputs)
    prop_strs = [
        "IOSurfaceWidth", "IOSurfaceHeight", "IOSurfaceBytesPerRow",
        "IOSurfaceAllocSize", "IOSurfacePixelFormat",
        "IOSurfacePlaneWidth", "IOSurfacePlaneHeight",
        "IOSurfacePlaneBytesPerRow", "IOSurfacePlaneBase",
        "IOSurfacePlaneOffset", "IOSurfacePlaneSize",
        "IOSurfacePlaneInfo", "IOSurfaceCacheMode",
        "IOSurfaceName",
    ]
    
    for prop in prop_strs:
        addrs = find_string_refs(prop)
        for addr in addrs:
            label_address(addr.getOffset(), "prop_" + prop,
                          "IOSurface property key - size calculation input")
            labeled += 1
    
    print("  Labeled %d IOSurface string locations" % labeled)
    return labeled


# ============================================================
# PHASE 2: Find IOSurfaceRootUserClient VTable
# ============================================================
def phase2_find_vtable():
    print("\n" + "=" * 60)
    print("PHASE 2: Finding IOSurfaceRootUserClient VTable")
    print("=" * 60)
    
    # Search for the C++ mangled vtable symbol
    # Pattern: _ZTVN...IOSurfaceRootUserClient...
    vtable_patterns = [
        "VIOSurfaceRootUserClient",     # Partial mangled name
        "__ZTV25IOSurfaceRootUserClient", # Full vtable symbol
        "IOSurfaceRootUserClient::externalMethod",
        "IOSurfaceRootUserClient::getTargetAndMethodForIndex",
    ]
    
    vtable_hits = []
    for pat in vtable_patterns:
        addrs = find_string_refs(pat)
        for addr in addrs:
            vtable_hits.append((addr, pat))
            print("  Found '%s' at %s" % (pat, str(addr)))
            create_bookmark(addr.getOffset(), "VTable",
                           "IOSurfaceRootUserClient vtable component: " + pat)
    
    # Also search in the symbol table
    sym_table = currentProgram.getSymbolTable()
    sym_iter = sym_table.getAllSymbols(True)
    iosurface_syms = []
    count = 0
    while sym_iter.hasNext() and count < 500000:
        sym = sym_iter.next()
        name = sym.getName()
        if "IOSurface" in name:
            iosurface_syms.append((sym.getAddress(), name))
            if len(iosurface_syms) <= 50:
                print("  Symbol: %s at %s" % (name, str(sym.getAddress())))
        count += 1
    
    if len(iosurface_syms) > 50:
        print("  ... (%d total IOSurface symbols)" % len(iosurface_syms))
    
    # Find vtable specifically
    for addr, name in iosurface_syms:
        if "vtable" in name.lower() or "VTV" in name or "_ZTV" in name:
            print("\n  >>> VTABLE FOUND: %s at %s <<<" % (name, str(addr)))
            label_address(addr.getOffset(), "IOSURFACE_VTABLE",
                          "CHAIN_B: IOSurfaceRootUserClient virtual table")
            create_bookmark(addr.getOffset(), "CRITICAL",
                           "IOSurfaceRootUserClient vtable - enumerate methods")
    
    return vtable_hits, iosurface_syms


# ============================================================
# PHASE 3: Find externalMethod Dispatch Table
# ============================================================
def phase3_find_dispatch():
    print("\n" + "=" * 60)
    print("PHASE 3: Finding externalMethod Dispatch Table")
    print("=" * 60)
    
    # IOSurfaceRootUserClient::externalMethod uses either:
    # A) A static IOExternalMethodDispatch array
    # B) getTargetAndMethodForIndex that returns individual entries
    
    # Search for getTargetAndMethodForIndex
    addrs = find_string_refs("getTargetAndMethodForIndex")
    for addr in addrs:
        print("  getTargetAndMethodForIndex string at %s" % str(addr))
        
        # Find xrefs to this string -> the function itself
        xrefs = find_xrefs_to(addr)
        for xref in xrefs:
            func_addr = xref.getFromAddress()
            print("    Referenced from %s" % str(func_addr))
            
            # Try to get the containing function
            func = getFunctionAt(func_addr)
            if func is None:
                func = getFunctionContaining(func_addr)
            
            if func:
                print("    In function: %s at %s" % (func.getName(), str(func.getEntryPoint())))
                create_bookmark(func.getEntryPoint().getOffset(), "Dispatch",
                               "Contains getTargetAndMethodForIndex - dispatch logic")
    
    # Search for externalMethod
    ext_method_addrs = find_string_refs("externalMethod")
    for addr in ext_method_addrs[:10]:
        xrefs = find_xrefs_to(addr)
        for xref in xrefs:
            func = getFunctionContaining(xref.getFromAddress())
            if func and "IOSurface" in func.getName():
                print("  externalMethod in IOSurface function: %s at %s" % 
                      (func.getName(), str(func.getEntryPoint())))
                label_address(func.getEntryPoint().getOffset(),
                              "IOSurfaceRootUserClient_externalMethod",
                              "CHAIN_B: Main dispatch function - REVERSE THIS")
                create_bookmark(func.getEntryPoint().getOffset(), "CRITICAL",
                               "externalMethod - selector dispatch switch/table")
    
    # Look for IOExternalMethodDispatch structure arrays
    # struct IOExternalMethodDispatch {
    #     IOExternalMethodAction function;  // 8 bytes (pointer)
    #     uint32_t checkScalarInputCount;   // 4 bytes
    #     uint32_t checkStructureInputSize; // 4 bytes
    #     uint32_t checkScalarOutputCount;  // 4 bytes
    #     uint32_t checkStructureOutputSize;// 4 bytes
    # };
    # Total: 24 bytes per entry
    # A dispatch table for ~20 selectors would be 20*24 = 480 bytes
    
    print("\n  Searching for IOExternalMethodDispatch arrays...")
    print("  (Look for sequences of 24-byte entries with valid function pointers)")
    print("  Manual: In Ghidra, go to externalMethod, find the table base from")
    print("  the comparison instruction 'CMP selector, #max_selector'")
    print("  then 'LDR function_ptr, [table_base, selector, LSL#3]'")


# ============================================================
# PHASE 4: IOSurface Code Path Analysis
# ============================================================
def phase4_analyze_code_paths():
    print("\n" + "=" * 60)
    print("PHASE 4: IOSurface Code Path Analysis")
    print("=" * 60)
    
    # Find all IOSurface functions via symbol table
    func_mgr = currentProgram.getFunctionManager()
    all_funcs = func_mgr.getFunctions(True)
    
    iosurface_funcs = []
    for func in all_funcs:
        name = func.getName()
        if "IOSurface" in name:
            iosurface_funcs.append(func)
    
    print("  Found %d IOSurface functions" % len(iosurface_funcs))
    
    # Categorize functions
    categories = {
        "create": [],
        "destroy": [],
        "property": [],
        "shared": [],
        "lock": [],
        "map": [],
        "other": [],
    }
    
    for func in iosurface_funcs:
        name = func.getName().lower()
        if "create" in name or "alloc" in name or "init" in name:
            categories["create"].append(func)
        elif "destroy" in name or "free" in name or "dealloc" in name or "release" in name:
            categories["destroy"].append(func)
        elif "property" in name or "value" in name or "set" in name or "get" in name:
            categories["property"].append(func)
        elif "shared" in name or "list" in name:
            categories["shared"].append(func)
        elif "lock" in name or "unlock" in name:
            categories["lock"].append(func)
        elif "map" in name or "descriptor" in name:
            categories["map"].append(func)
        else:
            categories["other"].append(func)
    
    for cat, funcs in categories.items():
        if funcs:
            print("\n  [%s] %d functions:" % (cat.upper(), len(funcs)))
            for func in funcs[:15]:
                print("    %s at %s" % (func.getName(), str(func.getEntryPoint())))
                
                # Bookmark vuln-relevant categories
                if cat in ("create", "destroy", "shared", "property"):
                    create_bookmark(func.getEntryPoint().getOffset(), "CodePath",
                                   "IOSurface %s path: %s" % (cat, func.getName()))


# ============================================================
# PHASE 5: Decompile Critical Functions
# ============================================================
def phase5_decompile_critical():
    print("\n" + "=" * 60)
    print("PHASE 5: Decompiling Critical IOSurface Functions")
    print("=" * 60)
    
    # Set up decompiler
    decomp = DecompInterface()
    decomp.openProgram(currentProgram)
    
    # Find functions near key strings (these are likely the handlers)
    critical_strings = [
        "s_create_surface",
        "CSBufferPitch",
        "IOSurfaceSharedListEntry",
        "bogus surface handle",
        "IOSurface: %s exceeds maximum value",
    ]
    
    decompiled = []
    for search in critical_strings:
        addrs = find_string_refs(search)
        for addr in addrs:
            xrefs = find_xrefs_to(addr)
            for xref in xrefs:
                func = getFunctionContaining(xref.getFromAddress())
                if func and func not in [d[0] for d in decompiled]:
                    print("\n  Decompiling: %s (refs '%s')" % (func.getName(), search))
                    
                    result = decomp.decompileFunction(func, 30, ConsoleTaskMonitor())
                    if result and result.decompileCompleted():
                        c_code = result.getDecompiledFunction().getC()
                        # Show first 40 lines
                        lines = c_code.split('\n')[:40]
                        for line in lines:
                            print("    " + line)
                        if len(c_code.split('\n')) > 40:
                            print("    ... (%d more lines)" % (len(c_code.split('\n')) - 40))
                        
                        decompiled.append((func, search, c_code))
                        
                        # Mark for manual review
                        create_bookmark(func.getEntryPoint().getOffset(), "Decompile",
                                       "Decompiled - refs '%s' - CHECK FOR VULNS" % search)
    
    decomp.dispose()
    print("\n  Decompiled %d critical functions" % len(decompiled))
    return decompiled


# ============================================================
# PHASE 6: Mark Attack Surface
# ============================================================
def phase6_mark_attack_surface():
    print("\n" + "=" * 60)
    print("PHASE 6: Marking Complete Attack Surface")
    print("=" * 60)
    
    # Create analysis bookmarks summary
    bm = currentProgram.getBookmarkManager()
    
    print("""
  CHAIN B ATTACK PLAN - Manual Analysis Checklist:
  ================================================
  
  1. VTABLE: Find IOSurfaceRootUserClient vtable
     -> List all virtual methods (externalMethod is key)
     
  2. DISPATCH: In externalMethod, find:
     -> Max selector check (CMP Xn, #count)
     -> Dispatch table base (ADR/ADRP + ADD)
     -> Jump to handler (BR/BLR with table offset)
     
  3. SIZE CHECKS: In s_create_surface handler:
     -> Where bytesPerRow * height is computed
     -> Where allocationSize is validated
     -> Is there safe_mul / __builtin_mul_overflow?
     
  4. SHARED MEMORY: In shared list management:
     -> IOSurfaceSharedListEntry alloc path
     -> IOBufferMemoryDescriptor::inTaskWithOptions call
     -> Race window between alloc and prepare
     
  5. REFCOUNT: In use_count handlers:
     -> os_refcnt_increment / os_refcnt_decrement
     -> 'resurrection' check bypass possibility
     
  6. PROPERTIES: In s_set_value / s_get_value:
     -> OSData allocation from user input
     -> Size validation before copy
     -> Type checking on property objects
     
  KEY OFFSETS TO START:
     IOSurfaceRootUserClient string: search for it
     s_create_surface string:        search for it  
     CSBufferPitch string:           search for it
     IOSurfaceSharedListEntry:       search for it
     VTable region (mangled names):  around file offset 0x842000-0x845000
  """)


# ============================================================
# MAIN
# ============================================================
def main():
    print("=" * 60)
    print("IOSurface Deep Analysis - Chain B Ghidra Script")
    print("Target: iOS 26.3 kernelcache (A13/T8030)")
    print("=" * 60)
    
    labeled = phase1_label_strings()
    vtable_hits, syms = phase2_find_vtable()
    phase3_find_dispatch()
    phase4_analyze_code_paths()
    
    # Phase 5 can be slow - uncomment if needed
    # decompiled = phase5_decompile_critical()
    
    phase6_mark_attack_surface()
    
    print("\n" + "=" * 60)
    print("ANALYSIS COMPLETE")
    print("  Labels created: %d+" % labeled)
    print("  IOSurface symbols: %d" % len(syms))
    print("  VTable candidates: %d" % len(vtable_hits))
    print("")
    print("  Next: Open Bookmark Manager to see all analysis targets")
    print("  Filter by category: CRITICAL, Vulnerability, Dispatch, CodePath")
    print("=" * 60)


main()
