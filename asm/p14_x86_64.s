.intel_syntax noprefix
.section .text
.global rco_p14_project_batch_avx512

# rco_p14_project_batch_avx512(const double* in, size_t count, __int128* out)
# rdi: in, rsi: count, rdx: out
rco_p14_project_batch_avx512:
    push rbp
    mov rbp, rsp
    push rbx
    
    and rsp, -64
    sub rsp, 64

    vbroadcastsd zmm1, qword ptr [rip + P14_SCALE_F64]
    vbroadcastsd zmm2, qword ptr [rip + P14_BIAS_F64]
    vpbroadcastq zmm3, qword ptr [rip + P14_SIGN_MASK]

.Lloop:
    cmp rsi, 8
    jl .Lremainder
    
    vmovupd zmm0, zmmword ptr [rdi]
    vmulpd zmm0, zmm0, zmm1
    
    vpandq zmm4, zmm0, zmm3      
    vporq  zmm4, zmm4, zmm2      
    vaddpd zmm0, zmm0, zmm4      
    
    vcvttpd2qq zmm0, zmm0
    vmovdqa64 zmmword ptr [rsp], zmm0
    
    xor rcx, rcx
.Lstore_loop:
    mov rax, [rsp + rcx*8]
    
    mov rbx, rcx
    shl rbx, 4
    
    # Store low 64 bits
    mov [rdx + rbx], rax
    
    # Branchless sign extension for ALL elements
    bt  rax, 63
    sbb rax, rax
    # Store high 64 bits
    mov [rdx + rbx + 8], rax
    
    inc rcx
    cmp rcx, 8
    jl .Lstore_loop
    
    add rdi, 64
    add rdx, 128
    sub rsi, 8
    jmp .Lloop

.Lremainder:
    lea rsp, [rbp - 8]
    pop rbx
    pop rbp
    ret

.section .rodata
.align 64
P14_SCALE_F64: .double 1e14
P14_BIAS_F64:  .double 0.5
P14_SIGN_MASK: .quad 0x8000000000000000
