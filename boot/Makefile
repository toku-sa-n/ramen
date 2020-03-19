CC = x86_64-w64-mingw32-gcc
CFLAGS = -shared -nostdlib -mno-red-zone -fno-stack-protector -Wall -e EfiMain

EFIFILE = bootx64.efi

all:$(EFIFILE)

%.efi:%.dll
	objcopy --target=efi-app-x86_64 $< $@

%.dll:%.c
	$(CC) $(CFLAGS) $< -o $@

copy_to_usb:$(EFIFILE)
ifeq ($(USB_DEVICE_PATH),)
	echo 'Specify device path by $$USB_DEVICE_PATH environment variable.' >&2
else
	sudo mount $(USB_DEVICE_PATH) /mnt
	sudo cp $(EFIFILE) /mnt/efi/boot/
	sudo umount /mnt
endif

clean:
	rm -f $(EFIFILE)
