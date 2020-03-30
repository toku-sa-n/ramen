#include "efi.h"
#include "efi_constants.h"
#include "efi_memory.h"
#include "efi_utils.h"
#include "gop.h"
#include "utils.h"

EFI_STATUS PrepareFilesystem(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable, OUT EFI_FILE_PROTOCOL** efi_file_system)
{
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL* efi_fio;
    EFI_LOADED_IMAGE_PROTOCOL* efi_loaded_image_protocol;

    EFI_STATUS return_status = SystemTable->BootServices->HandleProtocol(ImageHandle, (EFI_GUID*)&kEfiLoadedImageProtocolGuid, (VOID**)&efi_loaded_image_protocol);
    if (EFI_ERROR(return_status)) {
        return return_status;
    }

    return_status = SystemTable->BootServices->HandleProtocol(efi_loaded_image_protocol->DeviceHandle, (EFI_GUID*)&kEfiSimpleFileSystemProtocolGuid, (VOID**)&efi_fio);
    if (EFI_ERROR(return_status)) {
        return return_status;
    }

    return_status = efi_fio->OpenVolume(efi_fio, efi_file_system);
    if (EFI_ERROR(return_status)) {
        return return_status;
    }

    return EFI_SUCCESS;
}

EFI_STATUS TerminateBootServices(IN EFI_HANDLE* ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable)
{
    UINTN MemoryMapSize, MapKey, DescriptorSize;
    UINT32 DescriptorVersion;
    EFI_MEMORY_DESCRIPTOR* MemoryMap;

    PrepareMemoryMap(SystemTable, &MemoryMapSize, &MemoryMap, &MapKey, &DescriptorSize, &DescriptorVersion);
    return SystemTable->BootServices->ExitBootServices(*ImageHandle, MapKey);
}

void SetGraphicsSettings(EFI_GRAPHICS_OUTPUT_PROTOCOL* gop)
{
    struct __attribute__((__packed__)) VramSettings {
        uint16_t bpp;
        uint16_t screen_x;
        uint16_t screen_y;
        uint64_t ptr;
    }* vram_settings = (VramSettings*)0x0FF2;

    vram_settings->bpp = 32;
    vram_settings->screen_x = gop->Mode->Info->HorizontalResolution;
    vram_settings->screen_y = gop->Mode->Info->VerticalResolution;
    vram_settings->ptr = gop->Mode->FrameBufferBase;
}

void AbortBooting(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable)
{
    SystemTable->BootServices->FreePages(kPhysicalAddressHeadFile, kNumPagesForOS);
    SystemTable->BootServices->Exit(ImageHandle, EFI_LOAD_ERROR, 0, NULL);
}

extern "C" EFI_STATUS EFIAPI EfiMain(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable)
{
    EFI_FILE_PROTOCOL* efi_file_system = NULL;

#define EXIT_ON_ERROR(condition, message)       \
    if (EFI_ERROR(condition)) {                 \
        Print(SystemTable, (CHAR16*)message);   \
        AbortBooting(ImageHandle, SystemTable); \
        return EFI_LOAD_ERROR;                  \
    }

    Print(SystemTable, (CHAR16*)L"Preparing filesystem...\n");
    EXIT_ON_ERROR(PrepareFilesystem(ImageHandle, SystemTable, &efi_file_system), L"Failed to prepare filesystem\n");

    Print(SystemTable, (CHAR16*)L"Initializing GOP...\n");
    EFI_GRAPHICS_OUTPUT_PROTOCOL* gop = NULL;
    EXIT_ON_ERROR(InitGop(ImageHandle, SystemTable, &gop), L"Failed to initialize GOP.\n");

    EXIT_ON_ERROR(ReadFileToMemory(SystemTable, efi_file_system, (CHAR16*)L"kernel.bin", (VOID*)kPhysicalAddressKernelFile), L"Failed to read kernel image.\n");

    EXIT_ON_ERROR(ReadFileToMemory(SystemTable, efi_file_system, (CHAR16*)L"head.asm.o", (VOID*)kPhysicalAddressHeadFile), L"Failed to read head file.\n");

    SetGraphicsSettings(gop);

    EXIT_ON_ERROR(TerminateBootServices(&ImageHandle, SystemTable), L"Failed to terminate boot services.\n");

    void (*jmp_to_header)(void) = (void (*)(void))0x0500;
    jmp_to_header();

    return EFI_SUCCESS;

#undef EXIT_ON_ERROR
}
