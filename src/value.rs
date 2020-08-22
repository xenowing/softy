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
}
