def generate_addition_chain():
    # Target exponent: d = (2^129 - 7) / 5
    d_target = 136112946768375385385349842972707284581
    assert d_target == 0x66666666666666666666666666666665

    # Base: x = x^1
    # We want to build the pattern 0x6666...
    # 0x6 = 0110 in binary = 6
    # 0x66 = 01100110 in binary = 102
    
    # We can use a window-based addition chain.
    # The exponent is 31 repetitions of 0x6 (4 bits), followed by 0x5 (4 bits).
    # Total bits: 31 * 4 + 4 = 128? Wait.
    # d_target has 127 bits.
    # hex string: '0x66666666666666666666666666666665'
    # Length of hex: 32 hex chars = 128 bits.
    # But wait, the top hex char is '6' = 0110. The top bit is 0, so it's 127 bits long!
    
    # Let's verify the blocks:
    # 0x6 = 6. 
    # x^2 = sq(x)
    # x^4 = sq(x^2)
    # x^6 = x^4 * x^2
    # Let x6 = x^6.
    
    # Now we have the block 0x6 (which is 6).
    # To get 0x66 (which is 0x6 * 16 + 0x6):
    # x6_shifted = (x6)^16  (4 squarings)
    # x66 = x6_shifted * x6
    
    # We can double the size of the block!
    # Let t1 = x6 (length 1 nibble, value 0x6)
    # t2 = (t1^(2^4)) * t1 = 0x66 (length 2 nibbles)
    # t4 = (t2^(2^8)) * t2 = 0x6666 (length 4 nibbles)
    # t8 = (t4^(2^16)) * t4 = 0x66666666 (length 8 nibbles)
    # t16 = (t8^(2^32)) * t8 = 0x6666666666666666 (length 16 nibbles)
    # t31 = (t16^(2^60)) * (t8^(2^28)) * (t4^(2^12)) * (t2^(2^4)) * t1 ?
    # Better: just use simple left-to-right doubling for the 31 nibbles, or a smaller tree.
    # Since we need 31 nibbles of '6' and 1 nibble of '5', we can just do:
    # t1 = 6
    # t2 = (t1<<4) + t1 = 0x66
    # t4 = (t2<<8) + t2 = 0x6666
    # t8 = (t4<<16) + t4 = 0x66666666
    # t16 = (t8<<32) + t8 = 0x6666666666666666
    
    # Now we have 16 nibbles. We need 31 nibbles.
    # t31 = (t16 << 60) + (t8 << 28) + (t4 << 12) + (t2 << 4) + t1
    # Let's check mathematically:
    t1 = 0x6
    t2 = (t1 << 4) + t1
    t4 = (t2 << 8) + t2
    t8 = (t4 << 16) + t4
    t16 = (t8 << 32) + t8
    
    t24 = (t16 << 32) + t8
    t28 = (t24 << 16) + t4
    t30 = (t28 << 8) + t2
    t31 = (t30 << 4) + t1
    
    assert t31 == 0x6666666666666666666666666666666
    
    # Now we need 0x66666666666666666666666666666665
    # Which is (t31 << 4) + 5
    d_calc = (t31 << 4) + 5
    assert d_calc == d_target
    
    # We can compute this with exactly:
    # t1: 2 sq, 1 mul (from x, let x2 = x^2, x4 = x2^2, x6 = x4 * x2)
    # x5: x4 * x
    # t2: 4 sq, 1 mul
    # t4: 8 sq, 1 mul
    # t8: 16 sq, 1 mul
    # t16: 32 sq, 1 mul
    # t24: 32 sq, 1 mul
    # t28: 16 sq, 1 mul
    # t30: 8 sq, 1 mul
    # t31: 4 sq, 1 mul
    # final: 4 sq, 1 mul (with x5)
    
    # Total squarings: 2 + 4 + 8 + 16 + 32 + 32 + 16 + 8 + 4 + 4 = 126 squarings! (Wait, 126? Let's check: d is 127 bits. So 126 squarings is optimal!)
    # Total muls: 1 (for x6) + 1 (for x5) + 1*9 = 11 multiplications!
    
    print("Addition chain is completely verified in python.")
    print(f"Total squarings: 126")
    print(f"Total multiplications: 11")

generate_addition_chain()
