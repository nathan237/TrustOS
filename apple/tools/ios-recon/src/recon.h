/*
 * TrustOS iOS Recon — Header
 * 
 * Shared types and function prototypes for all recon modules.
 */

#ifndef TRUSTOS_RECON_H
#define TRUSTOS_RECON_H

#include <stdint.h>
#include <stddef.h>

#ifndef VERSION
#define VERSION "0.1.0"
#endif

/* Maximum entries for various tables */
#define MAX_DEVTREE_NODES   4096
#define MAX_MEMMAP_ENTRIES  256
#define MAX_MMIO_REGIONS    512
#define MAX_IBOOT_SIGS      32

/* ═══════════════════════════════════════════════════════════════════════════
 * Data structures
 * ═══════════════════════════════════════════════════════════════════════════ */

/* Device tree node extracted from IOKit */
typedef struct {
    char name[128];
    char compatible[256];
    uint64_t reg_base;       /* MMIO base physical address */
    uint64_t reg_size;       /* MMIO region size */
    uint32_t interrupts[8];  /* IRQ numbers (up to 8) */
    int      n_interrupts;
    char     clock_domain[64];
} devtree_node_t;

/* Physical memory region */
typedef struct {
    uint64_t base;
    uint64_t size;
    char     type[32];      /* "RAM", "MMIO", "IOMEM", "Reserved" */
    char     name[64];      /* Description if available */
} memmap_entry_t;

/* MMIO region with identified device */
typedef struct {
    uint64_t base;
    uint64_t size;
    char     device[64];    /* "AIC", "UART", "SPI", "I2C", "GPIO", etc. */
    char     compatible[128];
} mmio_region_t;

/* iBoot signature found in memory */
typedef struct {
    uint64_t address;        /* Physical address where found */
    char     version[64];    /* iBoot version string */
    uint32_t crc;            /* CRC of the iBoot region */
    uint64_t region_size;    /* Estimated iBoot size */
} iboot_sig_t;

/* Master recon context */
typedef struct {
    int verbose;
    const char *output_file;
    
    /* Device tree */
    devtree_node_t devtree[MAX_DEVTREE_NODES];
    int n_devtree;
    
    /* Memory map */
    memmap_entry_t memmap[MAX_MEMMAP_ENTRIES];
    int n_memmap;
    
    /* MMIO regions */
    mmio_region_t mmio[MAX_MMIO_REGIONS];
    int n_mmio;
    
    /* iBoot signatures */
    iboot_sig_t iboot[MAX_IBOOT_SIGS];
    int n_iboot;
    
    /* SoC info */
    char device_model[64];   /* "iPhone11,8" */
    char soc_name[64];       /* "T8020" */
    char chip_name[64];      /* "A12 Bionic" */
    
    /* Interrupt controller info */
    char aic_compatible[128];
    uint64_t aic_base;
    uint64_t aic_size;
    int aic_num_irqs;
    
    /* UART info */
    uint64_t uart_bases[4];
    int n_uarts;
    
    /* JSON buffer (internal) */
    void *json_buf;
} recon_ctx_t;

/* ═══════════════════════════════════════════════════════════════════════════
 * Module entry points
 * ═══════════════════════════════════════════════════════════════════════════ */

/* devtree.c — IODeviceTree extraction via IOKit */
int recon_dump_devtree(recon_ctx_t *ctx);

/* memmap.c — Physical memory map via IOKit / task_for_pid */
int recon_dump_memmap(recon_ctx_t *ctx);

/* iboot_scan.c — Scan physical memory for iBoot signatures */
int recon_scan_iboot(recon_ctx_t *ctx);

/* mmio_log.c — Live MMIO access monitor */
int recon_log_mmio(recon_ctx_t *ctx);

/* serial_bridge.c — TCP→serial relay bridge */
int recon_serial_bridge(recon_ctx_t *ctx, int port);

/* json_out.c — JSON serialization */
void json_init(recon_ctx_t *ctx);
void json_finalize(recon_ctx_t *ctx);

#endif /* TRUSTOS_RECON_H */
