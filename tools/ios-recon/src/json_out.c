/*
 * TrustOS iOS Recon — JSON Output
 *
 * Serializes all recon data to a JSON file that can be consumed
 * by TrustOS driver generators and the kernel build system.
 *
 * The output JSON is designed to be parsed by:
 *   tools/gen_apple_drivers.py  — Generates Rust driver stubs
 *   kernel/src/drivers/apple/   — Runtime hardware config
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "recon.h"

static FILE *out_fp = NULL;
static int indent_level = 0;

static void json_indent(void) {
    for (int i = 0; i < indent_level; i++) fprintf(out_fp, "  ");
}

static void json_str(const char *key, const char *val, int comma) {
    json_indent();
    /* Escape special chars in value */
    fprintf(out_fp, "\"%s\": \"", key);
    for (const char *p = val; *p; p++) {
        switch (*p) {
            case '"':  fprintf(out_fp, "\\\""); break;
            case '\\': fprintf(out_fp, "\\\\"); break;
            case '\n': fprintf(out_fp, "\\n"); break;
            case '\r': fprintf(out_fp, "\\r"); break;
            case '\t': fprintf(out_fp, "\\t"); break;
            default:
                if ((unsigned char)*p >= 0x20)
                    fputc(*p, out_fp);
                break;
        }
    }
    fprintf(out_fp, "\"%s\n", comma ? "," : "");
}

static void json_u64(const char *key, uint64_t val, int comma) {
    json_indent();
    if (val <= 0xFFFFFFFF) {
        fprintf(out_fp, "\"%s\": %llu%s\n", key, (unsigned long long)val, comma ? "," : "");
    } else {
        fprintf(out_fp, "\"%s\": \"0x%llx\"%s\n", key, (unsigned long long)val, comma ? "," : "");
    }
}

static void json_int(const char *key, int val, int comma) {
    json_indent();
    fprintf(out_fp, "\"%s\": %d%s\n", key, val, comma ? "," : "");
}

void json_init(recon_ctx_t *ctx) {
    if (ctx->output_file) {
        out_fp = fopen(ctx->output_file, "w");
        if (!out_fp) {
            fprintf(stderr, "[!] Cannot open %s for writing\n", ctx->output_file);
            out_fp = stdout;
        }
    } else {
        out_fp = stdout;
    }
}

void json_finalize(recon_ctx_t *ctx) {
    if (!out_fp) return;
    
    /* Only write JSON if we have data */
    if (ctx->n_devtree == 0 && ctx->n_memmap == 0 && ctx->n_iboot == 0) {
        printf("[*] No data collected — skipping JSON output\n");
        return;
    }
    
    /* If output is stdout and we didn't specify --output, write to a temp file */
    FILE *json_fp = out_fp;
    if (out_fp == stdout && ctx->output_file == NULL) {
        json_fp = fopen("/tmp/trustos_recon.json", "w");
        if (!json_fp) json_fp = stdout;
        else printf("\n[*] Writing JSON to /tmp/trustos_recon.json\n");
    }
    
    FILE *save_fp = out_fp;
    out_fp = json_fp;
    indent_level = 0;
    
    fprintf(out_fp, "{\n");
    indent_level++;
    
    /* === Device info === */
    json_str("tool_version", VERSION, 1);
    json_str("device_model", ctx->device_model, 1);
    json_str("soc_name", ctx->soc_name, 1);
    json_str("chip_name", ctx->chip_name, 1);
    
    /* === Interrupt Controller === */
    json_indent();
    fprintf(out_fp, "\"interrupt_controller\": {\n");
    indent_level++;
    json_str("compatible", ctx->aic_compatible, 1);
    json_u64("base", ctx->aic_base, 1);
    json_u64("size", ctx->aic_size, 1);
    json_int("num_irqs", ctx->aic_num_irqs, 0);
    indent_level--;
    json_indent();
    fprintf(out_fp, "},\n");
    
    /* === UARTs === */
    json_indent();
    fprintf(out_fp, "\"uarts\": [\n");
    indent_level++;
    for (int i = 0; i < ctx->n_uarts; i++) {
        json_indent();
        fprintf(out_fp, "\"0x%llx\"%s\n", 
                (unsigned long long)ctx->uart_bases[i],
                i < ctx->n_uarts - 1 ? "," : "");
    }
    indent_level--;
    json_indent();
    fprintf(out_fp, "],\n");
    
    /* === Device Tree === */
    json_indent();
    fprintf(out_fp, "\"device_tree\": [\n");
    indent_level++;
    for (int i = 0; i < ctx->n_devtree; i++) {
        devtree_node_t *n = &ctx->devtree[i];
        json_indent();
        fprintf(out_fp, "{\n");
        indent_level++;
        json_str("name", n->name, 1);
        json_str("compatible", n->compatible, 1);
        json_u64("reg_base", n->reg_base, 1);
        json_u64("reg_size", n->reg_size, 1);
        
        json_indent();
        fprintf(out_fp, "\"interrupts\": [");
        for (int j = 0; j < n->n_interrupts; j++) {
            fprintf(out_fp, "%u%s", n->interrupts[j], j < n->n_interrupts - 1 ? ", " : "");
        }
        fprintf(out_fp, "],\n");
        
        json_str("clock_domain", n->clock_domain, 0);
        indent_level--;
        json_indent();
        fprintf(out_fp, "}%s\n", i < ctx->n_devtree - 1 ? "," : "");
    }
    indent_level--;
    json_indent();
    fprintf(out_fp, "],\n");
    
    /* === Memory Map === */
    json_indent();
    fprintf(out_fp, "\"memory_map\": [\n");
    indent_level++;
    for (int i = 0; i < ctx->n_memmap; i++) {
        memmap_entry_t *m = &ctx->memmap[i];
        json_indent();
        fprintf(out_fp, "{\n");
        indent_level++;
        json_str("name", m->name, 1);
        json_str("type", m->type, 1);
        json_u64("base", m->base, 1);
        json_u64("size", m->size, 0);
        indent_level--;
        json_indent();
        fprintf(out_fp, "}%s\n", i < ctx->n_memmap - 1 ? "," : "");
    }
    indent_level--;
    json_indent();
    fprintf(out_fp, "],\n");
    
    /* === MMIO Regions === */
    json_indent();
    fprintf(out_fp, "\"mmio_regions\": [\n");
    indent_level++;
    for (int i = 0; i < ctx->n_mmio; i++) {
        mmio_region_t *r = &ctx->mmio[i];
        json_indent();
        fprintf(out_fp, "{\n");
        indent_level++;
        json_str("device", r->device, 1);
        json_str("compatible", r->compatible, 1);
        json_u64("base", r->base, 1);
        json_u64("size", r->size, 0);
        indent_level--;
        json_indent();
        fprintf(out_fp, "}%s\n", i < ctx->n_mmio - 1 ? "," : "");
    }
    indent_level--;
    json_indent();
    fprintf(out_fp, "],\n");
    
    /* === iBoot Signatures === */
    json_indent();
    fprintf(out_fp, "\"iboot_signatures\": [\n");
    indent_level++;
    for (int i = 0; i < ctx->n_iboot; i++) {
        iboot_sig_t *s = &ctx->iboot[i];
        json_indent();
        fprintf(out_fp, "{\n");
        indent_level++;
        json_str("version", s->version, 1);
        json_u64("address", s->address, 1);
        json_indent();
        fprintf(out_fp, "\"crc\": \"0x%08x\",\n", s->crc);
        json_u64("region_size", s->region_size, 0);
        indent_level--;
        json_indent();
        fprintf(out_fp, "}%s\n", i < ctx->n_iboot - 1 ? "," : "");
    }
    indent_level--;
    json_indent();
    fprintf(out_fp, "]\n");
    
    indent_level--;
    fprintf(out_fp, "}\n");
    
    if (json_fp != stdout && json_fp != save_fp) {
        fclose(json_fp);
    }
    
    out_fp = save_fp;
}
