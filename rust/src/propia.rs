//! Etapa 14: una mente que arma sus propios rasgos.
//!
//! En vez de la lista hecha a mano (`rasgos_de`: "me acerco a X", "lo que alivia la
//! necesidad del foco está para allá"), la mente ve **átomos** crudos de dos clases:
//!
//! - **del estado**: franjas del cuerpo, el foco, noche, lluvia, qué hay acá, qué lleva,
//!   qué toca. Tienen un peso por acción.
//! - **de cada movimiento** (egocéntricos, como tener un frente para cada paso posible):
//!   qué hay en la celda a la que lleva, qué ve o huele en esa dirección y a qué
//!   distancia, qué altura tiene, si está tapada o hay fuego, si es hacia donde le
//!   dijeron o hacia donde recuerda algo. Tienen un solo peso, el mismo para los cinco
//!   movimientos (lo que aprende de "comida adelante" vale para cualquier lado).
//!
//! Con eso arma **unidades**: cada átomo solo y un banco de conjunciones de 2 o 3 átomos
//! (pueden mezclar las dos clases: "con hambre" y "comida adelante") que nacen de lo que
//! está viviendo, prefiriendo los átomos raros. Solo se tocan las activas.
//!
//! Plasticidad (generate-and-test; Dohare et al. 2024): cada tanto, las unidades
//! maduras que menos sirven (poco peso, casi nunca activas, o activas casi siempre: no
//! distinguen nada) se reinician con átomos de lo que acaba de pasar. Las de un átomo
//! nunca se reinician.
use crate::azar::Azar;
use crate::config::Config;
use crate::mente::{MOVIMIENTOS, NACC};
use crate::mundo::*;

/// átomos del estado: 0..N_ESTADO; de un movimiento: N_ESTADO..N_ATOMOS
pub const N_ESTADO: usize = 68;
/// (44 de siempre + 49 de DISTANCIA_PROPIA='fina': 42 de "me acerco" y 7 de "me alejo")
pub const N_MOVER: usize = 93;
pub const N_ATOMOS: usize = N_ESTADO + N_MOVER;
const SIN_ATOMO: u16 = u16::MAX;
/// marca de una unidad del estado en la lista de activas (las de movimiento llevan
/// en esos bits para qué movimientos está activa)
const DEL_ESTADO: u8 = 0x80;

#[derive(Clone, Copy)]
pub struct Unidad {
    pub atomos: [u16; 3],
    pub k: u8,      // cuántos átomos (0 = vacía, todavía no nació)
    pub clave: u16, // su átomo más raro al nacer: la unidad se busca solo cuando está ese
    pub mover: bool, // tiene algún átomo de movimiento: un solo peso (w[0]) para los 5
    pub v: f64,
    pub w: [f64; NACC],
    pub util: f64,  // promedio de cuánto pesa cuando está activa
    pub veces: i64, // cuántas veces estuvo activa
    pub nace: i64,  // paso en que nació
    pub q: [f64; NPRED], // (PREDICE_PROPIO) lo que predice de los cambios del cuerpo
}

/// lo que predicen las unidades: suba y baja de cada necesidad. Las primeras 3 (energía, integridad,
/// calor) siempre; (PREDICE_TODAS, paso 5) 1 suma el agua, 2 suma todas. Las que no se usan quedan en 0
/// y los bucles van solo hasta `np`, así con PREDICE_TODAS = 0 la cuenta es la misma de antes.
pub const NPRED: usize = 14;
const PREDICE: [usize; 7] = [crate::mente::ENERGIA, crate::mente::INTEGRIDAD, crate::mente::CALOR, crate::mente::AGUA_N, crate::mente::CANSANCIO, crate::mente::CURIOSIDAD, crate::mente::ESTRES];
fn npred(c: &Config) -> usize {
    match c.predice_todas { 0 => 3, 1 => 4, _ => 7 }
}
const GAMMA_PRED: f64 = 0.9;
/// (paso 6) casillero de `q` donde las unidades de movimiento guardan la salida a la que llevan
const SL: usize = NPRED - 1;
#[inline]
fn lejos_activo(c: &Config) -> bool {
    c.salida_lejos > 0 && c.salida_propia > 0 && c.predice_actor > 0.0 && c.predice_todas < 2
}

impl Unidad {
    fn vacia() -> Unidad {
        Unidad { atomos: [SIN_ATOMO; 3], k: 0, clave: SIN_ATOMO, mover: false, v: 0.0, w: [0.0; NACC], util: 0.0, veces: 0, nace: 0, q: [0.0; NPRED] }
    }
}

/// Lo justo para saber si una unidad está activa, aparte y compacto (8 bytes): así la
/// búsqueda de activas no lee la unidad entera. No se guarda: sale de `unidades`.
#[derive(Clone, Copy)]
struct Forma {
    atomos: [u16; 3],
    k: u8,
    mover: bool,
}

impl Forma {
    fn de(u: &Unidad) -> Forma {
        Forma { atomos: u.atomos, k: u.k, mover: u.mover }
    }
}

/// Lo que siente en un momento, en átomos.
#[derive(Clone, Default, PartialEq)]
pub struct Sentir {
    pub estado: Vec<u16>,     // ordenados
    pub mover: [Vec<u16>; 5], // por movimiento (índices >= N_ESTADO), ordenados
}

pub struct Propia {
    rng: Azar,
    pub unidades: Vec<Unidad>, // las primeras N_ATOMOS son los átomos solos
    indice: Vec<Vec<u32>>,     // átomo -> unidades del banco cuyo primer átomo es ese
    forma: Vec<Forma>,         // copia compacta de la forma de cada unidad
    // lo que sintió en s2 y sus activas: si al elegir siente lo mismo (y no nació nadie), se reusan
    previa: Option<(Sentir, Vec<(u32, u8)>)>,
    // huellas densas por unidad; `huella` lista las que no están en cero
    ev: Vec<f64>,
    ew: Vec<[f64; NACC]>,
    marca: Vec<bool>,
    huella: Vec<u32>,
    cuenta: Vec<i64>, // cuántas veces estuvo cada átomo (su frecuencia = cuenta / t)
    ahora: Sentir,     // lo de s (al elegir)
    activas: Vec<(u32, u8)>,
    elegida: usize,
    pub vs: f64, // V propio de s
    pub salida: u32, // (paso 4) franja de "hay salida a la vista" del último sentir mirado (no se guarda: se recalcula)
    pub t: i64,
    pub nacidas: i64,
    pub reiniciadas: i64,
    // (NACER_SORPRESA) el momento más sorpresivo desde el último reinicio: de ahí nacen las nuevas
    molde: Sentir,
    molde_a: usize,
    molde_d: f64,
    // (PREDICE_PROPIO) las activas de s2 y el cuerpo del paso anterior
    activas2: Vec<(u32, u8)>,
    cuerpo_antes: [f64; 7],
    /// (DUDA_PROPIA) último paso en que cada unidad de movimiento estuvo activa para el movimiento hecho
    probada: Vec<i64>,
    /// (VALE_LA_PENA) qué pasos hacia una espina valen la pena por lo de detrás (no se guarda: se recalcula al elegir)
    pub vale: [bool; 5],
    pub costo: [f64; 5],
}

#[inline]
fn franja4(x: f64) -> usize {
    ((x * 4.0) as usize).min(3)
}

#[inline]
fn dir_de(dx: i64, dy: i64) -> usize {
    if dy.abs() >= dx.abs() { if dy < 0 { 0 } else { 1 } } else if dx < 0 { 2 } else { 3 }
}

/// Los átomos de este momento (sin orden de importancia, sin "hacia dónde ir").
pub fn sentir(vista: &Vista, cuerpo: &[f64; 7], ncuerpo: usize, n: usize, dr: usize, dicha: usize,
              tacto: Option<&Tacto>, clima: u8, fina: bool) -> Sentir {
    let mut s = Sentir::default();
    let e = &mut s.estado;
    e.push(0); // siempre (solo para el crítico)
    for k in 0..ncuerpo {
        e.push((1 + k * 4 + franja4(cuerpo[k])) as u16);
    }
    e.push((29 + n) as u16);
    if clima & 1 != 0 { e.push(36); }
    if clima & 2 != 0 { e.push(37); }
    if let Some(t) = tacto {
        if let Some(o) = t.lleva {
            e.push(38);
            for (p, &si) in o.props().iter().enumerate() {
                if si { e.push((39 + p) as u16); }
            }
        }
        if let Some(o) = t.aqui {
            for (p, &si) in o.props().iter().enumerate() {
                if si { e.push((45 + p) as u16); }
            }
        }
        e.push((51 + t.h_aqui.min(2)) as u16);
        e.push((54 + t.paredes.min(4)) as u16);
        if t.cubierto { e.push(59); }
        if t.dura { e.push(60); }
    }
    let m = N_ESTADO;
    for &((dx, dy), cod) in &vista.v {
        let cod = cod.min(6) as usize;
        if (dx, dy) == (0, 0) {
            s.estado.push((61 + cod) as u16);
            s.mover[4].push((m + cod) as u16);
            continue;
        }
        if let Some(a) = MOVIMIENTOS[..4].iter().position(|&x| x == (dx, dy)) {
            s.mover[a].push((m + cod) as u16);
        }
        let d = dx.abs() + dy.abs();
        if fina {
            // (paso 4f) a cada movimiento que la acerca, "me acerco a esto a distancia d"; al que la aleja, "me alejo"
            for (a, &(mx, my)) in MOVIMIENTOS[..4].iter().enumerate() {
                let d2 = (dx - mx).abs() + (dy - my).abs();
                if d2 < d {
                    s.mover[a].push((m + 44 + cod * 6 + (d as usize - 1).min(5)) as u16);
                } else {
                    s.mover[a].push((m + 86 + cod) as u16);
                }
            }
        } else {
            let dist = match d { 1 => 0, 2 => 1, 3 | 4 => 2, _ => 3 };
            s.mover[dir_de(dx, dy)].push((m + 7 + cod * 4 + dist) as u16);
        }
    }
    for a in 0..5 {
        if let Some(t) = tacto {
            s.mover[a].push((m + 35 + t.h_dest[a].clamp(0, 3) as usize) as u16);
            if t.techo_dest[a] { s.mover[a].push((m + 39) as u16); }
            if t.fuego_dest[a] { s.mover[a].push((m + 40) as u16); }
            if a < 4 && t.lleva.is_some() { s.mover[a].push((m + 41) as u16); }
        }
        if dicha == a { s.mover[a].push((m + 42) as u16); }
        if dr == a { s.mover[a].push((m + 43) as u16); }
    }
    s.estado.sort_unstable();
    s.estado.dedup();
    for r in s.mover.iter_mut() {
        r.sort_unstable();
        r.dedup();
    }
    s
}

impl Propia {
    pub fn new(semilla: u64, c: &Config) -> Propia {
        let mut unidades = Vec::with_capacity(N_ATOMOS + c.propios as usize);
        for a in 0..N_ATOMOS {
            unidades.push(Unidad { atomos: [a as u16, SIN_ATOMO, SIN_ATOMO], k: 1, clave: a as u16, mover: a >= N_ESTADO,
                                   ..Unidad::vacia() });
        }
        for _ in 0..c.propios {
            unidades.push(Unidad::vacia());
        }
        let n = unidades.len();
        let forma = unidades.iter().map(Forma::de).collect();
        Propia {
            rng: Azar::new(semilla ^ 0x14_0b5e55), unidades, indice: vec![vec![]; N_ATOMOS], forma, previa: None,
            ev: vec![0.0; n], ew: vec![[0.0; NACC]; n], marca: vec![false; n], huella: vec![],
            cuenta: vec![0; N_ATOMOS], ahora: Sentir::default(), activas: vec![], elegida: 4, vs: 0.0, salida: 0,
            t: 0, nacidas: 0, reiniciadas: 0, molde: Sentir::default(), molde_a: 4, molde_d: 0.0,
            activas2: vec![], cuerpo_antes: [f64::NAN; 7], probada: vec![0; n], vale: [false; 5], costo: [0.0; 5],
        }
    }

    /// las unidades activas: (unidad, DEL_ESTADO o los movimientos para los que está activa)
    pub fn activas_de(&self, s: &Sentir) -> Vec<(u32, u8)> {
        let mut presente = [false; N_ESTADO];
        for &a in &s.estado {
            presente[a as usize] = true;
        }
        let mut mascara = [0u8; N_MOVER];
        for (a, r) in s.mover.iter().enumerate() {
            for &x in r {
                mascara[x as usize - N_ESTADO] |= 1 << a;
            }
        }
        let mut act: Vec<(u32, u8)> = Vec::with_capacity(256);
        for &a in &s.estado {
            act.push((a as u32, DEL_ESTADO));
        }
        for (i, &m) in mascara.iter().enumerate() {
            if m != 0 {
                act.push(((N_ESTADO + i) as u32, m));
            }
        }
        let probar = |a: usize, act: &mut Vec<(u32, u8)>| {
            for &u in &self.indice[a] {
                let un = &self.forma[u as usize];
                let mut m: u8 = 0x1f;
                let mut ok = true;
                for &b in &un.atomos[..un.k as usize] {
                    let b = b as usize;
                    if b < N_ESTADO {
                        ok &= presente[b];
                    } else {
                        m &= mascara[b - N_ESTADO];
                    }
                }
                if ok {
                    if !un.mover {
                        act.push((u, DEL_ESTADO));
                    } else if m != 0 {
                        act.push((u, m));
                    }
                }
            }
        };
        for &a in &s.estado {
            probar(a as usize, &mut act);
        }
        for (i, &m) in mascara.iter().enumerate() {
            if m != 0 {
                probar(N_ESTADO + i, &mut act);
            }
        }
        act
    }

    pub fn valor(&self, act: &[(u32, u8)]) -> f64 {
        act.iter().filter(|x| x.1 == DEL_ESTADO).map(|&(u, _)| self.unidades[u as usize].v).sum()
    }

    /// al elegir: guarda s y suma a cada acción lo que dicen sus unidades
    pub fn preferencias(&mut self, s: Sentir, h: &mut [f64; NACC], na: usize, cuerpo: &[f64; 7], c: &Config) {
        let act = match self.previa.take() {
            Some((ps, pa)) if ps == s => pa,
            _ => self.activas_de(&s),
        };
        if c.duda_propia > 0.0 {
            // lo que no se prueba se pone en duda: el peso de cada unidad de movimiento pesa según su confianza
            for &(u, m) in &act {
                let un = &self.unidades[u as usize];
                if m == DEL_ESTADO {
                    if u != 0 {
                        for b in 0..na {
                            h[b] += un.w[b];
                        }
                    }
                } else {
                    let f = self.confianza(u as usize, c);
                    for a in 0..5 {
                        if m >> a & 1 == 1 {
                            h[a] += f * un.w[0];
                        }
                    }
                }
            }
        } else {
            self.sumar(&act, h, na);
        }
        if c.predice_actor > 0.0 {
            // cuánto vale cada suba/baja predicha: lo que acercaría esa necesidad a su equilibrio
            let mut gusto = [0.0; NPRED];
            for (j, &k) in PREDICE[..npred(c)].iter().enumerate() {
                let falta = c.peso[k] * (c.equilibrio[k] - cuerpo[k]);
                gusto[2 * j] = falta;
                gusto[2 * j + 1] = -falta;
            }
            let mut xa = [0.0f64; 5];
            for &(u, m) in &act {
                if m != DEL_ESTADO {
                    let un = &self.unidades[u as usize];
                    let x: f64 = (0..2 * npred(c)).map(|i| gusto[i] * un.q[i]).sum::<f64>() * c.predice_actor * self.confianza(u as usize, c);
                    for a in 0..5 {
                        if m >> a & 1 == 1 {
                            if c.tope_actor > 0.0 || c.tope_costo > 0.0 || c.temp_costo {
                                xa[a] += x;
                            } else {
                                h[a] += x;
                            }
                        }
                    }
                }
            }
            // (TOPE_ACTOR) lo que el alivio predicho suma o resta a cada movimiento no pasa de ±TOPE_ACTOR
            if c.tope_actor > 0.0 {
                for a in 0..5 {
                    h[a] += xa[a].clamp(-c.tope_actor, c.tope_actor);
                }
            } else if c.tope_costo > 0.0 {
                // (TOPE_COSTO) solo el costo predicho tiene tope; el alivio entra entero
                for a in 0..5 {
                    h[a] += xa[a].max(-c.tope_costo);
                }
            } else if c.temp_costo {
                // (TEMP_COSTO) entra entero, pero se anota el costo para que la elección lo entibie con el malestar
                for a in 0..5 {
                    h[a] += xa[a];
                    self.costo[a] = xa[a].min(0.0);
                }
            }
        }
        if lejos_activo(c) {
            // (paso 6) a qué franja de salida predice que lleva cada movimiento
            let p = Self::salida_mov(&self.unidades, &act);
            for a in 0..5 {
                h[a] += c.salida_lejos_peso * p[a];
            }
        }
        self.salida = if c.salida_propia > 0 { self.salida_de(&act, cuerpo, c) } else { 0 };
        self.vs = self.valor(&act);
        self.ahora = s;
        self.activas = act;
    }

    /// (VALE_LA_PENA) mirar un paso más allá de cada espina vecina: lo que las unidades ya aprendidas dirían del
    /// mismo movimiento si estuviera parado en la espina (la vista corrida una celda, sin tacto: no sabe alturas
    /// de allá). Solo suma (el miedo ya lo pesa el paso normal) y marca en `vale` los pasos donde sumó.
    /// Se llama después de `preferencias`; no toca el azar ni nada que se guarde.
    pub fn mirar_detras(&mut self, vista: &Vista, cuerpo: &[f64; 7], ncuerpo: usize, n: usize, clima: u8,
                        h: &mut [f64; NACC], c: &Config) {
        self.vale = [false; 5];
        let mut gusto = [0.0; NPRED];
        if c.predice_actor > 0.0 {
            for (j, &k) in PREDICE[..npred(c)].iter().enumerate() {
                let falta = c.peso[k] * (c.equilibrio[k] - cuerpo[k]);
                gusto[2 * j] = falta;
                gusto[2 * j + 1] = -falta;
            }
        }
        for a in 0..4 {
            let (mx, my) = MOVIMIENTOS[a];
            if vista.get((mx, my)) != Some(PELIGRO) {
                continue;
            }
            let v2 = Vista { v: vista.v.iter().map(|&((x, y), k)| ((x - mx, y - my), k)).collect() };
            let mut s2 = sentir(&v2, cuerpo, ncuerpo, n, 99, 99, None, clima, c.distancia_fina);
            // el estado es el de ahora (sigue con hambre); de los movimientos, solo el que sigue derecho
            s2.estado = self.ahora.estado.clone();
            for b in 0..5 {
                if b != a {
                    s2.mover[b].clear();
                }
            }
            let mut x = 0.0;
            for (u, m) in self.activas_de(&s2) {
                if m == DEL_ESTADO || m >> a & 1 == 0 {
                    continue;
                }
                let un = &self.unidades[u as usize];
                let f = self.confianza(u as usize, c);
                x += f * un.w[0];
                if c.predice_actor > 0.0 {
                    x += (0..2 * npred(c)).map(|i| gusto[i] * un.q[i]).sum::<f64>() * c.predice_actor * f;
                }
            }
            if x > 0.0 {
                h[a] += c.vale_la_pena * x;
                self.vale[a] = true;
            }
        }
    }

    /// el sentir de s (para que el hipocampo lo guarde)
    pub fn ahora(&self) -> &Sentir {
        &self.ahora
    }

    /// (DUDA_PROPIA) cuánto cree todavía lo que aprendió la unidad u: baja con el tiempo sin probarla
    pub fn confianza(&self, u: usize, c: &Config) -> f64 {
        if c.duda_propia <= 0.0 {
            return 1.0;
        }
        c.duda_propia / (c.duda_propia + (self.t - self.probada[u]).max(0) as f64)
    }

    /// suma a cada acción lo que dicen estas unidades activas
    pub fn sumar(&self, act: &[(u32, u8)], h: &mut [f64; NACC], na: usize) {
        for &(u, m) in act {
            let un = &self.unidades[u as usize];
            if m == DEL_ESTADO {
                if u != 0 {
                    for b in 0..na {
                        h[b] += un.w[b];
                    }
                }
            } else {
                for a in 0..5 {
                    if m >> a & 1 == 1 {
                        h[a] += un.w[0];
                    }
                }
            }
        }
    }

    /// repaso al dormir: un paso sin huellas (`d` ya trae el error TD, la fracción y el peso)
    pub fn repasar(&mut self, act: &[(u32, u8)], d: f64, probs: &[f64; NACC], a: usize, na: usize, c: &Config) {
        if act.is_empty() {
            return;
        }
        let n = act.len() as f64;
        let ac = if c.critico_propio == 1 { 0.0 } else { c.aprende_propio_critico / n * d };
        let aa = c.aprende_propio_actor / n * d;
        for &(u, m) in act {
            let un = &mut self.unidades[u as usize];
            if m == DEL_ESTADO {
                un.v += ac;
                if u != 0 {
                    for b in 0..na {
                        un.w[b] += aa * ((b == a) as u8 as f64 - probs[b]);
                    }
                }
            } else {
                let mut g = if a < 5 && m >> a & 1 == 1 { 1.0 } else { 0.0 };
                for b in 0..5 {
                    if m >> b & 1 == 1 {
                        g -= probs[b];
                    }
                }
                un.w[0] += aa * g;
            }
        }
    }

    /// V propio de s2 (para el error TD)
    /// (aprender las recetas, paso 4) "hay salida a la vista": el mejor alivio que predicen las unidades
    /// de movimiento entre los 5 movimientos de ahora, como fracción de lo lejos que está el cuerpo de su
    /// equilibrio, en SALIDA_PROPIA franjas (0 = ninguna salida). Nada dice qué es comida: es lo que predicen.
    pub fn salida_de(&self, act: &[(u32, u8)], cuerpo: &[f64; 7], c: &Config) -> u32 {
        let mut gusto = [0.0; NPRED];
        let mut lejos = 0.0;
        for (j, &k) in PREDICE[..npred(c)].iter().enumerate() {
            let d = c.equilibrio[k] - cuerpo[k];
            let falta = c.peso[k] * d;
            lejos += falta * d;
            gusto[2 * j] = falta;
            gusto[2 * j + 1] = -falta;
        }
        let mut al = [0.0f64; 5];
        for &(u, m) in act {
            if m != DEL_ESTADO {
                let un = &self.unidades[u as usize];
                let x: f64 = (0..2 * npred(c)).map(|i| gusto[i] * un.q[i]).sum::<f64>();
                for a in 0..5 {
                    if m >> a & 1 == 1 {
                        al[a] += x;
                    }
                }
            }
        }
        let mejor = al.iter().cloned().fold(f64::MIN, f64::max);
        if lejos < 1e-4 || mejor <= 0.0 {
            return 0;
        }
        let r = mejor / lejos;
        if c.salida_debug {
            eprintln!("salida {r:.5} {mejor:.6} {lejos:.6}");
        }
        let g = c.salida_propia as u32;
        let mut f = 0;
        let mut u = c.salida_umbral;
        while f + 1 < g && r >= u {
            f += 1;
            u *= 3.0;
        }
        f
    }

    /// (paso 6, SALIDA_LEJOS) la salida predicha para cada movimiento: suma del último casillero de `q`
    /// de las unidades de movimiento activas para él (ese casillero no lo usa PREDICE_TODAS < 2)
    fn salida_mov(unidades: &[Unidad], act: &[(u32, u8)]) -> [f64; 5] {
        let mut p = [0.0; 5];
        for &(u, m) in act {
            if m != DEL_ESTADO {
                let q = unidades[u as usize].q[SL];
                for a in 0..5 {
                    if m >> a & 1 == 1 {
                        p[a] += q;
                    }
                }
            }
        }
        p
    }

    /// (paso 6) aprende con TD a qué franja de salida lleva el movimiento elegido
    fn aprender_lejos(&mut self, c: &Config) {
        let a = self.elegida;
        let g = (c.salida_propia - 1).max(1) as f64;
        let mut y = self.salida as f64 / g; // la franja de s2 (la dejó valor_de)
        if c.salida_lejos == 2 {
            let p2 = Self::salida_mov(&self.unidades, &self.activas2);
            let mejor = p2.iter().cloned().fold(f64::MIN, f64::max);
            y = (1.0 - GAMMA_PRED) * y + GAMMA_PRED * mejor;
        }
        let pa = Self::salida_mov(&self.unidades, &self.activas)[a];
        let na = self.activas.iter().filter(|&&(_, m)| m != DEL_ESTADO && m >> a & 1 == 1).count();
        if na == 0 {
            return;
        }
        let e = c.aprende_propio_critico / na as f64 * (y - pa);
        for k in 0..self.activas.len() {
            let (u, m) = self.activas[k];
            if m != DEL_ESTADO && m >> a & 1 == 1 {
                self.unidades[u as usize].q[SL] += e;
            }
        }
    }

    pub fn valor_de(&mut self, s: &Sentir, cuerpo: &[f64; 7], c: &Config) -> f64 {
        let act = self.activas_de(s);
        let v = self.valor(&act);
        self.salida = if c.salida_propia > 0 { self.salida_de(&act, cuerpo, c) } else { 0 };
        if c.predice_propio > 0.0 || c.predice_actor > 0.0 {
            self.activas2 = act.clone();
        }
        self.previa = Some((s.clone(), act));
        v
    }

    fn predicho(&self, act: &[(u32, u8)], np: usize) -> [f64; NPRED] {
        let mut q = [0.0; NPRED];
        for &(u, m) in act {
            if m == DEL_ESTADO {
                let un = &self.unidades[u as usize];
                for i in 0..np {
                    q[i] += un.q[i];
                }
            }
        }
        q
    }

    /// las unidades aprenden a predecir los cambios del cuerpo (TD con sus propias huellas de estado)
    fn predecir(&mut self, cuerpo: &[f64; 7], n: f64, c: &Config) {
        let np = 2 * npred(c);
        let antes = self.cuerpo_antes;
        self.cuerpo_antes = *cuerpo;
        if antes[0].is_nan() {
            return;
        }
        let mut cum = [0.0; NPRED];
        for (j, &k) in PREDICE[..npred(c)].iter().enumerate() {
            let d = (cuerpo[k] - antes[k]) * 10.0;
            cum[2 * j] = d.max(0.0).min(1.0);
            cum[2 * j + 1] = (-d).max(0.0).min(1.0);
        }
        let q1 = self.predicho(&self.activas, np);
        let q2 = self.predicho(&self.activas2, np);
        let mut d = [0.0; NPRED];
        for i in 0..np {
            d[i] = c.aprende_propio_critico / n * (cum[i] + GAMMA_PRED * q2[i] - q1[i]);
        }
        for &u in &self.huella {
            let u = u as usize;
            let e = self.ev[u];
            if e != 0.0 {
                let un = &mut self.unidades[u];
                for i in 0..np {
                    un.q[i] += d[i] * e;
                }
            }
        }
        if lejos_activo(c) && self.elegida < 5 {
            self.aprender_lejos(c);
        }
        if c.predice_actor > 0.0 && self.elegida < 5 {
            // (PREDICE_ACTOR) las de movimiento del elegido aprenden lo que ese movimiento agrega
            let a = self.elegida;
            let mut qa = q1;
            let mut na = 0.0;
            for &(u, m) in &self.activas {
                if m != DEL_ESTADO && m >> a & 1 == 1 {
                    na += 1.0;
                    for i in 0..np {
                        qa[i] += self.unidades[u as usize].q[i];
                    }
                }
            }
            if na > 0.0 {
                let r = c.aprende_propio_critico / na;
                let mut e = [0.0; NPRED];
                for i in 0..np {
                    e[i] = r * (cum[i] + GAMMA_PRED * q2[i] - qa[i]);
                }
                for k in 0..self.activas.len() {
                    let (u, m) = self.activas[k];
                    if m != DEL_ESTADO && m >> a & 1 == 1 {
                        let un = &mut self.unidades[u as usize];
                        for i in 0..np {
                            un.q[i] += e[i];
                        }
                    }
                }
            }
        }
    }

    /// aprende de un paso: `delta` es el error TD de toda la mente; `probs` las de s
    pub fn aprender(&mut self, delta: f64, probs: &[f64; NACC], a: usize, na: usize, cuerpo: &[f64; 7], c: &Config) {
        if self.activas.is_empty() {
            return;
        }
        self.t += 1;
        self.elegida = a;
        if a < 5 && c.duda_propia > 0.0 {
            let t = self.t;
            for &(u, m) in &self.activas {
                if m != DEL_ESTADO && m >> a & 1 == 1 {
                    self.probada[u as usize] = t;
                }
            }
        }
        for &x in &self.ahora.estado {
            self.cuenta[x as usize] += 1;
        }
        if a < 5 {
            for &x in &self.ahora.mover[a] {
                self.cuenta[x as usize] += 1;
            }
        }
        if c.nacer_sorpresa && delta.abs() > self.molde_d {
            self.molde = self.ahora.clone();
            self.molde_a = a;
            self.molde_d = delta.abs();
        }
        let decae = c.serotonina * c.huella;
        let n = self.activas.len() as f64;
        // las huellas se desvanecen; las que quedan en casi cero se sueltan
        let (ev, ew, marca, forma) = (&mut self.ev, &mut self.ew, &mut self.marca, &self.forma);
        self.huella.retain(|&u| {
            let u = u as usize;
            ev[u] *= decae;
            let mut m = ev[u];
            // las de movimiento solo usan ew[0] (el resto queda siempre en cero)
            let usa = if forma[u].mover { 1 } else { NACC };
            for x in ew[u][..usa].iter_mut() {
                *x *= decae;
                m = m.max(x.abs());
            }
            if m < 0.01 {
                ev[u] = 0.0;
                ew[u] = [0.0; NACC];
                marca[u] = false;
                false
            } else {
                true
            }
        });
        for &(u, m) in &self.activas {
            let u = u as usize;
            let e = &mut self.ew[u];
            if m == DEL_ESTADO {
                self.ev[u] = 1.0;
                for b in 0..na {
                    e[b] -= probs[b];
                }
                e[a] += 1.0;
            } else {
                // d log π(a) / d w: [a está entre los suyos] − Σ π de los suyos
                let mut g = if a < 5 && m >> a & 1 == 1 { 1.0 } else { 0.0 };
                for b in 0..5 {
                    if m >> b & 1 == 1 {
                        g -= probs[b];
                    }
                }
                e[0] += g;
            }
            if !self.marca[u] {
                self.marca[u] = true;
                self.huella.push(u as u32);
            }
        }
        let ac = if c.critico_propio == 1 { 0.0 } else { c.aprende_propio_critico / n * delta };
        let aa = c.aprende_propio_actor / n * delta;
        let t = self.t;
        for &u in &self.huella {
            let u = u as usize;
            let un = &mut self.unidades[u];
            let e = &self.ew[u];
            // ritmo por unidad: las selectivas (raras) aprenden más rápido por vez, hasta RITMO_RARO veces
            let r = if c.ritmo_raro > 1.0 {
                let frec = (un.veces as f64 / (t - un.nace).max(1) as f64).max(1e-9);
                (c.tope_activa / frec).sqrt().clamp(1.0, c.ritmo_raro)
            } else {
                1.0
            };
            if un.mover {
                un.w[0] += r * aa * e[0];
            } else {
                un.v += r * ac * self.ev[u];
                if c.tope_v_propio > 0.0 {
                    un.v = un.v.clamp(-c.tope_v_propio, c.tope_v_propio);
                }
                if u != 0 {
                    for b in 0..na {
                        un.w[b] += r * aa * e[b];
                    }
                }
            }
        }
        if c.ancla_v > 0.0 {
            // la tabla y las unidades persiguen el mismo δ: sin esto el reparto entre las dos deriva sin freno
            let encoge = 1.0 - c.ancla_v;
            for &(u, m) in &self.activas {
                if m == DEL_ESTADO {
                    self.unidades[u as usize].v *= encoge;
                }
            }
        }
        if c.decae_propio > 0.0 {
            // olvidar por uso: solo las unidades activas este tick
            let encoge = 1.0 - c.decae_propio;
            for &(u, _) in &self.activas {
                let un = &mut self.unidades[u as usize];
                if un.mover {
                    un.w[0] *= encoge;
                } else {
                    for b in 0..na {
                        un.w[b] *= encoge;
                    }
                }
            }
        }
        if c.predice_propio > 0.0 || c.predice_actor > 0.0 {
            self.predecir(cuerpo, n, c);
        }
        // utilidad: cuánto pesan cuando están activas
        for &(u, _) in &self.activas {
            let un = &mut self.unidades[u as usize];
            let mut peso = if un.mover {
                un.w[0].abs()
            } else {
                un.v.abs() + un.w[..na].iter().map(|x| x.abs()).sum::<f64>() / na as f64
            };
            if c.predice_propio > 0.0 {
                peso += c.predice_propio * un.q.iter().map(|x| x.abs()).sum::<f64>();
            }
            un.util += 0.02 * (peso - un.util);
            un.veces += 1;
        }
        self.crecer(c);
    }

    /// nacen unidades nuevas (mientras haya lugar) y se reinician las que menos sirven
    fn crecer(&mut self, c: &Config) {
        if self.t < 500 {
            return;
        }
        if self.nacidas < c.propios {
            for _ in 0..c.nuevos_por_tick {
                if self.nacidas >= c.propios {
                    break;
                }
                let u = N_ATOMOS + self.nacidas as usize;
                self.nacer(u, c);
                self.nacidas += 1;
            }
            return;
        }
        if self.t % c.reemplazo_cada != 0 {
            return;
        }
        let t = self.t;
        let mut maduras: Vec<(f64, usize)> = (N_ATOMOS..self.unidades.len())
            .filter(|&u| t - self.unidades[u].nace >= c.madurez_propio)
            .map(|u| {
                let un = &self.unidades[u];
                let frec = un.veces as f64 / (t - un.nace).max(1) as f64;
                // una unidad casi siempre activa no distingue nada: es la primera en irse
                (if un.k == 0 || frec > c.tope_activa { -1.0 } else { un.util * frec }, u)
            })
            .collect();
        let k = (c.reemplazo_fraccion * c.propios as f64).ceil() as usize;
        if maduras.len() <= k {
            return;
        }
        maduras.select_nth_unstable_by(k, |a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut peores: Vec<usize> = maduras[..k].iter().map(|x| x.1).collect();
        peores.sort_unstable();
        let sorpresa = c.nacer_sorpresa && self.molde_d > 0.0;
        if sorpresa {
            std::mem::swap(&mut self.ahora, &mut self.molde);
            std::mem::swap(&mut self.elegida, &mut self.molde_a);
        }
        for u in peores {
            self.nacer(u, c);
            self.reiniciadas += 1;
        }
        if sorpresa {
            std::mem::swap(&mut self.ahora, &mut self.molde);
            std::mem::swap(&mut self.elegida, &mut self.molde_a);
            self.molde_d = 0.0;
        }
    }

    /// la unidad `u` (re)nace: 2 o 3 átomos de lo que acaba de pasar (el estado y el
    /// movimiento que hizo), prefiriendo los raros, hasta que sea selectiva
    fn nacer(&mut self, u: usize, c: &Config) {
        self.previa = None;
        let viejo = self.unidades[u];
        if viejo.k > 0 {
            let l = &mut self.indice[viejo.clave as usize];
            if let Some(i) = l.iter().position(|&x| x as usize == u) {
                l.swap_remove(i);
            }
        }
        self.ev[u] = 0.0;
        self.ew[u] = [0.0; NACC];
        if self.marca[u] {
            self.marca[u] = false;
            self.huella.retain(|&x| x as usize != u);
        }
        self.unidades[u] = Unidad { nace: self.t, ..Unidad::vacia() };
        self.forma[u] = Forma::de(&self.unidades[u]);
        let t = self.t.max(1) as f64;
        let mut pool: Vec<(u16, f64)> = self.ahora.estado.iter().copied().filter(|&a| a != 0)
            .map(|a| (a, self.cuenta[a as usize] as f64 / t)).collect();
        if self.elegida < 5 {
            pool.extend(self.ahora.mover[self.elegida].iter().map(|&a| (a, self.cuenta[a as usize] as f64 / t)));
        }
        if pool.len() < 2 {
            return;
        }
        let mut mejor: (f64, Vec<u16>) = (f64::INFINITY, vec![]);
        for _ in 0..8 {
            let mut quedan = pool.clone();
            let mut elegidos: Vec<u16> = vec![];
            let mut frec = 1.0;
            while elegidos.len() < 3 && !quedan.is_empty() {
                let pesos: Vec<f64> = quedan.iter().map(|x| (1.0 - x.1.min(1.0)).powi(2) + 1e-3).collect();
                let i = self.rng.choices_pesos(&pesos, 1)[0];
                let (a, f) = quedan.swap_remove(i);
                elegidos.push(a);
                frec *= f;
                if elegidos.len() >= 2 && frec <= c.tope_activa / 4.0 {
                    break;
                }
            }
            if elegidos.len() >= 2 && frec < mejor.0 {
                mejor = (frec, elegidos);
            }
            if mejor.0 <= c.tope_activa / 4.0 {
                break;
            }
        }
        let mut elegidos = mejor.1;
        let frec_de = |a: u16| pool.iter().find(|x| x.0 == a).map_or(1.0, |x| x.1);
        let clave = *elegidos.iter().min_by(|&&a, &&b| frec_de(a).total_cmp(&frec_de(b)).then(a.cmp(&b))).unwrap_or(&SIN_ATOMO);
        if elegidos.len() < 2 {
            return;
        }
        let k = elegidos.len();
        elegidos.sort_unstable();
        let mut atomos = [SIN_ATOMO; 3];
        atomos[..k].copy_from_slice(&elegidos);
        let mover = elegidos.iter().any(|&a| a as usize >= N_ESTADO);
        self.unidades[u] = Unidad { atomos, k: k as u8, clave, mover, nace: self.t, ..Unidad::vacia() };
        self.forma[u] = Forma::de(&self.unidades[u]);
        self.indice[clave as usize].push(u as u32);
    }

    pub fn olvidar_paso(&mut self) {
        for &u in &self.huella {
            let u = u as usize;
            self.ev[u] = 0.0;
            self.ew[u] = [0.0; NACC];
            self.marca[u] = false;
        }
        self.huella.clear();
        self.activas.clear();
        self.previa = None;
        self.activas2.clear();
        self.cuerpo_antes = [f64::NAN; 7];
    }

    /// cuántas unidades del banco viven y cuántas mezclan estado y movimiento
    /// (diagnóstico, solo mira) las activas de s: (unidad, DEL_ESTADO o los movimientos)
    pub fn activas_ahora(&self) -> &[(u32, u8)] {
        &self.activas
    }

    /// (diagnóstico, solo mira) true si la marca es de una unidad del estado
    pub fn es_del_estado(m: u8) -> bool {
        m == DEL_ESTADO
    }

    pub fn resumen(&self) -> (i64, i64) {
        let banco = &self.unidades[N_ATOMOS..];
        let vivas = banco.iter().filter(|u| u.k > 0).count() as i64;
        let mezclan = banco.iter()
            .filter(|u| u.mover && u.atomos[..u.k as usize].iter().any(|&a| (a as usize) < N_ESTADO))
            .count() as i64;
        (vivas, mezclan)
    }
}

crate::pieza!(Unidad { atomos, k, clave, mover, v, w, util, veces, nace, q });
crate::pieza!(Sentir { estado, mover });
/// lo que se guarda de Propia (todo menos `forma`, que se rearma de `unidades` al cargar)
struct Guardada {
    rng: Azar, unidades: Vec<Unidad>, indice: Vec<Vec<u32>>, ev: Vec<f64>, ew: Vec<[f64; NACC]>, marca: Vec<bool>,
    huella: Vec<u32>, cuenta: Vec<i64>, ahora: Sentir, activas: Vec<(u32, u8)>, elegida: usize, vs: f64, t: i64,
    nacidas: i64, reiniciadas: i64, molde: Sentir, molde_a: usize, molde_d: f64, activas2: Vec<(u32, u8)>,
    cuerpo_antes: [f64; 7],
}

crate::pieza!(Guardada { rng, unidades, indice, ev, ew, marca, huella, cuenta, ahora, activas, elegida, vs, t,
                         nacidas, reiniciadas, molde, molde_a, molde_d, activas2, cuerpo_antes });

impl crate::guardar::Pieza for Propia {
    fn guardar(&self, e: &mut crate::guardar::Escritor) {
        let p = self;
        // mismos campos y en el mismo orden que antes: los archivos viejos siguen sirviendo
        macro_rules! g { ($($f:ident),*) => { $( e.campo(stringify!($f)); crate::guardar::Pieza::guardar(&p.$f, e); )* } }
        g!(rng, unidades, indice, ev, ew, marca, huella, cuenta, ahora, activas, elegida, vs, t,
           nacidas, reiniciadas, molde, molde_a, molde_d, activas2, cuerpo_antes);
        e.campo("probada");
        crate::guardar::Pieza::guardar(&p.probada, e);
    }
    fn cargar(l: &mut crate::guardar::Lector) -> Self {
        let g = Guardada::cargar(l);
        let n = g.unidades.len();
        let probada = if l.campo_opcional("probada") { crate::guardar::Pieza::cargar(l) } else { vec![g.t; n] };
        let forma = g.unidades.iter().map(Forma::de).collect();
        Propia {
            rng: g.rng, unidades: g.unidades, indice: g.indice, forma, previa: None, ev: g.ev, ew: g.ew, marca: g.marca,
            huella: g.huella, cuenta: g.cuenta, ahora: g.ahora, activas: g.activas, elegida: g.elegida, vs: g.vs, salida: 0,
            t: g.t, nacidas: g.nacidas, reiniciadas: g.reiniciadas, molde: g.molde, molde_a: g.molde_a,
            molde_d: g.molde_d, activas2: g.activas2, cuerpo_antes: g.cuerpo_antes, probada, vale: [false; 5], costo: [0.0; 5],
        }
    }
}
