#ifndef BOOT_INCLUDE_EFI_CONSTANTS_H
#define BOOT_INCLUDE_EFI_CONSTANTS_H

// 0x0500 ~ 0x002FFFFF will be used by OS.
// (Strictly speaking, the range is much narrower.)
// Needed page number is 3MB / 4KB = 256 * 3;
const EFI_PHYSICAL_ADDRESS kNumPagesForOS = 256 * 3;

const EFI_GUID kEfiLoadedImageProtocolGuid = EFI_LOADED_IMAGE_PROTOCOL_GUID;
const EFI_GUID kEfiSimpleFileSystemProtocolGuid = EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID;
const EFI_GUID kEfiFileInfoId = EFI_FILE_INFO_ID;
const EFI_GUID kEfiGraphicsOutputProtocolGuid = EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID;

const EFI_PHYSICAL_ADDRESS kPhysicalAddressHeadFile = 0x0500;
const EFI_PHYSICAL_ADDRESS kPhysicalAddressKernelFile = 0x00200000;

#endif
