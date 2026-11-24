//! Azar propio (versión 2): PCG32 sembrado con splitmix64. Idéntico a angri/azar.py.

const MULT: u64 = 6364136223846793005;

fn splitmix64(x: u64) -> (u64, u64) {
    let x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    (x, z ^ (z >> 31))
}

#[derive(Clone)]
pub struct Azar {
    estado: u64,
    inc: u64,
}

impl Azar {
    pub fn new(semilla: u64) -> Azar {
        let (s, a) = splitmix64(semilla);
        let (_, b) = splitmix64(s);
        let mut r = Azar { estado: 0, inc: (b << 1) | 1 };
        r.u32();
        r.estado = r.estado.wrapping_add(a);
        r.u32();
        r
    }

    #[inline]
    pub fn u32(&mut self) -> u32 {
        let viejo = self.estado;
        self.estado = viejo.wrapping_mul(MULT).wrapping_add(self.inc);
        let xs = (((viejo >> 18) ^ viejo) >> 27) as u32;
        let rot = (viejo >> 59) as u32;
        xs.rotate_right(rot)
    }

    #[inline]
    pub fn random(&mut self) -> f64 {
        let a = (self.u32() >> 5) as u64;
        let b = (self.u32() >> 6) as u64;
        (a * 67108864 + b) as f64 / 9007199254740992.0
    }

    pub fn debajo(&mut self, n: u64) -> u64 {
        assert!(n > 0 && n <= 1u64 << 32, "rango inválido");
        let k = 64 - (n - 1).leading_zeros();
        if k == 0 {
            return 0;
        }
        loop {
            let r = (self.u32() >> (32 - k)) as u64;
            if r < n {
                return r;
            }
        }
    }

    pub fn randrange(&mut self, a: i64, b: i64) -> i64 {
        a + self.debajo((b - a) as u64) as i64
    }

    pub fn randint(&mut self, a: i64, b: i64) -> i64 {
        a + self.debajo((b - a + 1) as u64) as i64
    }

    pub fn indice(&mut self, n: usize) -> usize {
        self.debajo(n as u64) as usize
    }

    pub fn choice<T: Copy>(&mut self, s: &[T]) -> T {
        s[self.indice(s.len())]
    }

    pub fn shuffle<T>(&mut self, x: &mut [T]) {
        let mut i = x.len();
        while i > 1 {
            i -= 1;
            let j = self.indice(i + 1);
            x.swap(i, j);
        }
    }

    pub fn sample<T: Copy>(&mut self, pob: &[T], k: usize) -> Vec<T> {
        let mut pool: Vec<T> = pob.to_vec();
        let n = pool.len();
        assert!(k <= n);
        let mut res = Vec::with_capacity(k);
        for i in 0..k {
            let j = self.indice(n - i);
            res.push(pool[j]);
            pool[j] = pool[n - i - 1];
        }
        res
    }

    /// choices con pesos: devuelve índices (acumulados de izquierda a derecha + bisect_right).
    pub fn choices_pesos(&mut self, pesos: &[f64], k: usize) -> Vec<usize> {
        let n = pesos.len();
        let mut acum = Vec::with_capacity(n);
        let mut t = 0.0f64;
        for &w in pesos {
            t += w;
            acum.push(t);
        }
        (0..k).map(|_| {
            let x = self.random() * t;
            bisect_right(&acum, x, n - 1)
        }).collect()
    }
}

fn bisect_right(a: &[f64], x: f64, hi: usize) -> usize {
    let (mut lo, mut hi) = (0usize, hi);
    while lo < hi {
        let mid = (lo + hi) / 2;
        if x < a[mid] {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

crate::pieza!(Azar { estado, inc });
