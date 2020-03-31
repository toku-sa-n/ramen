#ifndef BOOT_INCLUDE_EFI_MEMORY_H
#define BOOT_INCLUDE_EFI_MEMORY_H

EFI_STATUS AllocateMemoryForOS(IN EFI_SYSTEM_TABLE* SystemTable);
VOID* Malloc(IN EFI_SYSTEM_TABLE* SystemTable, IN EFI_PHYSICAL_ADDRESS n);
VOID Free(IN EFI_SYSTEM_TABLE* SystemTable, IN VOID* p);
void Memcpy(void* dst, const void* src, size_t n);
EFI_STATUS ReadFileToMemory(EFI_SYSTEM_TABLE* SystemTable, IN EFI_FILE_PROTOCOL* file_system, IN CHAR16* file_name, OUT VOID** address);
EFI_STATUS PrepareMemoryMap(IN EFI_SYSTEM_TABLE* SystemTable, OUT UINTN* MemoryMapSize, OUT EFI_MEMORY_DESCRIPTOR** MemoryMap, OUT UINTN* MapKey, OUT UINTN* DescriptorSize, OUT UINT32* DescriptorVersion);
void PrintMemoryContents(IN EFI_SYSTEM_TABLE* SystemTable, IN EFI_PHYSICAL_ADDRESS start, IN UINTN length);
#endif
