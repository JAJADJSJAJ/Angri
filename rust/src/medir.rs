//! Vara de medir (antes medir.py y comparar.py): vidas reproducibles sin disco,
//! repartidas en hilos. Misma semilla = misma vida exacta, así dos versiones se
//! comparan en igualdad de condiciones.
//!
//!   angri medir [TICKS] [N] [azar|reflejos] [CLAVE=VALOR ...] [-j J] [--config ARCHIVO]
//!     angri medir                      -> 50.000 ticks, semillas 0..4
//!     angri medir 50000 32 -j 12       -> 32 vidas en 12 hilos (mismo resultado)
//!     angri medir 50000 5 azar         -> mente vieja al azar puro (el piso); reflejos: con reflejos
//!     angri medir 50000 8 PESO_NECESIDAD.curiosidad=0 MUNDO=bosque   -> ablación
//!
//!   angri comparar TICKS N "AJUSTES A" "AJUSTES B" [-j J] [--config ARCHIVO]
//!     angri comparar 300000 256 "" "MUNDO=bosque" -j 12
//!   Comparación en pares: la misma semilla con A y con B. La suerte de cada vida
//!   se cancela en buena parte; muestra B − A con su error estándar
//!   (|dif| > 2 e.e. = diferencia real).
use crate::config::{armar, Config};
use std::sync::atomic::{AtomicUsize, Ordering};

type Metricas = Vec<(String, f64)>;

fn una(semilla: u64, ticks: i64, c: &Config) -> Metricas {
    crate::metricas(semilla, ticks, c).into_iter()
        .map(|(k, v)| (k, v.parse().unwrap_or(f64::NAN)))
        .collect()
}

/// Corre las vidas (config, semilla) en `hilos` hilos y las devuelve en orden.
fn vidas(tareas: &[(&Config, u64)], ticks: i64, hilos: usize) -> Vec<Metricas> {
    let siguiente = AtomicUsize::new(0);
    let mut out: Vec<Option<Metricas>> = (0..tareas.len()).map(|_| None).collect();
    let hechas = std::sync::Mutex::new(&mut out);
    std::thread::scope(|s| {
        for _ in 0..hilos.max(1).min(tareas.len().max(1)) {
            s.spawn(|| loop {
                let i = siguiente.fetch_add(1, Ordering::SeqCst);
                if i >= tareas.len() {
                    break;
                }
                let (c, semilla) = tareas[i];
                let m = una(semilla, ticks, c);
                hechas.lock().unwrap()[i] = Some(m);
            });
        }
    });
    out.into_iter().map(|m| m.unwrap()).collect()
}

fn media(xs: &[f64]) -> f64 {
    crate::mente::psum(xs.iter().copied()) / xs.len() as f64
}

fn pdesvio(xs: &[f64]) -> f64 {
    let m = media(xs);
    (crate::mente::psum(xs.iter().map(|x| (x - m) * (x - m))) / xs.len() as f64).sqrt()
}

fn desvio(xs: &[f64]) -> f64 {
    let m = media(xs);
    (crate::mente::psum(xs.iter().map(|x| (x - m) * (x - m))) / (xs.len() - 1) as f64).sqrt()
}

fn mediana(xs: &[f64]) -> f64 {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}

fn valor(m: &Metricas, k: &str) -> Option<f64> {
    m.iter().find(|e| e.0 == k).map(|e| e.1).filter(|x| x.is_finite())
}

/// Saca `-j J`, `--config ARCHIVO` y `--desde K` (primera semilla); devuelve (hilos, archivo, resto, desde).
fn opciones(args: &[String]) -> (usize, Option<String>, Vec<String>, u64) {
    let (mut hilos, mut archivo, mut resto, mut desde) = (1usize, None, vec![], 0u64);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-j" => {
                hilos = args.get(i + 1).and_then(|x| x.parse().ok()).expect("-j J");
                i += 1;
            }
            "--config" => {
                archivo = Some(args.get(i + 1).expect("--config ARCHIVO").clone());
                i += 1;
            }
            "--desde" => {
                desde = args.get(i + 1).and_then(|x| x.parse().ok()).expect("--desde K");
                i += 1;
            }
            _ => resto.push(args[i].clone()),
        }
        i += 1;
    }
    (hilos, archivo, resto, desde)
}

pub fn medir(args: &[String]) {
    let (hilos, archivo, resto, desde) = opciones(args);
    let mut ajustes: Vec<String> = resto.iter().filter(|a| a.contains('=')).cloned().collect();
    let modo = resto.iter().find(|a| *a == "azar" || *a == "reflejos").cloned();
    if let Some(m) = &modo {
        ajustes.push("MENTE=azar".into());
        ajustes.push(format!("REFLEJOS={}", if m == "reflejos" { "True" } else { "False" }));
    }
    let nums: Vec<&String> = resto.iter().filter(|a| !a.contains('=') && Some(*a) != modo.as_ref()).collect();
    let ticks: i64 = nums.first().map_or(50_000, |x| x.parse().expect("TICKS"));
    let n: u64 = nums.get(1).map_or(5, |x| x.parse().expect("N"));
    let (_, c) = armar(archivo.as_deref(), &ajustes);
    let tareas: Vec<(&Config, u64)> = (desde..desde + n).map(|s| (&c, s)).collect();
    let vs = vidas(&tareas, ticks, hilos);
    let mente = if c.mente_azar { if c.reflejos { "reflejos" } else { "azar puro" } } else { "aprende" };
    println!("{n} vidas de {ticks} ticks, mente: {mente}\n");
    // e.e. = error estándar del promedio (desvío/√n): diferencias menores a ~2 e.e. son ruido
    println!("{:<10} {:>10} {:>9} {:>8} {:>9}", "métrica", "promedio", "desvío", "e.e.", "mediana");
    let mas_larga = vs.iter().max_by_key(|v| v.len()).unwrap();
    for (k, _) in mas_larga {
        let xs: Vec<f64> = vs.iter().filter_map(|v| valor(v, k)).collect();
        if xs.is_empty() {
            continue;
        }
        let d = pdesvio(&xs);
        println!("{:<10} {:>10.3} {:>9.3} {:>8.3} {:>9.3}", k, media(&xs), d, d / (xs.len() as f64).sqrt(), mediana(&xs));
    }
}

pub fn comparar(args: &[String]) {
    let (hilos, archivo, resto, desde) = opciones(args);
    if resto.len() != 4 {
        eprintln!("uso: angri comparar TICKS N \"AJUSTES A\" \"AJUSTES B\" [-j J] [--config ARCHIVO] [--desde K]");
        std::process::exit(1);
    }
    let ticks: i64 = resto[0].parse().expect("TICKS");
    let n: u64 = resto[1].parse().expect("N");
    let aj = |t: &str| t.split_whitespace().map(String::from).collect::<Vec<_>>();
    let (_, ca) = armar(archivo.as_deref(), &aj(&resto[2]));
    let (_, cb) = armar(archivo.as_deref(), &aj(&resto[3]));
    let tareas: Vec<(&Config, u64)> = [&ca, &cb].iter().flat_map(|c| (desde..desde + n).map(move |s| (*c, s))).collect();
    let vs = vidas(&tareas, ticks, hilos);
    let (va, vb) = vs.split_at(n as usize);
    let nombre = |t: &str| if t.is_empty() { "(config actual)".to_string() } else { t.to_string() };
    println!("{n} pares de {ticks} ticks\nA: {}\nB: {}\n", nombre(&resto[2]), resto[3]);
    println!("{:<12} {:>9} {:>9} {:>9} {:>8} {:>6}", "métrica", "A", "B", "B−A", "e.e.", "z");
    for (k, _) in &va[0] {
        let pares: Vec<(f64, f64)> = va.iter().zip(vb)
            .filter_map(|(x, y)| Some((valor(x, k)?, valor(y, k)?))).collect();
        if pares.len() < 2 {
            continue;
        }
        let d: Vec<f64> = pares.iter().map(|(x, y)| y - x).collect();
        let ee = desvio(&d) / (d.len() as f64).sqrt();
        let z = if ee != 0.0 { media(&d) / ee } else { 0.0 };
        let xa: Vec<f64> = pares.iter().map(|p| p.0).collect();
        let xb: Vec<f64> = pares.iter().map(|p| p.1).collect();
        println!("{:<12} {:>9.3} {:>9.3} {:>9.3} {:>8.3} {:>6.1}", k, media(&xa), media(&xb), media(&d), ee, z);
    }
}
