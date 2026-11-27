//! Pruebas propias de Rust (paso 2): no necesitan Python.
//!
//!   cd rust && cargo test --release
//!
//! - azar: PCG32 y splitmix64 dan los mismos números que angri/azar.py (valores
//!   anotados desde Python).
//! - determinismo: misma semilla y config = misma vida dos veces.
//! - retomar: cortar, guardar a disco, cargar y seguir = no cortar (grilla y
//!   bosque, varias semillas y cortes; corteza/tálamo, PIDE, mente vieja, todo).
//! - referencia: la vida de hoy (cuando Rust = Python, probado con
//!   probar_rust.py) quedó anotada en rust/pruebas/referencia.txt como huellas
//!   de la traza tick a tick, de la foto final y las métricas. Si un cambio la
//!   mueve, la prueba falla. Si el cambio es a propósito (y está igual en
//!   Python, con probar_rust.py en IDÉNTICO), se regenera con
//!       ANGRI_REGENERAR=1 cargo test --release referencia
//!   y se commitea el referencia.txt nuevo.
//!
//! Las configs (rust/pruebas/config-*.txt) son copias de angri/config.py con
//! ajustes; las arma rust/pruebas/hacer_configs.py.
use crate::azar::{splitmix64, Azar};
use crate::config::Config;
use crate::vida::{Angri, Evento};

const CONFIGS: [&str; 10] = ["grilla", "bosque", "corteza-talamo", "corteza-talamo-bosque", "pide", "azar", "todo", "objetos", "propios", "freno"];

fn carpeta() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pruebas")
}

fn config(nombre: &str) -> Config {
    let t = std::fs::read_to_string(carpeta().join(format!("config-{nombre}.txt"))).expect("config de prueba");
    Config::leer(&t)
}

/// FNV-1a de 64 bits (chico, sin bibliotecas; no es para seguridad).
struct Huella(u64);

impl Huella {
    fn new() -> Huella {
        Huella(0xcbf29ce484222325)
    }
    fn mas(&mut self, s: &str) {
        for b in s.bytes().chain(std::iter::once(b'\n')) {
            self.0 = (self.0 ^ b as u64).wrapping_mul(0x100000001b3);
        }
    }
}

fn huella(s: &str) -> u64 {
    let mut h = Huella::new();
    h.mas(s);
    h.0
}

// ------------------------------------------------------------------ azar
#[test]
fn azar_splitmix64() {
    // de angri/azar.py: splitmix64(x) -> (x nuevo, salida)
    assert_eq!(splitmix64(0), (0x9e3779b97f4a7c15, 0xe220a8397b1dcdaf));
    assert_eq!(splitmix64(1), (0x9e3779b97f4a7c16, 0x910a2dec89025cc1));
    assert_eq!(splitmix64(12345), (0x9e3779b97f4aac4e, 0x22118258a9d111a0));
}

#[test]
fn azar_pcg32() {
    // de angri/azar.py: a = Azar(42); a._u32() x5; a.random() x3; a.randrange(0, 10) x8; ...
    let mut a = Azar::new(42);
    let u: Vec<u32> = (0..5).map(|_| a.u32()).collect();
    assert_eq!(u, [3508393247, 2846903365, 3050928809, 2850731726, 4131377665]);
    let r: Vec<f64> = (0..3).map(|_| a.random()).collect();
    assert_eq!(r, [0.6154775608190679, 0.944290153590829, 0.5396930408092822]);
    let rr: Vec<i64> = (0..8).map(|_| a.randrange(0, 10)).collect();
    assert_eq!(rr, [5, 6, 7, 2, 1, 3, 9, 5]);
    assert_eq!(a.randint(1, 6), 3);
    assert_eq!(a.choice(&[5, 6, 7]), 6);
    // b = Azar(7); shuffle(range(10)); sample(range(20), 4); choices([0,1,2], [.2,.5,.3], k=5)
    let mut b = Azar::new(7);
    let mut x: Vec<i64> = (0..10).collect();
    b.shuffle(&mut x);
    assert_eq!(x, [6, 1, 3, 2, 7, 4, 8, 0, 9, 5]);
    assert_eq!(b.sample(&(0..20).collect::<Vec<i64>>(), 4), [6, 4, 11, 15]);
    assert_eq!(b.choices_pesos(&[0.2, 0.5, 0.3], 5), [1, 1, 0, 1, 2]);
    assert_eq!(Azar::new(0).random(), 0.5640298223034855);
    assert_eq!(Azar::new(u64::MAX).random(), 0.30456530641542434);
}

// ------------------------------------------------------------------ vidas
/// Vive `ticks` ticks y suma cada renglón de la traza (lo que compara probar_rust.py) a la huella.
fn vivir(a: &mut Angri, c: &Config, ticks: i64, h: &mut Huella) {
    for _ in 0..ticks {
        a.evento = Evento::Nada;
        a.tick(c);
        h.mas(&crate::linea_traza(a, c));
    }
}

fn traza(a: &mut Angri, c: &Config, ticks: i64) -> u64 {
    let mut h = Huella::new();
    vivir(a, c, ticks, &mut h);
    h.0
}

#[test]
fn determinismo() {
    for nombre in CONFIGS {
        let c = config(nombre);
        for semilla in [0, 5] {
            let (mut a, mut b) = (Angri::new(semilla, &c), Angri::new(semilla, &c));
            assert_eq!(traza(&mut a, &c, 6000), traza(&mut b, &c, 6000), "{nombre} semilla {semilla}");
            assert!(a.volcar() == b.volcar(), "{nombre} semilla {semilla}: la mente quedó distinta");
            assert_eq!(crate::una_vida(semilla, 3000, &c), crate::una_vida(semilla, 3000, &c));
        }
        // y otra semilla da otra vida (si no, la prueba no prueba nada)
        let (mut a, mut b) = (Angri::new(0, &c), Angri::new(1, &c));
        assert_ne!(traza(&mut a, &c, 2000), traza(&mut b, &c, 2000), "{nombre}: semillas 0 y 1 iguales");
    }
}

#[test]
fn retomar() {
    for nombre in CONFIGS {
        let c = config(nombre);
        for (semilla, ticks, corte) in [(0, 12000, 6000), (1, 12000, 6000), (2, 9000, 1), (3, 9000, 7777)] {
            let mut a = Angri::new(semilla, &c);
            let ha = traza(&mut a, &c, ticks);
            let mut b = Angri::new(semilla, &c);
            let mut h = Huella::new();
            vivir(&mut b, &c, corte, &mut h);
            // a disco y de vuelta, como `angri vivir`
            let ruta = std::env::temp_dir().join(format!("angri-prueba-{}-{nombre}-{semilla}.txt", std::process::id()));
            crate::guardar::escribir_archivo(&ruta, &b.volcar()).unwrap();
            drop(b);
            let texto = std::fs::read_to_string(&ruta).unwrap();
            let _ = std::fs::remove_file(&ruta);
            let mut b = Angri::desde_texto(&texto);
            vivir(&mut b, &c, ticks - corte, &mut h);
            assert_eq!(ha, h.0, "{nombre} semilla {semilla} cortada en {corte}: la traza cambió");
            assert!(a.volcar() == b.volcar(), "{nombre} semilla {semilla} cortada en {corte}: la mente quedó distinta");
            assert_eq!(crate::foto::foto(&a, &c), crate::foto::foto(&b, &c));
        }
    }
}

// ------------------------------------------------------------------ referencia
const TICKS_REF: i64 = 20000;
const SEMILLAS_REF: [u64; 3] = [0, 1, 2];

fn renglon_referencia(nombre: &str, semilla: u64, c: &Config) -> String {
    let mut a = Angri::new(semilla, c);
    let t = traza(&mut a, c, TICKS_REF);
    let foto = huella(&crate::foto::foto(&a, c));
    let metricas = crate::una_vida(semilla, TICKS_REF, c);
    format!("{nombre} {semilla} {TICKS_REF} traza={t:016x} foto={foto:016x} {metricas}")
}

#[test]
fn referencia() {
    let archivo = carpeta().join("referencia.txt");
    let mut nuevos = vec![];
    for nombre in CONFIGS {
        let c = config(nombre);
        for s in SEMILLAS_REF {
            nuevos.push(renglon_referencia(nombre, s, &c));
        }
    }
    if std::env::var("ANGRI_REGENERAR").is_ok() {
        let cab = "# Vida de referencia del núcleo en Rust (ver rust/src/pruebas.rs). Una línea por config y semilla:\n\
                   # config semilla ticks huella-de-la-traza huella-de-la-foto-final métricas (medir.una_vida).\n\
                   # Regenerar SOLO si el cambio es a propósito: ANGRI_REGENERAR=1 cargo test --release referencia\n";
        std::fs::write(&archivo, format!("{cab}{}\n", nuevos.join("\n"))).unwrap();
        eprintln!("referencia regenerada en {}", archivo.display());
        return;
    }
    let texto = std::fs::read_to_string(&archivo).expect("falta rust/pruebas/referencia.txt");
    let viejos: Vec<&str> = texto.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).collect();
    assert_eq!(viejos.len(), nuevos.len(), "la referencia tiene otra cantidad de vidas");
    let mal: Vec<String> = viejos.iter().zip(&nuevos).filter(|(v, n)| *v != n)
        .map(|(v, n)| format!("  antes: {v}\n  ahora: {n}")).collect();
    assert!(mal.is_empty(), "la vida cambió respecto de la referencia:\n{}\n\
            Si es a propósito: ANGRI_REGENERAR=1 cargo test --release referencia", mal.join("\n"));
}

// ------------------------------------------------------------------ objetos (13b)
/// Física de los objetos: un refugio hecho a mano (dos paredes, una viga encima) tapa la
/// lluvia, corta el viento y el predador no entra; una viga sin apoyo en la punta se cae.
#[test]
fn objetos_refugio() {
    use crate::mundo::*;
    let c = config("objetos");
    let mut b = Bosque::new(9, &c);
    for p in b.pilas.iter_mut() {
        p.clear();
    }
    // un lugar de llanura abierto: 5x5 libre de roca, agua y comida
    let libre = |b: &Bosque, x: i64, y: i64| b.en(x, y) == VACIO && b.zona_en(x, y) == b'l';
    let (x, y) = (2..b.w - 3).flat_map(|x| (2..b.h - 3).map(move |y| (x, y)))
        .find(|&(x, y)| (-2..=2).all(|dx| (-2..=2).all(|dy| libre(&b, x + dx, y + dy)))).expect("llanura");
    let piedra = Objeto { largo: 1, peso: 0.5, dureza: 0.9, flota: false, arde: false, flexible: false, eje: 0 };
    let rama = Objeto { largo: 3, peso: 0.2, dureza: 0.4, flota: true, arde: true, flexible: false, eje: 0 };
    let hoja = Objeto { largo: 1, peso: 0.02, dureza: 0.05, flota: true, arde: true, flexible: true, eje: 0 };
    assert!(b.puede_predador_pub(x, y));
    // viga larga sin apoyo en la punta: se cae
    for _ in 0..2 {
        assert_eq!(b.poner(x - 1, y, piedra, 4), Hecho::Suelta);
    }
    assert_eq!(b.poner(x - 1, y, rama, 3), Hecho::Cae);
    assert!(!b.cubierto(x, y));
    // con otra pared enfrente, la misma viga queda de techo
    for _ in 0..2 {
        b.poner(x + 1, y, piedra, 4);
    }
    assert_eq!(b.pilas[(y * b.w + x) as usize].len(), 1, "la viga caída quedó acá");
    b.pilas[(y * b.w + x) as usize].clear();
    assert_eq!(b.poner(x - 1, y, rama, 3), Hecho::Viga);
    assert!(b.cubierto(x - 1, y) && b.cubierto(x, y) && b.cubierto(x + 1, y));
    // hueco chico entre dos paredes: el predador no entra, Angri sí
    assert!(!b.puede_predador_pub(x, y));
    assert!(b.refugio(x, y));
    assert!(b.transitable(x, y));
    // de noche y con lluvia, adentro está más templado que afuera
    b.lluvia = 300;
    b.tiempo = c.dia - 1;
    assert!(b.noche(&c));
    let afuera = b.temperatura(x, y + 2, &c);
    let adentro = b.temperatura(x, y, &c);
    assert!(adentro > afuera + 0.1, "adentro {adentro} afuera {afuera}");
    // sacar la viga destapa
    assert_eq!(b.agarrar(x - 1, y, &c).map(|o| o.largo), Some(3));
    assert!(!b.cubierto(x, y));
    // algo pesado sobre una hoja no se sostiene
    let (hx, hy) = (x, y + 2);
    b.poner(hx, hy, hoja, 4);
    assert_eq!(b.poner(hx, hy, piedra, 4), Hecho::Cae);
    // una piedra que no flota se hunde en el agua
    let agua = (0..b.h).flat_map(|yy| (0..b.w).map(move |xx| (xx, yy))).find(|&(xx, yy)| b.en(xx, yy) == AGUA).unwrap();
    assert_eq!(b.poner(agua.0, agua.1, piedra, 4), Hecho::Hunde);
    assert_eq!(b.poner(agua.0, agua.1, rama, 4), Hecho::Suelta);
    // atar dos ramas con una liana: una sola, más larga
    let liana = Objeto { largo: 3, peso: 0.06, dureza: 0.1, flota: true, arde: false, flexible: true, eje: 0 };
    let (ax, ay) = (x, y - 2);
    let corta = Objeto { largo: 2, ..rama };
    b.poner(ax, ay, corta, 4);
    b.poner(ax, ay, corta, 4);
    assert!(b.atar(ax, ay, &liana));
    assert_eq!(b.pilas[(ay * b.w + ax) as usize].last().unwrap().largo, 4);
    // golpear con piedra la parte en dos
    assert_eq!(b.golpear(ax, ay, Some(&piedra), &c), Hecho::Rompe);
    assert_eq!(b.pilas[(ay * b.w + ax) as usize].len(), 2);
}
