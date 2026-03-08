/*
 * TrustOS iOS Recon — TCP Serial Bridge
 *
 * Creates a TCP server on the jailbroken iPhone that accepts connections
 * and relays data to/from the physical serial port (Lightning/USB-C).
 *
 * Use case: Connect from your PC via USB→SSH tunnel to get a
 * serial debug channel without special hardware.
 *
 * Usage on iPhone:  ./trustos-recon --serial-bridge 9999
 * Usage on PC:      ssh -L 9999:localhost:9999 root@<iphone>
 *                   Then: screen localhost:9999
 *
 * The bridge also supports "virtual serial" mode where it opens
 * the UART device (/dev/ttyHSL0 or similar) for direct hardware access.
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <signal.h>
#include <termios.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <sys/select.h>

#include "recon.h"

static volatile int g_bridge_running = 1;

static void bridge_sig_handler(int sig) {
    (void)sig;
    g_bridge_running = 0;
}

/* Known UART device paths on jailbroken iOS */
static const char *uart_paths[] = {
    "/dev/tty.uart",       /* Apple default UART node */
    "/dev/ttyHSL0",        /* High-speed UART (some devices) */
    "/dev/tty.iap",        /* iAP (Lightning accessory) serial */
    "/dev/tty.serial",     /* Generic serial alias */
    "/dev/tty.BLTH",       /* Bluetooth serial */
    NULL
};

static int open_uart(int verbose) {
    for (int i = 0; uart_paths[i] != NULL; i++) {
        int fd = open(uart_paths[i], O_RDWR | O_NOCTTY | O_NONBLOCK);
        if (fd >= 0) {
            printf("[+] Opened UART: %s (fd=%d)\n", uart_paths[i], fd);
            
            /* Configure: 115200 8N1 */
            struct termios tty;
            memset(&tty, 0, sizeof(tty));
            tcgetattr(fd, &tty);
            
            cfsetispeed(&tty, B115200);
            cfsetospeed(&tty, B115200);
            
            tty.c_cflag &= ~PARENB;        /* No parity */
            tty.c_cflag &= ~CSTOPB;        /* 1 stop bit */
            tty.c_cflag &= ~CSIZE;
            tty.c_cflag |= CS8;            /* 8 data bits */
            tty.c_cflag |= CLOCAL | CREAD; /* Local, enable RX */
            tty.c_lflag = 0;               /* Raw mode */
            tty.c_iflag = 0;
            tty.c_oflag = 0;
            tty.c_cc[VMIN] = 0;
            tty.c_cc[VTIME] = 1;           /* 100ms timeout */
            
            tcsetattr(fd, TCSANOW, &tty);
            return fd;
        }
        if (verbose) {
            printf("  [try] %s — %s\n", uart_paths[i], strerror(errno));
        }
    }
    return -1;
}

static int start_tcp_server(int port) {
    int sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        perror("socket");
        return -1;
    }
    
    int opt = 1;
    setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));
    
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(port);
    
    if (bind(sockfd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        perror("bind");
        close(sockfd);
        return -1;
    }
    
    if (listen(sockfd, 1) < 0) {
        perror("listen");
        close(sockfd);
        return -1;
    }
    
    printf("[+] TCP server listening on port %d\n", port);
    return sockfd;
}

/* Bridge loop: relay data between TCP client and UART */
static void bridge_loop(int client_fd, int uart_fd) {
    char buf[4096];
    fd_set rfds;
    struct timeval tv;
    int maxfd = (client_fd > uart_fd ? client_fd : uart_fd) + 1;
    
    printf("[*] Bridge active: TCP <-> UART\n");
    printf("[*] Press Ctrl+C to stop\n\n");
    
    uint64_t rx_bytes = 0, tx_bytes = 0;
    
    while (g_bridge_running) {
        FD_ZERO(&rfds);
        FD_SET(client_fd, &rfds);
        if (uart_fd >= 0) FD_SET(uart_fd, &rfds);
        
        tv.tv_sec = 1;
        tv.tv_usec = 0;
        
        int ret = select(maxfd, &rfds, NULL, NULL, &tv);
        if (ret < 0) {
            if (errno == EINTR) continue;
            perror("select");
            break;
        }
        
        /* TCP → UART: data from PC to serial port */
        if (FD_ISSET(client_fd, &rfds)) {
            ssize_t n = read(client_fd, buf, sizeof(buf));
            if (n <= 0) {
                printf("[*] TCP client disconnected\n");
                break;
            }
            if (uart_fd >= 0) {
                write(uart_fd, buf, n);
            }
            tx_bytes += n;
        }
        
        /* UART → TCP: data from serial port to PC */
        if (uart_fd >= 0 && FD_ISSET(uart_fd, &rfds)) {
            ssize_t n = read(uart_fd, buf, sizeof(buf));
            if (n > 0) {
                write(client_fd, buf, n);
                rx_bytes += n;
            }
        }
    }
    
    printf("\n[+] Bridge stats: TX %llu bytes, RX %llu bytes\n",
           (unsigned long long)tx_bytes, (unsigned long long)rx_bytes);
}

/* Virtual serial mode: no real UART, just TCP echo + recon data */
static void virtual_bridge(int client_fd, recon_ctx_t *ctx) {
    (void)ctx;
    
    char banner[] = 
        "\r\n=== TrustOS Virtual Serial Bridge ===\r\n"
        "No hardware UART found. Running in virtual mode.\r\n"
        "Type 'info' for device info, 'quit' to disconnect.\r\n\r\n"
        "trustos-recon> ";
    
    write(client_fd, banner, strlen(banner));
    
    char line[256];
    int pos = 0;
    
    while (g_bridge_running) {
        char c;
        ssize_t n = read(client_fd, &c, 1);
        if (n <= 0) break;
        
        /* Echo character */
        write(client_fd, &c, 1);
        
        if (c == '\r' || c == '\n') {
            line[pos] = '\0';
            write(client_fd, "\r\n", 2);
            
            if (strcmp(line, "quit") == 0 || strcmp(line, "exit") == 0) {
                write(client_fd, "Bye!\r\n", 6);
                break;
            } else if (strcmp(line, "info") == 0) {
                char info[512];
                int len = snprintf(info, sizeof(info),
                    "Device: %s\r\nSoC: %s\r\nUARTs: %d\r\nAIC: %s @ 0x%llx\r\n",
                    ctx->device_model, ctx->soc_name, ctx->n_uarts,
                    ctx->aic_compatible, 
                    (unsigned long long)ctx->aic_base);
                write(client_fd, info, len);
            } else if (pos > 0) {
                char msg[] = "Unknown command. Try: info, quit\r\n";
                write(client_fd, msg, strlen(msg));
            }
            
            const char *prompt = "trustos-recon> ";
            write(client_fd, prompt, strlen(prompt));
            pos = 0;
        } else if (pos < (int)sizeof(line) - 1) {
            line[pos++] = c;
        }
    }
}

int recon_serial_bridge(recon_ctx_t *ctx, int port) {
    if (port <= 0 || port > 65535) {
        printf("[!] Invalid port: %d\n", port);
        return -1;
    }
    
    printf("[*] Starting serial bridge on port %d...\n", port);
    
    signal(SIGINT, bridge_sig_handler);
    signal(SIGPIPE, SIG_IGN);
    
    /* Try to open a real UART */
    int uart_fd = open_uart(ctx->verbose);
    if (uart_fd < 0) {
        printf("[!] No hardware UART found — using virtual serial mode\n");
        printf("[*] Virtual mode provides an interactive recon shell\n");
    }
    
    /* Start TCP server */
    int server_fd = start_tcp_server(port);
    if (server_fd < 0) {
        if (uart_fd >= 0) close(uart_fd);
        return -1;
    }
    
    printf("[*] Waiting for connection...\n");
    printf("[*] Connect with: ssh -L %d:localhost:%d root@<iphone>\n", port, port);
    printf("[*]   Then:       nc localhost %d\n", port);
    
    while (g_bridge_running) {
        struct sockaddr_in client_addr;
        socklen_t addrlen = sizeof(client_addr);
        int client_fd = accept(server_fd, (struct sockaddr *)&client_addr, &addrlen);
        
        if (client_fd < 0) {
            if (errno == EINTR) continue;
            perror("accept");
            break;
        }
        
        printf("[+] Client connected\n");
        
        if (uart_fd >= 0) {
            bridge_loop(client_fd, uart_fd);
        } else {
            virtual_bridge(client_fd, ctx);
        }
        
        close(client_fd);
        printf("[*] Client disconnected, waiting for next...\n");
    }
    
    close(server_fd);
    if (uart_fd >= 0) close(uart_fd);
    
    printf("[+] Serial bridge stopped\n");
    return 0;
}
