# A12 T8020 B1 — Double-Abort Deep Analysis

## Date: Session 26 — ROM Binary Disassembly

---

## 1. CRITICAL FINDING: TWO SEPARATE ABORT PATHS

The A12 SecureROM has TWO distinct EP abort code paths, and they behave DIFFERENTLY:

### Path A: dwc3_usb_reset (0x10000C448) — USB BUS RESET handler
```
C448: tbz w23, #0xc, C51C
C44C: mov w0, #2
C450: bl  DF78            ; usb_state_dispatch(USB_RESET)
C454-C478: zero DWC3 registers

C480: mov w22, #5         ; start at EP5
C484: b   C49C
C488:   orr w0, w22, #0x80
C48C:   bl  E158          ; ep_abort_wrapper(EP_IN)
C490:   mov x0, x22
C494:   bl  E158          ; ep_abort_wrapper(EP_OUT)
C498:   mov x22, x19
C49C:   subs w19, w22, #1
C4A0:   b.ge C488         ; loop EP5→EP4→...→EP0

C4A4: add x0, x21, #0x60
C4A8: mov w1, #0x3C0
C4AC: bl  bzero           ; ★★★ WIPE ALL EP STATES ★★★

C4B0-C510: reinit EP0 (ep_busy=1, program SETUP TRB)
```

**NO extra EP0_IN abort after loop!** Goes directly to bzero.

### Path B: dwc3_core_stop (0x10000BB88) — DFU EXIT quiesce path
```
BB88: ... setup ...
BBD8: mov w19, #5         ; start at EP5
BBDC: b   BBF4
BBE0:   orr w0, w19, #0x80
BBE4:   bl  E158          ; ep_abort_wrapper(EP_IN)
BBE8:   mov x0, x19
BBEC:   bl  E158          ; ep_abort_wrapper(EP_OUT)
BBF0:   mov x19, x22
BBF4:   subs w22, w19, #1
BBF8:   b.ge BBE0         ; loop EP5→EP4→...→EP0

BBFC: mov w0, #0x80
BC00: bl  E158            ; ★★★ EXTRA EP0_IN ABORT ★★★

BC04-BC44: zero DWC3 registers
```

**HAS extra EP0_IN abort at BC00!** This is the double-abort.

### Path C: ep0_handler SETUP processing (0x10000D94C)
```
D974: ldr x8, [x8]          ; load SETUP data
D978-D97C: store SETUP data
D980: mov w0, #0x80
D984: bl  D2D0               ; ★ hal_abort_call(EP0_IN) — abort per SETUP!
D988: ... continue processing SETUP ...
```

**Every SETUP triggers a single EP0_IN abort** via `hal_abort_call`.

---

## 2. THE DOUBLE-ABORT ONLY EXISTS IN dwc3_core_stop (PATH B)

| Code Path | When Called | EP0_IN Aborts | Double-Abort? |
|-----------|-----------|---------------|---------------|
| dwc3_usb_reset (C448) | USB bus reset from host | 1 (in loop) | **NO** |
| dwc3_core_stop (BB88) | DFU exit → quiesce → teardown | 2 (loop + extra at BC00) | **YES** |
| ep0_handler (D94C) | Each SETUP packet | 1 (at D984) | **NO** |

**The internet research (alfiecg.uk) describes the double-abort as blocking the ZLP leak.**
**But the double-abort is ONLY in `dwc3_core_stop`, NOT in `dwc3_usb_reset`!**

---

## 3. ZLP ALLOCATION DURING ABORT — DETAILED TRACE

### dwc3_ep_abort (C084) — How it processes queues:
```
C1C0-C1C4: compute ep_state pointer
C1C8: str wzr, [x8, #0x78]          ; clear field
C1CC: str xzr, [x8, #0x70]          ; clear field
C1D0: ldr x20, [x8, #0x98]          ; SAVE completed_head
C1D4: stp xzr, xzr, [x8, #0x98]    ; CLEAR completed head+tail
C1D8: ldr x19, [x8, #0x88]          ; SAVE pending_head
C1DC: stp xzr, xzr, [x8, #0x88]    ; CLEAR pending head+tail
C1E0: bl  lock_release

; Process COMPLETED queue:
C1E4: b   C1F8
C1E8:   ldr x21, [x20, #0x28]       ; next = io_req->next
C1EC:   mov x0, x20
C1F0:   bl  E0A8                     ; callback_and_free(io_req)
C1F4:   mov x20, x21
C1F8:   cbnz x20, C1E8              ; loop completed

; Process PENDING queue (with abort flag):
C200: b   C218
C204:   ldr x21, [x19, #0x28]       ; next = io_req->next
C208:   str w20, [x19, #0x10]       ; set abort flag = 1
C20C:   mov x0, x19
C210:   bl  E0A8                     ; callback_and_free(io_req)
C214:   mov x19, x21
C218:   cbnz x19, C204              ; loop pending
C21C: ... ret
```

**CRITICAL**: Queue heads are SAVED then CLEARED (C1D0-C1DC) BEFORE iterating.
When a callback allocates a ZLP → start_transfer → queue to pending:
- The pending queue was just cleared → ZLP becomes the NEW head
- The abort loop uses the SAVED pointers → doesn't see the new ZLP
- **The ZLP survives the abort!**

### zlp_send (D334) — ZLP callback is NULL:
```
D33C: mov w1, #0          ; size = 0
D340: mov x0, #0          ; buffer = NULL
D344: mov x2, #0          ; callback = NULL ★
D348: bl  D368             ; ep0_transfer_setup(NULL, 0, NULL)
; ... then start_transfer
```

ZLP io_request has callback=NULL → when aborted, just free'd (no recursive alloc).

### standard_device_request_cb (D3D0) — The trigger:
```
D3D0: ldr w8, [x0, #0x14]    ; io_length
D3D4: cbz w8, D3F8            ; if 0, return
D3D8: and w9, w8, #0x3f       ; check mod 64
D3DC: cbnz w9, D3F8           ; if not multiple of 0x40, return
D3E0-D3E8: load wLength from SETUP packet
D3EC: cmp w9, w8              ; wLength > io_length ?
D3F0: b.ls D3F8               ; if not, return
D3F4: b   D334                ; → zlp_send! ★
D3F8: ret
```

Conditions: `io_length > 0 && io_length % 64 == 0 && wLength > io_length`

---

## 4. ZLP LEAK ANALYSIS PER CODE PATH

### 4A. USB RESET path (dwc3_usb_reset at C448):

```
STEP 1: C480-C4A0 abort loop processes EP5→EP0
  At EP0_IN: dwc3_ep_abort(0x80)
    - Saves+clears pending/completed
    - Iterates pending: for each io_req → callback → some call zlp_send
    - ZLPs queued to NOW-EMPTY pending queue (new entries)
    - Old io_reqs freed
    → After abort: EP0_IN pending has fresh ZLP(s)

STEP 2: Loop continues to EP0_OUT (nothing there)

STEP 3: C4A4: bzero(DWC3_STATE+0x60, 0x3C0)
  → EP0_IN pending head/tail ZEROED
  → ZLP io_request(s) are ORPHANED on heap
  → NEVER freed

STEP 4: C4D8: ep_busy = 1 (restored)
  → EP0 reinited, ready for new SETUPs

★★★ RESULT: ZLP io_request(s) are LEAKED! ★★★
```

**On the USB RESET path, the double-abort does NOT exist, and the bzero
at C4A4 orphans any ZLPs created during the abort callbacks.**

### 4B. DFU EXIT path (dwc3_core_stop at BB88):

```
STEP 1: BBE0-BBF8 abort loop processes EP5→EP0
  Same as above: ZLPs created by callbacks, queued to EP0_IN

STEP 2: BBFC-BC00: ep_abort_wrapper(0x80) — EXTRA EP0_IN abort!
  - Finds ZLPs in pending queue
  - ZLP callback = NULL → skip callback → just free
  → ZLPs properly freed ✓

STEP 3: BC04+: zero DWC3 registers

★★★ RESULT: ZLPs are CAUGHT and FREED by the extra abort ★★★
```

**On the DFU EXIT path, the extra abort at BC00 catches all ZLPs.**

### 4C. Per-SETUP abort (ep0_handler at D94C):

```
SETUP N arrives:
  D984: dwc3_ep_abort(EP0_IN)
    - Drains pending queue from previous SETUP:
      - io_reqN-1 → callback → ZLP_N-1 → queued to pending
      - ZLP_N-2 (from two SETUPs ago) → callback=NULL → freed ✓
    - After abort: pending has ZLP_N-1
  Continue: process SETUP N → create io_reqN → queue to pending
  Now: pending = ZLP_N-1 + io_reqN

SETUP N+1 arrives:
  D984: abort EP0_IN
    - ZLP_N-1 freed ✓ (callback=NULL)
    - io_reqN freed (callback → ZLP_N → queued)
    - After abort: pending has ZLP_N
```

**Each SETUP catches the PREVIOUS SETUP's ZLP but creates a new one.
There is always ONE surviving ZLP at any time. This is the "chain" of
aborts described by alfiecg.uk as "for each setup packet".**

---

## 5. THE EP0_HANDLER ABORT MECHANISM

The alfiecg.uk quote now makes more sense:
> "the abort that is subsequently triggered also aborts EP0_IN for each setup packet"

This refers to the ep0_handler at D984, which aborts EP0_IN on each SETUP.
The "abort called twice" counts:
1. First abort: processes the PENDING io_request → callback fires → ZLP allocated
2. Second abort: the NEXT SETUP's abort at D984 catches the ZLP → freed

This is NOT the same as the extra EP0_IN abort in dwc3_core_stop (BC00).
It's a SEQUENTIAL pattern: each SETUP's abort catches the previous one's ZLP.

**BUT: The LAST ZLP (from the final io_request before USB RESET) is NOT caught!**

---

## 6. ZLP SURVIVAL DURING USB RESET

When the host triggers USB RESET after sending N SETUPs:

```
Time →
SETUP 1 → abort(empty) → io_req1 queued → pending=[io_req1]
SETUP 2 → abort([io_req1]) → ZLP1 created → io_req2 queued → pending=[ZLP1, io_req2]
SETUP 3 → abort([ZLP1, io_req2]) → ZLP1 freed ✓, ZLP2 created → pending=[ZLP2, io_req3]
...
SETUP N → abort → ZLPN-2 freed, ZLPN-1 created → pending=[ZLPN-1, io_reqN]

USB RESET:
  C448: dwc3_usb_reset
  C480 loop EP0_IN abort:
    - ZLPN-1 freed (callback=NULL) ✓
    - io_reqN → callback → ZLPN created → queued to pending
    → After abort: pending has ZLPN
  C4A4: bzero → pending pointer zeroed
  → ZLPN ORPHANED ON HEAP → ★★★ LEAKED ★★★
```

**One ZLP (0x30 bytes heap, bucket 2) leaks per USB RESET.**

---

## 7. LEAK SIZE AND HEAP IMPACT

### ZLP io_request:
- calloc(1, 0x30) at D390 → user size = 0x30
- With 0x40 chunk header → total chunk = 0x80 bytes
- Bucket = 32 - clz(0x80/64) = 32 - clz(2) = **bucket 2**

### io_buffer (gDFU+0x28):
- memalign(0x40, 0x800) → user size = 0x800
- Bucket = 32 - clz(0x800/64) = 32 - clz(32) = **bucket 6**

### Assessment:
- Leaked ZLP: bucket 2 (0x80-byte chunks)
- Target io_buffer: bucket 6 (0x800-byte chunks)
- **Different buckets → no direct freelist interference**
- Leaked chunks consume contiguous heap space but can't directly
  force io_buffer to a different address

### Can we leak enough to matter?
- Each USB RESET leaks 0x80 bytes (chunk with header)
- Heap is in SRAM: ~64KB usable (not 64MB — SRAM is 0x19C000000-0x1A0000000 
  but active heap zone is much smaller)
- With N USB RESETs, we leak N × 0x80 bytes
- If heap zone is ~32KB, we'd need ~400 USB RESETs to exhaust it
- Each USB RESET takes ~ms → 400 × few ms = sub-second!

**THIS MAY BE FEASIBLE.**

---

## 8. EXPLOITATION THEORY

### 8A. Heap Exhaustion via ZLP Leak

If we leak enough ZLP io_requests (each 0x80 bytes in bucket 2), 
the bucket-2 free list eventually empties. The allocator then splits 
chunks from larger buckets to satisfy bucket-2 requests. This cascades 
up through the bucket hierarchy until it affects bucket 6 (where 
io_buffer lives).

Eventually, the heap runs out of space or the allocation pattern 
changes, causing memalign(0x800) to return a different address.

### 8B. Target: Make io_buffer land at a different address

The UAF works when:
1. USB_STATE+0x28 = address A (set during DNLOAD SETUP)
2. free(A) — buffer freed during DFU exit
3. new io_buffer = address B ≠ A (re-entry allocates at different address)
4. Write to A overwrites freed/reallocated memory → UAF exploitable

Currently: LIFO heap returns same address (B == A) → UAF useless.
If ZLP leaks perturb the heap enough → B ≠ A → UAF WORKS!

### 8C. Practical Exploit Flow

```
PHASE 0 — Heap Spray (ZLP leak accumulation):
  for i in 1..N:
    Enter DFU (or stay in DFU across USB RESETs)
    Send SETUP with wLength > io_length, io_length % 64 == 0
    Trigger USB RESET
    → 1 ZLP leaked per iteration

PHASE 1 — Trigger UAF:  
  Send DFU_DNLOAD with data → USB_STATE+0x28 = io_buffer = A
  Send DFU_ABORT → dfuDone=1 → dfu_run exits
  dfu_run → usb_quiesce_and_reinit (A578)
    → dwc3_core_stop (BB88) — with double-abort (ZLP properly freed here)
    → free(A) via E72C
  → USB_STATE+0x28 = A (dangling!)

PHASE 2 — Re-entry with heap perturbation:
  DFU re-enters → dfu_init → memalign(0x800)
  If enough ZLPs leaked → bucket 6 fragmented → returns B ≠ A!
  
PHASE 3 — Exploit:
  Send forged DFU_ABORT with wLength > 0
  → DATA phase programmed → write to USB_STATE+0x28 = A (freed!)
  → Attacker writes to freed memory at A
  → A might now contain heap metadata or other structures
  → Heap metadata corruption → code execution
```

---

## 9. PROBLEMS AND OPEN QUESTIONS

### 9A. Does the leak actually work?

The ZLP is allocated during the abort at C1E8-C1F8 (pending queue drain).
The callback is called via blr x8 at E0C4. We need to verify:
1. Is D3D0 actually the callback for EP0 IN io_requests? 
2. What io_length values are available? (must be > 0, multiple of 0x40)
3. Can we control wLength to be > io_length?

For standard USB descriptors:
- GET_DESCRIPTOR(Device): 18 bytes → NOT multiple of 0x40 → NO ZLP
- GET_DESCRIPTOR(Config): varies → might be multiple of 0x40
- Vendor requests: custom → might be controllable

### 9B. How many ZLPs per USB RESET?

During the USB RESET abort of EP0_IN, multiple io_requests may be pending.
But on DWC3, the per-SETUP abort at D984 limits accumulation. Typically 
only ONE io_request is pending when USB RESET fires.

→ Only 1 ZLP leaked per USB RESET (from the one pending io_request).

### 9C. Heap behavior after many leaks

After N leaks in bucket 2:
- Does the allocator eventually split from bucket 6?
- How does the LIFO return behavior change?
- Need empirical testing on actual device.

### 9D. Data zeroing after free

Even if we get B ≠ A, free() calls bzero on user data at F560.
Address A's contents are zeroed when freed. We'd need to fill it 
with controlled data AFTER the free and BEFORE the re-alloc.

### 9E. Safe unlink in heap

Any freelist corruption attempt must pass safe unlink checks:
- FE60: chunk.prev->next == chunk
- FE70: chunk.next->prev == chunk

---

## 10. NEXT STEPS

### P0: Verify callback registration
Confirm that D3D0 is stored at io_req+0x20 for EP0 IN io_requests.
Trace the ADRP/ADD that generates the D3D0 address and the STR that
stores it at +0x20.

### P1: Find descriptors with io_length % 64 == 0
Check all standard/class/vendor USB descriptors served by the DFU device.
Which ones have a response length that is a multiple of 64?
If wLength is set larger → ZLP condition met.

### P2: Measure actual heap zone size
Determine the size of the active heap zone in T8020 B1 SecureROM.
This tells us how many USB RESETs are needed. Zone address from 
chunk_header+0x18 validation: zones at 0x19C011E88 or 0x19C011468.

### P3: Test ZLP leak on actual device
Connect iPhone XR to Facedancer/custom USB host.
Send SETUPs with appropriate wLength, trigger USB RESETs.
Monitor device behavior (timing, enumeration speed changes).
After N resets, trigger DFU_ABORT + re-entry and observe if crash
changes (indicating different heap layout).

### P4: Explore STALL-based accumulation
On DWC3, if we can STALL EP0_IN without going through ep0_handler
(e.g., via dwc3_ep_stall HAL[0x40] triggered by some request), 
we might accumulate multiple io_requests without the per-SETUP abort,
leading to MULTIPLE ZLP leaks per USB RESET.

---

## 11. SUMMARY

| Finding | Status |
|---------|--------|
| Double-abort exists in dwc3_core_stop (BB88:BC00) | ✅ CONFIRMED |
| Double-abort does NOT exist in dwc3_usb_reset (C448) | ✅ CONFIRMED |
| EP0_IN abort per SETUP (D984) creates a chain | ✅ CONFIRMED |
| One ZLP leaks per USB RESET | ✅ CONFIRMED (theoretical) |
| ZLP size = 0x30 user / 0x80 chunk (bucket 2) | ✅ CONFIRMED |
| Multiple USB RESETs could exhaust heap | ⚠️ THEORETICAL |
| Heap perturbation could break LIFO | ⚠️ THEORETICAL |
| Practical exploitation via accumulated leaks | ❓ REQUIRES TESTING |

~~**The ZLP memory leak on A12 is NOT fully patched on the USB RESET path.**~~
~~**The protection (double-abort) only exists in the DFU EXIT path (dwc3_core_stop).**~~
~~**This is a POTENTIAL avenue for exploiting the checkm8 UAF on A12.**~~

**UPDATE: SECTIONS 12-15 BELOW DISPROVE THIS. The ZLP leak is IMPOSSIBLE because
Layers 1+2 prevent ZLP creation entirely. The missing double-abort is IRRELEVANT.**

---

## 12. DEEP CALLBACK TRACING — THE REAL PICTURE (Session 28)

### All callers of D3D0 (standard_device_request_cb):

The D3D0 callback is loaded at exactly **3 sites** in the ep0_handler:

| Address | Request             | Response Size        | mod64 | ZLP? |
|---------|---------------------|----------------------|-------|------|
| DBCC    | GET_STATUS          | 2 bytes              | 2     | NO   |
| DC7C    | GET_DESCRIPTOR      | min(wLength, descLen)| var   | var  |
| DD74    | GET_CONFIGURATION   | 1 byte               | 1     | NO   |

**Only GET_DESCRIPTOR could theoretically trigger ZLP**, if any descriptor
has bLength % 64 == 0.

### DFU responses use callback=NULL (LAYER 1):

DFU interface requests (DNLOAD, GETSTATUS, GETSTATE, CLRSTATUS, ABORT) are
dispatched through `usb_core_do_transfer` at E5DC:

```
E5C8: adrp x8, #0x19c010000
E5CC: add  x8, x8, #0xbe0        ; gDFU
E5D0: ldr  x1, [x8, #0x28]       ; io_buffer
E5D4: mov  w0, #0x80              ; EP0_IN
E5D8: mov  x3, #0                 ; ★★★ callback = NULL ★★★
E5DC: bl   E0D8                   ; usb_core_do_transfer
```

When aborted, `dwc3_callback_and_free` (E0A8) checks:
```
E0B8: ldr x8, [x19, #0x20]       ; load callback
E0BC: cbz x8, E0C8               ; ★ NULL → skip callback → just free
```

**ALL DFU io_requests have callback=NULL → D3D0 is NEVER reached for DFU.**
**No ZLP can EVER be created during DFU request abort.**

### Direct zlp_send callers:

`zlp_send` (D334) is called directly from exactly 2 sites:
- **DAD8**: After DATA OUT completion when last chunk = 64 bytes (status ZLP, cb=NULL)
- **DDFC**: After interface handler returns 0 (status ZLP, cb=NULL)

Both create ZLPs with callback=NULL. When aborted → freed immediately, no recursion.

---

## 13. DESCRIPTOR SIZE ANALYSIS — ALL REAL DESCRIPTORS (Session 28)

### Descriptor dispatch (DCDC jump table at DF5C):

| Type | Handler | Response Size | mod64 |
|------|---------|---------------|-------|
| 1 (DEVICE)           | DCE8 | min(wLength, 18)  | 18 |
| 2 (CONFIGURATION)    | DE1C | min(wLength, ~27)  | ~27 |
| 3 (STRING)           | DE30 | min(wLength, bLen) | var |
| 6 (DEVICE_QUALIFIER) | DE50 | min(wLength, 10)   | 10 |
| 7 (OTHER_SPEED_CFG)  | DE7C | like config        | ~27 |

### String descriptor handler (DE30):

```
DE30: ubfx w10, w8, #0x10, #8      ; string index
DE34: adrp x8, #0x19c010000
DE38: add  x8, x8, #0xb20          ; USB_STATE
DE3C: cbz  w10, DEB8               ; index 0 → ROM constant
DE40: cmp  w10, #1
DE44: b.ne DEC4                    ; index 1 → [USB_STATE+0x30]
      ...
DEB8: adr  x1, #0x10001FCE8        ; string 0 = Language ID (ROM)
      ...
DEC4: cmp  w10, #9                 ; max index = 9
DEC8: b.hi DF00                    ; out of range → error
DECC: add  x10, x8, w10, uxtw #3  ; pointer table
DED0: add  x10, x10, #0x70        ; [USB_STATE + 0x70 + idx*8]
DED4: ldr  x1, [x10]              ; load descriptor pointer
DED8: cbz  x1, DF00               ; NULL check
DEDC: ldrb w10, [x1]              ; bLength
DEE0: cmp  w9, w10                ; min(wLength, bLength)
```

### Actual DFU mode string descriptors (verified from ROM):

| Index | Content                            | Chars | bLength | mod64 | ZLP? |
|-------|---------------------------------------|-------|---------|-------|------|
| 0     | Language ID (0x0409)                  | —     | **4**   | 4     | NO   |
| 1     | "Apple Inc."                          | 10    | **22**  | 22    | NO   |
| 2     | "Apple Mobile Device (DFU Mode)"      | 30    | **62**  | 62    | NO   |
| 3     | "CPID:8020 CPRV:11 ... SRTG:[...]"   | 97    | **196** | 4     | NO   |

String 0: ROM constant at 0x1FCE8 = `{04 03 09 04}` → bLength=4
Strings 1-3: Dynamically built in SRAM from ASCII format strings:
- "Apple Inc." at ROM 0x1C334
- "Apple Mobile Device (DFU Mode)" at ROM 0x1C25A
- Serial format: ROM 0x1C279 + " SRTG:[%s]" at ROM 0x1C2C2

**NONE have bLength % 64 == 0. The D3D0 alignment check BLOCKS all of them.**

### Sensitivity analysis:

The serial string includes `SRTG:[iBoot-X.Y.Z.W.V]`. The iBoot version
string length is fixed per ROM build. For T8020 B1:
- SRTG = "iBoot-3865.0.0.4.7" = 18 chars → serial bLength = 196, mod64 = 4

A hypothetical ROM with SRTG of exactly 16 chars would give bLength=192 (mod64=0).
But no known A12 ROM has this value. And Apple controls this string at build time.

---

## 14. THE THREE LAYERS OF ZLP LEAK PREVENTION

```
┌─────────────────────────────────────────────────────────┐
│ LAYER 1: DFU callback = NULL                            │
│   E5D8: mov x3, #0                                     │
│   All DFU io_requests have NULL callback.               │
│   On abort: E0BC cbz → skip D3D0 → just free.          │
│   ★ BLOCKS ALL DFU-TRIGGERED ZLP CREATION               │
├─────────────────────────────────────────────────────────┤
│ LAYER 2: D3D0 alignment check                           │
│   D3D8: and w9, w8, #0x3f (io_length % 64)             │
│   D3DC: cbnz w9, ret (if not aligned → return)          │
│   No standard or string descriptor has bLength%64==0.   │
│   ★ BLOCKS ALL STANDARD-REQUEST ZLP CREATION             │
├─────────────────────────────────────────────────────────┤
│ LAYER 3: Double-abort in dwc3_core_stop                 │
│   BBFC: mov w0, #0x80; bl E158                          │
│   Extra EP0_IN abort catches any surviving ZLPs.        │
│   ★ ONLY EXISTS IN DFU EXIT PATH (NOT USB RESET)        │
│   ★ REDUNDANT: Layers 1+2 already prevent creation      │
└─────────────────────────────────────────────────────────┘
```

**Layer 3 (the double-abort) is a DEFENSE IN DEPTH measure.
Its absence on the USB RESET path (C448) does NOT matter because
Layers 1+2 prevent any ZLP from being created in the first place.**

---

## 15. DEFINITIVE VERDICT

### Can the checkm8 UAF ZLP leak work on A12 T8020 B1?

**NO.**

The ZLP memory leak requires D3D0 callback to trigger `zlp_send`.
D3D0 triggers ONLY when: `io_length > 0 && io_length % 64 == 0 && wLength > io_length`

- **DFU requests**: callback=NULL → D3D0 never called → NO ZLP
- **GET_STATUS**: io_length=2 → 2%64≠0 → NO ZLP
- **GET_CONFIGURATION**: io_length=1 → 1%64≠0 → NO ZLP
- **GET_DESCRIPTOR(Device)**: io_length=18 → 18%64≠0 → NO ZLP
- **GET_DESCRIPTOR(DevQual)**: io_length=10 → 10%64≠0 → NO ZLP
- **GET_DESCRIPTOR(Config)**: io_length≈27 → 27%64≠0 → NO ZLP
- **GET_DESCRIPTOR(String 0)**: io_length=4 → 4%64≠0 → NO ZLP
- **GET_DESCRIPTOR(String 1)**: io_length=22 → 22%64≠0 → NO ZLP
- **GET_DESCRIPTOR(String 2)**: io_length=62 → 62%64≠0 → NO ZLP
- **GET_DESCRIPTOR(String 3)**: io_length=196 → 196%64≠0 → NO ZLP
- **DFU_UPLOAD**: NOT IMPLEMENTED in this ROM

**EVERY POSSIBLE USB RESPONSE IS BLOCKED BY LAYERS 1 OR 2.**

### Three layers, zero paths through:

| Attack Vector | Layer 1 (NULL cb) | Layer 2 (mod64) | Layer 3 (dbl-abort) | Result |
|---------------|:-:|:-:|:-:|--------|
| DFU GETSTATUS abort | ✅ BLOCKED | — | — | BLOCKED |
| DFU GETSTATE abort  | ✅ BLOCKED | — | — | BLOCKED |
| DFU DNLOAD abort    | ✅ BLOCKED | — | — | BLOCKED |
| GET_DESCRIPTOR(dev) | — | ✅ BLOCKED (18) | — | BLOCKED |
| GET_DESCRIPTOR(str0)| — | ✅ BLOCKED (4) | — | BLOCKED |
| GET_DESCRIPTOR(str1)| — | ✅ BLOCKED (22) | — | BLOCKED |
| GET_DESCRIPTOR(str2)| — | ✅ BLOCKED (62) | — | BLOCKED |
| GET_DESCRIPTOR(str3)| — | ✅ BLOCKED (196) | — | BLOCKED |
| GET_DESCRIPTOR(cfg) | — | ✅ BLOCKED (~27) | — | BLOCKED |
| USB RESET after all | — | — | ❌ MISSING | **Irrelevant** |

### What WOULD it take?

For a SOFTWARE exploit of the A12 checkm8 UAF, you would need:
1. A way to make the ROM serve a USB descriptor with bLength % 64 == 0, OR
2. A way to make DFU use a non-NULL callback, OR
3. Finding a completely different ZLP allocation path not involving D3D0

None of these exist in the T8020 B1 ROM binary. The only remaining
attack surface is **hardware fault injection** (EMFI, voltage glitching)
to corrupt control flow or memory at the physical level.

### Final Classification:

```
A12 T8020 B1 SecureROM — checkm8 UAF Status
├── UAF Bug:           PRESENT (same DWC3 controller as A11)
├── ZLP Memory Leak:   BLOCKED by 3 independent layers
│   ├── Layer 1:       DFU callback = NULL
│   ├── Layer 2:       Alignment check in D3D0 (no descriptor fits)
│   └── Layer 3:       Double-abort in DFU exit (defense in depth)
├── Heap Feng Shui:    IMPOSSIBLE (no leak primitive)
├── Software Exploit:  NOT FEASIBLE
└── Hardware Exploit:  POSSIBLE (EMFI/glitch — not ROM-level)
```
