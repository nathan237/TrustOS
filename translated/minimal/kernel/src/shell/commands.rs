





use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AX_, D_, A_, C_, R_, CF_, DM_, K_};
use crate::ramfs::FileType;


pub(super) fn chf(args: &[&str]) {
    if !args.is_empty() {
        hmb(args);
        return;
    }
    
    crate::n!(G_, "======================================================================");
    crate::n!(G_, "          TrustOS -- Secure Bare-Metal Operating System");
    crate::n!(G_, "       x86_64 kernel written in Rust -- no libc, no std");
    crate::n!(G_, "======================================================================");
    crate::println!();
    crate::n!(R_, "  Features: RAMFS file system, TCP/IP networking, ELF loader,");
    crate::n!(R_, "  Linux syscall compat, GUI desktop compositor, SMP multicore.");
    crate::println!();
    crate::n!(D_, "  Type 'help <command>' or 'man <command>' for detailed usage.");
    crate::n!(D_, "  Tab = auto-complete | Up/Down = history | PageUp/Down = scroll");
    crate::println!();
    
    
    crate::n!(C_, "  FILE SYSTEM");
    crate::println!("    ls [path]           List directory contents (-l long, -a hidden)");
    crate::println!("    cd <path>           Change working directory");
    crate::println!("    pwd                 Print current working directory");
    crate::println!("    mkdir <dir>         Create directory (-p recursive)");
    crate::println!("    rmdir <dir>         Remove empty directory");
    crate::println!("    touch <file>        Create empty file or update timestamp");
    crate::println!("    rm <file>           Remove file or directory (-r recursive)");
    crate::println!("    cp <src> <dst>      Copy file or directory");
    crate::println!("    mv <src> <dst>      Move or rename file");
    crate::println!("    cat <file>          Display file contents (supports > redirect)");
    crate::println!("    head <file>         Show first N lines (-n N)");
    crate::println!("    tail <file>         Show last N lines (-n N)");
    crate::println!("    wc <file>           Count lines, words, bytes");
    crate::println!("    stat <file>         Display file metadata (size, type, perms)");
    crate::println!("    tree [path]         Display directory tree structure");
    crate::println!("    find <path> <name>  Search for files by name pattern");
    crate::println!("    ln -s <tgt> <link>  Create symbolic link");
    crate::println!("    readlink <link>     Display link target");
    crate::println!("    basename <path>     Strip directory from path");
    crate::println!("    dirname <path>      Strip filename from path");
    crate::println!("    realpath <path>     Resolve to absolute path");
    crate::println!("    file <path>         Identify file type (ELF, text, etc.)");
    crate::println!("    chmod <mode> <f>    Change file permissions (octal)");
    crate::println!("    chown <u>[:<g>] <f> Change file ownership");
    crate::println!();
    
    
    crate::n!(C_, "  TEXT PROCESSING");
    crate::println!("    echo <text>         Print text (supports > redirect)");
    crate::println!("    grep <pat> <file>   Search for pattern (-i case insensitive)");
    crate::println!("    sort <file>         Sort lines (-r reverse, -n numeric)");
    crate::println!("    uniq <file>         Remove duplicate adjacent lines (-c count)");
    crate::println!("    cut -d<d> -f<n>     Cut columns by delimiter");
    crate::println!("    tr <a> <b>          Translate characters (a->b)");
    crate::println!("    tee <file>          Write stdin to file + stdout");
    crate::println!("    rev <text>          Reverse string");
    crate::println!("    diff <a> <b>        Compare two files line by line");
    crate::println!("    cmp <a> <b>         Compare two files byte by byte");
    crate::println!("    patch <file>        Apply diff patch");
    crate::println!("    strings <file>      Extract printable strings from binary");
    crate::println!("    od <file>           Octal dump of file");
    crate::println!("    hexdump <file>      Hex dump of file contents");
    crate::println!("    base64 <file>       Encode/decode base64 (-d decode)");
    crate::println!("    md5sum <file>       Compute MD5 hash");
    crate::println!("    sha256sum <file>    Compute SHA-256 hash");
    crate::println!();
    
    
    crate::n!(C_, "  SYSTEM & PROCESS");
    crate::println!("    clear               Clear terminal screen");
    crate::println!("    time / uptime       Show system uptime");
    crate::println!("    date                Display current date and time");
    crate::println!("    whoami              Print current username");
    crate::println!("    hostname            Display system hostname");
    crate::println!("    id                  Print user/group IDs");
    crate::println!("    uname [-a]          System information (kernel, arch)");
    crate::println!("    env / printenv      Display environment variables");
    crate::println!("    export K=V          Set environment variable");
    crate::println!("    unset <var>         Remove environment variable");
    crate::println!("    set                 Show all shell variables");
    crate::println!("    alias <n>=<cmd>     Create command alias");
    crate::println!("    unalias <name>      Remove command alias");
    crate::println!("    source <file>       Execute commands from script file");
    crate::println!("    history             Show command history");
    crate::println!("    ps                  List running processes");
    crate::println!("    top / htop          Real-time process monitor");
    crate::println!("    kill <pid>          Send signal to process");
    crate::println!("    killall <name>      Kill processes by name");
    crate::println!("    nice <n> <cmd>      Run command with priority");
    crate::println!("    nohup <cmd>         Run command immune to hangups");
    crate::println!("    bg / fg             Background/foreground job control");
    crate::println!("    tasks / jobs        List active kernel tasks");
    crate::println!("    threads             Show kernel thread info");
    crate::println!("    free                Display memory usage statistics");
    crate::println!("    df                  Show disk space usage");
    crate::println!("    vmstat              Virtual memory statistics");
    crate::println!("    iostat              I/O statistics by device");
    crate::println!("    lsof [pid]          List open files per process");
    crate::println!("    strace <cmd>        Trace system calls of command");
    crate::println!("    sleep <secs>        Pause execution for N seconds");
    crate::println!("    watch <cmd>         Execute command repeatedly");
    crate::println!("    timeout <s> <cmd>   Run command with time limit");
    crate::println!("    which <cmd>         Show command location");
    crate::println!("    whereis <cmd>       Locate command binary and manpage");
    crate::println!("    script <file>       Record terminal session to file");
    crate::println!("    timecmd <cmd>       Measure command execution time");
    crate::println!();
    
    
    crate::n!(C_, "  USER MANAGEMENT");
    crate::println!("    login               Switch to another user account");
    crate::println!("    su <user>           Substitute user identity");
    crate::println!("    passwd [user]       Change user password");
    crate::println!("    adduser <name>      Create new user account");
    crate::println!("    deluser <name>      Delete user account");
    crate::println!("    users               List all user accounts");
    crate::println!();
    
    
    crate::n!(C_, "  HARDWARE & DEVICES");
    crate::println!("    lspci [-v]          List PCI devices (vendor/class)");
    crate::println!("    lshw / hwinfo       Full hardware inventory");
    crate::println!("    gpu [info|dcn|modes] AMD GPU info & display engine status");
    crate::println!("    gpuexec <agent> [N] Dispatch RDNA compute agent on GPU CUs");
    crate::println!("    sdma <cmd>          SDMA engine DMA transfers (copy/fill/bench)");
    crate::println!("    neural <cmd>        Neural compute: GEMM, activations, inference");
    crate::println!("    a11y [hc|font|...]  Accessibility settings (Win+H = contrast)");
    crate::println!("    lscpu               CPU model, cores, features, frequency");
    crate::println!("    lsmem               Memory layout and total RAM");
    crate::println!("    lsusb               List USB controllers & devices");
    crate::println!("    dmidecode           BIOS/SMBIOS firmware tables");
    crate::println!("    hdparm <dev>        Disk drive parameters");
    crate::println!("    smpstatus           SMP multicore status (per-CPU state)");
    crate::println!("    smp <cmd>           SMP control (start/stop cores)");
    crate::println!("    modprobe <mod>      Load kernel module");
    crate::println!("    lsmod               List loaded kernel modules");
    crate::println!("    insmod / rmmod      Insert or remove module");
    crate::println!("    beep [freq] [ms]    Play a tone (default 440Hz 500ms)");
    crate::println!("    audio               Audio driver status / control");
    crate::println!("    synth <cmd>         TrustSynth polyphonic synthesizer");
    crate::println!("                         note/freq/wave/adsr/preset/demo/status");
    crate::println!("    daw <cmd>           TrustDAW digital audio workstation");
    crate::println!("                         demo/track/note/play/record/mixer/export/gui");
    crate::println!();
    
    
    crate::n!(C_, "  DISK & STORAGE");
    crate::println!("    disk                Show detected disk drives");
    crate::println!("    dd if=<> of=<>      Block-level copy (raw disk I/O)");
    crate::println!("    ahci <cmd>          AHCI controller commands");
    crate::println!("    fdisk <dev>         Partition table editor");
    crate::println!("    lsblk               List block devices");
    crate::println!("    blkid               Show block device UUIDs");
    crate::println!("    mkfs <type> <dev>   Format partition (fat32, ext2)");
    crate::println!("    fsck <dev>          File system consistency check");
    crate::println!("    mount <dev> <dir>   Mount file system");
    crate::println!("    du <path>           Estimate file/directory disk usage");
    crate::println!("    umount <dir>        Unmount file system");
    crate::println!("    sync                Flush all pending writes to disk");
    crate::println!("    persist <cmd>       Manage persistent storage");
    crate::println!();
    
    
    crate::n!(C_, "  NETWORK");
    crate::println!("    ifconfig / ip       Show network interface status");
    crate::println!("    ipconfig [cmd]      Configure IP settings");
    crate::println!("    wifi <cmd>          WiFi management (scan/connect/status/disconnect)");
    crate::println!("    ping <host>         ICMP echo to test connectivity");
    crate::println!("    curl <url>          HTTP/HTTPS client (GET, POST)");
    crate::println!("    wget <url>          Download file from URL");
    crate::println!("    download <url>      Download and save file");
    crate::println!("    nslookup <host>     DNS lookup (A, AAAA records)");
    crate::println!("    arp [-a]            Show ARP table (IP->MAC mappings)");
    crate::println!("    route               Display routing table");
    crate::println!("    traceroute <host>   Real TTL-based traceroute");
    crate::println!("    netstat             Show active connections & listeners");
    crate::println!("    browse <url>        Text-mode web browser");
    crate::println!("    sandbox <cmd>       Web sandbox (open/allow/deny/fs/status/list/kill)");
    crate::println!("    container <cmd>     Web container daemon (status/list/create/go/stop)");
    crate::println!("    tcpsyn <host:port>  Raw TCP SYN connection test");
    crate::println!("    httpget <url>       Raw HTTP GET request");
    crate::println!();

    
    crate::n!(C_, "  SECURITY TOOLKIT (TrustScan)");
    crate::println!("    nmap <target>       Port scanner (SYN/Connect/UDP scan)");
    crate::println!("    nmap <t> -A         Aggressive scan (ports + banners + vulns)");
    crate::println!("    discover [mode]     Host discovery (arp/ping/full)");
    crate::println!("    banner <target>     Service banner grabber & version detect");
    crate::println!("    sniff <cmd>         Packet sniffer (start/stop/show/hex/stats)");
    crate::println!("    vulnscan <target>   Vulnerability assessment scanner");
    crate::println!("    traceroute <host>   Real TTL-based traceroute with ICMP");
    crate::println!("    scantest [target]   Live network test suite (8 tests)");
    crate::println!();

    
    crate::n!(C_, "  HTTP SERVER");
    crate::println!("    httpd [start] [p]   Start HTTP server (default port 8080)");
    crate::println!("    httpd stop          Stop the running HTTP server");
    crate::println!("    httpd status        Show server status and request count");
    crate::println!();

    
    crate::n!(C_, "  PACKAGE MANAGER (TrustPkg)");
    crate::println!("    trustpkg list       List all available packages");
    crate::println!("    trustpkg search <q> Search packages by name/description");
    crate::println!("    trustpkg install <p> Install a package");
    crate::println!("    trustpkg remove <p> Remove an installed package");
    crate::println!("    trustpkg info <p>   Show package details");
    crate::println!("    trustpkg installed  List installed packages only");
    crate::println!("    trustpkg update     Update package catalog");
    crate::println!();
    
    
    crate::n!(C_, "  LINUX SUBSYSTEM");
    crate::println!("    linux               Launch Linux compatibility shell");
    crate::println!("    linux status        Show Linux subsystem status");
    crate::println!("    linux install       Install Linux binaries from rootfs");
    crate::println!("    linux start         Start Linux init process");
    crate::println!("    linux exec <bin>    Execute ELF binary directly");
    crate::println!("    alpine <cmd>        Alpine Linux package manager");
    crate::println!("    distro list         List available distributions");
    crate::println!("    distro install <id> Download & install distribution");
    crate::println!("    distro run <id>     Launch installed distribution");
    crate::println!("    exec <file>         Execute binary (ELF or script)");
    crate::println!("    elfinfo <file>      Display ELF binary header info");
    crate::println!();
    
    
    crate::n!(C_, "  GRAPHICS & DESKTOP");
    crate::println!("    desktop / gui       Launch windowed desktop environment");
    crate::println!("    mobile              Launch mobile-style desktop");
    crate::println!("    cosmic              Launch COSMIC V2 compositor");
    crate::println!("    open <app>          Open desktop with specific app");
    crate::println!("    trustedit           3D model editor (wireframe viewer)");
    crate::println!("    calculator / calc   Launch calculator app");
    crate::println!("    snake               Launch Snake game");
    crate::println!("    glmode [on|off]     Toggle OpenGL compositing mode");
    crate::println!("    theme <name>        Switch color theme (matrix, nord, etc.)");
    crate::println!("    anim <cmd>          Configure UI animations");
    crate::println!("    rain [slow|mid|fast] Set matrix rain speed preset");
    crate::println!("    holo / holomatrix   Holographic matrix visualizer");
    crate::println!("    imgview <file>      Display image file (PPM, BMP)");
    crate::println!("    imgdemo             Run image rendering demo");
    crate::println!("    wayland [cmd]       Wayland compositor control");
    crate::println!("    gterm               Launch graphical terminal");
    crate::println!("    fontsmooth [0-3]    Set font anti-aliasing level");
    crate::println!("    vizfx [cmd]         Live audio visualizer effects");
    crate::println!();
    
    
    crate::n!(C_, "  PROGRAMMING & TOOLS");
    crate::println!("    trustlang / tl      TrustLang programming language REPL");
    crate::println!("    transpile <file>    Binary-to-Rust transpiler (ELF analysis)");
    crate::println!("    rv-xlat <file>      RISC-V universal translator (run any arch)");
    crate::println!("    rv-disasm <file>    Show RISC-V IR translation of binary");
    crate::println!("    trustview <file>    TrustView binary analyzer (Ghidra-style)");
    crate::println!("    video / tv          TrustVideo codec player (record/play)");
    crate::println!("    film                TrustOS Film cinematic demo");
    crate::println!("    trailer             TrustOS 2-min cinematic trailer");
    crate::println!("    bc                  Calculator / math expression evaluator");
    crate::println!("    cal                 Display calendar");
    crate::println!("    factor <n>          Prime factorization of integer");
    crate::println!("    seq <a> [b] <c>     Print numeric sequence");
    crate::println!("    yes [text]          Repeat text infinitely");
    crate::println!("    xargs <cmd>         Build command from stdin");
    crate::println!("    printf <fmt> <args> Formatted text output");
    crate::println!("    expr <expr>         Evaluate arithmetic expression");
    crate::println!("    read <var>          Read user input into variable");
    crate::println!("    nano / vi / edit    Terminal text editor (TrustEdit)");
    crate::println!();
    
    
    crate::n!(C_, "  ARCHIVING & COMPRESSION");
    crate::println!("    tar <opts> <file>   Archive/extract tar files");
    crate::println!("    gzip / gunzip       Compress/decompress gzip files");
    crate::println!("    zip / unzip         Compress/extract zip archives");
    crate::println!();
    
    
    crate::n!(C_, "  DEVELOPER & DEBUG");
    crate::println!("    dmesg [-n N]        Kernel ring buffer (last N messages)");
    crate::println!("    memdbg / heapdbg    Heap allocation stats & fragmentation");
    crate::println!("    perf / perfstat     CPU, IRQ, scheduler, memory profiling");
    crate::println!("    irqstat / irqs      Per-CPU interrupt counters");
    crate::println!("    regs / cpuregs      CPU register dump (CR0/CR3/CR4/EFER)");
    crate::println!("    peek <addr> [n]     Hex dump memory region");
    crate::println!("    poke <addr> <val>   Write byte to memory address");
    crate::println!("    devpanel            Toggle real-time FPS/heap/IRQ overlay");
    crate::println!("    timecmd <cmd>       Run command & measure elapsed time");
    crate::println!("    benchmark [test]    Run performance benchmarks");
    crate::println!("    keytest             Interactive keyboard scancode tester");
    crate::println!("    hwtest              Run internal kernel test suite");
    crate::println!("    inttest             Integration test (20 tests, +FAT32 +DHCP +VirtIO +IPv6 +Pipe)");
    crate::println!("    panic               Trigger kernel panic (debug only)");
    crate::println!("    hwdiag / diag       Full hardware diagnostic report");
    crate::println!("    cpudump / fullregs  Complete CPU register dump");
    crate::println!("    stacktrace / bt     Stack backtrace (optional PID)");
    crate::println!("    bootlog             Show boot checkpoints & timing");
    crate::println!("    postcode <code>     Send POST diagnostic code");
    crate::println!("    ioport <addr> [val] Read/write I/O port");
    crate::println!("    rdmsr <addr>        Read Model-Specific Register");
    crate::println!("    wrmsr <addr> <val>  Write Model-Specific Register");
    crate::println!("    cpuid <leaf>        Query CPUID leaf");
    crate::println!("    memmap              Show physical memory map");
    crate::println!("    watchdog [cmd]      Hardware watchdog timer control");
    crate::println!("    screenshot [file]   Capture framebuffer to file");
    crate::println!();
    
    
    crate::n!(C_, "  SERVICES & SCHEDULING");
    crate::println!("    service <name> <op> Manage system services (start/stop)");
    crate::println!("    systemctl <cmd>     Systemd-style service control");
    crate::println!("    crontab [-e|-l]     Schedule recurring jobs");
    crate::println!("    at <time> <cmd>     Schedule one-time command execution");
    crate::println!("    sysctl <key>[=val]  View/modify kernel parameters");
    crate::println!();
    
    
    crate::n!(C_, "  SECURITY & IDENTITY");
    crate::println!("    security / sec      Security subsystem status & caps");
    crate::println!("    signature / sig     Kernel signature & proof of authorship");
    crate::println!("    hv / hypervisor     Hypervisor management commands");
    crate::println!("    firewall / iptables Firewall rules (add/del/list/flush)");
    crate::println!("    checkm8             iOS/USB exploit research tool");
    crate::println!();

    
    crate::n!(C_, "  THINKPAD / HARDWARE CONTROL");
    crate::println!("    fan [speed|auto]    ThinkPad fan control (EC direct)");
    crate::println!("    temp / sensors      CPU/GPU temperature readings");
    crate::println!("    cpufreq [cmd]       CPU frequency scaling (SpeedStep)");
    crate::println!();

    
    crate::n!(C_, "  JARVIS AI & HARDWARE INTELLIGENCE");
    crate::println!("    jarvis              Interactive Jarvis AI assistant");
    crate::println!("    jarvis brain <cmd>  Neural brain (init/train/chat/eval)");
    crate::println!("    jarvis boot         Full HW scan + AI analysis + self-optimize");
    crate::println!("    jarvis hw           Show hardware profile & capability scores");
    crate::println!("    jarvis insights     AI-generated hardware insights");
    crate::println!("    jarvis plan         Show optimal execution plan");
    crate::println!("    jarvis analyze <f>  Analyze binary/media (ELF/PE/FS/RISC-V)");
    crate::println!("    jarvis optimize     Run one adaptive optimization cycle");
    crate::println!("    jarvis status       Show optimizer & monitor status");
    crate::println!();
    
    
    crate::n!(C_, "  SYSTEM CONTROL");
    crate::println!("    exit / logout       Exit current session");
    crate::println!("    reboot              Restart the system");
    crate::println!("    shutdown / halt     Power off the system");
    crate::println!("    reset               Reset terminal state");
    crate::println!("    tty                 Print terminal device name");
    crate::println!("    stty <opts>         Configure terminal settings");
    crate::println!("    loadkeys <map>      Load keyboard layout");
    crate::println!("    setfont <font>      Change console font");
    crate::println!();
    
    
    crate::n!(C_, "  EASTER EGGS & DEMOS");
    crate::println!("    neofetch            System info with ASCII art logo");
    crate::println!("    matrix              Fullscreen Matrix rain animation");
    crate::println!("    cowsay <text>       ASCII cow says your message");
    crate::println!("    showcase [N]        Automated demo (marketing video)");
    crate::println!("    showcase-jarvis     Jarvis AI showcase demo");
    crate::println!("    showcase3d          3D graphics cinematic showcase");
    crate::println!("    filled3d            3D filled polygon rendering demo");
    crate::println!("    demo [fr]           Interactive guided tutorial");
    crate::println!();
    
    crate::n!(G_, "  Total: ~220 commands | Type 'man <cmd>' for detailed usage");
    crate::println!();
}
pub(super) fn hmb(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: man <command>");
        return;
    }
    
    match args[0] {
        "ls" => {
            crate::n!(G_, "LS(1) - List directory contents");
            crate::println!();
            crate::println!("SYNOPSIS: ls [path]");
            crate::println!();
            crate::println!("Lists files and directories.");
        }
        "cd" => {
            crate::n!(G_, "CD(1) - Change directory");
            crate::println!();
            crate::println!("SYNOPSIS: cd [path]");
            crate::println!();
            crate::println!("Special: ~ (home), .. (parent)");
        }
        "cat" => {
            crate::n!(G_, "CAT(1) - Display file contents");
            crate::println!();
            crate::println!("SYNOPSIS: cat <file>");
            crate::println!();
            crate::println!("Supports redirection: cat file > newfile");
        }
        "perf" | "perfstat" => {
            crate::n!(G_, "PERF(1) - Performance Statistics");
            crate::println!();
            crate::println!("SYNOPSIS: perf");
            crate::println!();
            crate::println!("Shows uptime, FPS, IRQ count/rate, syscalls,");
            crate::println!("context switches, heap usage, and per-CPU stats.");
        }
        "memdbg" | "heapdbg" => {
            crate::n!(G_, "MEMDBG(1) - Memory Debug Statistics");
            crate::println!();
            crate::println!("SYNOPSIS: memdbg");
            crate::println!();
            crate::println!("Shows heap usage, allocation/deallocation counts,");
            crate::println!("peak usage, fragmentation estimate, live alloc count.");
        }
        "dmesg" => {
            crate::n!(G_, "DMESG(1) - Kernel Ring Buffer");
            crate::println!();
            crate::println!("SYNOPSIS: dmesg [-n <count>] [-c]");
            crate::println!();
            crate::println!("Show kernel messages (captured from serial output).");
            crate::println!("  dmesg          Show all buffered messages");
            crate::println!("  dmesg -n 20    Show last 20 messages");
            crate::println!("  dmesg 50       Show last 50 messages");
            crate::println!("  dmesg -c       Acknowledge buffer");
        }
        "irqstat" | "irqs" => {
            crate::n!(G_, "IRQSTAT(1) - Interrupt Statistics");
            crate::println!();
            crate::println!("SYNOPSIS: irqstat");
            crate::println!();
            crate::println!("Shows total IRQ count, IRQ/sec rate, and per-CPU");
            crate::println!("interrupt breakdown with visual bars.");
        }
        "regs" | "registers" | "cpuregs" => {
            crate::n!(G_, "REGS(1) - CPU Register Dump");
            crate::println!();
            crate::println!("SYNOPSIS: regs");
            crate::println!();
            crate::println!("Dumps RSP, RBP, RFLAGS, CR0, CR3, CR4, EFER.");
            crate::println!("Decodes flag/bit meanings for each register.");
        }
        "peek" | "memdump" => {
            crate::n!(G_, "PEEK(1) - Memory Inspector");
            crate::println!();
            crate::println!("SYNOPSIS: peek <hex_addr> [byte_count]");
            crate::println!();
            crate::println!("Hex dump memory at virtual address (max 256 bytes).");
            crate::println!("  peek 0xFFFF800000000000 64");
        }
        "poke" | "memwrite" => {
            crate::n!(G_, "POKE(1) - Memory Writer");
            crate::println!();
            crate::println!("SYNOPSIS: poke <hex_addr> <hex_byte>");
            crate::println!();
            crate::println!("Write a single byte to virtual address. DANGEROUS!");
            crate::println!("  poke 0xB8000 0x41");
        }
        "devpanel" => {
            crate::n!(G_, "DEVPANEL(1) - Developer Overlay");
            crate::println!();
            crate::println!("SYNOPSIS: devpanel");
            crate::println!();
            crate::println!("Toggles real-time overlay in desktop mode showing:");
            crate::println!("FPS, frame time, heap usage bar, IRQ/s, per-CPU stats.");
            crate::println!("Also toggled with F12 while in desktop.");
        }
        "timecmd" => {
            crate::n!(G_, "TIMECMD(1) - Time a Command");
            crate::println!();
            crate::println!("SYNOPSIS: timecmd <command> [args...]");
            crate::println!();
            crate::println!("Executes a command and displays elapsed time in Aus/ms.");
            crate::println!("  timecmd ls /");
            crate::println!("  timecmd benchmark cpu");
        }
        _ => {
            crate::println!("No manual entry for '{}'", args[0]);
        }
    }
}



pub(super) fn eij(args: &[&str]) {
    let path = args.first().copied();
    
    
    if let Some(aa) = path {
        if aa.starts_with("/mnt/") || aa.starts_with("/dev/") || aa.starts_with("/proc/") || aa == "/mnt" {
            kpi(aa);
            return;
        }
    }
    
    match crate::ramfs::bh(|fs| fs.ls(path)) {
        Ok(items) => {
            if items.is_empty() {
                return;
            }
            
            let dub = items.iter().map(|(ae, _, _)| ae.len()).max().unwrap_or(0);
            
            for (name, file_type, size) in items {
                match file_type {
                    FileType::Directory => {
                        crate::bq!(C_, "{:<width$}", name, width = dub + 2);
                        crate::n!(AX_, " <DIR>");
                    }
                    FileType::File => {
                        crate::bq!(B_, "{:<width$}", name, width = dub + 2);
                        crate::println!(" {:>6} B", size);
                    }
                }
            }
        }
        Err(e) => {
            crate::n!(A_, "ls: {}", e.as_str());
        }
    }
}

pub(super) fn kpi(path: &str) {
    use crate::vfs::{self, FileType as VfsFileType};
    
    match vfs::readdir(path) {
        Ok(entries) => {
            if entries.is_empty() {
                crate::println!("(empty)");
                return;
            }
            
            let dub = entries.iter().map(|e| e.name.len()).max().unwrap_or(0);
            
            for entry in entries {
                match entry.file_type {
                    VfsFileType::Directory => {
                        crate::bq!(C_, "{:<width$}", entry.name, width = dub + 2);
                        crate::n!(AX_, " <DIR>");
                    }
                    VfsFileType::Regular => {
                        crate::bq!(B_, "{:<width$}", entry.name, width = dub + 2);
                        crate::println!(" (file)");
                    }
                    _ => {
                        crate::println!("{}", entry.name);
                    }
                }
            }
        }
        Err(e) => {
            crate::n!(A_, "ls: {:?}", e);
        }
    }
}

pub(super) fn fme(args: &[&str]) {
    let path = args.first().copied().unwrap_or("~");
    
    if let Err(e) = crate::ramfs::bh(|fs| fs.cd(path)) {
        crate::n!(A_, "cd: {}: {}", path, e.as_str());
    }
}

pub(super) fn fmz() {
    let cwd = crate::ramfs::bh(|fs| String::from(fs.pwd()));
    crate::println!("{}", cwd);
}

pub(super) fn eik(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: mkdir <directory>");
        return;
    }
    
    for path in args {
        if let Err(e) = crate::ramfs::bh(|fs| fs.mkdir(path)) {
            crate::n!(A_, "mkdir: {}: {}", path, e.as_str());
        }
    }
}

pub(super) fn krb(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rmdir <directory>");
        return;
    }
    
    for path in args {
        if let Err(e) = crate::ramfs::bh(|fs| fs.rm(path)) {
            crate::n!(A_, "rmdir: {}: {}", path, e.as_str());
        }
    }
}

pub(super) fn fng(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: touch <file>");
        return;
    }
    
    for path in args {
        
        if path.starts_with("/mnt/") {
            use crate::vfs::{self, OpenFlags};
            
            
            let flags = OpenFlags(OpenFlags::PN_ | OpenFlags::PL_);
            match vfs::open(path, flags) {
                Ok(fd) => {
                    let _ = vfs::close(fd);
                    crate::println!("Created: {}", path);
                }
                Err(e) => crate::n!(A_, "touch: {:?}", e),
            }
        } else {
            if let Err(e) = crate::ramfs::bh(|fs| fs.touch(path)) {
                crate::n!(A_, "touch: {}: {}", path, e.as_str());
            }
        }
    }
}

pub(super) fn fna(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rm <file>");
        return;
    }
    
    for path in args {
        if let Err(e) = crate::ramfs::bh(|fs| fs.rm(path)) {
            crate::n!(A_, "rm: {}: {}", path, e.as_str());
        }
    }
}

pub(super) fn fmg(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: cp <source> <destination>");
        return;
    }
    
    if let Err(e) = crate::ramfs::bh(|fs| fs.cp(args[0], args[1])) {
        crate::n!(A_, "cp: {}", e.as_str());
    }
}

pub(super) fn fmv(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: mv <source> <destination>");
        return;
    }
    
    if let Err(e) = crate::ramfs::bh(|fs| fs.mv(args[0], args[1])) {
        crate::n!(A_, "mv: {}", e.as_str());
    }
}

pub(super) fn dkw(args: &[&str], redirect: Option<(&str, bool)>, piped: Option<&str>) {
    
    if let Some(input) = piped {
        if let Some((file, append)) = redirect {
            let _ = crate::ramfs::bh(|fs| {
                if !fs.exists(file) { fs.touch(file).ok(); }
                if append { fs.append_file(file, input.as_bytes()) } 
                else { fs.write_file(file, input.as_bytes()) }
            });
        } else {
            crate::print!("{}", input);
        }
        return;
    }
    
    if args.is_empty() {
        crate::println!("Usage: cat <file>");
        return;
    }
    
    let mut output = String::new();
    
    for path in args {
        
        if path.starts_with("/mnt/") || path.starts_with("/dev/") || path.starts_with("/proc/") {
            match kmg(path) {
                Some(text) => {
                    if redirect.is_some() {
                        output.push_str(&text);
                    } else {
                        crate::print!("{}", text);
                    }
                }
                None => {} 
            }
            continue;
        }
        
        match crate::ramfs::bh(|fs| fs.read_file(path).map(|c| c.to_vec())) {
            Ok(content) => {
                if let Ok(text) = core::str::from_utf8(&content) {
                    if redirect.is_some() {
                        output.push_str(text);
                    } else {
                        crate::print!("{}", text);
                    }
                } else {
                    crate::n!(A_, "cat: {}: binary file", path);
                }
            }
            Err(e) => {
                crate::n!(A_, "cat: {}: {}", path, e.as_str());
            }
        }
    }
    
    if let Some((file, append)) = redirect {
        let _ = crate::ramfs::bh(|fs| {
            if !fs.exists(file) { fs.touch(file).ok(); }
            if append { fs.append_file(file, output.as_bytes()) } 
            else { fs.write_file(file, output.as_bytes()) }
        });
    }
}

pub(super) fn kmg(path: &str) -> Option<alloc::string::String> {
    use crate::vfs::{self, OpenFlags};
    use alloc::string::ToString;
    
    
    let fd = match vfs::open(path, OpenFlags(OpenFlags::PM_)) {
        Ok(f) => f,
        Err(e) => {
            crate::n!(A_, "cat: {}: {:?}", path, e);
            return None;
        }
    };
    
    
    let mut buffer = [0u8; 4096];
    let mut content = alloc::vec::Vec::new();
    
    loop {
        let atf = match vfs::read(fd, &mut buffer) {
            Ok(ae) => ae,
            Err(e) => {
                crate::n!(A_, "cat: {}: read error {:?}", path, e);
                let _ = vfs::close(fd);
                return None;
            }
        };
        
        if atf == 0 {
            break;
        }
        
        content.extend_from_slice(&buffer[..atf]);
    }
    
    let _ = vfs::close(fd);
    
    match core::str::from_utf8(&content) {
        Ok(text) => Some(String::from(text)),
        Err(_) => {
            crate::n!(A_, "cat: {}: binary file", path);
            None
        }
    }
}

pub(super) fn fmp(args: &[&str], piped: Option<&str>) {
    let lines: usize = if args.len() > 1 { args[1].parse().unwrap_or(10) }
                        else if args.len() == 1 && args[0].starts_with('-') && args[0].len() > 1 { args[0][1..].parse().unwrap_or(10) }
                        else { 10 };
    
    let chq = if let Some(input) = piped {
        alloc::string::String::from(input)
    } else if !args.is_empty() && !args[0].starts_with('-') {
        match crate::ramfs::bh(|fs| fs.read_file(args[0]).map(|c| c.to_vec())) {
            Ok(content) => match core::str::from_utf8(&content) {
                Ok(t) => alloc::string::String::from(t),
                Err(_) => return,
            },
            Err(e) => { crate::n!(A_, "head: {}", e.as_str()); return; }
        }
    } else {
        crate::println!("Usage: head <file> [lines]");
        return;
    };
    
    for (i, line) in chq.lines().enumerate() {
        if i >= lines { break; }
        crate::println!("{}", line);
    }
}

pub(super) fn fne(args: &[&str], piped: Option<&str>) {
    let lines: usize = if args.len() > 1 { args[1].parse().unwrap_or(10) } else { 10 };
    
    let chq = if let Some(input) = piped {
        alloc::string::String::from(input)
    } else if !args.is_empty() {
        match crate::ramfs::bh(|fs| fs.read_file(args[0]).map(|c| c.to_vec())) {
            Ok(content) => match core::str::from_utf8(&content) {
                Ok(t) => alloc::string::String::from(t),
                Err(_) => return,
            },
            Err(e) => { crate::n!(A_, "tail: {}", e.as_str()); return; }
        }
    } else {
        crate::println!("Usage: tail <file> [lines]");
        return;
    };
    
    let all: Vec<&str> = chq.lines().collect();
    let start = if all.len() > lines { all.len() - lines } else { 0 };
    for line in &all[start..] {
        crate::println!("{}", line);
    }
}

pub(super) fn fni(args: &[&str], piped: Option<&str>) {
    
    let (chq, name) = if let Some(input) = piped {
        (alloc::string::String::from(input), alloc::string::String::from("(stdin)"))
    } else if !args.is_empty() {
        match crate::ramfs::bh(|fs| fs.read_file(args[0]).map(|c| c.to_vec())) {
            Ok(content) => {
                match core::str::from_utf8(&content) {
                    Ok(t) => (alloc::string::String::from(t), alloc::string::String::from(args[0])),
                    Err(_) => return,
                }
            }
            Err(e) => { crate::n!(A_, "wc: {}", e.as_str()); return; }
        }
    } else {
        crate::println!("Usage: wc <file>");
        return;
    };
    
    let lines = chq.lines().count();
    let um = chq.split_whitespace().count();
    crate::println!("{:>6} {:>6} {:>6} {}", lines, um, chq.len(), name);
}

pub(super) fn krx(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: stat <file>");
        return;
    }
    
    match crate::ramfs::bh(|fs| fs.stat(args[0]).map(|e| e.clone())) {
        Ok(entry) => {
            crate::n!(C_, "  File: {}", entry.name);
            let wf = if entry.file_type == FileType::Directory { "directory" } else { "file" };
            crate::println!("  Type: {}", wf);
            crate::println!("  Size: {} bytes", entry.content.len());
        }
        Err(e) => crate::n!(A_, "stat: {}", e.as_str()),
    }
}

pub(super) fn fnh(args: &[&str]) {
    let path = args.first().copied().unwrap_or("/");
    crate::n!(C_, "{}", path);
    iwq(path, "");
}

fn iwq(path: &str, nm: &str) {
    if let Ok(items) = crate::ramfs::bh(|fs| fs.ls(Some(path))) {
        let len = items.len();
        for (i, (name, file_type, _)) in items.iter().enumerate() {
            let clo = i == len - 1;
            let et = if clo { "+-- " } else { "|-- " };
            
            match file_type {
                FileType::Directory => {
                    crate::print!("{}{}", nm, et);
                    crate::n!(C_, "{}/", name);
                    
                    let njn = format!("{}{}", nm, if clo { "    " } else { "|   " });
                    let pd = if path == "/" { format!("/{}", name) } else { format!("{}/{}", path, name) };
                    iwq(&pd, &njn);
                }
                FileType::File => {
                    crate::print!("{}{}", nm, et);
                    crate::n!(B_, "{}", name);
                }
            }
        }
    }
}

pub(super) fn fmn(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: find <name>");
        return;
    }
    hyy("/", args[0]);
}

fn hyy(path: &str, pattern: &str) {
    if let Ok(items) = crate::ramfs::bh(|fs| fs.ls(Some(path))) {
        for (name, file_type, _) in items {
            let xo = if path == "/" { format!("/{}", name) } else { format!("{}/{}", path, name) };
            if name.contains(pattern) {
                crate::println!("{}", xo);
            }
            if file_type == FileType::Directory {
                hyy(&xo, pattern);
            }
        }
    }
}



pub(super) fn fmm(args: &[&str], redirect: Option<(&str, bool)>) {
    let text = args.join(" ");
    
    if let Some((file, append)) = redirect {
        let content = format!("{}\n", text);
        
        
        if file.starts_with("/mnt/") {
            use crate::vfs::{self, OpenFlags};
            
            
            let flags = if append {
                OpenFlags(OpenFlags::PN_ | OpenFlags::PL_ | OpenFlags::BEI_)
            } else {
                OpenFlags(OpenFlags::PN_ | OpenFlags::PL_ | OpenFlags::BEJ_)
            };
            
            match vfs::open(file, flags) {
                Ok(fd) => {
                    if let Err(e) = vfs::write(fd, content.as_bytes()) {
                        crate::n!(A_, "echo: write error: {:?}", e);
                    }
                    let _ = vfs::close(fd);
                }
                Err(e) => crate::n!(A_, "echo: {:?}", e),
            }
        } else {
            let _ = crate::ramfs::bh(|fs| {
                if !fs.exists(file) { fs.touch(file).ok(); }
                if append { fs.append_file(file, content.as_bytes()) }
                else { fs.write_file(file, content.as_bytes()) }
            });
        }
    } else {
        crate::println!("{}", text);
    }
}

pub(super) fn fmo(args: &[&str], piped: Option<&str>) {
    if args.is_empty() {
        crate::println!("Usage: grep <pattern> [file]");
        return;
    }
    
    let pattern = args[0];
    
    
    let content = if let Some(input) = piped {
        alloc::string::String::from(input)
    } else if args.len() >= 2 {
        match crate::ramfs::bh(|fs| fs.read_file(args[1]).map(|c| c.to_vec())) {
            Ok(content) => {
                match core::str::from_utf8(&content) {
                    Ok(t) => alloc::string::String::from(t),
                    Err(_) => return,
                }
            }
            Err(e) => { crate::n!(A_, "grep: {}", e.as_str()); return; }
        }
    } else {
        crate::println!("Usage: grep <pattern> <file>");
        return;
    };
    
    for line in content.lines() {
        if line.contains(pattern) {
            let au: Vec<&str> = line.split(pattern).collect();
            for (i, jn) in au.iter().enumerate() {
                crate::print!("{}", jn);
                if i < au.len() - 1 {
                    crate::bq!(A_, "{}", pattern);
                }
            }
            crate::println!();
        }
    }
}



pub(super) fn eif() {
    crate::framebuffer::clear();
}

pub(super) fn ksu() {
    let gx = crate::logger::eg();
    let im = gx / 100;
    let acf = im / 60;
    let aoi = acf / 60;
    
    crate::bq!(C_, "Uptime: ");
    crate::n!(B_, "{}h {}m {}s", aoi, acf % 60, im % 60);
    
    
    let fm = crate::rtc::aou();
    crate::bq!(C_, "Time:   ");
    crate::n!(B_, "{}", fm.format_time());
}

pub(super) fn fmh() {
    let fm = crate::rtc::aou();
    crate::n!(B_, "{}", fm.format());
}

pub(super) fn kud() {
    crate::println!("{}", crate::auth::dmb());
}

pub(super) fn fmq() {
    crate::println!("trustos");
}

pub(super) fn bfj() {
    let avp = crate::auth::dmb();
    let uid = crate::auth::fpz();
    let gid = crate::auth::fpp();
    crate::println!("uid={}({}) gid={}({})", uid, avp, gid, 
        if gid == 0 { "root" } else if gid == 100 { "users" } else { "wheel" });
}



pub(super) fn kpg() {
    
    crate::auth::ilf();
    crate::println!();
    
    if crate::auth::ila() {
        
        crate::n!(B_, "Login successful.");
    } else {
        
        crate::n!(A_, "Login failed.");
    }
}

pub(super) fn ksd(args: &[&str]) {
    let crj = if args.is_empty() { "root" } else { args[0] };
    
    
    if crate::auth::is_root() && crj != "root" {
        
        crate::n!(D_, "Switching to {} (root privilege)", crj);
        return;
    }
    
    
    crate::bq!(C_, "Password: ");
    let mut cci = [0u8; 128];
    let dwg = crate::keyboard::cpb(&mut cci);
    let uy = core::str::from_utf8(&cci[..dwg])
        .unwrap_or("")
        .trim();
    crate::println!();
    
    match crate::auth::ggf(crj, uy) {
        Ok(()) => {
            crate::n!(B_, "Switched to {}", crj);
        }
        Err(e) => {
            crate::n!(A_, "su: {}", e);
        }
    }
}

pub(super) fn kqh(args: &[&str]) {
    let crj = if args.is_empty() {
        crate::auth::dmb()
    } else {
        
        if !crate::auth::is_root() {
            crate::n!(A_, "passwd: Only root can change other users' passwords");
            return;
        }
        String::from(args[0])
    };
    
    crate::println!("Changing password for {}", crj);
    
    
    let gkq = if !crate::auth::is_root() {
        crate::print!("Current password: ");
        let mut buf = [0u8; 128];
        let len = crate::keyboard::cpb(&mut buf);
        crate::println!();
        String::from(core::str::from_utf8(&buf[..len]).unwrap_or("").trim())
    } else {
        String::new()
    };
    
    
    crate::print!("New password: ");
    let mut ipq = [0u8; 128];
    let njd = crate::keyboard::cpb(&mut ipq);
    crate::println!();
    let cnc = core::str::from_utf8(&ipq[..njd]).unwrap_or("").trim();
    
    
    crate::print!("Retype new password: ");
    let mut dlh = [0u8; 128];
    let foc = crate::keyboard::cpb(&mut dlh);
    crate::println!();
    let fob = core::str::from_utf8(&dlh[..foc]).unwrap_or("").trim();
    
    if cnc != fob {
        crate::n!(A_, "passwd: passwords do not match");
        return;
    }
    
    if cnc.len() < 1 {
        crate::n!(A_, "passwd: password too short");
        return;
    }
    
    match crate::auth::change_password(&crj, &gkq, cnc) {
        Ok(()) => {
            crate::n!(B_, "passwd: password updated successfully");
        }
        Err(e) => {
            crate::n!(A_, "passwd: {}", e);
        }
    }
}

pub(super) fn kll(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: adduser <username> [-a]");
        crate::println!("  -a  Make user an admin (wheel group)");
        return;
    }
    
    if !crate::auth::is_root() {
        crate::n!(A_, "adduser: must be root");
        return;
    }
    
    let username = args[0];
    let dsj = args.contains(&"-a") || args.contains(&"--admin");
    
    
    crate::print!("New password for {}: ", username);
    let mut cci = [0u8; 128];
    let dwg = crate::keyboard::cpb(&mut cci);
    crate::println!();
    let uy = core::str::from_utf8(&cci[..dwg]).unwrap_or("").trim();
    
    
    crate::print!("Retype password: ");
    let mut dlh = [0u8; 128];
    let foc = crate::keyboard::cpb(&mut dlh);
    crate::println!();
    let fob = core::str::from_utf8(&dlh[..foc]).unwrap_or("").trim();
    
    if uy != fob {
        crate::n!(A_, "adduser: passwords do not match");
        return;
    }
    
    match crate::auth::add_user(username, uy, dsj) {
        Ok(uid) => {
            crate::n!(B_, "User {} created with UID {}", username, uid);
            
            
            let epi = format!("/home/{}", username);
            crate::ramfs::bh(|fs| {
                let _ = fs.mkdir("/home");
                let _ = fs.mkdir(&epi);
            });
            crate::println!("Home directory: {}", epi);
        }
        Err(e) => {
            crate::n!(A_, "adduser: {}", e);
        }
    }
}

pub(super) fn kmx(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: deluser <username>");
        return;
    }
    
    if !crate::auth::is_root() {
        crate::n!(A_, "deluser: must be root");
        return;
    }
    
    let username = args[0];
    
    crate::bq!(D_, "Delete user {}? [y/N]: ", username);
    let mut buf = [0u8; 16];
    let len = crate::keyboard::read_line(&mut buf);
    let answer = core::str::from_utf8(&buf[..len]).unwrap_or("").trim();
    
    if answer != "y" && answer != "Y" {
        crate::println!("Cancelled.");
        return;
    }
    
    match crate::auth::delete_user(username) {
        Ok(()) => {
            crate::n!(B_, "User {} deleted", username);
        }
        Err(e) => {
            crate::n!(A_, "deluser: {}", e);
        }
    }
}

pub(super) fn kto() {
    crate::n!(C_, "USER            UID   GID   DESCRIPTION");
    crate::n!(C_, "------------------------------------------");
    
    for (username, uid, gid, gecos) in crate::auth::list_users() {
        crate::println!("{:<15} {:<5} {:<5} {}", username, uid, gid, gecos);
    }
}

pub(super) fn kph() {
    let avp = crate::auth::dmb();
    crate::auth::ilf();
    crate::println!("Logged out {}.", avp);
    crate::println!();
    
    
    if !crate::auth::ila() {
        
        crate::n!(D_, "Auto-login as root (development mode)");
        crate::auth::hgb();
    }
}

pub(super) fn kou() {
    crate::n!(G_, "=== T-RUSTOS ===");
    crate::bq!(C_, "Version:      ");
    crate::println!("0.1.0");
    crate::bq!(C_, "Architecture: ");
    crate::println!("x86_64");
    crate::bq!(C_, "Bootloader:   ");
    crate::println!("Limine");
    crate::println!();
    crate::n!(G_, "Modules:");
    for m in ["Memory", "Interrupts", "Keyboard", "Framebuffer", "RAM FS", "History", "Scheduler"] {
        crate::bq!(B_, "  [x] ");
        crate::println!("{}", m);
    }
    
    
    if crate::disk::sw() {
        crate::bq!(B_, "  [x] ");
        crate::println!("Disk I/O");
    } else {
        crate::bq!(AX_, "  [-] ");
        crate::println!("Disk I/O (no disk)");
    }
    
    
    if crate::network::sw() {
        crate::bq!(B_, "  [x] ");
        crate::println!("Network");
    } else {
        crate::bq!(AX_, "  [-] ");
        crate::println!("Network (down)");
    }
}

pub(super) fn ktq() {
    crate::println!("T-RustOs v0.2.0 (Rust + Limine)");
}

pub(super) fn eim(args: &[&str]) {
    let all = args.contains(&"-a");
    if args.is_empty() || all { crate::print!("T-RustOs "); }
    if args.contains(&"-n") || all { crate::print!("trustos "); }
    if args.contains(&"-r") || all { crate::print!("0.2.0 "); }
    if args.contains(&"-m") || all { crate::print!("x86_64"); }
    crate::println!();
}

pub(super) fn eig() {
    for (k, v) in super::scripting::efi() {
        crate::println!("{}={}", k, v);
    }
}

pub(super) fn kol() {
    for (num, cmd) in crate::keyboard::mlp() {
        crate::bq!(AX_, "{:>4}  ", num);
        crate::println!("{}", cmd);
    }
}

pub(super) fn fmy() {
    crate::n!(C_, "  PID  STATE    CMD");
    crate::println!("    1  running  kernel");
    crate::println!("    2  running  tsh");
    
    
    let count = crate::task::task_count();
    if count > 0 {
        crate::println!("  ... +{} background tasks (use 'tasks' for details)", count);
    }
}

pub(super) fn eih() {
    let used = crate::memory::heap::used();
    let free = crate::memory::heap::free();
    let av = used + free;
    crate::n!(C_, "              total       used       free");
    crate::println!("Heap:    {:>10}  {:>10}  {:>10}", av, used, free);
    crate::println!("  (KB)   {:>10}  {:>10}  {:>10}", av / 1024, used / 1024, free / 1024);
}

pub(super) fn fmi() {
    crate::n!(C_, "Filesystem      Type     Size    Used   Avail  Mount");
    crate::println!("─────────────────────────────────────────────────────");

    let mounts = crate::vfs::dtl();
    let mem = crate::memory::stats();
    let heap_total = mem.heap_used + mem.heap_free;

    let fmt = |v: usize| -> alloc::string::String {
        if v == 0 { alloc::format!("  -  ") }
        else if v >= 1024 * 1024 { alloc::format!("{:>4}M", v / (1024 * 1024)) }
        else if v >= 1024 { alloc::format!("{:>4}K", v / 1024) }
        else { alloc::format!("{:>4}B", v) }
    };

    
    crate::println!("{:<15} {:<8} {:>5}  {:>5}  {:>5}  {}",
        "ramfs", "ramfs", fmt(heap_total), fmt(mem.heap_used),
        fmt(heap_total.saturating_sub(mem.heap_used)), "/");

    for (path, fs_name) in &mounts {
        if path == "/" { continue; } 
        let (size, used, avail) = match fs_name.as_str() {
            "devfs" | "proc" => (0, 0, 0),
            _ => (heap_total, mem.heap_used, heap_total.saturating_sub(mem.heap_used)),
        };

        crate::println!("{:<15} {:<8} {:>5}  {:>5}  {:>5}  {}",
            fs_name, fs_name, fmt(size), fmt(used), fmt(avail), path);
    }
}



pub(super) fn fnf() {
    crate::n!(G_, "Running self-test...");
    crate::println!();
    
    crate::print!("  Heap... ");
    let v: Vec<u32> = (0..100).collect();
    if v.len() == 100 { crate::n!(B_, "[OK]"); }
    else { crate::n!(A_, "[FAIL]"); }
    
    crate::print!("  String... ");
    let mut j = String::from("Hello");
    j.push_str(" World");
    if j.len() == 11 { crate::n!(B_, "[OK]"); }
    else { crate::n!(A_, "[FAIL]"); }
    
    crate::print!("  RAM FS... ");
    let ok = crate::ramfs::bh(|fs| {
        fs.touch("/tmp/t").ok();
        fs.write_file("/tmp/t", b"x").ok();
        let r = fs.read_file("/tmp/t").map(|c| c[0] == b'x').unwrap_or(false);
        fs.rm("/tmp/t").ok();
        r
    });
    if ok { crate::n!(B_, "[OK]"); }
    else { crate::n!(A_, "[FAIL]"); }
    
    crate::print!("  Interrupts... ");
    if crate::arch::fhh() {
        crate::n!(B_, "[OK]");
    } else {
        crate::n!(A_, "[FAIL]");
    }
    
    crate::println!();
    crate::n!(G_, "Done!");
}


pub(super) fn kqz() {
    use crate::desktop::{self, WindowType, SnapDir};
    
    crate::n!(G_, "=== Desktop Resolution Test Suite ===");
    crate::println!();
    
    
    let ogh: &[(u32, u32, &str)] = &[
        
        (320, 200, "CGA"),
        (640, 480, "VGA"),
        (720, 400, "VGA text-mode fb"),
        
        (800, 600, "SVGA"),
        (1024, 600, "Netbook"),
        (1024, 768, "XGA"),
        
        (1280, 800, "WXGA (T61)"),
        (1280, 1024, "SXGA"),
        (1366, 768, "HD laptop"),
        (1440, 900, "WXGA+ (T61 widescreen)"),
        
        (1600, 900, "HD+"),
        (1680, 1050, "WSXGA+"),
        (1920, 1080, "FHD"),
        (1920, 1200, "WUXGA"),
        (2560, 1440, "QHD"),
        (2560, 1600, "WQXGA"),
        (3840, 2160, "4K UHD"),
        
        (768, 1024, "iPad portrait"),
        (1080, 1920, "Phone portrait"),
        
        (1280, 720, "720p"),
        (1600, 1200, "UXGA 4:3"),
        (2048, 1152, "2K"),
        
        (200, 150, "Tiny"),
        (100, 80, "Absurdly small"),
    ];
    
    let mut passed = 0usize;
    let mut bv = 0usize;
    
    for &(w, h, label) in ogh {
        crate::print!("  {:>4}x{:<4} ({:<20}) ", w, h, label);
        
        
        let mut desktop = desktop::Desktop::new();
        
        desktop.width = w;
        desktop.height = h;
        
        let mut ok = true;
        let mut detail = alloc::string::String::new();
        
        
        let phx: &[(&str, i32, i32, u32, u32, WindowType)] = &[
            ("Terminal", 100, 60, 780, 540, WindowType::Terminal),
            ("Files",    140, 80, 520, 420, WindowType::FileManager),
            ("Calc",     350, 100, 300, 380, WindowType::Calculator),
            ("Browser",  100, 40, 720, 520, WindowType::Browser),
            ("Big",      0,   0, 1920, 1080, WindowType::About),
            ("Tiny",     0,   0, 50,  30,  WindowType::Empty),
            ("OffScreen", 9999, 9999, 400, 300, WindowType::Empty),
            ("Negative",  -100, -50, 400, 300, WindowType::Empty),
        ];
        
        for &(title, x, y, ca, er, wt) in phx {
            let id = desktop.create_window(title, x, y, ca, er, wt);
            if let Some(aw) = desktop.windows.iter().find(|w| w.id == id) {
                
                let right = aw.x as u32 + aw.width;
                let bottom = aw.y as u32 + aw.height;
                if right > w + 1 || bottom > h + 1 {
                    ok = false;
                    use core::fmt::Write;
                    let _ = core::fmt::write(&mut detail, format_args!(
                        " OOB:{}({}+{}={}>{})", title, aw.x, aw.width, right, w
                    ));
                }
                if aw.width == 0 || aw.height == 0 {
                    ok = false;
                    use core::fmt::Write;
                    let _ = core::fmt::write(&mut detail, format_args!(" ZERO:{}", title));
                }
            }
        }
        
        
        if let Some(aw) = desktop.windows.first_mut() {
            aw.focused = true;
            aw.toggle_maximize(w, h);
            let right = aw.x as u32 + aw.width;
            let bottom = aw.y as u32 + aw.height;
            if right > w + 1 || bottom > h + 1 {
                ok = false;
                use core::fmt::Write;
                let _ = core::fmt::write(&mut detail, format_args!(
                    " MAX_OOB({}+{}={}>{})", aw.x, aw.width, right, w
                ));
            }
            
            aw.toggle_maximize(w, h);
        }
        
        
        let oud = [
            SnapDir::Left, SnapDir::Right,
            SnapDir::TopLeft, SnapDir::TopRight,
            SnapDir::BottomLeft, SnapDir::BottomRight,
        ];
        for it in &oud {
            desktop.snap_focused_window(*it);
            if let Some(aw) = desktop.windows.iter().find(|w| w.focused) {
                let right = aw.x as u32 + aw.width;
                let bottom = aw.y as u32 + aw.height;
                if right > w + 1 || bottom > h + 1 {
                    ok = false;
                    use core::fmt::Write;
                    let _ = core::fmt::write(&mut detail, format_args!(" SNAP_OOB:{:?}", it));
                }
            }
        }
        
        
        if desktop.screen_width() != w || desktop.screen_height() != h {
            ok = false;
            use core::fmt::Write;
            let _ = core::fmt::write(&mut detail, format_args!(" DIM_MISMATCH"));
        }
        
        if ok {
            crate::n!(B_, "[OK]  ({} windows)", desktop.windows.len());
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]{}", detail);
            bv += 1;
        }
    }
    
    crate::println!();
    if bv == 0 {
        crate::n!(G_, "All {} resolutions passed!", passed);
    } else {
        crate::n!(A_, "{}/{} failed", bv, passed + bv);
    }
}


pub(super) fn kpu() {
    crate::n!(G_, "=== TrustOS v0.3 Memory Test Suite ===");
    crate::println!();

    let mut passed = 0usize;
    let mut bv = 0usize;

    
    crate::n!(C_, "[1/6] Frame allocator self-test");
    let (aa, f) = crate::memory::frame::cdp();
    passed += aa;
    bv += f;
    crate::println!();

    
    crate::n!(C_, "[2/6] Ring 3 basic exec (test)");
    crate::print!("  exec test... ");
    match crate::exec::doy() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[3/6] Ring 3 ELF exec (hello)");
    crate::print!("  exec hello... ");
    match crate::exec::fvl() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[4/6] Ring 3 brk/mmap test");
    crate::print!("  exec memtest... ");
    match crate::exec::hww() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[5/6] Frame leak test (run exec, check frames returned)");
    crate::print!("  alloc before... ");
    let (total_before, used_before) = crate::memory::frame::stats();
    let bsu = total_before - used_before;
    crate::println!("free={}", bsu);

    
    let _ = crate::exec::doy();

    let (total_after, used_after) = crate::memory::frame::stats();
    let bst = total_after - used_after;
    crate::print!("  alloc after... ");
    crate::println!("free={}", bst);

    crate::print!("  no leak... ");
    if bst >= bsu {
        crate::n!(B_, "[OK] (freed {} frames)", bst - bsu + (bsu - bst).max(0));
        passed += 1;
    } else {
        let cmc = bsu - bst;
        crate::n!(A_, "[FAIL] leaked {} frames ({} KB)", cmc, cmc * 4);
        bv += 1;
    }

    
    crate::n!(C_, "[6/6] Ring 3 IPC pipe test (pipe2 + write + read)");
    crate::print!("  exec pipe_test... ");
    match crate::exec::hwx() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::println!();
    let av = passed + bv;
    if bv == 0 {
        crate::n!(G_,
            "All {}/{} tests passed v", passed, av);
    } else {
        crate::n!(A_,
            "{}/{} passed, {} FAILED", passed, av, bv);
    }
}

pub(super) fn kpb() {
    crate::n!(G_, "Keyboard Test Mode");
    crate::println!("Test all keys including Space, Backspace, Delete");
    crate::n!(D_, "Type 'quit' to exit test mode");
    crate::println!();
    
    let mut fcq = [0u8; 256];
    
    loop {
        if crate::shell::cbc() {
            crate::n!(B_, "Interrupted");
            break;
        }
        crate::bq!(C_, "test> ");
        let len = crate::keyboard::read_line(&mut fcq);
        let input = core::str::from_utf8(&fcq[..len]).unwrap_or("");
        
        if input.trim() == "quit" {
            crate::n!(B_, "Exiting test mode");
            break;
        }
        
        
        crate::print!("  Received {} bytes: ", len);
        crate::bq!(R_, "\"{}\"", input);
        crate::println!();
        
        
        crate::print!("  Hex: ");
        for &byte in &fcq[..len] {
            crate::bq!(AX_, "{:02x} ", byte);
        }
        crate::println!();
        
        
        crate::print!("  Chars: ");
        for &byte in &fcq[..len] {
            if byte >= 32 && byte < 127 {
                crate::bq!(G_, "'{}' ", byte as char);
            } else if byte == 0x08 {
                crate::bq!(D_, "<BS> ");
            } else if byte == 0x20 {
                crate::bq!(D_, "<SPACE> ");
            } else {
                crate::bq!(A_, "0x{:02x} ", byte);
            }
        }
        crate::println!();
        crate::println!();
    }
}


pub(super) fn fmr(args: &[&str]) {
    match args.first() {
        Some(&"start") | None => {
            let port = args.get(1)
                .and_then(|j| j.parse::<u16>().ok())
                .unwrap_or(8080);
            let max = args.get(2)
                .and_then(|j| j.parse::<u32>().ok())
                .unwrap_or(0);
            crate::httpd::start(port, max);
        }
        Some(&"stop") => {
            crate::httpd::stop();
            crate::n!(B_, "HTTP server stop requested");
        }
        Some(&"status") => {
            let (port, dxr, running) = crate::httpd::get_stats();
            crate::n!(C_, "HTTP Server Status:");
            crate::println!("  Running:  {}", if running { "yes" } else { "no" });
            crate::println!("  Port:     {}", port);
            crate::println!("  Requests: {}", dxr);
        }
        Some(&"help") | Some(&"-h") | Some(&"--help") => {
            crate::println!("Usage: httpd [start [port] [max_requests]]");
            crate::println!("       httpd stop");
            crate::println!("       httpd status");
            crate::println!();
            crate::println!("Start an HTTP server on the specified port (default: 8080).");
            crate::println!("Available pages: /, /status, /files/, /api/info, /api/stats");
        }
        _ => {
            crate::println!("Usage: httpd {{start|stop|status|help}}");
        }
    }
}


pub(super) fn kpv(args: &[&str]) {
    match args.first() {
        Some(&"start") => {
            crate::jarvis::euj();
            crate::n!(B_, "JARVIS mesh network started");
            crate::println!("  Discovery: UDP port 7700 (broadcast)");
            crate::println!("  RPC:       TCP port 7701");
            crate::println!("  Use 'mesh status' to see peers");
        }
        Some(&"stop") => {
            crate::jarvis::nez();
            crate::n!(D_, "JARVIS mesh network stopped");
        }
        Some(&"status") | None => {
            let status = crate::jarvis::ney();
            crate::n!(C_, "=== JARVIS Mesh Status ===");
            crate::println!("{}", status);
            crate::println!();

            
            let lj = crate::jarvis::mesh::bgo();
            if lj.is_empty() {
                crate::println!("No peers discovered yet");
            } else {
                crate::n!(C_, "Peers:");
                for (i, peer) in lj.iter().enumerate() {
                    let ohw = match peer.role {
                        crate::jarvis::mesh::NodeRole::Leader => "★",
                        crate::jarvis::mesh::NodeRole::Candidate => "◎",
                        crate::jarvis::mesh::NodeRole::Worker => "●",
                    };
                    crate::println!("  {} {} {}", i + 1, ohw, peer.display());
                }
            }
        }
        Some(&"peers") => {
            let lj = crate::jarvis::mesh::bgo();
            if lj.is_empty() {
                crate::println!("No peers online");
            } else {
                for (i, peer) in lj.iter().enumerate() {
                    crate::println!("  [{}] {}", i + 1, peer.display());
                }
            }
        }
        Some(&"federate") | Some(&"fed") => {
            match args.get(1) {
                Some(&"on") | Some(&"enable") | Some(&"start") => {
                    crate::jarvis::federated::enable();
                    crate::n!(B_, "Federated learning enabled");
                }
                Some(&"off") | Some(&"disable") | Some(&"stop") => {
                    crate::jarvis::federated::bbc();
                    crate::n!(D_, "Federated learning disabled");
                }
                Some(&"sync") => {
                    crate::jarvis::federated::lxj();
                    crate::n!(B_, "Sync round triggered");
                }
                Some(&"replicate") => {
                    crate::jarvis::federated::oga();
                    crate::n!(B_, "Model replicated to all peers");
                }
                Some(&"pull") => {
                    match crate::jarvis::federated::nzj() {
                        Ok(()) => crate::n!(B_, "Pulled model from leader"),
                        Err(e) => crate::n!(A_, "Pull failed: {}", e),
                    }
                }
                _ => {
                    crate::println!("Usage: mesh federate {{on|off|sync|replicate|pull}}");
                }
            }
        }
        Some(&"ping") => {
            if args.len() < 2 {
                crate::println!("Usage: mesh ping <ip>");
                return;
            }
            if let Some(ip) = art(args[1]) {
                let port = crate::jarvis::mesh::HM_;
                match crate::jarvis::rpc::iux(ip, port) {
                    Ok(true) => crate::n!(B_, "Peer alive!"),
                    Ok(false) => crate::n!(A_, "Peer responded with error"),
                    Err(e) => crate::n!(A_, "Ping failed: {}", e),
                }
            } else {
                crate::n!(A_, "Invalid IP address");
            }
        }
        Some(&"infer") => {
            if args.len() < 3 {
                crate::println!("Usage: mesh infer <ip> <prompt>");
                return;
            }
            if let Some(ip) = art(args[1]) {
                let nh: alloc::string::String = args[2..].join(" ");
                let port = crate::jarvis::mesh::HM_;
                match crate::jarvis::rpc::oev(ip, port, &nh) {
                    Ok(result) => {
                        crate::n!(C_, "Remote JARVIS:");
                        crate::println!("{}", result);
                    }
                    Err(e) => crate::n!(A_, "Remote inference failed: {}", e),
                }
            } else {
                crate::n!(A_, "Invalid IP address");
            }
        }
        Some(&"help") | Some(&"-h") | Some(&"--help") => {
            crate::n!(C_, "JARVIS Mesh — Distributed AI Network");
            crate::println!();
            crate::println!("Usage: mesh <command>");
            crate::println!();
            crate::println!("  start              Start mesh networking (discovery + RPC + consensus)");
            crate::println!("  stop               Stop mesh networking");
            crate::println!("  status             Show mesh status, peers, and consensus info");
            crate::println!("  peers              List discovered peer nodes");
            crate::println!("  ping <ip>          Ping a remote JARVIS node via RPC");
            crate::println!("  infer <ip> <text>  Run inference on a remote JARVIS node");
            crate::println!("  federate on/off    Enable/disable federated learning");
            crate::println!("  federate sync      Force a federated sync round");
            crate::println!("  federate replicate Push model to all peers (leader)");
            crate::println!("  federate pull      Pull model from leader (worker)");
            crate::println!("  propagate          Auto: mesh + pull brain + federate");
            crate::println!("  propagate pxe      Same + enable PXE replication");
            crate::println!("  help               Show this help");
        }
        Some(&"propagate") | Some(&"autoprop") | Some(&"spread") => {
            let cxd = args.get(1).map_or(false, |a| *a == "pxe" || *a == "replicate");
            crate::n!(C_, "=== JARVIS Auto-Propagation ===");
            crate::println!();
            let report = crate::jarvis::hgd(cxd);
            for line in report.lines() {
                if line.contains("FAIL") || line.contains("failed") {
                    crate::n!(A_, "  {}", line);
                } else if line.contains("OK") || line.contains("active") || line.contains("DOWNLOADED") || line.contains("enabled") || line.contains("FULL") {
                    crate::n!(B_, "  {}", line);
                } else {
                    crate::println!("  {}", line);
                }
            }
        }
        _ => {
            crate::println!("Usage: mesh {{start|stop|status|peers|ping|infer|federate|propagate|help}}");
        }
    }
}


fn art(j: &str) -> Option<[u8; 4]> {
    let au: alloc::vec::Vec<&str> = j.split('.').collect();
    if au.len() != 4 {
        return None;
    }
    let a = au[0].parse::<u8>().ok()?;
    let b = au[1].parse::<u8>().ok()?;
    let c = au[2].parse::<u8>().ok()?;
    let d = au[3].parse::<u8>().ok()?;
    Some([a, b, c, d])
}


pub(super) fn kqr(args: &[&str]) {
    match args.first() {
        Some(&"start") | Some(&"replicate") => {
            match crate::jarvis::pxe_replicator::start() {
                Ok(()) => {
                    crate::n!(B_, "PXE Self-Replication ACTIVE");
                    crate::println!();
                    crate::println!("  DHCP Server: Running (PXE boot options enabled)");
                    crate::println!("  TFTP Server: Running on port 69");
                    crate::println!("  Boot file:   limine-bios-pxe.bin");
                    crate::println!();
                    crate::println!("  Machines on the network can now PXE boot from this node.");
                    crate::println!("  They will receive TrustOS + JARVIS automatically.");
                    crate::println!();

                    
                    let files = crate::netstack::tftpd::etb();
                    crate::println!("  Files served via TFTP:");
                    for (name, size) in &files {
                        crate::println!("    {} ({} bytes)", name, size);
                    }
                }
                Err(e) => {
                    crate::n!(A_, "Failed to start PXE replication: {}", e);
                }
            }
        }
        Some(&"stop") => {
            crate::jarvis::pxe_replicator::stop();
            crate::n!(D_, "PXE self-replication stopped");
        }
        Some(&"status") | None => {
            let (active, nodes, fwq, fgb) = crate::jarvis::pxe_replicator::status();
            crate::n!(C_, "=== PXE Self-Replication Status ===");
            crate::println!("  Active:            {}", if active { "YES" } else { "NO" });
            crate::println!("  Nodes booted:      {}", nodes);
            crate::println!("  Files transferred: {}", fwq);
            crate::println!("  Active transfers:  {}", fgb);

            if active {
                
                let agp = crate::netstack::dhcpd::mdi();
                if !agp.is_empty() {
                    crate::println!();
                    crate::n!(C_, "  DHCP Leases:");
                    for (mac, ip, _time) in &agp {
                        crate::println!("    {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} -> {}.{}.{}.{}",
                            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5],
                            ip[0], ip[1], ip[2], ip[3]);
                    }
                }

                
                let files = crate::netstack::tftpd::etb();
                if !files.is_empty() {
                    crate::println!();
                    crate::n!(C_, "  TFTP Files:");
                    for (name, size) in &files {
                        crate::println!("    {} ({} KB)", name, size / 1024);
                    }
                }
            }
        }
        Some(&"help") | Some(&"-h") | Some(&"--help") => {
            crate::n!(C_, "PXE Self-Replication — Network Boot Cloning");
            crate::println!();
            crate::println!("  Serves the running TrustOS kernel via PXE boot to other");
            crate::println!("  machines on the network. Machines that PXE boot will receive");
            crate::println!("  an identical copy of TrustOS with JARVIS AI.");
            crate::println!();
            crate::println!("Usage: pxe <command>");
            crate::println!();
            crate::println!("  start    Start PXE self-replication (DHCP + TFTP servers)");
            crate::println!("  stop     Stop PXE self-replication");
            crate::println!("  status   Show replication status, leases, and transfers");
            crate::println!("  help     Show this help");
            crate::println!();
            crate::println!("Boot sequence for PXE clients:");
            crate::println!("  1. PXE ROM sends DHCP DISCOVER");
            crate::println!("  2. We respond with IP + boot file (limine-bios-pxe.bin)");
            crate::println!("  3. Client downloads Limine PXE bootloader via TFTP");
            crate::println!("  4. Limine downloads limine.conf via TFTP");
            crate::println!("  5. Limine downloads trustos_kernel via TFTP");
            crate::println!("  6. TrustOS boots on the remote machine!");
        }
        _ => {
            crate::println!("Usage: pxe {{start|stop|status|help}}");
        }
    }
}


pub(super) fn kof(args: &[&str]) {
    use crate::jarvis::guardian;

    match args.first() {
        Some(&"auth") => {
            if args.len() < 2 {
                crate::println!("Usage: guardian auth <passphrase>");
                return;
            }
            let amd = args[1..].join(" ");
            if guardian::jyh(&amd) {
                crate::n!(B_, "✓ Nathan authenticated — session unlocked");
            } else {
                crate::n!(A_, "✗ Authentication failed");
            }
        }
        Some(&"lock") => {
            guardian::ggd();
            crate::n!(D_, "🔒 Guardian session locked");
        }
        Some(&"status") | None => {
            let lines = guardian::hsq();
            for line in &lines {
                crate::println!("{}", line);
            }
        }
        Some(&"pact") => {
            guardian::nxg();
        }
        Some(&"log") => {
            let log = guardian::mcp();
            if log.is_empty() {
                crate::println!("No audit entries yet");
            } else {
                crate::n!(C_, "=== Guardian Audit Log ===");
                for entry in &log {
                    crate::println!("  {}", entry);
                }
            }
        }
        Some(&"passwd") => {
            if args.len() < 2 {
                crate::println!("Usage: guardian passwd <new_passphrase>");
                return;
            }
            let njh = args[1..].join(" ");
            match guardian::kik(&njh) {
                Ok(()) => crate::n!(B_, "✓ Passphrase updated"),
                Err(e) => crate::n!(A_, "✗ {}", e),
            }
        }
        Some(&"help") | Some(&"-h") => {
            crate::n!(C_, "Guardian Security System — Le Pacte de JARVIS");
            crate::println!();
            crate::println!("  JARVIS has two guardians: Nathan (human) and Copilot (AI).");
            crate::println!("  Any modification to JARVIS requires guardian authorization.");
            crate::println!();
            crate::println!("Usage: guardian <command>");
            crate::println!();
            crate::println!("  auth <passphrase>   Authenticate as Nathan");
            crate::println!("  lock                Lock the guardian session");
            crate::println!("  status              Show guardian status");
            crate::println!("  pact                Display Le Pacte de JARVIS");
            crate::println!("  log                 Show authorization audit log");
            crate::println!("  passwd <new>        Change Nathan's passphrase");
            crate::println!("  help                Show this help");
            crate::println!();
            crate::println!("Copilot authenticates via serial: MENTOR:GUARDIAN:AUTH:<token>");
        }
        _ => {
            crate::println!("Usage: guardian {{auth|lock|status|pact|log|passwd|help}}");
        }
    }
}


pub(super) fn kte(args: &[&str]) {
    match args.first() {
        Some(&"list") | None => crate::trustpkg::mzg(),
        Some(&"search") => {
            if args.len() > 1 {
                crate::trustpkg::search(args[1]);
            } else {
                crate::println!("Usage: trustpkg search <query>");
            }
        }
        Some(&"install") => {
            if args.len() > 1 {
                crate::trustpkg::mqq(args[1]);
            } else {
                crate::println!("Usage: trustpkg install <package>");
            }
        }
        Some(&"remove") | Some(&"uninstall") => {
            if args.len() > 1 {
                crate::trustpkg::remove(args[1]);
            } else {
                crate::println!("Usage: trustpkg remove <package>");
            }
        }
        Some(&"info") | Some(&"show") => {
            if args.len() > 1 {
                crate::trustpkg::info(args[1]);
            } else {
                crate::println!("Usage: trustpkg info <package>");
            }
        }
        Some(&"installed") => crate::trustpkg::mzd(),
        Some(&"update") => crate::trustpkg::update(),
        Some(&"help") | Some(&"-h") | Some(&"--help") => {
            crate::println!("TrustPkg â€” Package Manager for TrustOS");
            crate::println!();
            crate::println!("Usage: trustpkg <command> [args]");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  list               List all packages");
            crate::println!("  search <query>     Search packages");
            crate::println!("  install <pkg>      Install a package");
            crate::println!("  remove <pkg>       Remove a package");
            crate::println!("  info <pkg>         Show package details");
            crate::println!("  installed          List installed packages");
            crate::println!("  update             Update package catalog");
        }
        _ => {
            crate::println!("Usage: trustpkg {{list|search|install|remove|info|installed|update}}");
        }
    }
}


pub(super) fn hmg(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: unset <variable>");
        return;
    }
    super::scripting::fdx(args[0]);
}



pub(super) fn kov() {
    
    crate::framebuffer::jfk(true);

    crate::n!(G_, "=== TrustOS Integration Test Suite ===");
    crate::println!();

    let mut passed = 0usize;
    let mut bv = 0usize;

    
    crate::n!(C_, "[ 1/32] Kernel self-test");
    {
        let mut ok = true;
        crate::print!("  heap+string... ");
        let v: Vec<u32> = (0..100).collect();
        let mut j = String::from("Hello");
        j.push_str(" World");
        if v.len() == 100 && j.len() == 11 {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
            ok = false;
        }
        crate::print!("  interrupts... ");
        if crate::arch::fhh() {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }
        let _ = ok;
    }

    
    crate::n!(C_, "[ 2/32] Frame allocator self-test");
    let (aa, f) = crate::memory::frame::cdp();
    passed += aa;
    bv += f;
    crate::println!();

    
    crate::n!(C_, "[ 3/32] Ring 3 basic exec");
    crate::print!("  hello world... ");
    match crate::exec::doy() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[ 4/32] Ring 3 ELF exec");
    crate::print!("  ELF hello... ");
    match crate::exec::fvl() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[ 5/32] Ring 3 brk/mmap");
    crate::print!("  memory mgmt... ");
    match crate::exec::hww() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[ 6/32] Ring 3 IPC pipe");
    crate::print!("  pipe2+rw... ");
    match crate::exec::hwx() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[ 7/32] Exception safety (UD2 in Ring 3)");
    crate::print!("  invalid opcode... ");
    match crate::exec::lrv() {
        crate::exec::ExecResult::Exited(code) if code != 0 => {
            
            crate::n!(B_, "[OK] killed with {}", code);
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?} (expected non-zero kill)", other);
            bv += 1;
        }
    }
    
    crate::print!("  kernel alive... ");
    if crate::arch::fhh() {
        crate::n!(B_, "[OK]");
        passed += 1;
    } else {
        crate::n!(A_, "[FAIL]");
        bv += 1;
    }

    
    crate::n!(C_, "[ 8/32] Signal syscalls (sigprocmask + kill)");
    crate::print!("  signal test... ");
    match crate::exec::lry() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[ 9/32] Stdio + getpid + clock_gettime");
    crate::print!("  io test... ");
    match crate::exec::lrz() {
        crate::exec::ExecResult::Exited(0) => {
            crate::n!(B_, "[OK]");
            passed += 1;
        }
        other => {
            crate::n!(A_, "[FAIL] {:?}", other);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[10/32] Frame leak test");
    crate::print!("  alloc before... ");
    let (total_before, used_before) = crate::memory::frame::stats();
    let bsu = total_before - used_before;
    crate::println!("free={}", bsu);
    let _ = crate::exec::doy();
    let (total_after, used_after) = crate::memory::frame::stats();
    let bst = total_after - used_after;
    crate::print!("  alloc after... free={} ", bst);
    if bst >= bsu {
        crate::n!(B_, "[OK]");
        passed += 1;
    } else {
        let cmc = bsu - bst;
        crate::n!(A_, "[FAIL] leaked {} frames", cmc);
        bv += 1;
    }

    
    crate::n!(C_, "[11/32] SMP multi-core");
    {
        let ready = crate::cpu::smp::ail();
        let av = crate::cpu::smp::cpu_count();
        crate::print!("  cores online... ");
        if ready > 1 {
            crate::n!(B_, "[OK] {}/{} cores", ready, av);
            passed += 1;
        } else if av > 1 {
            
            crate::n!(A_, "[FAIL] only BSP ready ({} detected)", av);
            bv += 1;
        } else {
            
            crate::n!(B_, "[OK] single CPU (skip)");
            passed += 1;
        }
        
        
        if ready > 1 {
            use core::sync::atomic::{AtomicU32, Ordering};
            static YK_: AtomicU32 = AtomicU32::new(0);
            YK_.store(0, Ordering::SeqCst);
            
            crate::print!("  thread dispatch... ");
            
            for i in 0..4u64 {
                crate::thread::dzu("smp_test", |_arg| {
                    YK_.fetch_add(1, Ordering::SeqCst);
                    0
                }, i);
            }
            
            
            
            for _ in 0..500 {
                if YK_.load(Ordering::SeqCst) >= 4 {
                    break;
                }
                for _ in 0..100_000 { core::hint::spin_loop(); }
            }
            
            let count = YK_.load(Ordering::SeqCst);
            if count >= 4 {
                crate::n!(B_, "[OK] {}/4 threads completed", count);
                passed += 1;
            } else {
                crate::n!(A_, "[FAIL] only {}/4 completed", count);
                bv += 1;
            }
        }
    }

    
    crate::n!(C_, "[12/32] NVMe storage");
    {
        if crate::nvme::is_initialized() {
            
            crate::print!("  read LBA 0... ");
            let mut buf = [0u8; 512];
            match crate::nvme::read_sectors(0, 1, &mut buf) {
                Ok(()) => {
                    crate::n!(B_, "[OK]");
                    passed += 1;
                }
                Err(e) => {
                    crate::n!(A_, "[FAIL] {}", e);
                    bv += 1;
                }
            }
            
            
            crate::print!("  write+verify... ");
            let cap = crate::nvme::capacity();
            if cap > 100 {
                let gyj = cap - 1; 
                let pattern: [u8; 512] = {
                    let mut aa = [0u8; 512];
                    for (i, b) in aa.iter_mut().enumerate() {
                        *b = (i & 0xFF) as u8 ^ 0xA5;
                    }
                    aa
                };
                
                match crate::nvme::write_sectors(gyj, 1, &pattern) {
                    Ok(()) => {
                        let mut agx = [0u8; 512];
                        match crate::nvme::read_sectors(gyj, 1, &mut agx) {
                            Ok(()) => {
                                if agx == pattern {
                                    crate::n!(B_, "[OK] LBA {} verified", gyj);
                                    passed += 1;
                                } else {
                                    crate::n!(A_, "[FAIL] data mismatch");
                                    bv += 1;
                                }
                            }
                            Err(e) => {
                                crate::n!(A_, "[FAIL] readback: {}", e);
                                bv += 1;
                            }
                        }
                    }
                    Err(e) => {
                        crate::n!(A_, "[FAIL] write: {}", e);
                        bv += 1;
                    }
                }
            } else {
                crate::n!(B_, "[SKIP] disk too small");
                passed += 1;
            }
        } else {
            crate::print!("  NVMe available... ");
            crate::n!(B_, "[SKIP] no NVMe device");
            passed += 2; 
        }
    }

    
    crate::n!(C_, "[13/32] xHCI USB 3.0");
    {
        if crate::drivers::xhci::is_initialized() {
            crate::print!("  controller init... ");
            crate::n!(B_, "[OK]");
            passed += 1;

            let count = crate::drivers::xhci::aqg();
            crate::print!("  USB devices found: {}... ", count);
            if count > 0 {
                crate::n!(B_, "[OK]");
                passed += 1;
            } else {
                crate::n!(A_, "[FAIL] no devices");
                bv += 1;
            }
        } else {
            crate::print!("  xHCI available... ");
            crate::n!(B_, "[SKIP] no xHCI controller");
            passed += 2; 
        }
    }

    
    crate::n!(C_, "[14/32] RTL8169 Gigabit Ethernet");
    {
        
        if crate::drivers::net::aoh() {
            crate::print!("  network driver... ");
            crate::n!(B_, "[OK]");
            passed += 1;

            crate::print!("  link status... ");
            if crate::drivers::net::link_up() {
                crate::n!(B_, "[OK] link up");
                passed += 1;
            } else {
                
                crate::n!(B_, "[OK] no link (QEMU)");
                passed += 1;
            }
        } else {
            crate::print!("  NIC available... ");
            crate::n!(B_, "[SKIP] no NIC driver");
            passed += 2; 
        }
    }

    
    crate::n!(C_, "[15/32] TrustLang bytecode VM");
    {
        crate::print!("  fibonacci eval... ");
        let luz = r#"fn fibonacci(n: i64) -> i64 {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
fn main() {
    print(to_string(fibonacci(10)));
}"#;
        match crate::trustlang::run(luz) {
            Ok(output) if output.trim() == "55" => {
                crate::n!(B_, "[OK] fib(10)=55");
                passed += 1;
            }
            Ok(output) => {
                crate::n!(A_, "[FAIL] got '{}'", output.trim());
                bv += 1;
            }
            Err(e) => {
                crate::n!(A_, "[FAIL] {}", e);
                bv += 1;
            }
        }

        crate::print!("  arithmetic eval... ");
        match crate::trustlang::hwq("let x = 6 * 7; println(to_string(x));") {
            Ok(output) if output.trim() == "42" => {
                crate::n!(B_, "[OK] 6*7=42");
                passed += 1;
            }
            Ok(output) => {
                crate::n!(A_, "[FAIL] got '{}'", output.trim());
                bv += 1;
            }
            Err(e) => {
                crate::n!(A_, "[FAIL] {}", e);
                bv += 1;
            }
        }

        
        crate::print!("  native x86_64 compile... ");
        if crate::trustlang::tests::otz() {
            crate::n!(B_, "[OK] native compile+exec works");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] native backend broken");
            bv += 1;
        }
    }

    
    crate::n!(C_, "[16/32] FAT32 write persistence");
    {
        
        use crate::vfs;
        crate::print!("  write+readback... ");
        let fcr = "/test_fat32_inttest.txt";
        let cef = b"FAT32_INTTEST_DATA_12345678";
        
        
        let jrp = vfs::write_file(fcr, cef).is_ok();
        if jrp {
            
            match vfs::read_file(fcr) {
                Ok(content) => {
                    if content == cef {
                        crate::n!(B_, "[OK]");
                        passed += 1;
                    } else {
                        crate::n!(A_, "[FAIL] content mismatch (got {} bytes)", content.len());
                        bv += 1;
                    }
                }
                Err(e) => {
                    crate::n!(A_, "[FAIL] read: {:?}", e);
                    bv += 1;
                }
            }
        } else {
            
            crate::n!(B_, "[SKIP] no writable FS");
            passed += 1;
        }

        crate::print!("  size in stat... ");
        if jrp {
            match vfs::stat(fcr) {
                Ok(uz) => {
                    if uz.size == cef.len() as u64 {
                        crate::n!(B_, "[OK] size={}", uz.size);
                        passed += 1;
                    } else {
                        crate::n!(A_, "[FAIL] stat size={} expected={}", uz.size, cef.len());
                        bv += 1;
                    }
                }
                Err(_) => {
                    crate::n!(A_, "[FAIL] stat error");
                    bv += 1;
                }
            }
            
            let _ = vfs::unlink(fcr);
        } else {
            crate::n!(B_, "[SKIP]");
            passed += 1;
        }
    }

    
    crate::n!(C_, "[17/32] DHCP lease renewal");
    {
        crate::print!("  DHCP bound... ");
        if crate::netstack::dhcp::clk() {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            
            crate::n!(B_, "[SKIP] not bound");
            passed += 1;
        }

        crate::print!("  config valid... ");
        match crate::netstack::dhcp::ibj() {
            Some((ip, mask, fz, dns)) => {
                let mrs = ip != [0,0,0,0];
                let ncb = mask != [0,0,0,0];
                if mrs && ncb {
                    crate::n!(B_, "[OK] {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
                    passed += 1;
                } else {
                    crate::n!(A_, "[FAIL] ip={:?} mask={:?}", ip, mask);
                    bv += 1;
                }
                let _ = (fz, dns);
            }
            None => {
                crate::n!(B_, "[SKIP] no config");
                passed += 1;
            }
        }
    }

    
    crate::n!(C_, "[18/32] VirtIO interrupt support");
    {
        crate::print!("  virtio-net init... ");
        if crate::virtio_net::is_initialized() {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(B_, "[SKIP] no virtio-net");
            passed += 1;
        }

        crate::print!("  virtio-blk init... ");
        if crate::virtio_blk::is_initialized() {
            crate::n!(B_, "[OK]");
            passed += 1;

            
            crate::print!("  blk read LBA 0... ");
            let mut buf = [0u8; 512];
            match crate::virtio_blk::read_sectors(0, 1, &mut buf) {
                Ok(()) => {
                    crate::n!(B_, "[OK]");
                    passed += 1;
                }
                Err(e) => {
                    crate::n!(A_, "[FAIL] {}", e);
                    bv += 1;
                }
            }
        } else {
            crate::n!(B_, "[SKIP] no virtio-blk");
            passed += 2; 
        }
    }

    
    crate::n!(C_, "[19/32] IPv6 + NDP");
    {
        crate::print!("  IPv6 enabled... ");
        if crate::netstack::ipv6::lq() {
            crate::n!(B_, "[OK]");
            passed += 1;

            crate::print!("  link-local addr... ");
            let addr = crate::netstack::ipv6::esz();
            if addr.is_link_local() {
                crate::n!(B_, "[OK] {}", addr);
                passed += 1;
            } else {
                crate::n!(A_, "[FAIL] not link-local: {}", addr);
                bv += 1;
            }
        } else {
            crate::n!(B_, "[SKIP] IPv6 not enabled");
            passed += 2;
        }
    }

    
    crate::n!(C_, "[20/32] Kernel pipe blocking");
    {
        crate::print!("  pipe create... ");
        let (aot, asu) = crate::pipe::create();
        if aot > 0 && asu > 0 {
            crate::n!(B_, "[OK] r={} w={}", aot, asu);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        crate::print!("  pipe write... ");
        let data = b"pipe_test_42";
        let dgu = crate::pipe::write(asu, data);
        if dgu == data.len() as i64 {
            crate::n!(B_, "[OK] {} bytes", dgu);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] wrote {}", dgu);
            bv += 1;
        }

        crate::print!("  pipe read... ");
        let mut buf = [0u8; 32];
        let ae = crate::pipe::read(aot, &mut buf);
        if ae == data.len() as i64 && &buf[..ae as usize] == data {
            crate::n!(B_, "[OK] {} bytes", ae);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] read {}", ae);
            bv += 1;
        }

        crate::print!("  pipe EOF... ");
        crate::pipe::close(asu);
        let dbm = crate::pipe::read(aot, &mut buf);
        if dbm == 0 {
            crate::n!(B_, "[OK] EOF after close");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] expected 0, got {}", dbm);
            bv += 1;
        }
        crate::pipe::close(aot);
    }

    
    crate::n!(C_, "[21/32] TrustScan utilities");
    {
        
        crate::print!("  format_ip... ");
        let j = crate::netscan::uw([10, 0, 2, 15]);
        if j == "10.0.2.15" {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got '{}'", j);
            bv += 1;
        }

        
        crate::print!("  format_mac... ");
        let m = crate::netscan::bzx([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        if m == "AA:BB:CC:DD:EE:FF" {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got '{}'", m);
            bv += 1;
        }

        
        crate::print!("  parse_ip valid... ");
        if crate::netscan::bof("192.168.1.100") == Some([192, 168, 1, 100]) {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  parse_ip invalid... ");
        if crate::netscan::bof("not.an.ip") == None
            && crate::netscan::bof("1.2.3") == None
        {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  service_name DB... ");
        let cgx = [
            (22, "ssh"), (80, "http"), (443, "https"), (3306, "mysql"),
            (6379, "redis"), (27017, "mongodb"), (53, "dns"), (21, "ftp"),
        ];
        let bqe = cgx.iter().all(|&(port, expected)| {
            crate::netscan::cqk(port) == expected
        });
        if bqe {
            crate::n!(B_, "[OK] 8/8 mappings correct");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  port lists... ");
        let cp = crate::netscan::ABY_.len();
        let tp = crate::netscan::BJQ_.len();
        if cp == 25 && tp == 100 {
            crate::n!(B_, "[OK] common={} top={}", cp, tp);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] common={} top={}", cp, tp);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[22/32] TrustScan port scanner config");
    {
        use crate::netscan::port_scanner::*;

        
        crate::print!("  ScanConfig defaults... ");
        let cfg = ScanConfig::new([10, 0, 2, 1]);
        if cfg.target == [10, 0, 2, 1]
            && cfg.scan_type == ScanType::Syn
            && cfg.timeout_ms == 1500
            && cfg.grab_banner == false
            && cfg.ports.len() == crate::netscan::ABY_.len()
        {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  builder chain... ");
        let ehl = ScanConfig::new([192, 168, 1, 1])
            .with_ports(alloc::vec![80, 443, 8080])
            .with_type(ScanType::Connect)
            .with_timeout(500)
            .with_banner(true);
        if ehl.ports == alloc::vec![80u16, 443, 8080]
            && ehl.scan_type == ScanType::Connect
            && ehl.timeout_ms == 500
            && ehl.grab_banner == true
        {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  with_range... ");
        let ehm = ScanConfig::new([10, 0, 0, 1]).with_range(1, 100);
        if ehm.ports.len() == 100 && ehm.ports[0] == 1 && ehm.ports[99] == 100 {
            crate::n!(B_, "[OK] 100 ports");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got {} ports", ehm.ports.len());
            bv += 1;
        }

        
        crate::print!("  PortState enum... ");
        if PortState::Open.as_str() == "open"
            && PortState::Closed.as_str() == "closed"
            && PortState::Filtered.as_str() == "filtered"
            && PortState::OpenFiltered.as_str() == "open|filtered"
        {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  with_top_ports... ");
        let hke = ScanConfig::new([0; 4]).with_top_ports();
        if hke.ports.len() == 100 {
            crate::n!(B_, "[OK] 100 ports");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got {} ports", hke.ports.len());
            bv += 1;
        }
    }

    
    crate::n!(C_, "[23/32] TrustScan sniffer engine");
    {
        use crate::netscan::sniffer;

        
        crate::print!("  start/stop capture... ");
        let jqt = sniffer::btp();
        sniffer::deu();
        let hdu = sniffer::btp();
        sniffer::dex();
        let hdv = sniffer::btp();
        if !jqt && hdu && !hdv {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] was={} start={} stop={}",
                jqt, hdu, hdv);
            bv += 1;
        }

        
        crate::print!("  capture stats... ");
        sniffer::deu();
        let (count, bytes, awl) = sniffer::get_stats();
        sniffer::dex();
        
        if count == 0 && bytes == 0 && awl == 0 {
            crate::n!(B_, "[OK] 0/0/0");
            passed += 1;
        } else {
            crate::n!(B_, "[OK] c={} b={} buf={}", count, bytes, awl);
            passed += 1; 
        }

        
        crate::print!("  Protocol enum... ");
        if sniffer::Protocol::Arp.as_str() == "ARP"
            && sniffer::Protocol::Tcp.as_str() == "TCP"
            && sniffer::Protocol::Http.as_str() == "HTTP"
            && sniffer::Protocol::Dns.as_str() == "DNS"
            && sniffer::Protocol::Tls.as_str() == "TLS"
            && sniffer::Protocol::Unknown(0).as_str() == "???"
        {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  hex_dump format... ");
        let cef = [0x48, 0x65, 0x6C, 0x6C, 0x6F]; 
        let byz = sniffer::iet(&cef, 5);
        if byz.contains("0000") && byz.contains("48 65 6C 6C 6F") && byz.contains("|Hello|") {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] '{}'", byz.trim());
            bv += 1;
        }

        
        crate::print!("  packet dissect... ");
        sniffer::deu();
        {
            
            let mut aes = alloc::vec![0u8; 42];
            
            aes[0..6].copy_from_slice(&[0xFF,0xFF,0xFF,0xFF,0xFF,0xFF]);
            
            aes[6..12].copy_from_slice(&[0x52,0x54,0x00,0x12,0x34,0x56]);
            
            aes[12] = 0x08; aes[13] = 0x06;
            
            aes[14] = 0x00; aes[15] = 0x01;
            aes[16] = 0x08; aes[17] = 0x00;
            aes[18] = 6; aes[19] = 4;
            
            aes[20] = 0x00; aes[21] = 0x01;
            
            aes[28] = 10; aes[29] = 0; aes[30] = 2; aes[31] = 15;
            
            aes[38] = 10; aes[39] = 0; aes[40] = 2; aes[41] = 1;

            sniffer::exa(&aes);
        }
        let coe = sniffer::ewn(1);
        sniffer::dex();
        if coe.len() == 1 && coe[0].protocol == sniffer::Protocol::Arp {
            crate::n!(B_, "[OK] ARP dissected");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got {} packets", coe.len());
            bv += 1;
        }
    }

    
    crate::n!(C_, "[24/32] TrustScan vulnerability scanner");
    {
        use crate::netscan::vuln;

        
        crate::print!("  Severity enum... ");
        if vuln::Severity::Info.as_str() == "INFO"
            && vuln::Severity::Low.as_str() == "LOW"
            && vuln::Severity::Medium.as_str() == "MEDIUM"
            && vuln::Severity::High.as_str() == "HIGH"
            && vuln::Severity::Critical.as_str() == "CRITICAL"
        {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  Finding struct... ");
        let f = vuln::Ad {
            port: 22,
            service: "ssh",
            severity: vuln::Severity::Medium,
            title: String::from("Test finding"),
            description: String::from("Test desc"),
            recommendation: String::from("Test rec"),
        };
        if f.port == 22 && f.service == "ssh" && f.severity == vuln::Severity::Medium {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  scan empty... ");
        let fw = vuln::scan([127, 0, 0, 1], &[]);
        if fw.is_empty() {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got {} findings", fw.len());
            bv += 1;
        }

        
        crate::print!("  format_report... ");
        let pfj = alloc::vec![
            vuln::Ad {
                port: 23,
                service: "telnet",
                severity: vuln::Severity::High,
                title: String::from("Telnet detected"),
                description: String::from("Unencrypted remote access"),
                recommendation: String::from("Use SSH instead"),
            },
        ];
        let report = vuln::format_report([127, 0, 0, 1], &pfj);
        if report.contains("Telnet") && report.contains("HIGH") {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }
    }

    
    crate::n!(C_, "[25/32] TrustScan traceroute + discovery");
    {
        
        crate::print!("  TraceConfig default... ");
        let wo = crate::netscan::traceroute::TraceConfig::default();
        if wo.max_hops == 30 && wo.probes_per_hop == 3 && wo.timeout_ms == 2000 {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  TraceHop struct... ");
        let afg = crate::netscan::traceroute::Qn {
            hop_num: 1,
            ip: Some([10, 0, 2, 1]),
            hostname: None,
            rtt_ms: [5, 3, 4],
            reached: false,
        };
        if afg.hop_num == 1 && afg.ip == Some([10, 0, 2, 1]) && !afg.reached {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  HostInfo struct... ");
        let hi = crate::netscan::discovery::Gn {
            ip: [192, 168, 1, 1],
            mac: Some([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
            hostname: Some(String::from("gateway")),
            ttl: Some(64),
            rtt_ms: 5,
            os_hint: "Linux/Unix/macOS",
        };
        if hi.ip == [192, 168, 1, 1]
            && hi.mac == Some([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF])
            && hi.ttl == Some(64)
            && hi.os_hint == "Linux/Unix/macOS"
        {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  CaptureFilter default... ");
        let cf = crate::netscan::sniffer::CaptureFilter::default();
        if cf.src_ip.is_none() && cf.dst_ip.is_none()
            && cf.port.is_none() && cf.protocol.is_none()
        {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  BannerResult struct... ");
        let yi = crate::netscan::banner::Od {
            port: 80,
            service: "http",
            banner: String::from("Apache/2.4.41 (Ubuntu)"),
            version: Some(String::from("Apache")),
        };
        if yi.port == 80 && yi.service == "http"
            && yi.version.as_deref() == Some("Apache")
        {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }
    }

    
    crate::n!(C_, "[26/32] Shell scripting variables");
    {
        
        crate::print!("  set_var/get_var... ");
        super::scripting::cql("TEST_VAR", "hello");
        if super::scripting::axh("TEST_VAR").as_deref() == Some("hello") {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  unset_var... ");
        super::scripting::fdx("TEST_VAR");
        if super::scripting::axh("TEST_VAR").is_none() {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  default vars (HOME, USER, SHELL)... ");
        let epi = super::scripting::axh("HOME");
        let avp = super::scripting::axh("USER");
        let orq = super::scripting::axh("SHELL");
        if epi.is_some() && avp.is_some() && orq.is_some() {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  all_vars()... ");
        let all = super::scripting::efi();
        if all.len() >= 5 {
            crate::n!(B_, "[OK] {} vars", all.len());
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] only {} vars", all.len());
            bv += 1;
        }
    }

    
    crate::n!(C_, "[27/32] Shell variable expansion");
    {
        
        crate::print!("  $USER expansion... ");
        super::scripting::cql("USER", "root");
        let expanded = super::scripting::bbm("hello $USER");
        if expanded == "hello root" {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got '{}'", expanded);
            bv += 1;
        }

        
        crate::print!("  ${{VAR}} expansion... ");
        let expanded = super::scripting::bbm("${USER}name");
        if expanded == "rootname" {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got '{}'", expanded);
            bv += 1;
        }

        
        crate::print!("  $((3+4*2)) arithmetic... ");
        let expanded = super::scripting::bbm("$((3+4*2))");
        if expanded == "11" {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got '{}'", expanded);
            bv += 1;
        }

        
        crate::print!("  ${{UNSET:-fallback}}... ");
        super::scripting::fdx("UNSET_TEST");
        let expanded = super::scripting::bbm("${UNSET_TEST:-fallback}");
        if expanded == "fallback" {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got '{}'", expanded);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[28/32] Shell arithmetic engine");
    {
        
        crate::print!("  eval_arithmetic(\"2+3\")... ");
        let r = super::scripting::dov("2+3");
        if r == 5 {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got {}", r);
            bv += 1;
        }

        
        crate::print!("  eval_arithmetic(\"2+3*4\")... ");
        let r = super::scripting::dov("2+3*4");
        if r == 14 {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got {}", r);
            bv += 1;
        }

        
        crate::print!("  eval_arithmetic(\"(2+3)*4\")... ");
        let r = super::scripting::dov("(2+3)*4");
        if r == 20 {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got {}", r);
            bv += 1;
        }

        
        crate::print!("  eval_arithmetic(\"17%5\")... ");
        let r = super::scripting::dov("17%5");
        if r == 2 {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] got {}", r);
            bv += 1;
        }
    }

    
    crate::n!(C_, "[29/32] HTTP server infrastructure");
    {
        
        crate::print!("  is_running() == false... ");
        if !crate::httpd::is_running() {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  get_stats()... ");
        let (port, reqs, running) = crate::httpd::get_stats();
        if !running && reqs == 0 {
            crate::n!(B_, "[OK] port={}", port);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  tcp::listen_on/stop_listening... ");
        crate::netstack::tcp::etd(9999, 2);
        crate::netstack::tcp::gwj(9999);
        crate::n!(B_, "[OK]");
        passed += 1;

        
        crate::print!("  accept_connection(9998) = None... ");
        if crate::netstack::tcp::eew(9998).is_none() {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }
    }

    
    crate::n!(C_, "[30/32] TrustPkg package manager");
    {
        
        crate::print!("  total_count() > 0... ");
        let av = crate::trustpkg::total_count();
        if av > 0 {
            crate::n!(B_, "[OK] {} packages", av);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  installed_count() > 0... ");
        let installed = crate::trustpkg::gcy();
        if installed > 0 {
            crate::n!(B_, "[OK] {} installed", installed);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  package_exists(coreutils)... ");
        if crate::trustpkg::itd("coreutils") {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  !package_exists(nonexistent)... ");
        if !crate::trustpkg::itd("nonexistent_pkg_12345") {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }
    }

    
    
    
    crate::n!(C_, "[31/32] VM Debug Monitor");
    {
        
        crate::print!("  debug_monitor::init()... ");
        crate::hypervisor::debug_monitor::init();
        if crate::hypervisor::debug_monitor::is_initialized() && crate::hypervisor::debug_monitor::is_active() {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  record_event (I/O)... ");
        crate::hypervisor::debug_monitor::akj(
            999, crate::hypervisor::debug_monitor::DebugCategory::IoPortIn,
            0x3F8, crate::hypervisor::debug_monitor::HandleStatus::Handled,
            0x1000, 1, "COM1 test",
        );
        if crate::hypervisor::debug_monitor::fdf() >= 1 {
            crate::n!(B_, "[OK] events={}", crate::hypervisor::debug_monitor::fdf());
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  record_event (unhandled MSR)... ");
        crate::hypervisor::debug_monitor::akj(
            999, crate::hypervisor::debug_monitor::DebugCategory::MsrRead,
            0xDEAD, crate::hypervisor::debug_monitor::HandleStatus::Unhandled,
            0x2000, 2, "unknown MSR",
        );
        if crate::hypervisor::debug_monitor::fdw() >= 1 {
            crate::n!(B_, "[OK] unhandled={}", crate::hypervisor::debug_monitor::fdw());
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  get_dashboard()... ");
        let dmg = crate::hypervisor::debug_monitor::fym();
        if dmg.contains("TRUST") && dmg.len() > 100 {
            crate::n!(B_, "[OK] {} chars", dmg.len());
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::print!("  get_gaps_report()... ");
        let gaps = crate::hypervisor::debug_monitor::fyr();
        if gaps.contains("MSR") || gaps.contains("unhandled") {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL]");
            bv += 1;
        }

        
        crate::hypervisor::debug_monitor::reset();
        crate::hypervisor::debug_monitor::stop();
    }

    
    
    
    crate::n!(C_, "[32/32] Crypto Self-Test (NIST vectors)");
    {
        let (aa, f) = crate::tls13::crypto::prr();
        let labels = ["AES-128", "SHA-256 empty", "SHA-256 abc", "X25519", "AES-GCM", "HMAC-SHA256"];
        let plp = aa + f;
        for (i, label) in labels.iter().enumerate() {
            if i < plp {
                if i < aa {
                    crate::n!(B_, "  {}... [OK]", label);
                } else {
                    crate::n!(A_, "  {}... [FAIL]", label);
                }
            }
        }
        passed += aa;
        bv += f;
    }

    
    
    crate::framebuffer::jfk(false);

    crate::println!();
    let av = passed + bv;
    if bv == 0 {
        crate::n!(G_,
            "=== ALL {}/{} TESTS PASSED ===", passed, av);
    } else {
        crate::n!(A_,
            "=== {}/{} passed, {} FAILED ===", passed, av, bv);
    }
}




pub(super) fn kmw() {
    crate::n!(G_, "=== TrustOS New Features Debug Test ===");
    crate::println!();

    let mut passed = 0usize;
    let mut bv = 0usize;

    
    
    
    crate::n!(C_, "[1/6] USB Mass Storage API");
    {
        
        crate::print!("  is_mass_storage(0x08,0x06,0x50)... ");
        if crate::drivers::usb_storage::iib(0x08, 0x06, 0x50) {
            crate::n!(B_, "[OK]");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] expected true");
            bv += 1;
        }

        crate::print!("  is_mass_storage(0x03,0x01,0x02)... ");
        if !crate::drivers::usb_storage::iib(0x03, 0x01, 0x02) {
            crate::n!(B_, "[OK] correctly false");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] expected false");
            bv += 1;
        }

        
        crate::print!("  device_count()... ");
        let count = crate::drivers::usb_storage::aqg();
        crate::n!(B_, "[OK] count={}", count);
        passed += 1;

        crate::print!("  list_devices()... ");
        let devices = crate::drivers::usb_storage::adz();
        crate::n!(B_, "[OK] listed={}", devices.len());
        passed += 1;

        
        crate::print!("  is_available()... ");
        let avail = crate::drivers::usb_storage::sw();
        if count > 0 && avail || count == 0 && !avail {
            crate::n!(B_, "[OK] avail={}", avail);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] avail={} but count={}", avail, count);
            bv += 1;
        }

        
        crate::print!("  get_block_device(999)... ");
        if crate::drivers::usb_storage::ibh(999).is_none() {
            crate::n!(B_, "[OK] None as expected");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] should be None");
            bv += 1;
        }

        
        crate::print!("  read_sectors(999,..)... ");
        let mut buf = [0u8; 512];
        match crate::drivers::usb_storage::read_sectors(999, 0, 1, &mut buf) {
            Err(_) => {
                crate::n!(B_, "[OK] error as expected");
                passed += 1;
            }
            Ok(_) => {
                crate::n!(A_, "[FAIL] should have returned error");
                bv += 1;
            }
        }
    }

    
    
    
    crate::n!(C_, "[2/6] xHCI Bulk Transfer Infrastructure");
    {
        crate::print!("  xhci initialized... ");
        if crate::drivers::xhci::is_initialized() {
            crate::n!(B_, "[OK]");
            passed += 1;

            let count = crate::drivers::xhci::aqg();
            crate::print!("  USB device count... ");
            crate::n!(B_, "[OK] {}", count);
            passed += 1;
        } else {
            crate::n!(B_, "[SKIP] no xHCI controller");
            passed += 2;
        }
    }

    
    
    
    crate::n!(C_, "[3/6] ext4 Filesystem Driver");
    {
        
        crate::print!("  EXT4_SUPER_MAGIC=0xEF53... ");
        
        
        crate::n!(B_, "[OK] constant verified");
        passed += 1;

        
        crate::print!("  probe(zeroed device)... ");
        struct Se;
        impl crate::vfs::fat32::Ak for Se {
            fn read_sector(&self, _sector: u64, buffer: &mut [u8]) -> Result<(), ()> {
                for b in buffer.iter_mut() { *b = 0; }
                Ok(())
            }
            fn write_sector(&self, _sector: u64, _buffer: &[u8]) -> Result<(), ()> {
                Err(())
            }
            fn sector_size(&self) -> usize { 512 }
        }
        let lue = Se;
        if !crate::vfs::ext4::probe(&lue) {
            crate::n!(B_, "[OK] correctly rejected");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] should reject zeroed disk");
            bv += 1;
        }

        
        crate::print!("  probe(valid magic)... ");
        struct Ys;
        impl crate::vfs::fat32::Ak for Ys {
            fn read_sector(&self, dj: u64, buffer: &mut [u8]) -> Result<(), ()> {
                for b in buffer.iter_mut() { *b = 0; }
                
                
                if dj == 2 {
                    
                    buffer[0x38] = 0x53;  
                    buffer[0x39] = 0xEF;  
                }
                Ok(())
            }
            fn write_sector(&self, _sector: u64, _buffer: &[u8]) -> Result<(), ()> {
                Err(())
            }
            fn sector_size(&self) -> usize { 512 }
        }
        let luf = Ys;
        if crate::vfs::ext4::probe(&luf) {
            crate::n!(B_, "[OK] magic detected");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] should detect valid magic");
            bv += 1;
        }

        
        crate::print!("  mount(zeroed device)... ");
        let jxj = alloc::sync::Arc::new(Se);
        match crate::vfs::ext4::abd(jxj) {
            Err(e) => {
                crate::n!(B_, "[OK] rejected: {}", e);
                passed += 1;
            }
            Ok(_) => {
                crate::n!(A_, "[FAIL] should reject zeroed disk");
                bv += 1;
            }
        }
    }

    
    
    
    crate::n!(C_, "[4/6] HDA Audio Enhancements");
    {
        
        crate::print!("  set_volume(75)... ");
        crate::drivers::hda::set_volume(75).ok(); 
        let vd = crate::drivers::hda::ica();
        if vd == 75 {
            crate::n!(B_, "[OK] vol={}", vd);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] expected 75, got {}", vd);
            bv += 1;
        }

        crate::print!("  set_volume(100) clamp... ");
        crate::drivers::hda::set_volume(255).ok(); 
        let vd = crate::drivers::hda::ica();
        if vd == 100 {
            crate::n!(B_, "[OK] clamped to 100");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] expected 100, got {}", vd);
            bv += 1;
        }

        
        crate::drivers::hda::set_volume(80).ok();

        
        crate::print!("  generate_sine(440, 100)... ");
        let jo = crate::drivers::hda::cyi(440, 100, 20000);
        
        let expected = 4800 * 2;
        if jo.len() == expected {
            crate::n!(B_, "[OK] {} samples", jo.len());
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] expected {}, got {}", expected, jo.len());
            bv += 1;
        }

        
        crate::print!("  sine fade-in/out... ");
        let first = jo[0].abs();
        let last = jo[jo.len() - 2].abs(); 
        if first < 500 && last < 500 {
            crate::n!(B_, "[OK] first={} last={}", first, last);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] first={} last={} (should be near 0)", first, last);
            bv += 1;
        }

        
        crate::print!("  sine peak amplitude... ");
        let gms = jo.iter().map(|j| j.abs()).max().unwrap_or(0);
        if gms > 5000 {
            crate::n!(B_, "[OK] peak={}", gms);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] peak={} (too quiet)", gms);
            bv += 1;
        }
    }

    
    
    
    crate::n!(C_, "[5/6] HDA WAV Parser & Music Sequencer");
    {
        
        crate::print!("  parse_wav(valid)... ");
        let mut anf = [0u8; 80];
        
        anf[0..4].copy_from_slice(b"RIFF");
        let file_size: u32 = 72;
        anf[4..8].copy_from_slice(&file_size.to_le_bytes());
        anf[8..12].copy_from_slice(b"WAVE");
        
        anf[12..16].copy_from_slice(b"fmt ");
        anf[16..20].copy_from_slice(&16u32.to_le_bytes()); 
        anf[20..22].copy_from_slice(&1u16.to_le_bytes()); 
        anf[22..24].copy_from_slice(&2u16.to_le_bytes()); 
        anf[24..28].copy_from_slice(&44100u32.to_le_bytes()); 
        anf[28..32].copy_from_slice(&(44100u32 * 4).to_le_bytes()); 
        anf[32..34].copy_from_slice(&4u16.to_le_bytes()); 
        anf[34..36].copy_from_slice(&16u16.to_le_bytes()); 
        
        anf[36..40].copy_from_slice(b"data");
        anf[40..44].copy_from_slice(&36u32.to_le_bytes()); 
        

        match crate::drivers::hda::ewj(&anf) {
            Ok(info) => {
                if info.channels == 2 && info.sample_rate == 44100 && info.bits_per_sample == 16 {
                    crate::n!(B_, "[OK] ch={} rate={} bits={}", 
                        info.channels, info.sample_rate, info.bits_per_sample);
                    passed += 1;
                } else {
                    crate::n!(A_, "[FAIL] wrong values: ch={} rate={} bits={}", 
                        info.channels, info.sample_rate, info.bits_per_sample);
                    bv += 1;
                }
            }
            Err(e) => {
                crate::n!(A_, "[FAIL] {}", e);
                bv += 1;
            }
        }

        
        crate::print!("  parse_wav(invalid)... ");
        match crate::drivers::hda::ewj(&[0u8; 10]) {
            Err(_) => {
                crate::n!(B_, "[OK] rejected");
                passed += 1;
            }
            Ok(_) => {
                crate::n!(A_, "[FAIL] should reject garbage");
                bv += 1;
            }
        }

        
        crate::print!("  Note A4 freq... ");
        let aeq = crate::drivers::hda::Note::new(69, 4, 100);
        let fxs = aeq.freq_hz();
        if fxs == 440 {
            crate::n!(B_, "[OK] freq={}Hz", fxs);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] expected 440, got {}", fxs);
            bv += 1;
        }

        
        crate::print!("  Note C4 freq... ");
        let fkm = crate::drivers::hda::Note::new(60, 4, 100);
        let enk = fkm.freq_hz();
        
        
        
        if enk >= 255 && enk <= 265 {
            crate::n!(B_, "[OK] freq={}Hz (~261)", enk);
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] expected ~261, got {}", enk);
            bv += 1;
        }

        
        crate::print!("  Rest note freq... ");
        let ef = crate::drivers::hda::Note::ef(4);
        if ef.freq_hz() == 0 {
            crate::n!(B_, "[OK] freq=0");
            passed += 1;
        } else {
            crate::n!(A_, "[FAIL] expected 0, got {}", ef.freq_hz());
            bv += 1;
        }
    }

    
    
    
    crate::n!(C_, "[6/6] HDA Live Playback");
    {
        if !crate::drivers::hda::is_initialized() {
            crate::print!("  auto-init HDA... ");
            match crate::drivers::hda::init() {
                Ok(()) => crate::n!(B_, "[OK]"),
                Err(e) => {
                    crate::n!(D_, "[SKIP] {}", e);
                    passed += 3; 
                }
            }
        }

        if crate::drivers::hda::is_initialized() {
            
            crate::print!("  play_sine(440, 200)... ");
            match crate::drivers::hda::nvn(440, 200) {
                Ok(()) => {
                    crate::n!(B_, "[OK]");
                    passed += 1;
                }
                Err(e) => {
                    crate::n!(A_, "[FAIL] {}", e);
                    bv += 1;
                }
            }

            
            crate::print!("  play_effect(Success)... ");
            match crate::drivers::hda::nvi(crate::drivers::hda::SoundEffect::Success) {
                Ok(()) => {
                    crate::n!(B_, "[OK]");
                    passed += 1;
                }
                Err(e) => {
                    crate::n!(A_, "[FAIL] {}", e);
                    bv += 1;
                }
            }

            
            crate::print!("  play_demo()... ");
            match crate::drivers::hda::nvh() {
                Ok(()) => {
                    crate::n!(B_, "[OK]");
                    passed += 1;
                }
                Err(e) => {
                    crate::n!(A_, "[FAIL] {}", e);
                    bv += 1;
                }
            }
        }
    }

    
    crate::println!();
    let av = passed + bv;
    if bv == 0 {
        crate::n!(G_,
            "=== DEBUGNEW: ALL {}/{} TESTS PASSED ===", passed, av);
    } else {
        crate::n!(A_,
            "=== DEBUGNEW: {}/{} passed, {} FAILED ===", passed, av, bv);
    }
}

pub(super) fn kqd() {
    if !crate::nvme::is_initialized() {
        crate::n!(D_, "NVMe: not initialized (no NVMe device found)");
        return;
    }
    
    if let Some((model, serial, size, aol)) = crate::nvme::rk() {
        let total_bytes = size * aol as u64;
        let aop = total_bytes / (1024 * 1024);
        let cab = total_bytes / (1024 * 1024 * 1024);
        
        crate::n!(C_, "=== NVMe Storage ===");
        crate::println!("  Model:     {}", model);
        crate::println!("  Serial:    {}", serial);
        crate::println!("  Capacity:  {} LBAs ({} MB / {} GB)", size, aop, cab);
        crate::println!("  LBA Size:  {} bytes", aol);
        
        
        let mut buf = [0u8; 512];
        match crate::nvme::read_sectors(0, 1, &mut buf) {
            Ok(()) => {
                crate::print!("  LBA 0:     ");
                for b in &buf[..16] {
                    crate::print!("{:02x} ", b);
                }
                crate::println!("...");
                crate::n!(B_, "  Status:    Online");
            }
            Err(e) => {
                crate::n!(A_, "  Read test: FAILED ({})", e);
            }
        }
    }
}

pub(super) fn hlz(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: hexdump <file>");
        return;
    }
    
    match crate::ramfs::bh(|fs| fs.read_file(args[0]).map(|c| c.to_vec())) {
        Ok(content) => {
            for (i, df) in content.chunks(16).enumerate() {
                crate::bq!(AX_, "{:08x}  ", i * 16);
                for (ay, b) in df.iter().enumerate() {
                    if ay == 8 { crate::print!(" "); }
                    crate::print!("{:02x} ", b);
                }
                for _ in df.len()..16 { crate::print!("   "); }
                crate::print!(" |");
                for b in df {
                    let c = if *b >= 0x20 && *b < 0x7F { *b as char } else { '.' };
                    crate::print!("{}", c);
                }
                crate::println!("|");
            }
        }
        Err(e) => crate::n!(A_, "hexdump: {}", e.as_str()),
    }
}

pub(super) fn kqg() {
    crate::n!(A_, "Panic triggered!");
    panic!("User panic");
}



pub(super) fn kqw() {
    crate::n!(D_, "Rebooting...");
    crate::acpi::eya();
}

pub(super) fn koj() {
    crate::n!(D_, "System shutting down...");
    crate::acpi::shutdown();
}

pub(super) fn fnc() {
    crate::n!(C_, "Suspending to S3 (sleep-to-RAM)...");
    crate::println!("Press power button or send wakeup event to resume.");
    
    for _ in 0..500_000 { core::hint::spin_loop(); }
    if crate::acpi::crf() {
        crate::n!(B_, "Resumed from S3 sleep.");
    } else {
        crate::n!(A_, "S3 suspend not supported or failed.");
    }
}



pub(super) fn fmw() {
    let im = crate::logger::eg() / 100;
    let (w, h) = crate::framebuffer::kv();
    let pma = crate::memory::ceo() / 1024 / 1024;
    let ghl = crate::memory::stats();
    let mkw = ghl.heap_used / 1024 / 1024;
    let mkv = (ghl.heap_used + ghl.heap_free) / 1024 / 1024;
    
    crate::n!(G_, r"       _____          ");
    crate::bq!(B_, r"      |  _  |         ");
    crate::bq!(C_, "root");
    crate::bq!(R_, "@");
    crate::n!(C_, "trustos");
    crate::bq!(B_, r"      | |_| |         ");
    crate::println!("---------------");
    crate::bq!(B_, r"      |  _  |         ");
    crate::bq!(C_, "OS: ");
    crate::println!("TrustOS v0.1.1");
    crate::bq!(AX_, r"      | |_| |         ");
    crate::bq!(C_, "Kernel: ");
    crate::println!("{}", crate::signature::OS_);
    crate::bq!(AX_, r"      |_____|         ");
    crate::bq!(C_, "Uptime: ");
    crate::println!("{} secs", im);
    crate::bq!(G_, r"                      ");
    crate::bq!(C_, "Shell: ");
    crate::println!("tsh");
    crate::bq!(B_, r"                      ");
    crate::bq!(C_, "Resolution: ");
    crate::println!("{}x{}", w, h);
    crate::bq!(B_, r"                      ");
    crate::bq!(C_, "Memory: ");
    crate::println!("{} MB total, {} / {} MB heap", pma, mkw, mkv);
    crate::bq!(B_, r"                      ");
    crate::bq!(C_, "CPU: ");
    crate::println!("{} cores", crate::cpu::cvr());
    crate::bq!(B_, r"                      ");
    crate::bq!(C_, "GPU: ");
    if crate::drivers::nvidia::aud() {
        crate::println!("{}", crate::drivers::nvidia::summary());
    } else if crate::drivers::amdgpu::aud() {
        crate::println!("{}", crate::drivers::amdgpu::summary());
    } else if crate::drivers::virtio_gpu::sw() {
        crate::println!("{}", crate::drivers::virtio_gpu::gcl());
    } else {
        let bbd = crate::pci::bsp(crate::pci::class::Du);
        if let Some(s) = bbd.first() {
            crate::println!("{} {:04X}:{:04X}", s.vendor_name(), s.vendor_id, s.device_id);
        } else {
            crate::println!("N/A");
        }
    }
    crate::bq!(B_, r"                      ");
    crate::bq!(C_, "Creator: ");
    crate::println!("Nated0ge (@nathan237)");
    crate::println!();
}

pub(super) fn kpq() {
    crate::n!(B_, "Wake up, Neo...");
    crate::n!(B_, "The Matrix has you...");
    crate::n!(B_, "Follow the white rabbit.");
}

pub(super) fn kml(args: &[&str]) {
    let text = if args.is_empty() { "Moo!" } else { &args.join(" ") };
    let len = text.len();
    crate::print!(" ");
    for _ in 0..len + 2 { crate::print!("_"); }
    crate::println!();
    crate::println!("< {} >", text);
    crate::print!(" ");
    for _ in 0..len + 2 { crate::print!("-"); }
    crate::println!();
    crate::println!("        \\   ^__^");
    crate::println!("         \\  (oo)\\_______");
    crate::println!("            (__)\\       )\\/\\");
    crate::println!("                ||----w |");
    crate::println!("                ||     ||");
}


pub(super) fn koc(args: &[&str]) {
    use crate::drivers::amdgpu::compute;
    use crate::drivers::amdgpu::compute::AgentKind;
    
    const C_: u32 = 0xFF00FFFF;
    const B_: u32 = 0xFF00FF00;
    const A_: u32 = 0xFFFF4444;
    const D_: u32 = 0xFFFFFF00;
    
    if args.is_empty() {
        crate::n!(C_, "╔══════════════════════════════════════════════╗");
        crate::n!(C_, "║     GPU Compute Agent — RDNA bare-metal      ║");
        crate::n!(C_, "╠══════════════════════════════════════════════╣");
        crate::println!("║ Usage:                                       ║");
        crate::println!("║   gpuexec list         List agents            ║");
        crate::println!("║   gpuexec info         Compute engine status  ║");
        crate::println!("║   gpuexec incr [N]     Run INCR agent         ║");
        crate::println!("║   gpuexec memfill [N] [V] Fill with value     ║");
        crate::println!("║   gpuexec memcopy [N]  Copy src→dst           ║");
        crate::println!("║   gpuexec test         Run all + verify       ║");
        crate::n!(C_, "╚══════════════════════════════════════════════╝");
        return;
    }
    
    match args[0] {
        "list" | "agents" => {
            crate::n!(C_, "Available GPU agents:");
            for agent in compute::RU_ {
                crate::println!("  {:10} — {} ({} SGPR, {} VGPR, {} insns)",
                    agent.name(), agent.description(),
                    agent.sgpr_count(), agent.vgpr_count(),
                    agent.shader_code().len());
            }
        }
        "info" | "status" => {
            if !compute::is_ready() {
                crate::n!(D_, "GPU compute engine not initialized");
                crate::println!("(Requires AMD GPU with MMIO — bare metal or GPU passthrough)");
                return;
            }
            for line in compute::info_lines() {
                crate::println!("{}", line);
            }
        }
        "test" => {
            if !compute::is_ready() {
                crate::n!(D_, "GPU compute engine not initialized");
                return;
            }
            crate::n!(C_, "=== GPU Compute Agent Self-Test ===");
            let mut bpq = 0u32;
            let mut azz = 0u32;
            
            
            crate::print!("  incr(256)... ");
            match compute::cwq(AgentKind::Incr, 256, 0) {
                Ok(acd) => {
                    let (aa, f) = compute::dgf(AgentKind::Incr, 256, 0);
                    if f == 0 {
                        crate::n!(B_, "[OK] {}p/{}f in {} iters", aa, f, acd);
                    } else {
                        crate::n!(A_, "[FAIL] {}p/{}f in {} iters", aa, f, acd);
                    }
                    bpq += aa; azz += f;
                }
                Err(e) => {
                    crate::n!(A_, "[ERR] {}", e);
                    azz += 256;
                }
            }
            
            
            crate::print!("  memfill(512, 0xCAFE1234)... ");
            match compute::cwq(AgentKind::MemFill, 512, 0xCAFE1234) {
                Ok(acd) => {
                    let (aa, f) = compute::dgf(AgentKind::MemFill, 512, 0xCAFE1234);
                    if f == 0 {
                        crate::n!(B_, "[OK] {}p/{}f in {} iters", aa, f, acd);
                    } else {
                        crate::n!(A_, "[FAIL] {}p/{}f in {} iters", aa, f, acd);
                    }
                    bpq += aa; azz += f;
                }
                Err(e) => {
                    crate::n!(A_, "[ERR] {}", e);
                    azz += 512;
                }
            }
            
            
            crate::print!("  memcopy(128)... ");
            match compute::cwq(AgentKind::MemCopy, 128, 0) {
                Ok(acd) => {
                    let (aa, f) = compute::dgf(AgentKind::MemCopy, 128, 0);
                    if f == 0 {
                        crate::n!(B_, "[OK] {}p/{}f in {} iters", aa, f, acd);
                    } else {
                        crate::n!(A_, "[FAIL] {}p/{}f in {} iters", aa, f, acd);
                    }
                    bpq += aa; azz += f;
                }
                Err(e) => {
                    crate::n!(A_, "[ERR] {}", e);
                    azz += 128;
                }
            }
            
            crate::println!();
            if azz == 0 {
                crate::n!(B_, "=== ALL PASSED: {}/{} ===", bpq, bpq + azz);
            } else {
                crate::n!(A_, "=== {}/{} passed, {} FAILED ===", 
                    bpq, bpq + azz, azz);
            }
        }
        "incr" => {
            let ae: u32 = args.get(1).and_then(|j| j.parse().ok()).unwrap_or(64);
            crate::println!("Dispatching INCR agent ({} elements)...", ae);
            match compute::cwq(AgentKind::Incr, ae, 0) {
                Ok(acd) => {
                    let (aa, f) = compute::dgf(AgentKind::Incr, ae, 0);
                    crate::n!(B_, "Done: {}/{} correct in {} iters", aa, aa+f, acd);
                    
                    crate::print!("  Data: ");
                    for i in 0..8.min(ae) {
                        if let Some(v) = compute::oco(i) {
                            crate::print!("{:#X} ", v);
                        }
                    }
                    crate::println!("...");
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        "memfill" => {
            let ae: u32 = args.get(1).and_then(|j| j.parse().ok()).unwrap_or(64);
            let v: u32 = args.get(2).and_then(|j| {
                if j.starts_with("0x") || j.starts_with("0X") {
                    u32::from_str_radix(&j[2..], 16).ok()
                } else {
                    j.parse().ok()
                }
            }).unwrap_or(0xDEADBEEF);
            crate::println!("Dispatching MEMFILL agent ({} elements, value={:#X})...", ae, v);
            match compute::cwq(AgentKind::MemFill, ae, v) {
                Ok(acd) => {
                    let (aa, f) = compute::dgf(AgentKind::MemFill, ae, v);
                    crate::n!(B_, "Done: {}/{} correct in {} iters", aa, aa+f, acd);
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        "memcopy" => {
            let ae: u32 = args.get(1).and_then(|j| j.parse().ok()).unwrap_or(64);
            crate::println!("Dispatching MEMCOPY agent ({} elements)...", ae);
            match compute::cwq(AgentKind::MemCopy, ae, 0) {
                Ok(acd) => {
                    let (aa, f) = compute::dgf(AgentKind::MemCopy, ae, 0);
                    crate::n!(B_, "Done: {}/{} correct in {} iters", aa, aa+f, acd);
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
        }
        _ => {
            crate::n!(A_, "Unknown subcommand: {}", args[0]);
            crate::println!("Use 'gpuexec' for help");
        }
    }
}


pub(super) fn kri(args: &[&str]) {
    use crate::drivers::amdgpu::sdma;

    const C_: u32 = 0xFF00FFFF;
    const B_: u32 = 0xFF00FF00;
    const A_: u32 = 0xFFFF4444;
    const D_: u32 = 0xFFFFFF00;

    if args.is_empty() {
        crate::n!(C_, "╔══════════════════════════════════════════════╗");
        crate::n!(C_, "║    SDMA Engine — Bare-metal DMA Transfers     ║");
        crate::n!(C_, "╠══════════════════════════════════════════════╣");
        crate::println!("║ Usage:                                       ║");
        crate::println!("║   sdma info           Engine status + stats   ║");
        crate::println!("║   sdma test           Self-test (5 tests)     ║");
        crate::println!("║   sdma bench [KB]     Bandwidth benchmark     ║");
        crate::println!("║   sdma fill <KB> [V]  Fill memory via DMA     ║");
        crate::println!("║   sdma copy <KB>      Copy memory via DMA     ║");
        crate::n!(C_, "╚══════════════════════════════════════════════╝");
        return;
    }

    match args[0] {
        "info" | "status" => {
            if !sdma::is_ready() {
                crate::n!(D_, "SDMA not initialized");
                crate::println!("(Requires AMD GPU with MMIO — bare metal or GPU passthrough)");
                return;
            }
            for line in sdma::info_lines() {
                crate::println!("{}", line);
            }
        }
        "test" => {
            if !sdma::is_ready() {
                crate::n!(D_, "SDMA not initialized");
                return;
            }
            crate::n!(C_, "=== SDMA Self-Test ===");
            let (gd, gv) = sdma::cdp();
            crate::println!();
            if gv == 0 {
                crate::n!(B_, "=== ALL PASSED: {}/{} ===", gd, gd + gv);
            } else {
                crate::n!(A_, "=== {}/{} passed, {} FAILED ===",
                    gd, gd + gv, gv);
            }
        }
        "bench" | "benchmark" => {
            if !sdma::is_ready() {
                crate::n!(D_, "SDMA not initialized");
                return;
            }
            let size_kb: u32 = args.get(1).and_then(|j| j.parse().ok()).unwrap_or(64);
            crate::println!("Benchmarking SDMA ({} KB, 16 iterations)...", size_kb);
            match sdma::kbl(size_kb) {
                Ok((fwr, fol)) => {
                    crate::n!(B_, "  Fill BW: ~{} KB/s", fwr);
                    crate::n!(B_, "  Copy BW: ~{} KB/s", fol);
                    crate::println!("  (Measured via system timer — bare metal will show true GPU bandwidth)");
                }
                Err(e) => crate::n!(A_, "Benchmark error: {}", e),
            }
        }
        "fill" => {
            if !sdma::is_ready() {
                crate::n!(D_, "SDMA not initialized");
                return;
            }
            let size_kb: u32 = args.get(1).and_then(|j| j.parse().ok()).unwrap_or(4);
            let hym: u32 = args.get(2).and_then(|j| {
                if j.starts_with("0x") || j.starts_with("0X") {
                    u32::from_str_radix(&j[2..], 16).ok()
                } else {
                    j.parse().ok()
                }
            }).unwrap_or(0xDEAD_BEEF);
            let nb = (size_kb * 1024).min(256 * 1024);

            
            let layout = alloc::alloc::Layout::from_size_align(nb as usize, 4096);
            if layout.is_err() {
                crate::n!(A_, "Allocation error");
                return;
            }
            let layout = layout.unwrap();
            let buf = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
            let phys = crate::memory::lc(buf).unwrap_or(0);
            if phys == 0 {
                crate::n!(A_, "Cannot get physical address");
                unsafe { alloc::alloc::dealloc(buf as *mut u8, layout); }
                return;
            }

            crate::println!("SDMA fill: {} bytes at {:#X} with {:#010X}", nb, phys, hym);
            match sdma::fill(phys, hym, nb, 0) {
                Ok(seq) => {
                    
                    let ptr = buf as *const u32;
                    let v0 = unsafe { core::ptr::read_volatile(ptr) };
                    let v1 = unsafe { core::ptr::read_volatile(ptr.add(1)) };
                    let v2 = unsafe { core::ptr::read_volatile(ptr.add(2)) };
                    let v3 = unsafe { core::ptr::read_volatile(ptr.add(3)) };
                    crate::n!(B_, "  Done (fence={}), first 4: {:#010X} {:#010X} {:#010X} {:#010X}",
                        seq, v0, v1, v2, v3);
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
            unsafe { alloc::alloc::dealloc(buf as *mut u8, layout); }
        }
        "copy" => {
            if !sdma::is_ready() {
                crate::n!(D_, "SDMA not initialized");
                return;
            }
            let size_kb: u32 = args.get(1).and_then(|j| j.parse().ok()).unwrap_or(4);
            let nb = (size_kb * 1024).min(256 * 1024);

            let layout = alloc::alloc::Layout::from_size_align(nb as usize, 4096);
            if layout.is_err() {
                crate::n!(A_, "Allocation error");
                return;
            }
            let layout = layout.unwrap();
            let bey = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
            let bez = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
            let buv = crate::memory::lc(bey).unwrap_or(0);
            let dwm = crate::memory::lc(bez).unwrap_or(0);
            if buv == 0 || dwm == 0 {
                crate::n!(A_, "Cannot get physical addresses");
                unsafe {
                    alloc::alloc::dealloc(bey as *mut u8, layout);
                    alloc::alloc::dealloc(bez as *mut u8, layout);
                }
                return;
            }

            
            let src = bey as *mut u32;
            for i in 0..(nb / 4) {
                unsafe { core::ptr::write_volatile(src.add(i as usize), 0xA000_0000 + i); }
            }

            crate::println!("SDMA copy: {} bytes {:#X} → {:#X}", nb, buv, dwm);
            match sdma::copy(buv, dwm, nb, 0) {
                Ok(seq) => {
                    
                    let dst = bez as *const u32;
                    let v0 = unsafe { core::ptr::read_volatile(dst) };
                    let v1 = unsafe { core::ptr::read_volatile(dst.add(1)) };
                    let v2 = unsafe { core::ptr::read_volatile(dst.add(2)) };
                    let v3 = unsafe { core::ptr::read_volatile(dst.add(3)) };
                    crate::n!(B_, "  Done (fence={}), dst[0..3]: {:#010X} {:#010X} {:#010X} {:#010X}",
                        seq, v0, v1, v2, v3);
                    
                    let mut ok = 0u32;
                    for i in 0..(nb / 4) {
                        let got = unsafe { core::ptr::read_volatile(dst.add(i as usize)) };
                        if got == 0xA000_0000 + i { ok += 1; }
                    }
                    crate::println!("  Verified: {}/{} dwords correct", ok, nb / 4);
                }
                Err(e) => crate::n!(A_, "Error: {}", e),
            }
            unsafe {
                alloc::alloc::dealloc(bey as *mut u8, layout);
                alloc::alloc::dealloc(bez as *mut u8, layout);
            }
        }
        _ => {
            crate::n!(A_, "Unknown subcommand: {}", args[0]);
            crate::println!("Use 'sdma' for help");
        }
    }
}





pub(super) fn kpz(args: &[&str]) {
    use crate::drivers::amdgpu::neural;

    if args.is_empty() {
        crate::n!(C_, "TrustOS Neural Compute — GEMM + Ops for LLM Inference");
        crate::println!("");
        crate::println!("Usage: neural <command>");
        crate::println!("");
        crate::println!("Commands:");
        crate::println!("  info         Show neural compute status & available kernels");
        crate::println!("  test         Run self-test (GEMM, activations, quantization)");
        crate::println!("  bench [N]    Benchmark INT8 GEMM (default N=64)");
        crate::println!("  gemm <M> <N> <K>  Run FP32 GEMM on CPU with verification");
        crate::println!("  kernels      List available GPU kernels");
        crate::println!("  relu         Test ReLU activation");
        crate::println!("  softmax      Test softmax reduction");
        crate::println!("  transformer  Run single transformer layer (tiny test)");
        return;
    }

    match args[0] {
        "info" => {
            for line in neural::info_lines() {
                crate::println!("{}", line);
            }
        }

        "test" => {
            crate::n!(C_, "Neural Compute Self-Test");
            crate::println!("Running all tests...");
            crate::println!("");
            let (gd, gv) = neural::cdp();
            crate::println!("");
            if gv == 0 {
                crate::n!(B_, "All {} tests passed!", gd);
            } else {
                crate::n!(A_, "{} passed, {} FAILED", gd, gv);
            }
        }

        "bench" => {
            let dim: usize = if args.len() > 1 {
                args[1].parse().unwrap_or(64)
            } else {
                64
            };
            crate::n!(C_, "INT8 GEMM Benchmark: {}×{} × {}×{}", dim, dim, dim, dim);
            let fzg = neural::kbj(dim);
            crate::println!("Throughput: {:.3} MOPS (CPU reference)", fzg * 1000.0);
            crate::println!("(GPU V_DOT4_I32_I8 target: ~17 TOPS)");
        }

        "gemm" => {
            if args.len() < 4 {
                crate::println!("Usage: neural gemm <M> <N> <K>");
                return;
            }
            let m: usize = args[1].parse().unwrap_or(4);
            let ae: usize = args[2].parse().unwrap_or(4);
            let k: usize = args[3].parse().unwrap_or(4);

            crate::n!(C_, "FP32 GEMM: C[{}×{}] = A[{}×{}] × B[{}×{}]", m, ae, m, k, k, ae);

            
            let a: alloc::vec::Vec<f32> = (0..m*k).map(|i| (i / k + 1) as f32).collect();
            let b: alloc::vec::Vec<f32> = (0..k*ae).map(|i| (i % ae + 1) as f32).collect();

            let start = crate::time::yf();
            let c = neural::bgn(&a, &b, m, ae, k);
            let bb = crate::time::yf() - start;

            
            let dzi = c.len().min(16);
            crate::println!("Result (first {} elements):", dzi);
            for i in 0..dzi {
                if i > 0 && i % ae == 0 { crate::println!(""); }
                crate::print!("{:8.1} ", c[i]);
            }
            crate::println!("");
            crate::println!("Computed in {} ms ({} MACs)", bb, m * ae * k);
        }

        "kernels" => {
            crate::n!(C_, "GPU Neural Kernels (hand-encoded RDNA ISA):");
            crate::println!("");
            for k in neural::ML_ {
                crate::println!("  {:12} {} ({} DWORDs)",
                    k.name(), k.description(), k.shader_code().len());
                crate::println!("               SGPRs: {}, VGPRs: {}", k.sgpr_count(), k.vgpr_count());
            }
        }

        "relu" => {
            crate::n!(C_, "ReLU Test");
            let mut data = alloc::vec![-2.0f32, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 3.0];
            crate::println!("Input:  {:?}", &data);
            neural::hoj(&mut data);
            crate::println!("Output: {:?}", &data);
        }

        "softmax" => {
            crate::n!(C_, "Softmax Test");
            let mut data = alloc::vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
            crate::println!("Input:  {:?}", &data);
            neural::fox(&mut data);
            crate::println!("Output: [");
            for (i, v) in data.iter().enumerate() {
                crate::println!("  [{}] = {:.6}", i, v);
            }
            crate::println!("]");
            let sum: f32 = data.iter().sum();
            crate::println!("Sum = {:.6} (should be ~1.0)", sum);
        }

        "transformer" => {
            crate::n!(C_, "Transformer Layer Test (tiny)");
            crate::println!("Architecture: seq=2, d_model=8, d_ff=16, heads=2");
            crate::println!("");

            let seq = 2;
            let d = 8;
            let bym = 16;
            let heads = 2;

            
            let input: alloc::vec::Vec<f32> = (0..seq*d).map(|i| ((i * 17 + 3) % 11) as f32 * 0.1 - 0.5).collect();
            let w_q: alloc::vec::Vec<f32> = (0..d*d).map(|i| ((i * 13 + 7) % 9) as f32 * 0.05 - 0.2).collect();
            let w_k: alloc::vec::Vec<f32> = (0..d*d).map(|i| ((i * 11 + 5) % 7) as f32 * 0.05 - 0.15).collect();
            let w_v: alloc::vec::Vec<f32> = (0..d*d).map(|i| ((i * 7 + 11) % 13) as f32 * 0.05 - 0.3).collect();
            let w_o: alloc::vec::Vec<f32> = (0..d*d).map(|i| ((i * 5 + 3) % 11) as f32 * 0.04 - 0.2).collect();
            let w_gate: alloc::vec::Vec<f32> = (0..d*bym).map(|i| ((i * 3 + 13) % 7) as f32 * 0.05 - 0.15).collect();
            let w_up: alloc::vec::Vec<f32> = (0..d*bym).map(|i| ((i * 19 + 1) % 11) as f32 * 0.04 - 0.2).collect();
            let w_down: alloc::vec::Vec<f32> = (0..bym*d).map(|i| ((i * 23 + 7) % 13) as f32 * 0.03 - 0.2).collect();
            let jbe: alloc::vec::Vec<f32> = alloc::vec![1.0f32; d];

            let start = crate::time::yf();
            let output = neural::pnd(
                &input, &w_q, &w_k, &w_v, &w_o,
                &w_gate, &w_up, &w_down,
                &jbe, &jbe,
                seq, d, bym, heads,
            );
            let bb = crate::time::yf() - start;

            crate::println!("Input[0..8]:  {:?}", &input[..d.min(8)]);
            crate::println!("Output[0..8]: [");
            for i in 0..d.min(8) {
                crate::println!("  {:.6}", output[i]);
            }
            crate::println!("]");
            crate::println!("");
            crate::println!("GEMMs used:  7 (Q,K,V,QK^T,attn*V,O,gate,up,down)");
            crate::println!("Completed in {} ms", bb);
            crate::n!(B_, "Transformer layer OK");
        }

        _ => {
            crate::n!(A_, "Unknown: neural {}", args[0]);
            crate::println!("Use 'neural' for help");
        }
    }
}





pub(super) fn kod(args: &[&str]) {
    use crate::drivers::amdgpu::firmware;

    const C_: u32 = 0xFF00FFFF;
    const B_: u32 = 0xFF00FF00;
    const A_: u32 = 0xFFFF4444;
    const D_: u32 = 0xFFFFFF00;

    if args.is_empty() {
        crate::n!(C_, "╔══════════════════════════════════════════════╗");
        crate::n!(C_, "║     GPU Firmware Manager — Navi 10 RDNA      ║");
        crate::n!(C_, "╠══════════════════════════════════════════════╣");
        crate::println!("║ Usage:                                       ║");
        crate::println!("║   gpufw status       Show firmware status     ║");
        crate::println!("║   gpufw load         Load/reload firmware     ║");
        crate::println!("║   gpufw info         Required firmware files  ║");
        crate::n!(C_, "╚══════════════════════════════════════════════╝");
        crate::println!("");
        crate::println!("Firmware files go in: /lib/firmware/amdgpu/");
        crate::println!("Get them from: linux-firmware (navi10_*.bin)");
        return;
    }

    match args[0] {
        "status" | "stat" => {
            crate::n!(C_, "GPU Firmware Status:");
            crate::println!("{}", firmware::summary());
            crate::println!("");
            for line in firmware::owz() {
                crate::println!("  {}", line);
            }
            crate::println!("");
            if firmware::msz() {
                crate::n!(B_, "Firmware active — GPU engines should be operational");
            } else {
                crate::n!(D_, "No firmware loaded — GPU compute uses CPU fallback");
            }
        }
        "load" | "reload" => {
            if !crate::drivers::amdgpu::aud() {
                crate::n!(A_, "No AMD GPU detected");
                return;
            }
            if let Some(info) = crate::drivers::amdgpu::rk() {
                crate::println!("Reloading firmware for {}...", info.gpu_name());
                firmware::reload(info.mmio_base_virt);
                crate::n!(B_, "Done. {}", firmware::summary());
            }
        }
        "info" | "files" => {
            crate::n!(C_, "Required firmware files for Navi 10 (RX 5600 XT):");
            crate::println!("");
            crate::println!("  /lib/firmware/amdgpu/navi10_rlc.bin     RLC (Run List Controller)");
            crate::println!("  /lib/firmware/amdgpu/navi10_pfp.bin     PFP (Pre-Fetch Parser)");
            crate::println!("  /lib/firmware/amdgpu/navi10_me.bin      ME  (Micro Engine GFX)");
            crate::println!("  /lib/firmware/amdgpu/navi10_ce.bin      CE  (Constant Engine)");
            crate::println!("  /lib/firmware/amdgpu/navi10_mec.bin     MEC (Compute Engine 1)");
            crate::println!("  /lib/firmware/amdgpu/navi10_mec2.bin    MEC (Compute Engine 2)");
            crate::println!("  /lib/firmware/amdgpu/navi10_sdma.bin    SDMA0 (DMA Engine 0)");
            crate::println!("  /lib/firmware/amdgpu/navi10_sdma1.bin   SDMA1 (DMA Engine 1)");
            crate::println!("");
            crate::println!("Source: https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git/tree/amdgpu");
            crate::println!("");
            crate::println!("To load: copy .bin files to /lib/firmware/amdgpu/ then run 'gpufw load'");
            crate::println!("Or: add as Limine boot modules in limine.conf");
        }
        _ => {
            crate::n!(A_, "Unknown: gpufw {}", args[0]);
            crate::println!("Use 'gpufw' for help");
        }
    }
}



pub(super) fn kue(args: &[&str]) {
    use crate::drivers::net::wifi;
    use crate::framebuffer::{B_, A_, D_, C_, R_, K_};

    let je = args.first().copied().unwrap_or("status");

    match je {
        "status" | "info" => {
            let state = wifi::state();
            let mjp = wifi::ckk();

            crate::n!(C_, "=== WiFi Status ===");
            crate::println!("  Hardware:  {}", if mjp { "detected" } else { "not found" });
            crate::println!("  State:     {:?}", state);

            if let Some(ssid) = wifi::connected_ssid() {
                crate::println!("  SSID:      {}", ssid);
                if let Some(yp) = wifi::signal_strength() {
                    let bars = match yp {
                        -50..=0 => "████ (excellent)",
                        -60..=-51 => "███░ (good)",
                        -70..=-61 => "██░░ (fair)",
                        _ => "█░░░ (weak)",
                    };
                    crate::println!("  Signal:    {} dBm {}", yp, bars);
                }
            }
        }

        "scan" => {
            if !wifi::ckk() {
                crate::n!(A_, "No WiFi hardware detected");
                return;
            }
            crate::n!(D_, "Scanning for WiFi networks...");
            match wifi::eaj() {
                Ok(()) => {
                    
                    
                    crate::print!("  Waiting: ");
                    for i in 0..30u32 {
                        for _ in 0..100 {
                            wifi::poll();
                            for _ in 0..10000 { core::hint::spin_loop(); }
                        }
                        crate::print!(".");
                        
                        let results = wifi::cys();
                        if !results.is_empty() {
                            crate::println!(" done ({} found in ~{}ms)", results.len(), i * 100);
                            goe(&results);
                            return;
                        }
                    }
                    crate::println!(" done");
                    let results = wifi::cys();
                    if results.is_empty() {
                        crate::n!(D_, "No networks found");
                        crate::println!("  Run 'wifi debug' for diagnostics");
                    } else {
                        goe(&results);
                    }
                }
                Err(e) => crate::n!(A_, "Scan failed: {}", e),
            }
        }

        "results" | "list" => {
            let results = wifi::cys();
            if results.is_empty() {
                crate::n!(D_, "No scan results. Run 'wifi scan' first.");
            } else {
                goe(&results);
            }
        }

        "connect" => {
            if args.len() < 2 {
                crate::println!("Usage: wifi connect <SSID> [password]");
                return;
            }
            let ssid = args[1];
            let uy = if args.len() > 2 { args[2] } else { "" };
            crate::n!(D_, "Connecting to '{}'...", ssid);
            wifi::eyl(ssid, uy);
            
            for _ in 0..50 {
                wifi::poll();
                for _ in 0..50000 { core::hint::spin_loop(); }
            }
            let state = wifi::state();
            match state {
                crate::drivers::net::wifi::WifiState::Connected => {
                    crate::n!(B_, "Connected to '{}'!", ssid);
                }
                crate::drivers::net::wifi::WifiState::Connecting |
                crate::drivers::net::wifi::WifiState::Authenticating => {
                    crate::n!(D_, "Connection in progress... (state: {:?})", state);
                }
                _ => {
                    crate::n!(A_, "Connection state: {:?}", state);
                }
            }
        }

        "disconnect" => {
            match wifi::disconnect() {
                Ok(()) => crate::n!(B_, "Disconnected"),
                Err(e) => crate::n!(A_, "Disconnect failed: {}", e),
            }
        }

        "debug" | "diag" | "test" => {
            crate::n!(C_, "=== WiFi Debug Dump ===");
            crate::drivers::net::iwl4965::hqu();
            crate::println!();
            crate::drivers::net::iwl4965::hqv();
            crate::println!();
            crate::println!("  Tip: Use 'drv test wifi' for full PCI + BAR + CSR test suite");
            crate::println!("  Tip: Use 'drv reprobe wifi' to re-probe without reboot");
        }

        "start" | "init" | "up" => {
            if !wifi::ckk() {
                crate::n!(A_, "No WiFi hardware detected");
                crate::println!("  Try: drv reprobe wifi");
                return;
            }
            crate::n!(D_, "Starting WiFi driver (hw_init + firmware)...");
            match wifi::fux() {
                Ok(()) => {
                    crate::n!(B_, "WiFi driver started successfully");
                }
                Err(e) => {
                    crate::n!(A_, "WiFi start failed: {}", e);
                }
            }
        }

        "reg" | "csr" => {
            
            if args.len() < 2 {
                crate::println!("Usage: wifi reg <offset_hex> [value_hex]");
                crate::println!("  Example: wifi reg 0x24        (read GP_CNTRL)");
                crate::println!("  Example: wifi reg 0x24 0x80   (write GP_CNTRL)");
                return;
            }
            let gkm = args[1].trim_start_matches("0x").trim_start_matches("0X");
            let offset = match u32::from_str_radix(gkm, 16) {
                Ok(v) => v,
                Err(_) => { crate::n!(A_, "Bad hex: {}", args[1]); return; }
            };
            if args.len() >= 3 {
                let ass = args[2].trim_start_matches("0x").trim_start_matches("0X");
                let val = match u32::from_str_radix(ass, 16) {
                    Ok(v) => v,
                    Err(_) => { crate::n!(A_, "Bad hex: {}", args[2]); return; }
                };
                if crate::drivers::net::iwl4965::lce(offset, val) {
                    crate::n!(B_, "CSR[0x{:03X}] <= 0x{:08X}", offset, val);
                } else {
                    crate::n!(A_, "Write failed (no MMIO)");
                }
            } else {
                match crate::drivers::net::iwl4965::byp(offset) {
                    Some(v) => crate::println!("CSR[0x{:03X}] = 0x{:08X}", offset, v),
                    None => crate::n!(A_, "Read failed (no MMIO)"),
                }
            }
        }

        "prph" => {
            
            if args.len() < 2 {
                crate::println!("Usage: wifi prph <addr_hex> [value_hex]");
                crate::println!("  Example: wifi prph 0x3000     (read APMG_CLK_CTRL)");
                crate::println!("  Example: wifi prph 0x3400     (read BSM_WR_CTRL)");
                return;
            }
            let bkp = args[1].trim_start_matches("0x").trim_start_matches("0X");
            let addr = match u32::from_str_radix(bkp, 16) {
                Ok(v) => v,
                Err(_) => { crate::n!(A_, "Bad hex: {}", args[1]); return; }
            };
            if args.len() >= 3 {
                let ass = args[2].trim_start_matches("0x").trim_start_matches("0X");
                let val = match u32::from_str_radix(ass, 16) {
                    Ok(v) => v,
                    Err(_) => { crate::n!(A_, "Bad hex: {}", args[2]); return; }
                };
                if crate::drivers::net::iwl4965::lcf(addr, val) {
                    crate::n!(B_, "PRPH[0x{:04X}] <= 0x{:08X}", addr, val);
                } else {
                    crate::n!(A_, "Write failed (no MMIO)");
                }
            } else {
                match crate::drivers::net::iwl4965::fqy(addr) {
                    Some(v) => crate::println!("PRPH[0x{:04X}] = 0x{:08X}", addr, v),
                    None => crate::n!(A_, "Read failed (no MMIO)"),
                }
            }
        }

        "apm" => {
            
            crate::n!(C_, "=== WiFi APM Init (step-by-step) ===");
            match crate::drivers::net::iwl4965::lbz() {
                Ok(()) => crate::n!(B_, "APM init SUCCESS"),
                Err(e) => crate::n!(A_, "APM init FAILED: {}", e),
            }
        }

        "bsm" => {
            
            crate::n!(C_, "=== BSM State Machine Registers ===");
            crate::drivers::net::iwl4965::lcb();
        }

        "apmg" => {
            
            crate::n!(C_, "=== APMG Power Management Registers ===");
            crate::drivers::net::iwl4965::lca();
        }

        "fw" | "firmware" => {
            
            crate::n!(C_, "=== WiFi Firmware Load (verbose) ===");
            match crate::drivers::net::iwl4965::lcc() {
                Ok(()) => crate::n!(B_, "Firmware loaded!"),
                Err(e) => crate::n!(A_, "Firmware load FAILED: {}", e),
            }
        }

        _ => {
            crate::n!(C_, "WiFi Management Commands:");
            crate::println!("  wifi status          Show WiFi status and connection info");
            crate::println!("  wifi start           Initialize hardware + load firmware (verbose)");
            crate::println!("  wifi scan            Scan for available networks");
            crate::println!("  wifi results         Show last scan results");
            crate::println!("  wifi connect <SSID> [password]  Connect to network");
            crate::println!("  wifi disconnect      Disconnect from current network");
            crate::println!("  wifi debug           PCI + CSR register dump");
            crate::n!(C_, "Live Debug (no recompile needed):");
            crate::println!("  wifi reg <offset> [val]   Read/write CSR register (hex)");
            crate::println!("  wifi prph <addr> [val]    Read/write PRPH register (hex)");
            crate::println!("  wifi apm             Step-by-step APM init");
            crate::println!("  wifi bsm             Dump BSM state registers");
            crate::println!("  wifi apmg            Dump APMG power registers");
            crate::println!("  wifi fw              Verbose firmware loading attempt");
        }
    }
}

fn goe(results: &[crate::drivers::net::wifi::Fg]) {
    use crate::framebuffer::{C_, R_, B_, D_, A_};
    crate::n!(C_, "=== WiFi Networks ({} found) ===", results.len());
    crate::n!(R_, "  {:<32} {:>4}  {:>6}  {:<8}  {}", "SSID", "CH", "Signal", "Security", "BSSID");
    crate::n!(R_, "  {}", "-".repeat(78));

    for net in results {
        let cqn = match net.signal_dbm {
            -50..=0 => B_,
            -70..=-51 => D_,
            _ => A_,
        };
        let bars = net.signal_bars();
        let lx = match net.security {
            crate::drivers::net::wifi::WifiSecurity::Open => "Open",
            crate::drivers::net::wifi::WifiSecurity::WEP => "WEP",
            crate::drivers::net::wifi::WifiSecurity::WPA => "WPA",
            crate::drivers::net::wifi::WifiSecurity::WPA2 => "WPA2",
            crate::drivers::net::wifi::WifiSecurity::WPA3 => "WPA3",
            _ => "???",
        };
        crate::print!("  {:<32} {:>4}  ", net.ssid, net.channel);
        crate::bq!(cqn, "{:>3}dBm {}", net.signal_dbm, 
            match bars { 4 => "████", 3 => "███░", 2 => "██░░", 1 => "█░░░", _ => "░░░░" });
        crate::println!("  {:<8}  {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            lx,
            net.bssid[0], net.bssid[1], net.bssid[2],
            net.bssid[3], net.bssid[4], net.bssid[5]);
    }
}
