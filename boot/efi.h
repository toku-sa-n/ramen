#ifndef BOOT_EFI
#define BOOT_EFI

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <uchar.h>

#include "efi_guid.h"

#define IN
#define OUT
#define OPTIONAL
#define EFIAPI
#define EFI_SUCCESS 0
#define EFI_UNSUPPORTED 3

#define CONST const

typedef bool BOOLEAN;
typedef void VOID;
typedef VOID* EFI_HANDLE;
typedef VOID* EFI_EVENT;

typedef char16_t CHAR16;
typedef int16_t INT16;
typedef int32_t INT32;
typedef int64_t INT64;
typedef uint8_t UINT8;
typedef uint16_t UINT16;
typedef uint32_t UINT32;
typedef uint64_t UINT64;
typedef INT64 INTN;
typedef UINT64 UINTN;
typedef UINTN EFI_STATUS;

#define EFI_FILE_MODE_READ 0x0000000000000001

#define EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL 0x00000001

#define EFI_ERROR(STATUS) (((INTN)(STATUS)) < 0)

typedef struct {
    UINT64 Signature;
    UINT32 Revision;
    UINT32 HeaderSize;
    UINT32 CRC32;
    UINT32 Reserved;
} EFI_TABLE_HEADER;

typedef struct {
    UINT16 ScanCode;
    CHAR16 UnicodeChar;
} EFI_INPUT_KEY;

typedef struct _EFI_SIMPLE_TEXT_INPUT_PROTOCOL EFI_SIMPLE_TEXT_INPUT_PROTOCOL;

typedef EFI_STATUS(EFIAPI* EFI_INPUT_RESET)(
    IN EFI_SIMPLE_TEXT_INPUT_PROTOCOL* This,
    IN BOOLEAN ExtendedVerification);

typedef EFI_STATUS(EFIAPI* EFI_INPUT_READ_KEY)(
    IN EFI_SIMPLE_TEXT_INPUT_PROTOCOL* This,
    OUT EFI_INPUT_KEY* Key);

typedef struct _EFI_SIMPLE_TEXT_INPUT_PROTOCOL {
    EFI_INPUT_RESET Reset;
    EFI_INPUT_READ_KEY ReadKeyStroke;
    EFI_EVENT WaitForKey;
} EFI_SIMPLE_TEXT_INPUT_PROTOCOL;

typedef struct _EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;

typedef EFI_STATUS(EFIAPI* EFI_TEXT_RESET)(
    IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
    IN BOOLEAN ExtendedVerification);

typedef EFI_STATUS(EFIAPI* EFI_TEXT_STRING)(
    IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
    IN CHAR16* String);

typedef EFI_STATUS(EFIAPI* EFI_TEXT_TEST_STRING)(
    IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
    IN CHAR16* String);

typedef EFI_STATUS(EFIAPI* EFI_TEXT_QUERY_MODE)(
    IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
    IN UINTN ModeNumber,
    OUT UINTN* Columns,
    OUT UINTN* Rows);

typedef EFI_STATUS(EFIAPI* EFI_TEXT_SET_MODE)(
    IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
    IN UINTN ModeNumber);

typedef EFI_STATUS(EFIAPI* EFI_TEXT_SET_ATTRIBUTE)(
    IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
    IN UINTN Attribute);

typedef EFI_STATUS(EFIAPI* EFI_TEXT_CLEAR_SCREEN)(
    IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This);

typedef EFI_STATUS(EFIAPI* EFI_TEXT_SET_CURSOR_POSITION)(
    IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
    IN UINTN Column,
    IN UINTN Row);

typedef EFI_STATUS(EFIAPI* EFI_TEXT_ENABLE_CURSOR)(
    IN EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* This,
    IN BOOLEAN Visible);

typedef struct {
    INT32 MaxMode;

    INT32 Mode;
    INT32 Attribute;
    INT32 CursorColumn;
    INT32 CursorRow;
    BOOLEAN CursorVisible;
} SIMPLE_TEXT_OUTPUT_MODE;

typedef struct _EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL {
    EFI_TEXT_RESET Reset;
    EFI_TEXT_STRING OutputString;
    EFI_TEXT_TEST_STRING TestString;
    EFI_TEXT_QUERY_MODE QueryMode;
    EFI_TEXT_SET_MODE SetMode;
    EFI_TEXT_SET_ATTRIBUTE SetAttribute;
    EFI_TEXT_CLEAR_SCREEN ClearScreen;
    EFI_TEXT_SET_CURSOR_POSITION SetCursorPosition;
    EFI_TEXT_ENABLE_CURSOR EnableCursor;
    SIMPLE_TEXT_OUTPUT_MODE* Mode;
} EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;

typedef struct {
    UINT32 Resolution;
    UINT32 Accuracy;
    BOOLEAN SetsToZero;
} EFI_TIME_CAPABILITIES;

typedef struct {
    UINT16 Year;
    UINT8 Month;
    UINT8 Day;
    UINT8 Hour;
    UINT8 Minute;
    UINT8 Second;
    UINT8 Pad1;
    UINT32 Nanosecond;
    INT16 TimeZone;
    UINT8 Daylight;
    UINT8 Pad2;
} EFI_TIME;

typedef EFI_STATUS(EFIAPI* EFI_GET_TIME)(
    OUT EFI_TIME* Time,
    OUT EFI_TIME_CAPABILITIES* Capabilities OPTIONAL);

typedef EFI_STATUS(EFIAPI* EFI_SET_TIME)(
    IN EFI_TIME* Time);

typedef EFI_STATUS(EFIAPI* EFI_GET_WAKEUP_TIME)(
    OUT BOOLEAN* Enabled,
    OUT BOOLEAN* Pending,
    OUT EFI_TIME* Time);

typedef EFI_STATUS(EFIAPI* EFI_SET_WAKEUP_TIME)(
    IN BOOLEAN Enable,
    IN EFI_TIME* Time OPTIONAL);

typedef UINT64 EFI_PHYSICAL_ADDRESS;
typedef UINT64 EFI_VIRTUAL_ADDRESS;

typedef struct {
    UINT32 Type;
    EFI_PHYSICAL_ADDRESS PhysicalStart;
    EFI_VIRTUAL_ADDRESS VirtualStart;
    UINT64 NumberOfPages;
    UINT64 Attribute;
} EFI_MEMORY_DESCRIPTOR;

typedef EFI_STATUS(EFIAPI* EFI_SET_VIRTUAL_ADDRESS_MAP)(
    IN UINTN MemoryMapSize,
    IN UINTN DescriptorSize,
    IN UINT32 DescriptorVersion,
    IN EFI_MEMORY_DESCRIPTOR* VirtualMap);

typedef EFI_STATUS(EFIAPI* EFI_CONVERT_POINTER)(
    IN UINTN DebugDisposition,
    IN VOID** Address);

typedef struct {
    UINT32 Data1;
    UINT16 Data2;
    UINT16 Data3;
    UINT8 Data4[8];
} EFI_GUID;

typedef EFI_STATUS(EFIAPI* EFI_GET_VARIABLE)(
    IN CHAR16* VariableName,
    IN EFI_GUID* VendorGuid,
    OUT UINT32* Attributes OPTIONAL,
    IN OUT UINTN* DataSize,
    OUT VOID* Data OPTIONAL);

typedef EFI_STATUS(EFIAPI* EFI_GET_NEXT_VARIABLE_NAME)(
    IN OUT UINTN* VariableNameSize,
    IN OUT CHAR16* VariableName,
    IN OUT EFI_GUID* VendorGuid);

typedef EFI_STATUS(EFIAPI* EFI_SET_VARIABLE)(
    IN CHAR16* VariableName,
    IN EFI_GUID* VendorGuid,
    IN UINT32 Attributes,
    IN UINTN DataSize,
    IN VOID* Data);

typedef EFI_STATUS(EFIAPI* EFI_GET_NEXT_HIGH_MONO_COUNT)(
    OUT UINT32* HighCount);

typedef enum {
    EfiResetCold,
    EfiResetWarm,
    EfiResetShutdown,
    EfiResetPlatformSpecific,
} EFI_RESET_TYPE;

typedef struct {
    EFI_GUID CapsuleGuid;
    UINT32 HeaderSize;
    UINT32 Flags;
    UINT32 CapsuleImageSize;
} EFI_CAPSULE_HEADER;

typedef EFI_STATUS(EFIAPI* EFI_UPDATE_CAPSULE)(
    IN EFI_CAPSULE_HEADER** CapsuleHeaderArray,
    IN UINTN CapsuleCount,
    IN EFI_PHYSICAL_ADDRESS ScatterGatherList OPTIONAL);

typedef EFI_STATUS(EFIAPI* EFI_QUERY_VARIABLE_INFO)(
    IN UINT32 Attributes,
    OUT UINT64* MaximumVariableStorageSize,
    OUT UINT64* RemainingVariableStorageSize,
    OUT UINT64* MaximumVariableSize);

typedef VOID(EFIAPI* EFI_RESET_SYSTEM)(
    IN EFI_RESET_TYPE ResetType,
    IN EFI_STATUS ResetStatus,
    IN UINTN DataSize,
    IN VOID* ResetData OPTIONAL);

typedef EFI_STATUS(EFIAPI* EFI_QUERY_CAPSULE_CAPABILITIES)(
    IN EFI_CAPSULE_HEADER** CapsuleHeaderArray,
    IN UINTN CapsuleCount,
    OUT UINT64* MaximumCapsuleSize,
    OUT EFI_RESET_TYPE* ResetType);

typedef struct {
    EFI_TABLE_HEADER Hdr;

    EFI_GET_TIME GetTime;
    EFI_SET_TIME SetTime;
    EFI_GET_WAKEUP_TIME GetWakeupTime;
    EFI_SET_WAKEUP_TIME SetWakeupTime;

    EFI_SET_VIRTUAL_ADDRESS_MAP SetVirtualAddressMap;
    EFI_CONVERT_POINTER ConvertPointer;

    EFI_GET_VARIABLE GetVariable;
    EFI_GET_NEXT_VARIABLE_NAME GetNextVariableName;
    EFI_SET_VARIABLE SetVariable;

    EFI_GET_NEXT_HIGH_MONO_COUNT GetNextHighMonotonicCount;
    EFI_RESET_SYSTEM ResetSystem;

    EFI_UPDATE_CAPSULE UpdateCapsule;
    EFI_QUERY_CAPSULE_CAPABILITIES QueryCapsuleCapabilities;

    EFI_QUERY_VARIABLE_INFO QueryVariableInfo;
} EFI_RUNTIME_SERVICES;

typedef UINTN EFI_TPL;

typedef VOID(EFIAPI* EFI_RESTORE_TPL)(
    IN EFI_TPL OldTpl);

typedef enum {
    AllocateAnyPages,
    AllocateMaxAddress,
    AllocateAddress,
    MaxAllocateType
} EFI_ALLOCATE_TYPE;

typedef enum {
    EfiReservedMemoryType,
    EfiLoaderCode,
    EfiLoaderData,
    EfiBootServicesCode,
    EfiBootServicesData,
    EfiRuntimeServicesCode,
    EfiRuntimeServicesData,
    EfiConventionalMemory,
    EfiUnusableMemory,
    EfiACPIReclaimMemory,
    EfiACPIMemoryNVS,
    EfiMemoryMappedIO,
    EfiMemoryMappedIOPortSpace,
    EfiPalCode,
    EfiPersistentMemory,
    EfiMaxMemoryType,
} EFI_MEMORY_TYPE;

typedef EFI_STATUS(EFIAPI* EFI_ALLOCATE_PAGES)(
    IN EFI_ALLOCATE_TYPE Type,
    IN EFI_MEMORY_TYPE MemoryType,
    IN UINTN Pages,
    IN OUT EFI_PHYSICAL_ADDRESS* Memory);

typedef EFI_TPL(EFIAPI* EFI_RAISE_TPL)(
    IN EFI_TPL NewTpl);

typedef EFI_STATUS(EFIAPI* EFI_FREE_PAGES)(
    IN EFI_PHYSICAL_ADDRESS Memory,
    IN UINTN Pages);

typedef EFI_STATUS(EFIAPI* EFI_GET_MEMORY_MAP)(
    IN OUT UINTN* MemoryMapSize,
    OUT EFI_MEMORY_DESCRIPTOR* MemoryMap,
    OUT UINTN* MapKey,
    OUT UINTN* DescriptorSize,
    OUT UINT32* DescriptorVersion);

typedef EFI_STATUS(EFIAPI* EFI_ALLOCATE_POOL)(
    IN EFI_MEMORY_TYPE PoolType,
    IN UINTN Size,
    OUT VOID** Buffer);

typedef EFI_STATUS(EFIAPI* EFI_FREE_POOL)(
    IN VOID* Buffer);

typedef VOID(EFIAPI* EFI_EVENT_NOTIFY)(
    IN EFI_EVENT Event,
    IN VOID* Context);

typedef EFI_STATUS(EFIAPI* EFI_CREATE_EVENT)(
    IN UINT32 Type,
    IN EFI_TPL NotifyTpl,
    IN EFI_EVENT_NOTIFY NotifyFunction OPTIONAL,
    IN VOID* NotifyContext OPTIONAL,
    OUT EFI_EVENT* Event);

typedef enum {
    TimerCancel,
    TimerPeriodic,
    TimerRelative,
} EFI_TIMER_DELAY;

typedef EFI_STATUS(EFIAPI* EFI_SET_TIMER)(
    IN EFI_EVENT Event,
    IN EFI_TIMER_DELAY Type,
    IN UINT64 TriggerTime);

typedef EFI_STATUS(EFIAPI* EFI_WAIT_FOR_EVENT)(
    IN UINTN NumberOfEvents,
    IN EFI_EVENT* Event,
    OUT UINTN* Index);

typedef EFI_STATUS(EFIAPI* EFI_SIGNAL_EVENT)(
    IN EFI_EVENT Event);

typedef EFI_STATUS(EFIAPI* EFI_CLOSE_EVENT)(
    IN EFI_EVENT Event);

typedef EFI_STATUS(EFIAPI* EFI_CHECK_EVENT)(
    IN EFI_EVENT Event);

typedef enum {
    EFI_NATIVE_INTERFACE
} EFI_INTERFACE_TYPE;

typedef EFI_STATUS(EFIAPI* EFI_INSTALL_PROTOCOL_INTERFACE)(
    IN OUT EFI_HANDLE* Handle,
    IN EFI_GUID* Protocol,
    IN EFI_INTERFACE_TYPE InterfaceType,
    IN VOID* Interface);

typedef EFI_STATUS(EFIAPI* EFI_REINSTALL_PROTOCOL_INTERFACE)(
    IN EFI_HANDLE Handle,
    IN EFI_GUID* Protocol,
    IN VOID* OldInterface,
    IN VOID* NewInterface);

typedef EFI_STATUS(EFIAPI* EFI_UNINSTALL_PROTOCOL_INTERFACE)(
    IN EFI_HANDLE Handle,
    IN EFI_GUID* Protocol,
    IN VOID* Interface);

typedef EFI_STATUS(EFIAPI* EFI_HANDLE_PROTOCOL)(
    IN EFI_HANDLE Handle,
    IN EFI_GUID* Protocol,
    OUT VOID** Interface);

typedef EFI_STATUS(EFIAPI* EFI_REGISTER_PROTOCOL_NOTIFY)(
    IN EFI_GUID* Protocol,
    IN EFI_EVENT Event,
    OUT VOID** Registration);

typedef enum {
    AllHandles,
    ByRegisterNotify,
    ByProtocol
} EFI_LOCATE_SEARCH_TYPE;

typedef struct _EFI_DEVICE_PATH_PROTOCOL {
    UINT8 Type;
    UINT8 SubType;
    UINT8 Length[2];
} EFI_DEVICE_PATH_PROTOCOL;

typedef EFI_STATUS(EFIAPI* EFI_LOCATE_HANDLE)(
    IN EFI_LOCATE_SEARCH_TYPE SearchType,
    IN EFI_GUID* Protocol OPTIONAL,
    IN VOID* SearchKey OPTIONAL,
    IN OUT UINTN* BufferSize,
    OUT EFI_HANDLE* Buffer);

typedef EFI_STATUS(EFIAPI* EFI_LOCATE_DEVICE_PATH)(
    IN EFI_GUID* Protocol,
    IN OUT EFI_DEVICE_PATH_PROTOCOL** DevicePath,
    OUT EFI_HANDLE* Device);

typedef EFI_STATUS(EFIAPI* EFI_INSTALL_CONFIGURATION_TABLE)(
    IN EFI_GUID* Guid,
    IN VOID* Table);

typedef EFI_STATUS(EFIAPI* EFI_IMAGE_START)(
    IN EFI_HANDLE ImageHandle,
    OUT UINTN* ExitDataSize,
    OUT CHAR16** ExitData OPTIONAL);

typedef EFI_STATUS(EFIAPI* EFI_EXIT)(
    IN EFI_HANDLE ImageHandle,
    IN EFI_STATUS ExitStatus,
    IN UINTN ExitDataSize,
    IN CHAR16* ExitData OPTIONAL);

typedef EFI_STATUS(EFIAPI* EFI_IMAGE_UNLOAD)(
    IN EFI_HANDLE ImageHandle);

typedef EFI_STATUS(EFIAPI* EFI_EXIT_BOOT_SERVICES)(
    IN EFI_HANDLE ImageHandle,
    IN UINTN MapKey);

typedef EFI_STATUS(EFIAPI* EFI_GET_NEXT_MONOTONIC_COUNT)(
    OUT UINT64* Count);

typedef EFI_STATUS(EFIAPI* EFI_STALL)(
    IN UINTN Microseconds);

typedef EFI_STATUS(EFIAPI* EFI_SET_WATCHDOG_TIMER)(
    IN UINTN Timeout,
    IN UINT64 WatchdogCode,
    IN UINTN DataSize,
    IN CHAR16* WatchdogData OPTIONAL);

typedef EFI_STATUS(EFIAPI* EFI_CONNECT_CONTROLLER)(
    IN EFI_HANDLE ControllerHandle,
    IN EFI_HANDLE* DriverImageHandle OPTIONAL,
    IN EFI_DEVICE_PATH_PROTOCOL* RemainingDevicePath OPTIONAL,
    IN BOOLEAN Recursive);

typedef EFI_STATUS(EFIAPI* EFI_DISCONNECT_CONTROLLER)(
    IN EFI_HANDLE ControllerHandle,
    IN EFI_HANDLE DriverImageHandle OPTIONAL,
    IN EFI_HANDLE ChildHandle OPTIONAL);

typedef EFI_STATUS(EFIAPI* EFI_OPEN_PROTOCOL)(
    IN EFI_HANDLE Handle,
    IN EFI_GUID* Protocol,
    OUT VOID** Interface OPTIONAL,
    IN EFI_HANDLE AgentHandle,
    IN EFI_HANDLE ControllerHandle,
    IN UINT32 Attributes);

typedef EFI_STATUS(EFIAPI* EFI_CLOSE_PROTOCOL)(
    IN EFI_HANDLE Handle,
    IN EFI_GUID* Protocol,
    IN EFI_HANDLE AgentHandle,
    IN EFI_HANDLE ControllerHandle);

typedef struct {
    EFI_HANDLE AgentHandle;
    EFI_HANDLE ControllerHandle;
    UINT32 Attributes;
    UINT32 OpenCount;
} EFI_OPEN_PROTOCOL_INFORMATION_ENTRY;

typedef EFI_STATUS(EFIAPI* EFI_PROTOCOLS_PER_HANDLE)(
    IN EFI_HANDLE Handle,
    OUT EFI_GUID*** ProtocolBuffer,
    OUT UINTN* ProtocolBufferCount);

typedef EFI_STATUS(EFIAPI* EFI_LOCATE_HANDLE_BUFFER)(
    IN EFI_LOCATE_SEARCH_TYPE SearchType,
    IN EFI_GUID* Protocol OPTIONAL,
    IN VOID* SearchKey OPTIONAL,
    OUT UINTN* NoHandles,
    OUT EFI_HANDLE** Buffer);

typedef EFI_STATUS(EFIAPI* EFI_LOCATE_PROTOCOL)(
    IN EFI_GUID* Protocol,
    IN VOID* Registration OPTIONAL,
    OUT VOID** Interface);

typedef EFI_STATUS(EFIAPI* EFI_INSTALL_MULTIPLE_PROTOCOL_INTERFACES)(
    IN OUT EFI_HANDLE* Handle,
    ...);

typedef EFI_STATUS(EFIAPI* EFI_OPEN_PROTOCOL_INFORMATION)(
    IN EFI_HANDLE Handle,
    IN EFI_GUID* Protocol,
    OUT EFI_OPEN_PROTOCOL_INFORMATION_ENTRY** EntryBuffer,
    OUT UINTN* EntryCount);

typedef EFI_STATUS(EFIAPI* EFI_IMAGE_LOAD)(
    IN BOOLEAN BootPolicy,
    IN EFI_HANDLE ParentImageHandle,
    IN EFI_DEVICE_PATH_PROTOCOL* DevicePath,
    IN VOID* SourceBufer OPTIONAL,
    IN UINTN SourceSize,
    OUT EFI_HANDLE* ImageHandle);

typedef EFI_STATUS(EFIAPI* EFI_UNINSTALL_MULTIPLE_PROTOCOL_INTERFACES)(
    IN EFI_HANDLE Handle,
    ...);

typedef EFI_STATUS(EFIAPI* EFI_CALCULATE_CRC32)(
    IN VOID* Data,
    IN UINTN DataSize,
    OUT UINT32* Crc32);

typedef VOID(EFIAPI* EFI_COPY_MEM)(
    IN VOID* Destination,
    IN VOID* Source,
    IN UINTN Length);

typedef VOID(EFIAPI* EFI_SET_MEM)(
    IN VOID* Buffer,
    IN UINTN Size,
    IN UINT8 Value);

typedef EFI_STATUS(EFIAPI* EFI_CREATE_EVENT_EX)(
    IN UINT32 Type,
    IN EFI_TPL NotifyTpl,
    IN EFI_EVENT_NOTIFY NotifyFunction OPTIONAL,
    IN CONST VOID* NotifyContext OPTIONAL,
    IN CONST EFI_GUID* EventGroup OPTIONAL,
    OUT EFI_EVENT* Event);

typedef struct {
    EFI_TABLE_HEADER Hdr;

    EFI_RAISE_TPL RaiseTP;
    EFI_RESTORE_TPL RestoreTPL;

    EFI_ALLOCATE_PAGES AllocatePages;
    EFI_FREE_PAGES FreePages;
    EFI_GET_MEMORY_MAP GetMemoryMap;
    EFI_ALLOCATE_POOL AllocatePool;
    EFI_FREE_POOL FreePool;

    EFI_CREATE_EVENT CreateEvent;
    EFI_SET_TIMER SetTimer;
    EFI_WAIT_FOR_EVENT WaitForEvent;
    EFI_SIGNAL_EVENT SignalEvent;
    EFI_CLOSE_EVENT CloseEvent;
    EFI_CHECK_EVENT CheckEvent;

    EFI_INSTALL_PROTOCOL_INTERFACE InstallProtocolInterface;
    EFI_REINSTALL_PROTOCOL_INTERFACE ReinstallProtocolInterface;
    EFI_UNINSTALL_PROTOCOL_INTERFACE UninstallProtocolInterface;
    EFI_HANDLE_PROTOCOL HandleProtocol;
    VOID* Reserved;
    EFI_REGISTER_PROTOCOL_NOTIFY RegisterProtocolNotify;
    EFI_LOCATE_HANDLE LocateHandle;
    EFI_LOCATE_DEVICE_PATH LocateDevicePath;
    EFI_INSTALL_CONFIGURATION_TABLE InstallConfigurationTable;

    EFI_IMAGE_LOAD LoadImage;
    EFI_IMAGE_START StartImage;
    EFI_EXIT Exit;
    EFI_IMAGE_UNLOAD UnloadImage;
    EFI_EXIT_BOOT_SERVICES ExitBootServices;

    EFI_GET_NEXT_MONOTONIC_COUNT GetNextMonotonicCount;
    EFI_STALL Stall;
    EFI_SET_WATCHDOG_TIMER SetWatchdogTimer;

    EFI_CONNECT_CONTROLLER ConnectController;
    EFI_DISCONNECT_CONTROLLER DisconnectController;

    EFI_OPEN_PROTOCOL OpenProtocol;
    EFI_CLOSE_PROTOCOL CloseProtocol;
    EFI_OPEN_PROTOCOL_INFORMATION OpenProtocolInformation;

    EFI_PROTOCOLS_PER_HANDLE ProtocolsPerHandle;
    EFI_LOCATE_HANDLE_BUFFER LocateHandleBuffer;
    EFI_LOCATE_PROTOCOL LocateProtocol;
    EFI_INSTALL_MULTIPLE_PROTOCOL_INTERFACES InstallMultipleProtocolInterfaces;
    EFI_UNINSTALL_MULTIPLE_PROTOCOL_INTERFACES UninstallMultipleProtocolInterfaces;

    EFI_CALCULATE_CRC32 CalculateCrc32;

    EFI_COPY_MEM CopyMem;
    EFI_SET_MEM SetMem;
    EFI_CREATE_EVENT_EX CreateEventEX;
} EFI_BOOT_SERVICES;

typedef struct {
    EFI_GUID VendorGuid;
    VOID* VendorTable;
} EFI_CONFIGURATION_TABLE;

typedef struct {
    EFI_TABLE_HEADER Hdr;
    CHAR16* FirmwareVendor;
    UINT32 FirmwareRevision;
    EFI_HANDLE ConsoleInHandle;
    EFI_SIMPLE_TEXT_INPUT_PROTOCOL* ConIn;
    EFI_HANDLE ConsoleOutHandle;
    EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* ConOut;
    EFI_HANDLE StandardErrorHandle;
    EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL* StdErr;
    EFI_RUNTIME_SERVICES* RuntimeServices;
    EFI_BOOT_SERVICES* BootServices;
    UINTN NumberOfTableEntries;
    EFI_CONFIGURATION_TABLE* ConfigurationTable;
} EFI_SYSTEM_TABLE;

typedef struct _EFI_SIMPLE_FILE_SYSTEM_PROTOCOL EFI_SIMPLE_FILE_SYSTEM_PROTOCOL;

typedef struct _EFI_FILE_PROTOCOL EFI_FILE_PROTOCOL;

typedef EFI_STATUS(EFIAPI* EFI_FILE_OPEN)(
    IN EFI_FILE_PROTOCOL* This,
    OUT EFI_FILE_PROTOCOL** NewHandle,
    IN CHAR16* FileName,
    IN UINT64 OpenMode,
    IN UINT64 Attributes);

typedef EFI_STATUS(EFIAPI* EFI_FILE_CLOSE)(
    IN EFI_FILE_PROTOCOL* This);

typedef EFI_STATUS(EFIAPI* EFI_FILE_DELETE)(
    IN EFI_FILE_PROTOCOL* This);

typedef EFI_STATUS(EFIAPI* EFI_FILE_READ)(
    IN EFI_FILE_PROTOCOL* This,
    IN OUT UINTN* BufferSize,
    OUT VOID* Buffer);

typedef EFI_STATUS(EFIAPI* EFI_FILE_WRITE)(
    IN EFI_FILE_PROTOCOL* This,
    IN OUT UINTN* BufferSize,
    IN VOID* Buffer);

typedef EFI_STATUS(EFIAPI* EFI_FILE_GET_POSITION)(
    IN EFI_FILE_PROTOCOL* This,
    OUT UINT64* Position);

typedef EFI_STATUS(EFIAPI* EFI_FILE_SET_POSITION)(
    IN EFI_FILE_PROTOCOL* This,
    IN UINT64 Position);

typedef EFI_STATUS(EFIAPI* EFI_FILE_GET_INFO)(
    IN EFI_FILE_PROTOCOL* This,
    IN EFI_GUID* InformationType,
    IN OUT UINTN* BufferSize,
    OUT VOID* Buffer);

typedef EFI_STATUS(EFIAPI* EFI_FILE_SET_INFO)(
    IN EFI_FILE_PROTOCOL* This,
    IN EFI_GUID* InformationType,
    IN UINTN BufferSize,
    IN VOID* Buffer);

typedef EFI_STATUS(EFIAPI* EFI_FILE_FLUSH)(
    IN EFI_FILE_PROTOCOL* This);

typedef struct {
    EFI_EVENT Event;
    EFI_STATUS Status;
    UINTN BufferSize;
    VOID* Buffer;
} EFI_FILE_IO_TOKEN;

typedef EFI_STATUS(EFIAPI* EFI_FILE_OPEN_EX)(
    IN EFI_FILE_PROTOCOL* This,
    OUT EFI_FILE_PROTOCOL** NewHandle,
    IN CHAR16* FileName,
    IN UINT64 OpenMode,
    IN UINT64 Attributes,
    IN OUT EFI_FILE_IO_TOKEN* Token);

typedef EFI_STATUS(EFIAPI* EFI_FILE_READ_EX)(
    IN EFI_FILE_PROTOCOL* This,
    IN OUT EFI_FILE_IO_TOKEN* Token);

typedef EFI_STATUS(EFIAPI* EFI_FILE_WRITE_EX)(
    IN EFI_FILE_PROTOCOL* This,
    IN OUT EFI_FILE_IO_TOKEN* Token);

typedef EFI_STATUS(EFIAPI* EFI_FILE_FLUSH_EX)(
    IN EFI_FILE_PROTOCOL* This,
    IN OUT EFI_FILE_IO_TOKEN* Token);

typedef struct _EFI_FILE_PROTOCOL {
    UINT64 Revision;
    EFI_FILE_OPEN Open;
    EFI_FILE_CLOSE Close;
    EFI_FILE_DELETE Delete;
    EFI_FILE_READ Read;
    EFI_FILE_WRITE Write;
    EFI_FILE_GET_POSITION GetPosition;
    EFI_FILE_SET_POSITION SetPosition;
    EFI_FILE_GET_INFO GetInfo;
    EFI_FILE_SET_INFO SetInfo;
    EFI_FILE_FLUSH Flush;
    EFI_FILE_OPEN_EX OpenEx;
    EFI_FILE_READ_EX ReadEx;
    EFI_FILE_WRITE_EX WriteEx;
    EFI_FILE_FLUSH_EX FlushEx;
} EFI_FILE_PROTOCOL;

typedef EFI_STATUS(EFIAPI* EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_OPEN_VOLUME)(
    IN EFI_SIMPLE_FILE_SYSTEM_PROTOCOL* This,
    OUT EFI_FILE_PROTOCOL** Root);

typedef struct _EFI_SIMPLE_FILE_SYSTEM_PROTOCOL {
    UINT64 Revision;
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_OPEN_VOLUME OpenVolume;
} EFI_SIMPLE_FILE_SYSTEM_PROTOCOL;

typedef struct {
    UINT32 Revision;
    EFI_HANDLE ParentHandle;
    EFI_SYSTEM_TABLE* SystemTable;

    EFI_HANDLE DeviceHandle;
    EFI_DEVICE_PATH_PROTOCOL* FilePath;
    VOID* Reserved;

    UINT32 LoadOptionsSize;
    VOID* LoadOptions;

    VOID* ImageBase;
    UINT64 ImageSize;
    EFI_MEMORY_TYPE ImageCodeType;
    EFI_MEMORY_TYPE ImageDataType;
    EFI_IMAGE_UNLOAD Unload;
} EFI_LOADED_IMAGE_PROTOCOL;

typedef struct EFI_GRAPHICS_OUTPUT_PROTOCOL EFI_GRAPHICS_OUTPUT_PROTOCOL;

typedef enum {
    PixelRedGreenBlueReserved8BitPerColor,
    PixelBlueGreenRedReserved8BitPerColor,
    PixelBitMask,
    PixelBitOnly,
    PixelFormatMax,
} EFI_GRAPHICS_PIXEL_FORMAT;

typedef struct {
    UINT32 RedMask;
    UINT32 GreenMask;
    UINT32 BlueMask;
    UINT32 ReservedMask;
} EFI_PIXEL_BITMASK;

typedef struct {
    UINT32 Version;
    UINT32 HorizontalResolution;
    UINT32 VerticalResolution;
    EFI_GRAPHICS_PIXEL_FORMAT PixelFormat;
    EFI_PIXEL_BITMASK PixelInformation;
    UINT32 PixelsPerScanLine;
} EFI_GRAPHICS_OUTPUT_MODE_INFORMATION;

typedef EFI_STATUS(EFIAPI* EFI_GRAPHICS_OUTPUT_PROTOCOL_QUERY_MODE)(
    IN EFI_GRAPHICS_OUTPUT_PROTOCOL* This,
    IN UINT32 ModeNumber,
    OUT UINTN* SizeOfInfo,
    OUT EFI_GRAPHICS_OUTPUT_MODE_INFORMATION** Info);

typedef EFI_STATUS(EFIAPI* EFI_GRAPHICS_OUTPUT_PROTOCOL_SET_MODE)(
    IN EFI_GRAPHICS_OUTPUT_PROTOCOL* This,
    IN UINT32 ModeNumber);

typedef struct {
    UINT8 Blue;
    UINT8 Green;
    UINT8 Red;
    UINT8 Reserved;
} EFI_GRAPHICS_OUTPUT_BLT_PIXEL;

typedef enum {
    EfiBltVideoFill,
    EfiBltVideoToBltBuffer,
    EfiBltBufferToVideo,
    EfiBltVideoToVideo,
    EfiGraphicsOutputBitOperationMax,
} EFI_GRAPHICS_OUTPUT_BLT_OPERATION;

typedef EFI_STATUS(EFIAPI* EFI_GRAPHICS_OUTPUT_PROTOCOL_BLT)(
    IN EFI_GRAPHICS_OUTPUT_PROTOCOL* This,
    IN OUT EFI_GRAPHICS_OUTPUT_BLT_PIXEL* BltBuffer OPTIONAL,
    IN EFI_GRAPHICS_OUTPUT_BLT_OPERATION BltOperation,
    IN UINTN SourceX,
    IN UINTN SourceY,
    IN UINTN DestinationX,
    IN UINTN DestinationY,
    IN UINTN Width,
    IN UINTN Height,
    IN UINTN Delta OPTIONAL);

typedef struct {
    UINT32 MaxMode;
    UINT32 Mode;
    EFI_GRAPHICS_OUTPUT_MODE_INFORMATION* Info;
    UINTN SizeOfInfo;
    EFI_PHYSICAL_ADDRESS FrameBufferBase;
    UINTN FrameBufferSize;
} EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE;

typedef struct EFI_GRAPHICS_OUTPUT_PROTOCOL {
    EFI_GRAPHICS_OUTPUT_PROTOCOL_QUERY_MODE QueryMode;
    EFI_GRAPHICS_OUTPUT_PROTOCOL_SET_MODE SetMode;
    EFI_GRAPHICS_OUTPUT_PROTOCOL_BLT Blt;
    EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE* Mode;
} EFI_GRAPHICS_OUTPUT_PROTOCOL;

typedef struct {
    UINT32 SizeOfEdid;
    UINT8* Edid;
} EFI_EDID_DISCOVERED_PROTOCOL;

#endif
