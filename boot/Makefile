CC = x86_64-w64-mingw32-gcc
CFLAGS = -shared -nostdlib -mno-red-zone -fno-stack-protector -Wall -e EfiMain

EFIFILE = bootx64.efi

all:$(EFIFILE)

%.efi:%.dll
	objcopy --target=efi-app-x86_64 $< $@

%.dll:%.c
	$(CC) $(CFLAGS) $< -o $@

clean:
	rm -f $(EFIFILE)
