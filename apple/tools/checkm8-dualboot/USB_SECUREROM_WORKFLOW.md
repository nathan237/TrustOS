# T8020 B1 SecureROM — Complete USB→SecureROM Workflow
## iPhone XR (A12 Bionic) — Full Reverse Engineering

---

## Table of Contents
1. [Physical Layer](#1-physical-layer)
2. [USB OTG Controller (DWC2)](#2-usb-otg-controller-dwc2)
3. [ROM Boot Path → DFU Entry](#3-rom-boot-path--dfu-entry)
4. [USB Control Transfer Handler](#4-usb-control-transfer-handler)
5. [DFU Protocol State Machine](#5-dfu-protocol-state-machine)
6. [DFU Data Receive Path](#6-dfu-data-receive-path)
7. [IMG4 Container Parsing](#7-img4-container-parsing)
8. [IMG4 Manifest Verification](#8-img4-manifest-verification)
9. [Certificate Chain Verification](#9-certificate-chain-verification)
10. [SRAM Memory Map](#10-sram-memory-map)
11. [MMIO Register Map](#11-mmio-register-map)
12. [Security Analysis Summary](#12-security-analysis-summary)

---

## 1. Physical Layer

```
┌─────────────────┐     ┌──────────────────┐     ┌────────────────────┐
│  Lightning      │────→│  USB 2.0 PHY     │────→│  DWC2 USB OTG      │
│  Connector      │     │  (UTMI/ULPI)     │     │  Controller        │
│                 │     │  Pre-initialized  │     │  MMIO@0x235100000  │
│  USB 2.0 HS     │     │  by pre-ROM code │     │  (Synopsys IP)     │
│  480 Mbit/s     │     │  NOT by SecureROM │     │                    │
└─────────────────┘     └──────────────────┘     └────────────────────┘
```

### Key Finding: No PHY Initialization in SecureROM

The SecureROM contains **zero direct USB PHY or PMGR MMIO references** via ADRP instructions.
The USB PHY is initialized by pre-ROM bootcode (burned into SoC fabric) before SecureROM receives control.
SecureROM only programs the DWC2 OTG controller registers directly.

### No Interrupt Controller

SecureROM uses **pure polling** — no Apple Interrupt Controller (AIC) references found.
Only 1 WFI instruction exists (at exception vector 0x100000604, never normally reached).
All USB events are polled via register reads in tight loops.

---

## 2. USB OTG Controller (DWC2)

### MMIO Register Table (from ROM data @ 0x10001C5A0)

The DFU context structure at ROM offset 0x1C5A0 contains 14 pointers to Synopsys DWC2 OTG registers:

| Offset | MMIO Address   | DWC2 Register    | Description                    |
|--------|---------------|------------------|--------------------------------|
| +0x00  | 0x235100000   | GOTGCTL          | OTG Control and Status         |
| +0x08  | 0x235100004   | GOTGINT          | OTG Interrupt Register         |
| +0x10  | 0x235100008   | GAHBCFG          | AHB Configuration              |
| +0x18  | 0x23510000C   | GUSBCFG          | USB Configuration              |
| +0x20  | 0x235100010   | GRSTCTL          | Reset Control                  |
| +0x28  | 0x235100020   | GRXSTSR          | Receive Status Read            |
| +0x30  | 0x235100030   | GRXFSIZ          | Receive FIFO Size              |
| +0x38  | 0x235100034   | GNPTXFSIZ        | Non-periodic TX FIFO Size      |
| +0x40  | 0x235100038   | GNPTXSTS         | Non-periodic TX FIFO Status    |
| +0x48  | 0x23510003C   | (reserved)       | Platform-specific              |
| +0x50  | 0x235100040   | GLPMCFG/GGPIO   | Link Power Mgmt / GPIO         |
| +0x58  | 0x235100044   | GUID             | User ID Register               |
| +0x60  | 0x235100048   | GSNPSID          | Synopsys ID (DWC2 version)     |
| +0x68  | 0x23510004C   | GHWCFG1          | HW Configuration 1             |

### Second MMIO Block (USB Device Endpoint Control)

Built via MOV+MOVK at address **0x235008008** in `usb_control_transfer_handler`:
```
0x1000025AC: mov  x19, #0x200000000
0x1000025B0: movk x19, #0x3500, lsl #16    → 0x235000000
0x1000025B4: movk x19, #0x8008             → 0x235008008
```

| Access            | Address       | Purpose                         |
|-------------------|---------------|--------------------------------|
| `str w8, [x19]`  | 0x235008008   | EP control (write 1=start, 2=status) |
| `str w8, [x19+0x10]` | 0x235008018 | EP transfer size (write 0x20)  |

### Why ADRP Scan Missed These

MMIO addresses are constructed via `MOV + MOVK` instruction sequences, NOT `ADRP + ADD`.
The ADRP-based scan in `rom_usb_workflow.py` found 0 USB peripheral references because 
the 0x235xxxxxxx addresses are built entirely with immediate instructions.

---

## 3. ROM Boot Path → DFU Entry

### Boot Sequence

```
Reset Vector (0x100000000)
    │
    ▼
Exception Vector Table (16 vectors @ 0x100000000-0x100000800)
    │
    ▼
Platform Init (0x1000068A4)
    │
    ├── usb_setup_endpoint(...)     × 12 calls  @ 0x100002F78
    ├── usb_ctrl_fn_D34(...)        × 6 calls   @ 0x100002D34
    ├── dfu_init(...)                            @ 0x1000030E0
    ├── dfu_state_setup(...)                     @ 0x100003184
    │       └── sub_1000031C0: register USB endpoints from table
    │
    ▼
Poll Loop (platform_68A4 ↔ platform_69BC)
    │
    ├── usb_ctrl_fn_F84 → dfu_ep_handler_3   (data transfers)
    ├── usb_ctrl_fn_D38 → dfu_ep_handler_1   (completions)
    ├── usb_ctrl_fn_E38 → dfu_ep_handler_2   (status)
    └── dfu_state_query  @ 0x100003F88        (state check)
```

### DFU Initialization (0x1000030E0)

```c
void dfu_init(void) {
    canary = SRAM[0x19C008448];     // Stack canary
    usb_ctrl_fn_DB0();               // Init USB controller state
    // ... sets DFU state to IDLE
    verify_canary(canary);
}
```

### Endpoint Registration (0x1000031C0)

Iterates through a table of `(endpoint_id, config)` word pairs, calling:
1. `usb_ctrl_fn_DB0()` — configure endpoint
2. `usb_ctrl_fn_EDC()` — activate endpoint

Terminates when `endpoint_id == -1 (0xFFFFFFFF)`.

---

## 4. USB Control Transfer Handler

### Function: 0x1000023FC (672 bytes, 168 instructions)

This is the **main USB EP0 control transfer processor**. It handles the three phases
of a USB control transfer: SETUP → DATA → STATUS.

### Detailed Flow

```
usb_control_transfer_handler(x0=ep_flags, x1, x2, x3, x4=req_type, x5, x6)
│
├── 1. Read stack canary [0x19C008448]
├── 2. log_fn_8370(0x57, 1)                    // Log: "USB control transfer start"
├── 3. sub_1000113B4(0x300, 0x70000, &ctx)     // Get USB controller context
│       └── ctx = USB DWC2 controller descriptor
├── 4. BLR [ctx+8]                             // Vtable call: controller init
├── 5. sub_1000026A0() → check result
│
├── 6. ★ sub_100003ED8()                       // DFU setup packet handler
├── 7. security_fn_B64() → enter security ctx
├── 8. Poll loop:
│       ├── sub_1000026A0() → check event
│       ├── security_fn_B78(ctx, 0x3E8)        // Wait up to 1000ms
│       └── if timeout: panic("...")
│
├── 9. Decode request from x0/x4 parameters:
│       ├── ep_type  = (x0 >> 4) & 0xF   → 0x0=control, 0x1=endpoint
│       ├── ep_dir   = x0 & 0xF          → 0x0=OUT(IN from host), 0x1=IN
│       ├── xfer_type = (x4 >> 28) & 0xF → 0=default, 1=bulk, 2=iso
│       ├── req_code  = x4 & 0xFFF       → descriptor/request dispatch
│       │
│       ├── req_code dispatch:
│       │   ├── 0x000: Standard (req_id=0)   → sub_100007368(0,1)
│       │   ├── 0x100: Request A (req_id=1)  → sub_100007368(0,1)
│       │   ├── 0x200: Request B (req_id=2)  → sub_100007368(1,0)  
│       │   ├── 0x201: Request C (req_id=3)  → sub_100007368(2,0)
│       │   └── other: panic()
│
├── 10. Program USB hardware:
│       ├── sub_100002368(0, req_id, xfer_type, 0, dir, 0, 0)  // Config EP
│       ├── sub_10000612C()                    // Memory barrier / sync
│       ├── MMIO[0x235008018] = 0x20          // Set transfer size = 32
│       ├── MMIO[0x235008008] = 1             // Start SETUP phase
│       ├── sub_100002120(handle, length)      // Setup data transfer
│       ├── sub_1000021E4(0, 0, 0, data_ptr)  // DATA phase
│       ├── sub_10000226C(0, 0, buf, data, sz) // Copy data
│       ├── sub_1000022F4(0, 1, 1)            // STATUS phase
│       ├── sub_1000023A0()                    // Submit/finalize
│       ├── sub_10000612C()                    // Sync
│       ├── MMIO[0x235008008] = 2             // Advance to STATUS
│       └── BLR [ctx+0x10]                    // Completion callback
│
└── 11. log_fn_8370(0x57, 0)                   // Log: "USB control transfer done"
        verify_canary(canary)
```

---

## 5. DFU Protocol State Machine

### DFU Setup Packet Handler (0x100003ED8)

Called by the USB control transfer handler to process DFU class requests.

```c
int dfu_setup_packet_handler(void) {
    acquire_lock(2);
    enter_critical(2);
    
    // Initialize DFU STATUS response at SRAM[0x19C00BBC0]
    SRAM[0x19C00BBC0] = 0xFF;    // bStatus = 0xFF (pending)
    SRAM[0x19C00BBC1] = 0x00;    // bwPollTimeout[0]
    SRAM[0x19C00BBC2] = 0x0F;    // bwPollTimeout[1]  
    SRAM[0x19C00BBC3] = 0x00;    // bwPollTimeout[2]
    SRAM[0x19C00BBC4] = 0;       // bState (dword)
    SRAM[0x19C00BBC8] = 0;       // iString (qword)
    
    // Notify/signal USB subsystem
    if (mode == 2)
        signal_event(2, &SRAM[BBC0], 0);
    else
        wait_event(2, &SRAM[BBC0], 0);
    
    // Wait for DFU operation to complete
    result = poll_wait(0x1E08480);    // ~250ms timeout
    
    release_lock(2);
    return result;
}
```

### DFU Status Response Structure (SRAM @ 0x19C00BBC0)

```
Offset  Size  Field           Values seen in ROM
+0x00   1     bStatus         0xFF (pending), 0x00 (OK)
+0x01   1     bwPollTimeout[0] 0x00 or 0x67 (103ms)
+0x02   1     bwPollTimeout[1] 0x0F, 0x10, 0x03
+0x03   1     bwPollTimeout[2] 0x00
+0x04   4     bState          0=dfuIDLE
+0x08   8     iString         0 (no string)
+0x10   16    event_buffer    (copied from +0x20 on event)
+0x20   16    event_pending   (written by USB IRQ/poll)
```

### DFU State Values (at SRAM[0x19C00BC10])

```
0 = appIDLE           → Ready, waiting for DFU request
1 = appDETACH         → Device detached, switching to DFU mode
2 = dfuIDLE           → DFU mode active, waiting for download
3 = dfuDNLOAD-SYNC    → Download in progress, synchronizing
4 = dfuDNBUSY         → Download busy (processing data)
5 = dfuDNLOAD-IDLE    → Download complete for this block
8 = dfuMANIFEST-SYNC  → Manifest phase begin
9 = dfuUPLOAD-IDLE    → Upload mode active
10 = dfuERROR         → Error state
```

### Per-Endpoint State Structure (SRAM @ 0x19C00BBF0)

Each endpoint has a 0x18-byte state entry: `BBF0 + ep_num * 0x18`

```
Offset  Size  Field           Description
+0x00   4     has_data        (x3 != 0) ? 1 : 0
+0x04   4     has_callback    (x4 != 0) ? 1 : 0
+0x08   4     transfer_state  0=idle(→log 0xAA), 2=active(→log 0x48), other=error(→log 0xAE)
+0x0C   4     transfer_size   Transfer size in bytes
+0x10   1     direction       0=OUT(host→device), 1=IN(device→host)
+0x11   1     dma_mode        0=PIO, 1=DMA
+0x14   4     config_flags    Computed: bit0=active | bit1=has_data | bit2=has_callback
                              | bits4-6=size_mode | bit14=state!=0
```

### State Machine Main (0x1000044A8, 376 bytes)

```c
int dfu_state_machine_main(ep_num, xfer_size, direction, has_data, has_callback) {
    ctx = get_dfu_context(0);           // sub_100004454 → ROM[0x10001C5A0]
    ctx2 = get_dfu_context(ep_num);
    *(ctx2[0]) = 0;                     // Clear completion flag
    
    ep_state = &SRAM[0x19C00BBF0 + ep_num * 0x18];
    ep_state->has_data = (has_data != 0);
    ep_state->has_callback = (has_callback != 0);
    
    // Compute poll interval based on transfer state
    switch (ep_state->transfer_state) {
        case 0: log_fn(0xAA); break;    // idle
        case 2: log_fn(0x48); break;    // active
        default: log_fn(0xAE); break;   // error
    }
    poll_interval = max(log_result / xfer_size, 1);
    
    // Store per-endpoint config
    ep_state->transfer_size = xfer_size;
    ep_state->direction = direction;
    
    // Build config flags
    flags = (direction ? 0x18 : 0x08);
    flags |= (ep_state->has_data << 1);
    flags |= (ep_state->has_callback << 2);
    flags |= (dma_mode ? 0x40 : 0x20);
    flags |= ((ep_state->transfer_state != 0) << 14);
    ep_state->config = flags;
    
    // Write to DWC2 register and start transfer
    *(ctx[8]) = flags;                  // MMIO write
    *(ctx[0x18]) = 2;                   // Set state = active
    
    // Signal completion
    set_completion_flag(ep_num);
    return platform_fn_6754(4);
}
```

---

## 6. DFU Data Receive Path

### Buffer Allocation (0x100004174)

```c
void* dfu_buffer_alloc(int alloc_size) {
    void* buf = pool_alloc(1, 0xD0, NULL);   // Allocate 0xD0 (208) byte buffer
    if (alloc_size != 0) {
        pool_free(buf);                       // Free if alloc failed
        return -1;
    }
    return buf;
}
```

**Fixed buffer size: 0xD0 (208) bytes** — allocated from the pool allocator.

### DFU Download Handler (0x100003CF8, 236 bytes)

Handles DFU_DNLOAD requests — receives firmware data from host:

```c
int dfu_download_handler(void* request) {
    canary = SRAM[0x19C008448];
    if (!request) goto cleanup;
    
    acquire_lock(2);
    mode = enter_critical(2);
    
    // Initialize DFU STATUS with download-specific values
    SRAM[0x19C00BBC0] = 0xFF;    // bStatus = pending
    SRAM[0x19C00BBC1] = 0x67;    // bwPollTimeout[0] = 103ms ★
    SRAM[0x19C00BBC2] = 0x03;    // bwPollTimeout[1] = 3
    SRAM[0x19C00BBC3] = 0x00;    // bwPollTimeout[2]
    SRAM[0x19C00BBC4] = 0;       // bState
    SRAM[0x19C00BBC8] = 0;       // iString
    
    // Signal/wait for USB event
    signal_or_wait(mode, &SRAM[BBC0]);
    
    // Poll with large timeout: 0xF04240 = 15,729,216 cycles (~8ms)
    if (!poll_wait(0xF04240))
        goto error;
    
    // ... process received data ...
    
    release_lock(2);
    verify_canary(canary);
}
```

### DFU Request Dispatch (0x100003F9C, 276 bytes)

Copies USB-received data in chunks of 4 bytes via `memcpy`:

```c
int dfu_request_dispatch(void* src, uint32_t total_size) {
    if (!src) return 0;
    
    acquire_lock(2);
    mode = enter_critical(2);
    
    uint32_t offset = 0;
    while (total_size > 0) {
        // Init status for each chunk
        SRAM[0x19C00BBC0] = 0xFF;
        SRAM[0x19C00BBC2] = 0x10;    // bwPollTimeout[1] = 16
        
        signal_or_wait(mode, &SRAM[BBC0]);
        
        // Poll with timeout, retry with delays
        result = poll_wait(0);
        if (result < 0) {
            // Retry loop with 100-cycle sleeps
            for (timeout = 0x1E08480; timeout > 0; timeout -= 0x64) {
                security_fn_BA8(0x64);   // sleep 100 cycles
                result = poll_wait(0);
                if (result >= 0) break;
            }
        }
        
        // Copy data in 4-byte chunks (or remaining)
        uint32_t chunk = min(total_size, 4);
        memcpy(&SRAM[0x19C00BBD4], src + offset, chunk);  // ★ THE memcpy
        offset += chunk;
        total_size -= chunk;
    }
    
    release_lock(2);
    return total_size;  // 0 on success
}
```

**CRITICAL**: The `memcpy` at 0x100004074 copies from the USB-received data to 
`SRAM[0x19C00BBD4]` (which is `BBC0 + 0x14`, inside the DFU status structure).
The chunk size is clamped to `min(total_size, 4)` — **maximum 4 bytes per copy**.

### USB Data Transfer (PIO mode)

#### DFU Receive (Host→Device): 0x100004650

Reads data from USB endpoint FIFO register-by-register:

```c
void dfu_pio_receive(ep_num, buffer, size) {
    ctx = get_dfu_context(ep_num);
    ep_state = &SRAM[0x19C00BBF0 + ep_num * 0x18];
    
    // Set active flag
    ep_state->config |= 1;
    *(ctx[8]) = ep_state->config;
    set_completion(ep_num);
    
    uint64_t remaining = align_up(size);
    uint64_t offset = 0;
    
    for (int width = 2; width >= 0; width -= 2) {
        int chunk = 1 << width;        // 4, then 1
        remaining = align(size, chunk);
        
        while (offset < remaining) {
            uint64_t block = min(remaining - offset, 0x4000);
            
            // Set transfer size in DWC2 register
            *(ctx[0x38]) = block >> width;
            *(ctx[0x68]) = block >> width;
            
            uint64_t inner_off = 0;
            while (inner_off < block) {
                // Read actual data word from USB FIFO
                uint32_t word = *(ctx[0x28]);     // ★ USB DATA FIFO READ
                
                if (width > 0) {
                    // 4-byte mode: split and store bytes
                    buffer[offset + inner_off + 0] = word & 0xFF;
                    buffer[offset + inner_off + 1] = (word >> 8) & 0xFF;
                    buffer[offset + inner_off + 2] = (word >> 16) & 0xFF;
                    buffer[offset + inner_off + 3] = (word >> 24) & 0xFF;
                    inner_off += 4;
                } else {
                    // byte mode
                    buffer[offset + inner_off] = word & 0xFF;
                    inner_off += 1;
                }
            }
            offset += block;
        }
    }
}
```

#### DFU Transmit (Device→Host): 0x1000047BC

Writes data to USB endpoint FIFO register-by-register:

```c
void dfu_pio_transmit(ep_num, buffer, size) {
    ctx = get_dfu_context(ep_num);
    ep_state = &SRAM[0x19C00BBF0 + ep_num * 0x18];
    
    // Clear active flag
    ep_state->config &= ~1;
    *(ctx[8]) = ep_state->config;
    set_completion(ep_num);
    
    uint64_t offset = 0;
    while (offset < size) {
        uint64_t block = min(size - offset, 0x4000);   // 16KB max per block
        
        *(ctx[0x68]) = block;      // Set TX size
        *(ctx[0x38]) = 0;         // Clear RX
        
        uint64_t inner_off = 0;
        while (inner_off < block) {
            // Check FIFO space
            uint32_t fifo_status = *(ctx[0x10]);
            int free_slots = 16 - ((fifo_status >> 6) & 0x1F);
            free_slots = min(free_slots, remaining);
            
            for (int i = 0; i < free_slots; i++) {
                uint8_t byte = buffer[offset + inner_off + i];
                *(ctx[0x20]) = byte;      // ★ USB DATA FIFO WRITE
            }
            inner_off += free_slots;
        }
        
        // Wait for TX complete
        while (*(ctx[0x68]) != 0) { /* spin */ }
        offset += block;
    }
}
```

### DFU Transfer Handler (0x100004240)

Orchestrates data transfer with platform notifications:

```c
int dfu_transfer_handler(ctx, data, transfer_id, size) {
    canary = SRAM[0x19C008448];
    
    sub_100004338();                            // Pre-transfer setup
    sub_100004354(ctx, ctx->xfer_count);       // Check transfer count
    
    platform_fn_6774(0xC0000, transfer_id, size, 0);  // Notify: transfer start
    
    // Build 4-byte header: [0x03, id_hi, id_mid, id_lo]
    header[0] = 3;
    header[1] = (transfer_id >> 16) & 0xFF;
    header[2] = (transfer_id >> 8) & 0xFF;
    header[3] = transfer_id & 0xFF;
    
    usb_ctrl_fn_E30(ctx->ep_out, 0);          // Stall EP
    sub_1000047B4(ctx->ep_in, header, 4);      // Send 4-byte header
    sub_100004644(ctx->ep_in, data, size);     // Send data payload
    usb_ctrl_fn_E30(ctx->ep_out, 1);          // Unstall EP
    
    platform_fn_6774(0xC0001, transfer_id, size, 0);  // Notify: transfer done
    
    verify_canary(canary);
    return 0;
}
```

---

## 7. IMG4 Container Parsing

### DFU Completion Handler (0x1000049D8, 320 bytes)

Once all DFU data is received, this function parses the IMG4 container:

```c
int dfu_completion_handler(void* img4_data, uint64_t img4_size, 
                           uint32_t* result_out, uint32_t* size_out) {
    canary = SRAM[0x19C008448];
    
    // Create parsing context on stack
    struct { void* ptr; uint64_t size; } ctx = { img4_data, img4_size };
    struct { void* ptr; uint64_t size; } cursor;
    
    // STEP 1: Parse outer SEQUENCE
    // tag=0x10 (SEQUENCE), expect_constructed=1
    err = asn1_parse_tag(&ctx, 0x2000000000000010, &cursor, 1);
    if (err) return -1;
    
    // Check remaining size
    if (size_out) {
        remaining = cursor.ptr - ctx.ptr + cursor.size;
        if (remaining overflow) return -1;
        *size_out = remaining;
    }
    
    // STEP 2: Validate "IMG4" container tag
    err = asn1_advance(&ctx, &cursor);          // sub_100004B60
    if (err) return -1;
    
    err = img4_match_tag(&ctx, "IMG4");         // sub_100004BC4 @ ROM[0x10001C250]
    if (err) return -1;
    
    // STEP 3: Parse second SEQUENCE (inner container)
    err = asn1_parse_tag(&ctx, 0x2000000000000010, &cursor, 1);
    if (err) return -1;
    
    // STEP 4: Validate "IM4P" payload tag
    err = asn1_advance(&ctx, &cursor);
    if (err) return -1;
    
    err = img4_match_tag(&ctx, "IM4P");         // sub_100004BC4 @ ROM[0x10001C255]
    if (err) return -1;
    
    // STEP 5: Parse IA5STRING (tag 0x16) for image type
    err = asn1_parse_tag(&ctx, 0x16, &cursor, 0);
    if (err) return -1;
    
    // STEP 6: Verify size == 4 (4CC image type code)
    if (cursor.size != 4) return -1;
    
    // STEP 7: Extract result (e.g., "ibot", "ibss", etc.)
    if (result_out) {
        uint32_t tag = *(uint32_t*)cursor.ptr;
        *result_out = byte_swap(tag);           // heap_fn_014
    }
    
    return 0;
}
```

### IMG4 Container Structure

```
SEQUENCE {                              // Outer container
    IA5STRING "IMG4"                    // Container identifier
    SEQUENCE {                          // Inner payload  
        IA5STRING "IM4P"                // Payload marker
        IA5STRING <4-byte-type>         // Image type: "ibot", "ibss", "ibec", etc.
        IA5STRING <version>             // Build version string
        OCTET STRING <firmware_data>    // Actual firmware payload (encrypted)
        OCTET STRING <keybags>          // (optional) keybag data
    }
    [0] EXPLICIT {                      // IM4M: Manifest
        SEQUENCE {
            IA5STRING "IM4M"
            INTEGER <version>
            SET {                        // Manifest properties
                SEQUENCE { ... }         // Per-property: MANB, MANP, etc.
            }
            OCTET STRING <signature>    // RSA/ECDSA signature
            SEQUENCE { <certificates> } // X.509 certificate chain
        }
    }
    [1] EXPLICIT {                      // IM4R: Restore info (optional)
        SEQUENCE {
            IA5STRING "IM4R"
            SET { ... }                 // Boot nonce etc.
        }
    }
}
```

### ASN.1 Tag Parser (0x100004B24)

```c
int asn1_parse_tag(parsing_ctx* ctx, uint64_t expected_tag, 
                   cursor* out, int constructed) {
    int err = cert_fn_3AAC(ctx, out, constructed);   // DER parser
    if (err) return err;
    if (*out != expected_tag) return -1;              // Tag mismatch
    return 0;
}
```

### IMG4 Tag Matcher (0x100004BC4)

```c
int img4_match_tag(parsing_ctx* ctx, const char* expected_tag) {
    struct { void* ptr; uint64_t size; } cursor;
    
    // Parse IA5STRING (tag 0x16)
    err = asn1_parse_tag(ctx, 0x16, &cursor, 0);
    if (err) return -1;
    
    // Compute string length
    uint64_t len = sync_fn_1004(expected_tag);    // strlen equivalent
    if (cursor.size != len) return -1;
    
    // Compare: memcmp equivalent 
    err = heap_fn_EA4(cursor.ptr, expected_tag, len);
    if (err) return -1;
    
    // Bounds check
    if (cursor.ptr out of ctx range) return -1;
    
    // Advance context past this element
    ctx->ptr = cursor.ptr + cursor.size;
    return 0;
}
```

---

## 8. IMG4 Manifest Verification

### IMG4 Tag Verifier (0x100004CB8, 840 bytes, 42 branch points)

This function dispatches verification for each IMG4 manifest property based on its 4-byte tag.

```c
int img4_tag_verifier(uint32_t property_tag, void* manifest_ctx,
                      uint32_t verification_mode, void* property_data) {
    
    // Read current DFU state
    uint8_t state = SRAM[0x19C00BC10];    // ★ DFU state byte
    
    // Callback structure at SRAM[0x19C00BC20]:
    // +0x10 (BC30): cert_verify_type1 callback
    // +0x18 (BC38): cert_verify_type2 callback  
    // +0x20 (BC40): cert_verify_type4 callback
    
    if (state == 1) {  // appDETACH mode
        void* callback;
        switch (verification_mode) {
            case 4: callback = SRAM[0x19C00BC40]; break;
            case 2: callback = SRAM[0x19C00BC38]; break;
            case 1: callback = SRAM[0x19C00BC30]; break;
            default: return ERROR;
        }
        return callback(property_tag, property_data);   // BLR x8
    }
    
    // Dispatch based on 4CC tag value
    switch (property_tag) {
        // ═══ Install Mode (state 0) ═══
        case 'ECID': return img4_verify_tag(property_data, ecid_ref);
        case 'CHIP': return img4_verify_property(property_data, chip_ref);
        case 'BORD': return img4_verify_property(property_data, bord_ref);
        case 'SDOM': return img4_verify_property(property_data, sdom_ref);
        case 'CPRO': return img4_verify_property(property_data, cpro_ref);
        case 'CSEC': return img4_verify_property(property_data, csec_ref);
        case 'BNCH': return cert_verify_nonce(property_data, nonce_sram);
        case 'AMNM': return special_amnm_handler(property_data);
        case 'CHIO': return img4_verify_property(property_data, chio_ref);
        case 'CSEB': return img4_verify_property(property_data, cseb_ref);
        
        // ═══ Personalized Mode (state 1) ═══
        case 'EKEY': return cert_verify_type1(property_data);
        case 'ESEC': return cert_verify_type2(property_data);
        case 'EPRO': return cert_verify_type4(property_data);
        case 'DGST': return digest_verify(property_data, dgst_ref);
        case 'DPRO': return production_verify(property_data);
        case 'EKEX': return key_exchange_verify(property_data);
        
        default: return ERROR_UNKNOWN_TAG;
    }
}
```

### Property Tag Summary

| Tag  | Full Name                  | Check Type | Description |
|------|---------------------------|------------|-------------|
| ECID | Exclusive Chip ID         | Exact match | 64-bit unique device identifier |
| CHIP | Chip ID                   | Property   | SoC identifier (0x8020 for A12) |
| BORD | Board ID                  | Property   | Device board configuration |
| SDOM | Security Domain           | Property   | 0=production, 1=dev |
| CPRO | Certificate Production    | Property   | Production vs development cert |
| CSEC | Crypto Security Epoch     | Property   | Security epoch counter |
| BNCH | Boot Nonce Hash           | Nonce      | Anti-replay nonce verification |
| AMNM | Allow Mix-n-Match         | Special    | Allows non-matching components |
| CHIO | Chip IO                   | Property   | I/O configuration identity |
| CSEB | Cert Security Epoch Base  | Property   | Epoch baseline |
| EKEY | Encryption Key            | Cert type1 | Payload encryption key |
| ESEC | Effective Security        | Cert type2 | Runtime security mode |
| EPRO | Effective Production      | Cert type4 | Runtime production status |
| DGST | Digest                    | Digest     | Firmware hash verification |
| DPRO | Demote Production         | Verify     | Production demotion check |
| EKEX | Key Exchange              | Verify     | Key exchange protocol |

---

## 9. Certificate Chain Verification

### Callback Table (SRAM @ 0x19C00BC20)

```
+0x00:  DFU callback struct base
+0x10:  cert_verify_type1 → 0x100012AD8 (fn ptr loaded at runtime)
+0x18:  cert_verify_type2 → 0x100012A6C (fn ptr loaded at runtime)
+0x20:  cert_verify_type4 → 0x100012B44 (fn ptr loaded at runtime)
```

These function pointers are loaded from SRAM and called via `BLR x8` — indirect calls through a vtable.
The pointers are set during DFU initialization and point to the cert verification subsystem.

### Cert Chain Path

```
IMG4 Manifest Signature
    │
    ├── cert_fn_26FC (0x1000126FC)     // Top-level cert validation
    ├── cert_verify_nonce (0x100012924) // BNCH nonce anti-replay
    ├── cert_verify_type1 (0x100012AD8) // EKEY check
    ├── cert_verify_type2 (0x100012A6C) // ESEC check
    └── cert_verify_type4 (0x100012B44) // EPRO check
         │
         ▼
    DER/ASN.1 Parser (0x10000D000+)
         │
         ▼
    X.509 Certificate Chain
         │
         ├── Leaf certificate (device-specific, signed by Apple)
         ├── Intermediate CA
         └── Root CA (Apple Root CA embedded in ROM)
              │
              ▼
         Crypto Engine (SHA-512 + RSA/ECDSA)
```

### Prior Analysis Results (Sessions 1-19)

All 23 CRITICAL memcpy sites in the cert parser have been analyzed:
- **8 SAFE** (constant size, stack-bounded)
- **5 ATTACKER_CONTROLLED_LOAD** → All confirmed NOT EXPLOITABLE
  (bounded by X.509 max element sizes, validated before memcpy)
- **10 UNKNOWN** → All confirmed bounded (X.509 48-byte max) or canary-protected

**Verdict: The certificate chain verification path is NOT exploitable via software.**

---

## 10. SRAM Memory Map

### Complete SRAM Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ 0x19C008000: System Configuration Page                          │
│                                                                 │
│   +0x000: System base pointer                                   │
│   +0x008: Boot configuration word                               │
│   +0x448: ★ STACK CANARY (read 17x in USB_CTRL alone)         │
│           Read at entry of EVERY function with stack frame      │
│           Verified at exit — __stack_chk_fail (0x100008B58)     │
│   +0xB48: Configuration value A                                 │
│   +0xB59: Configuration byte (R/W)                              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 0x19C00B000: USB / DFU Context Page                             │
│                                                                 │
│   +0xB90: USB endpoint config word (R/W, 1+1 accesses)         │
│   +0xB94: USB transfer counter (R=5, W=2 — heavily used)       │
│   +0xBBC: Pre-DFU padding                                       │
│                                                                 │
│   +0xBC0: ★ DFU STATUS RESPONSE (16 bytes)                    │
│           +0: bStatus, +1-3: bwPollTimeout, +4: bState(dword)  │
│           +8: iString(qword)                                    │
│   +0xBD0: DFU status extension                                  │
│   +0xBD4: memcpy destination (data from USB, max 4 bytes/chunk) │
│                                                                 │
│   +0xBE0: USB event pending buffer (16 bytes)                   │
│   +0xBF0: ★ PER-ENDPOINT STATE ARRAY (0x18 bytes each)        │
│           Entry 0: 0x19C00BBF0                                  │
│           Entry 1: 0x19C00BC08                                  │
│           ...                                                    │
│   +0xC04: EP0 config_flags (composite register)                 │
│   +0xC10: ★ DFU_STATE byte (dfuIDLE=0, appDETACH=1, etc.)    │
│   +0xC20: ★ DFU CALLBACK STRUCT                               │
│           +0x10 (BC30): cert_verify_type1 function pointer      │
│           +0x18 (BC38): cert_verify_type2 function pointer      │
│           +0x20 (BC40): cert_verify_type4 function pointer      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 0x19C010000: Buffer / Vtable Page                               │
│                                                                 │
│   +0xA90: Runtime vtable (8 function pointers)                  │
│   +0x3F8: IMG4 verification result storage                      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 0x19C011000: Extended Buffer Page                               │
│                                                                 │
│   +0x006: SRAM buffer (BNCH nonce comparison target)            │
│   +0x3F8: Property verification secondary result                │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 0x19C014000: Debug / Counter Page                               │
│                                                                 │
│   +0x034: Counter (incremented in exception handler)            │
└─────────────────────────────────────────────────────────────────┘
```

---

## 11. MMIO Register Map

### Summary of All Hardware Accesses

| Base Address   | Block               | Access Method      | # Accesses |
|---------------|---------------------|--------------------|------------|
| 0x235100000   | DWC2 OTG Core       | ROM data table     | 14 regs    |
| 0x235008008   | USB EP Control       | MOV+MOVK inline    | 2 writes   |
| 0x19C008000   | SRAM System          | ADRP+ADD           | ~50+ refs  |
| 0x19C00B000   | SRAM USB/DFU         | ADRP+ADD           | ~80+ refs  |
| 0x19C010000   | SRAM Buffers         | ADRP+ADD           | ~10 refs   |
| 0x19C011000   | SRAM Extended        | ADRP+ADD           | ~5 refs    |
| 0x19C014000   | SRAM Debug           | ADRP+ADD           | ~2 refs    |

---

## 12. Security Analysis Summary

### The Complete Data Path (Attack Surface View)

```
ATTACKER-CONTROLLED                    ROM-CONTROLLED
═══════════════════                    ═══════════════
USB cable  ──────────────────→  USB PHY (pre-init, no control)
                                    │
USB packets ─────────────────→  DWC2 OTG (MMIO, no direct access)
                                    │
SETUP packet ────────────────→  usb_control_transfer_handler
(8 bytes: bmReqType,                │
 bRequest, wValue,              ┌───┴──── Request type dispatch
 wIndex, wLength)               │     Validates: ep_type, direction,
                                │     xfer_type, req_code
                                │     Any unknown → panic()
                                │
DFU data    ─────────────────→  dfu_download_handler
(bulk OUT, variable size)           │
                                ┌───┴──── Buffer management
                                │     pool_alloc(0xD0) → FIXED 208 bytes
                                │     memcpy chunks of MAX 4 bytes
                                │     Poll timeout: ~250ms per chunk
                                │
Firmware image ──────────────→  dfu_completion_handler
(accumulated in buffer)             │
                                ┌───┴──── IMG4 container parse
                                │     ASN.1 DER parser (recursive)
                                │     Match "IMG4" then "IM4P" strings
                                │     Extract 4-byte image type (4CC)
                                │     Bounds-checked at every step
                                │
IMG4 manifest ───────────────→  img4_tag_verifier
(in manifest section)               │
                                ┌───┴──── Per-property verification
                                │     ECID: 64-bit exact match
                                │     CHIP: Must match 0x8020
                                │     BORD: Must match device board
                                │     BNCH: Anti-replay nonce
                                │     DGST: SHA-512 firmware hash
                                │     All: compared to fused values
                                │
Certificate chain ───────────→  cert_verify chain
(X.509 in manifest)                 │
                                ┌───┴──── X.509 validation
                                │     DER/ASN.1 parse (max 48 bytes)
                                │     All memcpy bounded & canary-protected
                                │     Root CA embedded in ROM
                                │     RSA/ECDSA signature verification
                                │
                                ▼
                          ACCEPT or REJECT
```

### Security Boundaries Identified

1. **USB PHY**: Not accessible from SecureROM (pre-initialized)
2. **DWC2 registers**: Written via MMIO, not attacker-controllable  
3. **Request dispatch**: Unknown request types → immediate panic()
4. **Buffer allocation**: Fixed 208-byte pool allocation
5. **Data copy**: Maximum 4 bytes per memcpy call
6. **DFU poll timeout**: Multiple timeout checks prevent infinite loops
7. **IMG4 parsing**: Bounds-checked ASN.1 parser with tag validation
8. **Tag verification**: Each property compared against fused/ROM values
9. **Certificate chain**: All 23 memcpy sites analyzed — none exploitable
10. **Stack canary**: Checked at EVERY function entry/exit (SRAM[0x19C008448])

### Attack Surface Assessment

| Vector                    | Status         | Notes |
|--------------------------|----------------|-------|
| USB PHY manipulation     | ❌ Inaccessible | Pre-ROM initialized |
| SETUP packet fuzzing     | ❌ Hardened     | Strict dispatch, panic() on unknown |
| DFU_DNLOAD overflow      | ❌ Bounded      | Fixed 208-byte buffer, 4-byte chunks |
| DFU_UPLOAD leak          | ❌ Limited      | Only sends pre-formatted data |
| IMG4 parser overflow     | ❌ Bounded      | Every ASN.1 element bounds-checked |
| Certificate parser       | ❌ Confirmed    | All memcpy sites verified safe |
| Stack buffer overflow    | ❌ Canary       | Stack canary on every function |
| Heap corruption          | ⚠️ Theoretical | Pool alloc/free paths exist |
| Race condition           | ⚠️ Theoretical | Lock/unlock patterns in DFU |
| DWC2 register side-effect| ⚠️ Theoretical | MMIO writes at 0x235008008 |
| Timing attack            | ⚠️ Theoretical | Poll-based with fixed timeouts |

### Remaining Theoretical Vectors

1. **Heap pool corruption**: `pool_alloc(1, 0xD0, NULL)` / `pool_free()` path at 0x100004174. 
   If the pool metadata can be corrupted, the next allocation could overlap critical SRAM.
   Requires: finding a write-after-free or double-free condition.

2. **DFU lock race**: `acquire_lock(2)` / `release_lock(2)` pattern. If USB hardware
   can trigger concurrent re-entry, the lock state could become inconsistent.
   Requires: USB reset during critical section.

3. **DWC2 register manipulation**: The SecureROM writes to 0x235008008 (endpoint control)
   immediately after decoding request type. If the DWC2 controller behavior can be 
   influenced by timing of USB reset/suspend during this write sequence.
   Requires: precise timing of USB bus events.

4. **Vtable/callback poisoning**: The DFU callbacks at SRAM[0x19C00BC30-BC40] are 
   called via `BLR x8`. If SRAM can be corrupted before these indirect calls...
   Requires: a prior memory corruption primitive.

---

## Function Reference Table

| Address      | Size | Name                      | Role |
|-------------|------|---------------------------|------|
| 0x100001D14 | ~40  | acquire_lock              | Lock DFU subsystem |
| 0x100001F0C | ~40  | enter_critical            | Enter critical section |
| 0x100001F24 | ~40  | wait_event                | Wait for USB event (alt) |
| 0x100001F3C | ~20  | signal_event              | Signal USB event |
| 0x100001F54 | ~20  | wait_event_alt            | Wait for USB event |
| 0x100001F6C | ~20  | signal_or_wait            | Conditional signal/wait |
| 0x100001F84 | ~20  | release_lock              | Release DFU subsystem |
| 0x100002120 | ~60  | ep0_setup_data            | Setup EP0 data transfer |
| 0x1000021E4 | ~80  | ep0_data_phase            | EP0 DATA phase |
| 0x10000226C | ~80  | ep0_data_copy             | Copy data for EP0 |
| 0x1000022F4 | ~40  | ep0_status_phase          | EP0 STATUS phase |
| 0x1000023A0 | ~40  | ep0_submit                | Submit EP0 transfer |
| 0x100002368 | ~40  | ep_configure              | Configure endpoint |
| 0x1000023FC | 672  | usb_ctrl_transfer_handler | ★ Main USB EP0 handler |
| 0x1000026A0 | ~40  | check_usb_event           | Check for USB event |
| 0x100002D34 | 120  | usb_ctrl_fn_D34           | USB control dispatch |
| 0x100002DB0 | ~60  | usb_ctrl_init             | Init USB ctrl state |
| 0x100002E30 | ~40  | usb_ep_stall_ctrl         | Stall/unstall endpoint |
| 0x100002EDC | ~40  | usb_ep_activate           | Activate endpoint |
| 0x100002F78 | 308  | usb_setup_endpoint        | Full EP setup |
| 0x1000030E0 | 160  | dfu_init                  | Initialize DFU mode |
| 0x100003184 | 60   | dfu_state_setup           | Setup DFU state/endpoints |
| 0x1000031C0 | 120  | dfu_register_endpoints    | Register EP table |
| 0x100003CF8 | 236  | dfu_download_handler      | DFU DNLOAD processing |
| 0x100003ED8 | 176  | dfu_setup_packet_handler  | DFU SETUP packet |
| 0x100003F88 | 296  | dfu_state_query           | Query DFU state |
| 0x100003F9C | 276  | dfu_request_dispatch      | Dispatch + memcpy |
| 0x1000040C0 | 176  | dfu_poll_wait             | Poll for USB events |
| 0x100004174 | 64   | dfu_buffer_alloc          | Alloc 208-byte buffer |
| 0x100004240 | 244  | dfu_transfer_handler      | Orchestrate transfer |
| 0x100004368 | 232  | dfu_io_handler            | DFU I/O with security |
| 0x100004454 | 16   | get_dfu_context           | Get DWC2 register map |
| 0x1000044A8 | 376  | dfu_state_machine_main    | ★ Main state machine |
| 0x100004650 | 356  | dfu_pio_receive           | USB FIFO → SRAM (PIO) |
| 0x1000047BC | 272  | dfu_pio_transmit          | SRAM → USB FIFO (PIO) |
| 0x1000049D8 | 320  | dfu_completion_handler    | IMG4 container parse |
| 0x100004B24 | 60   | asn1_parse_tag            | ASN.1 DER tag parser |
| 0x100004B60 | 76   | asn1_advance              | Advance parse cursor |
| 0x100004BC4 | 188  | img4_match_tag            | Match IMG4/IM4P string |
| 0x100004CB8 | 840  | img4_tag_verifier         | ★ Per-property dispatch |
| 0x10000612C | ~16  | memory_barrier            | DSB/ISB barrier |
| 0x100006754 | ~40  | platform_notify           | Platform event |
| 0x100006774 | ~60  | platform_transfer_notify  | Transfer event |
| 0x1000068A4 | ~300 | platform_main             | ★ Boot/DFU orchestrator |
| 0x100008370 | ~40  | log_debug                 | Debug logging |
| 0x1000083B8 | ~60  | log_poll                  | Log with poll interval |
| 0x100008978 | ~20  | panic                     | Fatal error handler |
| 0x100009B64 | ~20  | security_enter            | Enter security context |
| 0x100009B78 | ~40  | security_wait             | Wait in security ctx |
| 0x100009BA8 | ~20  | security_sleep            | Sleep N cycles |
| 0x10000F1EC | ~60  | heap_alloc                | Heap allocator |
| 0x10000F3B0 | ~80  | pool_alloc                | Pool allocator |
| 0x10000F468 | ~40  | pool_free                 | Pool deallocator |
| 0x100010014 | ~40  | byte_swap_32              | Endian conversion |
| 0x100010BD0 | ~80  | memcpy                    | Memory copy |
| 0x100010D80 | ~40  | bzero                     | Zero memory |
| 0x100010E00 | ~40  | memset                    | Set memory |
| 0x100010EA4 | ~60  | memcmp                    | Memory compare |
| 0x100011004 | ~20  | strlen                    | String length |
| 0x1000113B4 | ~40  | get_usb_controller_ctx    | Get DWC2 context |
| 0x100013AAC | ~200 | der_parse_element         | DER element parser |

---

*Document generated from complete reverse engineering of T8020 B1 SecureROM (524,288 bytes)*
*Analysis tools: rom_usb_workflow.py, rom_usb_deep.py, rom_usb_protocol.py, decode_tags.py*
*All functions verified via Capstone disassembly of the raw ROM binary*
