/*
 * TrustOS iOS Recon — Device Tree Dump
 *
 * Extracts the full IODeviceTree from IOKit on jailbroken iOS.
 * This gives us MMIO base addresses, IRQ assignments, clock domains,
 * and "compatible" strings for every hardware peripheral.
 *
 * Key devices we're looking for (Apple Silicon A10-A14):
 *   - AIC (Apple Interrupt Controller) — replaces ARM GIC
 *   - Samsung-derived UART — Apple's serial ports
 *   - PMGR (Power Manager) — clock gates
 *   - SIO/DMA — DMA engine
 *   - GPIO — pin multiplexing
 *   - ANS/NVMe — Apple NVMe storage
 *
 * This is the Asahi Linux approach: enumerate everything via
 * the device tree before writing bare-metal drivers.
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

/* IOKit headers — available on iOS via Theos SDK or extracted headers */
#ifdef __APPLE__
#include <IOKit/IOKitLib.h>
#include <CoreFoundation/CoreFoundation.h>
#else
/* Stub definitions for cross-compilation on non-Apple */
typedef void *io_registry_entry_t;
typedef void *io_iterator_t;
typedef unsigned int kern_return_t;
#define KERN_SUCCESS 0
#endif

#include "recon.h"

/* ═══════════════════════════════════════════════════════════════════════════
 * IOKit Device Tree Walker
 * ═══════════════════════════════════════════════════════════════════════════ */

#ifdef __APPLE__

/* Helper: extract a CFData property as uint64 */
static uint64_t get_reg_property(io_registry_entry_t entry, const char *key) {
    CFTypeRef prop = IORegistryEntryCreateCFProperty(
        entry, 
        CFStringCreateWithCString(kCFAllocatorDefault, key, kCFStringEncodingUTF8),
        kCFAllocatorDefault, 0);
    
    if (!prop) return 0;
    
    uint64_t value = 0;
    if (CFGetTypeID(prop) == CFDataGetTypeID()) {
        CFDataRef data = (CFDataRef)prop;
        CFIndex len = CFDataGetLength(data);
        if (len >= 8) {
            CFDataGetBytes(data, CFRangeMake(0, 8), (UInt8 *)&value);
        } else if (len >= 4) {
            uint32_t val32 = 0;
            CFDataGetBytes(data, CFRangeMake(0, 4), (UInt8 *)&val32);
            value = val32;
        }
    } else if (CFGetTypeID(prop) == CFNumberGetTypeID()) {
        CFNumberGetValue((CFNumberRef)prop, kCFNumberSInt64Type, &value);
    }
    
    CFRelease(prop);
    return value;
}

/* Helper: extract a CFString or CFData property as C string */
static int get_string_property(io_registry_entry_t entry, const char *key, 
                                char *buf, size_t buflen) {
    CFTypeRef prop = IORegistryEntryCreateCFProperty(
        entry,
        CFStringCreateWithCString(kCFAllocatorDefault, key, kCFStringEncodingUTF8),
        kCFAllocatorDefault, 0);
    
    if (!prop) {
        buf[0] = '\0';
        return -1;
    }
    
    if (CFGetTypeID(prop) == CFStringGetTypeID()) {
        CFStringGetCString((CFStringRef)prop, buf, buflen, kCFStringEncodingUTF8);
    } else if (CFGetTypeID(prop) == CFDataGetTypeID()) {
        CFDataRef data = (CFDataRef)prop;
        CFIndex len = CFDataGetLength(data);
        if ((size_t)len >= buflen) len = buflen - 1;
        CFDataGetBytes(data, CFRangeMake(0, len), (UInt8 *)buf);
        buf[len] = '\0';
    } else {
        buf[0] = '\0';
    }
    
    CFRelease(prop);
    return 0;
}

/* Helper: extract interrupt numbers from "interrupts" property */
static int get_interrupts(io_registry_entry_t entry, uint32_t *irqs, int max_irqs) {
    CFTypeRef prop = IORegistryEntryCreateCFProperty(
        entry,
        CFSTR("interrupts"),
        kCFAllocatorDefault, 0);
    
    if (!prop) return 0;
    
    int count = 0;
    if (CFGetTypeID(prop) == CFDataGetTypeID()) {
        CFDataRef data = (CFDataRef)prop;
        CFIndex len = CFDataGetLength(data);
        const uint32_t *vals = (const uint32_t *)CFDataGetBytePtr(data);
        count = (int)(len / sizeof(uint32_t));
        if (count > max_irqs) count = max_irqs;
        for (int i = 0; i < count; i++) {
            irqs[i] = vals[i];
        }
    }
    
    CFRelease(prop);
    return count;
}

/* Recursively walk the IODeviceTree and extract all nodes */
static void walk_devtree(io_registry_entry_t entry, recon_ctx_t *ctx, int depth) {
    if (ctx->n_devtree >= MAX_DEVTREE_NODES) return;
    
    io_name_t name;
    IORegistryEntryGetName(entry, name);
    
    devtree_node_t *node = &ctx->devtree[ctx->n_devtree];
    memset(node, 0, sizeof(*node));
    
    strncpy(node->name, name, sizeof(node->name) - 1);
    get_string_property(entry, "compatible", node->compatible, sizeof(node->compatible));
    
    /* "reg" property = MMIO base + size (usually <base, size> pairs) */
    node->reg_base = get_reg_property(entry, "reg");
    
    /* Some nodes have separate "AAPL,phandle" or "reg" with size in second word */
    /* We also look for "#address-cells" to know the width */
    CFTypeRef reg_prop = IORegistryEntryCreateCFProperty(
        entry, CFSTR("reg"), kCFAllocatorDefault, 0);
    if (reg_prop && CFGetTypeID(reg_prop) == CFDataGetTypeID()) {
        CFDataRef data = (CFDataRef)reg_prop;
        CFIndex len = CFDataGetLength(data);
        if (len >= 16) {
            /* Likely  <base_hi, base_lo, size_hi, size_lo> or <base64, size64> */
            uint64_t vals[2];
            CFDataGetBytes(data, CFRangeMake(0, 16), (UInt8 *)vals);
            node->reg_base = vals[0];
            node->reg_size = vals[1];
        }
    }
    if (reg_prop) CFRelease(reg_prop);
    
    node->n_interrupts = get_interrupts(entry, node->interrupts, 8);
    get_string_property(entry, "clock-ids", node->clock_domain, sizeof(node->clock_domain));
    
    /* Print it */
    if (ctx->verbose || node->reg_base != 0 || node->compatible[0] != '\0') {
        printf("  %*s%-24s", depth * 2, "", node->name);
        if (node->compatible[0]) 
            printf(" compat=%.40s", node->compatible);
        if (node->reg_base)
            printf(" reg=0x%llx", (unsigned long long)node->reg_base);
        if (node->reg_size)
            printf("+0x%llx", (unsigned long long)node->reg_size);
        if (node->n_interrupts > 0) {
            printf(" irq=[");
            for (int i = 0; i < node->n_interrupts; i++) {
                printf("%u%s", node->interrupts[i], i < node->n_interrupts - 1 ? "," : "");
            }
            printf("]");
        }
        printf("\n");
    }
    
    /* Detect special devices */
    if (strstr(node->compatible, "apple,aic") || strstr(node->name, "aic")) {
        strncpy(ctx->aic_compatible, node->compatible, sizeof(ctx->aic_compatible) - 1);
        ctx->aic_base = node->reg_base;
        ctx->aic_size = node->reg_size;
        printf("  [!!!] Found AIC (Apple Interrupt Controller) @ 0x%llx\n", 
               (unsigned long long)node->reg_base);
    }
    
    if (strstr(node->compatible, "apple,s5l-uart") || 
        strstr(node->compatible, "samsung,uart") ||
        strstr(node->name, "uart") || strstr(node->name, "serial")) {
        if (ctx->n_uarts < 4) {
            ctx->uart_bases[ctx->n_uarts++] = node->reg_base;
            printf("  [!!!] Found UART @ 0x%llx\n", (unsigned long long)node->reg_base);
        }
    }
    
    ctx->n_devtree++;
    
    /* Recurse into children */
    io_iterator_t children;
    kern_return_t kr = IORegistryEntryGetChildIterator(entry, "IODeviceTree", &children);
    if (kr == KERN_SUCCESS) {
        io_registry_entry_t child;
        while ((child = IOIteratorNext(children)) != 0) {
            walk_devtree(child, ctx, depth + 1);
            IOObjectRelease(child);
        }
        IOObjectRelease(children);
    }
}

int recon_dump_devtree(recon_ctx_t *ctx) {
    printf("[*] Enumerating IODeviceTree...\n");
    
    /* Get the root of the device tree */
    io_registry_entry_t root = IORegistryEntryFromPath(
        kIOMainPortDefault, "IODeviceTree:/");
    
    if (!root) {
        printf("[!] Failed to open IODeviceTree root\n");
        printf("[!] Make sure you have root access and IOKit is available\n");
        return -1;
    }
    
    /* Get device model */
    get_string_property(root, "model", ctx->device_model, sizeof(ctx->device_model));
    get_string_property(root, "target-type", ctx->soc_name, sizeof(ctx->soc_name));
    
    /* If model not at root, try IOPlatformExpertDevice */
    if (ctx->device_model[0] == '\0') {
        io_registry_entry_t platform = IOServiceGetMatchingService(
            kIOMainPortDefault,
            IOServiceMatching("IOPlatformExpertDevice"));
        if (platform) {
            get_string_property(platform, "model", ctx->device_model, sizeof(ctx->device_model));
            IOObjectRelease(platform);
        }
    }
    
    printf("[+] Device: %s\n", ctx->device_model[0] ? ctx->device_model : "Unknown");
    printf("[+] SoC: %s\n", ctx->soc_name[0] ? ctx->soc_name : "Unknown");
    printf("[*] Walking device tree...\n\n");
    
    walk_devtree(root, ctx, 0);
    IOObjectRelease(root);
    
    printf("\n[+] Found %d device tree nodes\n", ctx->n_devtree);
    printf("[+] AIC: %s @ 0x%llx (size 0x%llx)\n", 
           ctx->aic_compatible, (unsigned long long)ctx->aic_base, 
           (unsigned long long)ctx->aic_size);
    printf("[+] UARTs found: %d\n", ctx->n_uarts);
    for (int i = 0; i < ctx->n_uarts; i++) {
        printf("    UART%d @ 0x%llx\n", i, (unsigned long long)ctx->uart_bases[i]);
    }
    
    return 0;
}

#else /* !__APPLE__ */

int recon_dump_devtree(recon_ctx_t *ctx) {
    (void)ctx;
    printf("[!] Device tree dump requires iOS/macOS (IOKit framework)\n");
    printf("[!] Cross-compile this tool and run on the jailbroken device\n");
    return -1;
}

#endif /* __APPLE__ */
