/*
 * TrustOS iOS Recon — iBoot Memory Scanner
 *
 * Scans physical RAM for iBoot images loaded by SecureROM.
 * On iOS, iBoot remains mapped in memory even after kernel handoff.
 * Finding it lets us reverse-engineer the boot sequence and
 * understand the exact hardware initialization iBoot performs.
 *
 * Signatures we scan for:
 *   - "iBoot-" followed by version (e.g., "iBoot-7459.101.2")
 *   - "iBEC-" / "iBSS-" for DFU-mode bootloaders
 *   - "iBootStage" strings
 *   - Mach-O headers (__TEXT,__text) from iBoot executable
 *
 * This info is critical for:
 *   1. Knowing exactly which peripherals iBoot initializes
 *   2. Understanding the SoC power-on sequence
 *   3. Finding MMIO register values that iBoot sets
 *   4. Eventual checkm8 payload development (for iPhone 7)
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <mach/mach.h>

#include "recon.h"

/* iBoot magic signatures */
static const char *IBOOT_SIGS[] = {
    "iBoot-",
    "iBEC-",
    "iBSS-",
    "iBootStage",
    "Apple Mobile Device (Recovery Mode)",
    "SecureROM for ",
    "CPFM:",           /* Chip Fuse Mode */
    "CEPO:",           /* Chip Epoch */
    "BORD:",           /* Board ID */
    "CHIP:",           /* Chip ID */
    NULL
};

/* Mach-O magic numbers */
#define MH_MAGIC_64     0xFEEDFACFU
#define MH_CIGAM_64     0xCFFAEDFEU

static uint32_t simple_crc32(const uint8_t *data, size_t len) {
    uint32_t crc = 0xFFFFFFFF;
    for (size_t i = 0; i < len; i++) {
        crc ^= data[i];
        for (int j = 0; j < 8; j++) {
            crc = (crc >> 1) ^ (0xEDB88320 & -(crc & 1));
        }
    }
    return ~crc;
}

#ifdef __APPLE__

/* Scan a memory region for iBoot signatures */
static int scan_region(recon_ctx_t *ctx, uint64_t base, uint64_t size, 
                        const uint8_t *data) {
    int found = 0;
    
    for (size_t offset = 0; offset < size - 64; offset += 4) {
        /* Check for iBoot string signatures */
        for (int s = 0; IBOOT_SIGS[s] != NULL; s++) {
            size_t siglen = strlen(IBOOT_SIGS[s]);
            if (offset + siglen >= size) continue;
            
            if (memcmp(data + offset, IBOOT_SIGS[s], siglen) == 0) {
                if (ctx->n_iboot >= MAX_IBOOT_SIGS) goto done;
                
                iboot_sig_t *sig = &ctx->iboot[ctx->n_iboot];
                sig->address = base + offset;
                
                /* Extract version string (up to null or newline) */
                size_t vlen = 0;
                while (offset + siglen + vlen < size && 
                       data[offset + vlen] >= 0x20 && 
                       data[offset + vlen] < 0x7F &&
                       vlen < sizeof(sig->version) - 1) {
                    sig->version[vlen] = data[offset + vlen];
                    vlen++;
                }
                sig->version[vlen] = '\0';
                
                /* Compute CRC of surrounding 4KB for identification */
                uint64_t crc_start = (offset > 2048) ? offset - 2048 : 0;
                uint64_t crc_len = 4096;
                if (crc_start + crc_len > size) crc_len = size - crc_start;
                sig->crc = simple_crc32(data + crc_start, (size_t)crc_len);
                
                printf("  [FOUND] %s @ phys 0x%llx (CRC32: 0x%08x)\n",
                       sig->version,
                       (unsigned long long)sig->address,
                       sig->crc);
                
                ctx->n_iboot++;
                found++;
            }
        }
        
        /* Check for Mach-O header (iBoot is a Mach-O executable) */
        if (offset + 4 <= size) {
            uint32_t magic = *(uint32_t *)(data + offset);
            if (magic == MH_MAGIC_64 || magic == MH_CIGAM_64) {
                printf("  [MACH-O] Found 64-bit Mach-O header @ phys 0x%llx\n",
                       (unsigned long long)(base + offset));
                
                /* Try to estimate image size from load commands */
                if (ctx->n_iboot < MAX_IBOOT_SIGS) {
                    iboot_sig_t *sig = &ctx->iboot[ctx->n_iboot++];
                    sig->address = base + offset;
                    snprintf(sig->version, sizeof(sig->version), 
                             "Mach-O@0x%llx", (unsigned long long)(base + offset));
                    sig->crc = magic;
                    sig->region_size = 0;  /* TODO: parse load commands */
                    found++;
                }
            }
        }
    }
    
done:
    return found;
}

int recon_scan_iboot(recon_ctx_t *ctx) {
    printf("[*] Scanning physical memory for iBoot signatures...\n");
    printf("[*] This may take a few seconds on devices with >2GB RAM\n\n");
    
    /* Get kernel task port (jailbreak provides this) */
    mach_port_t kernel_task = MACH_PORT_NULL;
    kern_return_t kr = task_for_pid(mach_task_self(), 0, &kernel_task);
    
    if (kr != KERN_SUCCESS || kernel_task == MACH_PORT_NULL) {
        printf("[!] task_for_pid(0) failed (kr=%d)\n", kr);
        printf("[!] Your jailbreak must provide tfp0. Try:\n");
        printf("[!]   - Dopamine: should work automatically\n");
        printf("[!]   - unc0ver: enable 'Get task_for_pid(0)' in settings\n");
        printf("[!]   - Taurine: should work automatically\n");
        
        /* Fallback: try scanning via /dev/kmem */
        printf("[*] Trying /dev/kmem fallback...\n");
        int fd = open("/dev/kmem", O_RDONLY);
        if (fd < 0) {
            printf("[!] /dev/kmem not available either. Cannot scan.\n");
            return -1;
        }
        close(fd);
        printf("[!] /dev/kmem fallback not implemented yet\n");
        return -1;
    }
    
    printf("[+] Got kernel task port: 0x%x\n", kernel_task);
    
    /* 
     * iBoot typically lives in low physical memory.
     * On A12, it's often around 0x800000000 + some offset.
     * We scan the first 256MB of DRAM at 0x800000000.
     *
     * Also scan SRAM region where SecureROM data may linger.
     */
    struct scan_range {
        const char *name;
        uint64_t base;
        uint64_t size;
    } ranges[] = {
        { "SRAM",        0x19C000000ULL,  0x00080000ULL  },  /* 512 KB SRAM */
        { "DRAM-low",    0x800000000ULL,  0x10000000ULL  },  /* First 256 MB */
        { "DRAM-iboot",  0x810000000ULL,  0x10000000ULL  },  /* Next 256 MB */
        { "DRAM-kernel", 0xFFFFC000ULL,   0x00100000ULL  },  /* KernelCache area */
    };
    
    int total_found = 0;
    
    for (int r = 0; r < 4; r++) {
        printf("\n[*] Scanning %s (0x%llx, %llu MB)...\n",
               ranges[r].name,
               (unsigned long long)ranges[r].base,
               (unsigned long long)ranges[r].size / (1024 * 1024));
        
        /* Read in 1MB chunks via vm_read */
        uint64_t chunk_size = 1024 * 1024;  /* 1 MB */
        
        for (uint64_t off = 0; off < ranges[r].size; off += chunk_size) {
            vm_offset_t data = 0;
            mach_msg_type_number_t data_size = 0;
            
            uint64_t addr = ranges[r].base + off;
            uint64_t remaining = ranges[r].size - off;
            if (remaining < chunk_size) chunk_size = remaining;
            
            kr = vm_read(kernel_task, (vm_address_t)addr, (vm_size_t)chunk_size,
                         &data, &data_size);
            
            if (kr != KERN_SUCCESS) {
                if (ctx->verbose) {
                    printf("  [skip] 0x%llx not readable\n", (unsigned long long)addr);
                }
                continue;
            }
            
            int found = scan_region(ctx, addr, data_size, (const uint8_t *)data);
            total_found += found;
            
            vm_deallocate(mach_task_self(), data, data_size);
        }
    }
    
    printf("\n[+] iBoot scan complete: %d signatures found\n", total_found);
    
    if (total_found > 0) {
        printf("[+] Summary:\n");
        for (int i = 0; i < ctx->n_iboot; i++) {
            printf("    %d. %s @ 0x%llx (CRC: 0x%08x)\n",
                   i + 1, ctx->iboot[i].version,
                   (unsigned long long)ctx->iboot[i].address,
                   ctx->iboot[i].crc);
        }
        printf("\n[*] TIP: Dump a found iBoot image with:\n");
        printf("    dd if=/dev/kmem bs=1 skip=<offset> count=2097152 of=iboot.bin\n");
        printf("    Then analyze with: tools/analyze_iboot.py iboot.bin\n");
    }
    
    return 0;
}

#else

int recon_scan_iboot(recon_ctx_t *ctx) {
    (void)ctx;
    printf("[!] iBoot scan requires jailbroken iOS (tfp0)\n");
    return -1;
}

#endif
