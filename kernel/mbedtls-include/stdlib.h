#ifndef MBEDTLS_STDLIB_H
#define MBEDTLS_STDLIB_H

#include <stddef.h>

void *mbedtls_platform_calloc(size_t n, size_t size);
void mbedtls_platform_free(void *ptr);

void *malloc(size_t size);
void *calloc(size_t n, size_t size);
void *realloc(void *ptr, size_t size);
void free(void *ptr);

#define RAND_MAX 32767
int rand(void);
void srand(unsigned int seed);

#endif
