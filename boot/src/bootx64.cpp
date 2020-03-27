#include "efi.h"
#include "efi_utils.h"
#include "gop.h"
#include "utils.h"

static EFI_GUID kEfiLoadedImageProtocolGuid = EFI_LOADED_IMAGE_PROTOCOL_GUID;
static EFI_GUID kEfiSimpleFileSystemProtocolGuid = EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID;
static EFI_GUID kEfiFileInfoId = EFI_FILE_INFO_ID;

static EFI_PHYSICAL_ADDRESS kPhysicalAddressHeadFile = 0x0500;
static EFI_PHYSICAL_ADDRESS kPhysicalAddressKernelFile = 0x00200000;

// 0x0500 ~ 0x002FFFFF will be used by OS.
// (Strictly speaking, the range is much narrower.)
// Needed page number is 2MB / 4KB = 256 * 2;
const EFI_PHYSICAL_ADDRESS kNumPagesForOS = 256 * 3;

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
    return SystemTable->BootServices->AllocatePages(AllocateAddress, EfiLoaderData, kNumPagesForOS, &kPhysicalAddressHeadFile);
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

EFI_STATUS GetFileBytes(IN EFI_SYSTEM_TABLE* SystemTable, IN EFI_FILE_PROTOCOL* file_system, IN CHAR16* file_name, OUT EFI_PHYSICAL_ADDRESS* file_bytes)
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
        status = file_handle->GetInfo(file_handle, &kEfiFileInfoId, &info_size, file_info);
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

void Memcpy(void* dst, const void* src, size_t n)
{
    void *ptr1, *ptr2;
    size_t sz;
    __asm__ volatile("REP MOVSB;"
                     : "=D"(ptr1), "=S"(ptr2), "=c"(sz)
                     : "D"(dst), "S"(src), "c"(n)
                     : "memory");
}

EFI_STATUS ReadFileToMemory(EFI_SYSTEM_TABLE* SystemTable, IN EFI_FILE_PROTOCOL* file_system, IN CHAR16* file_name, IN VOID* address)
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

    VOID* buffer = Malloc(SystemTable, file_size);
    if (!buffer) {
        opened_file->Close(opened_file);
        return EFI_OUT_OF_RESOURCES;
    }

    EFI_STATUS return_status = opened_file->Read(opened_file, (UINTN*)&file_size, buffer);
    opened_file->Close(opened_file);

    Memcpy(address, buffer, file_size);
    Free(SystemTable, buffer);

    return return_status;
}

EFI_STATUS TerminateBootServices(IN EFI_HANDLE* ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable)
{
    UINTN size_of_memory_map = sizeof(EFI_MEMORY_DESCRIPTOR);
    EFI_MEMORY_DESCRIPTOR* memory_map = (EFI_MEMORY_DESCRIPTOR*)Malloc(SystemTable, size_of_memory_map);
    UINTN map_key, size_of_descriptor;
    UINT32 descriptor_version;

    Print(SystemTable, (CHAR16*)L"Terminating boot services...\n");

    while (SystemTable->BootServices->GetMemoryMap(&size_of_memory_map, memory_map, &map_key, &size_of_descriptor, &descriptor_version) == EFI_BUFFER_TOO_SMALL) {
        Print(SystemTable, (CHAR16*)L"Retrying...\n");
        Free(SystemTable, memory_map);
        memory_map = (EFI_MEMORY_DESCRIPTOR*)Malloc(SystemTable, size_of_memory_map);
    }
    Print(SystemTable, (CHAR16*)L"OK...\n");

    return SystemTable->BootServices->ExitBootServices(*ImageHandle, map_key);
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

void PrintMemoryContents(IN EFI_SYSTEM_TABLE* SystemTable, IN EFI_PHYSICAL_ADDRESS start, IN UINTN length)
{
    for (UINTN i = 0; i < length; i++) {
        CHAR16* str = NULL;
        OSSPrintf(str, u"%02X ", *(unsigned char*)(start + i));
        Print(SystemTable, str);
    }
    Print(SystemTable, (CHAR16*)L"\n");
}

extern "C" EFI_STATUS EFIAPI EfiMain(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable)
{
    EFI_FILE_PROTOCOL* efi_file_system = NULL;

#define EXIT_ON_ERROR(condition, message)     \
    if (EFI_ERROR(condition)) {               \
        Print(SystemTable, (CHAR16*)message); \
        while (1)                             \
            ;                                 \
    }

    Print(SystemTable, (CHAR16*)L"Preparing filesystem...\n");
    EXIT_ON_ERROR(PrepareFilesystem(ImageHandle, SystemTable, &efi_file_system), L"Failed to prepare filesystem\n");

    Print(SystemTable, (CHAR16*)L"Initializing GOP...\n");
    EFI_GRAPHICS_OUTPUT_PROTOCOL* gop = NULL;
    EXIT_ON_ERROR(InitGop(ImageHandle, SystemTable, &gop), L"Failed to initialize GOP.\n");

    EXIT_ON_ERROR(ReadFileToMemory(SystemTable, efi_file_system, (CHAR16*)L"kernel.bin", (VOID*)kPhysicalAddressKernelFile), L"Failed to read kernel image.\n");

    EXIT_ON_ERROR(ReadFileToMemory(SystemTable, efi_file_system, (CHAR16*)L"head.asm.o", (VOID*)kPhysicalAddressHeadFile), L"Failed to read head file.\n");

    SetGraphicsSettings(gop);

    PrintMemoryContents(SystemTable, 0x0FF8, 8);
    PrintMemoryContents(SystemTable, 0x00C00000, 32);
    PrintMemoryContents(SystemTable, gop->Mode->FrameBufferBase, 32);

    EXIT_ON_ERROR(TerminateBootServices(&ImageHandle, SystemTable), L"Failed to terminate boot services.\n");

    void (*jmp_to_header)(void) = (void (*)(void))0x0500;
    jmp_to_header();

    return EFI_SUCCESS;

#undef EXIT_ON_ERROR
}
