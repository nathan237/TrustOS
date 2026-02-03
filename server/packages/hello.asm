; Minimal Linux x86_64 ELF that prints "Hello from TrustOS!"
; Compile with: nasm -f bin hello.asm -o hello
; This creates a raw ELF without external dependencies

BITS 64

; ELF Header
ehdr:
    db 0x7f, "ELF"          ; Magic number
    db 2                     ; 64-bit
    db 1                     ; Little endian
    db 1                     ; ELF version
    db 0                     ; OS/ABI (System V)
    dq 0                     ; Padding
    dw 2                     ; Executable
    dw 0x3e                  ; x86_64
    dd 1                     ; ELF version
    dq _start                ; Entry point
    dq phdr - ehdr           ; Program header offset
    dq 0                     ; Section header offset
    dd 0                     ; Flags
    dw ehdr_size             ; ELF header size
    dw phdr_size             ; Program header entry size
    dw 1                     ; Number of program headers
    dw 0                     ; Section header entry size
    dw 0                     ; Number of section headers
    dw 0                     ; Section name string table index
ehdr_size equ $ - ehdr

; Program Header
phdr:
    dd 1                     ; PT_LOAD
    dd 5                     ; Flags: R+X
    dq 0                     ; Offset in file
    dq ehdr                  ; Virtual address (0x400000 typically)
    dq ehdr                  ; Physical address
    dq file_size             ; Size in file
    dq file_size             ; Size in memory
    dq 0x1000                ; Alignment
phdr_size equ $ - phdr

; Code
_start:
    ; write(1, msg, msg_len)
    mov rax, 1               ; syscall: write
    mov rdi, 1               ; fd: stdout
    lea rsi, [rel msg]       ; buf: message
    mov rdx, msg_len         ; count
    syscall

    ; exit(0)
    mov rax, 60              ; syscall: exit
    xor rdi, rdi             ; status: 0
    syscall

msg: db "Hello from TrustOS Linux interpreter!", 10
msg_len equ $ - msg

file_size equ $ - ehdr
