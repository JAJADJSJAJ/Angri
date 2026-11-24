//! Núcleo de Angri en Rust: corre una vida y devuelve las métricas de
//! medir.una_vida en JSON. La config llega por stdin (la arma medir.py --rust).
//!
//!   angri vida SEMILLA TICKS   < config  -> métricas en JSON (una línea)
//!   angri traza SEMILLA TICKS  < config  -> una línea por tick (para comparar con Python)
//!   angri velocidad SEMILLA TICKS < config -> ticks/s
//!   angri retomar SEMILLA TICKS < config  -> cortar, guardar, cargar y seguir da IDÉNTICO
//!   angri vivir [--carpeta DIR] ...        -> vida continua (ver vivir.rs)
mod azar;
mod config;
mod extra;
mod guardar;
mod lenguaje;
mod mente;
mod mundo;
mod vida;
mod vivir;

use config::Config;
use mente::*;
use mundo::*;
use std::io::{Read, Write};
use vida::{Angri, Evento};

fn num(x: f64) -> String {
    if x.is_finite() { format!("{:?}", x) } else { "null".into() }
}

fn media_bool(xs: &[bool]) -> f64 {
    xs.iter().filter(|&&b| b).count() as f64 / xs.len() as f64
}

fn media_int(xs: &[i64]) -> f64 {
    xs.iter().sum::<i64>() as f64 / xs.len() as f64
}

fn una_vida(semilla: u64, ticks: i64, c: &Config) -> String {
    let mut a = Angri::new(semilla, c);
    let corte = (ticks as f64 * 0.8) as i64;
    let (mut malestar, mut fin) = (0.0f64, 0.0f64);
    let (mut heridas, mut comidas, mut novedades, mut hambre, mut alto) = (0i64, 0i64, 0i64, 0i64, 0i64);
    let bosque = c.bosque;
    let (mut sed, mut frio, mut mordidas, mut enfermo, mut bebidas, mut dormido, mut refugio) = (0i64, 0i64, 0i64, 0i64, 0i64, 0i64, 0i64);
    let (mut racha, mut racha_max) = (0i64, 0i64);
    let mut plantada = 0i64;
    let (mut cerca_pred, mut huye) = (0i64, 0i64);
    let mut anio = 0.0f64;
    let mut por_anio: Vec<f64> = vec![];
    let largo_anio = 4 * c.estacion_bosque;
    let (mut tardanzas, mut rondas): (Vec<i64>, Vec<i64>) = (vec![], vec![]);
    // [nueva, vieja, ticks desde el cambio, ticks cerca de la vieja]
    let mut pendiente: Option<((i64, i64), (i64, i64), i64, i64)> = None;
    let ventana = if c.estacion != 0 { c.estacion / 2 } else { 0 };
    let r = c.radio_fuente;
    for t in 0..ticks {
        a.evento = Evento::Nada;
        let cambios = match &a.mundo { Mundo::G(g) => g.cambios, _ => 0 };
        let mut d0 = 99;
        if let Mundo::B(b) = &a.mundo {
            let (px, py) = b.predador;
            d0 = if b.harto == 0 { (px - a.x).abs() + (py - a.y).abs() } else { 99 };
        }
        a.tick(c);
        let cerca = |a: &Angri, p: (i64, i64)| (a.x - p.0).abs() <= r && (a.y - p.1).abs() <= r;
        if let Mundo::G(g) = &a.mundo {
            if g.cambios != cambios {
                if let Some(p) = pendiente {
                    tardanzas.push(ventana);
                    rondas.push(p.3);
                }
                let (vieja, nueva) = g.cambio.unwrap();
                pendiente = Some((nueva, vieja, 0, 0));
            }
        }
        if let Some(mut p) = pendiente {
            p.2 += 1;
            p.3 += (cerca(&a, p.1) && a.despierto) as i64;
            pendiente = Some(p);
            if a.evento == Evento::Come && cerca(&a, p.0) {
                tardanzas.push(p.2);
                rondas.push(p.3);
                pendiente = None;
            } else if p.2 >= ventana {
                tardanzas.push(ventana);
                rondas.push(p.3);
                pendiente = None;
            }
        }
        let m = a.malestar(c);
        malestar += m;
        if t >= corte {
            fin += m;
        }
        heridas += (a.evento == Evento::Lastima) as i64;
        comidas += (a.evento == Evento::Come) as i64;
        novedades += (a.evento == Evento::Descubre) as i64;
        hambre += (a.cuerpo.v[ENERGIA] < 0.3) as i64;
        alto += (m > c.malestar_alto) as i64;
        racha = if m > c.malestar_alto { racha + 1 } else { 0 };
        racha_max = racha_max.max(racha);
        if let Mundo::B(b) = &a.mundo {
            let v = &a.cuerpo.v;
            anio += m;
            if (t + 1) % largo_anio == 0 {
                por_anio.push(anio / largo_anio as f64);
                anio = 0.0;
            }
            if a.despierto && b.en(a.x, a.y) == AGUA && v[AGUA_N] > 0.9 {
                plantada += 1;
            }
            if d0 <= 5 && a.despierto && a.evento != Evento::Desmaya {
                let (px, py) = b.predador;
                cerca_pred += 1;
                huye += ((px - a.x).abs() + (py - a.y).abs() > d0) as i64;
            }
            sed += (v[AGUA_N] < 0.3) as i64;
            frio += (v[CALOR] < 0.4) as i64;
            mordidas += (a.evento == Evento::Muerde) as i64;
            enfermo += (a.evento == Evento::Enferma) as i64;
            bebidas += (a.evento == Evento::Bebe) as i64;
            if !a.despierto {
                dormido += 1;
                refugio += (b.zona_en(a.x, a.y) == Z_CUEVA) as i64;
            }
        }
    }
    let tk = ticks as f64;
    let mut o: Vec<(String, String)> = vec![];
    let mut f = |k: &str, v: String| o.push((k.to_string(), v));
    f("malestar", num(malestar / tk));
    f("mal. final", num(fin / (ticks - corte) as f64));
    f("heridas", heridas.to_string());
    f("comidas", comidas.to_string());
    f("novedades", novedades.to_string());
    f("suenos", a.suenos.to_string());
    f("% hambre", num((100 * hambre) as f64 / tk));
    f("% trabadas", num(100.0 * if hambre as f64 / tk > 0.6 { 1.0 } else { 0.0 }));
    f("% mal alto", num((100 * alto) as f64 / tk));
    f("racha alta", racha_max.to_string());
    if bosque {
        f("% sed", num((100 * sed) as f64 / tk));
        f("% frío", num((100 * frio) as f64 / tk));
        f("mordidas", mordidas.to_string());
        f("enfermó", enfermo.to_string());
        f("bebe", bebidas.to_string());
        f("% duerme cueva", num((100 * refugio) as f64 / dormido.max(1) as f64));
        f("desmayos", a.desmayos.to_string());
        for (i, k) in ["hambre", "sed", "frio", "heridas"].iter().enumerate() {
            f(&format!("desm. {k}"), a.causas[i].to_string());
        }
        f("% plantada", num((100 * plantada) as f64 / tk));
        f("% huye", num(if cerca_pred != 0 { (100 * huye) as f64 / cerca_pred as f64 } else { 0.0 }));
        for (i, x) in por_anio.iter().enumerate() {
            f(&format!("mal. año {}", i + 1), num(*x));
        }
    } else {
        f("encuentra", num(if tardanzas.is_empty() { 0.0 } else { media_int(&tardanzas) }));
        f("ronda vieja", num(if rondas.is_empty() { 0.0 } else { media_int(&rondas) }));
    }
    let ac = &a.oido.aciertos;
    if !ac.is_empty() {
        let n = ac.len();
        let tramo: Vec<bool> = ac[(n as f64 * 0.8) as usize..].iter().map(|x| x.0).collect();
        let nuevas: Vec<bool> = ac[n / 5..].iter().filter(|x| x.1).map(|x| x.0).collect();
        if let Some(h) = &a.habla {
            if !h.pedidos.is_empty() {
                let ped = &h.pedidos;
                let mut tramo_p: Vec<bool> = ped.iter().filter(|p| p.1 as f64 >= tk * 0.8).map(|p| p.0).collect();
                if tramo_p.is_empty() {
                    tramo_p = ped.iter().map(|p| p.0).collect();
                }
                f("pide bien %", num(100.0 * media_bool(&tramo_p)));
                f("pedidos", ped.len().to_string());
            }
        }
        f("entiende %", num(100.0 * media_bool(&tramo)));
        f("confianza", num(a.oido.confianza));
        if !nuevas.is_empty() {
            f("frase nueva %", num(100.0 * media_bool(&nuevas)));
        }
        if c.composicion != 0 {
            if let Some(ic) = a.oido.indice_composicion {
                let nuevas: Vec<bool> = ac[ic..].iter().filter(|x| x.1).map(|x| x.0).take(20).collect();
                f("examen %", num(100.0 * media_bool(&a.oido.examen)));
                f("compone n", nuevas.len().to_string());
                f("compone ok", nuevas.iter().filter(|&&b| b).count().to_string());
            }
        }
        if c.cambio_significado != 0 {
            if let Some(i) = a.oido.indice_cambio {
                let tramo: Vec<bool> = ac[i.min(n)..(i + 100).min(n)].iter().map(|x| x.0).collect();
                f("reaprende %", num(if tramo.is_empty() { 0.0 } else { 100.0 * media_bool(&tramo) }));
            }
        }
    }
    let partes: Vec<String> = o.iter().map(|(k, v)| format!("\"{}\": {}", k, v)).collect();
    format!("{{{}}}", partes.join(", "))
}

fn traza(semilla: u64, ticks: i64, c: &Config) {
    let mut a = Angri::new(semilla, c);
    let out = std::io::stdout();
    let mut out = std::io::BufWriter::new(out.lock());
    for _ in 0..ticks {
        a.tick(c);
        let mut l = format!("{} {} {} {} {:016x}", a.edad, a.x, a.y, a.despierto as i32, a.malestar(c).to_bits());
        for k in 0..a.cuerpo.n {
            l += &format!(" {:016x}", a.cuerpo.v[k].to_bits());
        }
        l += &format!(" {:016x} {:016x}", a.quimica.adenosina.to_bits(), a.mente.dopamina.to_bits());
        l += &format!(" {:016x} {:016x}", a.quimica.noradrenalina.to_bits(), a.oido.confianza.to_bits());
        let foco = a.talamo.as_ref().and_then(|t| t.foco).map_or(-1, |f| f as i64);
        let pedidos = a.habla.as_ref().map_or(0, |h| h.pedidos.len());
        l += &format!(" {} {} {}", foco, pedidos, a.oido.aciertos.len());
        writeln!(out, "{l}").unwrap();
    }
}

/// Prueba de retomar (como probar_retomar.py): una vida de un tirón contra la
/// misma vida cortada a la mitad, guardada a disco, cargada y seguida.
fn retomar(semilla: u64, ticks: i64, c: &Config) -> bool {
    let mut a = Angri::new(semilla, c);
    for _ in 0..ticks {
        a.tick(c);
    }
    let mut b = Angri::new(semilla, c);
    for _ in 0..ticks / 2 {
        b.tick(c);
    }
    let ruta = std::env::temp_dir().join(format!("angri-retomar-{}-{semilla}.txt", std::process::id()));
    guardar::escribir_archivo(&ruta, &b.volcar()).expect("no pude guardar");
    drop(b);
    let texto = std::fs::read_to_string(&ruta).expect("no pude leer");
    let _ = std::fs::remove_file(&ruta);
    let mut cc = Angri::desde_texto(&texto);
    for _ in 0..ticks - ticks / 2 {
        cc.tick(c);
    }
    let (ja, jc) = (a.volcar(), cc.volcar());
    println!("{} ticks, semilla {}: edad {}/{}, malestar {:.6}/{:.6}, mente {} bytes",
             ticks, semilla, a.edad, cc.edad, a.malestar(c), cc.malestar(c), ja.len());
    let igual = ja == jc;
    println!("{}", if igual { "IDÉNTICO" } else { "DISTINTO" });
    igual
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("vivir") {
        return vivir::vivir(&args[2..]);
    }
    if args.len() < 4 {
        eprintln!("uso: angri vida|traza|velocidad|retomar SEMILLA TICKS  < config");
        eprintln!("     angri vivir [--carpeta DIR] [--ticks N] [--semilla S] [--ventana W] [--guardar G]");
        std::process::exit(1);
    }
    let semilla: u64 = args[2].parse().expect("semilla");
    let ticks: i64 = args[3].parse().expect("ticks");
    let mut texto = String::new();
    std::io::stdin().read_to_string(&mut texto).unwrap();
    let c = Config::leer(&texto);
    match args[1].as_str() {
        "vida" => println!("{}", una_vida(semilla, ticks, &c)),
        "traza" => traza(semilla, ticks, &c),
        "retomar" => {
            if !retomar(semilla, ticks, &c) {
                std::process::exit(1);
            }
        }
        "velocidad" => {
            let t0 = std::time::Instant::now();
            let mut a = Angri::new(semilla, &c);
            for _ in 0..ticks {
                a.tick(&c);
            }
            let s = t0.elapsed().as_secs_f64();
            println!("{} ticks en {:.3} s: {:.0} ticks/s (malestar final {:.6})", ticks, s, ticks as f64 / s, a.malestar(&c));
        }
        m => {
            eprintln!("modo desconocido: {m}");
            std::process::exit(1);
        }
    }
}
