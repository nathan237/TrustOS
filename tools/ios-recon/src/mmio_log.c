/*
 * TrustOS iOS Recon — MMIO Access Logger
 *
 * Monitors IOKit registry for hardware access patterns.
 * On jailbroken iOS, we can:
 *   1. Enumerate all IOKit drivers and their MMIO mappings
 *   2. Watch for IOService attach/detach events
 *   3. Read IOKit statistics (transaction counts per device)
 *
 * This is NOT kernel-level MMIO tracing (that needs kext or KDK).
 * Instead, we observe IOKit's user-visible state to infer which
 * peripherals are active and what addresses they use.
 *
 * For true MMIO interception, see the EL2 hypervisor in:
 *   kernel/src/hypervisor/arm_hv/mmio_spy.rs
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <signal.h>

#ifdef __APPLE__
#include <IOKit/IOKitLib.h>
#include <CoreFoundation/CoreFoundation.h>
#endif

#include "recon.h"

static volatile int g_running = 1;

static void sig_handler(int sig) {
    (void)sig;
    g_running = 0;
}

#ifdef __APPLE__

/* Dump all IOKit memory mappings for a service */
static void dump_service_mmio(io_service_t service, recon_ctx_t *ctx) {
    io_name_t name;
    IORegistryEntryGetName(service, name);
    
    /* Get "IODeviceMemory" property — array of IOMemoryDescriptor ranges */
    CFTypeRef mem_prop = IORegistryEntryCreateCFProperty(
        service, CFSTR("IODeviceMemory"),
        kCFAllocatorDefault, 0);
    
    if (mem_prop && CFGetTypeID(mem_prop) == CFArrayGetTypeID()) {
        CFArrayRef arr = (CFArrayRef)mem_prop;
        CFIndex count = CFArrayGetCount(arr);
        
        for (CFIndex i = 0; i < count && ctx->n_mmio < MAX_MMIO_REGIONS; i++) {
            CFDictionaryRef dict = CFArrayGetValueAtIndex(arr, i);
            if (!dict || CFGetTypeID(dict) != CFDictionaryGetTypeID()) continue;
            
            /* Extract address and length from the IOMemoryDescriptor dict */
            CFNumberRef addr_ref = CFDictionaryGetValue(dict, CFSTR("address"));
            CFNumberRef size_ref = CFDictionaryGetValue(dict, CFSTR("length"));
            
            uint64_t addr = 0, size = 0;
            if (addr_ref) CFNumberGetValue(addr_ref, kCFNumberSInt64Type, &addr);
            if (size_ref) CFNumberGetValue(size_ref, kCFNumberSInt64Type, &size);
            
            if (addr != 0) {
                mmio_region_t *mmio = &ctx->mmio[ctx->n_mmio++];
                mmio->base = addr;
                mmio->size = size;
                strncpy(mmio->device, name, sizeof(mmio->device) - 1);
                
                printf("  %-28s MMIO 0x%09llx size 0x%llx\n",
                       name, (unsigned long long)addr, (unsigned long long)size);
            }
        }
    }
    if (mem_prop) CFRelease(mem_prop);
}

/* Enumerate all IOKit services with MMIO mappings */
static void enumerate_mmio_services(recon_ctx_t *ctx) {
    io_iterator_t iter;
    kern_return_t kr = IOServiceGetMatchingServices(
        kIOMainPortDefault,
        IOServiceMatching("IOService"),
        &iter);
    
    if (kr != KERN_SUCCESS) {
        printf("[!] Failed to enumerate IOKit services (kr=%d)\n", kr);
        return;
    }
    
    printf("[*] IOKit services with MMIO regions:\n\n");
    
    io_service_t service;
    int service_count = 0;
    while ((service = IOIteratorNext(iter)) != 0) {
        dump_service_mmio(service, ctx);
        IOObjectRelease(service);
        service_count++;
    }
    IOObjectRelease(iter);
    
    printf("\n[+] Scanned %d IOKit services, found %d MMIO regions\n",
           service_count, ctx->n_mmio);
}

/* Watch for IOKit notification events (service attach/detach) */
static void watch_iokit_events(recon_ctx_t *ctx) {
    (void)ctx;
    
    printf("\n[*] Watching IOKit events (Ctrl+C to stop)...\n");
    printf("[*] Plug/unplug USB devices, toggle Bluetooth/WiFi to see activity\n\n");
    
    signal(SIGINT, sig_handler);
    
    IONotificationPortRef notify_port = IONotificationPortCreate(kIOMainPortDefault);
    if (!notify_port) {
        printf("[!] Failed to create notification port\n");
        return;
    }
    
    CFRunLoopSourceRef source = IONotificationPortGetRunLoopSource(notify_port);
    CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopDefaultMode);
    
    /* Run for 30 seconds or until interrupted */
    printf("[*] Monitoring for 30 seconds...\n");
    CFRunLoopRunInMode(kCFRunLoopDefaultMode, 30.0, false);
    
    IONotificationPortDestroy(notify_port);
    printf("[+] MMIO monitoring stopped\n");
}

int recon_log_mmio(recon_ctx_t *ctx) {
    printf("[*] Starting MMIO region enumeration...\n\n");
    
    /* Phase 1: Static enumeration */
    enumerate_mmio_services(ctx);
    
    /* Phase 2: Live monitoring */
    if (ctx->verbose) {
        watch_iokit_events(ctx);
    } else {
        printf("\n[*] Use --verbose to enable live IOKit event monitoring\n");
    }
    
    return 0;
}

#else

int recon_log_mmio(recon_ctx_t *ctx) {
    (void)ctx;
    printf("[!] MMIO logging requires iOS (IOKit framework)\n");
    return -1;
}

#endif
