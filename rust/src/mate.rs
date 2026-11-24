//! Matemática propia: exp, sin, cos y pow sin la biblioteca de C del sistema.
//!
//! La de Windows y la de Linux (glibc) redondean distinto el último bit de
//! `exp`, `sin`, `cos` y `pow` en algunos valores: la misma semilla daba otra
//! vida en el i5. Estas usan solo +, −, × y ÷ de IEEE 754 (iguales en todas las
//! máquinas; Rust no fusiona operaciones por su cuenta) en doble-doble (~106
//! bits) y redondean una sola vez al final, así que dan el double más cercano al
//! valor exacto salvo casos rarísimos (y, sobre todo, lo mismo en todos lados).
//! Comprobado contra el valor exacto (80 dígitos) en 4.640 casos donde la de
//! Windows difería: la de acá era siempre la correcta. Cuesta: la vida corre
//! ~330.000 ticks/s en la grilla (antes ~540.000 con la del sistema).
//! Rango que usa Angri: exp de x ≤ 0 (softmax: x − máximo), sin/cos de ángulos
//! chicos (el río y las rocas del bosque), pow de base en (0, 1] con exponente
//! entero ≥ 0 (el rastro del predador).

#[derive(Clone, Copy)]
struct Dd(f64, f64); // valor = .0 + .1, con |.1| ≤ ulp(.0)/2

#[inline]
fn dos_sumas(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    (s, (a - (s - bb)) + (b - bb))
}

#[inline]
fn rapida(a: f64, b: f64) -> Dd {
    let s = a + b;
    Dd(s, b - (s - a))
}

#[inline]
fn partir(a: f64) -> (f64, f64) {
    let c = 134217729.0 * a; // 2^27 + 1 (Dekker)
    let hi = c - (c - a);
    (hi, a - hi)
}

#[inline]
fn dos_productos(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let (ah, al) = partir(a);
    let (bh, bl) = partir(b);
    (p, ((ah * bh - p) + ah * bl + al * bh) + al * bl)
}

impl Dd {
    #[inline]
    fn mas(self, o: Dd) -> Dd {
        let (s, e) = dos_sumas(self.0, o.0);
        let (t, f) = dos_sumas(self.1, o.1);
        let r = rapida(s, e + t);
        rapida(r.0, r.1 + f)
    }
    #[inline]
    fn por(self, o: Dd) -> Dd {
        let (p, e) = dos_productos(self.0, o.0);
        rapida(p, e + (self.0 * o.1 + self.1 * o.0))
    }
    #[inline]
    fn entre_f(self, b: f64) -> Dd {
        let q1 = self.0 / b;
        let (p, e) = dos_productos(q1, b);
        let r = self.mas(Dd(-p, -e));
        let q2 = r.0 / b;
        rapida(q1, q2)
    }
    #[inline]
    fn neg(self) -> Dd {
        Dd(-self.0, -self.1)
    }
}

fn dos_a_la(e: i32) -> f64 {
    debug_assert!((-1022..=1023).contains(&e));
    f64::from_bits(((1023 + e) as u64) << 52)
}

/// (d.0 + d.1) · 2^e redondeado una sola vez, también si cae en los subnormales.
/// d tiene que estar normalizado y lejos de los subnormales (|d.0| en [2^-600, 2^600]).
fn escalar(d: Dd, e: i64) -> f64 {
    if d.0 == 0.0 {
        return 0.0;
    }
    let ex = ((d.0.to_bits() >> 52) & 0x7ff) as i64 - 1023; // exponente de d.0
    let total = ex + e;
    if total > 1023 {
        return f64::INFINITY.copysign(d.0);
    }
    if total >= -1022 {
        // normal: escalar por partes es exacto
        let mut v = d.0 + d.1;
        let mut k = e;
        while k != 0 {
            let paso = k.clamp(-1000, 1000);
            v *= dos_a_la(paso as i32);
            k -= paso;
        }
        return v;
    }
    if total < -1076 {
        return 0.0f64.copysign(d.0);
    }
    // subnormal: llevarlo a unidades de 2^-1074 (exacto) y redondear al entero par más cercano
    let mut k = e + 1074;
    let (mut h, mut l) = (d.0, d.1);
    while k != 0 {
        let paso = k.clamp(-1000, 1000);
        let f = dos_a_la(paso as i32);
        h *= f;
        l *= f;
        k -= paso;
    }
    let mut n = h.round_ties_even();
    let resto = (h - n) + l; // h − n es exacto (|h| < 2^53)
    if resto > 0.5 || (resto == 0.5 && (n / 2.0).fract() != 0.0) {
        n += 1.0;
    } else if resto < -0.5 || (resto == -0.5 && (n / 2.0).fract() != 0.0) {
        n -= 1.0;
    }
    n * f64::from_bits(1) // n · 2^-1074, exacto
}

// constantes en triple double (de la biblioteca QD de Bailey)
const LN2: [f64; 3] = [6.931471805599452862e-01, 2.319046813846299558e-17, 5.707708438416212066e-34];
const PI_2: [f64; 3] = [1.570796326794896558e+00, 6.123233995736766036e-17, -1.497384904859169777e-33];

/// x − k·c (c en triple double), en doble-doble.
fn reducir(x: f64, k: f64, c: &[f64; 3]) -> Dd {
    let (p0, e0) = dos_productos(k, c[0]);
    let (p1, e1) = dos_productos(k, c[1]);
    let r = Dd(x, 0.0).mas(Dd(-p0, -e0));
    r.mas(Dd(-p1, -e1)).mas(Dd(-(k * c[2]), 0.0))
}

/// Serie de Taylor Σ rⁿ/n! empezando en el término `t` (n = n0), hasta que no suma.
fn serie(mut t: Dd, n0: u32, paso: u32, r_mult: Dd, signo: bool) -> Dd {
    let mut s = t;
    let mut n = n0;
    loop {
        // t ← t · r_mult / ((n+1)(n+2)...) según el paso (1 para exp, 2 para sin/cos)
        t = t.por(r_mult);
        for j in 1..=paso {
            t = t.entre_f((n + j) as f64);
        }
        n += paso;
        if signo {
            t = t.neg();
        }
        if t.0.abs() <= s.0.abs() * 1e-34 || t.0 == 0.0 {
            return s;
        }
        s = s.mas(t);
    }
}

/// Tablas de exp (se calculan una vez, con la serie lenta): 2^(j/64) y 1/n!.
struct TablasExp {
    dos: [Dd; 64],
    inv: [Dd; 13],
}

fn tablas() -> &'static TablasExp {
    static T: std::sync::OnceLock<TablasExp> = std::sync::OnceLock::new();
    T.get_or_init(|| {
        let mut inv = [Dd(1.0, 0.0); 13];
        for n in 1..13 {
            inv[n] = inv[n - 1].entre_f(n as f64);
        }
        let ln2 = Dd(LN2[0], LN2[1]);
        let mut dos = [Dd(1.0, 0.0); 64];
        for (j, d) in dos.iter_mut().enumerate().skip(1) {
            // j·ln2/64 en doble-doble (÷64 es exacto) y su exp por la serie entera
            let a = ln2.por(Dd(j as f64, 0.0)).mas(Dd(j as f64 * LN2[2], 0.0));
            let a = Dd(a.0 / 64.0, a.1 / 64.0);
            *d = serie(Dd(1.0, 0.0), 0, 1, a, false);
        }
        TablasExp { dos, inv }
    })
}

pub fn exp(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if x > 709.782712893384 {
        return f64::INFINITY;
    }
    if x < -745.2 {
        return 0.0;
    }
    if x == 0.0 {
        return 1.0;
    }
    // x = (64m + j)·ln2/64 + r, con |r| ≤ ln2/128: exp(x) = 2^m · 2^(j/64) · exp(r)
    let t = tablas();
    let k = (x * (64.0 / LN2[0])).round_ties_even();
    let c = [LN2[0] / 64.0, LN2[1] / 64.0, LN2[2] / 64.0];
    let r = reducir(x, k, &c);
    // la cola (n ≥ 6, < 4e-17 del total) alcanza con doubles; lo grande, en doble-doble
    let mut cola = t.inv[12].0;
    for n in (6..12).rev() {
        cola = cola * r.0 + t.inv[n].0;
    }
    let mut p = t.inv[5].mas(Dd(cola * r.0, 0.0));
    for n in (0..5).rev() {
        p = p.por(r).mas(t.inv[n]);
    }
    let ki = k as i64;
    let (m, j) = (ki.div_euclid(64), ki.rem_euclid(64) as usize);
    escalar(p.por(t.dos[j]), m)
}

/// sin y cos de r reducido (|r| ≤ π/4), en doble-doble.
fn sin_cos_red(r: Dd) -> (Dd, Dd) {
    let r2 = r.por(r);
    let s = serie(r, 1, 2, r2, true);
    let c = serie(Dd(1.0, 0.0), 0, 2, r2, true);
    (s, c)
}

fn cuadrante(x: f64) -> (i64, Dd) {
    assert!(x.abs() < 1e6, "mate: ángulo fuera de rango ({x})");
    let k = (x / PI_2[0]).round_ties_even();
    (k as i64, reducir(x, k, &PI_2))
}

pub fn sin(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return if x.is_finite() { x } else { f64::NAN };
    }
    let (k, r) = cuadrante(x);
    let (s, c) = sin_cos_red(r);
    let v = match k.rem_euclid(4) { 0 => s, 1 => c, 2 => s.neg(), _ => c.neg() };
    v.0 + v.1
}

pub fn cos(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let (k, r) = cuadrante(x);
    let (s, c) = sin_cos_red(r);
    let v = match k.rem_euclid(4) { 0 => c, 1 => s.neg(), 2 => c.neg(), _ => s };
    v.0 + v.1
}

/// b^n con n entero ≥ 0 (el único uso: el rastro que se desvanece).
pub fn pow(b: f64, n: f64) -> f64 {
    assert!(n >= 0.0 && n == n.trunc() && n < 9.0e15, "mate::pow solo con exponente entero ≥ 0 ({n})");
    if b == 0.0 {
        return if n == 0.0 { 1.0 } else { 0.0 };
    }
    // mantisas en doble-doble en [1, 2) y los exponentes de 2 aparte: nada se
    // acerca a los subnormales hasta el redondeo final
    fn llevar(d: Dd, e: &mut i64) -> Dd {
        let ex = ((d.0.to_bits() >> 52) & 0x7ff) as i64 - 1023;
        *e += ex;
        let f = dos_a_la((-ex) as i32);
        Dd(d.0 * f, d.1 * f)
    }
    let mut eb = 0i64;
    let mut base = llevar(Dd(b, 0.0), &mut eb);
    let (mut r, mut er) = (Dd(1.0, 0.0), 0i64);
    let mut m = n as u64;
    while m > 0 {
        if m & 1 == 1 {
            er += eb;
            r = llevar(r.por(base), &mut er);
            if er < -1200 {
                return 0.0;
            }
        }
        m >>= 1;
        if m > 0 {
            eb = (2 * eb).max(-1_000_000); // (más abajo ya es 0 a efectos prácticos)
            base = llevar(base.por(base), &mut eb);
        }
    }
    escalar(r, er)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn ulps(a: f64, b: f64) -> u64 {
        (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs()
    }

    #[test]
    fn valores_conocidos() {
        assert_eq!(exp(1.0), std::f64::consts::E);
        assert_eq!(exp(-1.0), 0.36787944117144233);
        assert_eq!(exp(-745.0), 5e-324);
        assert_eq!(exp(-746.0), 0.0);
        // (valores exactos calculados aparte con 80 dígitos)
        assert_eq!(sin(0.5235987755982988), 0.49999999999999994);
        assert_eq!(sin(3.7), -0.5298361409084934);
        assert_eq!(sin(12.345), -0.2195669967379325);
        assert_eq!(exp(-3.3), 0.036883167401240015);
        assert_eq!(exp(-0.7), 0.4965853037914095);
        assert_eq!(sin(std::f64::consts::PI), 1.2246467991473532e-16);
        assert_eq!(pow(0.5, 10.0), 0.0009765625);
        // subnormales: un solo redondeo (la biblioteca de C de Windows redondea dos veces)
        assert_eq!(pow(0.17735529327889688, 413.0), 5.924284799114e-311);
        assert_eq!(pow(0.06686857579810146, 269.0), 9.654677e-317);
        assert_eq!(pow(0.9, 0.0), 1.0);
        assert_eq!(pow(0.95, 1e6), 0.0);
    }

    #[test]
    fn cerca_de_la_del_sistema() {
        // a lo sumo 1 ulp de la biblioteca de C (que tampoco es exacta siempre)
        let mut r = crate::azar::Azar::new(1);
        for _ in 0..100_000 {
            let x = -40.0 * r.random();
            assert!(ulps(exp(x), x.exp()) <= 1, "exp({x})");
            let a = 20.0 * r.random();
            assert!(ulps(sin(a), a.sin()) <= 1, "sin({a})");
            assert!(ulps(cos(a), a.cos()) <= 1, "cos({a})");
            let b = r.random();
            let n = r.randrange(0, 500) as f64;
            assert!(ulps(pow(b, n), b.powf(n)) <= 1, "pow({b}, {n})");
        }
    }
}
