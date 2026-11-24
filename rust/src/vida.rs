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
    // (agregados después de Otro, así las mentes guardadas antes cargan igual)
    Trabaja,
    Duerme,
    Despierta,
    Nace,
    Apagado,
    // etapa 13b
    Hace,   // hizo algo con un objeto
    Prende, // prendió fuego
    Quema,  // pisó un fuego
}

/// contadores de lo que hizo con objetos (para medir): agarra, suelta, apila, viga, ata,
/// rompe, abre a golpes, frota, prende, se quema
pub const HECHOS: [&str; 10] = ["agarra", "suelta", "apila", "vigas", "ata", "rompe", "abre golpe", "frota", "prende", "quema"];

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
    // etapa 13b
    pub lleva: Option<Objeto>,
    pub mira: usize, // hacia dónde se movió la última vez (0..4 de MOVIMIENTOS)
    pub hechos: [i64; 10],
}

impl Angri {
    pub fn new(semilla: u64, c: &Config) -> Angri {
        let bosque = c.bosque;
        let cuerpo = Cuerpo::new(bosque, c);
        let aprende = !c.mente_azar;
        let mut mente = Ganglios::new(semilla);
        if c.propia {
            mente.propia = Some(crate::propia::Propia::new(semilla, c));
        }
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
            causas: [0; 4], malestar, evento: Evento::Nace,
            lleva: None, mira: 1, hechos: [0; 10],
        }
    }

    fn tacto(&self, c: &Config) -> Option<Tacto> {
        match &self.mundo {
            Mundo::B(b) if c.objetos => Some(b.tacto(self.x, self.y, self.lleva, c)),
            _ => None,
        }
    }

    /// bit 0: noche; bit 1: lluvia
    fn clima(&self, c: &Config) -> u8 {
        match &self.mundo {
            Mundo::B(b) => b.noche(c) as u8 | ((b.lluvia != 0) as u8) << 1,
            Mundo::G(_) => 0,
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
                let refugio = b.refugio(self.x, self.y);
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
        let mut quemado = false;
        let mut esfuerzo = 0.0;

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
            let tacto = self.tacto(c);
            let clima = self.clima(c);
            let accion = match &mut self.vieja {
                Some(v) => {
                    let m = v.elegir(&vista, necesidad, c);
                    MOVIMIENTOS.iter().position(|&x| x == m).unwrap()
                }
                None => {
                    let palabras = if c.frase { self.oido.palabras() } else { vec![] };
                    self.mente.elegir(&sentida, &self.cuerpo.v, necesidad, (self.x, self.y),
                                      self.quimica.noradrenalina, self.malestar, &palabras, dicha,
                                      tacto.as_ref(), clima, self.cuerpo.n, c)
                }
            };
            let (dx, dy) = if accion < 5 { MOVIMIENTOS[accion] } else { (0, 0) };
            if accion < 4 {
                self.mira = accion;
            }
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
            if accion >= 5 {
                esfuerzo = 0.5;
                self.hacer(accion, c);
            }
            if let Mundo::B(b) = &self.mundo {
                if b.objetos && b.fuego_en(self.x, self.y) {
                    self.cuerpo.rasparse();
                    self.evento = Evento::Quema;
                    self.hechos[9] += 1;
                    quemado = true;
                }
            }
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
        if c.objetos {
            // llevar cosas pesadas cansa; hacer algo con las manos también
            let carga = if se_movio { self.lleva.map_or(0.0, |o| o.peso) } else { 0.0 };
            let v = &mut self.cuerpo.v[ENERGIA];
            *v = (*v - c.gasto_energia_mover * (carga + esfuerzo)).max(0.0);
        }
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
            if (encontrado == PELIGRO && (se_movio || !bosque)) || mordido || quemado {
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
            let tacto = self.tacto(c);
            let clima = self.clima(c);
            self.mente.aprender(recompensa, &sentida, &self.cuerpo.v, foco, (self.x, self.y), tacto.as_ref(),
                                clima, self.cuerpo.n, c);
            if let Some(t) = &mut self.talamo {
                t.aprender(self.mente.dopamina, c);
            }
            if let Some(h) = &mut self.habla {
                h.aprender(self.mente.dopamina, c);
            }
            if let Some(h) = &mut self.hipocampo {
                if let Some(m) = self.mente.ultimo {
                    h.anotar(m, self.mente.ultimo_sentir.take(), c);
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

    /// (etapa 13b) agarrar, soltar, apilar, atar, golpear o frotar, acá
    fn hacer(&mut self, accion: usize, c: &Config) {
        let Mundo::B(b) = &mut self.mundo else { return };
        let (x, y) = (self.x, self.y);
        let hecho = match accion {
            AGARRAR if self.lleva.is_none() => match b.agarrar(x, y, c) {
                Some(o) => {
                    self.lleva = Some(o);
                    Hecho::Agarra
                }
                None => Hecho::Nada,
            },
            SOLTAR => match self.lleva.take() {
                Some(o) => b.poner(x, y, o, 4),
                None => Hecho::Nada,
            },
            APILAR => match self.lleva.take() {
                Some(o) => {
                    let (dx, dy) = MOVIMIENTOS[self.mira];
                    b.poner(x + dx, y + dy, o, self.mira)
                }
                None => Hecho::Nada,
            },
            ATAR => match self.lleva {
                Some(o) if b.atar(x, y, &o) => {
                    self.lleva = None;
                    Hecho::Ata
                }
                _ => Hecho::Nada,
            },
            GOLPEAR => b.golpear(x, y, self.lleva.as_ref(), c),
            FROTAR => b.frotar(x, y, self.lleva.as_ref(), c),
            _ => Hecho::Nada,
        };
        let k = match hecho {
            Hecho::Agarra => Some(0),
            Hecho::Suelta | Hecho::Hunde => Some(1),
            Hecho::Apila | Hecho::Cae => Some(2),
            Hecho::Viga => Some(3),
            Hecho::Ata => Some(4),
            Hecho::Rompe => Some(5),
            Hecho::Abre(_) => Some(6),
            Hecho::Frota => Some(7),
            Hecho::Prende => Some(8),
            Hecho::Nada | Hecho::Golpea => None,
        };
        if let Some(k) = k {
            self.hechos[k] += 1;
            if hecho == Hecho::Viga {
                self.hechos[2] += 1;
            }
            self.evento = if hecho == Hecho::Prende { Evento::Prende } else { Evento::Hace };
        }
        if let Hecho::Abre(t) = hecho {
            self.comer_tipo(t, c);
        }
    }

    fn comer_tipo(&mut self, t: u8, c: &Config) {
        let Mundo::B(b) = &mut self.mundo else { unreachable!() };
        if t == BAYA && b.rng.random() < c.prob_enferma {
            self.cuerpo.comer(0.1);
            self.cuerpo.enfermar();
            self.evento = Evento::Enferma;
        } else {
            self.cuerpo.comer(match t { FRUTA => 0.25, BAYA => 0.15, NUEZ => 0.35, _ => 0.3 });
            self.evento = Evento::Come;
        }
    }

    fn interactuar_bosque(&mut self, aqui: u8, entra: bool, c: &Config) {
        let Mundo::B(b) = &mut self.mundo else { unreachable!() };
        match aqui {
            COMIDA => match b.trabajar(self.x, self.y, c) {
                None => self.evento = Evento::Trabaja,
                Some(t) => self.comer_tipo(t, c),
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
        if let (Some(o), Mundo::B(b)) = (self.lleva.take(), &mut self.mundo) {
            b.tirar(self.x, self.y, o);
        }
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
            self.evento = Evento::Duerme;
        } else if !self.despierto && a <= despertar {
            self.despierto = true;
            self.evento = Evento::Despierta;
        }
    }

    pub fn malestar(&self, c: &Config) -> f64 {
        self.cuerpo.malestar(c)
    }
}

crate::pieza!(Angri { cuerpo, quimica, mente, vieja, corteza, hipocampo, talamo, maestro, habla, oido,
                     mundo, x, y, despierto, edad, suenos, desmayos, causas, malestar, evento,
                     lleva, mira, hechos });

impl crate::guardar::Pieza for Evento {
    fn guardar(&self, e: &mut crate::guardar::Escritor) {
        (*self as u8).guardar(e);
    }
    fn cargar(l: &mut crate::guardar::Lector) -> Self {
        use Evento::*;
        [Nada, Come, Descubre, Lastima, Muerde, Enferma, Bebe, Desmaya, Otro, Trabaja, Duerme, Despierta, Nace, Apagado, Hace, Prende, Quema][u8::cargar(l) as usize]
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
