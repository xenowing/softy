use crate::format::*;

pub struct Value {
    pub(crate) sign: bool,
    pub(crate) exp: u32,
    pub(crate) sig: u32,
    pub(crate) format: Format,
}

impl Value {
    pub fn from_comps(sign: bool, exp: u32, sig: u32, format: Format) -> Value {
        // TODO: Sanity check values against format

        Value {
            sign,
            exp,
            sig,
            format,
        }
    }

    pub fn to_bits(&self) -> u32 {
        let sign = if self.sign { 1 } else { 0 } << (self.format.num_exp_bits + self.format.num_sig_bits);
        let exp = self.exp << self.format.num_sig_bits;
        let sig = self.sig;
        sign | exp | sig
    }

    pub fn is_nan(&self) -> bool {
        self.exp == self.format.exp_max() && self.sig != 0
    }

    pub fn is_inf(&self) -> bool {
        self.exp == self.format.exp_max() && self.sig == 0
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_nan() {
        let f = Format::ieee754_single();

        let x = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        assert_eq!(x.is_nan(), false);

        let x = Value::from_comps(false, 128, 0, f.clone()); // 2.0

        assert_eq!(x.is_nan(), false);

        let x = Value::from_comps(false, 255, 0, f.clone()); // +inf

        assert_eq!(x.is_nan(), false);

        let x = Value::from_comps(false, 255, 1, f.clone()); // NaN

        assert_eq!(x.is_nan(), true);

        let x = Value::from_comps(false, 255, 1337, f.clone()); // NaN

        assert_eq!(x.is_nan(), true);

        let x = Value::from_comps(true, 255, 1337, f.clone()); // -NaN

        assert_eq!(x.is_nan(), true);
    }

    #[test]
    fn is_inf() {
        let f = Format::ieee754_single();

        let x = Value::from_comps(false, 127, 0, f.clone()); // 1.0

        assert_eq!(x.is_inf(), false);

        let x = Value::from_comps(false, 128, 0, f.clone()); // 2.0

        assert_eq!(x.is_inf(), false);

        let x = Value::from_comps(false, 255, 1, f.clone()); // NaN

        assert_eq!(x.is_inf(), false);

        let x = Value::from_comps(false, 255, 1337, f.clone()); // NaN

        assert_eq!(x.is_inf(), false);

        let x = Value::from_comps(true, 255, 1337, f.clone()); // -NaN

        assert_eq!(x.is_inf(), false);

        let x = Value::from_comps(false, 255, 0, f.clone()); // +inf

        assert_eq!(x.is_inf(), true);

        let x = Value::from_comps(true, 255, 0, f.clone()); // -inf

        assert_eq!(x.is_inf(), true);
    }
}
