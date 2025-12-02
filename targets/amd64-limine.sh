#!/bin/sh


cd "$(dirname "$1")"
mkdir -p boot
if [ ! -f boot/OVMF.fd ]; then
    curl -o boot/OVMF.fd https://raw.githubusercontent.com/retrage/edk2-nightly/refs/heads/master/bin/RELEASEX64_OVMF.fd
    chmod +w boot/OVMF.fd
fi
if [ ! -f boot/EFI/BOOT/BOOTX64.EFI ]; then
    mkdir -p boot/EFI/BOOT
    curl -o boot/EFI/BOOT/BOOTX64.EFI https://raw.githubusercontent.com/limine-bootloader/limine/refs/tags/v10.4.0-binary/BOOTX64.EFI
fi
cp "$(dirname "$0")/limine.conf" boot/limine.conf
cp "$(basename "$1")" boot/kernel.elf

sudo -E qemu-system-x86_64 -enable-kvm -cpu host -m 1G -smp 4 -bios boot/OVMF.fd \
    -serial stdio -monitor telnet::4444,server,nowait -nographic \
    -hda fat:rw:boot
