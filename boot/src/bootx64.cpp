#include "efi.h"
#include "efi_utils.h"
#include "gop.h"
#include "utils.h"

static EFI_GUID kEfiLoadedImageProtocolGuid = EFI_LOADED_IMAGE_PROTOCOL_GUID;
static EFI_GUID kEfiSimpleFileSystemProtocolGuid = EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID;

static EFI_PHYSICAL_ADDRESS kPhysicalAddressOS = 0x00100000;

// 0x00100000 ~ 0x002FFFFF will be used by OS.
// (Strictly speaking, the range is much narrower.)
// Needed page number is 2MB / 4KB = 256 * 2;
const EFI_PHYSICAL_ADDRESS kNumPagesForOS = 256 * 2;

EFI_STATUS PrepareFilesystem(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable, OUT EFI_FILE_PROTOCOL** efi_file_system)
{
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL* efi_fio;
    EFI_LOADED_IMAGE_PROTOCOL* efi_loaded_image_protocol;

    EFI_STATUS return_status = SystemTable->BootServices->HandleProtocol(ImageHandle, &kEfiLoadedImageProtocolGuid, (VOID**)&efi_loaded_image_protocol);
    if (EFI_ERROR(return_status)) {
        return return_status;
    }

    return_status = SystemTable->BootServices->HandleProtocol(efi_loaded_image_protocol->DeviceHandle, &kEfiSimpleFileSystemProtocolGuid, (VOID**)&efi_fio);
    if (EFI_ERROR(return_status)) {
        return return_status;
    }

    return_status = efi_fio->OpenVolume(efi_fio, efi_file_system);
    if (EFI_ERROR(return_status)) {
        return return_status;
    }

    return EFI_SUCCESS;
}

EFI_STATUS AllocateMemoryForOS(IN EFI_SYSTEM_TABLE* SystemTable)
{
    return SystemTable->BootServices->AllocatePages(AllocateAddress, EfiLoaderData, kNumPagesForOS, &kPhysicalAddressOS);
}

EFI_STATUS GetFileSize(IN EFI_FILE_PROTOCOL* file_system, IN CHAR16* file_name, OUT EFI_PHYSICAL_ADDRESS* file_size)
{
#define RETURN_ON_ERROR(condition)     \
    do {                               \
        EFI_STATUS STATUS = condition; \
        if (EFI_ERROR(STATUS)) {       \
            return STATUS;             \
        }                              \
    } while (0)

    EFI_FILE_PROTOCOL* file_handle = NULL;
    RETURN_ON_ERROR(file_system->Open(file_system, &file_handle, file_name, EFI_FILE_MODE_READ, 0));

    const UINT64 SET_POSITION_TO_EOF = 0xFFFFFFFFFFFFFFFF;
    RETURN_ON_ERROR(file_handle->SetPosition(file_handle, SET_POSITION_TO_EOF));

    RETURN_ON_ERROR(file_handle->GetPosition(file_handle, file_size));

    // Close function always succeeds.
    file_handle->Close(file_handle);

#undef RETURN_ON_ERROR

    return EFI_SUCCESS;
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

EFI_STATUS ReadFileToMemory(IN EFI_SYSTEM_TABLE* SystemTable, IN EFI_FILE_PROTOCOL* file_system, IN CHAR16* file_name, IN VOID* address)
{

#define RETURN_ON_ERROR(condition)     \
    do {                               \
        EFI_STATUS STATUS = condition; \
        if (EFI_ERROR(STATUS)) {       \
            return STATUS;             \
        }                              \
    } while (0)

    UINT64 file_size = 0;
    RETURN_ON_ERROR(GetFileSize(file_system, file_name, &file_size));

    Print(SystemTable, (CHAR16*)L"Got file size.\n");
    EFI_FILE_PROTOCOL* opened_file = NULL;
    RETURN_ON_ERROR(file_system->Open(file_system, &opened_file, file_name, EFI_FILE_MODE_READ, 0));

    Print(SystemTable, (CHAR16*)L"Opened file.\n");
#undef RETURN_ON_ERROR

    EFI_STATUS return_status = opened_file->Read(opened_file, (UINTN*)file_size, address);
    opened_file->Close(opened_file);

    return return_status;
}

extern "C" EFI_STATUS EFIAPI EfiMain(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable)
{
    EFI_FILE_PROTOCOL* efi_file_system = NULL;

#define LOOP_ON_ERROR(condition, message)     \
    if (EFI_ERROR(condition)) {               \
        Print(SystemTable, (CHAR16*)message); \
        while (1)                             \
            ;                                 \
    }

    Print(SystemTable, (CHAR16*)L"Preparing filesystem...\n");
    LOOP_ON_ERROR(PrepareFilesystem(ImageHandle, SystemTable, &efi_file_system), L"Failed to prepare filesystem\n");

    Print(SystemTable, (CHAR16*)L"Allocating memory...\n");
    LOOP_ON_ERROR(AllocateMemoryForOS(SystemTable), L"Failed to allocate memory for OS.\n");

    Print(SystemTable, (CHAR16*)L"Initializing GOP...\n");
    EFI_GRAPHICS_OUTPUT_PROTOCOL* gop = NULL;
    LOOP_ON_ERROR(InitGop(ImageHandle, SystemTable, &gop), L"Failed to initialize GOP.\n");

    LOOP_ON_ERROR(ReadFileToMemory(SystemTable, efi_file_system, (CHAR16*)L"ramen_os.img", (VOID*)0x00200000), L"Failed to read kernel image.\n");

    Print(SystemTable, (CHAR16*)L"Hello World!\n");
    Print(SystemTable, (CHAR16*)L"Make America Great Again!\n");
    while (1)
        ;
    return EFI_SUCCESS;

#undef LOOP_ON_ERROR
}
