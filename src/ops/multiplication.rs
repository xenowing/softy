use crate::value::*;

pub fn multiplication(source1: Value, source2: Value) -> Value {
    assert_eq!(source1.format, source2.format);

    // Treat denormal input(s) as zero
    let source1 = flush_denormal_to_zero(source1);
    let source2 = flush_denormal_to_zero(source2);

    let format = &source1.format;

    // TODO: NaN propagation test(s)
    let sig_quiet_bit = 1 << (format.num_sig_bits - 1);
    let quiet_nan = Value::from_comps(false, format.exp_max(), sig_quiet_bit, format.clone());

    if source1.is_nan() || source2.is_nan() {
        return quiet_nan;
    }

    let product_sign = source1.sign ^ source2.sign;

    if source1.exp == 0 || source2.exp == 0 {
        return Value::from_comps(product_sign, 0, 0, format.clone())
    }

    // TODO: Check for additional special cases/conditioning

    // Decode full sigs
    let hidden_bit = 1 << format.num_sig_bits;
    let source1_sig = hidden_bit | source1.sig;
    let source2_sig = hidden_bit | source2.sig;

    // Calculate product
    let exp_bias = (1 << (format.num_exp_bits - 1)) - 1;
    let mut product_exp = (source1.exp + source2.exp).wrapping_sub(exp_bias); // TODO: Handle overflow/underflow
    let product_sig = (source1_sig as u64) * (source2_sig as u64);

    // Normalize product
    let mut product_sig = (product_sig >> format.num_sig_bits) as u32;
    let product_sig_leading_zeros = product_sig.leading_zeros() - (32 - (format.num_sig_bits + 2));

    if product_sig_leading_zeros < 1 {
        let product_sig_shift_right = 1 - product_sig_leading_zeros;
        product_exp += product_sig_shift_right;
        product_sig >>= product_sig_shift_right;
    }

    // Remove hidden bit from product
    let product_sig = (product_sig as u32) & ((1 << format.num_sig_bits) - 1);
    Value::from_comps(product_sign, product_exp, product_sig, format.clone())
}

// TODO: dedupe
fn flush_denormal_to_zero(value: Value) -> Value {
    if value.exp == 0 {
        Value::from_comps(value.sign, value.exp, 0, value.format)
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::format::*;

    #[test]
    fn multiplication_basic() {
        let f = Format::ieee754_single();

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x3f800000); // 1.0

        let a = Value::from_comps(false, 128, 0, f.clone()); // 2.0
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x40000000); // 2.0

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(false, 128, 0, f.clone()); // 2.0

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x40000000); // 2.0

        let a = Value::from_comps(false, 128, 0, f.clone()); // 2.0
        let b = Value::from_comps(false, 128, 0, f.clone()); // 2.0

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x40800000); // 4.0

        let a = Value::from_comps(false, 128, 1 << 22, f.clone()); // 3.0
        let b = Value::from_comps(false, 128, 1 << 22, f.clone()); // 3.0

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x41100000); // 9.0

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(false, 124, 0, f.clone()); // 0.125

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x3e000000); // 0.125

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(true, 128, 0x260000, f.clone()); // -2.59375

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0xc0260000); // -2.59375

        let a = Value::from_comps(false, 136, 0, f.clone()); // 512.0
        let b = Value::from_comps(true, 128, 0x260000, f.clone()); // -2.59375

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0xc4a60000); // -1328.0

        let a = Value::from_comps(false, 0, 0, f.clone()); // 0.0
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0
    }

    #[test]
    fn multiplication_daz_ftz() {
        let f = Format::ieee754_single();

        let a = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number
        let b = Value::from_comps(false, 0, 0, f.clone()); // 0.0

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 0, 0, f.clone()); // 0.0
        let b = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number
        let b = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 128, 0, f.clone()); // 2.0
        let b = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(true, 128, 0, f.clone()); // -2.0
        let b = Value::from_comps(true, 0, 1337, f.clone()); // any negative denormalized number

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 0, 1337, f.clone()); // any positive denormalized number
        let b = Value::from_comps(false, 0, 1337, f.clone()); // any positive denormalized number

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 0, 1337, f.clone()); // any positive denormalized number
        let b = Value::from_comps(true, 0, 1337, f.clone()); // any negative denormalized number

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x80000000); // -0.0

        let a = Value::from_comps(true, 0, 1337, f.clone()); // any negative denormalized number
        let b = Value::from_comps(false, 0, 1337, f.clone()); // any positive denormalized number

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x80000000); // -0.0

        let a = Value::from_comps(true, 0, 1337, f.clone()); // any negative denormalized number
        let b = Value::from_comps(true, 0, 1337, f.clone()); // any negative denormalized number

        let res = multiplication(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0
    }
}
