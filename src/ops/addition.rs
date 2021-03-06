use crate::value::*;

use std::mem;

pub fn addition(source1: Value, source2: Value) -> Value {
    assert_eq!(source1.format, source2.format);

    // Treat denormal input(s) as zero
    let mut source1 = flush_denormal_to_zero(source1);
    let mut source2 = flush_denormal_to_zero(source2);

    // Ensure source with greater magnitude is lhs
    if source1.exp < source2.exp || (source1.exp == source2.exp && source1.sig < source2.sig) {
        mem::swap(&mut source1, &mut source2);
    }

    let format = &source1.format;

    // Propagate (replace!) NaNs
    let sig_quiet_bit = 1 << (format.num_sig_bits - 1);
    let quiet_nan = Value::from_comps(false, format.exp_max(), sig_quiet_bit, format.clone());

    if source1.is_nan() || source2.is_nan() {
        return quiet_nan;
    }

    // TODO: Is this case really important?
    if source1.is_inf() && source2.is_inf() && source1.sign != source2.sign {
        return quiet_nan;
    }

    // Decode full sigs
    let hidden_bit = 1 << format.num_sig_bits;
    let source1_sig = hidden_bit | source1.sig;
    let mut source2_sig = hidden_bit | source2.sig;

    // Align rhs point (if applicable)
    if source2.exp < source1.exp {
        let shift_digits = source1.exp - source2.exp;
        if shift_digits > format.num_sig_bits {
            source2_sig = 0;
        } else {
            source2_sig >>= shift_digits;
        }
    }

    // If sources' signs differ, negate rhs sig via two's complement
    let sig_including_hidden_and_overflow_bits_mask = (1 << (format.num_sig_bits + 2)) - 1;
    if source1.sign != source2.sign {
        source2_sig = (!source2_sig).wrapping_add(1) & sig_including_hidden_and_overflow_bits_mask;
    }

    // Calculate sum
    let sum_sign = source1.sign;
    let mut sum_exp = source1.exp;
    let mut sum_sig = (source1_sig + source2_sig) & sig_including_hidden_and_overflow_bits_mask;
    let is_sum_zero = sum_exp == 0 || sum_sig == 0;

    // Normalize sum in case of hidden bit overflow
    let sum_sig_overflow = ((sum_sig >> (format.num_sig_bits + 1)) & 1) != 0;
    if sum_sig_overflow {
        sum_exp += 1;
        sum_sig >>= 1;
    }

    // Check for infinity (exp overflow)
    let is_sum_inf = sum_exp >= format.exp_max();

    // Normalize sum in case of cancellations from potentially-negative rhs
    let sum_sig_leading_zeros = sum_sig.leading_zeros() - (32 - (format.num_sig_bits + 1));
    sum_sig <<= sum_sig_leading_zeros;
    let is_sum_zero = is_sum_zero || sum_sig_leading_zeros >= sum_exp;
    sum_exp = sum_exp.wrapping_sub(sum_sig_leading_zeros);

    if is_sum_inf {
        Value::from_comps(sum_sign, format.exp_max(), 0, format.clone())
    } else if is_sum_zero {
        // TODO: Handle sign properly (or not? :) )
        Value::from_comps(false, 0, 0, format.clone())
    } else {
        // Remove hidden bit from sum
        let sum_sig = sum_sig & ((1 << format.num_sig_bits) - 1);
        Value::from_comps(sum_sign, sum_exp, sum_sig, format.clone())
    }
}

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
    fn addition_basic() {
        let f = Format::ieee754_single();

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x40000000); // 2.0

        let a = Value::from_comps(false, 126, 0, f.clone()); // 0.5
        let b = Value::from_comps(false, 126, 0, f.clone()); // 0.5

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x3f800000); // 1.0

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(false, 126, 0, f.clone()); // 0.5

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x3fc00000); // 1.5

        let a = Value::from_comps(false, 0, 0, f.clone()); // 0.0
        let b = Value::from_comps(false, 0, 0, f.clone()); // 0.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(false, 0, 0, f.clone()); // 0.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x3f800000); // 1.0

        let a = Value::from_comps(false, 0, 0, f.clone()); // 0.0
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x3f800000); // 1.0

        let a = Value::from_comps(true, 127, 0, f.clone()); // -1.0
        let b = Value::from_comps(true, 127, 0, f.clone()); // -1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0xc0000000); // -2.0

        let a = Value::from_comps(true, 127, 1 << 22, f.clone()); // -1.5
        let b = Value::from_comps(true, 127, 0, f.clone()); // -1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0xc0200000); // -2.5

        let a = Value::from_comps(true, 128, 1 << 22, f.clone()); // -3.0
        let b = Value::from_comps(true, 128, 1 << 22, f.clone()); // -3.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0xc0c00000); // -6.0
    }

    #[test]
    fn addition_daz_ftz() {
        let f = Format::ieee754_single();

        let a = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number
        let b = Value::from_comps(false, 0, 0, f.clone()); // 0.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 0, 0, f.clone()); // 0.0
        let b = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number
        let b = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 128, 0, f.clone()); // 2.0
        let b = Value::from_comps(false, 0, 1337, f.clone()); // any denormalized number

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x40000000); // 2.0

        let a = Value::from_comps(true, 128, 0, f.clone()); // -2.0
        let b = Value::from_comps(true, 0, 1337, f.clone()); // any negative denormalized number

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0xc0000000); // -2.0

        let a = Value::from_comps(false, 0, 1337, f.clone()); // any positive denormalized number
        let b = Value::from_comps(true, 0, 1337, f.clone()); // any negative denormalized number

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0
    }

    #[test]
    fn addition_non_matching_signs() {
        let f = Format::ieee754_single();

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(true, 127, 0, f.clone()); // -1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(false, 0, 0, f.clone()); // 0.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x3f800000); // 1.0

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(true, 0, 0, f.clone()); // -0.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x3f800000); // 1.0

        let a = Value::from_comps(false, 0, 0, f.clone()); // 0.0
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x3f800000); // 1.0

        let a = Value::from_comps(true, 0, 0, f.clone()); // -0.0
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x3f800000); // 1.0

        let a = Value::from_comps(true, 127, 0, f.clone()); // -1.0
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(true, 0, 0, f.clone()); // -0.0
        let b = Value::from_comps(false, 0, 0, f.clone()); // 0.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x00000000); // 0.0

        let a = Value::from_comps(false, 127, 0, f.clone()); // 1.0
        let b = Value::from_comps(true, 127, 1 << 22, f.clone()); // -1.5

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0xbf000000); // -0.5

        let a = Value::from_comps(true, 127, 1 << 22, f.clone()); // -1.5
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0xbf000000); // -0.5

        let a = Value::from_comps(false, 142, 0, f.clone()); // 32768.0
        let b = Value::from_comps(true, 142, 1 << 6, f.clone()); // -32768.25

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0xbe800000); // -0.25
    }

    #[test]
    fn addition_nan() {
        let f = Format::ieee754_single();

        let a = Value::from_comps(false, 255, 1337, f.clone()); // any NaN
        let b = Value::from_comps(false, 0, 0, f.clone()); // 0.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7fc00000); // NaN

        let a = Value::from_comps(false, 0, 0, f.clone()); // 0.0
        let b = Value::from_comps(false, 255, 1337, f.clone()); // any NaN

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7fc00000); // NaN

        let a = Value::from_comps(false, 255, 0, f.clone()); // +inf
        let b = Value::from_comps(false, 255, 1337, f.clone()); // any NaN

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7fc00000); // NaN

        let a = Value::from_comps(false, 255, 1337, f.clone()); // any NaN
        let b = Value::from_comps(true, 255, 0, f.clone()); // -inf

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7fc00000); // NaN

        let a = Value::from_comps(false, 255, 1337, f.clone()); // any NaN
        let b = Value::from_comps(true, 255, 1338, f.clone()); // any NaN

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7fc00000); // NaN
    }

    #[test]
    fn addition_inf() {
        let f = Format::ieee754_single();

        let a = Value::from_comps(false, 255, 0, f.clone()); // +inf
        let b = Value::from_comps(false, 0, 0, f.clone()); // 0.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7f800000); // +inf

        let a = Value::from_comps(false, 255, 0, f.clone()); // +inf
        let b = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7f800000); // +inf

        let a = Value::from_comps(false, 255, 0, f.clone()); // +inf
        let b = Value::from_comps(false, 254, 0, f.clone()); // +max value

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7f800000); // +inf

        let a = Value::from_comps(false, 255, 0, f.clone()); // +inf
        let b = Value::from_comps(false, 255, 0, f.clone()); // +inf

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7f800000); // +inf

        let a = Value::from_comps(true, 255, 0, f.clone()); // -inf
        let b = Value::from_comps(true, 254, 0, f.clone()); // -max value

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0xff800000); // -inf

        let a = Value::from_comps(true, 255, 0, f.clone()); // -inf
        let b = Value::from_comps(true, 255, 0, f.clone()); // -inf

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0xff800000); // -inf

        // TODO: Is this case really important?
        let a = Value::from_comps(false, 255, 0, f.clone()); // +inf
        let b = Value::from_comps(true, 255, 0, f.clone()); // -inf

        let res = addition(a, b);

        assert_eq!(res.to_bits(), 0x7fc00000); // NaN
    }
}
