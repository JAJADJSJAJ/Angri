//! mente.py (la mente vieja: azar o reflejos), corteza.py y talamo.py.
use crate::azar::Azar;
use crate::config::Config;
use crate::mente::{alivia, hacia, psum, MOVIMIENTOS};
use crate::mundo::*;
use std::collections::HashMap;

// ------------------------------------------------------------------ mente vieja
pub struct MenteVieja {
    pub rng: Azar,
    prediccion: Option<u8>,
}

impl MenteVieja {
    pub fn new(semilla: u64) -> MenteVieja {
        MenteVieja { rng: Azar::new(semilla), prediccion: None }
    }

    pub fn elegir(&mut self, vista: &Vista, necesidad: usize, c: &Config) -> (i64, i64) {
        if !c.reflejos {
            return self.rng.choice(&MOVIMIENTOS);
        }
        // (mente.ALIVIA: solo energía, cansancio y curiosidad)
        let objetivo = match necesidad { 0 => COMIDA, 1 => DESCANSO, 2 => NOVEDAD, _ => 255 };
        if objetivo != 255 {
            for &m in &MOVIMIENTOS[..4] {
                if vista.get(m) == Some(objetivo) {
                    return m;
                }
            }
        }
        let seguros: Vec<(i64, i64)> = MOVIMIENTOS.iter().copied()
            .filter(|&m| matches!(vista.get(m), Some(x) if x != PELIGRO)).collect();
        if seguros.is_empty() { self.rng.choice(&MOVIMIENTOS) } else { self.rng.choice(&seguros) }
    }

    pub fn sorpresa(&mut self, encontrado: u8) -> f64 {
        let esperado = self.prediccion;
        self.prediccion = Some(encontrado);
        match esperado {
            None => 0.0,
            Some(e) => if e == encontrado { 0.0 } else { 1.0 },
        }
    }
}

// ------------------------------------------------------------------ corteza
const N_CELDAS: usize = 6; // · * ~ ? X #  (el agua cuenta como ·)

pub struct Corteza {
    pub wc: HashMap<u32, [f64; N_CELDAS]>,
    pub wm: HashMap<u32, f64>,
    pub rapido: HashMap<u32, f64>,
    pub lento: HashMap<u32, f64>,
    rasgos: Option<[u32; 5]>,
    clave: u32,
    pc: [f64; N_CELDAS],
    pm: f64,
    pub error: f64,
    pub progreso: f64,
}

impl Corteza {
    pub fn new() -> Corteza {
        Corteza { wc: HashMap::new(), wm: HashMap::new(), rapido: HashMap::new(), lento: HashMap::new(),
                  rasgos: None, clave: 0, pc: [0.0; N_CELDAS], pm: 0.0, error: 0.0, progreso: 0.0 }
    }

    fn zona(pos: (i64, i64), c: &Config) -> u32 {
        ((3 * pos.0).div_euclid(c.ancho).min(2) * 3 + (3 * pos.1).div_euclid(c.alto).min(2)) as u32
    }

    /// visto: lo que había en el destino (vista.get((dx, dy), "#"))
    pub fn predecir(&mut self, visto: u8, necesidad: usize, quieto: bool, pos: (i64, i64), c: &Config) {
        let (v, n, q) = (visto as u32, necesidad as u32, quieto as u32);
        let r = [0x10000 | v, 0x20000 | (v * 16 + n), 0x30000 | (q * 16 + n),
                 0x40000 | (Self::zona(pos, c) * 16 + v), 0x50000];
        self.rasgos = Some(r);
        self.clave = v * 16 + n;
        let mut logits = [0.0f64; N_CELDAS];
        for k in &r {
            if let Some(w) = self.wc.get(k) {
                for i in 0..N_CELDAS {
                    logits[i] += w[i];
                }
            }
        }
        let mut m = logits[0];
        for &l in &logits[1..] {
            if l > m {
                m = l;
            }
        }
        let mut e = [0.0f64; N_CELDAS];
        for i in 0..N_CELDAS {
            e[i] = (logits[i] - m).exp();
        }
        let z = psum(e.iter().copied());
        for i in 0..N_CELDAS {
            self.pc[i] = e[i] / z;
        }
        self.pm = psum(r.iter().map(|k| *self.wm.get(k).unwrap_or(&0.0)));
    }

    pub fn comparar(&mut self, encontrado: u8, delta_malestar: f64, c: &Config) -> f64 {
        let r = match self.rasgos { None => return 0.0, Some(r) => r };
        let j = if (encontrado as usize) < N_CELDAS { encontrado as usize } else { 0 };
        let err_c = 1.0 - self.pc[j];
        let err_m = delta_malestar - self.pm;
        let a = c.aprende_corteza / r.len() as f64;
        for k in &r {
            let wc = self.wc.entry(*k).or_insert([0.0; N_CELDAS]);
            for i in 0..N_CELDAS {
                wc[i] += a * (if i == j { 1.0 } else { 0.0 } - self.pc[i]);
            }
            let wm = self.wm.entry(*k).or_insert(0.0);
            *wm = *wm + a * err_m * c.escala_error_malestar;
        }
        self.error = 1.0f64.min(err_c + c.escala_error_malestar * err_m.abs());
        let k = self.clave;
        let mut rr = *self.rapido.get(&k).unwrap_or(&self.error);
        let mut ll = *self.lento.get(&k).unwrap_or(&self.error);
        rr += c.curiosidad_rapido * (self.error - rr);
        ll += c.curiosidad_lento * (self.error - ll);
        self.rapido.insert(k, rr);
        self.lento.insert(k, ll);
        self.progreso = if ll - rr > 0.0 { ll - rr } else { 0.0 };
        self.rasgos = None;
        self.error
    }
}

// ------------------------------------------------------------------ tálamo
pub struct Talamo {
    pub sesgo: [[f64; 2]; 5],
    pub foco: Option<usize>,
    ctx: [usize; 5],
    probs: Option<[f64; 5]>,
}

impl Talamo {
    pub fn new() -> Talamo {
        Talamo { sesgo: [[0.0; 2]; 5], foco: None, ctx: [0; 5], probs: None }
    }

    fn visible(vista: &Vista, n: usize) -> usize {
        let mut objetivo = alivia(n);
        if objetivo == 255 {
            objetivo = PELIGRO;
        }
        (vista.get((0, 0)) == Some(objetivo) || hacia(vista, objetivo) != 4) as usize
    }

    /// usa el MISMO azar que los ganglios (rng = mente.rng en Python)
    pub fn elegir(&mut self, vista: &Vista, cuerpo: &[f64; 7], rng: &mut Azar, c: &Config) -> usize {
        let mut urg = [0.0f64; 5];
        for n in 0..5 {
            urg[n] = c.peso[n] * (cuerpo[n] - c.equilibrio[n]).abs();
        }
        let mut tope = urg[0];
        for &u in &urg[1..] {
            if u > tope {
                tope = u;
            }
        }
        if tope == 0.0 {
            tope = 1.0;
        }
        let mut ctx = [0usize; 5];
        let mut sal = [0.0f64; 5];
        for n in 0..5 {
            ctx[n] = Self::visible(vista, n);
            sal[n] = c.ganancia_urgencia * urg[n] / tope + self.sesgo[n][ctx[n]];
        }
        if let Some(f) = self.foco {
            sal[f] += c.inercia_foco;
        }
        let mut m = sal[0];
        for &s in &sal[1..] {
            if s > m {
                m = s;
            }
        }
        let mut e = [0.0f64; 5];
        for n in 0..5 {
            e[n] = ((sal[n] - m) / c.temperatura_foco).exp();
        }
        let z = psum(e.iter().copied());
        let mut probs = [0.0f64; 5];
        for n in 0..5 {
            probs[n] = e[n] / z;
        }
        let f = rng.choices_pesos(&probs, 1)[0];
        self.probs = Some(probs);
        self.foco = Some(f);
        self.ctx = ctx;
        f
    }

    pub fn aprender(&mut self, dopamina: f64, c: &Config) {
        let probs = match self.probs { None => return, Some(p) => p };
        for n in 0..5 {
            let g = (if Some(n) == self.foco { 1.0 } else { 0.0 }) - probs[n];
            self.sesgo[n][self.ctx[n]] += c.aprende_talamo * dopamina * g;
        }
    }

    pub fn olvidar_paso(&mut self) {
        self.probs = None;
        self.foco = None;
    }
}

crate::pieza!(MenteVieja { rng, prediccion });
crate::pieza!(Corteza { wc, wm, rapido, lento, rasgos, clave, pc, pm, error, progreso });
crate::pieza!(Talamo { sesgo, foco, ctx, probs });
