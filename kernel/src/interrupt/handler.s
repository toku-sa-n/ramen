.intel_syntax noprefix

.set    INTERRUPT_STACK, 0xffffffffc0000000 - (0x1000 * 16 / 2)
.extern end_of_interrupt
.extern switch
.extern prepare_arguments
.extern assign_rax_from_register

.text
.code64

.macro  handler name
.global \name\()_asm
.extern h_20

\name\()_asm:
	push rbp
	mov  rbp, rsp
	push r15
	push r14
	push r13
	push r12
	push r11
	push r10
	push r9
	push r8
	push rdi
	push rsi
	push rdx
	push rcx
	push rbx
	push rax

	mov  rsp, INTERRUPT_STACK
	call \name
	mov  rsp, rax

	pop rax
	pop rbx
	pop rcx
	pop rdx
	pop rsi
	pop rdi
	pop r8
	pop r9
	pop r10
	pop r11
	pop r12
	pop r13
	pop r14
	pop r15

	pop rbp
	iretq
	.endm

	handler h_20
	handler h_80
	handler h_81
