///
///@file utils.h
///
///@brief This header contains miscellaneous functions.
///
#pragma once
#include <cstdarg>

///
/// @class Queue
/// @brief Ring queue
///
template <unsigned QueueSize>
class Queue {
    const int kFlagsOverRun_ = 1;

    unsigned char buf_[QueueSize];
    int next_pos_write_ = 0, next_pos_read_ = 0;
    int num_free_ = 32;
    int flags_ = 0;

public:
    ///
    /// @brief Add an element to queue.
    /// @param data Element to add.
    /// @return Return false if queue is full, otherwise return true.
    ///
    bool Enqueue(unsigned char data);

    ///
    /// @brief Get the element which is top of queue.
    /// @return Return false if queue is empty, otherwise return true.
    ///
    int Dequeue();

    ///
    /// @brief Return the number of elements in queue.
    /// @return The number of elements in queue.
    ///
    int GetNumElements();
};

template <unsigned QueueSize>
bool Queue<QueueSize>::Enqueue(unsigned char data)
{
    if (num_free_ == 0) {
        flags_ |= kFlagsOverRun_;
        return false;
    }

    buf_[next_pos_write_] = data;
    next_pos_write_ = (next_pos_write_ + 1) % QueueSize;

    num_free_--;
    return true;
}

template <unsigned QueueSize>
int Queue<QueueSize>::Dequeue()
{
    if (num_free_ == QueueSize) {
        return false;
    }

    int data = buf_[next_pos_read_];
    next_pos_read_ = (next_pos_read_ + 1) % QueueSize;

    num_free_++;
    return data;
}

template <typename T>
int IntToChars(T** str, int n, int base, bool zero_flag, int digits_num);

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

template <unsigned QueueSize>
int Queue<QueueSize>::GetNumElements()
{
    return QueueSize - num_free_;
}

template <typename T>
int OSVSPrintf(T* str, const T* format, va_list ap)
{
    int count = 0;

    while (*format != '\0') {
        if (*format != '%') {
            *str++ = *format++;
            count++;
            continue;
        }

        format++; // format points '%' so move it by 1.

        bool zero_flag = false;
        if (*format == '0') {
            zero_flag = true;
        }

        int digits_num = 0;
        while (*format >= '0' && *format <= '9') {
            digits_num *= 10;
            digits_num += *format++ - '0';
        }

        switch (*format) {
        case 'd':
            count += IntToChars<T>(&str, va_arg(ap, int), 10, zero_flag, digits_num);
            break;
        case 'X':
            count += IntToChars<T>(&str, va_arg(ap, int), 16, zero_flag, digits_num);
            break;
        case 'c':
            // Do not va_arg(ap, char). This causes a runtime error.
            count++;
            *str++ = va_arg(ap, int);
            break;
        }
        format++;
    }

    *str = '\0';
    return count;
}

template <typename T>
int OSSPrintf(T* str, const T* format, ...)
{
    va_list ap;
    va_start(ap, format);

    int count = OSVSPrintf<T>(str, format, ap);

    va_end(ap);

    return count;
}

///
/// @brief Fit a value between two values.
/// @param value Value to fit.
/// @param from Lower limit of value range.
/// @param to Upper limit of value range.
/// @return If value < from then return from. If value > to then return to. Otherwise return value.
///
template <typename T>
inline T Between(const T value, const T from, const T to)
{
    if (value < from) {
        return from;
    }

    if (value > to) {
        return to;
    }

    return value;
}
