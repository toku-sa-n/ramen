#include "../include/utils.h"
#include <cstring>
#include <iostream>
#include <stdarg.h>

const int kBufferSize = 1024;

auto ShowErrMsg = [](std::string msg, auto expected, auto actual) {
    std::cout << std::endl;
    std::cout << "--------------------------------" << std::endl;
    std::cout << msg << std::endl;
    std::cout << std::endl;
    std::cout << "Expected: " << expected << std::endl;
    std::cout << "  Actual: " << actual << std::endl;
    std::cout << "--------------------------------" << std::endl;
};

bool TestOSVSPrintf(int& num_test, int& num_success, const char* format, ...)
{
    num_test++;
    va_list ap;

    va_start(ap, format);
    char os_buf[kBufferSize] = { '\0' };
    int os_count = OSVSPrintf(os_buf, format, ap);
    va_end(ap);

    va_start(ap, format);
    char builtin_buf[kBufferSize] = { '\0' };
    int builtin_count = vsprintf(builtin_buf, format, ap);
    va_end(ap);

    if (os_count != builtin_count) {
        ShowErrMsg("Wrong return value!\n  Format: " + std::string(format), builtin_count, os_count);
        return false;
    }

    if (strcmp(os_buf, builtin_buf)) {
        ShowErrMsg("Test failed!\n  Format: " + std::string(format), builtin_buf, os_buf);
        return false;
    }

    num_success++;
    return true;
}

template <typename T>
bool TestBetween(int& num_test, int& num_success, T value, T from, T to, T expected)
{
    num_test++;
    T actual;
    if ((actual = Between(value, from, to)) == expected) {
        num_success++;
        return true;
    }

    ShowErrMsg("TestBetween failed!\n"
               "Value: "
            + std::to_string(value) + "\n"
                                      "From:  "
            + std::to_string(from) + "\n"
                                     "To:    "
            + std::to_string(to) + "\n",
        expected, actual);
    return false;
}

int main()
{
    int num_test = 0, num_success = 0;
#define TEST_OS_VSPRINTF(format, ...) \
    TestOSVSPrintf(num_test, num_success, format, ##__VA_ARGS__)
    TEST_OS_VSPRINTF("This");

    TEST_OS_VSPRINTF("%d", 0);
    TEST_OS_VSPRINTF("%d", 2);
    TEST_OS_VSPRINTF("%d", 35);
    TEST_OS_VSPRINTF("%d", 123456789);
    TEST_OS_VSPRINTF("%d", -3);
    TEST_OS_VSPRINTF("%2d", 3);
    TEST_OS_VSPRINTF("%5d", 3);
    TEST_OS_VSPRINTF("%3d%5d", 3, 5);
    TEST_OS_VSPRINTF("%03d%05d", 3, 5);

    TEST_OS_VSPRINTF("%X", 0);
    TEST_OS_VSPRINTF("%X", 1);
    TEST_OS_VSPRINTF("%X", 10);
    TEST_OS_VSPRINTF("%X", 0xABC);
    TEST_OS_VSPRINTF("This is %X pens.", 0xABC);
    TEST_OS_VSPRINTF("%3X%5X", 3, 5);

    TEST_OS_VSPRINTF("%d%X", 334, 0x345);

    TEST_OS_VSPRINTF("%c", 'a');
    TEST_OS_VSPRINTF("%c%c%c", 'R', 'G', 'B');
#undef TEST_OS_VSPRINTF

#define TEST_BETWEEN(typename, value, from, to, expected) \
    TestBetween<typename>(num_test, num_success, value, from, to, expected);

    TEST_BETWEEN(int, 3, 0, 5, 3);
    TEST_BETWEEN(int, 4, 0, 5, 4);
    TEST_BETWEEN(int, 9, 0, 5, 5);
    TEST_BETWEEN(int, -5, 0, 5, 0);

#undef TEST_BETWEEN

    std::cout << "The number of tests  : " << num_test << std::endl;
    std::cout << "The number of success: " << num_success << std::endl;
}
