#include "efi.h"

static EFI_GUID kEfiLoadedImageProtocolGuid = EFI_LOADED_IMAGE_PROTOCOL_GUID;
static EFI_GUID kEfiSimpleFileSystemProtocolGuid = EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID;

EFI_STATUS EFIAPI EfiMain(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable)
{
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL* efi_fio;
    EFI_LOADED_IMAGE_PROTOCOL* efi_loaded_image_protocol;

    // Prepare a filesystem.
    if (EFI_ERROR(SystemTable->BootServices->HandleProtocol(ImageHandle, &kEfiLoadedImageProtocolGuid, (VOID**)&efi_loaded_image_protocol))) {
        return 0;
    }

    if (EFI_ERROR(SystemTable->BootServices->HandleProtocol(efi_loaded_image_protocol->DeviceHandle, &kEfiSimpleFileSystemProtocolGuid, (VOID**)&efi_fio))) {
        return 0;
    }

    EFI_FILE_PROTOCOL* efi_file_system = NULL;

    if (EFI_ERROR(efi_fio->OpenVolume(efi_fio, &efi_file_system))) {
        return 0;
    }

    SystemTable->ConOut->OutputString(SystemTable->ConOut, (CHAR16*)L"Hello World!\r\n");
    SystemTable->ConOut->OutputString(SystemTable->ConOut, (CHAR16*)L"Make America Great Again!\r\n");
    while (1)
        ;
    return EFI_SUCCESS;
}
