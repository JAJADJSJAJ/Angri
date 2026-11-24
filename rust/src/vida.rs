//! El tick de angri/vida.py.
use crate::config::Config;
use crate::extra::{Corteza, MenteVieja, Talamo};
use crate::lenguaje::{Habla, Maestro, Oido};
use crate::mente::*;
use crate::mundo::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Evento {
    Nada,
    Come,
    Descubre,
    Lastima,
    Muerde,
    Enferma,
    Bebe,
    Desmaya,
    Otro,
}

pub struct Angri {
    pub cuerpo: Cuerpo,
    pub quimica: Quimica,
    pub mente: Ganglios,
    pub vieja: Option<MenteVieja>, // MENTE="azar": la mente vieja en vez de los ganglios
    pub corteza: Option<Corteza>,
    pub hipocampo: Option<Hipocampo>,
    pub talamo: Option<Talamo>,
    pub maestro: Option<Maestro>,
    pub habla: Option<Habla>,
    pub oido: Oido,
    pub mundo: Mundo,
    pub x: i64,
    pub y: i64,
    pub despierto: bool,
    pub edad: i64,
    pub suenos: i64,
    pub desmayos: i64,
    pub causas: [i64; 4], // hambre, sed, frio, heridas
    pub malestar: f64,
    pub evento: Evento,
}

impl Angri {
    pub fn new(semilla: u64, c: &Config) -> Angri {
        let bosque = c.bosque;
        let cuerpo = Cuerpo::new(bosque, c);
        let aprende = !c.mente_azar;
        let mente = Ganglios::new(semilla);
        let mundo = if bosque {
            Mundo::B(Bosque::new(semilla + 1, c))
        } else {
            Mundo::G(Grilla::new(semilla + 1, c))
        };
        let (x, y) = match &mundo { Mundo::B(b) => b.inicio, Mundo::G(_) => (8, 7) };
        let malestar = cuerpo.malestar(c);
        let maestro = if aprende && c.maestro { Some(Maestro::new(semilla)) } else { None };
        let habla = match &maestro {
            Some(m) if c.pide => Some(Habla::new(semilla, m)),
            _ => None,
        };
        Angri {
            cuerpo, quimica: Quimica::new(), mente,
            vieja: if aprende { None } else { Some(MenteVieja::new(semilla)) },
            corteza: if aprende && c.corteza { Some(Corteza::new()) } else { None },
            hipocampo: if aprende && c.hipocampo { Some(Hipocampo::new()) } else { None },
            talamo: if aprende && c.talamo { Some(Talamo::new()) } else { None },
            maestro, habla,
            oido: Oido::new(c), mundo, x, y, despierto: true, edad: 0, suenos: 0, desmayos: 0,
            causas: [0; 4], malestar, evento: Evento::Otro,
        }
    }

    fn vista_sentida(&self, c: &Config) -> (Vista, Vista) {
        let vista = self.mundo.vecindad(self.x, self.y, c);
        let sentida = match &self.mundo {
            Mundo::B(b) => b.sentir(self.x, self.y, &vista, c),
            Mundo::G(_) => vista.clone(),
        };
        (vista, sentida)
    }

    pub fn tick(&mut self, c: &Config) {
        let bosque = c.bosque;
        let mut mordido = false;
        match &mut self.mundo {
            Mundo::B(b) => {
                let refugio = b.zona_en(self.x, self.y) == Z_CUEVA;
                b.avanzar((self.x, self.y), refugio, c);
                if b.atacando {
                    self.cuerpo.lastimarse_mucho();
                    self.evento = Evento::Muerde;
                    self.despierto = true;
                    mordido = true;
                }
            }
            Mundo::G(g) => g.avanzar(c),
        }
        let mut se_movio = false;
        let mut sorpresa = 0.0;
        let mut encontrado = VACIO;

        if self.despierto {
            let (vista, sentida) = self.vista_sentida(c);
            let necesidad = match &mut self.talamo {
                Some(t) => t.elegir(&vista, &self.cuerpo.v, &mut self.mente.rng, c),
                None => self.cuerpo.necesidad_urgente(c),
            };
            self.oido.pasar();
            if let Some(m) = &mut self.maestro {
                if c.cambio_significado != 0 && self.edad == c.cambio_significado {
                    m.cambiar_significados();
                    self.oido.indice_cambio = Some(self.oido.aciertos.len());
                }
                if c.composicion != 0 && self.edad == c.composicion {
                    self.oido.indice_composicion = Some(self.oido.aciertos.len());
                    self.oido.probar_composicion(m, c);
                }
                let pedido = match &mut self.habla {
                    Some(h) => h.decidir(necesidad, &vista, self.edad, &self.cuerpo.v, c),
                    None => None,
                };
                let frase = m.hablar(&self.mundo, self.x, self.y, necesidad, &vista, self.edad, pedido, c);
                self.oido.oir(frase, (self.x, self.y), m.objetivo, c);
                if let Some(h) = &mut self.habla {
                    h.escuchar(frase, &self.cuerpo.v, necesidad, pedido, c);
                }
            }
            let dicha = if c.lexico { self.oido.direccion_dicha(c) } else { 4 };
            let (dx, dy) = match &mut self.vieja {
                Some(v) => v.elegir(&vista, necesidad, c),
                None => {
                    let palabras = if c.frase { self.oido.palabras() } else { vec![] };
                    self.mente.elegir(&sentida, &self.cuerpo.v, necesidad, (self.x, self.y),
                                      self.quimica.noradrenalina, self.malestar, &palabras, dicha, c)
                }
            };
            let (nx, ny) = match &self.mundo {
                Mundo::B(b) => {
                    let (nx, ny) = (self.x + dx, self.y + dy);
                    if b.transitable(nx, ny) { (nx, ny) } else { (self.x, self.y) }
                }
                Mundo::G(_) => ((self.x + dx).max(0).min(c.ancho - 1), (self.y + dy).max(0).min(c.alto - 1)),
            };
            if let Some(k) = &mut self.corteza {
                k.predecir(vista.get((dx, dy)).unwrap_or(ROCA), necesidad, (dx, dy) == (0, 0),
                           (self.x, self.y), c);
            }
            se_movio = (nx, ny) != (self.x, self.y);
            if c.lexico && se_movio {
                let d = match (dx, dy) { (0, -1) => 0, (0, 1) => 1, (-1, 0) => 2, _ => 3 };
                self.oido.caminar(d);
            }
            self.x = nx;
            self.y = ny;
            let aqui = self.mundo.en(self.x, self.y);
            if c.lexico {
                let v = self.mundo.vecindad(self.x, self.y, c);
                self.oido.ver(&v, (self.x, self.y), c);
            }
            sorpresa = match &mut self.vieja {
                Some(v) => v.sorpresa(aqui),
                None => self.mente.sorpresa(aqui),
            };
            encontrado = aqui;
            // (con Habla: lo que se le alivió al interactuar; solo en la grilla, como en Python)
            let antes: Option<Vec<f64>> = self.habla.as_ref().map(|_| {
                (0..self.cuerpo.n).map(|k| (self.cuerpo.v[k] - c.equilibrio[k]).abs()).collect()
            });
            if bosque {
                self.interactuar_bosque(aqui, se_movio, c);
            } else {
                self.interactuar(aqui);
            }
            if let (Some(h), Some(antes)) = (&mut self.habla, antes) {
                let mut n = 0;
                let mut bm = f64::NEG_INFINITY;
                for k in 0..self.cuerpo.n {
                    let b = antes[k] - (self.cuerpo.v[k] - c.equilibrio[k]).abs();
                    if b > bm {
                        bm = b;
                        n = k;
                    }
                }
                if bm > c.umbral_alivio && n < 5 {
                    h.aliviado(n, self.edad, c);
                }
            }
        }

        self.cuerpo.vivir(self.despierto, se_movio, c);
        if let Mundo::B(b) = &self.mundo {
            let aire = b.temperatura(self.x, self.y, c);
            self.cuerpo.clima(self.despierto, se_movio, aire, c);
            if self.cuerpo.v[INTEGRIDAD] <= 0.0 {
                self.desmayo(c);
            }
        }
        let antes = self.malestar;
        let malestar = self.cuerpo.malestar(c);
        self.malestar = malestar;
        if self.despierto && self.vieja.is_none() {
            let mut recompensa = c.recompensa_alivio * (antes - malestar) - c.recompensa_estado * malestar;
            if (encontrado == PELIGRO && (se_movio || !bosque)) || mordido {
                recompensa -= c.dolor;
            }
            if let Some(k) = &mut self.corteza {
                sorpresa = k.comparar(encontrado, malestar - antes, c);
                recompensa += c.peso_curiosidad * k.progreso;
            }
            let (_, sentida) = self.vista_sentida(c);
            let foco = match &self.talamo {
                Some(t) => t.foco.unwrap_or(0),
                None => self.cuerpo.necesidad_urgente(c),
            };
            self.mente.aprender(recompensa, &sentida, &self.cuerpo.v, foco, (self.x, self.y), c);
            if let Some(t) = &mut self.talamo {
                t.aprender(self.mente.dopamina, c);
            }
            if let Some(h) = &mut self.habla {
                h.aprender(self.mente.dopamina, c);
            }
            if let Some(h) = &mut self.hipocampo {
                if let Some(m) = self.mente.ultimo {
                    h.anotar(m, c);
                }
            }
        }
        self.quimica.actualizar(self.despierto, malestar, self.cuerpo.v[ESTRES], sorpresa, c);
        self.regular_sueno(c);
        self.edad += 1;
    }

    fn interactuar(&mut self, aqui: u8) {
        let Mundo::G(g) = &mut self.mundo else { unreachable!() };
        match aqui {
            COMIDA => {
                self.cuerpo.comer(0.25);
                g.consumir(self.x, self.y);
                self.evento = Evento::Come;
            }
            NOVEDAD => {
                self.cuerpo.descubrir();
                g.consumir(self.x, self.y);
                self.evento = Evento::Descubre;
            }
            PELIGRO => {
                self.cuerpo.lastimarse();
                self.evento = Evento::Lastima;
            }
            DESCANSO => self.cuerpo.descansar(),
            _ => {}
        }
    }

    fn interactuar_bosque(&mut self, aqui: u8, entra: bool, c: &Config) {
        let Mundo::B(b) = &mut self.mundo else { unreachable!() };
        match aqui {
            COMIDA => match b.trabajar(self.x, self.y, c) {
                None => self.evento = Evento::Otro,
                Some(t) => {
                    if t == BAYA && b.rng.random() < c.prob_enferma {
                        self.cuerpo.comer(0.1);
                        self.cuerpo.enfermar();
                        self.evento = Evento::Enferma;
                    } else {
                        self.cuerpo.comer(match t { FRUTA => 0.25, BAYA => 0.15, NUEZ => 0.35, _ => 0.3 });
                        self.evento = Evento::Come;
                    }
                }
            },
            AGUA => {
                self.cuerpo.beber(c);
                self.evento = Evento::Bebe;
            }
            NOVEDAD => {
                self.cuerpo.descubrir();
                b.consumir(self.x, self.y);
                self.evento = Evento::Descubre;
            }
            PELIGRO if entra => {
                self.cuerpo.rasparse();
                self.evento = Evento::Lastima;
            }
            DESCANSO => self.cuerpo.descansar(),
            _ => {}
        }
    }

    fn desmayo(&mut self, c: &Config) {
        let Mundo::B(b) = &self.mundo else { unreachable!() };
        let v = &mut self.cuerpo.v;
        let causa = if v[ENERGIA] < 0.05 { 0 } else if v[AGUA_N] < 0.05 { 1 } else if v[CALOR] < 0.3 { 2 } else { 3 };
        self.causas[causa] += 1;
        let (x, y) = (self.x, self.y);
        let elegir = |costo: &dyn Fn((i64, i64)) -> i64| {
            let mut m = b.cuevas[0];
            let mut cm = i64::MAX;
            for &q in &b.cuevas {
                let k = costo(q);
                if k < cm {
                    cm = k;
                    m = q;
                }
            }
            m
        };
        if c.desmayo_donde == "cueva" {
            let p = elegir(&|q: (i64, i64)| (q.0 - x).abs() + (q.1 - y).abs());
            self.x = p.0;
            self.y = p.1;
        } else if c.desmayo_donde == "cueva_agua" {
            let p = elegir(&|q: (i64, i64)| {
                let (ax, ay) = b.agua_de(q.0, q.1);
                (q.0 - x).abs() + (q.1 - y).abs() + (ax - q.0).abs() + (ay - q.1).abs()
            });
            self.x = p.0;
            self.y = p.1;
        }
        v[INTEGRIDAD] = v[INTEGRIDAD].max(c.desmayo_integridad);
        v[ENERGIA] = v[ENERGIA].max(c.desmayo_resto);
        v[AGUA_N] = v[AGUA_N].max(c.desmayo_resto);
        self.desmayos += 1;
        self.evento = Evento::Desmaya;
        if self.vieja.is_none() {
            self.mente.olvidar_paso();
        }
        self.malestar = self.cuerpo.malestar(c);
    }

    fn regular_sueno(&mut self, c: &Config) {
        let (mut dormir, mut despertar) = (c.umbral_dormir, c.umbral_despertar);
        if let Mundo::B(b) = &self.mundo {
            if b.noche(c) {
                dormir -= c.sueno_noche;
            } else {
                despertar += c.sueno_noche;
            }
        }
        let a = self.quimica.adenosina;
        if self.despierto && a >= dormir {
            self.despierto = false;
            self.suenos += 1;
            if self.vieja.is_none() {
                self.mente.olvidar_paso();
            }
            if let Some(t) = &mut self.talamo {
                t.olvidar_paso();
            }
            if let Some(h) = &mut self.hipocampo {
                h.dormir(&mut self.mente, c);
            }
            self.evento = Evento::Otro;
        } else if !self.despierto && a <= despertar {
            self.despierto = true;
            self.evento = Evento::Otro;
        }
    }

    pub fn malestar(&self, c: &Config) -> f64 {
        self.cuerpo.malestar(c)
    }
}

crate::pieza!(Angri { cuerpo, quimica, mente, vieja, corteza, hipocampo, talamo, maestro, habla, oido,
                     mundo, x, y, despierto, edad, suenos, desmayos, causas, malestar, evento });

impl crate::guardar::Pieza for Evento {
    fn guardar(&self, e: &mut crate::guardar::Escritor) {
        (*self as u8).guardar(e);
    }
    fn cargar(l: &mut crate::guardar::Lector) -> Self {
        use Evento::*;
        [Nada, Come, Descubre, Lastima, Muerde, Enferma, Bebe, Desmaya, Otro][u8::cargar(l) as usize]
    }
}

impl Angri {
    /// La mente entera como texto (para guardarla a disco).
    pub fn volcar(&self) -> String {
        use crate::guardar::*;
        let mut e = Escritor::new();
        e.tok(CABECERA);
        self.guardar(&mut e);
        e.s.push(char::from(10));
        e.s
    }

    pub fn desde_texto(texto: &str) -> Angri {
        use crate::guardar::*;
        let mut l = Lector::new(texto);
        for parte in CABECERA.split(' ') {
            assert_eq!(l.tok(), parte, "no es una mente de angri en Rust");
        }
        Angri::cargar(&mut l)
    }
}
