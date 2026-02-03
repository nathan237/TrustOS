#ifndef TRUSTOS_STDIO_H
#define TRUSTOS_STDIO_H

#include <stddef.h>
#include <stdarg.h>

int vsnprintf(char *s, size_t n, const char *format, va_list arg);
int snprintf(char *s, size_t n, const char *format, ...);
int printf(const char *format, ...);

int mbedtls_vsnprintf(char *s, size_t n, const char *format, va_list arg);
int mbedtls_snprintf(char *s, size_t n, const char *format, ...);

#endif
