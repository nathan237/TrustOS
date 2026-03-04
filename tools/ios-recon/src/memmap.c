/*
 * TrustOS iOS Recon — Physical Memory Map
 *
 * Extracts the physical memory layout from a jailbroken iPhone.
 * Uses IOKit's IOMemoryDescriptor and task_for_pid to map physical pages.
 *
 * On Apple Silicon (A10+), the physical memory map looks like:
 *   0x0_0000_0000 - 0x0_0FFF_FFFF  MMIO (peripherals)
 *   0x0_8000_0000 - 0x0_FFFF_FFFF  DRAM (main memory, ~2-4 GB)
 *   0x2_0000_0000 - 0x2_3FFF_FFFF  DRAM (continued on larger models)
 *
 * The exact layout varies by SoC generation (T8010/A10 vs T8020/A12).
 * This module discovers it empirically.
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <mach/mach.h>
#include <mach/vm_map.h>

#include "recon.h"

/* ═══════════════════════════════════════════════════════════════════════════
 * Known Apple SoC memory regions (from public RE work)
 * These are starting points — recon will verify/extend them
 * ═══════════════════════════════════════════════════════════════════════════ */

typedef struct {
    const char *name;
    uint64_t base;
    uint64_t size;
    const char *type;
} known_region_t;

/* T8020 (A12 Bionic) — iPhone XR/XS known regions */
static const known_region_t a12_regions[] = {
    { "SRAM",          0x19C000000ULL, 0x00080000ULL, "SRAM"     },
    { "MMIO-North",    0x200000000ULL, 0x10000000ULL, "MMIO"     },
    { "AIC",           0x23B100000ULL, 0x00008000ULL, "MMIO"     },
    { "PMGR",          0x23B700000ULL, 0x00040000ULL, "MMIO"     },
    { "GPIO",          0x23C100000ULL, 0x00004000ULL, "MMIO"     },
    { "UART0",         0x235010000ULL, 0x00004000ULL, "MMIO"     },
    { "UART2",         0x235018000ULL, 0x00004000ULL, "MMIO"     },
    { "SPI0",          0x235100000ULL, 0x00004000ULL, "MMIO"     },
    { "I2C0",          0x235014000ULL, 0x00004000ULL, "MMIO"     },
    { "DART (IOMMU)",  0x23B300000ULL, 0x00004000ULL, "MMIO"     },
    { "NVMe-ANS",      0x27C400000ULL, 0x00040000ULL, "MMIO"     },
    { "USB-DWC3",      0x39000000ULL,  0x00100000ULL, "MMIO"     },
    { "PCIe",          0x680000000ULL, 0x20000000ULL, "MMIO"     },
    { "DRAM",          0x800000000ULL, 0x100000000ULL,"RAM"      },  /* 4 GB */
    { NULL, 0, 0, NULL }
};

/* T8010 (A10 Fusion) — iPhone 7 known regions (checkm8 target!) */
static const known_region_t a10_regions[] = {
    { "SRAM",          0x180000000ULL, 0x00080000ULL, "SRAM"     },
    { "AIC",           0x20E100000ULL, 0x00008000ULL, "MMIO"     },
    { "PMGR",          0x20E080000ULL, 0x00040000ULL, "MMIO"     },
    { "GPIO",          0x20F100000ULL, 0x00004000ULL, "MMIO"     },
    { "UART0",         0x235200000ULL, 0x00004000ULL, "MMIO"     },
    { "DRAM",          0x800000000ULL, 0x0C0000000ULL,"RAM"      },  /* 3 GB */
    { NULL, 0, 0, NULL }
};

static const known_region_t *select_known_regions(const char *soc_name) {
    if (strstr(soc_name, "T8020") || strstr(soc_name, "A12") || 
        strstr(soc_name, "t8020")) {
        return a12_regions;
    }
    if (strstr(soc_name, "T8010") || strstr(soc_name, "A10") ||
        strstr(soc_name, "t8010")) {
        return a10_regions;
    }
    /* Default to A12 */
    return a12_regions;
}

/* ═══════════════════════════════════════════════════════════════════════════
 * Memory probing via /dev/kmem or task_for_pid(0)
 * ═══════════════════════════════════════════════════════════════════════════ */

#ifdef __APPLE__

static int probe_physical_page(uint64_t phys_addr, int verbose) {
    /*
     * On jailbroken iOS, we can use IOKit's IOMemoryDescriptor
     * to read physical memory. This requires:
     *   - Root access
     *   - tfp0 (task_for_pid(0)) — provided by jailbreak
     *   - Or /dev/kmem if available
     */
    
    /* Method 1: Try /dev/kmem (some jailbreaks patch this in) */
    int fd = open("/dev/kmem", O_RDONLY);
    if (fd >= 0) {
        uint64_t val = 0;
        if (pread(fd, &val, sizeof(val), (off_t)phys_addr) == sizeof(val)) {
            close(fd);
            return 1;  /* Readable */
        }
        close(fd);
    }
    
    /* Method 2: mach_vm_read via tfp0 */
    mach_port_t kernel_task = MACH_PORT_NULL;
    kern_return_t kr = task_for_pid(mach_task_self(), 0, &kernel_task);
    if (kr == KERN_SUCCESS && kernel_task != MACH_PORT_NULL) {
        vm_offset_t data = 0;
        mach_msg_type_number_t size = 0;
        kr = vm_read(kernel_task, (vm_address_t)phys_addr, 4096, &data, &size);
        if (kr == KERN_SUCCESS) {
            vm_deallocate(mach_task_self(), data, size);
            return 1;
        }
    }
    
    if (verbose) {
        printf("    [probe] 0x%llx — not accessible\n", (unsigned long long)phys_addr);
    }
    
    return 0;
}

int recon_dump_memmap(recon_ctx_t *ctx) {
    printf("[*] Mapping physical memory layout...\n");
    
    /* Start with known regions for this SoC */
    const known_region_t *known = select_known_regions(ctx->soc_name);
    
    printf("[*] Loading known regions for %s...\n", 
           ctx->soc_name[0] ? ctx->soc_name : "default (A12)");
    
    for (int i = 0; known[i].name != NULL && ctx->n_memmap < MAX_MEMMAP_ENTRIES; i++) {
        memmap_entry_t *entry = &ctx->memmap[ctx->n_memmap];
        entry->base = known[i].base;
        entry->size = known[i].size;
        strncpy(entry->type, known[i].type, sizeof(entry->type) - 1);
        strncpy(entry->name, known[i].name, sizeof(entry->name) - 1);
        ctx->n_memmap++;
        
        printf("  [known] %-16s 0x%09llx - 0x%09llx (%s)\n",
               known[i].name,
               (unsigned long long)known[i].base,
               (unsigned long long)(known[i].base + known[i].size),
               known[i].type);
    }
    
    /* Now probe to discover additional regions */
    printf("\n[*] Probing physical address space...\n");
    
    /* Probe common MMIO ranges at 16MB granularity */
    uint64_t probe_ranges[][2] = {
        { 0x200000000ULL, 0x300000000ULL },  /* MMIO North */
        { 0x380000000ULL, 0x400000000ULL },  /* MMIO South */
        { 0x600000000ULL, 0x700000000ULL },  /* PCIe */
    };
    
    for (int r = 0; r < 3; r++) {
        for (uint64_t addr = probe_ranges[r][0]; addr < probe_ranges[r][1]; 
             addr += 0x1000000) { /* 16 MB steps */
            if (probe_physical_page(addr, ctx->verbose)) {
                /* Found an accessible region — refine boundaries */
                if (ctx->n_mmio < MAX_MMIO_REGIONS) {
                    mmio_region_t *mmio = &ctx->mmio[ctx->n_mmio++];
                    mmio->base = addr;
                    mmio->size = 0x1000000;
                    snprintf(mmio->device, sizeof(mmio->device), "unknown@0x%llx", 
                             (unsigned long long)addr);
                    printf("  [discovered] MMIO @ 0x%llx (16MB block accessible)\n",
                           (unsigned long long)addr);
                }
            }
        }
    }
    
    printf("\n[+] Memory map: %d known regions + %d discovered MMIO blocks\n",
           ctx->n_memmap, ctx->n_mmio);
    
    return 0;
}

#else

int recon_dump_memmap(recon_ctx_t *ctx) {
    (void)ctx;
    printf("[!] Memory map dump requires jailbroken iOS (tfp0 / /dev/kmem)\n");
    return -1;
}

#endif
