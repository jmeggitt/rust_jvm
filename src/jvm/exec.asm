	global exec_x86_with_stack

%macro push_reg_x86_args 0
	push r9
	push r8
	push rcx
	push rdx
	push rsi
	push rdi
%endmacro

%macro pop_reg_x86_args 0
	pop rdi
	pop rsi
	pop rcx
	pop rdx
	pop r8
	pop r9
%endmacro

exec_x86_with_stack:
; rdi = function pointer
; rsi = dst stack base pointer
; rdx = dst stack pointer

	push rbp       ; Save previous stack frame
	mov rbp, rsp   ; Create new stack frame


	mov rax, rdi ; Put function pointer in return register for safe keeping

	push_reg_x86_args ; Save registers we will use in call

	push rbp ; Store base pointer for later
	mov [rsi-8], rsp ; Store rsp at the end of alternate stack

	; Switch to alternate stack
	mov rbp, rsi
	mov rsp, rdx

	pop_reg_x86_args ; pop arguments from stack

	call rax ; Execute the function

	pop rsp ; Switch back to original stack
	pop rbp ; Retrieve old base pointer

	pop_reg_x86_args ; Restore registers used in function call

	mov rsp, rbp   ; Restore previous stack frame
	pop rbp


	ret
