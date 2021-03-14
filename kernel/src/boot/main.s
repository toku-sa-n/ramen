// SPDX-License-Identifier: GPL-3.0-or-later

    .intel_syntax noprefix
    .global _start
    .extern os_main

_start:
    jmp os_main
