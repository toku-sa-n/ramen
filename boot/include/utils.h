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

template <unsigned QueueSize>
int Queue<QueueSize>::GetNumElements()
{
    return QueueSize - num_free_;
}

///
/// @brief sprintf function for my cpp os
/// @details This function supports these format specifiers: %%d, %%X, %%c
///
int OSSPrintf(char* str, const char* format, ...);

///
/// @brief vsprintf function for my cpp os
/// @details This function supports these format specifiers: %%d, %%X, %%c
///
int OSVSPrintf(char* str, const char* format, va_list ap);

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
