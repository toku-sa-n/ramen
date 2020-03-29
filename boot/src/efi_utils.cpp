#include "efi.h"

void Print(IN EFI_SYSTEM_TABLE* SystemTable, IN CHAR16* str)
{
    while (*str) {
        if (*str == '\n') {
            SystemTable->ConOut->OutputString(SystemTable->ConOut, (CHAR16*)L"\r\n");
        } else {
            CHAR16 temp[2] = { *str, '\0' };
            SystemTable->ConOut->OutputString(SystemTable->ConOut, temp);
        }
        str++;
    }
}
