//! TraceEvent — fixed-size record produced on the kernel hot path.
//!
//! Layout is `#[repr(C)]` and 8-byte aligned so a write fits in 6 cache lines
//! and can be memcpy'd by the drainer without parsing.

#[derive(Clone, Copy, Debug)]
#[repr(u16)]
pub enum TraceKind {
    Invalid       = 0,

    // Memory
    Alloc         = 1,
    Free          = 2,
    PageFault     = 3,

    // Scheduler
    SchedSwitch   = 10,
    SchedEnqueue  = 11,
    SchedWakeup   = 12,

    // IRQ
    IrqEntry      = 20,
    IrqExit       = 21,

    // Locks
    LockAcquire   = 30,
    LockRelease   = 31,

    // I/O
    BlockSubmit   = 40,
    BlockComplete = 41,
    NetRx         = 42,
    NetTx         = 43,

    // PMU snapshot
    PmuSample     = 50,

    // GPU
    GpuSubmit     = 60,
    GpuFence      = 61,

    // Marker / heartbeat
    Marker        = 100,
}

impl TraceKind {
    #[inline]
    pub fn from_u16(v: u16) -> TraceKind {
        match v {
            1   => TraceKind::Alloc,
            2   => TraceKind::Free,
            3   => TraceKind::PageFault,
            10  => TraceKind::SchedSwitch,
            11  => TraceKind::SchedEnqueue,
            12  => TraceKind::SchedWakeup,
            20  => TraceKind::IrqEntry,
            21  => TraceKind::IrqExit,
            30  => TraceKind::LockAcquire,
            31  => TraceKind::LockRelease,
            40  => TraceKind::BlockSubmit,
            41  => TraceKind::BlockComplete,
            42  => TraceKind::NetRx,
            43  => TraceKind::NetTx,
            50  => TraceKind::PmuSample,
            60  => TraceKind::GpuSubmit,
            61  => TraceKind::GpuFence,
            100 => TraceKind::Marker,
            _   => TraceKind::Invalid,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TraceKind::Invalid       => "invalid",
            TraceKind::Alloc         => "alloc",
            TraceKind::Free          => "free",
            TraceKind::PageFault     => "pgfault",
            TraceKind::SchedSwitch   => "sched_switch",
            TraceKind::SchedEnqueue  => "sched_enq",
            TraceKind::SchedWakeup   => "sched_wake",
            TraceKind::IrqEntry      => "irq_in",
            TraceKind::IrqExit       => "irq_out",
            TraceKind::LockAcquire   => "lock_acq",
            TraceKind::LockRelease   => "lock_rel",
            TraceKind::BlockSubmit   => "blk_sub",
            TraceKind::BlockComplete => "blk_done",
            TraceKind::NetRx         => "net_rx",
            TraceKind::NetTx         => "net_tx",
            TraceKind::PmuSample     => "pmu",
            TraceKind::GpuSubmit     => "gpu_sub",
            TraceKind::GpuFence      => "gpu_fence",
            TraceKind::Marker        => "marker",
        }
    }
}

/// 48-byte event record. `data` semantics depend on `kind`:
///
/// | kind            | data[0]  | data[1]  | data[2]  | data[3]  |
/// |-----------------|----------|----------|----------|----------|
/// | Alloc / Free    | size     | class    | ptr      | latency  |
/// | SchedSwitch     | from_tid | to_tid   | reason   | runtime  |
/// | IrqEntry/Exit   | vector   | latency  | _        | _        |
/// | LockAcquire     | lock_id  | wait_cyc | contend  | _        |
/// | PmuSample       | cycles   | instr    | l1_miss  | br_miss  |
/// | Marker          | tag0     | tag1     | tag2     | tag3     |
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TraceEvent {
    pub tsc:   u64,
    pub kind:  u16,
    pub cpu:   u8,
    pub flags: u8,
    pub _pad:  u32,
    pub data:  [u64; 4],
}

impl TraceEvent {
    pub const ZERO: TraceEvent = TraceEvent {
        tsc: 0, kind: 0, cpu: 0, flags: 0, _pad: 0, data: [0; 4],
    };
}

/// Flags bit layout
pub mod flag {
    pub const IRQ_CTX:        u8 = 1 << 0;
    pub const PREEMPT_OFF:    u8 = 1 << 1;
    pub const SAMPLED:        u8 = 1 << 2;  // produced via sampler, not full trace
}

pub const EVENT_SIZE: usize = core::mem::size_of::<TraceEvent>();
const _: () = assert!(EVENT_SIZE == 48, "TraceEvent must be exactly 48 bytes");
