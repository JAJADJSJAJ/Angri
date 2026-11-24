//! Maestro, Habla y Oido de angri/maestro.py.
use crate::azar::Azar;
use crate::config::Config;
use crate::mente::{alivia, psum};
use crate::mundo::*;
use std::collections::{HashMap, HashSet};

pub const COSAS: [u8; 3] = [COMIDA, DESCANSO, NOVEDAD];
pub const N_PALABRAS: usize = 8;

fn cosa_de(objetivo: u8) -> Option<usize> {
    COSAS.iter().position(|&k| k == objetivo)
}

pub struct Maestro {
    pub rng: Azar,
    pub palabra_dir: [usize; 4],  // norte, sur, oeste, este
    pub palabra_cosa: [usize; 3], // comida, cama, novedad
    pub objetivo: u8,             // 255 = None
    pub dichas: i64,
}

impl Maestro {
    pub fn new(semilla: u64) -> Maestro {
        let mut rng = Azar::new(semilla.wrapping_add(7));
        let mut orden: Vec<usize> = (0..N_PALABRAS).collect();
        rng.shuffle(&mut orden);
        Maestro {
            rng,
            palabra_dir: [orden[0], orden[1], orden[2], orden[3]],
            palabra_cosa: [orden[4], orden[5], orden[6]],
            objetivo: 255,
            dichas: 0,
        }
    }

    /// Prueba del ego: las palabras de dirección pasan a significar otra cosa.
    pub fn cambiar_significados(&mut self) {
        let p = self.palabra_dir;
        self.palabra_dir = [p[1], p[2], p[3], p[0]];
    }

    fn cerca(mundo: &Mundo, x: i64, y: i64, objetivo: u8) -> Option<(i64, i64)> {
        let (w, h) = mundo.ancho_alto();
        let g = mundo.celdas();
        let (mut mejor, mut dist) = (None, i64::MAX);
        for yy in 0..h {
            for xx in 0..w {
                if g[(yy * w + xx) as usize] == objetivo {
                    let d = (xx - x).abs() + (yy - y).abs();
                    if d < dist {
                        mejor = Some((xx, yy));
                        dist = d;
                    }
                }
            }
        }
        mejor
    }

    /// pedido: la palabra que dijo Angri (índice), o None.
    pub fn hablar(&mut self, mundo: &Mundo, x: i64, y: i64, necesidad: usize, vista: &Vista,
                  edad: i64, pedido: Option<usize>, c: &Config) -> Option<[usize; 2]> {
        self.objetivo = 255;
        let u_esp = self.rng.random();
        let u_habla = self.rng.random();
        let u_cosa = self.rng.randrange(0, 3) as usize;
        let cosa = pedido.and_then(|p| self.palabra_cosa.iter().position(|&q| q == p));
        let objetivo;
        if let Some(i) = cosa {
            objetivo = COSAS[i];
        } else if c.maestro_solo_si_pide && !(edad < c.maestro_inicio || u_esp < c.prob_espontaneo) {
            return None;
        } else {
            objetivo = if c.maestro_ciego { COSAS[u_cosa] } else { alivia(necesidad) };
            if cosa_de(objetivo).is_none() || u_habla >= c.prob_habla {
                return None;
            }
        }
        if c.composicion != 0 && objetivo != (if edad < c.composicion { COMIDA } else { DESCANSO }) {
            return None;
        }
        if vista.alguna(objetivo) {
            return None;
        }
        let (lx, ly) = Self::cerca(mundo, x, y, objetivo)?;
        let (dx, dy) = (lx - x, ly - y);
        let d = if dy.abs() >= dx.abs() {
            if dy < 0 { 0 } else { 1 }
        } else if dx < 0 { 2 } else { 3 };
        let mut frase = [self.palabra_cosa[cosa_de(objetivo).unwrap()], self.palabra_dir[d]];
        self.objetivo = objetivo;
        self.dichas += 1;
        if c.maestro_miente {
            for p in frase.iter_mut() {
                *p = self.rng.randrange(0, N_PALABRAS as i64) as usize;
            }
        }
        Some(frase)
    }
}

// ------------------------------------------------------------------ Habla (etapa 7-9)
pub struct Habla {
    pub rng: Azar,
    pub opciones: [Option<usize>; 4], // 3 palabras de cosas + callarse
    pub w: [[f64; 4]; 5],
    pub huella: Option<(usize, usize, i64)>,
    pub assoc: [[f64; 3]; 5],
    pub oidas: i64,
    pub edad: i64,
    pub ultima: Option<([bool; 3], i64)>,
    pub pedidos: Vec<(bool, i64)>,
}

impl Habla {
    pub fn new(semilla: u64, m: &Maestro) -> Habla {
        Habla {
            rng: Azar::new(semilla.wrapping_add(11)),
            opciones: [Some(m.palabra_cosa[0]), Some(m.palabra_cosa[1]), Some(m.palabra_cosa[2]), None],
            w: [[0.0; 4]; 5], huella: None, assoc: [[0.0; 3]; 5], oidas: 0, edad: 0, ultima: None,
            pedidos: vec![],
        }
    }

    /// Lo que siente: el foco (uno) o el estado del cuerpo. Lista (necesidad, peso) en orden.
    fn sentir(cuerpo: &[f64; 7], foco: usize, c: &Config) -> Vec<(usize, f64)> {
        if !c.escucha_cuerpo {
            return if foco < 5 { vec![(foco, 1.0)] } else { vec![] };
        }
        let x: Vec<f64> = (0..5).map(|k| c.peso[k] * (cuerpo[k] - c.equilibrio[k]).abs()).collect();
        let mut z = psum(x.iter().copied());
        if z == 0.0 {
            z = 1.0;
        }
        (0..5).map(|k| (k, x[k] / z)).collect()
    }

    pub fn escuchar(&mut self, frase: Option<[usize; 2]>, cuerpo: &[f64; 7], foco: usize,
                    pedido: Option<usize>, c: &Config) {
        if let Some(f) = frase {
            let oyo = [0, 1, 2].map(|i| f.contains(&self.opciones[i].unwrap()));
            self.ultima = Some((oyo, self.edad));
        }
        let f = match frase { Some(f) => f, None => return };
        if !c.escucha || !c.escucha_oido || (pedido.is_some() && !c.escucha_pedidos) {
            return;
        }
        let x = Self::sentir(cuerpo, foco, c);
        let oyo = [0, 1, 2].map(|i| if f.contains(&self.opciones[i].unwrap()) { 1.0 } else { 0.0 });
        for i in 0..3 {
            let err = oyo[i] - psum(x.iter().map(|&(n, v)| v * self.assoc[n][i]));
            for &(n, v) in &x {
                self.assoc[n][i] += c.aprende_escucha * v * err;
            }
        }
        self.oidas += 1;
    }

    pub fn aliviado(&mut self, necesidad: usize, edad: i64, c: &Config) {
        if !(c.escucha && c.escucha_alivio) {
            return;
        }
        let (oyo, cuando) = match self.ultima { None => return, Some(u) => u };
        if edad - cuando > c.ventana_alivio {
            return;
        }
        let a = &mut self.assoc[necesidad];
        for i in 0..3 {
            a[i] += c.aprende_escucha * ((if oyo[i] { 1.0 } else { 0.0 }) - a[i]);
        }
        self.ultima = None;
    }

    /// Devuelve el índice de la palabra que dice, o None.
    pub fn decidir(&mut self, necesidad: usize, vista: &Vista, edad: i64, cuerpo: &[f64; 7],
                   c: &Config) -> Option<usize> {
        self.edad = edad;
        let objetivo = alivia(necesidad);
        if objetivo == 255 || self.huella.is_some() || vista.alguna(objetivo)
            || self.rng.random() >= c.prob_habla
        {
            return None;
        }
        let i;
        if c.pide_azar {
            i = self.rng.randrange(0, 4) as usize;
        } else if c.escucha {
            let x = Self::sentir(cuerpo, necesidad, c);
            let a: Vec<f64> = (0..3).map(|j| psum(x.iter().map(|&(n, v)| v * self.assoc[n][j]))).collect();
            let mut m = a[0];
            for &v in &a[1..] {
                if v > m {
                    m = v;
                }
            }
            if m < c.umbral_escucha {
                return None;
            }
            let ps: Vec<f64> = a.iter().map(|&v| ((v - m) / c.temperatura_escucha).exp()).collect();
            let mut r = self.rng.random() * psum(ps.iter().copied());
            let mut k = 0;
            while k < 2 && r > ps[k] {
                r -= ps[k];
                k += 1;
            }
            i = k;
        } else {
            let w = self.w[necesidad];
            let mut m = w[0];
            for &v in &w[1..] {
                if v > m {
                    m = v;
                }
            }
            let ps: Vec<f64> = w.iter().map(|&x| ((x - m) / c.temperatura_habla).exp()).collect();
            let mut r = self.rng.random() * psum(ps.iter().copied());
            let mut k = 0;
            while k < ps.len() - 1 && r > ps[k] {
                r -= ps[k];
                k += 1;
            }
            i = k;
        }
        self.huella = Some((necesidad, i, c.ventana_habla));
        let p = self.opciones[i];
        if p.is_some() {
            self.pedidos.push((Some(i) == cosa_de(objetivo), edad));
        }
        p
    }

    pub fn aprender(&mut self, dopamina: f64, c: &Config) {
        let (n, i, quedan) = match self.huella { None => return, Some(h) => h };
        if !c.pide_azar && !c.escucha {
            self.w[n][i] += c.aprende_habla * dopamina;
        }
        self.huella = if quedan <= 1 { None } else { Some((n, i, quedan - 1)) };
    }
}

// ------------------------------------------------------------------ Oido
pub struct Pendiente {
    pub pos: (i64, i64),
    pub objetivo: u8,
    pub frase: [usize; 2],
    pub quedan: i64,
    pub ap: [f64; 4],
    pub nueva: bool,
}

pub struct Oido {
    pub eco: [i64; N_PALABRAS], // ticks que le quedan a cada palabra (0 = no suena)
    pub sentido: [[f64; 4]; N_PALABRAS],
    pub veces: [i64; N_PALABRAS],
    pub confianza: f64,
    pub pendiente: Option<Pendiente>,
    pub frases: HashSet<[usize; 2]>,
    pub aciertos: Vec<(bool, bool)>,
    pub sorpresa: f64,
    pub indice_cambio: Option<usize>,
    pub sorpresa_media: f64,
    pub indice_composicion: Option<usize>,
    pub examen: Vec<bool>,
    pub holistico: HashMap<[usize; 2], [f64; 4]>,
    pub pasos: i64,
}

pub fn argmax4(a: &[f64; 4]) -> usize {
    let mut m = 0;
    for i in 1..4 {
        if a[i] > a[m] {
            m = i;
        }
    }
    m
}

impl Oido {
    pub fn new(c: &Config) -> Oido {
        Oido {
            eco: [0; N_PALABRAS], sentido: [[0.25; 4]; N_PALABRAS], veces: [0; N_PALABRAS],
            confianza: c.confianza_inicial, pendiente: None, frases: HashSet::new(), aciertos: vec![],
            sorpresa: 0.0, indice_cambio: None, sorpresa_media: 0.0, indice_composicion: None,
            examen: vec![], holistico: HashMap::new(), pasos: 0,
        }
    }

    pub fn apuesta(&self, frase: &[usize; 2], c: &Config) -> [f64; 4] {
        if c.holistico {
            return *self.holistico.get(frase).unwrap_or(&[0.25; 4]);
        }
        let mut prob = [1.0f64; 4];
        for &p in frase {
            for i in 0..4 {
                prob[i] = prob[i] * self.sentido[p][i];
            }
        }
        let mut z = psum(prob.iter().copied());
        if z == 0.0 {
            z = 1.0;
        }
        [prob[0] / z, prob[1] / z, prob[2] / z, prob[3] / z]
    }

    /// Examen sin aprender: las 4 frases "cama + dirección".
    pub fn probar_composicion(&mut self, m: &Maestro, c: &Config) {
        let cama = m.palabra_cosa[1];
        self.examen = (0..4).map(|i| argmax4(&self.apuesta(&[cama, m.palabra_dir[i]], c)) == i).collect();
    }

    pub fn direccion_dicha(&self, c: &Config) -> usize {
        let p = match &self.pendiente {
            Some(p) if self.confianza >= c.confianza_minima && self.pasos <= c.pasos_dicha => p,
            _ => return 4,
        };
        let d = argmax4(&p.ap);
        if p.ap[d] >= c.certeza_dicha { d } else { 4 }
    }

    pub fn oir(&mut self, frase: Option<[usize; 2]>, pos: (i64, i64), objetivo: u8, c: &Config) {
        if let Some(f) = frase {
            for &p in &f {
                self.eco[p] = c.memoria_palabra;
            }
            let ap = self.apuesta(&f, c);
            let nueva = !self.frases.contains(&f);
            self.pendiente = Some(Pendiente { pos, objetivo, frase: f, quedan: c.espera_verificar, ap, nueva });
            self.frases.insert(f);
            self.pasos = 0;
        }
    }

    pub fn pasar(&mut self) {
        for e in self.eco.iter_mut() {
            if *e > 0 {
                *e -= 1;
            }
        }
    }

    /// tuple(sorted(eco)): las palabras que todavía suenan
    pub fn palabras(&self) -> Vec<usize> {
        (0..N_PALABRAS).filter(|&p| self.eco[p] > 0).collect()
    }

    pub fn caminar(&mut self, d: usize) {
        if let Some(p) = &self.pendiente {
            if d == argmax4(&p.ap) {
                self.pasos += 1;
            }
        }
    }

    pub fn ver(&mut self, vista: &Vista, pos: (i64, i64), c: &Config) {
        let p = match &mut self.pendiente { None => return, Some(p) => p };
        let (ox, oy) = p.pos;
        let mut mejor: Option<(i64, i64, i64)> = None;
        for &((dx, dy), cel) in &vista.v {
            if cel == p.objetivo {
                let t = ((pos.0 + dx - ox).abs() + (pos.1 + dy - oy).abs(), pos.0 + dx, pos.1 + dy);
                if mejor.map_or(true, |m| t < m) {
                    mejor = Some(t);
                }
            }
        }
        let (_, x, y) = match mejor {
            None => {
                if p.quedan <= 1 {
                    self.pendiente = None;
                } else {
                    p.quedan -= 1;
                }
                return;
            }
            Some(m) => m,
        };
        let (dx, dy) = (x - ox, y - oy);
        if dx == 0 && dy == 0 {
            self.pendiente = None;
            return;
        }
        let d = if dy.abs() >= dx.abs() {
            if dy < 0 { 0 } else { 1 }
        } else if dx < 0 { 2 } else { 3 };
        let ap = p.ap;
        let frase = p.frase;
        let nueva = p.nueva;
        self.sorpresa = 1.0 - ap[d];
        self.sorpresa_media += 0.05 * (self.sorpresa - self.sorpresa_media);
        self.aciertos.push((argmax4(&ap) == d, nueva));
        let util = (ap[d] - 0.25) / 0.75;
        self.confianza += c.aprende_confianza * (0.0f64.max(util) - self.confianza);
        if c.holistico {
            let s = self.holistico.entry(frase).or_insert([0.25; 4]);
            for i in 0..4 {
                s[i] = (s[i] + c.aprende_palabra * ((if i == d { 1.0 } else { 0.0 }) - ap[i])).max(0.02);
            }
            let z = psum(s.iter().copied());
            for i in 0..4 {
                s[i] = s[i] / z;
            }
            self.pendiente = None;
            return;
        }
        for &p in &frase {
            self.veces[p] += 1;
            let mut ritmo = c.aprende_palabra;
            if c.ego {
                ritmo = ritmo / (1.0 + self.veces[p] as f64 / 20.0);
            }
            ritmo = ritmo.max(c.aprende_palabra * c.piso_palabra);
            ritmo = c.ritmo_maximo.min(ritmo * (1.0 + c.sorpresa_ritmo * self.sorpresa_media));
            let s = &mut self.sentido[p];
            for i in 0..4 {
                s[i] += ritmo * ((if i == d { 1.0 } else { 0.0 }) - ap[i]);
                s[i] = s[i].max(0.02);
            }
            let z = psum(s.iter().copied());
            for i in 0..4 {
                s[i] = s[i] / z;
            }
        }
        self.pendiente = None;
    }
}

crate::pieza!(Maestro { rng, palabra_dir, palabra_cosa, objetivo, dichas });
crate::pieza!(Habla { rng, opciones, w, huella, assoc, oidas, edad, ultima, pedidos });
crate::pieza!(Pendiente { pos, objetivo, frase, quedan, ap, nueva });
crate::pieza!(Oido { eco, sentido, veces, confianza, pendiente, frases, aciertos, sorpresa, indice_cambio,
                    sorpresa_media, indice_composicion, examen, holistico, pasos });
