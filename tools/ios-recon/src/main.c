/*
 * TrustOS iOS Recon Tool — main entry point
 * 
 * Hardware reconnaissance for jailbroken iPhone devices.
 * Dumps device tree, memory map, iBoot signatures, MMIO regions.
 * Output: JSON hardware map for TrustOS driver development.
 *
 * Copyright (c) 2026 TrustOS Project — MIT License
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/utsname.h>

#include "recon.h"

static void print_banner(void) {
    printf("╔══════════════════════════════════════════════════╗\n");
    printf("║  TrustOS iOS Recon v" VERSION "                        ║\n");
    printf("║  Hardware reconnaissance for bare-metal drivers  ║\n");
    printf("╚══════════════════════════════════════════════════╝\n\n");
}

static void print_usage(const char *prog) {
    printf("Usage: %s [OPTIONS]\n\n", prog);
    printf("Options:\n");
    printf("  --all                Run all recon modules, dump to JSON\n");
    printf("  --devtree            Dump IODeviceTree (MMIO, IRQ, clocks)\n");
    printf("  --memmap             Physical memory map\n");
    printf("  --iboot-scan         Scan RAM for iBoot signatures\n");
    printf("  --mmio-log           Live MMIO access monitor\n");
    printf("  --serial-bridge PORT TCP→serial relay bridge\n");
    printf("  --output FILE        Output JSON file (default: stdout)\n");
    printf("  --verbose            Extra debug output\n");
    printf("  --help               This help text\n");
}

static void print_system_info(void) {
    struct utsname uts;
    if (uname(&uts) == 0) {
        printf("[*] System: %s %s %s\n", uts.sysname, uts.release, uts.machine);
    }
    
    /* Check if we're root */
    if (getuid() != 0) {
        printf("[!] WARNING: Not running as root. Some modules need root access.\n");
        printf("[!] Run with: sudo %s --all\n\n", "trustos-recon");
    } else {
        printf("[+] Running as root — full hardware access available\n");
    }
}

int main(int argc, char **argv) {
    int do_all = 0;
    int do_devtree = 0;
    int do_memmap = 0;
    int do_iboot = 0;
    int do_mmio = 0;
    int do_serial = 0;
    int serial_port = 0;
    const char *output_file = NULL;
    int verbose = 0;
    
    if (argc < 2) {
        print_banner();
        print_usage(argv[0]);
        return 1;
    }
    
    /* Parse arguments */
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--all") == 0) {
            do_all = 1;
        } else if (strcmp(argv[i], "--devtree") == 0) {
            do_devtree = 1;
        } else if (strcmp(argv[i], "--memmap") == 0) {
            do_memmap = 1;
        } else if (strcmp(argv[i], "--iboot-scan") == 0) {
            do_iboot = 1;
        } else if (strcmp(argv[i], "--mmio-log") == 0) {
            do_mmio = 1;
        } else if (strcmp(argv[i], "--serial-bridge") == 0) {
            do_serial = 1;
            if (i + 1 < argc) {
                serial_port = atoi(argv[++i]);
            } else {
                fprintf(stderr, "Error: --serial-bridge requires a port number\n");
                return 1;
            }
        } else if (strcmp(argv[i], "--output") == 0) {
            if (i + 1 < argc) {
                output_file = argv[++i];
            } else {
                fprintf(stderr, "Error: --output requires a filename\n");
                return 1;
            }
        } else if (strcmp(argv[i], "--verbose") == 0) {
            verbose = 1;
        } else if (strcmp(argv[i], "--help") == 0) {
            print_banner();
            print_usage(argv[0]);
            return 0;
        } else {
            fprintf(stderr, "Unknown option: %s\n", argv[i]);
            return 1;
        }
    }
    
    print_banner();
    print_system_info();
    
    /* Initialize JSON output context */
    recon_ctx_t ctx;
    memset(&ctx, 0, sizeof(ctx));
    ctx.verbose = verbose;
    ctx.output_file = output_file;
    json_init(&ctx);
    
    /* Run selected modules */
    if (do_all || do_devtree) {
        printf("\n[===== Device Tree Dump =====]\n");
        recon_dump_devtree(&ctx);
    }
    
    if (do_all || do_memmap) {
        printf("\n[===== Physical Memory Map =====]\n");
        recon_dump_memmap(&ctx);
    }
    
    if (do_all || do_iboot) {
        printf("\n[===== iBoot Memory Scan =====]\n");
        recon_scan_iboot(&ctx);
    }
    
    if (do_all || do_mmio) {
        printf("\n[===== MMIO Logger =====]\n");
        recon_log_mmio(&ctx);
    }
    
    if (do_serial) {
        printf("\n[===== Serial Bridge =====]\n");
        recon_serial_bridge(&ctx, serial_port);
    }
    
    /* Write JSON output */
    json_finalize(&ctx);
    
    printf("\n[+] Recon complete.\n");
    if (output_file) {
        printf("[+] Data written to: %s\n", output_file);
    }
    
    return 0;
}
