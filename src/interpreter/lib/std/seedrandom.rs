// https://github.com/davidbau/seedrandom

const WIDTH: usize = u8::MAX as usize + 1; // each RC4 output is 0 <= x < 256
const CHUNKS: u32 = 6; // at least six RC4 outputs for each double
const DIGITS: u32 = f64::MANTISSA_DIGITS - 1; // there are 52 significant digits in a double
const STARTDENOM: u64 = (u8::MAX as u64 + 1).pow(CHUNKS);
const SIGNIFICANCE: u64 = 2_u64.pow(DIGITS);
const OVERFLOW: u64 = SIGNIFICANCE * 2;

pub fn seedrandom(seed: &str) -> impl FnMut() -> f64 {
    let key = mixkey(seed);
    let mut arc4 = Arc4::new(key);
    move || {
        let mut n = arc4
            .g(CHUNKS)
            .into_iter()
            .fold(0_f64, |acc, v| acc * (u8::MAX as f64 + 1.0) + v as f64);
        let mut d = STARTDENOM as f64;
        let mut x = 0;
        while n < SIGNIFICANCE as f64 {
            n = (n + x as f64) * WIDTH as f64;
            d *= WIDTH as f64;
            x = *arc4.g(1).first().unwrap();
        }
        while n >= OVERFLOW as f64 {
            n /= 2.0;
            d /= 2.0;
            x >>= 1;
        }
        (n + x as f64) / d
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Arc4 {
    s: [u8; WIDTH],
    i: u8,
    j: u8,
}

impl Arc4 {
    pub fn new(key: Vec<u8>) -> Self {
        let key = if key.is_empty() { vec![0] } else { key };
        let keylen = key.len();
        let mut s = [0; WIDTH];
        for i in 0..=u8::MAX {
            s[i as usize] = i;
        }
        let mut j = 0_u8;
        for i in 0..=u8::MAX {
            let t = s[i as usize];
            j = (j as usize + key[i as usize % keylen] as usize + t as usize) as u8;
            s[i as usize] = s[j as usize];
            s[j as usize] = t;
        }
        let mut arc = Arc4 { s, i: 0, j: 0 };
        arc.g(u8::MAX as u32 + 1);
        arc
    }

    pub fn g(&mut self, count: u32) -> Vec<u8> {
        let mut r = Vec::new();
        for _ in 0..count {
            let i = (self.i as usize + 1) as u8;
            let t = self.s[i as usize];
            let j = (self.j as usize + t as usize) as u8;
            self.s[i as usize] = self.s[j as usize];
            self.s[j as usize] = t;
            r.push(
                self.s[(self.s[i as usize] as usize + self.s[j as usize] as usize) as u8 as usize],
            );
            self.i = i;
            self.j = j;
        }
        r
    }
}

fn mixkey(seed: &str) -> Vec<u8> {
    let mut key = Vec::new();
    let mut smear = 0_u32;
    for (j, c) in seed.chars().enumerate() {
        let i = j as u8;
        smear ^= key.get(i as usize).map_or(0, |i| *i as u32) * 19;
        let value = (smear + c as u32) as u8;
        if j <= u8::MAX as usize {
            key.push(value)
        } else {
            key[i as usize] = value;
        }
    }
    key
}
