//! Maestro y Oido de angri/maestro.py (Habla no está portada: PIDE apagado).
use crate::azar::Azar;
use crate::config::Config;
use crate::mente::{alivia, psum};
use crate::mundo::*;
use std::collections::HashSet;

const COSAS: [u8; 3] = [COMIDA, DESCANSO, NOVEDAD];
const N_PALABRAS: usize = 8;

pub struct Maestro {
    rng: Azar,
    palabra_dir: [usize; 4],  // norte, sur, oeste, este
    palabra_cosa: [usize; 3], // comida, cama, novedad
    pub objetivo: u8,         // 255 = None
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
        }
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

    pub fn hablar(&mut self, mundo: &Mundo, x: i64, y: i64, necesidad: usize, vista: &Vista,
                  edad: i64, c: &Config) -> Option<[usize; 2]> {
        self.objetivo = 255;
        let u_esp = self.rng.random();
        let u_habla = self.rng.random();
        let u_cosa = self.rng.randrange(0, 3) as usize;
        if c.maestro_solo_si_pide && !(edad < c.maestro_inicio || u_esp < c.prob_espontaneo) {
            return None;
        }
        let objetivo = if c.maestro_ciego { COSAS[u_cosa] } else { alivia(necesidad) };
        let ic = match COSAS.iter().position(|&k| k == objetivo) {
            Some(i) => i,
            None => return None,
        };
        if u_habla >= c.prob_habla {
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
        self.objetivo = objetivo;
        Some([self.palabra_cosa[ic], self.palabra_dir[d]])
    }
}

struct Pendiente {
    pos: (i64, i64),
    objetivo: u8,
    frase: [usize; 2],
    quedan: i64,
    ap: [f64; 4],
    nueva: bool,
}

pub struct Oido {
    sentido: [[f64; 4]; N_PALABRAS],
    veces: [i64; N_PALABRAS],
    pub confianza: f64,
    pendiente: Option<Pendiente>,
    frases: HashSet<[usize; 2]>,
    pub aciertos: Vec<(bool, bool)>,
    sorpresa: f64,
    sorpresa_media: f64,
    pasos: i64,
}

fn argmax4(a: &[f64; 4]) -> usize {
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
            sentido: [[0.25; 4]; N_PALABRAS], veces: [0; N_PALABRAS], confianza: c.confianza_inicial,
            pendiente: None, frases: HashSet::new(), aciertos: vec![], sorpresa: 0.0,
            sorpresa_media: 0.0, pasos: 0,
        }
    }

    fn apuesta(&self, frase: &[usize; 2]) -> [f64; 4] {
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
            let ap = self.apuesta(&f);
            let nueva = !self.frases.contains(&f);
            self.pendiente = Some(Pendiente { pos, objetivo, frase: f, quedan: c.espera_verificar, ap, nueva });
            self.frases.insert(f);
            self.pasos = 0;
        }
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
