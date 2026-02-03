; TrustOS Hello World - x86_64 Linux syscall ABI
; Assemble with: nasm -f elf64 hello.asm -o hello.o
; Link with: ld -o hello hello.o

global _start

section .text
_start:
    ; sys_write(1, message, len)
    mov rax, 1          ; syscall: write
    mov rdi, 1          ; fd: stdout
    lea rsi, [rel message]
    mov rdx, message_len
    syscall

    ; sys_exit(0)
    mov rax, 60         ; syscall: exit
    xor rdi, rdi        ; exit code 0
    syscall

section .rodata
message: db "Hello from TrustOS userland!", 10
message_len equ $ - message
