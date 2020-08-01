#include <stddef.h>

extern void* memcpy(void* dest, const void* src, size_t n);

void* memmove(void* dst, const void* src, size_t n)
{
    char buf[n];
    memcpy(buf, src, n);
    memcpy(dst, buf, n);

    return dst;
}
