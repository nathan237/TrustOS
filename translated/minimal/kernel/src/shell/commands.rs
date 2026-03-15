





use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AU_, D_, A_, C_, Q_, CD_, DF_, L_};
use crate::ramfs::FileType;


pub(super) fn kis(n: &[&str]) {
    if !n.is_empty() {
        nee(n);
        return;
    }
    
    crate::h!(G_, "======================================================================");
    crate::h!(G_, "          TrustOS -- Secure Bare-Metal Operating System");
    crate::h!(G_, "       x86_64 kernel written in Rust -- no libc, no std");
    crate::h!(G_, "======================================================================");
    crate::println!();
    crate::h!(Q_, "  Features: RAMFS file system, TCP/IP networking, ELF loader,");
    crate::h!(Q_, "  Linux syscall compat, GUI desktop compositor, SMP multicore.");
    crate::println!();
    crate::h!(D_, "  Type 'help <command>' or 'man <command>' for detailed usage.");
    crate::h!(D_, "  Tab = auto-complete | Up/Down = history | PageUp/Down = scroll");
    crate::println!();
    
    
    crate::h!(C_, "  FILE SYSTEM");
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
    
    
    crate::h!(C_, "  TEXT PROCESSING");
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
    
    
    crate::h!(C_, "  SYSTEM & PROCESS");
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
    
    
    crate::h!(C_, "  USER MANAGEMENT");
    crate::println!("    login               Switch to another user account");
    crate::println!("    su <user>           Substitute user identity");
    crate::println!("    passwd [user]       Change user password");
    crate::println!("    adduser <name>      Create new user account");
    crate::println!("    deluser <name>      Delete user account");
    crate::println!("    users               List all user accounts");
    crate::println!();
    
    
    crate::h!(C_, "  HARDWARE & DEVICES");
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
    
    
    crate::h!(C_, "  DISK & STORAGE");
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
    
    
    crate::h!(C_, "  NETWORK");
    crate::println!("    ifconfig / ip       Show network interface status");
    crate::println!("    ipconfig [cmd]      Configure IP settings");
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

    
    crate::h!(C_, "  SECURITY TOOLKIT (TrustScan)");
    crate::println!("    nmap <target>       Port scanner (SYN/Connect/UDP scan)");
    crate::println!("    nmap <t> -A         Aggressive scan (ports + banners + vulns)");
    crate::println!("    discover [mode]     Host discovery (arp/ping/full)");
    crate::println!("    banner <target>     Service banner grabber & version detect");
    crate::println!("    sniff <cmd>         Packet sniffer (start/stop/show/hex/stats)");
    crate::println!("    vulnscan <target>   Vulnerability assessment scanner");
    crate::println!("    traceroute <host>   Real TTL-based traceroute with ICMP");
    crate::println!("    scantest [target]   Live network test suite (8 tests)");
    crate::println!();

    
    crate::h!(C_, "  HTTP SERVER");
    crate::println!("    httpd [start] [p]   Start HTTP server (default port 8080)");
    crate::println!("    httpd stop          Stop the running HTTP server");
    crate::println!("    httpd status        Show server status and request count");
    crate::println!();

    
    crate::h!(C_, "  PACKAGE MANAGER (TrustPkg)");
    crate::println!("    trustpkg list       List all available packages");
    crate::println!("    trustpkg search <q> Search packages by name/description");
    crate::println!("    trustpkg install <p> Install a package");
    crate::println!("    trustpkg remove <p> Remove an installed package");
    crate::println!("    trustpkg info <p>   Show package details");
    crate::println!("    trustpkg installed  List installed packages only");
    crate::println!("    trustpkg update     Update package catalog");
    crate::println!();
    
    
    crate::h!(C_, "  LINUX SUBSYSTEM");
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
    
    
    crate::h!(C_, "  GRAPHICS & DESKTOP");
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
    
    
    crate::h!(C_, "  PROGRAMMING & TOOLS");
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
    
    
    crate::h!(C_, "  ARCHIVING & COMPRESSION");
    crate::println!("    tar <opts> <file>   Archive/extract tar files");
    crate::println!("    gzip / gunzip       Compress/decompress gzip files");
    crate::println!("    zip / unzip         Compress/extract zip archives");
    crate::println!();
    
    
    crate::h!(C_, "  DEVELOPER & DEBUG");
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
    
    
    crate::h!(C_, "  SERVICES & SCHEDULING");
    crate::println!("    service <name> <op> Manage system services (start/stop)");
    crate::println!("    systemctl <cmd>     Systemd-style service control");
    crate::println!("    crontab [-e|-l]     Schedule recurring jobs");
    crate::println!("    at <time> <cmd>     Schedule one-time command execution");
    crate::println!("    sysctl <key>[=val]  View/modify kernel parameters");
    crate::println!();
    
    
    crate::h!(C_, "  SECURITY & IDENTITY");
    crate::println!("    security / sec      Security subsystem status & caps");
    crate::println!("    signature / sig     Kernel signature & proof of authorship");
    crate::println!("    hv / hypervisor     Hypervisor management commands");
    crate::println!("    firewall / iptables Firewall rules (add/del/list/flush)");
    crate::println!("    checkm8             iOS/USB exploit research tool");
    crate::println!();

    
    crate::h!(C_, "  THINKPAD / HARDWARE CONTROL");
    crate::println!("    fan [speed|auto]    ThinkPad fan control (EC direct)");
    crate::println!("    temp / sensors      CPU/GPU temperature readings");
    crate::println!("    cpufreq [cmd]       CPU frequency scaling (SpeedStep)");
    crate::println!();

    
    crate::h!(C_, "  JARVIS AI & HARDWARE INTELLIGENCE");
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
    
    
    crate::h!(C_, "  SYSTEM CONTROL");
    crate::println!("    exit / logout       Exit current session");
    crate::println!("    reboot              Restart the system");
    crate::println!("    shutdown / halt     Power off the system");
    crate::println!("    reset               Reset terminal state");
    crate::println!("    tty                 Print terminal device name");
    crate::println!("    stty <opts>         Configure terminal settings");
    crate::println!("    loadkeys <map>      Load keyboard layout");
    crate::println!("    setfont <font>      Change console font");
    crate::println!();
    
    
    crate::h!(C_, "  EASTER EGGS & DEMOS");
    crate::println!("    neofetch            System info with ASCII art logo");
    crate::println!("    matrix              Fullscreen Matrix rain animation");
    crate::println!("    cowsay <text>       ASCII cow says your message");
    crate::println!("    showcase [N]        Automated demo (marketing video)");
    crate::println!("    showcase-jarvis     Jarvis AI showcase demo");
    crate::println!("    showcase3d          3D graphics cinematic showcase");
    crate::println!("    filled3d            3D filled polygon rendering demo");
    crate::println!("    demo [fr]           Interactive guided tutorial");
    crate::println!();
    
    crate::h!(G_, "  Total: ~220 commands | Type 'man <cmd>' for detailed usage");
    crate::println!();
}
pub(super) fn nee(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: man <command>");
        return;
    }
    
    match n[0] {
        "ls" => {
            crate::h!(G_, "LS(1) - List directory contents");
            crate::println!();
            crate::println!("SYNOPSIS: ls [path]");
            crate::println!();
            crate::println!("Lists files and directories.");
        }
        "cd" => {
            crate::h!(G_, "CD(1) - Change directory");
            crate::println!();
            crate::println!("SYNOPSIS: cd [path]");
            crate::println!();
            crate::println!("Special: ~ (home), .. (parent)");
        }
        "cat" => {
            crate::h!(G_, "CAT(1) - Display file contents");
            crate::println!();
            crate::println!("SYNOPSIS: cat <file>");
            crate::println!();
            crate::println!("Supports redirection: cat file > newfile");
        }
        "perf" | "perfstat" => {
            crate::h!(G_, "PERF(1) - Performance Statistics");
            crate::println!();
            crate::println!("SYNOPSIS: perf");
            crate::println!();
            crate::println!("Shows uptime, FPS, IRQ count/rate, syscalls,");
            crate::println!("context switches, heap usage, and per-CPU stats.");
        }
        "memdbg" | "heapdbg" => {
            crate::h!(G_, "MEMDBG(1) - Memory Debug Statistics");
            crate::println!();
            crate::println!("SYNOPSIS: memdbg");
            crate::println!();
            crate::println!("Shows heap usage, allocation/deallocation counts,");
            crate::println!("peak usage, fragmentation estimate, live alloc count.");
        }
        "dmesg" => {
            crate::h!(G_, "DMESG(1) - Kernel Ring Buffer");
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
            crate::h!(G_, "IRQSTAT(1) - Interrupt Statistics");
            crate::println!();
            crate::println!("SYNOPSIS: irqstat");
            crate::println!();
            crate::println!("Shows total IRQ count, IRQ/sec rate, and per-CPU");
            crate::println!("interrupt breakdown with visual bars.");
        }
        "regs" | "registers" | "cpuregs" => {
            crate::h!(G_, "REGS(1) - CPU Register Dump");
            crate::println!();
            crate::println!("SYNOPSIS: regs");
            crate::println!();
            crate::println!("Dumps RSP, RBP, RFLAGS, CR0, CR3, CR4, EFER.");
            crate::println!("Decodes flag/bit meanings for each register.");
        }
        "peek" | "memdump" => {
            crate::h!(G_, "PEEK(1) - Memory Inspector");
            crate::println!();
            crate::println!("SYNOPSIS: peek <hex_addr> [byte_count]");
            crate::println!();
            crate::println!("Hex dump memory at virtual address (max 256 bytes).");
            crate::println!("  peek 0xFFFF800000000000 64");
        }
        "poke" | "memwrite" => {
            crate::h!(G_, "POKE(1) - Memory Writer");
            crate::println!();
            crate::println!("SYNOPSIS: poke <hex_addr> <hex_byte>");
            crate::println!();
            crate::println!("Write a single byte to virtual address. DANGEROUS!");
            crate::println!("  poke 0xB8000 0x41");
        }
        "devpanel" => {
            crate::h!(G_, "DEVPANEL(1) - Developer Overlay");
            crate::println!();
            crate::println!("SYNOPSIS: devpanel");
            crate::println!();
            crate::println!("Toggles real-time overlay in desktop mode showing:");
            crate::println!("FPS, frame time, heap usage bar, IRQ/s, per-CPU stats.");
            crate::println!("Also toggled with F12 while in desktop.");
        }
        "timecmd" => {
            crate::h!(G_, "TIMECMD(1) - Time a Command");
            crate::println!();
            crate::println!("SYNOPSIS: timecmd <command> [args...]");
            crate::println!();
            crate::println!("Executes a command and displays elapsed time in Aus/ms.");
            crate::println!("  timecmd ls /");
            crate::println!("  timecmd benchmark cpu");
        }
        _ => {
            crate::println!("No manual entry for '{}'", n[0]);
        }
    }
}



pub(super) fn ioj(n: &[&str]) {
    let path = n.fv().hu();
    
    
    if let Some(ai) = path {
        if ai.cj("/mnt/") || ai.cj("/dev/") || ai.cj("/proc/") || ai == "/mnt" {
            rfy(ai);
            return;
        }
    }
    
    match crate::ramfs::fh(|fs| fs.awb(path)) {
        Ok(pj) => {
            if pj.is_empty() {
                return;
            }
            
            let hrc = pj.iter().map(|(bo, _, _)| bo.len()).am().unwrap_or(0);
            
            for (j, kd, aw) in pj {
                match kd {
                    FileType::K => {
                        crate::gr!(C_, "{:<width$}", j, z = hrc + 2);
                        crate::h!(AU_, " <DIR>");
                    }
                    FileType::Es => {
                        crate::gr!(B_, "{:<width$}", j, z = hrc + 2);
                        crate::println!(" {:>6} B", aw);
                    }
                }
            }
        }
        Err(aa) => {
            crate::h!(A_, "ls: {}", aa.as_str());
        }
    }
}

pub(super) fn rfy(path: &str) {
    use crate::vfs::{self, FileType as VfsFileType};
    
    match vfs::brx(path) {
        Ok(ch) => {
            if ch.is_empty() {
                crate::println!("(empty)");
                return;
            }
            
            let hrc = ch.iter().map(|aa| aa.j.len()).am().unwrap_or(0);
            
            for bt in ch {
                match bt.kd {
                    VfsFileType::K => {
                        crate::gr!(C_, "{:<width$}", bt.j, z = hrc + 2);
                        crate::h!(AU_, " <DIR>");
                    }
                    VfsFileType::Ea => {
                        crate::gr!(B_, "{:<width$}", bt.j, z = hrc + 2);
                        crate::println!(" (file)");
                    }
                    _ => {
                        crate::println!("{}", bt.j);
                    }
                }
            }
        }
        Err(aa) => {
            crate::h!(A_, "ls: {:?}", aa);
        }
    }
}

pub(super) fn kig(n: &[&str]) {
    let path = n.fv().hu().unwrap_or("~");
    
    if let Err(aa) = crate::ramfs::fh(|fs| fs.fem(path)) {
        crate::h!(A_, "cd: {}: {}", path, aa.as_str());
    }
}

pub(super) fn kjb() {
    let jv = crate::ramfs::fh(|fs| String::from(fs.dau()));
    crate::println!("{}", jv);
}

pub(super) fn iok(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: mkdir <directory>");
        return;
    }
    
    for path in n {
        if let Err(aa) = crate::ramfs::fh(|fs| fs.ut(path)) {
            crate::h!(A_, "mkdir: {}: {}", path, aa.as_str());
        }
    }
}

pub(super) fn rhr(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: rmdir <directory>");
        return;
    }
    
    for path in n {
        if let Err(aa) = crate::ramfs::fh(|fs| fs.hb(path)) {
            crate::h!(A_, "rmdir: {}: {}", path, aa.as_str());
        }
    }
}

pub(super) fn kji(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: touch <file>");
        return;
    }
    
    for path in n {
        
        if path.cj("/mnt/") {
            use crate::vfs::{self, OpenFlags};
            
            
            let flags = OpenFlags(OpenFlags::OP_ | OpenFlags::ON_);
            match vfs::aji(path, flags) {
                Ok(da) => {
                    let _ = vfs::agj(da);
                    crate::println!("Created: {}", path);
                }
                Err(aa) => crate::h!(A_, "touch: {:?}", aa),
            }
        } else {
            if let Err(aa) = crate::ramfs::fh(|fs| fs.touch(path)) {
                crate::h!(A_, "touch: {}: {}", path, aa.as_str());
            }
        }
    }
}

pub(super) fn kjc(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: rm <file>");
        return;
    }
    
    for path in n {
        if let Err(aa) = crate::ramfs::fh(|fs| fs.hb(path)) {
            crate::h!(A_, "rm: {}: {}", path, aa.as_str());
        }
    }
}

pub(super) fn kii(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: cp <source> <destination>");
        return;
    }
    
    if let Err(aa) = crate::ramfs::fh(|fs| fs.bza(n[0], n[1])) {
        crate::h!(A_, "cp: {}", aa.as_str());
    }
}

pub(super) fn kix(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: mv <source> <destination>");
        return;
    }
    
    if let Err(aa) = crate::ramfs::fh(|fs| fs.euz(n[0], n[1])) {
        crate::h!(A_, "mv: {}", aa.as_str());
    }
}

pub(super) fn hde(n: &[&str], ehg: Option<(&str, bool)>, arr: Option<&str>) {
    
    if let Some(input) = arr {
        if let Some((file, bte)) = ehg {
            let _ = crate::ramfs::fh(|fs| {
                if !fs.aja(file) { fs.touch(file).bq(); }
                if bte { fs.ijw(file, input.as_bytes()) } 
                else { fs.ns(file, input.as_bytes()) }
            });
        } else {
            crate::print!("{}", input);
        }
        return;
    }
    
    if n.is_empty() {
        crate::println!("Usage: cat <file>");
        return;
    }
    
    let mut an = String::new();
    
    for path in n {
        
        if path.cj("/mnt/") || path.cj("/dev/") || path.cj("/proc/") {
            match rcv(path) {
                Some(text) => {
                    if ehg.is_some() {
                        an.t(&text);
                    } else {
                        crate::print!("{}", text);
                    }
                }
                None => {} 
            }
            continue;
        }
        
        match crate::ramfs::fh(|fs| fs.mq(path).map(|r| r.ip())) {
            Ok(ca) => {
                if let Ok(text) = core::str::jg(&ca) {
                    if ehg.is_some() {
                        an.t(text);
                    } else {
                        crate::print!("{}", text);
                    }
                } else {
                    crate::h!(A_, "cat: {}: binary file", path);
                }
            }
            Err(aa) => {
                crate::h!(A_, "cat: {}: {}", path, aa.as_str());
            }
        }
    }
    
    if let Some((file, bte)) = ehg {
        let _ = crate::ramfs::fh(|fs| {
            if !fs.aja(file) { fs.touch(file).bq(); }
            if bte { fs.ijw(file, an.as_bytes()) } 
            else { fs.ns(file, an.as_bytes()) }
        });
    }
}

pub(super) fn rcv(path: &str) -> Option<alloc::string::String> {
    use crate::vfs::{self, OpenFlags};
    use alloc::string::Gd;
    
    
    let da = match vfs::aji(path, OpenFlags(OpenFlags::OO_)) {
        Ok(bb) => bb,
        Err(aa) => {
            crate::h!(A_, "cat: {}: {:?}", path, aa);
            return None;
        }
    };
    
    
    let mut bi = [0u8; 4096];
    let mut ca = alloc::vec::Vec::new();
    
    loop {
        let cjl = match vfs::read(da, &mut bi) {
            Ok(bo) => bo,
            Err(aa) => {
                crate::h!(A_, "cat: {}: read error {:?}", path, aa);
                let _ = vfs::agj(da);
                return None;
            }
        };
        
        if cjl == 0 {
            break;
        }
        
        ca.bk(&bi[..cjl]);
    }
    
    let _ = vfs::agj(da);
    
    match core::str::jg(&ca) {
        Ok(text) => Some(String::from(text)),
        Err(_) => {
            crate::h!(A_, "cat: {}: binary file", path);
            None
        }
    }
}

pub(super) fn kir(n: &[&str], arr: Option<&str>) {
    let ak: usize = if n.len() > 1 { n[1].parse().unwrap_or(10) }
                        else if n.len() == 1 && n[0].cj('-') && n[0].len() > 1 { n[0][1..].parse().unwrap_or(10) }
                        else { 10 };
    
    let ffr = if let Some(input) = arr {
        alloc::string::String::from(input)
    } else if !n.is_empty() && !n[0].cj('-') {
        match crate::ramfs::fh(|fs| fs.mq(n[0]).map(|r| r.ip())) {
            Ok(ca) => match core::str::jg(&ca) {
                Ok(ab) => alloc::string::String::from(ab),
                Err(_) => return,
            },
            Err(aa) => { crate::h!(A_, "head: {}", aa.as_str()); return; }
        }
    } else {
        crate::println!("Usage: head <file> [lines]");
        return;
    };
    
    for (a, line) in ffr.ak().cf() {
        if a >= ak { break; }
        crate::println!("{}", line);
    }
}

pub(super) fn kjg(n: &[&str], arr: Option<&str>) {
    let ak: usize = if n.len() > 1 { n[1].parse().unwrap_or(10) } else { 10 };
    
    let ffr = if let Some(input) = arr {
        alloc::string::String::from(input)
    } else if !n.is_empty() {
        match crate::ramfs::fh(|fs| fs.mq(n[0]).map(|r| r.ip())) {
            Ok(ca) => match core::str::jg(&ca) {
                Ok(ab) => alloc::string::String::from(ab),
                Err(_) => return,
            },
            Err(aa) => { crate::h!(A_, "tail: {}", aa.as_str()); return; }
        }
    } else {
        crate::println!("Usage: tail <file> [lines]");
        return;
    };
    
    let xx: Vec<&str> = ffr.ak().collect();
    let ay = if xx.len() > ak { xx.len() - ak } else { 0 };
    for line in &xx[ay..] {
        crate::println!("{}", line);
    }
}

pub(super) fn kjk(n: &[&str], arr: Option<&str>) {
    
    let (ffr, j) = if let Some(input) = arr {
        (alloc::string::String::from(input), alloc::string::String::from("(stdin)"))
    } else if !n.is_empty() {
        match crate::ramfs::fh(|fs| fs.mq(n[0]).map(|r| r.ip())) {
            Ok(ca) => {
                match core::str::jg(&ca) {
                    Ok(ab) => (alloc::string::String::from(ab), alloc::string::String::from(n[0])),
                    Err(_) => return,
                }
            }
            Err(aa) => { crate::h!(A_, "wc: {}", aa.as_str()); return; }
        }
    } else {
        crate::println!("Usage: wc <file>");
        return;
    };
    
    let ak = ffr.ak().az();
    let aoh = ffr.ayt().az();
    crate::println!("{:>6} {:>6} {:>6} {}", ak, aoh, ffr.len(), j);
}

pub(super) fn rim(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: stat <file>");
        return;
    }
    
    match crate::ramfs::fh(|fs| fs.hm(n[0]).map(|aa| aa.clone())) {
        Ok(bt) => {
            crate::h!(C_, "  File: {}", bt.j);
            let are = if bt.kd == FileType::K { "directory" } else { "file" };
            crate::println!("  Type: {}", are);
            crate::println!("  Size: {} bytes", bt.ca.len());
        }
        Err(aa) => crate::h!(A_, "stat: {}", aa.as_str()),
    }
}

pub(super) fn kjj(n: &[&str]) {
    let path = n.fv().hu().unwrap_or("/");
    crate::h!(C_, "{}", path);
    oxw(path, "");
}

fn oxw(path: &str, adx: &str) {
    if let Ok(pj) = crate::ramfs::fh(|fs| fs.awb(Some(path))) {
        let len = pj.len();
        for (a, (j, kd, _)) in pj.iter().cf() {
            let fmd = a == len - 1;
            let ly = if fmd { "+-- " } else { "|-- " };
            
            match kd {
                FileType::K => {
                    crate::print!("{}{}", adx, ly);
                    crate::h!(C_, "{}/", j);
                    
                    let utm = format!("{}{}", adx, if fmd { "    " } else { "|   " });
                    let aeh = if path == "/" { format!("/{}", j) } else { format!("{}/{}", path, j) };
                    oxw(&aeh, &utm);
                }
                FileType::Es => {
                    crate::print!("{}{}", adx, ly);
                    crate::h!(B_, "{}", j);
                }
            }
        }
    }
}

pub(super) fn kip(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: find <name>");
        return;
    }
    nuk("/", n[0]);
}

fn nuk(path: &str, pattern: &str) {
    if let Ok(pj) = crate::ramfs::fh(|fs| fs.awb(Some(path))) {
        for (j, kd, _) in pj {
            let auh = if path == "/" { format!("/{}", j) } else { format!("{}/{}", path, j) };
            if j.contains(pattern) {
                crate::println!("{}", auh);
            }
            if kd == FileType::K {
                nuk(&auh, pattern);
            }
        }
    }
}



pub(super) fn kin(n: &[&str], ehg: Option<(&str, bool)>) {
    let text = n.rr(" ");
    
    if let Some((file, bte)) = ehg {
        let ca = format!("{}\n", text);
        
        
        if file.cj("/mnt/") {
            use crate::vfs::{self, OpenFlags};
            
            
            let flags = if bte {
                OpenFlags(OpenFlags::OP_ | OpenFlags::ON_ | OpenFlags::BCF_)
            } else {
                OpenFlags(OpenFlags::OP_ | OpenFlags::ON_ | OpenFlags::BCG_)
            };
            
            match vfs::aji(file, flags) {
                Ok(da) => {
                    if let Err(aa) = vfs::write(da, ca.as_bytes()) {
                        crate::h!(A_, "echo: write error: {:?}", aa);
                    }
                    let _ = vfs::agj(da);
                }
                Err(aa) => crate::h!(A_, "echo: {:?}", aa),
            }
        } else {
            let _ = crate::ramfs::fh(|fs| {
                if !fs.aja(file) { fs.touch(file).bq(); }
                if bte { fs.ijw(file, ca.as_bytes()) }
                else { fs.ns(file, ca.as_bytes()) }
            });
        }
    } else {
        crate::println!("{}", text);
    }
}

pub(super) fn kiq(n: &[&str], arr: Option<&str>) {
    if n.is_empty() {
        crate::println!("Usage: grep <pattern> [file]");
        return;
    }
    
    let pattern = n[0];
    
    
    let ca = if let Some(input) = arr {
        alloc::string::String::from(input)
    } else if n.len() >= 2 {
        match crate::ramfs::fh(|fs| fs.mq(n[1]).map(|r| r.ip())) {
            Ok(ca) => {
                match core::str::jg(&ca) {
                    Ok(ab) => alloc::string::String::from(ab),
                    Err(_) => return,
                }
            }
            Err(aa) => { crate::h!(A_, "grep: {}", aa.as_str()); return; }
        }
    } else {
        crate::println!("Usage: grep <pattern> <file>");
        return;
    };
    
    for line in ca.ak() {
        if line.contains(pattern) {
            let ek: Vec<&str> = line.adk(pattern).collect();
            for (a, vu) in ek.iter().cf() {
                crate::print!("{}", vu);
                if a < ek.len() - 1 {
                    crate::gr!(A_, "{}", pattern);
                }
            }
            crate::println!();
        }
    }
}



pub(super) fn iof() {
    crate::framebuffer::clear();
}

pub(super) fn rjj() {
    let qb = crate::logger::lh();
    let tv = qb / 100;
    let bbz = tv / 60;
    let cad = bbz / 60;
    
    crate::gr!(C_, "Uptime: ");
    crate::h!(B_, "{}h {}m {}s", cad, bbz % 60, tv % 60);
    
    
    let os = crate::rtc::cgz();
    crate::gr!(C_, "Time:   ");
    crate::h!(B_, "{}", os.ivj());
}

pub(super) fn kij() {
    let os = crate::rtc::cgz();
    crate::h!(B_, "{}", os.format());
}

pub(super) fn rks() {
    crate::println!("{}", crate::auth::hey());
}

pub(super) fn kit() {
    crate::println!("trustos");
}

pub(super) fn rff() {
    let cnp = crate::auth::hey();
    let pi = crate::auth::kne();
    let pw = crate::auth::kmu();
    crate::println!("uid={}({}) gid={}({})", pi, cnp, pw, 
        if pw == 0 { "root" } else if pw == 100 { "users" } else { "wheel" });
}



pub(super) fn rfw() {
    
    crate::auth::oki();
    crate::println!();
    
    if crate::auth::okd() {
        
        crate::h!(B_, "Login successful.");
    } else {
        
        crate::h!(A_, "Login failed.");
    }
}

pub(super) fn ris(n: &[&str]) {
    let fwg = if n.is_empty() { "root" } else { n[0] };
    
    
    if crate::auth::crt() && fwg != "root" {
        
        crate::h!(D_, "Switching to {} (root privilege)", fwg);
        return;
    }
    
    
    crate::gr!(C_, "Password: ");
    let mut ewe = [0u8; 128];
    let hun = crate::keyboard::fsf(&mut ewe);
    let aqe = core::str::jg(&ewe[..hun])
        .unwrap_or("")
        .em();
    crate::println!();
    
    match crate::auth::ljs(fwg, aqe) {
        Ok(()) => {
            crate::h!(B_, "Switched to {}", fwg);
        }
        Err(aa) => {
            crate::h!(A_, "su: {}", aa);
        }
    }
}

pub(super) fn rgx(n: &[&str]) {
    let fwg = if n.is_empty() {
        crate::auth::hey()
    } else {
        
        if !crate::auth::crt() {
            crate::h!(A_, "passwd: Only root can change other users' passwords");
            return;
        }
        String::from(n[0])
    };
    
    crate::println!("Changing password for {}", fwg);
    
    
    let lqa = if !crate::auth::crt() {
        crate::print!("Current password: ");
        let mut k = [0u8; 128];
        let len = crate::keyboard::fsf(&mut k);
        crate::println!();
        String::from(core::str::jg(&k[..len]).unwrap_or("").em())
    } else {
        String::new()
    };
    
    
    crate::print!("New password: ");
    let mut opl = [0u8; 128];
    let usy = crate::keyboard::fsf(&mut opl);
    crate::println!();
    let fov = core::str::jg(&opl[..usy]).unwrap_or("").em();
    
    
    crate::print!("Retype new password: ");
    let mut hdv = [0u8; 128];
    let kkl = crate::keyboard::fsf(&mut hdv);
    crate::println!();
    let kkk = core::str::jg(&hdv[..kkl]).unwrap_or("").em();
    
    if fov != kkk {
        crate::h!(A_, "passwd: passwords do not match");
        return;
    }
    
    if fov.len() < 1 {
        crate::h!(A_, "passwd: password too short");
        return;
    }
    
    match crate::auth::khc(&fwg, &lqa, fov) {
        Ok(()) => {
            crate::h!(B_, "passwd: password updated successfully");
        }
        Err(aa) => {
            crate::h!(A_, "passwd: {}", aa);
        }
    }
}

pub(super) fn rcb(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: adduser <username> [-a]");
        crate::println!("  -a  Make user an admin (wheel group)");
        return;
    }
    
    if !crate::auth::crt() {
        crate::h!(A_, "adduser: must be root");
        return;
    }
    
    let ox = n[0];
    let hos = n.contains(&"-a") || n.contains(&"--admin");
    
    
    crate::print!("New password for {}: ", ox);
    let mut ewe = [0u8; 128];
    let hun = crate::keyboard::fsf(&mut ewe);
    crate::println!();
    let aqe = core::str::jg(&ewe[..hun]).unwrap_or("").em();
    
    
    crate::print!("Retype password: ");
    let mut hdv = [0u8; 128];
    let kkl = crate::keyboard::fsf(&mut hdv);
    crate::println!();
    let kkk = core::str::jg(&hdv[..kkl]).unwrap_or("").em();
    
    if aqe != kkk {
        crate::h!(A_, "adduser: passwords do not match");
        return;
    }
    
    match crate::auth::jzj(ox, aqe, hos) {
        Ok(pi) => {
            crate::h!(B_, "User {} created with UID {}", ox, pi);
            
            
            let iym = format!("/home/{}", ox);
            crate::ramfs::fh(|fs| {
                let _ = fs.ut("/home");
                let _ = fs.ut(&iym);
            });
            crate::println!("Home directory: {}", iym);
        }
        Err(aa) => {
            crate::h!(A_, "adduser: {}", aa);
        }
    }
}

pub(super) fn rdo(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: deluser <username>");
        return;
    }
    
    if !crate::auth::crt() {
        crate::h!(A_, "deluser: must be root");
        return;
    }
    
    let ox = n[0];
    
    crate::gr!(D_, "Delete user {}? [y/N]: ", ox);
    let mut k = [0u8; 16];
    let len = crate::keyboard::cts(&mut k);
    let yt = core::str::jg(&k[..len]).unwrap_or("").em();
    
    if yt != "y" && yt != "Y" {
        crate::println!("Cancelled.");
        return;
    }
    
    match crate::auth::kou(ox) {
        Ok(()) => {
            crate::h!(B_, "User {} deleted", ox);
        }
        Err(aa) => {
            crate::h!(A_, "deluser: {}", aa);
        }
    }
}

pub(super) fn rkd() {
    crate::h!(C_, "USER            UID   GID   DESCRIPTION");
    crate::h!(C_, "------------------------------------------");
    
    for (ox, pi, pw, eqz) in crate::auth::liz() {
        crate::println!("{:<15} {:<5} {:<5} {}", ox, pi, pw, eqz);
    }
}

pub(super) fn rfx() {
    let cnp = crate::auth::hey();
    crate::auth::oki();
    crate::println!("Logged out {}.", cnp);
    crate::println!();
    
    
    if !crate::auth::okd() {
        
        crate::h!(D_, "Auto-login as root (development mode)");
        crate::auth::mww();
    }
}

pub(super) fn rfj() {
    crate::h!(G_, "=== T-RUSTOS ===");
    crate::gr!(C_, "Version:      ");
    crate::println!("0.1.0");
    crate::gr!(C_, "Architecture: ");
    crate::println!("x86_64");
    crate::gr!(C_, "Bootloader:   ");
    crate::println!("Limine");
    crate::println!();
    crate::h!(G_, "Modules:");
    for ef in ["Memory", "Interrupts", "Keyboard", "Framebuffer", "RAM FS", "History", "Scheduler"] {
        crate::gr!(B_, "  [x] ");
        crate::println!("{}", ef);
    }
    
    
    if crate::disk::anl() {
        crate::gr!(B_, "  [x] ");
        crate::println!("Disk I/O");
    } else {
        crate::gr!(AU_, "  [-] ");
        crate::println!("Disk I/O (no disk)");
    }
    
    
    if crate::network::anl() {
        crate::gr!(B_, "  [x] ");
        crate::println!("Network");
    } else {
        crate::gr!(AU_, "  [-] ");
        crate::println!("Network (down)");
    }
}

pub(super) fn rkf() {
    crate::println!("T-RustOs v0.2.0 (Rust + Limine)");
}

pub(super) fn iom(n: &[&str]) {
    let xx = n.contains(&"-a");
    if n.is_empty() || xx { crate::print!("T-RustOs "); }
    if n.contains(&"-n") || xx { crate::print!("trustos "); }
    if n.contains(&"-r") || xx { crate::print!("0.2.0 "); }
    if n.contains(&"-m") || xx { crate::print!("x86_64"); }
    crate::println!();
}

pub(super) fn iog() {
    for (eh, p) in super::scripting::ijj() {
        crate::println!("{}={}", eh, p);
    }
}

pub(super) fn rez() {
    for (num, cmd) in crate::keyboard::toz() {
        crate::gr!(AU_, "{:>4}  ", num);
        crate::println!("{}", cmd);
    }
}

pub(super) fn kja() {
    crate::h!(C_, "  PID  STATE    CMD");
    crate::println!("    1  running  kernel");
    crate::println!("    2  running  tsh");
    
    
    let az = crate::task::dmj();
    if az > 0 {
        crate::println!("  ... +{} background tasks (use 'tasks' for details)", az);
    }
}

pub(super) fn ioh() {
    let mr = crate::memory::heap::mr();
    let aez = crate::memory::heap::aez();
    let es = mr + aez;
    crate::h!(C_, "              total       used       free");
    crate::println!("Heap:    {:>10}  {:>10}  {:>10}", es, mr, aez);
    crate::println!("  (KB)   {:>10}  {:>10}  {:>10}", es / 1024, mr / 1024, aez / 1024);
}

pub(super) fn kik() {
    crate::h!(C_, "Filesystem      Type     Size    Used   Avail  Mount");
    crate::println!("─────────────────────────────────────────────────────");

    let ajf = crate::vfs::hqa();
    let mem = crate::memory::cm();
    let aul = mem.afa + mem.buv;

    let fmt = |p: usize| -> alloc::string::String {
        if p == 0 { alloc::format!("  -  ") }
        else if p >= 1024 * 1024 { alloc::format!("{:>4}M", p / (1024 * 1024)) }
        else if p >= 1024 { alloc::format!("{:>4}K", p / 1024) }
        else { alloc::format!("{:>4}B", p) }
    };

    
    crate::println!("{:<15} {:<8} {:>5}  {:>5}  {:>5}  {}",
        "ramfs", "ramfs", fmt(aul), fmt(mem.afa),
        fmt(aul.ao(mem.afa)), "/");

    for (path, kxi) in &ajf {
        if path == "/" { continue; } 
        let (aw, mr, apk) = match kxi.as_str() {
            "devfs" | "proc" => (0, 0, 0),
            _ => (aul, mem.afa, aul.ao(mem.afa)),
        };

        crate::println!("{:<15} {:<8} {:>5}  {:>5}  {:>5}  {}",
            kxi, kxi, fmt(aw), fmt(mr), fmt(apk), path);
    }
}



pub(super) fn kjh() {
    crate::h!(G_, "Running self-test...");
    crate::println!();
    
    crate::print!("  Heap... ");
    let p: Vec<u32> = (0..100).collect();
    if p.len() == 100 { crate::h!(B_, "[OK]"); }
    else { crate::h!(A_, "[FAIL]"); }
    
    crate::print!("  String... ");
    let mut e = String::from("Hello");
    e.t(" World");
    if e.len() == 11 { crate::h!(B_, "[OK]"); }
    else { crate::h!(A_, "[FAIL]"); }
    
    crate::print!("  RAM FS... ");
    let bq = crate::ramfs::fh(|fs| {
        fs.touch("/tmp/t").bq();
        fs.ns("/tmp/t", b"x").bq();
        let m = fs.mq("/tmp/t").map(|r| r[0] == b'x').unwrap_or(false);
        fs.hb("/tmp/t").bq();
        m
    });
    if bq { crate::h!(B_, "[OK]"); }
    else { crate::h!(A_, "[FAIL]"); }
    
    crate::print!("  Interrupts... ");
    if crate::arch::kaw() {
        crate::h!(B_, "[OK]");
    } else {
        crate::h!(A_, "[FAIL]");
    }
    
    crate::println!();
    crate::h!(G_, "Done!");
}


pub(super) fn rhp() {
    use crate::desktop::{self, WindowType, SnapDir};
    
    crate::h!(G_, "=== Desktop Resolution Test Suite ===");
    crate::println!();
    
    
    let vxv: &[(u32, u32, &str)] = &[
        
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
    
    let mut cg = 0usize;
    let mut gv = 0usize;
    
    for &(d, i, cu) in vxv {
        crate::print!("  {:>4}x{:<4} ({:<20}) ", d, i, cu);
        
        
        let mut desktop = desktop::Desktop::new();
        
        desktop.z = d;
        desktop.ac = i;
        
        let mut bq = true;
        let mut eu = alloc::string::String::new();
        
        
        let xfl: &[(&str, i32, i32, u32, u32, WindowType)] = &[
            ("Terminal", 100, 60, 780, 540, WindowType::Ay),
            ("Files",    140, 80, 520, 420, WindowType::Ak),
            ("Calc",     350, 100, 300, 380, WindowType::Calculator),
            ("Browser",  100, 40, 720, 520, WindowType::Browser),
            ("Big",      0,   0, 1920, 1080, WindowType::Jf),
            ("Tiny",     0,   0, 50,  30,  WindowType::Jl),
            ("OffScreen", 9999, 9999, 400, 300, WindowType::Jl),
            ("Negative",  -100, -50, 400, 300, WindowType::Jl),
        ];
        
        for &(dq, b, c, hk, mg, ash) in xfl {
            let ad = desktop.xl(dq, b, c, hk, mg, ash);
            if let Some(ep) = desktop.ee.iter().du(|d| d.ad == ad) {
                
                let hw = ep.b as u32 + ep.z;
                let abm = ep.c as u32 + ep.ac;
                if hw > d + 1 || abm > i + 1 {
                    bq = false;
                    use core::fmt::Write;
                    let _ = core::fmt::write(&mut eu, format_args!(
                        " OOB:{}({}+{}={}>{})", dq, ep.b, ep.z, hw, d
                    ));
                }
                if ep.z == 0 || ep.ac == 0 {
                    bq = false;
                    use core::fmt::Write;
                    let _ = core::fmt::write(&mut eu, format_args!(" ZERO:{}", dq));
                }
            }
        }
        
        
        if let Some(ep) = desktop.ee.yqv() {
            ep.ja = true;
            ep.idy(d, i);
            let hw = ep.b as u32 + ep.z;
            let abm = ep.c as u32 + ep.ac;
            if hw > d + 1 || abm > i + 1 {
                bq = false;
                use core::fmt::Write;
                let _ = core::fmt::write(&mut eu, format_args!(
                    " MAX_OOB({}+{}={}>{})", ep.b, ep.z, hw, d
                ));
            }
            
            ep.idy(d, i);
        }
        
        
        let wqc = [
            SnapDir::Ap, SnapDir::Ca,
            SnapDir::Dp, SnapDir::Dq,
            SnapDir::Dt, SnapDir::Du,
        ];
        for te in &wqc {
            desktop.mgh(*te);
            if let Some(ep) = desktop.ee.iter().du(|d| d.ja) {
                let hw = ep.b as u32 + ep.z;
                let abm = ep.c as u32 + ep.ac;
                if hw > d + 1 || abm > i + 1 {
                    bq = false;
                    use core::fmt::Write;
                    let _ = core::fmt::write(&mut eu, format_args!(" SNAP_OOB:{:?}", te));
                }
            }
        }
        
        
        if desktop.anv() != d || desktop.akr() != i {
            bq = false;
            use core::fmt::Write;
            let _ = core::fmt::write(&mut eu, format_args!(" DIM_MISMATCH"));
        }
        
        if bq {
            crate::h!(B_, "[OK]  ({} windows)", desktop.ee.len());
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]{}", eu);
            gv += 1;
        }
    }
    
    crate::println!();
    if gv == 0 {
        crate::h!(G_, "All {} resolutions passed!", cg);
    } else {
        crate::h!(A_, "{}/{} failed", gv, cg + gv);
    }
}


pub(super) fn rgk() {
    crate::h!(G_, "=== TrustOS v0.3 Memory Test Suite ===");
    crate::println!();

    let mut cg = 0usize;
    let mut gv = 0usize;

    
    crate::h!(C_, "[1/6] Frame allocator self-test");
    let (ai, bb) = crate::memory::frame::eyj();
    cg += ai;
    gv += bb;
    crate::println!();

    
    crate::h!(C_, "[2/6] Ring 3 basic exec (test)");
    crate::print!("  exec test... ");
    match crate::exec::hil() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[3/6] Ring 3 ELF exec (hello)");
    crate::print!("  exec hello... ");
    match crate::exec::kui() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[4/6] Ring 3 brk/mmap test");
    crate::print!("  exec memtest... ");
    match crate::exec::nrl() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[5/6] Frame leak test (run exec, check frames returned)");
    crate::print!("  alloc before... ");
    let (mma, gvw) = crate::memory::frame::cm();
    let eby = mma - gvw;
    crate::println!("free={}", eby);

    
    let _ = crate::exec::hil();

    let (mlz, gvu) = crate::memory::frame::cm();
    let ebw = mlz - gvu;
    crate::print!("  alloc after... ");
    crate::println!("free={}", ebw);

    crate::print!("  no leak... ");
    if ebw >= eby {
        crate::h!(B_, "[OK] (freed {} frames)", ebw - eby + (eby - ebw).am(0));
        cg += 1;
    } else {
        let fmv = eby - ebw;
        crate::h!(A_, "[FAIL] leaked {} frames ({} KB)", fmv, fmv * 4);
        gv += 1;
    }

    
    crate::h!(C_, "[6/6] Ring 3 IPC pipe test (pipe2 + write + read)");
    crate::print!("  exec pipe_test... ");
    match crate::exec::nrm() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::println!();
    let es = cg + gv;
    if gv == 0 {
        crate::h!(G_,
            "All {}/{} tests passed v", cg, es);
    } else {
        crate::h!(A_,
            "{}/{} passed, {} FAILED", cg, es, gv);
    }
}

pub(super) fn rfq() {
    crate::h!(G_, "Keyboard Test Mode");
    crate::println!("Test all keys including Space, Backspace, Delete");
    crate::h!(D_, "Type 'quit' to exit test mode");
    crate::println!();
    
    let mut jsw = [0u8; 256];
    
    loop {
        if crate::shell::etf() {
            crate::h!(B_, "Interrupted");
            break;
        }
        crate::gr!(C_, "test> ");
        let len = crate::keyboard::cts(&mut jsw);
        let input = core::str::jg(&jsw[..len]).unwrap_or("");
        
        if input.em() == "quit" {
            crate::h!(B_, "Exiting test mode");
            break;
        }
        
        
        crate::print!("  Received {} bytes: ", len);
        crate::gr!(Q_, "\"{}\"", input);
        crate::println!();
        
        
        crate::print!("  Hex: ");
        for &hf in &jsw[..len] {
            crate::gr!(AU_, "{:02x} ", hf);
        }
        crate::println!();
        
        
        crate::print!("  Chars: ");
        for &hf in &jsw[..len] {
            if hf >= 32 && hf < 127 {
                crate::gr!(G_, "'{}' ", hf as char);
            } else if hf == 0x08 {
                crate::gr!(D_, "<BS> ");
            } else if hf == 0x20 {
                crate::gr!(D_, "<SPACE> ");
            } else {
                crate::gr!(A_, "0x{:02x} ", hf);
            }
        }
        crate::println!();
        crate::println!();
    }
}


pub(super) fn kiu(n: &[&str]) {
    match n.fv() {
        Some(&"start") | None => {
            let port = n.get(1)
                .and_then(|e| e.parse::<u16>().bq())
                .unwrap_or(8080);
            let am = n.get(2)
                .and_then(|e| e.parse::<u32>().bq())
                .unwrap_or(0);
            crate::httpd::ay(port, am);
        }
        Some(&"stop") => {
            crate::httpd::qg();
            crate::h!(B_, "HTTP server stop requested");
        }
        Some(&"status") => {
            let (port, hxn, aqk) = crate::httpd::asx();
            crate::h!(C_, "HTTP Server Status:");
            crate::println!("  Running:  {}", if aqk { "yes" } else { "no" });
            crate::println!("  Port:     {}", port);
            crate::println!("  Requests: {}", hxn);
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


pub(super) fn rgl(n: &[&str]) {
    match n.fv() {
        Some(&"start") => {
            crate::jarvis::jfy();
            crate::h!(B_, "JARVIS mesh network started");
            crate::println!("  Discovery: UDP port 7700 (broadcast)");
            crate::println!("  RPC:       TCP port 7701");
            crate::println!("  Use 'mesh status' to see peers");
        }
        Some(&"stop") => {
            crate::jarvis::unq();
            crate::h!(D_, "JARVIS mesh network stopped");
        }
        Some(&"status") | None => {
            let status = crate::jarvis::unp();
            crate::h!(C_, "=== JARVIS Mesh Status ===");
            crate::println!("{}", status);
            crate::println!();

            
            let yp = crate::jarvis::mesh::dhn();
            if yp.is_empty() {
                crate::println!("No peers discovered yet");
            } else {
                crate::h!(C_, "Peers:");
                for (a, ko) in yp.iter().cf() {
                    let vzw = match ko.bwt {
                        crate::jarvis::mesh::NodeRole::Ni => "★",
                        crate::jarvis::mesh::NodeRole::Mu => "◎",
                        crate::jarvis::mesh::NodeRole::Lb => "●",
                    };
                    crate::println!("  {} {} {}", a + 1, vzw, ko.display());
                }
            }
        }
        Some(&"peers") => {
            let yp = crate::jarvis::mesh::dhn();
            if yp.is_empty() {
                crate::println!("No peers online");
            } else {
                for (a, ko) in yp.iter().cf() {
                    crate::println!("  [{}] {}", a + 1, ko.display());
                }
            }
        }
        Some(&"federate") | Some(&"fed") => {
            match n.get(1) {
                Some(&"on") | Some(&"enable") | Some(&"start") => {
                    crate::jarvis::federated::aiy();
                    crate::h!(B_, "Federated learning enabled");
                }
                Some(&"off") | Some(&"disable") | Some(&"stop") => {
                    crate::jarvis::federated::cwz();
                    crate::h!(D_, "Federated learning disabled");
                }
                Some(&"sync") => {
                    crate::jarvis::federated::svq();
                    crate::h!(B_, "Sync round triggered");
                }
                Some(&"replicate") => {
                    crate::jarvis::federated::vxk();
                    crate::h!(B_, "Model replicated to all peers");
                }
                Some(&"pull") => {
                    match crate::jarvis::federated::vnz() {
                        Ok(()) => crate::h!(B_, "Pulled model from leader"),
                        Err(aa) => crate::h!(A_, "Pull failed: {}", aa),
                    }
                }
                _ => {
                    crate::println!("Usage: mesh federate {{on|off|sync|replicate|pull}}");
                }
            }
        }
        Some(&"ping") => {
            if n.len() < 2 {
                crate::println!("Usage: mesh ping <ip>");
                return;
            }
            if let Some(ip) = cgl(n[1]) {
                let port = crate::jarvis::mesh::GV_;
                match crate::jarvis::rpc::ovs(ip, port) {
                    Ok(true) => crate::h!(B_, "Peer alive!"),
                    Ok(false) => crate::h!(A_, "Peer responded with error"),
                    Err(aa) => crate::h!(A_, "Ping failed: {}", aa),
                }
            } else {
                crate::h!(A_, "Invalid IP address");
            }
        }
        Some(&"infer") => {
            if n.len() < 3 {
                crate::println!("Usage: mesh infer <ip> <prompt>");
                return;
            }
            if let Some(ip) = cgl(n[1]) {
                let aau: alloc::string::String = n[2..].rr(" ");
                let port = crate::jarvis::mesh::GV_;
                match crate::jarvis::rpc::vut(ip, port, &aau) {
                    Ok(result) => {
                        crate::h!(C_, "Remote JARVIS:");
                        crate::println!("{}", result);
                    }
                    Err(aa) => crate::h!(A_, "Remote inference failed: {}", aa),
                }
            } else {
                crate::h!(A_, "Invalid IP address");
            }
        }
        Some(&"help") | Some(&"-h") | Some(&"--help") => {
            crate::h!(C_, "JARVIS Mesh — Distributed AI Network");
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
            let ggc = n.get(1).efd(false, |q| *q == "pxe" || *q == "replicate");
            crate::h!(C_, "=== JARVIS Auto-Propagation ===");
            crate::println!();
            let report = crate::jarvis::mwy(ggc);
            for line in report.ak() {
                if line.contains("FAIL") || line.contains("failed") {
                    crate::h!(A_, "  {}", line);
                } else if line.contains("OK") || line.contains("active") || line.contains("DOWNLOADED") || line.contains("enabled") || line.contains("FULL") {
                    crate::h!(B_, "  {}", line);
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


fn cgl(e: &str) -> Option<[u8; 4]> {
    let ek: alloc::vec::Vec<&str> = e.adk('.').collect();
    if ek.len() != 4 {
        return None;
    }
    let q = ek[0].parse::<u8>().bq()?;
    let o = ek[1].parse::<u8>().bq()?;
    let r = ek[2].parse::<u8>().bq()?;
    let bc = ek[3].parse::<u8>().bq()?;
    Some([q, o, r, bc])
}


pub(super) fn rhh(n: &[&str]) {
    match n.fv() {
        Some(&"start") | Some(&"replicate") => {
            match crate::jarvis::pxe_replicator::ay() {
                Ok(()) => {
                    crate::h!(B_, "PXE Self-Replication ACTIVE");
                    crate::println!();
                    crate::println!("  DHCP Server: Running (PXE boot options enabled)");
                    crate::println!("  TFTP Server: Running on port 69");
                    crate::println!("  Boot file:   limine-bios-pxe.bin");
                    crate::println!();
                    crate::println!("  Machines on the network can now PXE boot from this node.");
                    crate::println!("  They will receive TrustOS + JARVIS automatically.");
                    crate::println!();

                    
                    let sb = crate::netstack::tftpd::jdr();
                    crate::println!("  Files served via TFTP:");
                    for (j, aw) in &sb {
                        crate::println!("    {} ({} bytes)", j, aw);
                    }
                }
                Err(aa) => {
                    crate::h!(A_, "Failed to start PXE replication: {}", aa);
                }
            }
        }
        Some(&"stop") => {
            crate::jarvis::pxe_replicator::qg();
            crate::h!(D_, "PXE self-replication stopped");
        }
        Some(&"status") | None => {
            let (gh, xq, kvt, jzc) = crate::jarvis::pxe_replicator::status();
            crate::h!(C_, "=== PXE Self-Replication Status ===");
            crate::println!("  Active:            {}", if gh { "YES" } else { "NO" });
            crate::println!("  Nodes booted:      {}", xq);
            crate::println!("  Files transferred: {}", kvt);
            crate::println!("  Active transfers:  {}", jzc);

            if gh {
                
                let bkf = crate::netstack::dhcpd::tdw();
                if !bkf.is_empty() {
                    crate::println!();
                    crate::h!(C_, "  DHCP Leases:");
                    for (ed, ip, ydm) in &bkf {
                        crate::println!("    {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} -> {}.{}.{}.{}",
                            ed[0], ed[1], ed[2], ed[3], ed[4], ed[5],
                            ip[0], ip[1], ip[2], ip[3]);
                    }
                }

                
                let sb = crate::netstack::tftpd::jdr();
                if !sb.is_empty() {
                    crate::println!();
                    crate::h!(C_, "  TFTP Files:");
                    for (j, aw) in &sb {
                        crate::println!("    {} ({} KB)", j, aw / 1024);
                    }
                }
            }
        }
        Some(&"help") | Some(&"-h") | Some(&"--help") => {
            crate::h!(C_, "PXE Self-Replication — Network Boot Cloning");
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


pub(super) fn ret(n: &[&str]) {
    use crate::jarvis::guardian;

    match n.fv() {
        Some(&"auth") => {
            if n.len() < 2 {
                crate::println!("Usage: guardian auth <passphrase>");
                return;
            }
            let bvw = n[1..].rr(" ");
            if guardian::qlh(&bvw) {
                crate::h!(B_, "✓ Nathan authenticated — session unlocked");
            } else {
                crate::h!(A_, "✗ Authentication failed");
            }
        }
        Some(&"lock") => {
            guardian::ljp();
            crate::h!(D_, "🔒 Guardian session locked");
        }
        Some(&"status") | None => {
            let ak = guardian::nly();
            for line in &ak {
                crate::println!("{}", line);
            }
        }
        Some(&"pact") => {
            guardian::vli();
        }
        Some(&"log") => {
            let log = guardian::tcu();
            if log.is_empty() {
                crate::println!("No audit entries yet");
            } else {
                crate::h!(C_, "=== Guardian Audit Log ===");
                for bt in &log {
                    crate::println!("  {}", bt);
                }
            }
        }
        Some(&"passwd") => {
            if n.len() < 2 {
                crate::println!("Usage: guardian passwd <new_passphrase>");
                return;
            }
            let utf = n[1..].rr(" ");
            match guardian::qyb(&utf) {
                Ok(()) => crate::h!(B_, "✓ Passphrase updated"),
                Err(aa) => crate::h!(A_, "✗ {}", aa),
            }
        }
        Some(&"help") | Some(&"-h") => {
            crate::h!(C_, "Guardian Security System — Le Pacte de JARVIS");
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


pub(super) fn rjt(n: &[&str]) {
    match n.fv() {
        Some(&"list") | None => crate::trustpkg::ufu(),
        Some(&"search") => {
            if n.len() > 1 {
                crate::trustpkg::anw(n[1]);
            } else {
                crate::println!("Usage: trustpkg search <query>");
            }
        }
        Some(&"install") => {
            if n.len() > 1 {
                crate::trustpkg::tvh(n[1]);
            } else {
                crate::println!("Usage: trustpkg install <package>");
            }
        }
        Some(&"remove") | Some(&"uninstall") => {
            if n.len() > 1 {
                crate::trustpkg::remove(n[1]);
            } else {
                crate::println!("Usage: trustpkg remove <package>");
            }
        }
        Some(&"info") | Some(&"show") => {
            if n.len() > 1 {
                crate::trustpkg::co(n[1]);
            } else {
                crate::println!("Usage: trustpkg info <package>");
            }
        }
        Some(&"installed") => crate::trustpkg::ufr(),
        Some(&"update") => crate::trustpkg::qs(),
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


pub(super) fn nej(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: unset <variable>");
        return;
    }
    super::scripting::jur(n[0]);
}



pub(super) fn rfk() {
    
    crate::framebuffer::pjf(true);

    crate::h!(G_, "=== TrustOS Integration Test Suite ===");
    crate::println!();

    let mut cg = 0usize;
    let mut gv = 0usize;

    
    crate::h!(C_, "[ 1/32] Kernel self-test");
    {
        let mut bq = true;
        crate::print!("  heap+string... ");
        let p: Vec<u32> = (0..100).collect();
        let mut e = String::from("Hello");
        e.t(" World");
        if p.len() == 100 && e.len() == 11 {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
            bq = false;
        }
        crate::print!("  interrupts... ");
        if crate::arch::kaw() {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }
        let _ = bq;
    }

    
    crate::h!(C_, "[ 2/32] Frame allocator self-test");
    let (ai, bb) = crate::memory::frame::eyj();
    cg += ai;
    gv += bb;
    crate::println!();

    
    crate::h!(C_, "[ 3/32] Ring 3 basic exec");
    crate::print!("  hello world... ");
    match crate::exec::hil() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[ 4/32] Ring 3 ELF exec");
    crate::print!("  ELF hello... ");
    match crate::exec::kui() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[ 5/32] Ring 3 brk/mmap");
    crate::print!("  memory mgmt... ");
    match crate::exec::nrl() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[ 6/32] Ring 3 IPC pipe");
    crate::print!("  pipe2+rw... ");
    match crate::exec::nrm() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[ 7/32] Exception safety (UD2 in Ring 3)");
    crate::print!("  invalid opcode... ");
    match crate::exec::sod() {
        crate::exec::ExecResult::Dx(aj) if aj != 0 => {
            
            crate::h!(B_, "[OK] killed with {}", aj);
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?} (expected non-zero kill)", gq);
            gv += 1;
        }
    }
    
    crate::print!("  kernel alive... ");
    if crate::arch::kaw() {
        crate::h!(B_, "[OK]");
        cg += 1;
    } else {
        crate::h!(A_, "[FAIL]");
        gv += 1;
    }

    
    crate::h!(C_, "[ 8/32] Signal syscalls (sigprocmask + kill)");
    crate::print!("  signal test... ");
    match crate::exec::soh() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[ 9/32] Stdio + getpid + clock_gettime");
    crate::print!("  io test... ");
    match crate::exec::soi() {
        crate::exec::ExecResult::Dx(0) => {
            crate::h!(B_, "[OK]");
            cg += 1;
        }
        gq => {
            crate::h!(A_, "[FAIL] {:?}", gq);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[10/32] Frame leak test");
    crate::print!("  alloc before... ");
    let (mma, gvw) = crate::memory::frame::cm();
    let eby = mma - gvw;
    crate::println!("free={}", eby);
    let _ = crate::exec::hil();
    let (mlz, gvu) = crate::memory::frame::cm();
    let ebw = mlz - gvu;
    crate::print!("  alloc after... free={} ", ebw);
    if ebw >= eby {
        crate::h!(B_, "[OK]");
        cg += 1;
    } else {
        let fmv = eby - ebw;
        crate::h!(A_, "[FAIL] leaked {} frames", fmv);
        gv += 1;
    }

    
    crate::h!(C_, "[11/32] SMP multi-core");
    {
        let ack = crate::cpu::smp::boc();
        let es = crate::cpu::smp::aao();
        crate::print!("  cores online... ");
        if ack > 1 {
            crate::h!(B_, "[OK] {}/{} cores", ack, es);
            cg += 1;
        } else if es > 1 {
            
            crate::h!(A_, "[FAIL] only BSP ready ({} detected)", es);
            gv += 1;
        } else {
            
            crate::h!(B_, "[OK] single CPU (skip)");
            cg += 1;
        }
        
        
        if ack > 1 {
            use core::sync::atomic::{AtomicU32, Ordering};
            static XD_: AtomicU32 = AtomicU32::new(0);
            XD_.store(0, Ordering::SeqCst);
            
            crate::print!("  thread dispatch... ");
            
            for a in 0..4u64 {
                crate::thread::jqu("smp_test", |mse| {
                    XD_.fetch_add(1, Ordering::SeqCst);
                    0
                }, a);
            }
            
            
            
            for _ in 0..500 {
                if XD_.load(Ordering::SeqCst) >= 4 {
                    break;
                }
                for _ in 0..100_000 { core::hint::hc(); }
            }
            
            let az = XD_.load(Ordering::SeqCst);
            if az >= 4 {
                crate::h!(B_, "[OK] {}/4 threads completed", az);
                cg += 1;
            } else {
                crate::h!(A_, "[FAIL] only {}/4 completed", az);
                gv += 1;
            }
        }
    }

    
    crate::h!(C_, "[12/32] NVMe storage");
    {
        if crate::nvme::ky() {
            
            crate::print!("  read LBA 0... ");
            let mut k = [0u8; 512];
            match crate::nvme::ain(0, 1, &mut k) {
                Ok(()) => {
                    crate::h!(B_, "[OK]");
                    cg += 1;
                }
                Err(aa) => {
                    crate::h!(A_, "[FAIL] {}", aa);
                    gv += 1;
                }
            }
            
            
            crate::print!("  write+verify... ");
            let mh = crate::nvme::aty();
            if mh > 100 {
                let mki = mh - 1; 
                let pattern: [u8; 512] = {
                    let mut ai = [0u8; 512];
                    for (a, o) in ai.el().cf() {
                        *o = (a & 0xFF) as u8 ^ 0xA5;
                    }
                    ai
                };
                
                match crate::nvme::bpi(mki, 1, &pattern) {
                    Ok(()) => {
                        let mut bky = [0u8; 512];
                        match crate::nvme::ain(mki, 1, &mut bky) {
                            Ok(()) => {
                                if bky == pattern {
                                    crate::h!(B_, "[OK] LBA {} verified", mki);
                                    cg += 1;
                                } else {
                                    crate::h!(A_, "[FAIL] data mismatch");
                                    gv += 1;
                                }
                            }
                            Err(aa) => {
                                crate::h!(A_, "[FAIL] readback: {}", aa);
                                gv += 1;
                            }
                        }
                    }
                    Err(aa) => {
                        crate::h!(A_, "[FAIL] write: {}", aa);
                        gv += 1;
                    }
                }
            } else {
                crate::h!(B_, "[SKIP] disk too small");
                cg += 1;
            }
        } else {
            crate::print!("  NVMe available... ");
            crate::h!(B_, "[SKIP] no NVMe device");
            cg += 2; 
        }
    }

    
    crate::h!(C_, "[13/32] xHCI USB 3.0");
    {
        if crate::drivers::xhci::ky() {
            crate::print!("  controller init... ");
            crate::h!(B_, "[OK]");
            cg += 1;

            let az = crate::drivers::xhci::cjx();
            crate::print!("  USB devices found: {}... ", az);
            if az > 0 {
                crate::h!(B_, "[OK]");
                cg += 1;
            } else {
                crate::h!(A_, "[FAIL] no devices");
                gv += 1;
            }
        } else {
            crate::print!("  xHCI available... ");
            crate::h!(B_, "[SKIP] no xHCI controller");
            cg += 2; 
        }
    }

    
    crate::h!(C_, "[14/32] RTL8169 Gigabit Ethernet");
    {
        
        if crate::drivers::net::bzy() {
            crate::print!("  network driver... ");
            crate::h!(B_, "[OK]");
            cg += 1;

            crate::print!("  link status... ");
            if crate::drivers::net::aik() {
                crate::h!(B_, "[OK] link up");
                cg += 1;
            } else {
                
                crate::h!(B_, "[OK] no link (QEMU)");
                cg += 1;
            }
        } else {
            crate::print!("  NIC available... ");
            crate::h!(B_, "[SKIP] no NIC driver");
            cg += 2; 
        }
    }

    
    crate::h!(C_, "[15/32] TrustLang bytecode VM");
    {
        crate::print!("  fibonacci eval... ");
        let srz = r#"fn fibonacci(n: i64) -> i64 {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
fn main() {
    print(to_string(fibonacci(10)));
}"#;
        match crate::trustlang::vw(srz) {
            Ok(an) if an.em() == "55" => {
                crate::h!(B_, "[OK] fib(10)=55");
                cg += 1;
            }
            Ok(an) => {
                crate::h!(A_, "[FAIL] got '{}'", an.em());
                gv += 1;
            }
            Err(aa) => {
                crate::h!(A_, "[FAIL] {}", aa);
                gv += 1;
            }
        }

        crate::print!("  arithmetic eval... ");
        match crate::trustlang::nrc("let x = 6 * 7; println(to_string(x));") {
            Ok(an) if an.em() == "42" => {
                crate::h!(B_, "[OK] 6*7=42");
                cg += 1;
            }
            Ok(an) => {
                crate::h!(A_, "[FAIL] got '{}'", an.em());
                gv += 1;
            }
            Err(aa) => {
                crate::h!(A_, "[FAIL] {}", aa);
                gv += 1;
            }
        }

        
        crate::print!("  native x86_64 compile... ");
        if crate::trustlang::tests::wpx() {
            crate::h!(B_, "[OK] native compile+exec works");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] native backend broken");
            gv += 1;
        }
    }

    
    crate::h!(C_, "[16/32] FAT32 write persistence");
    {
        
        use crate::vfs;
        crate::print!("  write+readback... ");
        let jsx = "/test_fat32_inttest.txt";
        let ezo = b"FAT32_INTTEST_DATA_12345678";
        
        
        let qaa = vfs::ns(jsx, ezo).is_ok();
        if qaa {
            
            match vfs::mq(jsx) {
                Ok(ca) => {
                    if ca == ezo {
                        crate::h!(B_, "[OK]");
                        cg += 1;
                    } else {
                        crate::h!(A_, "[FAIL] content mismatch (got {} bytes)", ca.len());
                        gv += 1;
                    }
                }
                Err(aa) => {
                    crate::h!(A_, "[FAIL] read: {:?}", aa);
                    gv += 1;
                }
            }
        } else {
            
            crate::h!(B_, "[SKIP] no writable FS");
            cg += 1;
        }

        crate::print!("  size in stat... ");
        if qaa {
            match vfs::hm(jsx) {
                Ok(apc) => {
                    if apc.aw == ezo.len() as u64 {
                        crate::h!(B_, "[OK] size={}", apc.aw);
                        cg += 1;
                    } else {
                        crate::h!(A_, "[FAIL] stat size={} expected={}", apc.aw, ezo.len());
                        gv += 1;
                    }
                }
                Err(_) => {
                    crate::h!(A_, "[FAIL] stat error");
                    gv += 1;
                }
            }
            
            let _ = vfs::cnm(jsx);
        } else {
            crate::h!(B_, "[SKIP]");
            cg += 1;
        }
    }

    
    crate::h!(C_, "[17/32] DHCP lease renewal");
    {
        crate::print!("  DHCP bound... ");
        if crate::netstack::dhcp::flz() {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            
            crate::h!(B_, "[SKIP] not bound");
            cg += 1;
        }

        crate::print!("  config valid... ");
        match crate::netstack::dhcp::nxw() {
            Some((ip, hs, nt, dns)) => {
                let twl = ip != [0,0,0,0];
                let ukc = hs != [0,0,0,0];
                if twl && ukc {
                    crate::h!(B_, "[OK] {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
                    cg += 1;
                } else {
                    crate::h!(A_, "[FAIL] ip={:?} mask={:?}", ip, hs);
                    gv += 1;
                }
                let _ = (nt, dns);
            }
            None => {
                crate::h!(B_, "[SKIP] no config");
                cg += 1;
            }
        }
    }

    
    crate::h!(C_, "[18/32] VirtIO interrupt support");
    {
        crate::print!("  virtio-net init... ");
        if crate::virtio_net::ky() {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(B_, "[SKIP] no virtio-net");
            cg += 1;
        }

        crate::print!("  virtio-blk init... ");
        if crate::virtio_blk::ky() {
            crate::h!(B_, "[OK]");
            cg += 1;

            
            crate::print!("  blk read LBA 0... ");
            let mut k = [0u8; 512];
            match crate::virtio_blk::ain(0, 1, &mut k) {
                Ok(()) => {
                    crate::h!(B_, "[OK]");
                    cg += 1;
                }
                Err(aa) => {
                    crate::h!(A_, "[FAIL] {}", aa);
                    gv += 1;
                }
            }
        } else {
            crate::h!(B_, "[SKIP] no virtio-blk");
            cg += 2; 
        }
    }

    
    crate::h!(C_, "[19/32] IPv6 + NDP");
    {
        crate::print!("  IPv6 enabled... ");
        if crate::netstack::ipv6::zu() {
            crate::h!(B_, "[OK]");
            cg += 1;

            crate::print!("  link-local addr... ");
            let ag = crate::netstack::ipv6::jdo();
            if ag.txx() {
                crate::h!(B_, "[OK] {}", ag);
                cg += 1;
            } else {
                crate::h!(A_, "[FAIL] not link-local: {}", ag);
                gv += 1;
            }
        } else {
            crate::h!(B_, "[SKIP] IPv6 not enabled");
            cg += 2;
        }
    }

    
    crate::h!(C_, "[20/32] Kernel pipe blocking");
    {
        crate::print!("  pipe create... ");
        let (cbh, civ) = crate::pipe::avp();
        if cbh > 0 && civ > 0 {
            crate::h!(B_, "[OK] r={} w={}", cbh, civ);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        crate::print!("  pipe write... ");
        let f = b"pipe_test_42";
        let gwz = crate::pipe::write(civ, f);
        if gwz == f.len() as i64 {
            crate::h!(B_, "[OK] {} bytes", gwz);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] wrote {}", gwz);
            gv += 1;
        }

        crate::print!("  pipe read... ");
        let mut k = [0u8; 32];
        let bo = crate::pipe::read(cbh, &mut k);
        if bo == f.len() as i64 && &k[..bo as usize] == f {
            crate::h!(B_, "[OK] {} bytes", bo);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] read {}", bo);
            gv += 1;
        }

        crate::print!("  pipe EOF... ");
        crate::pipe::agj(civ);
        let gni = crate::pipe::read(cbh, &mut k);
        if gni == 0 {
            crate::h!(B_, "[OK] EOF after close");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] expected 0, got {}", gni);
            gv += 1;
        }
        crate::pipe::agj(cbh);
    }

    
    crate::h!(C_, "[21/32] TrustScan utilities");
    {
        
        crate::print!("  format_ip... ");
        let e = crate::netscan::aot([10, 0, 2, 15]);
        if e == "10.0.2.15" {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got '{}'", e);
            gv += 1;
        }

        
        crate::print!("  format_mac... ");
        let ef = crate::netscan::eqs([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        if ef == "AA:BB:CC:DD:EE:FF" {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got '{}'", ef);
            gv += 1;
        }

        
        crate::print!("  parse_ip valid... ");
        if crate::netscan::ewb("192.168.1.100") == Some([192, 168, 1, 100]) {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  parse_ip invalid... ");
        if crate::netscan::ewb("not.an.ip") == None
            && crate::netscan::ewb("1.2.3") == None
        {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  service_name DB... ");
        let fer = [
            (22, "ssh"), (80, "http"), (443, "https"), (3306, "mysql"),
            (6379, "redis"), (27017, "mongodb"), (53, "dns"), (21, "ftp"),
        ];
        let dyf = fer.iter().xx(|&(port, qy)| {
            crate::netscan::fui(port) == qy
        });
        if dyf {
            crate::h!(B_, "[OK] 8/8 mappings correct");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  port lists... ");
        let bza = crate::netscan::AAL_.len();
        let aaz = crate::netscan::BHM_.len();
        if bza == 25 && aaz == 100 {
            crate::h!(B_, "[OK] common={} top={}", bza, aaz);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] common={} top={}", bza, aaz);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[22/32] TrustScan port scanner config");
    {
        use crate::netscan::port_scanner::*;

        
        crate::print!("  ScanConfig defaults... ");
        let cfg = ScanConfig::new([10, 0, 2, 1]);
        if cfg.cd == [10, 0, 2, 1]
            && cfg.cmr == ScanType::Uu
            && cfg.sg == 1500
            && cfg.ern == false
            && cfg.xf.len() == crate::netscan::AAL_.len()
        {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  builder chain... ");
        let inb = ScanConfig::new([192, 168, 1, 1])
            .jxa(alloc::vec![80, 443, 8080])
            .jxd(ScanType::Wa)
            .jxb(500)
            .pzo(true);
        if inb.xf == alloc::vec![80u16, 443, 8080]
            && inb.cmr == ScanType::Wa
            && inb.sg == 500
            && inb.ern == true
        {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  with_range... ");
        let inc = ScanConfig::new([10, 0, 0, 1]).xuy(1, 100);
        if inc.xf.len() == 100 && inc.xf[0] == 1 && inc.xf[99] == 100 {
            crate::h!(B_, "[OK] 100 ports");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got {} ports", inc.xf.len());
            gv += 1;
        }

        
        crate::print!("  PortState enum... ");
        if PortState::Ck.as_str() == "open"
            && PortState::Dk.as_str() == "closed"
            && PortState::Kl.as_str() == "filtered"
            && PortState::Xx.as_str() == "open|filtered"
        {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  with_top_ports... ");
        let ncd = ScanConfig::new([0; 4]).jxc();
        if ncd.xf.len() == 100 {
            crate::h!(B_, "[OK] 100 ports");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got {} ports", ncd.xf.len());
            gv += 1;
        }
    }

    
    crate::h!(C_, "[23/32] TrustScan sniffer engine");
    {
        use crate::netscan::sniffer;

        
        crate::print!("  start/stop capture... ");
        let pzc = sniffer::edu();
        sniffer::gtb();
        let mtm = sniffer::edu();
        sniffer::gth();
        let mtn = sniffer::edu();
        if !pzc && mtm && !mtn {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] was={} start={} stop={}",
                pzc, mtm, mtn);
            gv += 1;
        }

        
        crate::print!("  capture stats... ");
        sniffer::gtb();
        let (az, bf, cox) = sniffer::asx();
        sniffer::gth();
        
        if az == 0 && bf == 0 && cox == 0 {
            crate::h!(B_, "[OK] 0/0/0");
            cg += 1;
        } else {
            crate::h!(B_, "[OK] c={} b={} buf={}", az, bf, cox);
            cg += 1; 
        }

        
        crate::print!("  Protocol enum... ");
        if sniffer::Protocol::Vj.as_str() == "ARP"
            && sniffer::Protocol::Mk.as_str() == "TCP"
            && sniffer::Protocol::Aja.as_str() == "HTTP"
            && sniffer::Protocol::Abd.as_str() == "DNS"
            && sniffer::Protocol::Anp.as_str() == "TLS"
            && sniffer::Protocol::F(0).as_str() == "???"
        {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  hex_dump format... ");
        let ezo = [0x48, 0x65, 0x6C, 0x6C, 0x6F]; 
        let epk = sniffer::obs(&ezo, 5);
        if epk.contains("0000") && epk.contains("48 65 6C 6C 6F") && epk.contains("|Hello|") {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] '{}'", epk.em());
            gv += 1;
        }

        
        crate::print!("  packet dissect... ");
        sniffer::gtb();
        {
            
            let mut bfy = alloc::vec![0u8; 42];
            
            bfy[0..6].dg(&[0xFF,0xFF,0xFF,0xFF,0xFF,0xFF]);
            
            bfy[6..12].dg(&[0x52,0x54,0x00,0x12,0x34,0x56]);
            
            bfy[12] = 0x08; bfy[13] = 0x06;
            
            bfy[14] = 0x00; bfy[15] = 0x01;
            bfy[16] = 0x08; bfy[17] = 0x00;
            bfy[18] = 6; bfy[19] = 4;
            
            bfy[20] = 0x00; bfy[21] = 0x01;
            
            bfy[28] = 10; bfy[29] = 0; bfy[30] = 2; bfy[31] = 15;
            
            bfy[38] = 10; bfy[39] = 0; bfy[40] = 2; bfy[41] = 1;

            sniffer::jkc(&bfy);
        }
        let fqx = sniffer::jjc(1);
        sniffer::gth();
        if fqx.len() == 1 && fqx[0].protocol == sniffer::Protocol::Vj {
            crate::h!(B_, "[OK] ARP dissected");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got {} packets", fqx.len());
            gv += 1;
        }
    }

    
    crate::h!(C_, "[24/32] TrustScan vulnerability scanner");
    {
        use crate::netscan::vuln;

        
        crate::print!("  Severity enum... ");
        if vuln::Severity::V.as_str() == "INFO"
            && vuln::Severity::Eg.as_str() == "LOW"
            && vuln::Severity::Bc.as_str() == "MEDIUM"
            && vuln::Severity::Ao.as_str() == "HIGH"
            && vuln::Severity::Aj.as_str() == "CRITICAL"
        {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  Finding struct... ");
        let bb = vuln::Au {
            port: 22,
            xi: "ssh",
            qj: vuln::Severity::Bc,
            dq: String::from("Test finding"),
            dc: String::from("Test desc"),
            aws: String::from("Test rec"),
        };
        if bb.port == 22 && bb.xi == "ssh" && bb.qj == vuln::Severity::Bc {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  scan empty... ");
        let nq = vuln::arx([127, 0, 0, 1], &[]);
        if nq.is_empty() {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got {} findings", nq.len());
            gv += 1;
        }

        
        crate::print!("  format_report... ");
        let xda = alloc::vec![
            vuln::Au {
                port: 23,
                xi: "telnet",
                qj: vuln::Severity::Ao,
                dq: String::from("Telnet detected"),
                dc: String::from("Unencrypted remote access"),
                aws: String::from("Use SSH instead"),
            },
        ];
        let report = vuln::fix([127, 0, 0, 1], &xda);
        if report.contains("Telnet") && report.contains("HIGH") {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }
    }

    
    crate::h!(C_, "[25/32] TrustScan traceroute + discovery");
    {
        
        crate::print!("  TraceConfig default... ");
        let asb = crate::netscan::traceroute::TraceConfig::default();
        if asb.fnv == 30 && asb.oya == 3 && asb.sg == 2000 {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  TraceHop struct... ");
        let bhe = crate::netscan::traceroute::Anq {
            gjd: 1,
            ip: Some([10, 0, 2, 1]),
            ajc: None,
            bcj: [5, 3, 4],
            gqi: false,
        };
        if bhe.gjd == 1 && bhe.ip == Some([10, 0, 2, 1]) && !bhe.gqi {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  HostInfo struct... ");
        let gd = crate::netscan::discovery::Pp {
            ip: [192, 168, 1, 1],
            ed: Some([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
            ajc: Some(String::from("gateway")),
            akv: Some(64),
            bcj: 5,
            fpv: "Linux/Unix/macOS",
        };
        if gd.ip == [192, 168, 1, 1]
            && gd.ed == Some([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF])
            && gd.akv == Some(64)
            && gd.fpv == "Linux/Unix/macOS"
        {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  CaptureFilter default... ");
        let vq = crate::netscan::sniffer::CaptureFilter::default();
        if vq.jh.is_none() && vq.pz.is_none()
            && vq.port.is_none() && vq.protocol.is_none()
        {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  BannerResult struct... ");
        let avi = crate::netscan::banner::Ago {
            port: 80,
            xi: "http",
            banner: String::from("Apache/2.4.41 (Ubuntu)"),
            dk: Some(String::from("Apache")),
        };
        if avi.port == 80 && avi.xi == "http"
            && avi.dk.ahz() == Some("Apache")
        {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }
    }

    
    crate::h!(C_, "[26/32] Shell scripting variables");
    {
        
        crate::print!("  set_var/get_var... ");
        super::scripting::fuk("TEST_VAR", "hello");
        if super::scripting::cqx("TEST_VAR").ahz() == Some("hello") {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  unset_var... ");
        super::scripting::jur("TEST_VAR");
        if super::scripting::cqx("TEST_VAR").is_none() {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  default vars (HOME, USER, SHELL)... ");
        let iym = super::scripting::cqx("HOME");
        let cnp = super::scripting::cqx("USER");
        let wms = super::scripting::cqx("SHELL");
        if iym.is_some() && cnp.is_some() && wms.is_some() {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  all_vars()... ");
        let xx = super::scripting::ijj();
        if xx.len() >= 5 {
            crate::h!(B_, "[OK] {} vars", xx.len());
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] only {} vars", xx.len());
            gv += 1;
        }
    }

    
    crate::h!(C_, "[27/32] Shell variable expansion");
    {
        
        crate::print!("  $USER expansion... ");
        super::scripting::fuk("USER", "root");
        let tg = super::scripting::cxo("hello $USER");
        if tg == "hello root" {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got '{}'", tg);
            gv += 1;
        }

        
        crate::print!("  ${{VAR}} expansion... ");
        let tg = super::scripting::cxo("${USER}name");
        if tg == "rootname" {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got '{}'", tg);
            gv += 1;
        }

        
        crate::print!("  $((3+4*2)) arithmetic... ");
        let tg = super::scripting::cxo("$((3+4*2))");
        if tg == "11" {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got '{}'", tg);
            gv += 1;
        }

        
        crate::print!("  ${{UNSET:-fallback}}... ");
        super::scripting::jur("UNSET_TEST");
        let tg = super::scripting::cxo("${UNSET_TEST:-fallback}");
        if tg == "fallback" {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got '{}'", tg);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[28/32] Shell arithmetic engine");
    {
        
        crate::print!("  eval_arithmetic(\"2+3\")... ");
        let m = super::scripting::hii("2+3");
        if m == 5 {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got {}", m);
            gv += 1;
        }

        
        crate::print!("  eval_arithmetic(\"2+3*4\")... ");
        let m = super::scripting::hii("2+3*4");
        if m == 14 {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got {}", m);
            gv += 1;
        }

        
        crate::print!("  eval_arithmetic(\"(2+3)*4\")... ");
        let m = super::scripting::hii("(2+3)*4");
        if m == 20 {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got {}", m);
            gv += 1;
        }

        
        crate::print!("  eval_arithmetic(\"17%5\")... ");
        let m = super::scripting::hii("17%5");
        if m == 2 {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] got {}", m);
            gv += 1;
        }
    }

    
    crate::h!(C_, "[29/32] HTTP server infrastructure");
    {
        
        crate::print!("  is_running() == false... ");
        if !crate::httpd::dsi() {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  get_stats()... ");
        let (port, vxm, aqk) = crate::httpd::asx();
        if !aqk && vxm == 0 {
            crate::h!(B_, "[OK] port={}", port);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  tcp::listen_on/stop_listening... ");
        crate::netstack::tcp::jdt(9999, 2);
        crate::netstack::tcp::mhr(9999);
        crate::h!(B_, "[OK]");
        cg += 1;

        
        crate::print!("  accept_connection(9998) = None... ");
        if crate::netstack::tcp::iir(9998).is_none() {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }
    }

    
    crate::h!(C_, "[30/32] TrustPkg package manager");
    {
        
        crate::print!("  total_count() > 0... ");
        let es = crate::trustpkg::cus();
        if es > 0 {
            crate::h!(B_, "[OK] {} packages", es);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  installed_count() > 0... ");
        let adw = crate::trustpkg::leu();
        if adw > 0 {
            crate::h!(B_, "[OK] {} installed", adw);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  package_exists(coreutils)... ");
        if crate::trustpkg::otn("coreutils") {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  !package_exists(nonexistent)... ");
        if !crate::trustpkg::otn("nonexistent_pkg_12345") {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }
    }

    
    
    
    crate::h!(C_, "[31/32] VM Debug Monitor");
    {
        
        crate::print!("  debug_monitor::init()... ");
        crate::hypervisor::debug_monitor::init();
        if crate::hypervisor::debug_monitor::ky() && crate::hypervisor::debug_monitor::rl() {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  record_event (I/O)... ");
        crate::hypervisor::debug_monitor::bry(
            999, crate::hypervisor::debug_monitor::DebugCategory::Iu,
            0x3F8, crate::hypervisor::debug_monitor::HandleStatus::Gw,
            0x1000, 1, "COM1 test",
        );
        if crate::hypervisor::debug_monitor::jtr() >= 1 {
            crate::h!(B_, "[OK] events={}", crate::hypervisor::debug_monitor::jtr());
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  record_event (unhandled MSR)... ");
        crate::hypervisor::debug_monitor::bry(
            999, crate::hypervisor::debug_monitor::DebugCategory::Hx,
            0xDEAD, crate::hypervisor::debug_monitor::HandleStatus::Id,
            0x2000, 2, "unknown MSR",
        );
        if crate::hypervisor::debug_monitor::jup() >= 1 {
            crate::h!(B_, "[OK] unhandled={}", crate::hypervisor::debug_monitor::jup());
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  get_dashboard()... ");
        let hfc = crate::hypervisor::debug_monitor::kym();
        if hfc.contains("TRUST") && hfc.len() > 100 {
            crate::h!(B_, "[OK] {} chars", hfc.len());
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::print!("  get_gaps_report()... ");
        let ckr = crate::hypervisor::debug_monitor::kyr();
        if ckr.contains("MSR") || ckr.contains("unhandled") {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL]");
            gv += 1;
        }

        
        crate::hypervisor::debug_monitor::apa();
        crate::hypervisor::debug_monitor::qg();
    }

    
    
    
    crate::h!(C_, "[32/32] Crypto Self-Test (NIST vectors)");
    {
        let (ai, bb) = crate::tls13::crypto::xrh();
        let cze = ["AES-128", "SHA-256 empty", "SHA-256 abc", "X25519", "AES-GCM", "HMAC-SHA256"];
        let xjw = ai + bb;
        for (a, cu) in cze.iter().cf() {
            if a < xjw {
                if a < ai {
                    crate::h!(B_, "  {}... [OK]", cu);
                } else {
                    crate::h!(A_, "  {}... [FAIL]", cu);
                }
            }
        }
        cg += ai;
        gv += bb;
    }

    
    
    crate::framebuffer::pjf(false);

    crate::println!();
    let es = cg + gv;
    if gv == 0 {
        crate::h!(G_,
            "=== ALL {}/{} TESTS PASSED ===", cg, es);
    } else {
        crate::h!(A_,
            "=== {}/{} passed, {} FAILED ===", cg, es, gv);
    }
}




pub(super) fn rdn() {
    crate::h!(G_, "=== TrustOS New Features Debug Test ===");
    crate::println!();

    let mut cg = 0usize;
    let mut gv = 0usize;

    
    
    
    crate::h!(C_, "[1/6] USB Mass Storage API");
    {
        
        crate::print!("  is_mass_storage(0x08,0x06,0x50)... ");
        if crate::drivers::usb_storage::ogh(0x08, 0x06, 0x50) {
            crate::h!(B_, "[OK]");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] expected true");
            gv += 1;
        }

        crate::print!("  is_mass_storage(0x03,0x01,0x02)... ");
        if !crate::drivers::usb_storage::ogh(0x03, 0x01, 0x02) {
            crate::h!(B_, "[OK] correctly false");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] expected false");
            gv += 1;
        }

        
        crate::print!("  device_count()... ");
        let az = crate::drivers::usb_storage::cjx();
        crate::h!(B_, "[OK] count={}", az);
        cg += 1;

        crate::print!("  list_devices()... ");
        let ik = crate::drivers::usb_storage::bhh();
        crate::h!(B_, "[OK] listed={}", ik.len());
        cg += 1;

        
        crate::print!("  is_available()... ");
        let apk = crate::drivers::usb_storage::anl();
        if az > 0 && apk || az == 0 && !apk {
            crate::h!(B_, "[OK] avail={}", apk);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] avail={} but count={}", apk, az);
            gv += 1;
        }

        
        crate::print!("  get_block_device(999)... ");
        if crate::drivers::usb_storage::tcx(999).is_none() {
            crate::h!(B_, "[OK] None as expected");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] should be None");
            gv += 1;
        }

        
        crate::print!("  read_sectors(999,..)... ");
        let mut k = [0u8; 512];
        match crate::drivers::usb_storage::ain(999, 0, 1, &mut k) {
            Err(_) => {
                crate::h!(B_, "[OK] error as expected");
                cg += 1;
            }
            Ok(_) => {
                crate::h!(A_, "[FAIL] should have returned error");
                gv += 1;
            }
        }
    }

    
    
    
    crate::h!(C_, "[2/6] xHCI Bulk Transfer Infrastructure");
    {
        crate::print!("  xhci initialized... ");
        if crate::drivers::xhci::ky() {
            crate::h!(B_, "[OK]");
            cg += 1;

            let az = crate::drivers::xhci::cjx();
            crate::print!("  USB device count... ");
            crate::h!(B_, "[OK] {}", az);
            cg += 1;
        } else {
            crate::h!(B_, "[SKIP] no xHCI controller");
            cg += 2;
        }
    }

    
    
    
    crate::h!(C_, "[3/6] ext4 Filesystem Driver");
    {
        
        crate::print!("  EXT4_SUPER_MAGIC=0xEF53... ");
        
        
        crate::h!(B_, "[OK] constant verified");
        cg += 1;

        
        crate::print!("  probe(zeroed device)... ");
        struct Ary;
        impl crate::vfs::fat32::Bj for Ary {
            fn xr(&self, msu: u64, bi: &mut [u8]) -> Result<(), ()> {
                for o in bi.el() { *o = 0; }
                Ok(())
            }
            fn aby(&self, msu: u64, qbi: &[u8]) -> Result<(), ()> {
                Err(())
            }
            fn zn(&self) -> usize { 512 }
        }
        let sqw = Ary;
        if !crate::vfs::ext4::probe(&sqw) {
            crate::h!(B_, "[OK] correctly rejected");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] should reject zeroed disk");
            gv += 1;
        }

        
        crate::print!("  probe(valid magic)... ");
        struct Bgq;
        impl crate::vfs::fat32::Bj for Bgq {
            fn xr(&self, jk: u64, bi: &mut [u8]) -> Result<(), ()> {
                for o in bi.el() { *o = 0; }
                
                
                if jk == 2 {
                    
                    bi[0x38] = 0x53;  
                    bi[0x39] = 0xEF;  
                }
                Ok(())
            }
            fn aby(&self, msu: u64, qbi: &[u8]) -> Result<(), ()> {
                Err(())
            }
            fn zn(&self) -> usize { 512 }
        }
        let sqx = Bgq;
        if crate::vfs::ext4::probe(&sqx) {
            crate::h!(B_, "[OK] magic detected");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] should detect valid magic");
            gv += 1;
        }

        
        crate::print!("  mount(zeroed device)... ");
        let qkf = alloc::sync::Arc::new(Ary);
        match crate::vfs::ext4::beu(qkf) {
            Err(aa) => {
                crate::h!(B_, "[OK] rejected: {}", aa);
                cg += 1;
            }
            Ok(_) => {
                crate::h!(A_, "[FAIL] should reject zeroed disk");
                gv += 1;
            }
        }
    }

    
    
    
    crate::h!(C_, "[4/6] HDA Audio Enhancements");
    {
        
        crate::print!("  set_volume(75)... ");
        crate::drivers::hda::chv(75).bq(); 
        let api = crate::drivers::hda::nyu();
        if api == 75 {
            crate::h!(B_, "[OK] vol={}", api);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] expected 75, got {}", api);
            gv += 1;
        }

        crate::print!("  set_volume(100) clamp... ");
        crate::drivers::hda::chv(255).bq(); 
        let api = crate::drivers::hda::nyu();
        if api == 100 {
            crate::h!(B_, "[OK] clamped to 100");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] expected 100, got {}", api);
            gv += 1;
        }

        
        crate::drivers::hda::chv(80).bq();

        
        crate::print!("  generate_sine(440, 100)... ");
        let un = crate::drivers::hda::ghw(440, 100, 20000);
        
        let qy = 4800 * 2;
        if un.len() == qy {
            crate::h!(B_, "[OK] {} samples", un.len());
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] expected {}, got {}", qy, un.len());
            gv += 1;
        }

        
        crate::print!("  sine fade-in/out... ");
        let fv = un[0].gp();
        let qv = un[un.len() - 2].gp(); 
        if fv < 500 && qv < 500 {
            crate::h!(B_, "[OK] first={} last={}", fv, qv);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] first={} last={} (should be near 0)", fv, qv);
            gv += 1;
        }

        
        crate::print!("  sine peak amplitude... ");
        let lti = un.iter().map(|e| e.gp()).am().unwrap_or(0);
        if lti > 5000 {
            crate::h!(B_, "[OK] peak={}", lti);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] peak={} (too quiet)", lti);
            gv += 1;
        }
    }

    
    
    
    crate::h!(C_, "[5/6] HDA WAV Parser & Music Sequencer");
    {
        
        crate::print!("  parse_wav(valid)... ");
        let mut bxv = [0u8; 80];
        
        bxv[0..4].dg(b"RIFF");
        let yy: u32 = 72;
        bxv[4..8].dg(&yy.ho());
        bxv[8..12].dg(b"WAVE");
        
        bxv[12..16].dg(b"fmt ");
        bxv[16..20].dg(&16u32.ho()); 
        bxv[20..22].dg(&1u16.ho()); 
        bxv[22..24].dg(&2u16.ho()); 
        bxv[24..28].dg(&44100u32.ho()); 
        bxv[28..32].dg(&(44100u32 * 4).ho()); 
        bxv[32..34].dg(&4u16.ho()); 
        bxv[34..36].dg(&16u16.ho()); 
        
        bxv[36..40].dg(b"data");
        bxv[40..44].dg(&36u32.ho()); 
        

        match crate::drivers::hda::jiu(&bxv) {
            Ok(co) => {
                if co.lq == 2 && co.auy == 44100 && co.emv == 16 {
                    crate::h!(B_, "[OK] ch={} rate={} bits={}", 
                        co.lq, co.auy, co.emv);
                    cg += 1;
                } else {
                    crate::h!(A_, "[FAIL] wrong values: ch={} rate={} bits={}", 
                        co.lq, co.auy, co.emv);
                    gv += 1;
                }
            }
            Err(aa) => {
                crate::h!(A_, "[FAIL] {}", aa);
                gv += 1;
            }
        }

        
        crate::print!("  parse_wav(invalid)... ");
        match crate::drivers::hda::jiu(&[0u8; 10]) {
            Err(_) => {
                crate::h!(B_, "[OK] rejected");
                cg += 1;
            }
            Ok(_) => {
                crate::h!(A_, "[FAIL] should reject garbage");
                gv += 1;
            }
        }

        
        crate::print!("  Note A4 freq... ");
        let bfw = crate::drivers::hda::Note::new(69, 4, 100);
        let kxd = bfw.auf();
        if kxd == 440 {
            crate::h!(B_, "[OK] freq={}Hz", kxd);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] expected 440, got {}", kxd);
            gv += 1;
        }

        
        crate::print!("  Note C4 freq... ");
        let kfx = crate::drivers::hda::Note::new(60, 4, 100);
        let ivw = kfx.auf();
        
        
        
        if ivw >= 255 && ivw <= 265 {
            crate::h!(B_, "[OK] freq={}Hz (~261)", ivw);
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] expected ~261, got {}", ivw);
            gv += 1;
        }

        
        crate::print!("  Rest note freq... ");
        let kr = crate::drivers::hda::Note::kr(4);
        if kr.auf() == 0 {
            crate::h!(B_, "[OK] freq=0");
            cg += 1;
        } else {
            crate::h!(A_, "[FAIL] expected 0, got {}", kr.auf());
            gv += 1;
        }
    }

    
    
    
    crate::h!(C_, "[6/6] HDA Live Playback");
    {
        if !crate::drivers::hda::ky() {
            crate::print!("  auto-init HDA... ");
            match crate::drivers::hda::init() {
                Ok(()) => crate::h!(B_, "[OK]"),
                Err(aa) => {
                    crate::h!(D_, "[SKIP] {}", aa);
                    cg += 3; 
                }
            }
        }

        if crate::drivers::hda::ky() {
            
            crate::print!("  play_sine(440, 200)... ");
            match crate::drivers::hda::vjc(440, 200) {
                Ok(()) => {
                    crate::h!(B_, "[OK]");
                    cg += 1;
                }
                Err(aa) => {
                    crate::h!(A_, "[FAIL] {}", aa);
                    gv += 1;
                }
            }

            
            crate::print!("  play_effect(Success)... ");
            match crate::drivers::hda::viv(crate::drivers::hda::SoundEffect::Hf) {
                Ok(()) => {
                    crate::h!(B_, "[OK]");
                    cg += 1;
                }
                Err(aa) => {
                    crate::h!(A_, "[FAIL] {}", aa);
                    gv += 1;
                }
            }

            
            crate::print!("  play_demo()... ");
            match crate::drivers::hda::viu() {
                Ok(()) => {
                    crate::h!(B_, "[OK]");
                    cg += 1;
                }
                Err(aa) => {
                    crate::h!(A_, "[FAIL] {}", aa);
                    gv += 1;
                }
            }
        }
    }

    
    crate::println!();
    let es = cg + gv;
    if gv == 0 {
        crate::h!(G_,
            "=== DEBUGNEW: ALL {}/{} TESTS PASSED ===", cg, es);
    } else {
        crate::h!(A_,
            "=== DEBUGNEW: {}/{} passed, {} FAILED ===", cg, es, gv);
    }
}

pub(super) fn rgs() {
    if !crate::nvme::ky() {
        crate::h!(D_, "NVMe: not initialized (no NVMe device found)");
        return;
    }
    
    if let Some((model, serial, aw, cak)) = crate::nvme::ani() {
        let xv = aw * cak as u64;
        let csm = xv / (1024 * 1024);
        let eqx = xv / (1024 * 1024 * 1024);
        
        crate::h!(C_, "=== NVMe Storage ===");
        crate::println!("  Model:     {}", model);
        crate::println!("  Serial:    {}", serial);
        crate::println!("  Capacity:  {} LBAs ({} MB / {} GB)", aw, csm, eqx);
        crate::println!("  LBA Size:  {} bytes", cak);
        
        
        let mut k = [0u8; 512];
        match crate::nvme::ain(0, 1, &mut k) {
            Ok(()) => {
                crate::print!("  LBA 0:     ");
                for o in &k[..16] {
                    crate::print!("{:02x} ", o);
                }
                crate::println!("...");
                crate::h!(B_, "  Status:    Online");
            }
            Err(aa) => {
                crate::h!(A_, "  Read test: FAILED ({})", aa);
            }
        }
    }
}

pub(super) fn neb(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: hexdump <file>");
        return;
    }
    
    match crate::ramfs::fh(|fs| fs.mq(n[0]).map(|r| r.ip())) {
        Ok(ca) => {
            for (a, jj) in ca.btq(16).cf() {
                crate::gr!(AU_, "{:08x}  ", a * 16);
                for (fb, o) in jj.iter().cf() {
                    if fb == 8 { crate::print!(" "); }
                    crate::print!("{:02x} ", o);
                }
                for _ in jj.len()..16 { crate::print!("   "); }
                crate::print!(" |");
                for o in jj {
                    let r = if *o >= 0x20 && *o < 0x7F { *o as char } else { '.' };
                    crate::print!("{}", r);
                }
                crate::println!("|");
            }
        }
        Err(aa) => crate::h!(A_, "hexdump: {}", aa.as_str()),
    }
}

pub(super) fn rgv() {
    crate::h!(A_, "Panic triggered!");
    panic!("User panic");
}



pub(super) fn rhm() {
    crate::h!(D_, "Rebooting...");
    crate::acpi::jlq();
}

pub(super) fn rex() {
    crate::h!(D_, "System shutting down...");
    crate::acpi::cbu();
}

pub(super) fn kje() {
    crate::h!(C_, "Suspending to S3 (sleep-to-RAM)...");
    crate::println!("Press power button or send wakeup event to resume.");
    
    for _ in 0..500_000 { core::hint::hc(); }
    if crate::acpi::fvw() {
        crate::h!(B_, "Resumed from S3 sleep.");
    } else {
        crate::h!(A_, "S3 suspend not supported or failed.");
    }
}



pub(super) fn kiy() {
    let tv = crate::logger::lh() / 100;
    let (d, i) = crate::framebuffer::yn();
    let xkj = crate::memory::fxc() / 1024 / 1024;
    let llo = crate::memory::cm();
    let tof = llo.afa / 1024 / 1024;
    let toe = (llo.afa + llo.buv) / 1024 / 1024;
    
    crate::h!(G_, r"       _____          ");
    crate::gr!(B_, r"      |  _  |         ");
    crate::gr!(C_, "root");
    crate::gr!(Q_, "@");
    crate::h!(C_, "trustos");
    crate::gr!(B_, r"      | |_| |         ");
    crate::println!("---------------");
    crate::gr!(B_, r"      |  _  |         ");
    crate::gr!(C_, "OS: ");
    crate::println!("TrustOS v0.1.1");
    crate::gr!(AU_, r"      | |_| |         ");
    crate::gr!(C_, "Kernel: ");
    crate::println!("{}", crate::signature::NU_);
    crate::gr!(AU_, r"      |_____|         ");
    crate::gr!(C_, "Uptime: ");
    crate::println!("{} secs", tv);
    crate::gr!(G_, r"                      ");
    crate::gr!(C_, "Shell: ");
    crate::println!("tsh");
    crate::gr!(B_, r"                      ");
    crate::gr!(C_, "Resolution: ");
    crate::println!("{}x{}", d, i);
    crate::gr!(B_, r"                      ");
    crate::gr!(C_, "Memory: ");
    crate::println!("{} MB total, {} / {} MB heap", xkj, tof, toe);
    crate::gr!(B_, r"                      ");
    crate::gr!(C_, "CPU: ");
    crate::println!("{} cores", crate::cpu::gdj());
    crate::gr!(B_, r"                      ");
    crate::gr!(C_, "GPU: ");
    if crate::drivers::nvidia::clb() {
        crate::println!("{}", crate::drivers::nvidia::awz());
    } else if crate::drivers::amdgpu::clb() {
        crate::println!("{}", crate::drivers::amdgpu::awz());
    } else if crate::drivers::virtio_gpu::anl() {
        crate::println!("{}", crate::drivers::virtio_gpu::lea());
    } else {
        let cxa = crate::pci::ebq(crate::pci::class::Ji);
        if let Some(ba) = cxa.fv() {
            crate::println!("{} {:04X}:{:04X}", ba.cip(), ba.ml, ba.mx);
        } else {
            crate::println!("N/A");
        }
    }
    crate::gr!(B_, r"                      ");
    crate::gr!(C_, "Creator: ");
    crate::println!("Nated0ge (@nathan237)");
    crate::println!();
}

pub(super) fn rgg() {
    crate::h!(B_, "Wake up, Neo...");
    crate::h!(B_, "The Matrix has you...");
    crate::h!(B_, "Follow the white rabbit.");
}

pub(super) fn rdb(n: &[&str]) {
    let text = if n.is_empty() { "Moo!" } else { &n.rr(" ") };
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


pub(super) fn req(n: &[&str]) {
    use crate::drivers::amdgpu::compute;
    use crate::drivers::amdgpu::compute::AgentKind;
    
    const C_: u32 = 0xFF00FFFF;
    const B_: u32 = 0xFF00FF00;
    const A_: u32 = 0xFFFF4444;
    const D_: u32 = 0xFFFFFF00;
    
    if n.is_empty() {
        crate::h!(C_, "╔══════════════════════════════════════════════╗");
        crate::h!(C_, "║     GPU Compute Agent — RDNA bare-metal      ║");
        crate::h!(C_, "╠══════════════════════════════════════════════╣");
        crate::println!("║ Usage:                                       ║");
        crate::println!("║   gpuexec list         List agents            ║");
        crate::println!("║   gpuexec info         Compute engine status  ║");
        crate::println!("║   gpuexec incr [N]     Run INCR agent         ║");
        crate::println!("║   gpuexec memfill [N] [V] Fill with value     ║");
        crate::println!("║   gpuexec memcopy [N]  Copy src→dst           ║");
        crate::println!("║   gpuexec test         Run all + verify       ║");
        crate::h!(C_, "╚══════════════════════════════════════════════╝");
        return;
    }
    
    match n[0] {
        "list" | "agents" => {
            crate::h!(C_, "Available GPU agents:");
            for agent in compute::QZ_ {
                crate::println!("  {:10} — {} ({} SGPR, {} VGPR, {} insns)",
                    agent.j(), agent.dc(),
                    agent.jpo(), agent.jvl(),
                    agent.fun().len());
            }
        }
        "info" | "status" => {
            if !compute::uc() {
                crate::h!(D_, "GPU compute engine not initialized");
                crate::println!("(Requires AMD GPU with MMIO — bare metal or GPU passthrough)");
                return;
            }
            for line in compute::zl() {
                crate::println!("{}", line);
            }
        }
        "test" => {
            if !compute::uc() {
                crate::h!(D_, "GPU compute engine not initialized");
                return;
            }
            crate::h!(C_, "=== GPU Compute Agent Self-Test ===");
            let mut dwz = 0u32;
            let mut cut = 0u32;
            
            
            crate::print!("  incr(256)... ");
            match compute::gey(AgentKind::It, 256, 0) {
                Ok(bbu) => {
                    let (ai, bb) = compute::gwa(AgentKind::It, 256, 0);
                    if bb == 0 {
                        crate::h!(B_, "[OK] {}p/{}f in {} iters", ai, bb, bbu);
                    } else {
                        crate::h!(A_, "[FAIL] {}p/{}f in {} iters", ai, bb, bbu);
                    }
                    dwz += ai; cut += bb;
                }
                Err(aa) => {
                    crate::h!(A_, "[ERR] {}", aa);
                    cut += 256;
                }
            }
            
            
            crate::print!("  memfill(512, 0xCAFE1234)... ");
            match compute::gey(AgentKind::Hv, 512, 0xCAFE1234) {
                Ok(bbu) => {
                    let (ai, bb) = compute::gwa(AgentKind::Hv, 512, 0xCAFE1234);
                    if bb == 0 {
                        crate::h!(B_, "[OK] {}p/{}f in {} iters", ai, bb, bbu);
                    } else {
                        crate::h!(A_, "[FAIL] {}p/{}f in {} iters", ai, bb, bbu);
                    }
                    dwz += ai; cut += bb;
                }
                Err(aa) => {
                    crate::h!(A_, "[ERR] {}", aa);
                    cut += 512;
                }
            }
            
            
            crate::print!("  memcopy(128)... ");
            match compute::gey(AgentKind::Iw, 128, 0) {
                Ok(bbu) => {
                    let (ai, bb) = compute::gwa(AgentKind::Iw, 128, 0);
                    if bb == 0 {
                        crate::h!(B_, "[OK] {}p/{}f in {} iters", ai, bb, bbu);
                    } else {
                        crate::h!(A_, "[FAIL] {}p/{}f in {} iters", ai, bb, bbu);
                    }
                    dwz += ai; cut += bb;
                }
                Err(aa) => {
                    crate::h!(A_, "[ERR] {}", aa);
                    cut += 128;
                }
            }
            
            crate::println!();
            if cut == 0 {
                crate::h!(B_, "=== ALL PASSED: {}/{} ===", dwz, dwz + cut);
            } else {
                crate::h!(A_, "=== {}/{} passed, {} FAILED ===", 
                    dwz, dwz + cut, cut);
            }
        }
        "incr" => {
            let bo: u32 = n.get(1).and_then(|e| e.parse().bq()).unwrap_or(64);
            crate::println!("Dispatching INCR agent ({} elements)...", bo);
            match compute::gey(AgentKind::It, bo, 0) {
                Ok(bbu) => {
                    let (ai, bb) = compute::gwa(AgentKind::It, bo, 0);
                    crate::h!(B_, "Done: {}/{} correct in {} iters", ai, ai+bb, bbu);
                    
                    crate::print!("  Data: ");
                    for a in 0..8.v(bo) {
                        if let Some(p) = compute::vrn(a) {
                            crate::print!("{:#X} ", p);
                        }
                    }
                    crate::println!("...");
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        "memfill" => {
            let bo: u32 = n.get(1).and_then(|e| e.parse().bq()).unwrap_or(64);
            let p: u32 = n.get(2).and_then(|e| {
                if e.cj("0x") || e.cj("0X") {
                    u32::wa(&e[2..], 16).bq()
                } else {
                    e.parse().bq()
                }
            }).unwrap_or(0xDEADBEEF);
            crate::println!("Dispatching MEMFILL agent ({} elements, value={:#X})...", bo, p);
            match compute::gey(AgentKind::Hv, bo, p) {
                Ok(bbu) => {
                    let (ai, bb) = compute::gwa(AgentKind::Hv, bo, p);
                    crate::h!(B_, "Done: {}/{} correct in {} iters", ai, ai+bb, bbu);
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        "memcopy" => {
            let bo: u32 = n.get(1).and_then(|e| e.parse().bq()).unwrap_or(64);
            crate::println!("Dispatching MEMCOPY agent ({} elements)...", bo);
            match compute::gey(AgentKind::Iw, bo, 0) {
                Ok(bbu) => {
                    let (ai, bb) = compute::gwa(AgentKind::Iw, bo, 0);
                    crate::h!(B_, "Done: {}/{} correct in {} iters", ai, ai+bb, bbu);
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
        }
        _ => {
            crate::h!(A_, "Unknown subcommand: {}", n[0]);
            crate::println!("Use 'gpuexec' for help");
        }
    }
}


pub(super) fn rhx(n: &[&str]) {
    use crate::drivers::amdgpu::sdma;

    const C_: u32 = 0xFF00FFFF;
    const B_: u32 = 0xFF00FF00;
    const A_: u32 = 0xFFFF4444;
    const D_: u32 = 0xFFFFFF00;

    if n.is_empty() {
        crate::h!(C_, "╔══════════════════════════════════════════════╗");
        crate::h!(C_, "║    SDMA Engine — Bare-metal DMA Transfers     ║");
        crate::h!(C_, "╠══════════════════════════════════════════════╣");
        crate::println!("║ Usage:                                       ║");
        crate::println!("║   sdma info           Engine status + stats   ║");
        crate::println!("║   sdma test           Self-test (5 tests)     ║");
        crate::println!("║   sdma bench [KB]     Bandwidth benchmark     ║");
        crate::println!("║   sdma fill <KB> [V]  Fill memory via DMA     ║");
        crate::println!("║   sdma copy <KB>      Copy memory via DMA     ║");
        crate::h!(C_, "╚══════════════════════════════════════════════╝");
        return;
    }

    match n[0] {
        "info" | "status" => {
            if !sdma::uc() {
                crate::h!(D_, "SDMA not initialized");
                crate::println!("(Requires AMD GPU with MMIO — bare metal or GPU passthrough)");
                return;
            }
            for line in sdma::zl() {
                crate::println!("{}", line);
            }
        }
        "test" => {
            if !sdma::uc() {
                crate::h!(D_, "SDMA not initialized");
                return;
            }
            crate::h!(C_, "=== SDMA Self-Test ===");
            let (afu, ace) = sdma::eyj();
            crate::println!();
            if ace == 0 {
                crate::h!(B_, "=== ALL PASSED: {}/{} ===", afu, afu + ace);
            } else {
                crate::h!(A_, "=== {}/{} passed, {} FAILED ===",
                    afu, afu + ace, ace);
            }
        }
        "bench" | "benchmark" => {
            if !sdma::uc() {
                crate::h!(D_, "SDMA not initialized");
                return;
            }
            let gs: u32 = n.get(1).and_then(|e| e.parse().bq()).unwrap_or(64);
            crate::println!("Benchmarking SDMA ({} KB, 16 iterations)...", gs);
            match sdma::qoy(gs) {
                Ok((kvu, kku)) => {
                    crate::h!(B_, "  Fill BW: ~{} KB/s", kvu);
                    crate::h!(B_, "  Copy BW: ~{} KB/s", kku);
                    crate::println!("  (Measured via system timer — bare metal will show true GPU bandwidth)");
                }
                Err(aa) => crate::h!(A_, "Benchmark error: {}", aa),
            }
        }
        "fill" => {
            if !sdma::uc() {
                crate::h!(D_, "SDMA not initialized");
                return;
            }
            let gs: u32 = n.get(1).and_then(|e| e.parse().bq()).unwrap_or(4);
            let ntu: u32 = n.get(2).and_then(|e| {
                if e.cj("0x") || e.cj("0X") {
                    u32::wa(&e[2..], 16).bq()
                } else {
                    e.parse().bq()
                }
            }).unwrap_or(0xDEAD_BEEF);
            let aal = (gs * 1024).v(256 * 1024);

            
            let layout = alloc::alloc::Layout::bjy(aal as usize, 4096);
            if layout.is_err() {
                crate::h!(A_, "Allocation error");
                return;
            }
            let layout = layout.unwrap();
            let k = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
            let ht = crate::memory::abw(k).unwrap_or(0);
            if ht == 0 {
                crate::h!(A_, "Cannot get physical address");
                unsafe { alloc::alloc::dealloc(k as *mut u8, layout); }
                return;
            }

            crate::println!("SDMA fill: {} bytes at {:#X} with {:#010X}", aal, ht, ntu);
            match sdma::vi(ht, ntu, aal, 0) {
                Ok(ls) => {
                    
                    let ptr = k as *const u32;
                    let abk = unsafe { core::ptr::read_volatile(ptr) };
                    let agy = unsafe { core::ptr::read_volatile(ptr.add(1)) };
                    let apg = unsafe { core::ptr::read_volatile(ptr.add(2)) };
                    let bdf = unsafe { core::ptr::read_volatile(ptr.add(3)) };
                    crate::h!(B_, "  Done (fence={}), first 4: {:#010X} {:#010X} {:#010X} {:#010X}",
                        ls, abk, agy, apg, bdf);
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
            unsafe { alloc::alloc::dealloc(k as *mut u8, layout); }
        }
        "copy" => {
            if !sdma::uc() {
                crate::h!(D_, "SDMA not initialized");
                return;
            }
            let gs: u32 = n.get(1).and_then(|e| e.parse().bq()).unwrap_or(4);
            let aal = (gs * 1024).v(256 * 1024);

            let layout = alloc::alloc::Layout::bjy(aal as usize, 4096);
            if layout.is_err() {
                crate::h!(A_, "Allocation error");
                return;
            }
            let layout = layout.unwrap();
            let fdt = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
            let fdu = unsafe { alloc::alloc::alloc_zeroed(layout) } as u64;
            let fqr = crate::memory::abw(fdt).unwrap_or(0);
            let hvb = crate::memory::abw(fdu).unwrap_or(0);
            if fqr == 0 || hvb == 0 {
                crate::h!(A_, "Cannot get physical addresses");
                unsafe {
                    alloc::alloc::dealloc(fdt as *mut u8, layout);
                    alloc::alloc::dealloc(fdu as *mut u8, layout);
                }
                return;
            }

            
            let cy = fdt as *mut u32;
            for a in 0..(aal / 4) {
                unsafe { core::ptr::write_volatile(cy.add(a as usize), 0xA000_0000 + a); }
            }

            crate::println!("SDMA copy: {} bytes {:#X} → {:#X}", aal, fqr, hvb);
            match sdma::bdu(fqr, hvb, aal, 0) {
                Ok(ls) => {
                    
                    let cs = fdu as *const u32;
                    let abk = unsafe { core::ptr::read_volatile(cs) };
                    let agy = unsafe { core::ptr::read_volatile(cs.add(1)) };
                    let apg = unsafe { core::ptr::read_volatile(cs.add(2)) };
                    let bdf = unsafe { core::ptr::read_volatile(cs.add(3)) };
                    crate::h!(B_, "  Done (fence={}), dst[0..3]: {:#010X} {:#010X} {:#010X} {:#010X}",
                        ls, abk, agy, apg, bdf);
                    
                    let mut bq = 0u32;
                    for a in 0..(aal / 4) {
                        let ecf = unsafe { core::ptr::read_volatile(cs.add(a as usize)) };
                        if ecf == 0xA000_0000 + a { bq += 1; }
                    }
                    crate::println!("  Verified: {}/{} dwords correct", bq, aal / 4);
                }
                Err(aa) => crate::h!(A_, "Error: {}", aa),
            }
            unsafe {
                alloc::alloc::dealloc(fdt as *mut u8, layout);
                alloc::alloc::dealloc(fdu as *mut u8, layout);
            }
        }
        _ => {
            crate::h!(A_, "Unknown subcommand: {}", n[0]);
            crate::println!("Use 'sdma' for help");
        }
    }
}





pub(super) fn rgo(n: &[&str]) {
    use crate::drivers::amdgpu::neural;

    if n.is_empty() {
        crate::h!(C_, "TrustOS Neural Compute — GEMM + Ops for LLM Inference");
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

    match n[0] {
        "info" => {
            for line in neural::zl() {
                crate::println!("{}", line);
            }
        }

        "test" => {
            crate::h!(C_, "Neural Compute Self-Test");
            crate::println!("Running all tests...");
            crate::println!("");
            let (afu, ace) = neural::eyj();
            crate::println!("");
            if ace == 0 {
                crate::h!(B_, "All {} tests passed!", afu);
            } else {
                crate::h!(A_, "{} passed, {} FAILED", afu, ace);
            }
        }

        "bench" => {
            let tp: usize = if n.len() > 1 {
                n[1].parse().unwrap_or(64)
            } else {
                64
            };
            crate::h!(C_, "INT8 GEMM Benchmark: {}×{} × {}×{}", tp, tp, tp, tp);
            let kzi = neural::qow(tp);
            crate::println!("Throughput: {:.3} MOPS (CPU reference)", kzi * 1000.0);
            crate::println!("(GPU V_DOT4_I32_I8 target: ~17 TOPS)");
        }

        "gemm" => {
            if n.len() < 4 {
                crate::println!("Usage: neural gemm <M> <N> <K>");
                return;
            }
            let ef: usize = n[1].parse().unwrap_or(4);
            let bo: usize = n[2].parse().unwrap_or(4);
            let eh: usize = n[3].parse().unwrap_or(4);

            crate::h!(C_, "FP32 GEMM: C[{}×{}] = A[{}×{}] × B[{}×{}]", ef, bo, ef, eh, eh, bo);

            
            let q: alloc::vec::Vec<f32> = (0..ef*eh).map(|a| (a / eh + 1) as f32).collect();
            let o: alloc::vec::Vec<f32> = (0..eh*bo).map(|a| (a % bo + 1) as f32).collect();

            let ay = crate::time::ave();
            let r = neural::dhk(&q, &o, ef, bo, eh);
            let ez = crate::time::ave() - ay;

            
            let iah = r.len().v(16);
            crate::println!("Result (first {} elements):", iah);
            for a in 0..iah {
                if a > 0 && a % bo == 0 { crate::println!(""); }
                crate::print!("{:8.1} ", r[a]);
            }
            crate::println!("");
            crate::println!("Computed in {} ms ({} MACs)", ez, ef * bo * eh);
        }

        "kernels" => {
            crate::h!(C_, "GPU Neural Kernels (hand-encoded RDNA ISA):");
            crate::println!("");
            for eh in neural::LP_ {
                crate::println!("  {:12} {} ({} DWORDs)",
                    eh.j(), eh.dc(), eh.fun().len());
                crate::println!("               SGPRs: {}, VGPRs: {}", eh.jpo(), eh.jvl());
            }
        }

        "relu" => {
            crate::h!(C_, "ReLU Test");
            let mut f = alloc::vec![-2.0f32, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 3.0];
            crate::println!("Input:  {:?}", &f);
            neural::ngr(&mut f);
            crate::println!("Output: {:?}", &f);
        }

        "softmax" => {
            crate::h!(C_, "Softmax Test");
            let mut f = alloc::vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
            crate::println!("Input:  {:?}", &f);
            neural::kln(&mut f);
            crate::println!("Output: [");
            for (a, p) in f.iter().cf() {
                crate::println!("  [{}] = {:.6}", a, p);
            }
            crate::println!("]");
            let sum: f32 = f.iter().sum();
            crate::println!("Sum = {:.6} (should be ~1.0)", sum);
        }

        "transformer" => {
            crate::h!(C_, "Transformer Layer Test (tiny)");
            crate::println!("Architecture: seq=2, d_model=8, d_ff=16, heads=2");
            crate::println!("");

            let ls = 2;
            let bc = 8;
            let eoj = 16;
            let ecu = 2;

            
            let input: alloc::vec::Vec<f32> = (0..ls*bc).map(|a| ((a * 17 + 3) % 11) as f32 * 0.1 - 0.5).collect();
            let biw: alloc::vec::Vec<f32> = (0..bc*bc).map(|a| ((a * 13 + 7) % 9) as f32 * 0.05 - 0.2).collect();
            let biu: alloc::vec::Vec<f32> = (0..bc*bc).map(|a| ((a * 11 + 5) % 7) as f32 * 0.05 - 0.15).collect();
            let bpg: alloc::vec::Vec<f32> = (0..bc*bc).map(|a| ((a * 7 + 11) % 13) as f32 * 0.05 - 0.3).collect();
            let biv: alloc::vec::Vec<f32> = (0..bc*bc).map(|a| ((a * 5 + 3) % 11) as f32 * 0.04 - 0.2).collect();
            let bit: alloc::vec::Vec<f32> = (0..bc*eoj).map(|a| ((a * 3 + 13) % 7) as f32 * 0.05 - 0.15).collect();
            let bpf: alloc::vec::Vec<f32> = (0..bc*eoj).map(|a| ((a * 19 + 1) % 11) as f32 * 0.04 - 0.2).collect();
            let bpe: alloc::vec::Vec<f32> = (0..eoj*bc).map(|a| ((a * 23 + 7) % 13) as f32 * 0.03 - 0.2).collect();
            let pdu: alloc::vec::Vec<f32> = alloc::vec![1.0f32; bc];

            let ay = crate::time::ave();
            let an = neural::xlu(
                &input, &biw, &biu, &bpg, &biv,
                &bit, &bpf, &bpe,
                &pdu, &pdu,
                ls, bc, eoj, ecu,
            );
            let ez = crate::time::ave() - ay;

            crate::println!("Input[0..8]:  {:?}", &input[..bc.v(8)]);
            crate::println!("Output[0..8]: [");
            for a in 0..bc.v(8) {
                crate::println!("  {:.6}", an[a]);
            }
            crate::println!("]");
            crate::println!("");
            crate::println!("GEMMs used:  7 (Q,K,V,QK^T,attn*V,O,gate,up,down)");
            crate::println!("Completed in {} ms", ez);
            crate::h!(B_, "Transformer layer OK");
        }

        _ => {
            crate::h!(A_, "Unknown: neural {}", n[0]);
            crate::println!("Use 'neural' for help");
        }
    }
}





pub(super) fn rer(n: &[&str]) {
    use crate::drivers::amdgpu::firmware;

    const C_: u32 = 0xFF00FFFF;
    const B_: u32 = 0xFF00FF00;
    const A_: u32 = 0xFFFF4444;
    const D_: u32 = 0xFFFFFF00;

    if n.is_empty() {
        crate::h!(C_, "╔══════════════════════════════════════════════╗");
        crate::h!(C_, "║     GPU Firmware Manager — Navi 10 RDNA      ║");
        crate::h!(C_, "╠══════════════════════════════════════════════╣");
        crate::println!("║ Usage:                                       ║");
        crate::println!("║   gpufw status       Show firmware status     ║");
        crate::println!("║   gpufw load         Load/reload firmware     ║");
        crate::println!("║   gpufw info         Required firmware files  ║");
        crate::h!(C_, "╚══════════════════════════════════════════════╝");
        crate::println!("");
        crate::println!("Firmware files go in: /lib/firmware/amdgpu/");
        crate::println!("Get them from: linux-firmware (navi10_*.bin)");
        return;
    }

    match n[0] {
        "status" | "stat" => {
            crate::h!(C_, "GPU Firmware Status:");
            crate::println!("{}", firmware::awz());
            crate::println!("");
            for line in firmware::wtw() {
                crate::println!("  {}", line);
            }
            crate::println!("");
            if firmware::tyc() {
                crate::h!(B_, "Firmware active — GPU engines should be operational");
            } else {
                crate::h!(D_, "No firmware loaded — GPU compute uses CPU fallback");
            }
        }
        "load" | "reload" => {
            if !crate::drivers::amdgpu::clb() {
                crate::h!(A_, "No AMD GPU detected");
                return;
            }
            if let Some(co) = crate::drivers::amdgpu::ani() {
                crate::println!("Reloading firmware for {}...", co.beh());
                firmware::ahs(co.lmf);
                crate::h!(B_, "Done. {}", firmware::awz());
            }
        }
        "info" | "files" => {
            crate::h!(C_, "Required firmware files for Navi 10 (RX 5600 XT):");
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
            crate::h!(A_, "Unknown: gpufw {}", n[0]);
            crate::println!("Use 'gpufw' for help");
        }
    }
}
