#!/usr/bin/env python3
"""
TrustOS Package Server
======================
Serves a real APT-like repository over HTTP for TrustOS to download.

Usage:
    python pkg-server.py [--port 8080] [--host 0.0.0.0]

For QEMU user-mode networking, TrustOS reaches the host at 10.0.2.2.
For VirtualBox host-only, it's typically 192.168.56.1.
"""

import http.server
import json
import sys
import os
import time
import hashlib
import io
import tarfile
import threading
import urllib.request
from urllib.parse import urlparse, unquote, parse_qs

PORT = 8080
HOST = "0.0.0.0"
ALPINE_MIRROR = "https://dl-cdn.alpinelinux.org/alpine/v3.19"

# ─── Package Repository ──────────────────────────────────────────────────────
# Each package has: version, size_kb, description, depends, files
# files = dict of { install_path: file_content }

PACKAGES = {
    "vim": {
        "version": "9.0.2127-r0",
        "size_kb": 5824,
        "arch": "x86_64",
        "description": "Vi IMproved - enhanced vi editor",
        "depends": ["ncurses-libs", "vim-common"],
        "files": {
            "/usr/bin/vim": (
                "#!/bin/sh\n"
                "# vim 9.0.2127 - Vi IMproved\n"
                "# TrustOS package: vim\n"
                "echo 'Vim 9.0.2127'\n"
                "echo 'Usage: vim [options] [file ...]'\n"
                "echo '  -R  read-only mode'\n"
                "echo '  -n  no swap file'\n"
                "echo 'Commands: :q quit, :w save, :wq save+quit, i insert'\n"
            ),
            "/usr/share/vim/vimrc": (
                "\" TrustOS default vimrc\n"
                "set nocompatible\n"
                "syntax on\n"
                "set number\n"
                "set tabstop=4\n"
                "set shiftwidth=4\n"
                "set expandtab\n"
                "set autoindent\n"
                "set ruler\n"
                "set showcmd\n"
            ),
            "/usr/share/vim/help.txt": (
                "VIM - Vi IMproved 9.0 (2024 Dec 15)\n"
                "====================================\n"
                "\n"
                "Normal mode commands:\n"
                "  h j k l    - move cursor left/down/up/right\n"
                "  w b        - word forward/backward\n"
                "  0 $        - start/end of line\n"
                "  gg G       - start/end of file\n"
                "  dd yy p    - delete/yank/paste line\n"
                "  /pattern   - search forward\n"
                "  :wq        - save and quit\n"
                "  :q!        - quit without saving\n"
                "  i a o      - enter insert mode\n"
                "  ESC        - return to normal mode\n"
            ),
        }
    },
    "vim-common": {
        "version": "9.0.2127-r0",
        "size_kb": 6240,
        "arch": "x86_64",
        "description": "Vim common files",
        "depends": [],
        "files": {
            "/usr/share/vim/filetype.vim": (
                "\" Vim filetype detection\n"
                "augroup filetypedetect\n"
                "  au BufRead,BufNewFile *.rs setfiletype rust\n"
                "  au BufRead,BufNewFile *.py setfiletype python\n"
                "  au BufRead,BufNewFile *.js setfiletype javascript\n"
                "  au BufRead,BufNewFile *.sh setfiletype sh\n"
                "augroup END\n"
            ),
        }
    },
    "ncurses-libs": {
        "version": "6.4_p20231125-r0",
        "size_kb": 308,
        "arch": "x86_64",
        "description": "Ncurses libraries",
        "depends": [],
        "files": {
            "/usr/lib/libncurses.txt": "ncurses 6.4 - shared library placeholder\n",
        }
    },
    "nano": {
        "version": "7.2-r1",
        "size_kb": 680,
        "arch": "x86_64",
        "description": "Nano text editor",
        "depends": ["ncurses-libs"],
        "files": {
            "/usr/bin/nano": (
                "#!/bin/sh\n"
                "# GNU nano 7.2\n"
                "echo 'GNU nano 7.2'\n"
                "echo 'Usage: nano [options] [file]'\n"
                "echo '  Ctrl+O  save'\n"
                "echo '  Ctrl+X  exit'\n"
                "echo '  Ctrl+K  cut line'\n"
                "echo '  Ctrl+U  paste'\n"
                "echo '  Ctrl+W  search'\n"
            ),
            "/etc/nanorc": (
                "# TrustOS nanorc\n"
                "set autoindent\n"
                "set tabsize 4\n"
                "set linenumbers\n"
                "set mouse\n"
            ),
        }
    },
    "bash": {
        "version": "5.2.21-r0",
        "size_kb": 1320,
        "arch": "x86_64",
        "description": "The GNU Bourne Again shell",
        "depends": [],
        "files": {
            "/usr/bin/bash": (
                "#!/bin/sh\n"
                "# GNU bash 5.2.21\n"
                "echo 'GNU bash, version 5.2.21(1)-release (x86_64-alpine-linux-musl)'\n"
                "echo 'License GPLv3+'\n"
                "echo 'This is free software; you are free to change and redistribute it.'\n"
            ),
            "/etc/bash/bashrc": (
                "# TrustOS system-wide bashrc\n"
                "export PS1='\\u@trustos:\\w\\$ '\n"
                "alias ls='ls --color=auto'\n"
                "alias ll='ls -la'\n"
                "alias grep='grep --color=auto'\n"
            ),
        }
    },
    "curl": {
        "version": "8.5.0-r0",
        "size_kb": 440,
        "arch": "x86_64",
        "description": "URL data transfer tool",
        "depends": ["openssl", "libcurl", "ca-certificates"],
        "files": {
            "/usr/bin/curl": (
                "#!/bin/sh\n"
                "# curl 8.5.0 (x86_64-alpine-linux-musl)\n"
                "echo 'curl 8.5.0 (x86_64-alpine-linux-musl)'\n"
                "echo 'Release-Date: 2024-01-31'\n"
                "echo 'Protocols: http https ftp ftps'\n"
                "echo 'Features: SSL TLS HTTPS IPv6'\n"
            ),
        }
    },
    "wget": {
        "version": "1.21.4-r0",
        "size_kb": 480,
        "arch": "x86_64",
        "description": "Network utility to retrieve files from the Web",
        "depends": ["openssl"],
        "files": {
            "/usr/bin/wget": (
                "#!/bin/sh\n"
                "# GNU Wget 1.21.4\n"
                "echo 'GNU Wget 1.21.4'\n"
                "echo 'Usage: wget [OPTION]... [URL]...'\n"
                "echo '  -O file    save to file'\n"
                "echo '  -q         quiet mode'\n"
                "echo '  -c         resume download'\n"
            ),
        }
    },
    "git": {
        "version": "2.43.0-r0",
        "size_kb": 15360,
        "arch": "x86_64",
        "description": "Distributed version control system",
        "depends": ["openssl", "perl", "ca-certificates"],
        "files": {
            "/usr/bin/git": (
                "#!/bin/sh\n"
                "# git version 2.43.0\n"
                "echo 'git version 2.43.0'\n"
                "echo 'usage: git <command> [<args>]'\n"
                "echo ''\n"
                "echo 'Common commands:'\n"
                "echo '   clone    Clone a repository'\n"
                "echo '   init     Create an empty repository'\n"
                "echo '   add      Add file contents to index'\n"
                "echo '   commit   Record changes'\n"
                "echo '   push     Update remote refs'\n"
                "echo '   pull     Fetch and merge'\n"
                "echo '   status   Show working tree status'\n"
                "echo '   log      Show commit history'\n"
            ),
            "/etc/gitconfig": (
                "[core]\n"
                "    autocrlf = input\n"
                "    editor = vim\n"
                "[color]\n"
                "    ui = auto\n"
                "[init]\n"
                "    defaultBranch = main\n"
            ),
        }
    },
    "python3": {
        "version": "3.11.8-r0",
        "size_kb": 25600,
        "arch": "x86_64",
        "description": "High-level scripting language",
        "depends": ["libffi", "openssl"],
        "files": {
            "/usr/bin/python3": (
                "#!/bin/sh\n"
                "# Python 3.11.8\n"
                "echo 'Python 3.11.8 (Alpine Linux)'\n"
                "echo 'Type \"help\" for more information.'\n"
            ),
            "/usr/lib/python3/hello.py": (
                "#!/usr/bin/env python3\n"
                "\"\"\"Hello from TrustOS Python!\"\"\"\n"
                "\n"
                "import sys\n"
                "print(f'Hello from Python {sys.version}')\n"
                "print(f'Platform: {sys.platform}')\n"
            ),
        }
    },
    "gcc": {
        "version": "13.2.1-r1",
        "size_kb": 102400,
        "arch": "x86_64",
        "description": "GNU C compiler",
        "depends": ["binutils", "musl-dev"],
        "files": {
            "/usr/bin/gcc": (
                "#!/bin/sh\n"
                "# gcc (Alpine 13.2.1_git20231014) 13.2.1 20231014\n"
                "echo 'gcc (Alpine 13.2.1) 13.2.1 20231014'\n"
                "echo 'Target: x86_64-alpine-linux-musl'\n"
            ),
            "/usr/bin/cc": (
                "#!/bin/sh\n"
                "exec gcc \"$@\"\n"
            ),
        }
    },
    "rust": {
        "version": "1.75.0-r0",
        "size_kb": 204800,
        "arch": "x86_64",
        "description": "The Rust programming language",
        "depends": ["gcc", "musl-dev"],
        "files": {
            "/usr/bin/rustc": (
                "#!/bin/sh\n"
                "echo 'rustc 1.75.0 (82e1608df 2023-12-21)'\n"
                "echo 'Host: x86_64-unknown-linux-musl'\n"
            ),
            "/usr/bin/cargo": (
                "#!/bin/sh\n"
                "echo 'cargo 1.75.0 (1d8b05cdd 2023-11-20)'\n"
            ),
        }
    },
    "nodejs": {
        "version": "20.11.0-r0",
        "size_kb": 30720,
        "arch": "x86_64",
        "description": "JavaScript runtime built on V8",
        "depends": [],
        "files": {
            "/usr/bin/node": (
                "#!/bin/sh\n"
                "echo 'v20.11.0'\n"
            ),
            "/usr/bin/npm": (
                "#!/bin/sh\n"
                "echo 'npm 10.2.4'\n"
            ),
        }
    },
    "nginx": {
        "version": "1.24.0-r15",
        "size_kb": 1536,
        "arch": "x86_64",
        "description": "Lightweight HTTP server",
        "depends": ["openssl"],
        "files": {
            "/usr/sbin/nginx": (
                "#!/bin/sh\n"
                "echo 'nginx version: nginx/1.24.0'\n"
                "echo 'TLS SNI support enabled'\n"
            ),
            "/etc/nginx/nginx.conf": (
                "worker_processes auto;\n"
                "error_log /var/log/nginx/error.log warn;\n"
                "\n"
                "events {\n"
                "    worker_connections 1024;\n"
                "}\n"
                "\n"
                "http {\n"
                "    server {\n"
                "        listen 80;\n"
                "        server_name localhost;\n"
                "        location / {\n"
                "            root /var/www/html;\n"
                "            index index.html;\n"
                "        }\n"
                "    }\n"
                "}\n"
            ),
        }
    },
    "openssh": {
        "version": "9.6_p1-r0",
        "size_kb": 3072,
        "arch": "x86_64",
        "description": "OpenSSH server and client",
        "depends": ["openssl"],
        "files": {
            "/usr/bin/ssh": (
                "#!/bin/sh\n"
                "echo 'OpenSSH_9.6p1, OpenSSL 3.1.4'\n"
            ),
            "/usr/sbin/sshd": (
                "#!/bin/sh\n"
                "echo 'sshd: OpenSSH_9.6p1'\n"
            ),
            "/etc/ssh/sshd_config": (
                "# TrustOS sshd_config\n"
                "Port 22\n"
                "PermitRootLogin no\n"
                "PasswordAuthentication yes\n"
                "PubkeyAuthentication yes\n"
                "X11Forwarding no\n"
            ),
        }
    },
    "htop": {
        "version": "3.3.0-r0",
        "size_kb": 240,
        "arch": "x86_64",
        "description": "Interactive process viewer",
        "depends": ["ncurses-libs"],
        "files": {
            "/usr/bin/htop": (
                "#!/bin/sh\n"
                "echo 'htop 3.3.0'\n"
                "echo 'Usage: htop [-dChustv]'\n"
            ),
        }
    },
    "openssl": {
        "version": "3.1.4-r5",
        "size_kb": 7168,
        "arch": "x86_64",
        "description": "Toolkit for SSL/TLS",
        "depends": [],
        "files": {
            "/usr/bin/openssl": (
                "#!/bin/sh\n"
                "echo 'OpenSSL 3.1.4 24 Oct 2023 (Library: OpenSSL 3.1.4)'\n"
            ),
            "/usr/lib/libssl.txt": "OpenSSL 3.1.4 shared library placeholder\n",
            "/usr/lib/libcrypto.txt": "OpenSSL 3.1.4 crypto library placeholder\n",
        }
    },
    "ca-certificates": {
        "version": "20240226-r0",
        "size_kb": 640,
        "arch": "x86_64",
        "description": "Certificate authorities bundle",
        "depends": [],
        "files": {
            "/etc/ssl/certs/ca-certificates.crt": (
                "# TrustOS CA Bundle\n"
                "# Mozilla CA Certificate Store (Feb 2024)\n"
                "# Contains 143 root certificates\n"
            ),
        }
    },
    "libcurl": {
        "version": "8.5.0-r0",
        "size_kb": 768,
        "arch": "x86_64",
        "description": "cURL library",
        "depends": ["openssl"],
        "files": {
            "/usr/lib/libcurl.txt": "libcurl 8.5.0 shared library placeholder\n",
        }
    },
    "perl": {
        "version": "5.38.2-r0",
        "size_kb": 14336,
        "arch": "x86_64",
        "description": "Practical Extraction and Report Language",
        "depends": [],
        "files": {
            "/usr/bin/perl": (
                "#!/bin/sh\n"
                "echo 'This is perl 5, version 38, subversion 2 (v5.38.2)'\n"
            ),
        }
    },
    "libffi": {
        "version": "3.4.4-r3",
        "size_kb": 52,
        "arch": "x86_64",
        "description": "Portable foreign function interface library",
        "depends": [],
        "files": {
            "/usr/lib/libffi.txt": "libffi 3.4.4 shared library placeholder\n",
        }
    },
    "binutils": {
        "version": "2.41-r0",
        "size_kb": 7680,
        "arch": "x86_64",
        "description": "GNU binary utilities",
        "depends": [],
        "files": {
            "/usr/bin/ld": (
                "#!/bin/sh\n"
                "echo 'GNU ld (GNU Binutils) 2.41'\n"
            ),
            "/usr/bin/as": (
                "#!/bin/sh\n"
                "echo 'GNU assembler (GNU Binutils) 2.41'\n"
            ),
        }
    },
    "musl-dev": {
        "version": "1.2.4-r4",
        "size_kb": 1024,
        "arch": "x86_64",
        "description": "musl C library development files",
        "depends": [],
        "files": {
            "/usr/include/stdio.h": (
                "/* musl libc stdio.h */\n"
                "#ifndef _STDIO_H\n"
                "#define _STDIO_H\n"
                "/* Standard I/O functions */\n"
                "int printf(const char *fmt, ...);\n"
                "int fprintf(FILE *stream, const char *fmt, ...);\n"
                "int scanf(const char *fmt, ...);\n"
                "FILE *fopen(const char *path, const char *mode);\n"
                "int fclose(FILE *stream);\n"
                "#endif\n"
            ),
        }
    },
    "jq": {
        "version": "1.7.1-r0",
        "size_kb": 420,
        "arch": "x86_64",
        "description": "Command-line JSON processor",
        "depends": [],
        "files": {
            "/usr/bin/jq": (
                "#!/bin/sh\n"
                "echo 'jq-1.7.1'\n"
                "echo 'Usage: jq [OPTIONS...] FILTER [FILES...]'\n"
            ),
        }
    },
    "tree": {
        "version": "2.1.1-r0",
        "size_kb": 96,
        "arch": "x86_64",
        "description": "Directory listing in tree format",
        "depends": [],
        "files": {
            "/usr/bin/tree": (
                "#!/bin/sh\n"
                "echo 'tree v2.1.1'\n"
                "echo 'Usage: tree [-adfghilnpqrstuvxACDFQNSUX] [directory ...]'\n"
            ),
        }
    },
    "tmux": {
        "version": "3.3a-r5",
        "size_kb": 480,
        "arch": "x86_64",
        "description": "Terminal multiplexer",
        "depends": ["ncurses-libs"],
        "files": {
            "/usr/bin/tmux": (
                "#!/bin/sh\n"
                "echo 'tmux 3.3a'\n"
                "echo 'Usage: tmux [-2CDluvV] [-c shell] [-f file] [-L name] [-S path] [command]'\n"
            ),
        }
    },
    "neofetch": {
        "version": "7.1.0-r3",
        "size_kb": 120,
        "arch": "x86_64",
        "description": "System information tool",
        "depends": ["bash"],
        "files": {
            "/usr/bin/neofetch": (
                "#!/bin/sh\n"
                "echo '        _____       '\n"
                "echo '       /     \\\\     TrustOS v0.3.4'\n"
                "echo '      | () () |    Kernel: TrustOS Rust'\n"
                "echo '      |  ___  |    Shell: trustshell'\n"
                "echo '      | |   | |    CPU: x86_64'\n"
                "echo '       \\\\_____/     Memory: 256M'\n"
                "echo '                   Packages: (apt/apk)'\n"
            ),
        }
    },
    "make": {
        "version": "4.4.1-r2",
        "size_kb": 280,
        "arch": "x86_64",
        "description": "GNU make utility",
        "depends": [],
        "files": {
            "/usr/bin/make": (
                "#!/bin/sh\n"
                "echo 'GNU Make 4.4.1'\n"
                "echo 'Built for x86_64-alpine-linux-musl'\n"
            ),
        }
    },
}

# ─── Extra Packages (compact definitions to reach 100+) ──────────────────────

def _pkg_quick(name, version, size_kb, desc, depends=None, bin_name=None, extra_output=""):
    """Helper to create a simple package with auto-generated shell script."""
    bn = bin_name or name
    script = f"#!/bin/sh\necho '{name} {version}'\necho '{desc}'\n"
    if extra_output:
        script += extra_output + "\n"
    return {
        "version": version, "size_kb": size_kb, "arch": "x86_64",
        "description": desc, "depends": depends or [],
        "files": {f"/usr/bin/{bn}": script}
    }

_EXTRA = [
    # Shells
    ("zsh", "5.9.0-r0", 3200, "Z shell", ["ncurses-libs"], None, "echo 'Usage: zsh [options] [script]'"),
    ("fish", "3.7.0-r0", 6400, "Friendly interactive shell", ["ncurses-libs"], None, "echo 'Usage: fish [options] [command]'"),
    ("dash", "0.5.12-r0", 96, "POSIX compliant shell", [], None, ""),
    ("tcsh", "6.24.10-r0", 560, "Enhanced C shell", [], None, ""),
    ("ksh", "2020.0.0-r3", 1024, "KornShell", [], None, ""),
    # Editors
    ("emacs", "29.1-r0", 48000, "GNU Emacs editor", [], None, "echo 'C-x C-s save | C-x C-c quit | C-x C-f open'"),
    ("micro", "2.0.13-r0", 11264, "Modern terminal text editor", [], None, ""),
    ("helix", "23.10-r0", 24576, "Post-modern modal text editor", [], "hx", ""),
    ("ed", "1.20.1-r0", 52, "Classic UNIX line editor", [], None, ""),
    # Dev tools
    ("cmake", "3.27.8-r0", 9728, "Cross-platform build system", [], None, "echo 'Usage: cmake [options] <path-to-source>'"),
    ("gdb", "14.1-r0", 12800, "GNU debugger", [], None, "echo 'Usage: gdb [options] [program [core]]'"),
    ("valgrind", "3.22.0-r0", 22528, "Memory debugging tool", [], None, "echo 'Usage: valgrind [options] program [args]'"),
    ("llvm17", "17.0.5-r0", 102400, "LLVM compiler infrastructure", [], "llvm-config", ""),
    ("clang", "17.0.5-r0", 81920, "C language family frontend for LLVM", ["llvm17"], None, "echo 'Target: x86_64-alpine-linux-musl'"),
    ("go", "1.21.6-r0", 143360, "Go programming language", [], None, "echo 'Usage: go <command> [arguments]'"),
    ("ruby", "3.2.3-r0", 12288, "Ruby programming language", [], None, "echo 'Usage: ruby [options] [program] [arguments]'"),
    ("php83", "8.3.2-r0", 15360, "PHP programming language", [], "php", "echo 'Usage: php [options] [-f] <file>'"),
    ("lua5.4", "5.4.6-r2", 256, "Lua programming language", [], "lua", "echo 'Usage: lua [options] [script [args]]'"),
    ("zig", "0.11.0-r0", 51200, "Zig programming language", [], None, "echo 'Usage: zig <command> [options]'"),
    ("nim", "2.0.2-r0", 10240, "Nim programming language", [], None, "echo 'Usage: nim <command> [options] file.nim'"),
    ("openjdk17-jre", "17.0.10-r0", 204800, "OpenJDK 17 Runtime", [], "java", "echo 'Usage: java [options] <mainclass> [args]'"),
    ("elixir", "1.16.1-r0", 7680, "Elixir programming language", [], None, "echo 'Usage: elixir [options] [file]'"),
    ("R", "4.3.2-r0", 30720, "R statistical computing", [], None, "echo 'Usage: R [options] [< infile] [> outfile]'"),
    ("ghc", "9.4.8-r0", 204800, "Glasgow Haskell Compiler", [], None, "echo 'Usage: ghc [options] file.hs'"),
    ("erlang", "26.2.1-r0", 40960, "Erlang/OTP", [], "erl", ""),
    ("ocaml", "5.1.1-r0", 20480, "OCaml compiler", [], None, ""),
    ("sbcl", "2.4.0-r0", 15360, "Steel Bank Common Lisp", [], None, ""),
    ("nasm", "2.16.01-r0", 640, "Netwide Assembler", [], None, "echo 'Usage: nasm [options] file.asm'"),
    # System utilities
    ("coreutils", "9.4-r1", 6400, "GNU core utilities", [], None, "echo 'GNU coreutils 9.4'"),
    ("findutils", "4.9.0-r5", 640, "GNU find utilities", [], "find", "echo 'Usage: find [path] [expression]'"),
    ("grep", "3.11-r0", 320, "GNU grep", [], None, "echo 'Usage: grep [OPTION] PATTERN [FILE]'"),
    ("sed", "4.9-r2", 224, "GNU stream editor", [], None, "echo 'Usage: sed [OPTION] script [file]'"),
    ("gawk", "5.3.0-r0", 1024, "GNU awk", [], "awk", "echo 'Usage: awk [options] program [file]'"),
    ("diffutils", "3.10-r0", 384, "GNU diff utilities", [], "diff", "echo 'Usage: diff [OPTION] FILE1 FILE2'"),
    ("patch", "2.7.6-r10", 128, "GNU patch", [], None, "echo 'Usage: patch [options] [origfile [patchfile]]'"),
    ("file", "5.45-r1", 640, "File type identification", [], None, "echo 'Usage: file [options] file...'"),
    ("less", "643-r0", 192, "Pager program", [], None, "echo 'Usage: less [options] file...'"),
    ("which", "2.21-r4", 28, "Locate a command", [], None, ""),
    ("procps", "4.0.4-r0", 480, "Process monitoring utilities", [], "ps", "echo 'Usage: ps [options]'"),
    ("shadow", "4.14.3-r0", 480, "User/group management", [], "useradd", "echo 'Usage: useradd [options] LOGIN'"),
    ("util-linux", "2.39.3-r0", 4096, "System utilities", [], "lsblk", "echo 'Usage: lsblk [options] [device]'"),
    ("iproute2", "6.7.0-r0", 1024, "IP routing utilities", [], "ip", "echo 'Usage: ip [OPTIONS] OBJECT COMMAND'"),
    ("iptables", "1.8.10-r3", 640, "Linux firewall admin", [], None, "echo 'Usage: iptables [options]'"),
    ("net-tools", "2.10-r3", 320, "Classic networking tools", [], "ifconfig", "echo 'Usage: ifconfig [interface] [options]'"),
    ("e2fsprogs", "1.47.0-r5", 2048, "Ext2/3/4 filesystem utilities", [], "mkfs.ext4", ""),
    ("wireless-tools", "30-r0", 128, "Wireless extensions tools", [], "iwconfig", ""),
    # Databases & services
    ("docker-cli", "24.0.7-r0", 50000, "Docker container runtime", [], "docker", "echo 'Usage: docker [OPTIONS] COMMAND'"),
    ("redis", "7.2.4-r0", 4096, "In-memory data store", [], "redis-server", "echo 'Usage: redis-server [config-file]'"),
    ("postgresql16", "16.2-r0", 15360, "PostgreSQL database", [], "psql", "echo 'Usage: psql [OPTION] [DBNAME [USERNAME]]'"),
    ("mariadb", "10.11.6-r0", 25600, "MariaDB database", [], "mysql", "echo 'Usage: mysql [OPTIONS] [database]'"),
    ("sqlite", "3.44.2-r0", 1024, "SQLite database engine", [], "sqlite3", "echo 'Usage: sqlite3 [OPTIONS] [FILENAME] [SQL]'"),
    ("mongodb-tools", "100.9.4-r0", 20480, "MongoDB tools", [], "mongosh", ""),
    ("memcached", "1.6.23-r0", 256, "In-memory caching system", [], None, ""),
    # Web & network
    ("apache2", "2.4.58-r0", 5120, "Apache HTTP Server", [], "httpd", "echo 'Usage: httpd [options]'"),
    ("haproxy", "2.8.5-r0", 3072, "TCP/HTTP Load Balancer", [], None, ""),
    ("socat", "1.8.0.0-r0", 384, "Multipurpose relay", [], None, "echo 'Usage: socat [options] <address> <address>'"),
    ("nmap", "7.94-r0", 5120, "Network scanner", [], None, "echo 'Usage: nmap [options] target'"),
    ("tcpdump", "4.99.4-r1", 640, "Network packet analyzer", [], None, "echo 'Usage: tcpdump [options] [expression]'"),
    ("iperf3", "3.16-r0", 192, "Network bandwidth tool", [], None, "echo 'Usage: iperf3 [-s|-c host] [options]'"),
    ("bind-tools", "9.18.24-r0", 2048, "DNS tools", [], "dig", "echo 'Usage: dig [@server] name [type]'"),
    ("dnsmasq", "2.90-r0", 384, "DNS/DHCP server", [], None, ""),
    ("wireguard-tools", "1.0.20210914-r3", 64, "WireGuard VPN", [], "wg", "echo 'Usage: wg [command]'"),
    ("openvpn", "2.6.8-r0", 1024, "VPN solution", [], None, "echo 'Usage: openvpn [options]'"),
    ("squid", "6.6-r0", 7680, "HTTP caching proxy", [], None, ""),
    ("lynx", "2.8.9-r5", 2048, "Text-mode web browser", [], None, "echo 'Usage: lynx [options] URL'"),
    ("w3m", "0.5.3-r16", 1024, "Text-mode web browser", [], None, "echo 'Usage: w3m [options] URL'"),
    # Compression
    ("gzip", "1.13-r0", 96, "GNU zip compression", [], None, "echo 'Usage: gzip [OPTION] [FILE]'"),
    ("bzip2", "1.0.8-r6", 128, "Block-sorting compressor", [], None, "echo 'Usage: bzip2 [flags] [filenames]'"),
    ("xz", "5.4.5-r0", 256, "XZ Utils compression", [], None, "echo 'Usage: xz [OPTION] [FILE]'"),
    ("zip", "3.0-r12", 192, "Create ZIP archives", [], None, "echo 'Usage: zip [options] archive files'"),
    ("unzip", "6.0-r14", 192, "Extract ZIP archives", [], None, "echo 'Usage: unzip [options] archive'"),
    ("zstd", "1.5.5-r8", 384, "Zstandard compression", [], None, "echo 'Usage: zstd [options] [file]'"),
    ("lz4", "1.9.4-r5", 128, "Fast LZ compression", [], None, ""),
    ("p7zip", "17.05-r0", 2048, "7-Zip file archiver", [], "7z", "echo 'Usage: 7z <command> [options] archive'"),
    # Media
    ("ffmpeg", "6.1.1-r0", 20480, "Multimedia framework", [], None, "echo 'Usage: ffmpeg [options] [[infile options] -i infile]... {[outfile options] outfile}...'"),
    ("imagemagick", "7.1.1-r0", 15360, "Image manipulation", [], "convert", "echo 'Usage: convert [options] input output'"),
    ("sox", "14.4.2-r14", 1536, "Sound processing", [], None, "echo 'Usage: sox [options] infile outfile [effect]'"),
    ("mpv", "0.37.0-r0", 5120, "Media player", [], None, "echo 'Usage: mpv [options] file'"),
    # Modern CLI tools
    ("ripgrep", "14.1.0-r0", 6144, "Fast recursive grep", [], "rg", "echo 'Usage: rg [OPTIONS] PATTERN [PATH]'"),
    ("fd", "9.0.0-r0", 3072, "Fast find alternative", [], None, "echo 'Usage: fd [OPTIONS] [pattern] [path]'"),
    ("bat", "0.24.0-r0", 5120, "Cat with syntax highlighting", [], None, "echo 'Usage: bat [OPTIONS] [FILE]...'"),
    ("exa", "0.10.1-r3", 1536, "Modern ls replacement", [], None, "echo 'Usage: exa [options] [files]'"),
    ("fzf", "0.44.1-r0", 3072, "Fuzzy finder", [], None, "echo 'Usage: fzf [options]'"),
    ("delta", "0.16.5-r0", 5120, "Syntax-highlighting diff", [], None, ""),
    ("dust", "0.8.6-r0", 2048, "Disk usage viewer", [], None, "echo 'Usage: dust [options] [path]'"),
    ("bottom", "0.9.6-r0", 4096, "System monitor", [], "btm", "echo 'Usage: btm [OPTIONS]'"),
    ("procs", "0.14.4-r0", 4096, "Modern ps replacement", [], None, "echo 'Usage: procs [OPTIONS] [PATTERN]'"),
    ("tokei", "12.1.2-r4", 3072, "Code statistics", [], None, "echo 'Usage: tokei [FLAGS] [OPTIONS] [input]'"),
    ("hyperfine", "1.18.0-r0", 2048, "Benchmarking tool", [], None, "echo 'Usage: hyperfine [OPTIONS] <command>...'"),
    # VCS
    ("mercurial", "6.6.3-r0", 7680, "Mercurial VCS", [], "hg", "echo 'Usage: hg <command> [options]'"),
    ("subversion", "1.14.3-r0", 5120, "Subversion VCS", [], "svn", "echo 'Usage: svn <subcommand> [options] [args]'"),
    ("fossil", "2.23-r0", 3072, "Fossil VCS", [], None, "echo 'Usage: fossil <command> [options]'"),
    # Containers & cloud
    ("podman", "4.8.3-r0", 40960, "Daemonless container engine", [], None, "echo 'Usage: podman [options] command'"),
    ("buildah", "1.33.2-r0", 20480, "OCI image builder", [], None, "echo 'Usage: buildah [command]'"),
    ("skopeo", "1.14.2-r0", 15360, "Container image tools", [], None, "echo 'Usage: skopeo [command]'"),
    ("helm", "3.14.0-r0", 15360, "Kubernetes package manager", [], None, "echo 'Usage: helm [command]'"),
    ("kubectl", "1.29.1-r0", 20480, "Kubernetes CLI", [], None, "echo 'Usage: kubectl [command] [options]'"),
    # Scripting extras
    ("py3-pip", "23.3.2-r0", 5120, "Python package installer", ["python3"], "pip3", "echo 'Usage: pip3 install <package>'"),
    # Misc tools
    ("screen", "4.9.1-r0", 640, "Terminal multiplexer", [], None, "echo 'Usage: screen [options]'"),
    ("ltrace", "0.7.3-r8", 384, "Library call tracer", [], None, "echo 'Usage: ltrace [options] command'"),
    ("lsof", "4.99.3-r0", 320, "List open files", [], None, "echo 'Usage: lsof [options]'"),
    ("bc", "1.07.1-r4", 128, "Calculator language", [], None, "echo 'Usage: bc [options] [file]'"),
    ("cowsay", "3.04-r2", 24, "Talking cow", [], None, "echo ' ______\\necho '< moo! >\\necho ' ------\\necho '   ^__^\\necho '   (oo)_______\\necho '   (__)       )/\\necho '       ||----w|\\necho '       ||     ||'"),
    ("fortune", "0.1-r2", 1024, "Fortune cookie", [], None, "echo 'A journey of a thousand miles begins with a single step.'"),
    ("figlet", "2.2.5-r3", 128, "Large text banners", [], None, "echo ' _____          _   ___  ___\\necho '|_   _| _ _  _ | |_/ _ \\/ __|\\necho '  | || '_| || |__ __  \\__ \\\\\\necho '  |_||_|  \\_,_|_||___/|___/'"),
    ("sl", "5.05-r0", 24, "Steam locomotive", [], None, "echo '      ====        ________\\necho '  _D _|  |_______/        \\__I_I_____===__|\\necho '   |(_)---  |   H\\________/  |  |        ='"),
    ("ncdu", "2.3-r0", 192, "NCurses disk usage", [], None, "echo 'Usage: ncdu [options] [directory]'"),
    ("ranger", "1.9.3-r6", 640, "Console file manager", [], None, "echo 'Usage: ranger [options] [path]'"),
    ("mc", "4.8.31-r0", 3072, "Midnight Commander", [], None, "echo 'Usage: mc [options] [path1 [path2]]'"),
    ("ansible", "9.2.0-r0", 25600, "IT automation", [], None, "echo 'Usage: ansible [options] pattern'"),
    ("terraform", "1.7.2-r0", 81920, "Infrastructure as code", [], None, "echo 'Usage: terraform [global options] <subcommand>'"),
    ("certbot", "2.8.0-r0", 3072, "ACME client", [], None, "echo 'Usage: certbot [subcommand] [options]'"),
    ("fail2ban-server", "1.0.2-r0", 2048, "Intrusion prevention", [], "fail2ban-client", "echo 'Usage: fail2ban-client [options] <command>'"),
    ("mtr", "0.95-r2", 192, "Network diagnostic tool", [], None, "echo 'Usage: mtr [options] hostname'"),
    ("nethogs", "0.8.7-r0", 128, "Per-process bandwidth monitor", [], None, "echo 'Usage: nethogs [options] [device]'"),
    ("htop", "3.3.0-r0", 240, "Interactive process viewer", ["ncurses-libs"], None, "echo 'Usage: htop [-dChustv]'"),
    ("iotop", "0.6-r5", 96, "I/O monitoring tool", [], None, "echo 'Usage: iotop [options]'"),
    ("sysstat", "12.7.5-r0", 512, "System performance tools", [], "sar", "echo 'Usage: sar [options] [interval [count]]'"),
    ("smartmontools", "7.4-r0", 640, "S.M.A.R.T. monitoring", [], "smartctl", "echo 'Usage: smartctl [options] device'"),
]

for _name, _ver, _size, _desc, _deps, _bn, _extra in _EXTRA:
    if _name not in PACKAGES:
        PACKAGES[_name] = _pkg_quick(_name, _ver, _size, _desc, _deps, _bn or _name, _extra)

# ─── Alpine CDN Proxy ────────────────────────────────────────────────────────

class AlpineIndex:
    """Downloads and parses Alpine Linux APKINDEX for 15,000+ package discovery."""

    def __init__(self):
        self.packages = {}   # name -> {version, description, depends, size}
        self.loaded = False
        self.loading = False

    def load_async(self):
        """Start loading Alpine indices in a background thread."""
        if self.loading or self.loaded:
            return
        self.loading = True
        t = threading.Thread(target=self._load, daemon=True)
        t.start()

    def _load(self):
        for repo in ["main", "community"]:
            try:
                url = f"{ALPINE_MIRROR}/{repo}/x86_64/APKINDEX.tar.gz"
                print(f"[Alpine] Fetching {repo} index...")
                req = urllib.request.Request(url, headers={"User-Agent": "TrustOS-PkgServer/2.0"})
                data = urllib.request.urlopen(req, timeout=15).read()
                self._parse_tar(data)
                print(f"[Alpine] {repo}: {len(self.packages)} packages total")
            except Exception as e:
                print(f"[Alpine] Warning: could not load {repo}: {e}")
        self.loaded = True
        self.loading = False
        print(f"[Alpine] Index ready: {len(self.packages)} additional packages from Alpine CDN")

    def _parse_tar(self, data):
        try:
            with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as tar:
                for member in tar.getmembers():
                    if member.name == 'APKINDEX':
                        f = tar.extractfile(member)
                        if f:
                            self._parse_apkindex(f.read().decode('utf-8', errors='replace'))
        except Exception as e:
            print(f"[Alpine] Parse error: {e}")

    def _parse_apkindex(self, content):
        current = {}
        for line in content.split('\n'):
            if not line:
                if 'P' in current:
                    name = current['P']
                    if name not in PACKAGES:  # Don't override local packages
                        deps_raw = current.get('D', '')
                        deps = []
                        if deps_raw:
                            for d in deps_raw.split():
                                clean = d.split('=')[0].split('>')[0].split('<')[0].split('~')[0]
                                if clean and not clean.startswith('so:') and not clean.startswith('cmd:') and not clean.startswith('pc:'):
                                    deps.append(clean)
                        self.packages[name] = {
                            'version': current.get('V', '0'),
                            'description': current.get('T', name),
                            'depends': deps[:5],
                            'size': int(current.get('S', '0')),
                        }
                current = {}
            elif ':' in line:
                key, _, value = line.partition(':')
                current[key] = value

    def search(self, keyword, limit=100):
        """Search Alpine packages by keyword."""
        kw = keyword.lower()
        results = []
        for name, info in self.packages.items():
            if kw in name.lower() or kw in info.get('description', '').lower():
                results.append((name, info))
                if len(results) >= limit:
                    break
        return results

    def get(self, name):
        return self.packages.get(name)

    def generate_bundle(self, name):
        """Auto-generate a package bundle for any Alpine package."""
        info = self.packages.get(name)
        if not info:
            return None
        version = info['version']
        desc = info.get('description', name).replace("'", "").replace('"', '')
        bn = name
        is_lib = bn.startswith('lib') or bn.endswith('-libs')
        is_data = bn.endswith('-dev') or bn.endswith('-doc') or bn.endswith('-dbg') or bn.endswith('-static') or bn.endswith('-data') or bn.endswith('-common')

        lines_out = [f"PKG {name} {version}"]
        if is_lib:
            lines_out.append(f"FILE /usr/lib/{name}.info")
            lines_out.append(f"{name} {version} - {desc}")
        elif is_data:
            lines_out.append(f"FILE /usr/share/doc/{name}/README")
            lines_out.append(f"{name} {version}\n{desc}")
        else:
            script = f"#!/bin/sh\necho '{name} {version}'\necho '{desc}'"
            lines_out.append(f"FILE /usr/bin/{bn}")
            lines_out.append(script)
        lines_out.append("EOF")
        return '\n'.join(lines_out) + '\n'

alpine_index = AlpineIndex()

# ─── Package Bundle Format ────────────────────────────────────────────────────
# Simple text format the kernel can parse:
#   PKG <name> <version>
#   FILE <path>
#   <content lines...>
#   FILE <path>
#   <content lines...>
#   EOF

def build_package_bundle(name, pkg):
    """Build a text bundle for a package."""
    lines = [f"PKG {name} {pkg['version']}"]
    for path, content in pkg["files"].items():
        lines.append(f"FILE {path}")
        lines.append(content.rstrip('\n'))
    lines.append("EOF")
    return '\n'.join(lines) + '\n'

def build_index():
    """Build package index in simple text format."""
    lines = []
    for name, pkg in sorted(PACKAGES.items()):
        deps = ",".join(pkg["depends"]) if pkg["depends"] else "none"
        n_files = len(pkg["files"])
        lines.append(f"{name} {pkg['version']} {pkg['size_kb']} {pkg['arch']} {n_files} {deps} {pkg['description']}")
    return '\n'.join(lines) + '\n'

# ─── HTTP Handler ─────────────────────────────────────────────────────────────

class PkgHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        ts = time.strftime("%H:%M:%S")
        print(f"[{ts}] {self.address_string()} - {format % args}")

    def do_GET(self):
        path = unquote(self.path)

        # Package index
        if path in ("/repo/index", "/repo/Packages"):
            data = build_index().encode('utf-8')
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Content-Length", len(data))
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(data)
            return

        # Package search
        if path.startswith("/repo/search"):
            params = parse_qs(urlparse(self.path).query)
            keyword = params.get('q', [''])[0]
            if not keyword:
                self.send_response(400)
                self.send_header("Connection", "close")
                self.end_headers()
                self.wfile.write(b"Missing ?q= parameter\n")
                return
            results = []
            kw = keyword.lower()
            # Search local packages
            for name, pkg in sorted(PACKAGES.items()):
                if kw in name.lower() or kw in pkg['description'].lower():
                    deps = ",".join(pkg["depends"]) if pkg["depends"] else "none"
                    results.append(f"{name} {pkg['version']} {pkg['size_kb']} x86_64 {len(pkg['files'])} {deps} {pkg['description']}")
            # Search Alpine CDN index
            if alpine_index.loaded:
                for aname, ainfo in alpine_index.search(keyword, 100):
                    if not any(r.startswith(f"{aname} ") for r in results):
                        size_kb = max(ainfo['size'] // 1024, 1)
                        results.append(f"{aname} {ainfo['version']} {size_kb} x86_64 1 none {ainfo['description']}")
            data = ('\n'.join(results[:200]) + '\n').encode('utf-8') if results else b"No results\n"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Content-Length", len(data))
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(data)
            return

        # Individual package download
        if path.startswith("/repo/pool/"):
            pkg_name = path.split("/")[-1]
            # Strip .pkg or .deb extension
            for ext in (".pkg", ".deb", ".tar"):
                if pkg_name.endswith(ext):
                    pkg_name = pkg_name[:-len(ext)]
            # Also strip version suffix like "vim_9.0.2127-r0"
            base_name = pkg_name.split("_")[0] if "_" in pkg_name else pkg_name

            # Check local packages first
            if base_name in PACKAGES:
                data = build_package_bundle(base_name, PACKAGES[base_name]).encode('utf-8')
                self.send_response(200)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Content-Length", len(data))
                self.send_header("Connection", "close")
                self.end_headers()
                self.wfile.write(data)
                return

            # Alpine CDN fallback: auto-generate for any known Alpine package
            if alpine_index.loaded:
                bundle = alpine_index.generate_bundle(base_name)
                if bundle:
                    data = bundle.encode('utf-8')
                    self.log_message("Alpine auto-gen: %s", base_name)
                    self.send_response(200)
                    self.send_header("Content-Type", "application/octet-stream")
                    self.send_header("Content-Length", len(data))
                    self.send_header("Connection", "close")
                    self.end_headers()
                    self.wfile.write(data)
                    return

        # Repo info
        if path in ("/", "/repo", "/repo/"):
            alpine_n = len(alpine_index.packages) if alpine_index.loaded else 0
            info = {
                "name": "TrustOS Package Repository",
                "local_packages": len(PACKAGES),
                "alpine_packages": alpine_n,
                "total_available": len(PACKAGES) + alpine_n,
                "total_files": sum(len(p["files"]) for p in PACKAGES.values()),
            }
            data = json.dumps(info, indent=2).encode('utf-8')
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", len(data))
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(data)
            return

        # 404
        self.send_response(404)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(b"404 Not Found\n")

def main():
    port = PORT
    host = HOST
    for i, arg in enumerate(sys.argv[1:], 1):
        if arg == "--port" and i < len(sys.argv) - 1:
            port = int(sys.argv[i + 1])
        elif arg == "--host" and i < len(sys.argv) - 1:
            host = sys.argv[i + 1]

    # Start Alpine CDN index loading in background
    no_alpine = "--no-alpine" in sys.argv
    if not no_alpine:
        alpine_index.load_async()

    server = http.server.HTTPServer((host, port), PkgHandler)
    print()
    print(f"  TrustOS Package Server v2.0")
    print(f"  " + "─" * 40)
    print(f"  Local packages:  {len(PACKAGES)}")
    print(f"  Alpine CDN:      {'enabled (loading in background)' if not no_alpine else 'disabled (--no-alpine)'}")
    print(f"  Listening on:    {host}:{port}")
    print(f"  Endpoints:")
    print(f"    GET /repo/index          Package list")
    print(f"    GET /repo/pool/<n>.pkg   Download package")
    print(f"    GET /repo/search?q=<kw>  Search packages")
    print()

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down.")
        server.server_close()

if __name__ == "__main__":
    main()
