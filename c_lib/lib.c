#include <stddef.h>
#include <stdio.h>

void* memcpy(void* dst, const void* src, size_t n)
{
    void* ptr = dst;
    for (size_t i = 0; i < n; i++) {
        *(char*)ptr++ = *(char*)src++;
    }

    return dst;
}

void* memmove(void* dst, const void* src, size_t n)
{
    char buf[n];
    memcpy(buf, src, n);
    memcpy(dst, buf, n);

    return dst;
}

void* memset(void* ptr, int c, size_t n)
{
    void* ret_ptr = ptr;
    for (size_t i = 0; i < n; i++) {
        *(char*)ptr++ = (char)c;
    }
    return ret_ptr;
}
