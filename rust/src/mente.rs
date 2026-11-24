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
        if v[ENERGIA] < 0.3 {
            v[ESTRES] += 0.002;
        }
        if v[INTEGRIDAD] < 0.7 {
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
            v[ESTRES] += 0.002;
        }
        if v[AGUA_N] < 0.05 || v[ENERGIA] < 0.05 {
            v[INTEGRIDAD] -= c.recupera_integridad + c.dano_sed;
        }
        if v[AGUA_N] < 0.3 {
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
const N_RASGOS: usize = 507;

#[derive(Clone, Copy, Default)]
pub struct Rasgos {
    pub idx: [[u16; 8]; 5],
    pub n: [u8; 5],
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
    pub temp: [f64; 5],
}

fn hacia(vista: &Vista, objetivo: u8) -> usize {
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

fn rasgos_de(vista: &Vista, n: usize, dc: usize, d_o: usize, dr: usize, a: usize, dd: usize, out: &mut [u16; 8]) -> u8 {
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
        if a == dd { push(417 + n); }
    }
    k as u8
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
    probs: [f64; 5],
    temp: [f64; 5],
    necesidad: usize,
    pub dopamina: f64,
    pub ultimo: Option<Momento>,
}

impl Ganglios {
    pub fn new(semilla: u64) -> Ganglios {
        Ganglios {
            rng: Azar::new(semilla), v: HashMap::new(), exp_v: HashMap::new(),
            w: vec![0.0; N_RASGOS], exp_w: vec![0; N_RASGOS], lugares: Default::default(),
            huella_v: vec![], huella_w: vec![0.0; N_RASGOS], activo: vec![false; N_RASGOS],
            activos: vec![], prediccion: None, s: None, a: 0, rasgos: Rasgos::default(),
            probs: [0.0; 5], temp: [0.0; 5], necesidad: 0, dopamina: 0.0, ultimo: None,
        }
    }

    fn franja(x: f64, c: &Config) -> u32 {
        ((x * c.franjas as f64) as i64).min(c.franjas - 1) as u32
    }

    fn resumen(&self, vista: &Vista, cuerpo: &[f64; 7], n: usize, c: &Config) -> u32 {
        let celda = vista.get((0, 0)).unwrap_or(7) as u32;
        let come = (hacia(vista, COMIDA) != 4) as u32;
        (((n as u32 * 64 + Self::franja(cuerpo[ENERGIA], c)) * 64 + Self::franja(cuerpo[INTEGRIDAD], c)) * 8 + celda) * 2 + come
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

    pub fn elegir(&mut self, vista: &Vista, cuerpo: &[f64; 7], n: usize, pos: (i64, i64),
                  noradrenalina: f64, malestar: f64, dicha: usize, c: &Config) -> (i64, i64) {
        let dc = hacia(vista, COMIDA);
        let d_o = hacia(vista, alivia(n));
        let dr = self.recuerdo(n, pos);
        let mut rasgos = Rasgos::default();
        for a in 0..5 {
            rasgos.n[a] = rasgos_de(vista, n, dc, d_o, dr, a, dicha, &mut rasgos.idx[a]);
        }
        let mut h = [0.0f64; 5];
        for a in 0..5 {
            h[a] = psum(rasgos.de(a).iter().map(|&i| self.w[i as usize]));
        }
        let t_todo = c.temperatura_base + c.temperatura_sorpresa * noradrenalina + c.temperatura_malestar * malestar;
        let mut m = h[0];
        for &x in &h[1..] {
            if x > m {
                m = x;
            }
        }
        let mut temp = [t_todo; 5];
        if c.margen_explorar != 0.0 || c.peligro_sin_malestar {
            let t0 = c.temperatura_base + c.temperatura_sorpresa * noradrenalina;
            for b in 0..5 {
                if (c.margen_explorar != 0.0 && h[b] < m - c.margen_explorar)
                    || (c.peligro_sin_malestar && vista.get(MOVIMIENTOS[b]) == Some(PELIGRO))
                {
                    temp[b] = t0;
                }
            }
        }
        let mut e = [0.0f64; 5];
        for b in 0..5 {
            e[b] = ((h[b] - m) / temp[b]).exp();
        }
        let z = psum(e.iter().copied());
        let mut probs = [0.0f64; 5];
        for b in 0..5 {
            probs[b] = e[b] / z;
        }
        let a = self.rng.choices_pesos(&probs, 1)[0];
        self.s = Some(self.resumen(vista, cuerpo, n, c));
        self.a = a;
        self.rasgos = rasgos;
        self.probs = probs;
        self.temp = temp;
        self.necesidad = n;
        MOVIMIENTOS[a]
    }

    pub fn aprender(&mut self, recompensa: f64, vista: &Vista, cuerpo: &[f64; 7], n: usize,
                    pos: (i64, i64), c: &Config) {
        let s = match self.s { None => return, Some(s) => s };
        for l in self.lugares.iter_mut() {
            l.decaer(c.recuerdo_decae);
        }
        if recompensa > c.umbral_recuerdo {
            self.lugares[self.necesidad].poner(pos, 1.0);
        }
        let s2 = self.resumen(vista, cuerpo, n, c);
        self.ultimo = Some(Momento { s, rasgos: self.rasgos, a: self.a, r: recompensa, s2, pos,
                                     necesidad: self.necesidad, p: self.probs[self.a], temp: self.temp });
        if c.borrar_lugar_vacio {
            let objetivo = alivia(n);
            if objetivo != 255 && self.lugares[n].get(pos).is_some()
                && vista.get((0, 0)) != Some(objetivo) && hacia(vista, objetivo) == 4
            {
                self.lugares[n].borrar(pos);
            }
        }
        let v = *self.v.get(&s).unwrap_or(&0.0);
        let delta = recompensa + c.serotonina * *self.v.get(&s2).unwrap_or(&0.0) - v;
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
        for &(k, e) in &self.huella_v {
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
        for b in 0..5 {
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
        for &i in &self.activos {
            self.w[i] += Self::ritmo(c.aprende_actor, self.exp_w[i], c) * delta * hw[i];
        }
    }

    pub fn repasar(&mut self, m: &Momento, fraccion: f64, c: &Config) -> f64 {
        let mut h = [0.0f64; 5];
        for b in 0..5 {
            h[b] = psum(m.rasgos.de(b).iter().map(|&i| self.w[i as usize]));
        }
        let mut mx = h[0];
        for &x in &h[1..] {
            if x > mx {
                mx = x;
            }
        }
        let mut e = [0.0f64; 5];
        for b in 0..5 {
            e[b] = ((h[b] - mx) / m.temp[b]).exp();
        }
        let z = psum(e.iter().copied());
        let rho = if c.importancia { c.tope_importancia.min((e[m.a] / z) / m.p) } else { 1.0 };
        let vs = *self.v.get(&m.s).unwrap_or(&0.0);
        let delta = m.r + c.serotonina * *self.v.get(&m.s2).unwrap_or(&0.0) - vs;
        let ritmo = Self::ritmo(c.aprende_critico, *self.exp_v.get(&m.s).unwrap_or(&0), c);
        self.v.insert(m.s, vs + fraccion * rho * ritmo * delta);
        if !c.repaso_actor {
            return delta;
        }
        // grad: dict en orden de inserción (cada índice se actualiza por su cuenta)
        let mut gi: [u16; 40] = [0; 40];
        let mut gv: [f64; 40] = [0.0; 40];
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
        for b in 0..5 {
            let q = e[b] / z;
            for &i in m.rasgos.de(b) {
                sumar(i, q, true);
            }
        }
        for k in 0..ng {
            let g = gv[k];
            if g != 0.0 {
                let i = gi[k] as usize;
                self.w[i] += fraccion * rho * Self::ritmo(c.aprende_actor, self.exp_w[i], c) * delta * g;
            }
        }
        delta
    }

    pub fn olvidar_paso(&mut self) {
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
}

impl Hipocampo {
    pub fn new() -> Hipocampo {
        Hipocampo { dia: Default::default() }
    }
    pub fn anotar(&mut self, m: Momento, c: &Config) {
        self.dia.push_back(m);
        if self.dia.len() as i64 > c.capacidad_dia {
            self.dia.pop_front();
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
            for i in elegidos {
                g.repasar(&dia[i], c.aprende_repaso, c);
            }
        }
        self.dia.clear();
    }
}
