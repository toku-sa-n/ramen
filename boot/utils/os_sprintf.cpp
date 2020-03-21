#include "../include/utils.h"
#include <cstring>
#include <stdarg.h>

template <>
int IntToChars<char>(char** str, int n, int base, bool zero_flag, int digits_num)
{
    return IntToCharsCommon<char>(str, n, base, zero_flag, digits_num, "0123456789ABCDEF");
}

template <>
int IntToChars<char16_t>(char16_t** str, int n, int base, bool zero_flag, int digits_num)
{
    return IntToCharsCommon<char16_t>(str, n, base, zero_flag, digits_num, u"0123456789ABCDEF");
}
