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
from urllib.parse import urlparse, unquote

PORT = 8080
HOST = "0.0.0.0"

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

        # Individual package download
        if path.startswith("/repo/pool/"):
            pkg_name = path.split("/")[-1]
            # Strip .pkg or .deb extension
            for ext in (".pkg", ".deb", ".tar"):
                if pkg_name.endswith(ext):
                    pkg_name = pkg_name[:-len(ext)]
            # Also strip version suffix like "vim_9.0.2127-r0"
            base_name = pkg_name.split("_")[0] if "_" in pkg_name else pkg_name

            if base_name in PACKAGES:
                data = build_package_bundle(base_name, PACKAGES[base_name]).encode('utf-8')
                self.send_response(200)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Content-Length", len(data))
                self.send_header("Connection", "close")
                self.end_headers()
                self.wfile.write(data)
                return

        # Repo info
        if path in ("/", "/repo", "/repo/"):
            info = {
                "name": "TrustOS Package Repository",
                "packages": len(PACKAGES),
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

    server = http.server.HTTPServer((host, port), PkgHandler)
    print(f"╔══════════════════════════════════════════╗")
    print(f"║   TrustOS Package Server                 ║")
    print(f"╠══════════════════════════════════════════╣")
    print(f"║  Serving {len(PACKAGES):2d} packages on {host}:{port}     ║")
    print(f"║                                          ║")
    print(f"║  Index:   GET /repo/index                ║")
    print(f"║  Package: GET /repo/pool/<name>.pkg      ║")
    print(f"║                                          ║")
    print(f"║  QEMU user-net:   http://10.0.2.2:{port}  ║")
    print(f"║  VirtualBox:      http://192.168.56.1:{port}║")
    print(f"╚══════════════════════════════════════════╝")
    print()

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down.")
        server.server_close()

if __name__ == "__main__":
    main()
