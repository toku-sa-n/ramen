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
    LOOP_ON_ERROR(PrepareFilesystem(ImageHandle, SystemTable, &efi_file_system), "Failed to prepare filesystem\n");

    Print(SystemTable, (CHAR16*)L"Allocating memory...\n");
    LOOP_ON_ERROR(AllocateMemoryForOS(SystemTable), "Failed to allocate memory for OS.\n");

    Print(SystemTable, (CHAR16*)L"Initializing GOP...\n");
    EFI_GRAPHICS_OUTPUT_PROTOCOL* gop = NULL;
    LOOP_ON_ERROR(InitGop(ImageHandle, SystemTable, &gop), "Failed to initialize GOP.\n");

    Print(SystemTable, (CHAR16*)L"Hello World!\n");
    Print(SystemTable, (CHAR16*)L"Make America Great Again!\n");
    while (1)
        ;
    return EFI_SUCCESS;

#undef LOOP_ON_ERROR
}
