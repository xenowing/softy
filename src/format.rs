pub struct Format {
    num_exp_bits: u32,
    num_sig_bits: u32,
}

impl Format {
    pub fn new(num_exp_bits: u32, num_sig_bits: u32) -> Format {
        let num_storage_bits = 1 + num_exp_bits + num_sig_bits;
        let max_storage_bits = 32;
        if num_storage_bits > max_storage_bits {
            panic!("Requested format requires {} storage bits, which exceeds the maximum storage bit width of {} bits.", num_storage_bits, max_storage_bits);
        }

        Format {
            num_exp_bits,
            num_sig_bits,
        }
    }

    pub fn num_storage_bits(&self) -> u32 {
        1 + self.num_exp_bits + self.num_sig_bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_valid_format() {
        let f = Format::new(8, 23);
        assert_eq!(f.num_exp_bits, 8);
        assert_eq!(f.num_sig_bits, 23);

        let f = Format::new(10, 10);
        assert_eq!(f.num_exp_bits, 10);
        assert_eq!(f.num_sig_bits, 10);
    }

    #[test]
    #[should_panic(expected = "Requested format requires 33 storage bits, which exceeds the maximum storage bit width of 32 bits.")]
    fn new_exceeded_storage_bit_width_0() {
        // Panic
        let _ = Format::new(31, 1);
    }

    #[test]
    #[should_panic(expected = "Requested format requires 33 storage bits, which exceeds the maximum storage bit width of 32 bits.")]
    fn new_exceeded_storage_bit_width_1() {
        // Panic
        let _ = Format::new(1, 31);
    }

    #[test]
    #[should_panic(expected = "Requested format requires 2001 storage bits, which exceeds the maximum storage bit width of 32 bits.")]
    fn new_exceeded_storage_bit_width_2() {
        // Panic
        let _ = Format::new(2000, 0);
    }

    #[test]
    #[should_panic(expected = "Requested format requires 1338 storage bits, which exceeds the maximum storage bit width of 32 bits.")]
    fn new_exceeded_storage_bit_width_3() {
        // Panic
        let _ = Format::new(0, 1337);
    }

    #[test]
    fn num_storage_bits() {
        let single = Format::new(8, 23);
        assert_eq!(single.num_storage_bits(), 32);

        let half = Format::new(5, 10);
        assert_eq!(half.num_storage_bits(), 16);

        let bfloat16 = Format::new(8, 7);
        assert_eq!(bfloat16.num_storage_bits(), 16);

        let fp24 = Format::new(7, 16);
        assert_eq!(fp24.num_storage_bits(), 24);
    }
}
