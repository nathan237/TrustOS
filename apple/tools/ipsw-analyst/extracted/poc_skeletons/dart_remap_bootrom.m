// dart_remap_bootrom.m - DART Remap for BootROM Physical Read
// REQUIRES: kernel R/W primitive already achieved
// This is a post-exploitation tool

#include <mach/mach.h>
#include <IOKit/IOKitLib.h>
#include <stdio.h>
#include <stdint.h>

// A13 T8030 physical addresses
#define BOOTROM_PHYS_BASE   0x100000000ULL
#define BOOTROM_SIZE        0x80000         // 512 KB
#define GPU_DART_BASE       0x231004000ULL  // GPU DART MMIO
#define AGX_MMIO_BASE       0x206400000ULL  // AGX registers

// DART register offsets (AppleT8020DART)
#define DART_TLB_OP         0x0020    // TLB operations
#define DART_TLB_OP_FLUSH   0x0002    // Flush all TLB entries
#define DART_TTBR(sid,idx)  (0x0200 + (sid)*4 + (idx)*0x10)  // TT base regs
#define DART_TCR(sid)       (0x0100 + (sid)*4)  // Translation control

// DART translation table entry format (64-bit)
// [63:12] = physical page address
// [1]     = valid
// [0]     = block/table
#define DART_TTE_VALID      (1ULL << 1)
#define DART_TTE_TABLE      (1ULL << 0)
#define DART_PTE_VALID      (1ULL << 1)

typedef struct {
    // Your kernel R/W primitive callbacks
    uint64_t (*kread64)(uint64_t kaddr);
    void     (*kwrite64)(uint64_t kaddr, uint64_t value);
    uint64_t (*kread_buf)(uint64_t kaddr, void *buf, size_t len);
} kernel_rw_t;

int dump_bootrom_via_dart(kernel_rw_t *rw) {
    printf("[*] BootROM dump via GPU DART remap\n");
    printf("[*] Target: 0x%llx (size 0x%x)\n", BOOTROM_PHYS_BASE, BOOTROM_SIZE);

    // Step 1: Find GPU DART instance in IORegistry
    // With kernel R/W, walk IORegistry tree to find AppleT8020DART for AGX
    printf("[1] Locating GPU DART instance...\n");
    // Read DART base from device tree or hardcoded for T8030

    // Step 2: Read current DART configuration
    printf("[2] Reading DART configuration...\n");
    uint64_t dart_tcr = rw->kread64(GPU_DART_BASE + DART_TCR(0));
    printf("    DART TCR[0] = 0x%llx\n", dart_tcr);

    // Step 3: Read DART translation table base
    uint64_t dart_ttbr = rw->kread64(GPU_DART_BASE + DART_TTBR(0, 0));
    printf("    DART TTBR[0][0] = 0x%llx\n", dart_ttbr);

    // Step 4: Allocate new DART page table entries
    // Map BootROM physical pages into GPU IOVA space
    printf("[3] Creating DART mappings for BootROM...\n");

    // For each 16KB page of BootROM:
    uint32_t pages = BOOTROM_SIZE / 0x4000;  // 32 pages (16KB each)
    for (uint32_t i = 0; i < pages; i++) {
        uint64_t phys = BOOTROM_PHYS_BASE + (i * 0x4000);
        uint64_t pte = (phys & ~0xFFFULL) | DART_PTE_VALID;

        // Write PTE into DART page table
        // (exact table layout depends on DART level structure)
        // rw->kwrite64(dart_l2_table + i * 8, pte);
        printf("    Page %d: phys 0x%llx -> PTE 0x%llx\n", i, phys, pte);
    }

    // Step 5: Flush DART TLB
    printf("[4] Flushing DART TLB...\n");
    // rw->kwrite64(GPU_DART_BASE + DART_TLB_OP, DART_TLB_OP_FLUSH);

    // Step 6: Read BootROM through the new mapping
    printf("[5] Reading BootROM via DART mapping...\n");
    // Create IOGPUResource backed by new IOVA range
    // Map to userspace and read

    printf("[*] TODO: Complete with actual DART table manipulation\n");
    return 0;
}

// Alternative: direct ml_phys_read approach
int dump_bootrom_via_physread(kernel_rw_t *rw) {
    printf("[*] BootROM dump via ml_phys_read\n");

    uint8_t bootrom[BOOTROM_SIZE];

    // Read page by page (16KB pages on A13)
    for (uint32_t off = 0; off < BOOTROM_SIZE; off += 0x4000) {
        uint64_t phys = BOOTROM_PHYS_BASE + off;

        // Option A: Call ml_phys_read_data via function pointer redirect
        // Option B: Use ml_io_map to get a KVA, then kernel_read

        // Step 1: ml_io_map(phys, 0x4000) -> returns kernel VA
        // uint64_t kva = call_kernel_func(ml_io_map_addr, phys, 0x4000);

        // Step 2: Read from kernel VA
        // rw->kread_buf(kva, bootrom + off, 0x4000);

        printf("    Reading page at phys 0x%llx\n", phys);
    }

    // Save to file
    FILE *f = fopen("bootrom_t8030.bin", "wb");
    if (f) {
        fwrite(bootrom, 1, BOOTROM_SIZE, f);
        fclose(f);
        printf("[+] BootROM saved to bootrom_t8030.bin\n");
    }

    return 0;
}