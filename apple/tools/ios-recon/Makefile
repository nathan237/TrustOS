# TrustOS iOS Recon Tool
# Cross-compile: make CROSS=1
# Native (on device): make

CROSS ?= 0
TARGET_ARCH = arm64

ifeq ($(CROSS),1)
    CC = xcrun -sdk iphoneos clang
    CFLAGS = -arch arm64 -isysroot $(shell xcrun --sdk iphoneos --show-sdk-path)
else
    CC = clang
    CFLAGS = -arch arm64
endif

CFLAGS += -O2 -Wall -Wextra -std=c11
CFLAGS += -DVERSION=\"0.1.0\"
LDFLAGS = -framework IOKit -framework CoreFoundation

SOURCES = src/main.c src/devtree.c src/memmap.c src/iboot_scan.c src/mmio_log.c src/serial_bridge.c src/json_out.c
OBJECTS = $(SOURCES:.c=.o)
BINARY = trustos-recon

all: $(BINARY)

$(BINARY): $(OBJECTS)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $^

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(OBJECTS) $(BINARY)

install: $(BINARY)
	scp $(BINARY) root@$(IPHONE_IP):/var/root/

.PHONY: all clean install
