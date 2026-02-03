#include <stdarg.h>
#include <stddef.h>

int mbedtls_printf(const char *fmt, ...) {
    (void)fmt;
    return 0;
}

int mbedtls_snprintf(char *s, size_t n, const char *fmt, ...) {
    (void)fmt;
    if (n > 0 && s) {
        s[0] = '\0';
    }
    return 0;
}

int mbedtls_vsnprintf(char *s, size_t n, const char *fmt, va_list ap) {
    (void)fmt;
    (void)ap;
    if (n > 0 && s) {
        s[0] = '\0';
    }
    return 0;
}
