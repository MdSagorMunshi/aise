import mpmath
import os

def mul2_8(a, b):
    p = 0
    for _ in range(8):
        if b & 1: p ^= a
        hi = a & 0x80
        a <<= 1
        if hi: a ^= 0x11b
        b >>= 1
    return p

def inv2_8(a):
    if a == 0: return 0
    # exhaustive search for inverse
    for i in range(1, 256):
        if mul2_8(a, i) == 1:
            return i
    return 0

def mul2_16(a, b):
    p = 0
    for _ in range(16):
        if b & 1: p ^= a
        hi = a & 0x8000
        a <<= 1
        if hi: a ^= 0x1002b
        b >>= 1
    return p

def inv2_16(a):
    if a == 0: return 0
    # multiplicative group order is 2^16-1
    # x^(2^16-2) is the inverse
    res = 1
    base = a
    exp = (1 << 16) - 2
    while exp > 0:
        if exp & 1: res = mul2_16(res, base)
        base = mul2_16(base, base)
        exp >>= 1
    return res

def extgcd(a, b):
    old_s, s = 1, 0
    old_t, t = 0, 1
    old_r, r = a, b
    while r != 0:
        quotient = old_r // r
        old_r, r = r, old_r - quotient * r
        old_s, s = s, old_s - quotient * s
        old_t, t = t, old_t - quotient * t
    return old_r, old_s, old_t

def inv_p(a, p):
    g, x, y = extgcd(a, p)
    if g != 1: raise Exception("No inverse")
    return x % p

def bit_reverse_7(x):
    res = 0
    for i in range(7):
        res = (res << 1) | (x & 1)
        x >>= 1
    return res

def rotl7(x, r):
    return ((x << r) & 0x7F) | (x >> (7 - r))

def gen_sigma():
    sigma_a = []
    sigma_b = []
    sigma_c = []
    for i in range(128):
        sa = bit_reverse_7(i) ^ 0x5A
        sb = rotl7(i, 3) ^ 0x6B
        sc = rotl7(bit_reverse_7(i), 5) ^ 0x33
        sigma_a.append(sa)
        sigma_b.append(sb)
        sigma_c.append(sc)
    return sigma_a, sigma_b, sigma_c

def generate_rc(base_func, size_bits, count, chunks, chunk_size):
    # base_func takes `r` and returns `val`
    mpmath.mp.dps = 2000
    if size_bits == 16384:
        # Check if precision is enough
        mpmath.mp.dps = 6000
    res = []
    for r in range(count):
        val = base_func(r)
        frac_val = val - mpmath.floor(val)
        int_val = int(frac_val * (2**size_bits))
        bin_str = bin(int_val)[2:].zfill(size_bits)
        row = []
        for i in range(chunks):
            chunk_bin = bin_str[i*chunk_size : (i+1)*chunk_size]
            chunk_int = int(chunk_bin, 2)
            row.append(chunk_int)
        res.append(row)
    return res

def format_u128_array(arr, is_p=False):
    s = "[\n"
    for row in arr:
        s += "        [\n"
        for val in row:
            if is_p and val == (1<<127)-1:
                val = 0
            hi = val >> 64
            lo = val & ((1<<64)-1)
            s += f"            ({hi}u64, {lo}u64),\n"
        s += "        ],\n"
    s += "    ]"
    return s

def format_u8_matrix(mat):
    s = "[\n"
    for row in mat:
        s += "        [" + ", ".join(f"{x}u8" for x in row) + "],\n"
    s += "    ]"
    return s

def format_u16_matrix(mat):
    s = "[\n"
    for row in mat:
        s += "        [" + ", ".join(f"{x}u16" for x in row) + "],\n"
    s += "    ]"
    return s

def format_u128_matrix(mat):
    s = "[\n"
    for row in mat:
        s += "        [\n"
        for val in row:
            hi = val >> 64
            lo = val & ((1<<64)-1)
            s += f"            ({hi}u64, {lo}u64),\n"
        s += "        ],\n"
    s += "    ]"
    return s

def format_usize_array(arr):
    return "[" + ", ".join(str(x) for x in arr) + "]"

def main():
    print("Generating SIGMA arrays...")
    sa, sb, sc = gen_sigma()
    
    print("Generating M_COL...")
    x_val_col = [0x01,0x02,0x04,0x08,0x10,0x20,0x40,0x80,0x1B,0x36,0x6C,0xD8,0xAB,0x4D,0x9A,0x2F]
    y_val_col = [0xA1,0xB2,0xC3,0xD4,0xE5,0xF6,0x07,0x18,0x29,0x3A,0x4B,0x5C,0x6D,0x7E,0x8F,0x90]
    m_col = []
    for i in range(16):
        row = []
        for j in range(16):
            val = x_val_col[i] ^ y_val_col[j]
            row.append(inv2_8(val))
        m_col.append(row)
        
    print("Generating M_ROW...")
    u_val_row = [0x0001,0x0002,0x0004,0x0008,0x0010,0x0020,0x0040,0x0080]
    v_val_row = [0x0100,0x0200,0x0400,0x0800,0x1000,0x2000,0x4000,0x8000]
    m_row = []
    for i in range(8):
        row = []
        for j in range(8):
            val = u_val_row[i] ^ v_val_row[j]
            row.append(inv2_16(val))
        m_row.append(row)
        
    print("Generating M_COL_P...")
    p = (1 << 127) - 1
    m_col_p = []
    for i in range(16):
        row = []
        for j in range(16):
            val = ((i + 1) - (j + 17)) % p
            row.append(inv_p(val, p))
        m_col_p.append(row)
        
    print("Generating M_ROW_P...")
    m_row_p = []
    for i in range(8):
        row = []
        for j in range(8):
            val = ((i + 1) - (j + 9)) % p
            row.append(inv_p(val, p))
        m_row_p.append(row)
        
    print("Generating RC_A...")
    rc_a = generate_rc(lambda r: mpmath.pi**(r+2), 16384, 32, 128, 128)
    print("Generating RC_B...")
    rc_b = generate_rc(lambda r: mpmath.e**(r+2), 16384, 32, 128, 128)
    print("Generating RC_C...")
    rc_c = generate_rc(lambda r: mpmath.zeta(3)**(r+2), 128*127, 32, 128, 127)
    
    out_dir = "crates/aise-core/src"
    os.makedirs(out_dir, exist_ok=True)
    out_path = os.path.join(out_dir, "constants.rs")
    print(f"Writing to {out_path}...")
    
    with open(out_path, "w") as f:
        f.write("//! AISE Constants generated by constants_gen_aise.py\n\n")
        f.write(f"pub const SIGMA_A: [usize; 128] = {format_usize_array(sa)};\n\n")
        f.write(f"pub const SIGMA_B: [usize; 128] = {format_usize_array(sb)};\n\n")
        f.write(f"pub const SIGMA_C: [usize; 128] = {format_usize_array(sc)};\n\n")
        f.write(f"pub const M_COL: [[u8; 16]; 16] = {format_u8_matrix(m_col)};\n\n")
        f.write(f"pub const M_ROW: [[u16; 8]; 8] = {format_u16_matrix(m_row)};\n\n")
        f.write(f"pub const M_COL_P: [[(u64, u64); 16]; 16] = {format_u128_matrix(m_col_p)};\n\n")
        f.write(f"pub const M_ROW_P: [[(u64, u64); 8]; 8] = {format_u128_matrix(m_row_p)};\n\n")
        f.write(f"pub const RC_A: [[(u64, u64); 128]; 32] = {format_u128_array(rc_a, False)};\n\n")
        f.write(f"pub const RC_B: [[(u64, u64); 128]; 32] = {format_u128_array(rc_b, False)};\n\n")
        f.write(f"pub const RC_C: [[(u64, u64); 128]; 32] = {format_u128_array(rc_c, True)};\n\n")
        f.write("pub const C_B: (u64, u64) = (0xA51F3C29B78E2D4F, 0xD9C7B043E62A1508);\n")

    print("Done!")

if __name__ == "__main__":
    main()
