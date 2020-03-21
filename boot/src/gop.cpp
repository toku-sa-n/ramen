#include "efi.h"
#include "efi_utils.h"

static EFI_GUID kEfiEdidDiscoveredProtocolGuid = EFI_EDID_DISCOVERED_PROTOCOL_GUID;
static EFI_GUID kEfiGraphicsOutputProtocolGuid = EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID;

EFI_STATUS GetPreferredResolution(IN EFI_SYSTEM_TABLE* SystemTable, OUT UINT32* x, OUT UINT32* y)
{
    EFI_EDID_DISCOVERED_PROTOCOL* edid;
    EFI_STATUS return_status = SystemTable->BootServices->LocateProtocol(&kEfiEdidDiscoveredProtocolGuid, NULL, (VOID**)&edid);
    if (!EFI_ERROR(return_status)) {
        // See VESA E-EDID manual Table 3.1 and 3.21.
        *x = ((edid->Edid[58] & 0xF0) << 4) + edid->Edid[56];
        *y = ((edid->Edid[61] & 0xF0) << 4) + edid->Edid[59];
    }

    return return_status;
}

EFI_STATUS CheckGopInfo(IN EFI_GRAPHICS_OUTPUT_MODE_INFORMATION* info)
{
    if (info->PixelFormat != PixelBlueGreenRedReserved8BitPerColor) {
        return EFI_UNSUPPORTED;
    }

    // According to UEFI Specification 2.8 Errata A, P.479,
    // . : Pixel
    // P : Padding
    // ..........................................PPPPPPPPPP
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^|^^^^^^^^^^
    //             HorizontalResolution         | Paddings
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //                    PixelsPerScanLine
    //
    // This OS doesn't deal with pixel paddings, so return an error if pixel paddings exist.
    if (info->HorizontalResolution != info->PixelsPerScanLine) {
        return EFI_UNSUPPORTED;
    }

    return EFI_SUCCESS;
}

EFI_STATUS GetGop(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable, OUT EFI_GRAPHICS_OUTPUT_PROTOCOL** gop)
{
    // Don't replace handle_count with NULL. It won't work.
    UINTN handle_count = 0;
    EFI_HANDLE* handle_buffer;
    EFI_STATUS status = SystemTable->BootServices->LocateHandleBuffer(ByProtocol, &kEfiGraphicsOutputProtocolGuid, NULL, &handle_count, &handle_buffer);
    if (EFI_ERROR(status)) {
        return status;
    }

    return SystemTable->BootServices->OpenProtocol(handle_buffer[0], &kEfiGraphicsOutputProtocolGuid, (VOID**)gop, ImageHandle, NULL, EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL);
}

EFI_STATUS SetResolution(IN EFI_SYSTEM_TABLE* SystemTable, IN EFI_GRAPHICS_OUTPUT_PROTOCOL** gop, IN UINT32 horizontal, IN UINT32 vertical)
{
    for (UINT32 i = 0; i < (*gop)->Mode->MaxMode; i++) {
        UINTN size_of_info;
        EFI_GRAPHICS_OUTPUT_MODE_INFORMATION* info;
        (*gop)->QueryMode(*gop, i, &size_of_info, &info);
        if (!EFI_ERROR(CheckGopInfo(info)) && info->HorizontalResolution == horizontal && info->VerticalResolution == vertical) {
            (*gop)->SetMode(*gop, i);
            return EFI_SUCCESS;
        }
    }

    return EFI_UNSUPPORTED;
}

EFI_STATUS InitGop(IN EFI_HANDLE ImageHandle, IN EFI_SYSTEM_TABLE* SystemTable, OUT EFI_GRAPHICS_OUTPUT_PROTOCOL** gop)
{
#define RETURN_ON_ERROR(condition, message)       \
    do {                                          \
        EFI_STATUS STATUS = condition;            \
        if (EFI_ERROR(STATUS)) {                  \
            Print(SystemTable, (CHAR16*)message); \
            return STATUS;                        \
        }                                         \
    } while (0)

    UINT32 preferred_resolution_x = 0, preferred_resolution_y = 0;
    RETURN_ON_ERROR(GetPreferredResolution(SystemTable, &preferred_resolution_x, &preferred_resolution_y), "Error: Could not get information from EDID.\n");

    RETURN_ON_ERROR(GetGop(ImageHandle, SystemTable, gop), "Error: GOP not found.\n");

    Print(SystemTable, (CHAR16*)L"GOP Found.\n");

    RETURN_ON_ERROR(SetResolution(SystemTable, gop, preferred_resolution_x, preferred_resolution_y), "Error: Could not set preferred resolution.\n");

#undef RETURN_ON_ERROR

    return EFI_SUCCESS;
}
