global _start
_start:
	mov rdi, 69			; Pass 69 into func
	mov rax, 60			; Select func 60 (exit)
	syscall				; Call func