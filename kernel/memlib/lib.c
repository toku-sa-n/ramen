#include <stddef.h>

void* memset(void* dest, int c, size_t n)
{
    for (size_t i = 0; i < n; i++) {
        ((char*)dest)[i] = (char)c;
    }

    return dest;
}

void* memcpy(void* dest, const void* src, size_t n)
{
    for (size_t i = 0; i < n; i++) {
        ((char*)dest)[i] = ((char*)src)[i];
    }

    return dest;
}

void* memmove(void* dst, const void* src, size_t n)
{
    if (src < dst) {
        for (size_t i = 0; i < n; i++) {
            ((char*)dst)[n - 1 - i] = ((char*)src)[n - 1 - i];
        }
    } else {
        for (size_t i = 0; i < n; i++) {
            ((char*)dst)[i] = ((char*)src)[i];
        }
    }

    return dst;
}
