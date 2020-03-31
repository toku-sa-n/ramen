#include "efi.h"
#include "efi_constants.h"
#include "efi_utils.h"
#include "utils.h"

EFI_STATUS AllocateMemoryForOS(IN EFI_SYSTEM_TABLE* SystemTable)
{
    return SystemTable->BootServices->AllocatePages(AllocateAddress, EfiLoaderData, kNumPagesForOS, (EFI_PHYSICAL_ADDRESS*)&kPhysicalAddressHeadFile);
}

VOID* Malloc(IN EFI_SYSTEM_TABLE* SystemTable, IN EFI_PHYSICAL_ADDRESS n)
{
    VOID* result = NULL;
    EFI_STATUS return_status = SystemTable->BootServices->AllocatePool(EfiLoaderData, n, &result);
    return EFI_ERROR(return_status) ? NULL : result;
}

VOID Free(IN EFI_SYSTEM_TABLE* SystemTable, IN VOID* p)
{
    if (p) {
        SystemTable->BootServices->FreePool(p);
    }
}

void Memcpy(void* dst, const void* src, size_t n)
{
    void *ptr1, *ptr2;
    size_t sz;
    __asm__ volatile("REP MOVSB;"
                     : "=D"(ptr1), "=S"(ptr2), "=c"(sz)
                     : "D"(dst), "S"(src), "c"(n)
                     : "memory");
}

static EFI_STATUS GetFileBytes(IN EFI_SYSTEM_TABLE* SystemTable, IN EFI_FILE_PROTOCOL* file_system, IN CHAR16* file_name, OUT EFI_PHYSICAL_ADDRESS* file_bytes)
{
    Print(SystemTable, (CHAR16*)L"Enter GetFileBytes() function...\n");
    EFI_FILE_PROTOCOL* file_handle = NULL;
    EFI_STATUS status = file_system->Open(file_system, &file_handle, file_name, EFI_FILE_MODE_READ, 0);
    if (EFI_ERROR(status)) {
        return status;
    }

    Print(SystemTable, (CHAR16*)L"Successfully opened file...\n");

    UINTN info_size = sizeof(EFI_FILE_INFO);
    EFI_FILE_INFO* file_info;
    while (1) {
        file_info = (EFI_FILE_INFO*)Malloc(SystemTable, info_size);
        status = file_handle->GetInfo(file_handle, (EFI_GUID*)&kEfiFileInfoId, &info_size, file_info);
        if (!EFI_ERROR(status)) {
            *file_bytes = file_info->FileSize;
            CHAR16 str[1024];
            OSSPrintf(str, u"File size: %d\n", *file_bytes);
            Print(SystemTable, (CHAR16*)str);
            break;
        }

        if (status != EFI_BUFFER_TOO_SMALL) {
            CHAR16 str[1024];
            OSSPrintf(str, u"An error occurred. Error code: %d\n", status);
            Print(SystemTable, str);
            break;
        }

        Free(SystemTable, file_info);
    }

    // Close function always succeeds.
    file_handle->Close(file_handle);
    Free(SystemTable, file_info);

    return status;
}

static UINTN CalculateNumOfPagesForBytes(UINTN bytes)
{
    return (bytes + 0x1000 - 1) / 0x1000;
}

EFI_STATUS ReadFileToMemory(EFI_SYSTEM_TABLE* SystemTable, IN EFI_FILE_PROTOCOL* file_system, IN CHAR16* file_name, OUT VOID** address)
{
#define RETURN_ON_ERROR(condition)     \
    do {                               \
        EFI_STATUS STATUS = condition; \
        if (EFI_ERROR(STATUS)) {       \
            return STATUS;             \
        }                              \
    } while (0)

    UINT64 file_size = 0;
    RETURN_ON_ERROR(GetFileBytes(SystemTable, file_system, file_name, &file_size));

    EFI_FILE_PROTOCOL* opened_file = NULL;
    RETURN_ON_ERROR(file_system->Open(file_system, &opened_file, file_name, EFI_FILE_MODE_READ, 0));

#undef RETURN_ON_ERROR

    Print(SystemTable, (CHAR16*)L"File Opened.\n");

    // Though the value of parameter Memory of AllocatePages is ignored, specifying NULL as Memory will cause EFI_INVALID_PARAMETER.
    // Thus assign some random value to *address.
    *address = (VOID*)0x55aa55aa;
    EFI_STATUS status = SystemTable->BootServices->AllocatePages(AllocateAnyPages, EfiLoaderData, CalculateNumOfPagesForBytes(file_size), (EFI_PHYSICAL_ADDRESS*)*address);
    if (EFI_ERROR(status)) {
        opened_file->Close(opened_file);
        return status;
    }
    Print(SystemTable, (CHAR16*)L"File Allocated.\n");

    status = opened_file->Read(opened_file, (UINTN*)&file_size, *address);
    opened_file->Close(opened_file);

    return status;
}

EFI_STATUS PrepareMemoryMap(IN EFI_SYSTEM_TABLE* SystemTable, OUT UINTN* MemoryMapSize, OUT EFI_MEMORY_DESCRIPTOR** MemoryMap, OUT UINTN* MapKey, OUT UINTN* DescriptorSize, OUT UINT32* DescriptorVersion)
{
    *MemoryMapSize = sizeof(EFI_MEMORY_DESCRIPTOR);
    *MemoryMap = (EFI_MEMORY_DESCRIPTOR*)Malloc(SystemTable, *MemoryMapSize);

    EFI_STATUS return_status;
    while ((return_status = SystemTable->BootServices->GetMemoryMap(MemoryMapSize, *MemoryMap, MapKey, DescriptorSize, DescriptorVersion)) == EFI_BUFFER_TOO_SMALL) {
        Print(SystemTable, (CHAR16*)L"Retrying...\n");
        Free(SystemTable, *MemoryMap);
        *MemoryMap = (EFI_MEMORY_DESCRIPTOR*)Malloc(SystemTable, *MemoryMapSize);
    }

    return return_status;
}

void PrintMemoryContents(IN EFI_SYSTEM_TABLE* SystemTable, IN EFI_PHYSICAL_ADDRESS start, IN UINTN length)
{
    for (UINTN i = 0; i < length; i++) {
        CHAR16* str = NULL;
        OSSPrintf(str, u"%02X ", *(unsigned char*)(start + i));
        Print(SystemTable, str);
    }
    Print(SystemTable, (CHAR16*)L"\n");
}
