#include "efi.h"

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

EFI_STATUS EFIAPI EfiMain(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable)
{
    // Prepare a filesystem.
    EFI_FILE_PROTOCOL* efi_file_system = NULL;

    if (EFI_ERROR(PrepareFilesystem(ImageHandle, SystemTable, &efi_file_system))) {
        return 0;
    }

    // Allocate memory.
    if (EFI_ERROR(SystemTable->BootServices->AllocatePages(AllocateAddress, EfiLoaderData, kNumPagesForOS, &kPhysicalAddressOS))) {
        Print(SystemTable, (CHAR16*)L"Failed to allocate memory for OS.\n");
        while (1)
            ;
    }

    // Open kernel file.
    EFI_FILE_PROTOCOL* kernel_handle = NULL;
    if (EFI_ERROR(efi_file_system->Open(efi_file_system, &kernel_handle, (CHAR16*)L"ramen_os.sys", EFI_FILE_MODE_READ, 0))) {
        Print(SystemTable, (CHAR16*)L"Could not open kernel file.\n");
    }

    Print(SystemTable, (CHAR16*)L"Hello World!\n");
    Print(SystemTable, (CHAR16*)L"Make America Great Again!\n");
    while (1)
        ;
    return EFI_SUCCESS;
}
