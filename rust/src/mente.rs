//! cuerpo.py, quimica.py, ganglios.py e hipocampo.py.
use crate::azar::Azar;
use crate::config::Config;
use crate::mundo::*;
use std::collections::HashMap;

/// sum() de Python 3.12 sobre floats (Neumaier, como CPython).
#[inline]
pub fn psum<I: IntoIterator<Item = f64>>(xs: I) -> f64 {
    let (mut f, mut c) = (0.0f64, 0.0f64);
    for x in xs {
        let t = f + x;
        if f.abs() >= x.abs() {
            c += (f - t) + x;
        } else {
            c += (x - t) + f;
        }
        f = t;
    }
    if c != 0.0 && c.is_finite() {
        f += c;
    }
    f
}

#[inline]
fn clip(x: f64) -> f64 {
    0.0f64.max(x.min(1.0))
}
// max/min de Python con floats sin NaN dan lo mismo que estos.

pub const ENERGIA: usize = 0;
pub const CANSANCIO: usize = 1;
pub const CURIOSIDAD: usize = 2;
pub const ESTRES: usize = 3;
pub const INTEGRIDAD: usize = 4;
pub const AGUA_N: usize = 5;
pub const CALOR: usize = 6;

/// qué alivia cada necesidad (ganglios.ALIVIA); 255 = nada
pub fn alivia(n: usize) -> u8 {
    match n {
        ENERGIA => COMIDA,
        CANSANCIO => DESCANSO,
        CURIOSIDAD => NOVEDAD,
        AGUA_N => AGUA,
        CALOR => DESCANSO,
        _ => 255,
    }
}

// ------------------------------------------------------------------ cuerpo
pub struct Cuerpo {
    pub v: [f64; 7],
    pub n: usize, // 5 en la grilla, 7 en el bosque
}

impl Cuerpo {
    pub fn new(bosque: bool, c: &Config) -> Cuerpo {
        Cuerpo { v: c.equilibrio, n: if bosque { 7 } else { 5 } }
    }

    fn clip_todo(&mut self) {
        for k in 0..self.n {
            self.v[k] = clip(self.v[k]);
        }
    }

    pub fn vivir(&mut self, despierto: bool, se_movio: bool, c: &Config) {
        let v = &mut self.v;
        v[ENERGIA] -= c.gasto_energia_base + if se_movio { c.gasto_energia_mover } else { 0.0 };
        if despierto {
            v[CANSANCIO] += c.sube_cansancio_despierto;
            v[CURIOSIDAD] += c.sube_curiosidad;
        } else {
            v[CANSANCIO] -= c.sube_cansancio_despierto * 4.0;
            v[ENERGIA] += c.gasto_energia_base * 0.5;
        }
        v[ESTRES] -= c.calma_estres * v[ESTRES];
        v[INTEGRIDAD] += c.recupera_integridad;
        if v[ENERGIA] < 0.05 {
            v[INTEGRIDAD] -= c.dano_inanicion;
        }
        if v[ENERGIA] < 0.3 && c.estres_cobra != 2 {
            v[ESTRES] += 0.002;
        }
        if v[INTEGRIDAD] < 0.7 && c.estres_cobra != 2 {
            v[ESTRES] += 0.003;
        }
        self.clip_todo();
    }

    pub fn clima(&mut self, despierto: bool, se_movio: bool, aire: f64, c: &Config) {
        let v = &mut self.v;
        v[AGUA_N] -= c.gasto_agua * if despierto { 1.0 } else { 0.5 };
        v[CALOR] += c.templa * (aire + if se_movio { 0.1 } else { 0.0 } - v[CALOR]);
        if v[CALOR] < 0.3 {
            v[INTEGRIDAD] -= c.dano_frio;
            if c.estres_cobra != 2 { v[ESTRES] += 0.002; }
        }
        if v[AGUA_N] < 0.05 || v[ENERGIA] < 0.05 {
            v[INTEGRIDAD] -= c.recupera_integridad + c.dano_sed;
        }
        if v[AGUA_N] < 0.3 && c.estres_cobra != 2 {
            v[ESTRES] += 0.002;
        }
        self.clip_todo();
    }

    pub fn beber(&mut self, c: &Config) {
        self.v[AGUA_N] = clip(self.v[AGUA_N] + c.beber);
    }
    pub fn enfermar(&mut self) {
        self.v[INTEGRIDAD] = clip(self.v[INTEGRIDAD] - 0.1);
        self.v[ESTRES] = clip(self.v[ESTRES] + 0.2);
        self.v[ENERGIA] = clip(self.v[ENERGIA] - 0.1);
    }
    pub fn lastimarse_mucho(&mut self) {
        self.v[INTEGRIDAD] = clip(self.v[INTEGRIDAD] - 0.3);
        self.v[ESTRES] = clip(self.v[ESTRES] + 0.5);
    }
    pub fn rasparse(&mut self) {
        self.v[INTEGRIDAD] = clip(self.v[INTEGRIDAD] - 0.06);
        self.v[ESTRES] = clip(self.v[ESTRES] + 0.15);
    }
    pub fn comer(&mut self, cuanto: f64) {
        self.v[ENERGIA] = clip(self.v[ENERGIA] + cuanto);
    }
    pub fn descansar(&mut self) {
        self.v[CANSANCIO] = clip(self.v[CANSANCIO] - 0.01);
    }
    pub fn descubrir(&mut self) {
        self.v[CURIOSIDAD] = clip(self.v[CURIOSIDAD] - 0.3);
    }
    pub fn lastimarse(&mut self) {
        self.v[INTEGRIDAD] = clip(self.v[INTEGRIDAD] - 0.15);
        self.v[ESTRES] = clip(self.v[ESTRES] + 0.3);
    }

    pub fn malestar(&self, c: &Config) -> f64 {
        let total = psum((0..self.n).map(|k| c.peso[k] * (self.v[k] - c.equilibrio[k]).abs()));
        let maximo = psum((0..self.n).map(|k| c.peso[k]));
        (total / maximo).min(c.malestar_maximo)
    }

    pub fn necesidad_urgente(&self, c: &Config) -> usize {
        let mut m = 0;
        let mut vm = f64::NEG_INFINITY;
        for k in 0..self.n {
            let x = c.peso[k] * (self.v[k] - c.equilibrio[k]).abs();
            if x > vm {
                vm = x;
                m = k;
            }
        }
        m
    }
}

// ------------------------------------------------------------------ química
pub struct Quimica {
    pub adenosina: f64,
    pub cortisol: f64,
    pub dopamina: f64,
    pub noradrenalina: f64,
    previo: Option<f64>,
}

impl Quimica {
    pub fn new() -> Quimica {
        Quimica { adenosina: 0.0, cortisol: 0.0, dopamina: 0.0, noradrenalina: 0.0, previo: None }
    }
    pub fn actualizar(&mut self, despierto: bool, malestar: f64, estres: f64, sorpresa: f64, c: &Config) {
        if despierto {
            self.adenosina += c.adenosina_por_actividad;
        } else {
            self.adenosina -= c.adenosina_limpieza_sueno;
        }
        if estres > 0.4 {
            self.cortisol += c.cortisol_sube * (estres - 0.4);
        }
        self.cortisol -= c.cortisol_baja;
        self.dopamina = match self.previo {
            None => 0.0,
            Some(p) => (-1.0f64).max(1.0f64.min((p - malestar) * 50.0)),
        };
        self.previo = Some(malestar);
        self.noradrenalina = 0.9 * self.noradrenalina + 0.1 * sorpresa;
        self.adenosina = clip(self.adenosina);
        self.cortisol = clip(self.cortisol);
        self.noradrenalina = clip(self.noradrenalina);
    }
}

// ------------------------------------------------------------------ ganglios
pub const MOVIMIENTOS: [(i64, i64); 5] = [(0, -1), (0, 1), (-1, 0), (1, 0), (0, 0)];
/// acciones: 5 movimientos y (con OBJETOS) agarrar, soltar, apilar, atar, golpear, frotar
pub const NACC: usize = 11;

/// (etapa 14) las unidades propias activas en s y en s2 de un recuerdo
pub type ActivasMomento = (Vec<(u32, u8)>, Vec<(u32, u8)>);
pub const AGARRAR: usize = 5;
pub const SOLTAR: usize = 6;
pub const APILAR: usize = 7;
pub const ATAR: usize = 8;
pub const GOLPEAR: usize = 9;
pub const FROTAR: usize = 10;
const N_RASGOS: usize = 764;

#[derive(Clone, Copy)]
pub struct Rasgos {
    pub idx: [[u16; MAXR]; NACC],
    pub n: [u8; NACC],
    pub na: u8, // cuántas acciones hay (5 o 11)
    pub posibles: u16, // bit a: la acción a se puede hacer ahora (no se suelta lo que no se tiene)
}

impl Rasgos {
    #[inline]
    pub fn puede(&self, a: usize) -> bool {
        self.posibles >> a & 1 == 1
    }
}

/// (etapa 13b) qué acciones con objetos tienen sentido físico ahora
fn posibles(t: &Tacto) -> u16 {
    let mut p: u16 = 0b11111;
    let lleva = t.lleva.is_some();
    let hay = t.aqui.is_some();
    if !lleva && hay { p |= 1 << AGARRAR; }
    if lleva { p |= 1 << SOLTAR | 1 << APILAR; }
    if lleva && t.h_aqui >= 2 { p |= 1 << ATAR; }
    if t.dura || (lleva && hay) { p |= 1 << GOLPEAR; }
    if lleva && hay { p |= 1 << FROTAR; }
    p
}

/// rasgos por acción: 8 de siempre + 8 de PALABRA_SUELTA + 28 de FRASE
pub const MAXR: usize = 44;

impl Default for Rasgos {
    fn default() -> Rasgos {
        Rasgos { idx: [[0; MAXR]; NACC], n: [0; NACC], na: 5, posibles: 0b11111 }
    }
}

impl Rasgos {
    #[inline]
    pub fn de(&self, a: usize) -> &[u16] {
        &self.idx[a][..self.n[a] as usize]
    }
}

#[derive(Clone, Copy)]
pub struct Momento {
    pub s: u32,
    pub rasgos: Rasgos,
    pub a: usize,
    pub r: f64,
    pub s2: u32,
    pub pos: (i64, i64),
    pub necesidad: usize,
    pub p: f64,
    pub temp: [f64; NACC],
}

pub fn hacia(vista: &Vista, objetivo: u8) -> usize {
    if objetivo == 255 {
        return 4;
    }
    let (mut mejor, mut dist) = (4, 99);
    for &((dx, dy), c) in &vista.v {
        if c == objetivo && (dx, dy) != (0, 0) {
            let d = dx.abs() + dy.abs();
            if d < dist {
                dist = d;
                mejor = if dy.abs() >= dx.abs() {
                    if dy < 0 { 0 } else { 1 }
                } else if dx < 0 { 2 } else { 3 };
            }
        }
    }
    mejor
}

fn direccion(dx: i64, dy: i64) -> usize {
    if dx == 0 && dy == 0 {
        return 4;
    }
    if dy.abs() >= dx.abs() {
        if dy < 0 { 0 } else { 1 }
    } else if dx < 0 { 2 } else { 3 }
}

fn rasgos_de(vista: &Vista, n: usize, dc: usize, d_o: usize, dr: usize, a: usize, dd: usize,
             palabras: &[usize], c: &Config, out: &mut [u16; MAXR]) -> u8 {
    let destino = vista.get(MOVIMIENTOS[a]).unwrap_or(ROCA) as usize;
    let mut k = 0usize;
    let mut push = |x: usize| {
        out[k] = x as u16;
        k += 1;
    };
    if n >= 5 || destino >= 6 {
        push(if destino < 6 && n < 5 { destino * 5 + n } else { 422 + destino * 7 + n });
        if a == dc { push(if n < 5 { 30 + n } else { 471 + n }); }
        if a == d_o { push(if n < 5 { 35 + n } else { 478 + n }); }
        if a == 4 { push(if n < 5 { 40 + n } else { 485 + n }); }
        if a == dr { push(if n < 5 { 45 + n } else { 492 + n }); }
        push(if destino < 6 { 50 + destino } else { 506 });
        if a == 4 { push(56); }
        if a == dd { push(if n < 5 { 417 + n } else { 499 + n }); }
    } else {
        push(destino * 5 + n);
        if a == dc { push(30 + n); }
        if a == d_o { push(35 + n); }
        if a == 4 { push(40 + n); }
        if a == dr { push(45 + n); }
        push(50 + destino);
        if a == 4 { push(56); }
        if c.palabra_suelta {
            for &p in palabras { push(57 + p * 5 + a); }
        }
        if c.frase {
            for (i, &p) in palabras.iter().enumerate() {
                for &q in &palabras[i + 1..] { push(97 + (p * 8 + q) * 5 + a); }
            }
        }
        if a == dd { push(417 + n); }
    }
    k as u8
}

/// (etapa 13b) lo que siente de los objetos al moverse hacia `a`
fn rasgos_mover(t: &Tacto, n: usize, a: usize, out: &mut [u16; MAXR], k0: u8) -> u8 {
    let mut k = k0 as usize;
    let mut push = |x: usize| {
        out[k] = x as u16;
        k += 1;
    };
    let h = t.h_dest[a];
    if h == 1 { push(729 + n); }
    if h >= 2 { push(736 + n); }
    if t.techo_dest[a] { push(743 + n); }
    if t.fuego_dest[a] { push(750 + n); }
    if a < 4 && t.lleva.is_some() { push(757 + n); }
    k as u8
}

/// (etapa 13b) rasgos de una acción con objetos (k = 0..6: agarrar..frotar)
fn rasgos_objeto(t: &Tacto, n: usize, k: usize, out: &mut [u16; MAXR]) -> u8 {
    let mut j = 0usize;
    let mut push = |x: usize| {
        out[j] = x as u16;
        j += 1;
    };
    push(507 + k * 7 + n);
    push(549 + k * 2 + t.lleva.is_some() as usize);
    push(561 + k * 2 + t.aqui.is_some() as usize);
    if let Some(o) = t.lleva {
        for (p, &si) in o.props().iter().enumerate() {
            if si { push(573 + k * 6 + p); }
        }
    }
    if let Some(o) = t.aqui {
        for (p, &si) in o.props().iter().enumerate() {
            if si { push(609 + k * 6 + p); }
        }
    }
    push(645 + k * 2 + t.clima_malo as usize);
    push(657 + k * 3 + t.h_aqui.min(2) as usize);
    push(675 + k * 5 + t.paredes.min(4) as usize);
    push(705 + k * 2 + t.dura as usize);
    push(717 + k * 2 + t.cubierto as usize);
    j as u8
}

/// dict de Python {(x, y): fuerza} con su orden de inserción.
#[derive(Default)]
pub struct Lugares {
    pub v: Vec<((i64, i64), f64)>,
}

impl Lugares {
    fn get(&self, k: (i64, i64)) -> Option<f64> {
        self.v.iter().find(|e| e.0 == k).map(|e| e.1)
    }
    fn poner(&mut self, k: (i64, i64), f: f64) {
        if let Some(e) = self.v.iter_mut().find(|e| e.0 == k) {
            e.1 = f;
        } else {
            self.v.push((k, f));
        }
    }
    fn borrar(&mut self, k: (i64, i64)) {
        if let Some(i) = self.v.iter().position(|e| e.0 == k) {
            self.v.remove(i);
        }
    }
    fn decaer(&mut self, f: f64) {
        self.v.retain_mut(|e| {
            e.1 *= f;
            e.1 >= 0.05
        });
    }
}

pub struct Ganglios {
    pub rng: Azar,
    v: HashMap<u32, f64>,
    exp_v: HashMap<u32, i64>,
    w: Vec<f64>,
    exp_w: Vec<i64>,
    pub lugares: [Lugares; 7],
    huella_v: Vec<(u32, f64)>,
    huella_w: Vec<f64>,
    activo: Vec<bool>,
    activos: Vec<usize>,
    prediccion: Option<u8>,
    s: Option<u32>,
    a: usize,
    rasgos: Rasgos,
    probs: [f64; NACC],
    temp: [f64; NACC],
    necesidad: usize,
    pub dopamina: f64,
    pub ultimo: Option<Momento>,
    pub propia: Option<crate::propia::Propia>, // etapa 14
    /// (con REPASO_PROPIO) lo que sintió en s y en s2 del último paso, para el hipocampo
    pub ultimo_sentir: Option<(crate::propia::Sentir, crate::propia::Sentir)>,
}

impl Ganglios {
    pub fn new(semilla: u64) -> Ganglios {
        Ganglios {
            rng: Azar::new(semilla), v: HashMap::new(), exp_v: HashMap::new(),
            w: vec![0.0; N_RASGOS], exp_w: vec![0; N_RASGOS], lugares: Default::default(),
            huella_v: vec![], huella_w: vec![0.0; N_RASGOS], activo: vec![false; N_RASGOS],
            activos: vec![], prediccion: None, s: None, a: 0, rasgos: Rasgos::default(),
            probs: [0.0; NACC], temp: [0.0; NACC], necesidad: 0, dopamina: 0.0, ultimo: None, propia: None, ultimo_sentir: None,
        }
    }

    fn franja(x: f64, c: &Config) -> u32 {
        ((x * c.franjas as f64) as i64).min(c.franjas - 1) as u32
    }

    fn resumen(&self, vista: &Vista, cuerpo: &[f64; 7], n: usize, ncuerpo: usize, tacto: Option<&Tacto>, c: &Config) -> u32 {
        let celda = vista.get((0, 0)).unwrap_or(7) as u32;
        let come = (c.atajo_veo_comida && hacia(vista, COMIDA) != 4) as u32;
        let g = if self.propia.is_none() { c.critico_cuerpo } else { c.critico_cuerpo_propio };
        let (g, foco) = if g < 0 { (-g, n as u32 + 1) } else { (g, 0) };
        let s = if g > 0 {
            // (paso 2a) el cuerpo entero, sin foco: cuánto se aleja cada necesidad de su equilibrio
            // (la mano con CRITICO_CUERPO, la propia con CRITICO_CUERPO_PROPIO)
            let mut k: u32 = 0;
            for i in 0..ncuerpo {
                let eq = c.equilibrio[i];
                let lejos = (cuerpo[i] - eq).abs() / eq.max(1.0 - eq);
                k = k * g as u32 + ((lejos * g as f64) as i64).min(g - 1) as u32;
            }
            ((foco * (g as u32).pow(ncuerpo as u32) + k) * 8 + celda) * 2 + come
        } else {
            (((n as u32 * 64 + Self::franja(cuerpo[ENERGIA], c)) * 64 + Self::franja(cuerpo[INTEGRIDAD], c)) * 8 + celda) * 2 + come
        };
        let s = match tacto {
            None => s,
            Some(t) => s * 4 + t.lleva.is_some() as u32 * 2 + t.cubierto as u32,
        };
        // (CRITICO_VECINOS) lo que tengo adelante: el código de la celda en la dirección del último paso
        // (8 = me quedé quieto). Nada le dice qué celda duele: V aprende sola si lo de adelante duele.
        let s = if c.critico_vecinos {
            let adelante = if self.a < 4 { vista.get(MOVIMIENTOS[self.a]).unwrap_or(7) as u32 } else { 8 };
            s * 9 + adelante
        } else {
            s
        };
        // (aprender las recetas, paso 4) la propia: "hay salida a la vista" aprendida (SALIDA_PROPIA)
        match &self.propia {
            Some(p) if c.salida_propia > 0 => s * c.salida_propia as u32 + p.salida,
            _ => s,
        }
    }

    #[inline]
    fn ritmo(base: f64, exp: i64, c: &Config) -> f64 {
        (base * c.aprende_minimo).max(base / (1.0 + exp as f64 / c.calma).sqrt())
    }

    fn recuerdo(&self, n: usize, pos: (i64, i64)) -> usize {
        let (mut mejor, mut valor) = (4, 0.0);
        let (x, y) = pos;
        for &((lx, ly), f) in &self.lugares[n].v {
            let v = f / (1 + (lx - x).abs() + (ly - y).abs()) as f64;
            if v > valor {
                mejor = direccion(lx - x, ly - y);
                valor = v;
            }
        }
        mejor
    }

    /// Elige una acción: 0..5 son MOVIMIENTOS; con `tacto` (OBJETOS) también AGARRAR..FROTAR.
    pub fn elegir(&mut self, vista: &Vista, cuerpo: &[f64; 7], n: usize, pos: (i64, i64),
                  noradrenalina: f64, malestar: f64, palabras: &[usize], dicha: usize,
                  tacto: Option<&Tacto>, clima: u8, ncuerpo: usize, c: &Config) -> usize {
        let dc = if c.atajo_comida { hacia(vista, COMIDA) } else { 99 };
        let d_o = match (c.foco_ciego, if c.atajo_alivia { alivia(n) } else { 255 }) {
            (_, 255) if !c.atajo_alivia => 99,
            (1, 255) => 99,
            (2, o) if o == 255 || hacia(vista, o) == 4 => 99,
            (_, o) => hacia(vista, o),
        };
        let dr = if c.atajo_lugares { self.recuerdo(n, pos) } else { 99 };
        let na = if tacto.is_some() { NACC } else { 5 };
        let mut rasgos = Rasgos { na: na as u8, ..Rasgos::default() };
        if let Some(t) = tacto {
            rasgos.posibles = posibles(t);
        }
        if c.rasgos_mano {
            for a in 0..5 {
                rasgos.n[a] = rasgos_de(vista, n, dc, d_o, dr, a, dicha, palabras, c, &mut rasgos.idx[a]);
                if let Some(t) = tacto {
                    rasgos.n[a] = rasgos_mover(t, n, a, &mut rasgos.idx[a], rasgos.n[a]);
                }
            }
            if let Some(t) = tacto {
                for a in 5..NACC {
                    rasgos.n[a] = rasgos_objeto(t, n, a - 5, &mut rasgos.idx[a]);
                }
            }
        }
        let mut h = [0.0f64; NACC];
        for a in 0..na {
            h[a] = psum(rasgos.de(a).iter().map(|&i| self.w[i as usize]));
        }
        if let Some(p) = &mut self.propia {
            let at = crate::propia::sentir(vista, cuerpo, ncuerpo, n, dr, dicha, tacto, clima, c.distancia_fina);
            p.preferencias(at, &mut h, na, cuerpo, c);
        }
        let mut tm = c.temperatura_malestar * malestar;
        if c.temperatura_malestar_tope > 0.0 {
            tm = tm.min(c.temperatura_malestar_tope);
        }
        let t_todo = c.temperatura_base + c.temperatura_sorpresa * noradrenalina + tm;
        let mut m = h[0];
        for b in 1..na {
            if rasgos.puede(b) && h[b] > m {
                m = h[b];
            }
        }
        let mut temp = [t_todo; NACC];
        if c.margen_explorar != 0.0 || c.peligro_sin_malestar {
            let t0 = c.temperatura_base + c.temperatura_sorpresa * noradrenalina;
            for b in 0..na {
                if (c.margen_explorar != 0.0 && h[b] < m - c.margen_explorar)
                    || (c.peligro_sin_malestar && b < 5 && vista.get(MOVIMIENTOS[b]) == Some(PELIGRO))
                {
                    temp[b] = t0;
                }
            }
        }
        let mut e = [0.0f64; NACC];
        for b in 0..na {
            if rasgos.puede(b) {
                e[b] = crate::mate::exp((h[b] - m) / temp[b]);
            }
        }
        let z = psum(e[..na].iter().copied());
        let mut probs = [0.0f64; NACC];
        for b in 0..na {
            probs[b] = e[b] / z;
        }
        let a = self.rng.choices_pesos(&probs[..na], 1)[0];
        self.s = Some(self.resumen(vista, cuerpo, n, ncuerpo, tacto, c));
        self.a = a;
        self.rasgos = rasgos;
        self.probs = probs;
        self.temp = temp;
        self.necesidad = n;
        a
    }

    pub fn aprender(&mut self, recompensa: f64, vista: &Vista, cuerpo: &[f64; 7], n: usize,
                    pos: (i64, i64), tacto: Option<&Tacto>, clima: u8, ncuerpo: usize, fin: bool, c: &Config) {
        let s = match self.s { None => return, Some(s) => s };
        for l in self.lugares.iter_mut() {
            l.decaer(c.recuerdo_decae);
        }
        if recompensa > c.umbral_recuerdo {
            self.lugares[self.necesidad].poner(pos, 1.0);
        }
        if c.borrar_lugar_vacio && c.atajo_alivia {
            let objetivo = alivia(n);
            if objetivo != 255 && self.lugares[n].get(pos).is_some()
                && vista.get((0, 0)) != Some(objetivo) && hacia(vista, objetivo) == 4
            {
                self.lugares[n].borrar(pos);
            }
        }
        // (paso 4e) con la propia, CRITICO_PROPIO = 'unidades' deja solo el V de las unidades
        let tabla = !(self.propia.is_some() && c.critico_propio == 2);
        let v = if tabla { *self.v.get(&s).unwrap_or(&0.0) } else { 0.0 };
        let dr = if self.propia.is_some() && c.atajo_lugares { self.recuerdo(n, pos) } else if self.propia.is_some() { 99 } else { 0 };
        // la propia mira s2 antes de armar la clave (SALIDA_PROPIA la usa)
        let at2 = match &mut self.propia {
            None => None,
            Some(p) => {
                let at = crate::propia::sentir(vista, cuerpo, ncuerpo, n, dr, 4, tacto, clima, c.distancia_fina);
                let vp2 = p.valor_de(&at, cuerpo, c);
                Some((at, vp2))
            }
        };
        let s2 = self.resumen(vista, cuerpo, n, ncuerpo, tacto, c);
        self.ultimo = Some(Momento { s, rasgos: self.rasgos, a: self.a, r: recompensa, s2, pos,
                                     necesidad: self.necesidad, p: self.probs[self.a], temp: self.temp });
        let v2_tabla = if tabla { *self.v.get(&s2).unwrap_or(&0.0) } else { 0.0 };
        // desmayo como final de episodio: no hay "después" que valga
        let g2 = if fin { 0.0 } else { c.serotonina };
        let delta = match (&mut self.propia, at2) {
            (Some(p), Some((at, vp2))) => {
                let (vp, vp2) = if c.critico_propio == 1 { (0.0, 0.0) } else { (p.vs, vp2) };
                let d = recompensa + g2 * (v2_tabla + vp2) - (v + vp);
                if c.repaso_propio {
                    self.ultimo_sentir = Some((p.ahora().clone(), at));
                }
                d
            }
            _ => recompensa + g2 * v2_tabla - v,
        };
        self.dopamina = delta;
        let decae = c.serotonina * c.huella;
        self.huella_v.retain_mut(|e| {
            e.1 *= decae;
            e.1 >= 0.01
        });
        if let Some(e) = self.huella_v.iter_mut().find(|e| e.0 == s) {
            e.1 = 1.0;
        } else {
            self.huella_v.push((s, 1.0));
        }
        *self.exp_v.entry(s).or_insert(0) += 1;
        for &(k, e) in self.huella_v.iter().filter(|_| tabla) {
            let ritmo = Self::ritmo(c.aprende_critico, *self.exp_v.get(&k).unwrap_or(&0), c);
            let x = self.v.entry(k).or_insert(0.0);
            *x = *x + ritmo * delta * e;
        }
        let hw = &mut self.huella_w;
        for &i in &self.activos {
            hw[i] *= decae;
        }
        for &i in self.rasgos.de(self.a) {
            hw[i as usize] += 1.0;
            self.exp_w[i as usize] += 1;
        }
        for b in 0..self.rasgos.na as usize {
            let p = self.probs[b];
            for &i in self.rasgos.de(b) {
                let i = i as usize;
                hw[i] -= p;
                if !self.activo[i] {
                    self.activo[i] = true;
                    self.activos.push(i);
                }
            }
        }
        // (mano) ritmo dividido por cuántos rasgos tiene la acción elegida, como la propia
        let div = if c.actor_por_activos { (self.rasgos.de(self.a).len().max(1)) as f64 } else { 1.0 };
        let sp = c.decae_solo_positivos;
        let encoge_de = |w: f64| if sp && w <= 0.0 { 1.0 } else { 1.0 - c.decae_actor };
        for &i in &self.activos {
            self.w[i] += Self::ritmo(c.aprende_actor, self.exp_w[i], c) / div * delta * hw[i];
            if c.decae_actor > 0.0 {
                match c.decae_actor_modo {
                    0 => self.w[i] *= encoge_de(self.w[i]),
                    2 => if !(sp && self.w[i] <= 0.0) { self.w[i] *= 1.0 - c.decae_actor * hw[i].abs().min(1.0) },
                    _ => {}
                }
            }
        }
        if c.decae_actor > 0.0 && c.decae_actor_modo == 1 {
            // olvidar por uso: solo los rasgos presentes en alguna acción de este tick, una vez cada uno
            let mut vistos = [0u16; NACC * MAXR];
            let mut nv = 0usize;
            for b in 0..self.rasgos.na as usize {
                for &i in self.rasgos.de(b) {
                    if !vistos[..nv].contains(&i) {
                        vistos[nv] = i;
                        nv += 1;
                        let k = i as usize;
                        self.w[k] *= encoge_de(self.w[k]);
                    }
                }
            }
        }
        if c.decae_actor > 0.0 && c.decae_actor_modo == 3 {
            // olvidar por hacer: solo los rasgos de la acción elegida, una vez cada uno
            let mut vistos = [0u16; MAXR];
            let mut nv = 0usize;
            for &i in self.rasgos.de(self.a) {
                if !vistos[..nv].contains(&i) {
                    vistos[nv] = i;
                    nv += 1;
                    let k = i as usize;
                    self.w[k] *= encoge_de(self.w[k]);
                }
            }
        }
        if let Some(p) = &mut self.propia {
            p.aprender(delta, &self.probs, self.a, self.rasgos.na as usize, cuerpo, c);
        }
    }

    pub fn repasar(&mut self, m: &Momento, activas: Option<&ActivasMomento>, fraccion: f64, c: &Config) -> f64 {
        let na = m.rasgos.na as usize;
        let mut h = [0.0f64; NACC];
        for b in 0..na {
            h[b] = psum(m.rasgos.de(b).iter().map(|&i| self.w[i as usize]));
        }
        // (etapa 14) las unidades propias de ese momento
        let propio = match (&self.propia, activas) {
            (Some(p), Some((act, act2))) => {
                p.sumar(act, &mut h, na);
                let (vp, vp2) = (p.valor(act), p.valor(act2));
                Some((act, vp, vp2))
            }
            _ => None,
        };
        let mut mx = h[0];
        for b in 1..na {
            if m.rasgos.puede(b) && h[b] > mx {
                mx = h[b];
            }
        }
        let mut e = [0.0f64; NACC];
        for b in 0..na {
            if m.rasgos.puede(b) {
                e[b] = crate::mate::exp((h[b] - mx) / m.temp[b]);
            }
        }
        let z = psum(e[..na].iter().copied());
        let rho = if c.importancia { c.tope_importancia.min((e[m.a] / z) / m.p) } else { 1.0 };
        let tabla = !(self.propia.is_some() && c.critico_propio == 2);
        let vs = if tabla { *self.v.get(&m.s).unwrap_or(&0.0) } else { 0.0 };
        let vs2 = if tabla { *self.v.get(&m.s2).unwrap_or(&0.0) } else { 0.0 };
        let delta = match &propio {
            None => m.r + c.serotonina * vs2 - vs,
            Some((_, vp, vp2)) if c.critico_propio == 1 => { let _ = (vp, vp2); m.r + c.serotonina * vs2 - vs }
            Some((_, vp, vp2)) => m.r + c.serotonina * (vs2 + vp2) - (vs + vp),
        };
        if tabla {
            let ritmo = Self::ritmo(c.aprende_critico, *self.exp_v.get(&m.s).unwrap_or(&0), c);
            self.v.insert(m.s, vs + fraccion * rho * ritmo * delta);
        }
        if let (Some((act, _, _)), Some(p)) = (&propio, &mut self.propia) {
            let mut probs = [0.0f64; NACC];
            for b in 0..na {
                probs[b] = e[b] / z;
            }
            p.repasar(act, fraccion * rho * delta, &probs, m.a, na, c);
        }
        if !c.repaso_actor {
            return delta;
        }
        // grad: dict en orden de inserción (cada índice se actualiza por su cuenta)
        let mut gi: [u16; NACC * MAXR] = [0; NACC * MAXR];
        let mut gv: [f64; NACC * MAXR] = [0.0; NACC * MAXR];
        let mut ng = 0usize;
        let mut sumar = |i: u16, x: f64, resta: bool| {
            let k = match gi[..ng].iter().position(|&j| j == i) {
                Some(k) => k,
                None => {
                    gi[ng] = i;
                    gv[ng] = 0.0;
                    ng += 1;
                    ng - 1
                }
            };
            if resta { gv[k] = gv[k] - x } else { gv[k] = gv[k] + x }
        };
        for &i in m.rasgos.de(m.a) {
            sumar(i, 1.0, false);
        }
        for b in 0..na {
            let q = e[b] / z;
            for &i in m.rasgos.de(b) {
                sumar(i, q, true);
            }
        }
        for k in 0..ng {
            let g = gv[k];
            if g != 0.0 {
                let i = gi[k] as usize;
                let div = if c.actor_por_activos { (m.rasgos.de(m.a).len().max(1)) as f64 } else { 1.0 };
                self.w[i] += fraccion * rho * Self::ritmo(c.aprende_actor, self.exp_w[i], c) / div * delta * g;
            }
        }
        delta
    }

    pub fn olvidar_paso(&mut self) {
        if let Some(p) = &mut self.propia {
            p.olvidar_paso();
        }
        self.s = None;
        self.huella_v.clear();
        for x in self.huella_w.iter_mut() {
            *x = 0.0;
        }
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

// ------------------------------------------------------------------ hipocampo
pub struct Hipocampo {
    dia: std::collections::VecDeque<Momento>,
    /// (etapa 14, con REPASO_PROPIO) lo que sintió en cada momento del día, en paralelo a `dia`
    sentires: std::collections::VecDeque<(crate::propia::Sentir, crate::propia::Sentir)>,
}

impl Hipocampo {
    pub fn new() -> Hipocampo {
        Hipocampo { dia: Default::default(), sentires: Default::default() }
    }
    pub fn anotar(&mut self, m: Momento, sentir: Option<(crate::propia::Sentir, crate::propia::Sentir)>,
                  c: &Config) {
        self.dia.push_back(m);
        if let Some(s) = sentir {
            self.sentires.push_back(s);
        }
        if self.dia.len() as i64 > c.capacidad_dia {
            self.dia.pop_front();
            if self.sentires.len() as i64 > c.capacidad_dia {
                self.sentires.pop_front();
            }
        }
    }
    /// usa el MISMO azar que los ganglios (en Python comparten el objeto).
    pub fn dormir(&mut self, g: &mut Ganglios, c: &Config) {
        for l in g.lugares.iter_mut() {
            l.decaer(c.poda_lugares);
        }
        for m in &self.dia {
            if m.r > c.umbral_lugar {
                let l = &mut g.lugares[m.necesidad];
                let f = c.tope_lugar.min(l.get(m.pos).unwrap_or(0.0) + 1.0);
                l.poner(m.pos, f);
            }
        }
        if !self.dia.is_empty() {
            let dia: Vec<Momento> = self.dia.iter().copied().collect();
            let pesos: Vec<f64> = dia.iter().map(|m| c.prioridad_base + m.r.abs()).collect();
            let k = c.repasos as usize * dia.len();
            let elegidos = g.rng.choices_pesos(&pesos, k);
            let parejo = self.sentires.len() == dia.len();
            let cada = c.repaso_propio_cada.max(1) as usize;
            // las unidades activas de cada recuerdo se buscan una sola vez por noche: mientras
            // duerme no nace ninguna, así que salen iguales cada vez que vuelve al mismo recuerdo
            let mut cache: Vec<Option<ActivasMomento>> = vec![None; dia.len()];
            for (j, i) in elegidos.into_iter().enumerate() {
                let act = match (parejo && j % cada == 0, &g.propia) {
                    (true, Some(p)) => {
                        let (s, s2) = &self.sentires[i];
                        Some(&*cache[i].get_or_insert_with(|| (p.activas_de(s), p.activas_de(s2))))
                    }
                    _ => None,
                };
                g.repasar(&dia[i], act, c.aprende_repaso, c);
            }
        }
        self.dia.clear();
        self.sentires.clear();
    }
}

crate::pieza!(Cuerpo { v, n });
crate::pieza!(Quimica { adenosina, cortisol, dopamina, noradrenalina, previo });
crate::pieza!(Momento { s, rasgos, a, r, s2, pos, necesidad, p, temp });
crate::pieza!(Lugares { v });
crate::pieza!(Ganglios { rng, v, exp_v, w, exp_w, lugares, huella_v, huella_w, activo, activos, prediccion,
                        s, a, rasgos, probs, temp, necesidad, dopamina, ultimo, propia, ultimo_sentir });
crate::pieza!(Hipocampo { dia, sentires });

/// Rasgos: solo los índices usados de cada acción (el resto es relleno).
impl crate::guardar::Pieza for Rasgos {
    // (u8/u16 también son Pieza)
    fn guardar(&self, e: &mut crate::guardar::Escritor) {
        self.na.guardar(e);
        self.posibles.guardar(e);
        for a in 0..self.na as usize {
            self.n[a].guardar(e);
            for &i in self.de(a) {
                i.guardar(e);
            }
        }
    }
    fn cargar(l: &mut crate::guardar::Lector) -> Self {
        let mut r = Rasgos::default();
        r.na = u8::cargar(l);
        r.posibles = u16::cargar(l);
        for a in 0..r.na as usize {
            r.n[a] = u8::cargar(l);
            for k in 0..r.n[a] as usize {
                r.idx[a][k] = u16::cargar(l);
            }
        }
        r
    }
}
