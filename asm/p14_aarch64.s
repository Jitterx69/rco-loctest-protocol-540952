.section .text
.global rco_p14_project_batch_neon

# rco_p14_project_batch_neon(const double* in, size_t count, __int128* out)
# x0: in, x1: count, x2: out
rco_p14_project_batch_neon:
    # Load 10^14 into v2
    ldr d2, .P14_SCALE
    dup v2.2d, v2.d[0]

.loop:
    cmp x1, #2
    b.lt .remainder
    
    # Load 2 doubles
    ld1 {v0.2d}, [x0], #16
    
    # r_scaled = r * 10^14
    fmul v0.2d, v0.2d, v2.2d
    
    # r_biased = r_scaled + copysign(0.5, r_scaled)
    # FCVTAS rounds to nearest, ties away from zero.
    fcvtas v1.2d, v0.2d
    
    # Store as i128 with sign extension
    # First element
    st1 {v1.d}[0], [x2], #8
    mov x3, v1.d[0]
    asr x3, x3, #63
    str x3, [x2], #8
    
    # Second element
    st1 {v1.d}[1], [x2], #8
    mov x3, v1.d[1]
    asr x3, x3, #63
    str x3, [x2], #8
    
    sub x1, x1, #2
    b .loop

.remainder:
    ret

.align 3
.P14_SCALE: .double 1e14
