pub mod chacha20;
pub mod seedrandom;

const FRACTION_BITS: u8 = 52;
const FRACTION_MASK: u64 = (1 << FRACTION_BITS) - 1;
const SAFE_INTEGER_BITS: u8 = FRACTION_BITS + 1;
const MAX_SAFE_INTEGER: i64 = (1 << SAFE_INTEGER_BITS) - 1;

pub trait Rng {
    fn generate_int_by_bytes(&mut self, bytes: u8) -> u64;

    fn generate_int_by_bits(&mut self, bits: u8) -> u64 {
        let bytes = bits.div_ceil(8);
        let wasted_bits = (bytes << 3) - bits;
        self.generate_int_by_bytes(bytes) >> wasted_bits
    }

    fn generate_number_0_to_1(&mut self) -> f64 {
        let mut fraction = self.generate_int_by_bits(SAFE_INTEGER_BITS);
        let mut exponent = 1022_u64;
        let mut remaining_fraction_bits = SAFE_INTEGER_BITS - fraction.bit_width() as u8;

        while remaining_fraction_bits > 0 && exponent >= SAFE_INTEGER_BITS.into() {
            exponent -= u64::from(remaining_fraction_bits);
            fraction <<= remaining_fraction_bits;
            fraction |= self.generate_int_by_bits(remaining_fraction_bits);
            remaining_fraction_bits = SAFE_INTEGER_BITS - fraction.bit_width() as u8;
        }

        if remaining_fraction_bits > 0 {
            let shift = (exponent - 1).min(remaining_fraction_bits.into());
            exponent -= shift;
            fraction <<= shift;
            fraction |= self.generate_int_by_bits(shift as u8);
        }

        f64::from_bits(exponent << FRACTION_BITS | (fraction & FRACTION_MASK))
    }

    fn generate_uniform(&mut self, max_inclusive: u64) -> u64 {
        loop {
            let result = self.generate_int_by_bits(max_inclusive.bit_width() as u8);
            if result <= max_inclusive {
                return result;
            }
        }
    }

    fn generate_random_integer_in_range(&mut self, min: f64, max: f64) -> Option<i64> {
        let min = min.ceil() as i64;
        let max = max.floor() as i64;
        let (scale, sign) = if min < max {
            (max - min, 1)
        } else if min > max {
            (min - max, -1)
        } else {
            return Some(min);
        };

        if scale <= MAX_SAFE_INTEGER
            && min.abs() <= MAX_SAFE_INTEGER
            && max.abs() <= MAX_SAFE_INTEGER
        {
            Some(self.generate_uniform(scale as u64) as i64 * sign + min)
        } else {
            None
        }
    }
}
