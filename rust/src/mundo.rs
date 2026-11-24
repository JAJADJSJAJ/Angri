//! La grilla (angri/mundo.py) y el bosque (angri/bosque.py), con el mismo orden
//! de sorteos y de recorridos que en Python.
use crate::azar::Azar;
use crate::config::Config;

pub const VACIO: u8 = 0;
pub const COMIDA: u8 = 1;
pub const DESCANSO: u8 = 2;
pub const NOVEDAD: u8 = 3;
pub const PELIGRO: u8 = 4;
pub const ROCA: u8 = 5; // "#": roca o borde
pub const AGUA: u8 = 6;

// zonas
const Z_BOSQUE: u8 = b'b';
const Z_CLARO: u8 = b'c';
const Z_RIO: u8 = b'r';
const Z_ROCA: u8 = b'k';
pub const Z_CUEVA: u8 = b'q';
const Z_LLANURA: u8 = b'l';

/// Un dict de Python {(dx, dy): celda} chico, con su orden de inserción.
#[derive(Clone, Default)]
pub struct Vista {
    pub v: Vec<((i64, i64), u8)>,
}

impl Vista {
    pub fn get(&self, k: (i64, i64)) -> Option<u8> {
        self.v.iter().find(|(p, _)| *p == k).map(|(_, c)| *c)
    }
    pub fn tiene(&self, k: (i64, i64)) -> bool {
        self.v.iter().any(|(p, _)| *p == k)
    }
    pub fn poner(&mut self, k: (i64, i64), c: u8) {
        if let Some(e) = self.v.iter_mut().find(|(p, _)| *p == k) {
            e.1 = c;
        } else {
            self.v.push((k, c));
        }
    }
    pub fn alguna(&self, c: u8) -> bool {
        self.v.iter().any(|(_, x)| *x == c)
    }
}

// ------------------------------------------------------------------ grilla
pub struct Grilla {
    pub rng: Azar,
    pub fuentes: Vec<(i64, i64)>,
    pub tiempo: i64,
    pub cambios: u64, // cuántas veces cambió `cambio` (medir.py mira la identidad)
    pub cambio: Option<((i64, i64), (i64, i64))>,
    pub g: Vec<u8>,
    w: i64,
    h: i64,
}

impl Grilla {
    pub fn new(semilla: u64, c: &Config) -> Grilla {
        let (w, h) = (c.ancho, c.alto);
        let mut m = Grilla { rng: Azar::new(semilla), fuentes: vec![], tiempo: 0, cambios: 0,
                             cambio: None, g: vec![VACIO; (w * h) as usize], w, h };
        for y in 1..4 {
            for x in 1..5 {
                m.g[(y * w + x) as usize] = DESCANSO;
            }
        }
        let mut f = vec![];
        for _ in 0..c.fuentes_comida {
            let x = m.rng.randrange(0, w);
            let y = m.rng.randrange(5, h);
            f.push((x, y));
        }
        m.fuentes = f;
        for _ in 0..c.comida_inicial { m.poner(COMIDA, c); }
        for _ in 0..c.novedades_iniciales { m.poner(NOVEDAD, c); }
        for _ in 0..c.peligros { m.poner(PELIGRO, c); }
        m
    }

    #[inline]
    pub fn en(&self, x: i64, y: i64) -> u8 {
        self.g[(y * self.w + x) as usize]
    }

    fn poner(&mut self, cosa: u8, c: &Config) {
        for _ in 0..100 {
            let (x, y);
            if cosa == COMIDA && !self.fuentes.is_empty() {
                let (fx, fy) = self.rng.choice(&self.fuentes);
                x = fx + self.rng.randint(-c.radio_fuente, c.radio_fuente);
                y = fy + self.rng.randint(-c.radio_fuente, c.radio_fuente);
                if !(0 <= x && x < self.w && 0 <= y && y < self.h) {
                    continue;
                }
            } else {
                x = self.rng.randrange(0, self.w);
                y = self.rng.randrange(0, self.h);
            }
            if self.en(x, y) == VACIO {
                self.g[(y * self.w + x) as usize] = cosa;
                return;
            }
        }
    }

    pub fn consumir(&mut self, x: i64, y: i64) {
        let i = (y * self.w + x) as usize;
        if self.g[i] == COMIDA || self.g[i] == NOVEDAD {
            self.g[i] = VACIO;
        }
    }

    fn contar(&self, cosa: u8) -> i64 {
        self.g.iter().filter(|&&c| c == cosa).count() as i64
    }

    pub fn avanzar(&mut self, c: &Config) {
        self.tiempo += 1;
        if c.estacion != 0 && self.tiempo % c.estacion == 0 {
            self.cambiar_estacion(c);
        }
        if self.rng.random() < c.prob_nueva_comida && self.contar(COMIDA) < c.comida_maxima {
            self.poner(COMIDA, c);
        }
        if self.rng.random() < c.prob_mueve_peligro {
            self.mover_un_peligro();
        }
        if self.rng.random() < c.prob_nueva_novedad && self.contar(NOVEDAD) < c.novedad_maxima {
            self.poner(NOVEDAD, c);
        }
    }

    fn cambiar_estacion(&mut self, c: &Config) {
        let i = self.rng.indice(self.fuentes.len());
        let vieja = self.fuentes[i];
        let mut nueva = vieja;
        for _ in 0..100 {
            let x = self.rng.randrange(0, self.w);
            let y = self.rng.randrange(5, self.h);
            nueva = (x, y);
            if (nueva.0 - vieja.0).abs() + (nueva.1 - vieja.1).abs() > 8 {
                break;
            }
        }
        self.fuentes[i] = nueva;
        self.cambio = Some((vieja, nueva));
        self.cambios += 1;
        let (vx, vy) = vieja;
        let r = c.radio_fuente;
        for y in (vy - r).max(0)..(vy + r + 1).min(self.h) {
            for x in (vx - r).max(0)..(vx + r + 1).min(self.w) {
                let i = (y * self.w + x) as usize;
                if self.g[i] == COMIDA {
                    self.g[i] = VACIO;
                }
            }
        }
    }

    fn mover_un_peligro(&mut self) {
        let mut peligros = vec![];
        for y in 0..self.h {
            for x in 0..self.w {
                if self.en(x, y) == PELIGRO {
                    peligros.push((x, y));
                }
            }
        }
        if peligros.is_empty() {
            return;
        }
        let (x, y) = self.rng.choice(&peligros);
        let (dx, dy) = self.rng.choice(&[(0i64, -1i64), (0, 1), (-1, 0), (1, 0)]);
        let (nx, ny) = (x + dx, y + dy);
        if 0 <= nx && nx < self.w && 0 <= ny && ny < self.h && self.en(nx, ny) == VACIO {
            self.g[(y * self.w + x) as usize] = VACIO;
            self.g[(ny * self.w + nx) as usize] = PELIGRO;
        }
    }

    pub fn vecindad(&self, x: i64, y: i64, radio: i64) -> Vista {
        let mut v = Vista { v: Vec::with_capacity(25) };
        for dy in -radio..=radio {
            for dx in -radio..=radio {
                let (nx, ny) = (x + dx, y + dy);
                if 0 <= nx && nx < self.w && 0 <= ny && ny < self.h {
                    v.v.push(((dx, dy), self.en(nx, ny)));
                }
            }
        }
        v
    }
}

// ------------------------------------------------------------------ bosque
pub const FRUTA: u8 = 0;
pub const BAYA: u8 = 1;
pub const NUEZ: u8 = 2;
pub const RAIZ: u8 = 3;
// DA y TOPE en el orden del dict de Python: fruta, baya, nuez, raiz
const DA: [&[i64]; 4] = [&[0, 1, 2], &[0, 1], &[2, 3, 0], &[0, 1, 2, 3]];
const TOPE: [i64; 4] = [24, 10, 14, 14];

fn da(tipo: u8, est: i64) -> bool {
    DA[tipo as usize].contains(&est)
}

/// Una cosa suelta del bosque (etapa 13b). El mundo no sabe qué es: solo sus propiedades.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Objeto {
    pub largo: u8,
    pub peso: f64,
    pub dureza: f64,
    pub flota: bool,
    pub arde: bool,
    pub flexible: bool,
    /// 0 = suelta; 1..4 = viga acostada hacia MOVIMIENTOS[eje - 1]
    pub eje: u8,
}

impl Objeto {
    /// lo que la mente puede tocar: largo, pesado, duro, flota, arde, flexible
    pub fn props(&self) -> [bool; 6] {
        [self.largo >= 3, self.peso > 0.4, self.dureza > 0.6, self.flota, self.arde, self.flexible]
    }
}

/// Lo que Angri siente de los objetos (lo arma el mundo para la mente).
#[derive(Clone, Copy, Default)]
pub struct Tacto {
    pub lleva: Option<Objeto>,
    pub aqui: Option<Objeto>,
    pub h_aqui: i64,
    pub h_dest: [i64; 5],
    pub techo_dest: [bool; 5],
    pub fuego_dest: [bool; 5],
    pub paredes: i64,
    pub cubierto: bool,
    pub clima_malo: bool,
    pub dura: bool, // comida dura acá (nuez o raíz): golpear sirve
}

/// Qué pasó al hacer algo con un objeto.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Hecho {
    Nada,
    Agarra,
    Suelta,
    Apila,
    Viga,
    Cae,
    Hunde,
    Ata,
    Rompe,
    Abre(u8),
    Golpea,
    Frota,
    Prende,
}

const DIRS: [(i64, i64); 5] = [(0, -1), (0, 1), (-1, 0), (1, 0), (0, 0)];

pub struct Bosque {
    pub rng: Azar,
    pub w: i64,
    pub h: i64,
    pub tiempo: i64,
    pub lluvia: i64,
    pub crecida: i64,
    /// dict (x, y) -> [tipo, progreso], en orden de inserción
    pub comida: Vec<((i64, i64), u8, i64)>,
    pub predador: (i64, i64),
    pub harto: i64,
    rastro: Vec<i64>, // tick en que pasó Angri, -1 = nada
    pub atacando: bool,
    pub zona: Vec<u8>,
    pub g: Vec<u8>,
    rio_x: Vec<i64>,
    pub cuevas: Vec<(i64, i64)>,
    claros: Vec<(i64, i64)>,
    arboles: [Vec<(i64, i64)>; 4],
    cueva_predador: (i64, i64),
    pub inicio: (i64, i64),
    pub agua_cerca: Vec<(i64, i64)>, // (-1,-1) = no hay
    // --- objetos (etapa 13b); vacíos si OBJETOS está apagado ---
    pub objetos: bool,
    pub orng: Azar, // azar propio: con objetos, el resto del mundo sigue igual
    pub pilas: Vec<Vec<Objeto>>,
    pub techo: Vec<u8>,
    pub fuegos: Vec<((i64, i64), i64)>,
}

fn round_par(x: f64) -> i64 {
    x.round_ties_even() as i64
}

impl Bosque {
    pub fn new(semilla: u64, c: &Config) -> Bosque {
        let (w, h) = (c.bosque_ancho, c.bosque_alto);
        let n = (w * h) as usize;
        let mut b = Bosque {
            rng: Azar::new(semilla), w, h, tiempo: 0, lluvia: 0, crecida: 0, comida: vec![],
            predador: (0, 0), harto: 0, rastro: vec![-1; n], atacando: false,
            zona: vec![Z_BOSQUE; n], g: vec![VACIO; n], rio_x: vec![], cuevas: vec![],
            claros: vec![], arboles: [vec![], vec![], vec![], vec![]], cueva_predador: (0, 0),
            inicio: (0, 0), agua_cerca: vec![],
            objetos: c.objetos, orng: Azar::new(semilla ^ 0x0b7e_7051),
            pilas: vec![vec![]; if c.objetos { n } else { 0 }],
            techo: vec![0; if c.objetos { n } else { 0 }], fuegos: vec![],
        };
        b.generar(c);
        if c.objetos {
            for _ in 0..c.objetos_iniciales {
                b.nuevo_objeto();
            }
        }
        b
    }

    #[inline]
    fn i(&self, x: i64, y: i64) -> usize {
        (y * self.w + x) as usize
    }
    #[inline]
    pub fn en(&self, x: i64, y: i64) -> u8 {
        self.g[self.i(x, y)]
    }
    #[inline]
    pub fn zona_en(&self, x: i64, y: i64) -> u8 {
        self.zona[self.i(x, y)]
    }

    fn generar(&mut self, c: &Config) {
        let (w, h) = (self.w, self.h);
        let x0 = (w as f64 * 0.6) as i64;
        let fase = self.rng.random() * 6.28;
        for y in 0..h {
            let rx = x0 + (3.0 * crate::mate::sin(y as f64 / 7.0 + fase)) as i64;
            self.rio_x.push(rx);
            for x in 0..w {
                let i = self.i(x, y);
                if x > rx + 1 {
                    self.zona[i] = Z_LLANURA;
                } else if rx - 1 <= x && x <= rx {
                    self.zona[i] = Z_RIO;
                }
            }
        }
        for &(cx, cy, rr, n) in &[(6i64, 6i64, 6i64, 2i64), (x0 + 12, h - 12, 3, 1)] {
            for y in (cy - rr).max(0)..(cy + rr + 1).min(h) {
                for x in (cx - rr).max(0)..(cx + rr + 1).min(w) {
                    let i = self.i(x, y);
                    if (x - cx) * (x - cx) + (y - cy) * (y - cy) <= rr * rr && self.zona[i] != Z_RIO {
                        self.zona[i] = Z_ROCA;
                    }
                }
            }
            let mut puestos = 0;
            for _ in 0..200 {
                if puestos == n {
                    break;
                }
                let a = self.rng.random() * 6.28;
                let x = round_par(cx as f64 + (rr - 1) as f64 * crate::mate::cos(a));
                let y = round_par(cy as f64 + (rr - 1) as f64 * crate::mate::sin(a));
                if !(0 < x && x < w - 1 && 0 < y && y < h - 1) {
                    continue;
                }
                let afuera = [(0i64, -1i64), (0, 1), (-1, 0), (1, 0)].iter().any(|&(dx, dy)| {
                    let z = self.zona_en(x + dx, y + dy);
                    z != Z_ROCA && z != Z_CUEVA
                });
                let i = self.i(x, y);
                if afuera && self.zona[i] == Z_ROCA {
                    self.zona[i] = Z_CUEVA;
                    self.cuevas.push((x, y));
                    puestos += 1;
                }
            }
        }
        for _ in 0..6 {
            let (mut cx, mut cy) = (0, 0);
            for _ in 0..50 {
                cx = self.rng.randrange(4, x0 - 4);
                cy = self.rng.randrange(4, h - 4);
                if self.zona_en(cx, cy) == Z_BOSQUE {
                    break;
                }
            }
            let rr = self.rng.randint(3, 5);
            self.claros.push((cx, cy));
            for y in cy - rr..cy + rr + 1 {
                for x in cx - rr..cx + rr + 1 {
                    if 0 <= x && x < w && 0 <= y && y < h
                        && (x - cx) * (x - cx) + (y - cy) * (y - cy) <= rr * rr
                        && self.zona_en(x, y) == Z_BOSQUE
                    {
                        let i = self.i(x, y);
                        self.zona[i] = Z_CLARO;
                    }
                }
            }
        }
        for y in 0..h {
            for x in 0..w {
                let i = self.i(x, y);
                self.g[i] = match self.zona[i] {
                    Z_ROCA => ROCA,
                    Z_CUEVA => DESCANSO,
                    Z_RIO => AGUA,
                    _ => VACIO,
                };
            }
        }
        for k in 0..(c.charcos.max(0) as usize).min(self.claros.len()) {
            let (cx, cy) = self.claros[k];
            let i = self.i(cx, cy);
            self.g[i] = AGUA;
        }
        let celdas = |b: &Bosque, z: u8| -> Vec<(i64, i64)> {
            let mut v = vec![];
            for y in 0..h {
                for x in 0..w {
                    if b.zona_en(x, y) == z && b.en(x, y) == VACIO {
                        v.push((x, y));
                    }
                }
            }
            v
        };
        let bosque = celdas(self, Z_BOSQUE);
        self.arboles[FRUTA as usize] = self.rng.sample(&bosque, 10);
        self.arboles[NUEZ as usize] = self.rng.sample(&bosque, 8);
        self.arboles[BAYA as usize] = self.claros.clone();
        let llanura = celdas(self, Z_LLANURA);
        self.arboles[RAIZ as usize] = self.rng.sample(&llanura, 8);
        for _ in 0..45 {
            let (fx, fy) = if self.rng.random() < 0.5 {
                let a = &self.arboles[FRUTA as usize];
                a[self.rng.indice(a.len())]
            } else {
                bosque[self.rng.indice(bosque.len())]
            };
            let x = fx + self.rng.randint(-3, 3);
            let y = fy + self.rng.randint(-3, 3);
            if 0 <= x && x < w && 0 <= y && y < h && self.en(x, y) == VACIO && self.zona_en(x, y) == Z_BOSQUE {
                let i = self.i(x, y);
                self.g[i] = PELIGRO;
            }
        }
        for _ in 0..c.novedades_iniciales {
            self.novedad();
        }
        for tipo in 0..4u8 {
            let n = (TOPE[tipo as usize] as f64 * c.abundancia) as i64 / 2;
            for _ in 0..n {
                self.brotar(tipo, true, c);
            }
        }
        let c0 = self.claros[0];
        let mut mejor = bosque[0];
        let mut dm = -1;
        for &p in &bosque {
            let d = (p.0 - c0.0).abs() + (p.1 - c0.1).abs();
            if d > dm {
                dm = d;
                mejor = p;
            }
        }
        self.cueva_predador = mejor;
        self.predador = mejor;
        self.inicio = c0;
        self.distancias_agua();
    }

    fn novedad(&mut self) {
        for _ in 0..50 {
            let x = self.rng.randrange(0, self.w);
            let y = self.rng.randrange(0, self.h);
            if self.en(x, y) == VACIO {
                let i = self.i(x, y);
                self.g[i] = NOVEDAD;
                return;
            }
        }
    }

    fn brotar(&mut self, tipo: u8, forzar: bool, c: &Config) {
        if !forzar && !da(tipo, self.estacion(c)) {
            return;
        }
        let cuantas = self.comida.iter().filter(|e| e.1 == tipo).count() as f64;
        if cuantas >= TOPE[tipo as usize] as f64 * c.abundancia {
            return;
        }
        let a = &self.arboles[tipo as usize];
        let (fx, fy) = a[self.rng.indice(a.len())];
        let rr = if tipo == BAYA { 3 } else { 2 };
        let x = fx + self.rng.randint(-rr, rr);
        let y = fy + self.rng.randint(-rr, rr);
        if 0 <= x && x < self.w && 0 <= y && y < self.h && self.en(x, y) == VACIO {
            let i = self.i(x, y);
            self.g[i] = COMIDA;
            // (la celda estaba vacía: no hay entrada vieja en el dict)
            self.comida.push(((x, y), tipo, 0));
        }
    }

    fn distancias_agua(&mut self) {
        let (w, h) = (self.w, self.h);
        let mut cerca = vec![(-1i64, -1i64); (w * h) as usize];
        let mut frente = vec![];
        for y in 0..h {
            for x in 0..w {
                if self.en(x, y) == AGUA {
                    cerca[self.i(x, y)] = (x, y);
                    frente.push((x, y));
                }
            }
        }
        while !frente.is_empty() {
            let mut nuevo = vec![];
            for &(x, y) in &frente {
                for &(dx, dy) in &[(0i64, -1i64), (0, 1), (-1, 0), (1, 0)] {
                    let (px, py) = (x + dx, y + dy);
                    if 0 <= px && px < w && 0 <= py && py < h && cerca[(py * w + px) as usize].0 < 0 {
                        cerca[(py * w + px) as usize] = cerca[(y * w + x) as usize];
                        nuevo.push((px, py));
                    }
                }
            }
            frente = nuevo;
        }
        self.agua_cerca = cerca;
    }

    pub fn agua_de(&self, x: i64, y: i64) -> (i64, i64) {
        let a = self.agua_cerca[self.i(x, y)];
        if a.0 < 0 { (x + 99, y) } else { a }
    }

    // --- reloj ---
    pub fn noche(&self, c: &Config) -> bool {
        let hora = (self.tiempo.rem_euclid(c.dia)) as f64 / c.dia as f64;
        hora >= 1.0 - c.fraccion_noche
    }
    pub fn estacion(&self, c: &Config) -> i64 {
        (self.tiempo.div_euclid(c.estacion_bosque)).rem_euclid(4)
    }
    pub fn temperatura(&self, x: i64, y: i64, c: &Config) -> f64 {
        let z = self.zona_en(x, y);
        if z == Z_CUEVA {
            return c.temp_cueva;
        }
        let ts = [c.temp_dia, c.temp_dia + 0.1, c.temp_dia - 0.05, c.temp_dia - 0.2];
        let mut t = ts[self.estacion(c) as usize];
        if self.noche(c) {
            t -= c.frio_noche;
        }
        let tapado = self.objetos && self.techo[self.i(x, y)] > 0;
        if !tapado {
            if self.lluvia != 0 && z != Z_BOSQUE {
                t -= c.frio_lluvia;
            } else if self.lluvia != 0 {
                t -= c.frio_lluvia / 2.0;
            }
        }
        if self.objetos {
            if self.noche(c) || self.lluvia != 0 {
                t += c.corta_viento * (self.paredes(x, y) + tapado as i64) as f64;
            }
            if self.fuego_cerca(x, y, 1) {
                t += c.calor_fuego;
            }
        }
        t.min(1.0).max(0.0)
    }

    pub fn transitable(&self, x: i64, y: i64) -> bool {
        0 <= x && x < self.w && 0 <= y && y < self.h && self.en(x, y) != ROCA
            && (!self.objetos || self.pilas[self.i(x, y)].len() < 3)
    }

    pub fn consumir(&mut self, x: i64, y: i64) {
        let i = self.i(x, y);
        if self.g[i] == COMIDA || self.g[i] == NOVEDAD {
            self.g[i] = VACIO;
        }
        if let Some(k) = self.comida.iter().position(|e| e.0 == (x, y)) {
            self.comida.remove(k);
        }
    }

    pub fn vecindad(&self, x: i64, y: i64, c: &Config) -> Vista {
        let radio = if self.noche(c) { c.vista_noche } else { c.vista_dia };
        let mut v = Vista { v: Vec::with_capacity(28) };
        for dy in -radio..=radio {
            let ny = y + dy;
            if 0 <= ny && ny < self.h {
                for dx in -radio..=radio {
                    let nx = x + dx;
                    if 0 <= nx && nx < self.w {
                        v.v.push(((dx, dy), self.en(nx, ny)));
                    }
                }
            }
        }
        let (px, py) = self.predador;
        if (px - x).abs() <= radio && (py - y).abs() <= radio && self.harto == 0 {
            v.poner((px - x, py - y), PELIGRO);
        }
        v
    }

    pub fn sentir(&self, x: i64, y: i64, vista: &Vista, c: &Config) -> Vista {
        let mut s = vista.clone();
        let mut mejor = None;
        let mut d0 = c.olfato + 1;
        for e in &self.comida {
            let (cx, cy) = e.0;
            let d = (cx - x).abs() + (cy - y).abs();
            if d < d0 {
                mejor = Some((cx - x, cy - y));
                d0 = d;
            }
        }
        if let Some(m) = mejor {
            if !s.tiene(m) {
                s.v.push((m, COMIDA));
            }
        }
        let (ax, ay) = self.agua_de(x, y);
        if (ax - x).abs() + (ay - y).abs() <= c.humedad && !s.tiene((ax - x, ay - y)) {
            s.v.push(((ax - x, ay - y), AGUA));
        }
        let (px, py) = self.predador;
        let d = (px - x).abs() + (py - y).abs();
        if self.harto == 0 && (d <= c.olor_predador || (self.noche(c) && d <= c.oido_predador)) {
            if !s.tiene((px - x, py - y)) {
                s.v.push(((px - x, py - y), PELIGRO));
            }
        }
        s
    }

    pub fn avanzar(&mut self, angri: (i64, i64), refugiado: bool, c: &Config) {
        self.tiempo += 1;
        if self.lluvia != 0 {
            self.lluvia -= 1;
            if self.lluvia > 0 && self.crecida == 0 && self.rng.random() < c.prob_crecida {
                self.crecer(true);
            }
        } else if self.rng.random() < c.prob_lluvia {
            self.lluvia = self.rng.randint(200, 700);
        }
        if self.crecida != 0 {
            self.crecida -= 1;
            if self.crecida == 0 {
                self.crecer(false);
            }
        }
        if self.rng.random() < c.prob_brote {
            let t = self.rng.choice(&[FRUTA, FRUTA, BAYA, NUEZ, RAIZ]);
            self.brotar(t, false, c);
        }
        if self.rng.random() < c.prob_nueva_novedad
            && (self.g.iter().filter(|&&x| x == NOVEDAD).count() as i64) < c.novedad_maxima
        {
            self.novedad();
        }
        if self.tiempo % 100 == 0 {
            let est = self.estacion(c);
            let lista: Vec<((i64, i64), u8)> = self.comida.iter().map(|e| (e.0, e.1)).collect();
            for (p, tipo) in lista {
                if !da(tipo, est) && self.rng.random() < 0.1 {
                    self.consumir(p.0, p.1);
                }
            }
        }
        let i = self.i(angri.0, angri.1);
        self.rastro[i] = self.tiempo;
        if self.tiempo % 200 == 0 {
            let t = self.tiempo;
            for r in self.rastro.iter_mut() {
                if *r >= 0 && t - *r > c.rastro_dura {
                    *r = -1;
                }
            }
        }
        if self.objetos {
            self.avanzar_objetos(c);
        }
        self.mover_predador(angri, refugiado, c);
    }

    fn crecer(&mut self, si: bool) {
        self.crecida = if si { self.rng.randint(300, 900) } else { 0 };
        for y in 0..self.h {
            let rx = self.rio_x[y as usize];
            for x in (rx - 2).max(0)..(rx + 3).min(self.w) {
                let i = self.i(x, y);
                if self.zona[i] == Z_RIO {
                    self.g[i] = if si { PELIGRO } else { AGUA };
                } else if (self.g[i] == VACIO || self.g[i] == PELIGRO) && (x == rx - 2 || x == rx + 1 || x == rx + 2) {
                    self.g[i] = if si { PELIGRO } else { VACIO };
                }
            }
        }
        self.distancias_agua();
    }

    fn puede_predador(&self, x: i64, y: i64) -> bool {
        if !(0 <= x && x < self.w && 0 <= y && y < self.h) {
            return false;
        }
        let z = self.zona_en(x, y);
        let g = self.en(x, y);
        (z == Z_BOSQUE || z == Z_CLARO || z == Z_LLANURA) && g != AGUA && g != ROCA
            && (!self.objetos
                || (self.pilas[self.i(x, y)].len() < 2 && !self.hueco(x, y) && !self.fuego_cerca(x, y, 1)))
    }

    /// Donde el predador no llega: la cueva, o (con objetos) algo que Angri armó.
    pub fn refugio(&self, x: i64, y: i64) -> bool {
        self.zona_en(x, y) == Z_CUEVA
            || (self.objetos && self.en(x, y) != AGUA && self.zona_en(x, y) != Z_RIO && !self.puede_predador(x, y))
    }

    fn olor(&self, x: i64, y: i64, c: &Config) -> f64 {
        let mut mejor = 0.0;
        for &(dx, dy, f) in &[(0i64, 0i64, 1.0f64), (0, -1, 0.5), (0, 1, 0.5), (-1, 0, 0.5), (1, 0, 0.5)] {
            let (qx, qy) = (x + dx, y + dy);
            if !(0 <= qx && qx < self.w && 0 <= qy && qy < self.h) {
                continue;
            }
            let t = self.rastro[self.i(qx, qy)];
            if t >= 0 {
                let v = f * crate::mate::pow(c.rastro_decae, (self.tiempo - t) as f64);
                if v > mejor {
                    mejor = v;
                }
            }
        }
        mejor
    }

    fn mover_predador(&mut self, angri: (i64, i64), refugiado: bool, c: &Config) {
        self.atacando = false;
        let (px, py) = self.predador;
        if self.harto != 0 {
            self.harto -= 1;
            if self.rng.random() < 0.3 {
                let (gx, gy) = self.cueva_predador;
                let paso = if self.rng.random() < 0.5 {
                    (if gx == px { 0 } else if gx > px { 1 } else { -1 }, 0)
                } else {
                    (0, if gy == py { 0 } else if gy > py { 1 } else { -1 })
                };
                if self.puede_predador(px + paso.0, py + paso.1) {
                    self.predador = (px + paso.0, py + paso.1);
                }
            }
            return;
        }
        let umbral = if self.noche(c) { c.predador_noche } else { c.predador_dia };
        if self.rng.random() >= umbral {
            return;
        }
        let mut opciones = Vec::with_capacity(4);
        for &(dx, dy) in &[(0i64, -1i64), (0, 1), (-1, 0), (1, 0)] {
            if self.puede_predador(px + dx, py + dy) {
                opciones.push((px + dx, py + dy));
            }
        }
        if opciones.is_empty() {
            return;
        }
        let (ax, ay) = angri;
        let cerca = (ax - px).abs() + (ay - py).abs();
        let elegido;
        if cerca <= 2 && !refugiado {
            let mut m = opciones[0];
            let mut dm = i64::MAX;
            for &p in &opciones {
                let d = (p.0 - ax).abs() + (p.1 - ay).abs();
                if d < dm {
                    dm = d;
                    m = p;
                }
            }
            elegido = m;
        } else {
            let olores: Vec<f64> = opciones.iter().map(|p| self.olor(p.0, p.1, c)).collect();
            let mut im = 0;
            for k in 1..olores.len() {
                if olores[k] > olores[im] {
                    im = k;
                }
            }
            let m = olores[im];
            let p = if self.noche(c) { 0.9 } else { 0.5 };
            if m > 0.05 && self.rng.random() < p {
                elegido = opciones[im];
            } else {
                elegido = self.rng.choice(&opciones);
            }
        }
        self.predador = elegido;
        if elegido == (ax, ay) && !refugiado {
            self.atacando = true;
            self.harto = c.predador_harto;
        }
    }

    // ------------------------------------------------------------- objetos (13b)
    #[inline]
    fn dentro(&self, x: i64, y: i64) -> bool {
        0 <= x && x < self.w && 0 <= y && y < self.h
    }

    /// altura de lo que hay: la roca cuenta como 2, afuera del mapa como muro alto
    pub fn altura(&self, x: i64, y: i64) -> i64 {
        if !self.dentro(x, y) {
            return 9;
        }
        let base = if self.en(x, y) == ROCA { 2 } else { 0 };
        base + self.pilas.get(self.i(x, y)).map_or(0, |p| p.len() as i64)
    }

    /// cierra el paso a algo grande (el predador): afuera, roca o pila de 2
    fn cierra(&self, x: i64, y: i64) -> bool {
        self.altura(x, y) >= 2
    }

    /// hueco chico: cerrado de los dos lados opuestos
    fn hueco(&self, x: i64, y: i64) -> bool {
        (self.cierra(x - 1, y) && self.cierra(x + 1, y)) || (self.cierra(x, y - 1) && self.cierra(x, y + 1))
    }

    pub fn paredes(&self, x: i64, y: i64) -> i64 {
        DIRS[..4].iter().filter(|&&(dx, dy)| self.cierra(x + dx, y + dy)).count() as i64
    }

    pub fn fuego_en(&self, x: i64, y: i64) -> bool {
        self.fuegos.iter().any(|f| f.0 == (x, y))
    }

    fn fuego_cerca(&self, x: i64, y: i64, r: i64) -> bool {
        self.fuegos.iter().any(|f| (f.0 .0 - x).abs() + (f.0 .1 - y).abs() <= r)
    }

    pub fn cubierto(&self, x: i64, y: i64) -> bool {
        self.objetos && self.dentro(x, y) && self.techo[self.i(x, y)] > 0
    }

    pub fn total_objetos(&self) -> i64 {
        self.pilas.iter().map(|p| p.len() as i64).sum()
    }

    fn nuevo_objeto(&mut self) {
        for _ in 0..30 {
            let x = self.orng.randrange(0, self.w);
            let y = self.orng.randrange(0, self.h);
            let i = self.i(x, y);
            let z = self.zona[i];
            if self.g[i] != VACIO || z == Z_RIO || z == Z_CUEVA || !self.pilas[i].is_empty() {
                continue;
            }
            let roca = DIRS[..4].iter()
                .any(|&(dx, dy)| self.dentro(x + dx, y + dy) && self.zona_en(x + dx, y + dy) == Z_ROCA);
            let r = &mut self.orng;
            let u = r.random();
            let o = if z == Z_LLANURA || roca {
                if u < 0.6 {
                    // piedra (a veces una laja de largo 2)
                    let l: u8 = if r.random() < 0.2 { 2 } else { 1 };
                    let peso = 0.35 * l as f64 + 0.3 * r.random();
                    let dureza = 0.8 + 0.2 * r.random();
                    Objeto { largo: l, peso, dureza, flota: false, arde: false, flexible: false, eje: 0 }
                } else {
                    // pasto seco
                    Objeto { largo: 1, peso: 0.01, dureza: 0.02, flota: true, arde: true, flexible: true, eje: 0 }
                }
            } else if u < 0.6 {
                // rama
                let l = r.randint(1, 4) as u8;
                let peso = 0.06 * l as f64 + 0.04 * r.random();
                let dureza = 0.3 + 0.3 * r.random();
                Objeto { largo: l, peso, dureza, flota: true, arde: true, flexible: false, eje: 0 }
            } else if u < 0.85 {
                // liana
                let l = r.randint(2, 4) as u8;
                let arde = r.random() < 0.3;
                Objeto { largo: l, peso: 0.02 * l as f64, dureza: 0.1, flota: true, arde, flexible: true, eje: 0 }
            } else {
                // hoja grande
                let l = r.randint(1, 2) as u8;
                Objeto { largo: l, peso: 0.02, dureza: 0.05, flota: true, arde: true, flexible: true, eje: 0 }
            };
            self.pilas[i].push(o);
            return;
        }
    }

    fn avanzar_objetos(&mut self, c: &Config) {
        if self.orng.random() < c.prob_objeto && self.total_objetos() < c.objetos_maximo {
            self.nuevo_objeto();
        }
        if !self.fuegos.is_empty() {
            let lluvia = self.lluvia != 0;
            let w = self.w;
            let techo = &self.techo;
            for f in self.fuegos.iter_mut() {
                f.1 -= 1;
                if lluvia && techo[(f.0 .1 * w + f.0 .0) as usize] == 0 {
                    f.1 -= 5;
                }
            }
            self.fuegos.retain(|f| f.1 > 0);
        }
    }

    /// ¿la viga de (x, y), a altura k de su pila, está bien apoyada?
    fn apoyada(&self, x: i64, y: i64, k: i64, o: &Objeto) -> bool {
        let (dx, dy) = DIRS[o.eje as usize - 1];
        let l = o.largo as i64;
        let (ex, ey) = (x + dx * (l - 1), y + dy * (l - 1));
        let base = if self.en(x, y) == ROCA { 2 } else { 0 };
        base + k >= 2 && (l - 1 <= 1 || (self.dentro(ex, ey) && self.altura(ex, ey) >= 2))
    }

    /// recalcula qué celdas tapa cada viga; la que perdió el apoyo de la punta queda suelta
    fn recalcular_techo(&mut self) {
        for t in self.techo.iter_mut() {
            *t = 0;
        }
        for y in 0..self.h {
            for x in 0..self.w {
                let i = self.i(x, y);
                for k in 0..self.pilas[i].len() {
                    let o = self.pilas[i][k];
                    if o.eje == 0 {
                        continue;
                    }
                    if !self.apoyada(x, y, k as i64, &o) {
                        self.pilas[i][k].eje = 0;
                        continue;
                    }
                    let (dx, dy) = DIRS[o.eje as usize - 1];
                    for j in 0..o.largo as i64 {
                        let (cx, cy) = (x + dx * j, y + dy * j);
                        if self.dentro(cx, cy) {
                            let ci = self.i(cx, cy);
                            self.techo[ci] = self.techo[ci].saturating_add(1);
                        }
                    }
                }
            }
        }
    }

    pub fn tacto(&self, x: i64, y: i64, lleva: Option<Objeto>, c: &Config) -> Tacto {
        let i = self.i(x, y);
        let mut t = Tacto { lleva, aqui: self.pilas[i].last().copied(), h_aqui: self.pilas[i].len() as i64,
                            ..Default::default() };
        for a in 0..5 {
            let (px, py) = (x + DIRS[a].0, y + DIRS[a].1);
            t.h_dest[a] = self.altura(px, py).min(3);
            t.techo_dest[a] = self.cubierto(px, py);
            t.fuego_dest[a] = self.fuego_en(px, py);
        }
        t.paredes = self.paredes(x, y);
        t.cubierto = self.techo[i] > 0;
        t.clima_malo = self.noche(c) || self.lluvia != 0;
        t.dura = self.comida.iter().any(|e| e.0 == (x, y) && (e.1 == NUEZ || e.1 == RAIZ));
        t
    }

    /// agarra lo de arriba de la pila de acá, si puede levantarlo
    pub fn agarrar(&mut self, x: i64, y: i64, c: &Config) -> Option<Objeto> {
        let i = self.i(x, y);
        let o = *self.pilas[i].last()?;
        if o.peso > c.fuerza {
            return None;
        }
        self.pilas[i].pop();
        if o.eje != 0 {
            self.recalcular_techo();
        }
        Some(Objeto { eje: 0, ..o })
    }

    /// pone `o` arriba de la pila de (x, y). Con `dir` < 4 lo apoya mirando hacia ahí
    /// (si es largo y queda alto, es una viga).
    pub fn poner(&mut self, x: i64, y: i64, o: Objeto, dir: usize) -> Hecho {
        if !self.dentro(x, y) {
            return self.caer(x - DIRS[dir].0, y - DIRS[dir].1, o);
        }
        let i = self.i(x, y);
        if (self.g[i] == AGUA || self.zona[i] == Z_RIO) && !o.flota {
            return Hecho::Hunde;
        }
        if let Some(t) = self.pilas[i].last() {
            if o.peso > t.dureza + 0.3 || (t.flexible && o.peso > 0.1) {
                return self.caer(x, y, o);
            }
        }
        let mut o = Objeto { eje: 0, ..o };
        let mut hecho = if dir < 4 { Hecho::Apila } else { Hecho::Suelta };
        if dir < 4 && self.altura(x, y) >= 2 && o.largo >= 3 {
            o.eje = dir as u8 + 1;
            let k = self.pilas[i].len() as i64;
            if self.apoyada(x, y, k, &o) {
                hecho = Hecho::Viga;
            } else {
                // sobresale sin apoyo en la punta: se vuelca hacia ese lado
                return self.caer(x + DIRS[dir].0, y + DIRS[dir].1, o);
            }
        }
        self.pilas[i].push(o);
        if hecho == Hecho::Viga {
            self.recalcular_techo();
        }
        hecho
    }

    /// se cae: queda tirado en (x, y) si se puede, si no en un vecino al azar
    fn caer(&mut self, x: i64, y: i64, o: Objeto) -> Hecho {
        let o = Objeto { eje: 0, ..o };
        let libre = |b: &Bosque, x: i64, y: i64| b.dentro(x, y) && b.en(x, y) != ROCA;
        let (mut tx, mut ty) = (x, y);
        if !libre(self, x, y) {
            let ops: Vec<(i64, i64)> = DIRS[..4].iter().map(|&(dx, dy)| (x + dx, y + dy))
                .filter(|&(a, b)| libre(self, a, b)).collect();
            if ops.is_empty() {
                return Hecho::Hunde;
            }
            (tx, ty) = self.orng.choice(&ops);
        }
        let i = self.i(tx, ty);
        if (self.g[i] == AGUA || self.zona[i] == Z_RIO) && !o.flota {
            return Hecho::Hunde;
        }
        self.pilas[i].push(o);
        Hecho::Cae
    }

    /// atar: algo flexible une las dos cosas de arriba de la pila en una sola, más larga
    pub fn atar(&mut self, x: i64, y: i64, cuerda: &Objeto) -> bool {
        let i = self.i(x, y);
        let p = &mut self.pilas[i];
        if !cuerda.flexible || p.len() < 2 {
            return false;
        }
        let a = p.pop().unwrap();
        let b = p.pop().unwrap();
        p.push(Objeto {
            largo: (a.largo + b.largo).min(8), peso: a.peso + b.peso + cuerda.peso, dureza: a.dureza.min(b.dureza),
            flota: a.flota && b.flota, arde: a.arde || b.arde, flexible: false, eje: 0,
        });
        if a.eje != 0 || b.eje != 0 {
            self.recalcular_techo();
        }
        true
    }

    /// golpear: abre la comida dura (más rápido con algo duro en la mano);
    /// si no hay, con algo más duro puede partir lo de arriba de la pila
    pub fn golpear(&mut self, x: i64, y: i64, con: Option<&Objeto>, c: &Config) -> Hecho {
        if let Some(k) = self.comida.iter().position(|e| e.0 == (x, y) && (e.1 == NUEZ || e.1 == RAIZ)) {
            let tipo = self.comida[k].1;
            let trabajo = if tipo == NUEZ { c.nuez_ticks } else { c.raiz_ticks };
            self.comida[k].2 += 1 + con.map_or(0, |o| (o.dureza * trabajo as f64) as i64);
            if self.comida[k].2 >= trabajo {
                self.consumir(x, y);
                return Hecho::Abre(tipo);
            }
            return Hecho::Golpea;
        }
        let i = self.i(x, y);
        if let (Some(h), Some(&t)) = (con, self.pilas[i].last()) {
            if t.largo >= 2 && !t.flexible && h.dureza > t.dureza + 0.2 {
                self.pilas[i].pop();
                let l1 = t.largo / 2;
                let l2 = t.largo - l1;
                let f = l1 as f64 / t.largo as f64;
                self.pilas[i].push(Objeto { largo: l2, peso: t.peso * (1.0 - f), eje: 0, ..t });
                self.pilas[i].push(Objeto { largo: l1, peso: t.peso * f, eje: 0, ..t });
                if t.eje != 0 {
                    self.recalcular_techo();
                }
                return Hecho::Rompe;
            }
        }
        Hecho::Golpea
    }

    /// frotar lo que lleva contra lo de arriba: si los dos arden y no le llueve encima, puede prender
    pub fn frotar(&mut self, x: i64, y: i64, con: Option<&Objeto>, c: &Config) -> Hecho {
        let i = self.i(x, y);
        let (Some(h), Some(t)) = (con, self.pilas[i].last()) else { return Hecho::Nada };
        if !(h.arde && t.arde) || (self.lluvia != 0 && self.techo[i] == 0) || self.fuego_en(x, y)
            || self.orng.random() >= c.prob_prender
        {
            return Hecho::Frota;
        }
        let quema: i64 = self.pilas[i].iter().filter(|o| o.arde).map(|o| o.largo as i64).sum();
        let viga = self.pilas[i].iter().any(|o| o.arde && o.eje != 0);
        self.pilas[i].retain(|o| !o.arde);
        self.fuegos.push(((x, y), c.fuego_ticks * quema));
        if viga {
            self.recalcular_techo();
        }
        Hecho::Prende
    }

    /// deja caer lo que lleva (al desmayarse)
    pub fn tirar(&mut self, x: i64, y: i64, o: Objeto) {
        self.caer(x, y, o);
    }

    /// trabajar() de bosque.py: devuelve el tipo cuando se la come.
    pub fn trabajar(&mut self, x: i64, y: i64, c: &Config) -> Option<u8> {
        let k = self.comida.iter().position(|e| e.0 == (x, y))?;
        self.comida[k].2 += 1;
        let tipo = self.comida[k].1;
        let trabajo = match tipo { NUEZ => c.nuez_ticks, RAIZ => c.raiz_ticks, _ => 1 };
        if self.comida[k].2 >= trabajo {
            self.consumir(x, y);
            return Some(tipo);
        }
        None
    }
}

/// El mundo de una vida.
pub enum Mundo {
    G(Grilla),
    B(Bosque),
}

impl Mundo {
    pub fn en(&self, x: i64, y: i64) -> u8 {
        match self { Mundo::G(m) => m.en(x, y), Mundo::B(b) => b.en(x, y) }
    }
    pub fn vecindad(&self, x: i64, y: i64, c: &Config) -> Vista {
        match self { Mundo::G(m) => m.vecindad(x, y, 2), Mundo::B(b) => b.vecindad(x, y, c) }
    }
    pub fn ancho_alto(&self) -> (i64, i64) {
        match self { Mundo::G(m) => (m.w, m.h), Mundo::B(b) => (b.w, b.h) }
    }
    pub fn celdas(&self) -> &[u8] {
        match self { Mundo::G(m) => &m.g, Mundo::B(b) => &b.g }
    }
}

crate::pieza!(Grilla { rng, fuentes, tiempo, cambios, cambio, g, w, h });
crate::pieza!(Bosque { rng, w, h, tiempo, lluvia, crecida, comida, predador, harto, rastro, atacando,
                      zona, g, rio_x, cuevas, claros, arboles, cueva_predador, inicio, agua_cerca,
                      objetos, orng, pilas, techo, fuegos });
crate::pieza!(Objeto { largo, peso, dureza, flota, arde, flexible, eje });

impl crate::guardar::Pieza for Mundo {
    fn guardar(&self, e: &mut crate::guardar::Escritor) {
        match self {
            Mundo::G(m) => { e.tok("grilla"); m.guardar(e) }
            Mundo::B(b) => { e.tok("bosque"); b.guardar(e) }
        }
    }
    fn cargar(l: &mut crate::guardar::Lector) -> Self {
        match l.tok() {
            "grilla" => Mundo::G(Grilla::cargar(l)),
            _ => Mundo::B(Bosque::cargar(l)),
        }
    }
}

#[cfg(test)]
impl Bosque {
    pub fn puede_predador_pub(&self, x: i64, y: i64) -> bool {
        self.puede_predador(x, y)
    }
}
