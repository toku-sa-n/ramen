#ifndef BOOT_SRC_UTILS__UTILS_H
#define BOOT_SRC_UTILS__UTILS_H

template <typename T>
int IntToCharsCommon(T** str, int n, int base, bool zero_flag, int digits_num, const T* numbers)
{
    T buf[1024] = { '\0' };

    int ptr = 0;
    int digits = 0;

    bool minus_flag = false;
    if (n < 0) {
        n = -n;
        minus_flag = true;
    }

    if (n == 0) {
        buf[ptr++] = '0';
        digits++;
    } else {
        while (n > 0) {
            buf[ptr++] = numbers[n % base];
            n /= base;
            digits++;
        }
    }

    if (minus_flag) {
        buf[ptr++] = '-';
        digits++;
    }

    int num_padding = digits_num - digits;
    for (int i = 0; i < num_padding; i++) {
        buf[ptr++] = (zero_flag ? '0' : ' ');
        digits++;
    }

    for (int i = 0; i < ptr / 2; i++) {
        char temp = buf[i];
        buf[i] = buf[ptr - 1 - i];
        buf[ptr - 1 - i] = temp;
    }

    for (int i = 0; i < ptr; i++) {
        *(*str)++ = buf[i];
    }

    return digits;
}

#endif
