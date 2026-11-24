//! Vida continua: vive sin fin, guarda la mente cada
//! tanto con respaldos rotativos y termina bien con Ctrl-C o SIGTERM.
//!
//!   angri vivir [--carpeta DIR] [--ticks N] [--semilla S] [--ventana W] [--guardar G]
//!               [--panel] [--foto S] [--estado] [--mundo grilla|bosque]
//!               [--config ARCHIVO] [CLAVE=VALOR ...]
//!
//! --panel: dibuja el panel en la terminal a TICKS_POR_SEGUNDO.
//! --foto S: cada S segundos (1 por defecto; 0 = nunca) escribe DIR/foto.json, que lee
//!   `angri visor --carpeta DIR`.
//! --estado: cómo anda (edad al guardar, respaldos, últimas ventanas) y sale.
//!
//! La config: DIR/config.txt. Si no existe (la vida nace), se arma con la de
//! adentro del binario (o --config) más --mundo y los CLAVE=VALOR, y se guarda
//! ahí: la vida sigue siempre con la misma config aunque cambie la de por defecto.
//! Si existe DIR/mente.txt la retoma exactamente donde estaba; si no carga, retoma
//! del respaldo válido más reciente (DIR/respaldos/: los últimos 5 y uno por día, 30 días).
//! Log liviano en DIR/vida.csv, un renglón por ventana.
use crate::config::{armar, Config};
use crate::foto;
use crate::guardar::escribir_archivo;
use crate::mente::ENERGIA;
use crate::vida::{Angri, Evento};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static PARAR: AtomicBool = AtomicBool::new(false);

extern "C" fn al_recibir(_: i32) {
    PARAR.store(true, Ordering::SeqCst);
}

extern "C" {
    // signal() de la biblioteca de C (está en Windows y en Linux)
    fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
}

fn escuchar_senales() {
    unsafe {
        signal(2, al_recibir); // SIGINT (Ctrl-C)
        signal(15, al_recibir); // SIGTERM (systemd)
        #[cfg(windows)]
        signal(21, al_recibir); // SIGBREAK (Ctrl-Break en Windows)
    }
}

const ULTIMOS: usize = 5;
const DIAS: usize = 30;

/// Fecha de hoy (UTC) como AAAA-MM-DD, sin bibliotecas.
fn hoy() -> String {
    let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let z = s.div_euclid(86400) + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

fn respaldar(carpeta: &Path, texto: &str, edad: i64) -> std::io::Result<()> {
    let r = carpeta.join("respaldos");
    std::fs::create_dir_all(&r)?;
    escribir_archivo(&r.join(format!("mente-{edad:012}.txt")), texto)?;
    escribir_archivo(&r.join(format!("dia-{}.txt", hoy())), texto)?;
    for (prefijo, quedan) in [("mente-", ULTIMOS), ("dia-", DIAS)] {
        let mut v: Vec<PathBuf> = std::fs::read_dir(&r)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with(prefijo)
                && p.extension().map_or(false, |x| x == "txt"))
            .collect();
        v.sort();
        if v.len() > quedan {
            for p in &v[..v.len() - quedan] {
                std::fs::remove_file(p)?;
            }
        }
    }
    Ok(())
}

fn cargar(ruta: &Path) -> Option<Angri> {
    let texto = std::fs::read_to_string(ruta).ok()?;
    std::panic::catch_unwind(|| Angri::desde_texto(&texto)).ok()
}

fn retomar(carpeta: &Path) -> Option<Angri> {
    let archivo = carpeta.join("mente.txt");
    if !archivo.exists() {
        return None;
    }
    if let Some(a) = cargar(&archivo) {
        return Some(a);
    }
    let mut copias: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(carpeta.join("respaldos"))
        .map(|d| d.filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |x| x == "txt"))
            .map(|e| (e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH), e.path()))
            .collect())
        .unwrap_or_default();
    copias.sort();
    for (_, c) in copias.iter().rev() {
        if let Some(a) = cargar(c) {
            println!("AVISO: {} no cargó; retoma del respaldo {} ({} ticks).", archivo.display(), c.display(), a.edad);
            return Some(a);
        }
    }
    eprintln!("{} no carga y no hay respaldo válido: no hago nacer otra Angri encima. Revisalo a mano.",
              archivo.display());
    std::process::exit(3);
}

pub fn vivir(args: &[String]) {
    let mut carpeta = PathBuf::from(std::env::var("ANGRI_DATOS").unwrap_or_else(|_| "datos-rust".into()));
    let (mut ticks, mut semilla, mut ventana, mut guardar_cada) = (None, None, 10_000i64, 50_000i64);
    let (mut panel, mut cada_foto, mut ver_estado) = (false, 1.0f64, false);
    let (mut ajustes, mut archivo_base): (Vec<String>, Option<String>) = (vec![], None);
    let mut i = 0;
    while i < args.len() {
        let v = args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--panel" | "--estado" => {
                if args[i] == "--panel" { panel = true } else { ver_estado = true }
                i += 1;
                continue;
            }
            "--foto" => cada_foto = v.parse().expect("--foto"),
            "--carpeta" => carpeta = PathBuf::from(v),
            "--ticks" => ticks = Some(v.parse::<i64>().expect("--ticks")),
            "--semilla" => semilla = Some(v.parse::<u64>().expect("--semilla")),
            "--ventana" => ventana = v.parse().expect("--ventana"),
            "--guardar" => guardar_cada = v.parse().expect("--guardar"),
            "--mundo" => ajustes.push(format!("MUNDO={v}")),
            "--config" => archivo_base = Some(v),
            o if o.contains('=') => {
                ajustes.push(o.to_string());
                i += 1;
                continue;
            }
            o => {
                eprintln!("opción desconocida: {o}");
                std::process::exit(1);
            }
        }
        i += 2;
    }
    if ver_estado {
        return estado(&carpeta);
    }
    std::fs::create_dir_all(&carpeta).expect("no puedo crear la carpeta");
    let archivo_config = carpeta.join("config.txt");
    let c = match std::fs::read_to_string(&archivo_config) {
        Ok(t) => {
            if !ajustes.is_empty() || archivo_base.is_some() {
                eprintln!("AVISO: ya hay {} (la vida sigue con esa config); ignoro --mundo/--config/ajustes.",
                          archivo_config.display());
            }
            Config::leer(&t)
        }
        Err(_) => {
            let (t, c) = armar(archivo_base.as_deref(), &ajustes);
            escribir_archivo(&archivo_config, &t).expect("no puedo guardar config.txt");
            c
        }
    };
    escuchar_senales();
    let archivo = carpeta.join("mente.txt");
    let mut a = match retomar(&carpeta) {
        Some(mut a) => {
            println!("Retoma a Angri con {} ticks vividos.", a.edad);
            a.evento = Evento::Apagado;
            a
        }
        None => {
            let s = semilla.unwrap_or_else(|| {
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64
            });
            println!("Nace Angri (semilla {s}).");
            Angri::new(s, &c)
        }
    };
    let guardar = |a: &Angri| {
        let t = a.volcar();
        if let Err(e) = escribir_archivo(&archivo, &t) {
            eprintln!("AVISO: no pude guardar ({e})");
            return;
        }
        if let Err(e) = respaldar(&carpeta, &t, a.edad) {
            eprintln!("AVISO: no pude respaldar ({e})");
        }
    };
    let log = carpeta.join("vida.csv");
    let nuevo = !log.exists();
    // (los vida.csv de antes no tenían ram_mb: ahí no se agrega, para no romper las columnas)
    let con_ram = nuevo || std::fs::read_to_string(&log).map_or(false, |t| t.lines().next().unwrap_or("").ends_with(",ram_mb"));
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&log).expect("vida.csv");
    if nuevo {
        writeln!(f, "edad,malestar,hambre_pct,heridas,comidas,suenos,ticks_seg,ram_mb").unwrap();
    }
    let (mut n, mut malestar, mut hambre, mut heridas, mut comidas) = (0i64, 0.0f64, 0i64, 0i64, 0i64);
    let mut t0 = std::time::Instant::now();
    let mut hechos = 0i64;
    let archivo_foto = carpeta.join("foto.json");
    let mut proxima_foto = std::time::Instant::now();
    let pausa = std::time::Duration::from_secs_f64(1.0 / c.ticks_por_segundo.max(1) as f64);
    while ticks.map_or(true, |t| hechos < t) && !PARAR.load(Ordering::SeqCst) {
        a.evento = Evento::Nada;
        a.tick(&c);
        hechos += 1;
        n += 1;
        malestar += a.malestar(&c);
        hambre += (a.cuerpo.v[ENERGIA] < 0.3) as i64;
        heridas += (a.evento == Evento::Lastima) as i64;
        comidas += (a.evento == Evento::Come) as i64;
        if cada_foto > 0.0 && a.edad % 50 == 0 && std::time::Instant::now() >= proxima_foto {
            proxima_foto = std::time::Instant::now() + std::time::Duration::from_secs_f64(cada_foto);
            if let Err(e) = foto::escribir(&a, &c, &archivo_foto) {
                eprintln!("AVISO: no pude escribir la foto ({e})");
            }
        }
        if panel {
            println!("{}", foto::panel(&a, &c));
            std::thread::sleep(pausa);
        }
        if a.edad % ventana == 0 {
            let dt = t0.elapsed().as_secs_f64();
            let fila = format!("{},{:.4},{:.2},{},{},{},{:.0}", a.edad, malestar / n as f64,
                               100.0 * hambre as f64 / n as f64, heridas, comidas, a.suenos, n as f64 / dt);
            let ram = ram_mb();
            let fila = if con_ram { format!("{fila},{ram:.1}") } else { fila };
            writeln!(f, "{fila}").unwrap();
            f.flush().unwrap();
            if !panel {
                println!("edad={} malestar={:.4} hambre_pct={:.2} heridas={} comidas={} suenos={} ticks_seg={:.0}",
                     a.edad, malestar / n as f64, 100.0 * hambre as f64 / n as f64, heridas, comidas, a.suenos,
                     n as f64 / dt);
            }
            n = 0;
            malestar = 0.0;
            hambre = 0;
            heridas = 0;
            comidas = 0;
            t0 = std::time::Instant::now();
        }
        if a.edad % guardar_cada == 0 {
            guardar(&a);
        }
    }
    if PARAR.load(Ordering::SeqCst) {
        println!("Señal recibida: guarda y duerme.");
    }
    guardar(&a);
    if cada_foto > 0.0 {
        let _ = foto::escribir(&a, &c, &archivo_foto); // la última, para el visor
    }
    println!("Guardado en {} ({} ticks vividos).", archivo.display(), a.edad);
}

/// RAM actual del proceso en MB (Linux: VmRSS; Windows: working set). 0 si no se sabe.
fn ram_mb() -> f64 {
    if let Ok(t) = std::fs::read_to_string("/proc/self/status") {
        if let Some(l) = t.lines().find(|l| l.starts_with("VmRSS:")) {
            return l.split_whitespace().nth(1).and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0) / 1024.0;
        }
    }
    ram_windows()
}

#[cfg(windows)]
fn ram_windows() -> f64 {
    #[repr(C)]
    struct Pmc { cb: u32, fallos: u32, pico: usize, actual: usize, resto: [usize; 6] }
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(p: isize, c: *mut Pmc, cb: u32) -> i32;
    }
    let mut c = Pmc { cb: std::mem::size_of::<Pmc>() as u32, fallos: 0, pico: 0, actual: 0, resto: [0; 6] };
    if unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) } != 0 {
        c.actual as f64 / (1u64 << 20) as f64
    } else {
        0.0
    }
}

#[cfg(not(windows))]
fn ram_windows() -> f64 {
    0.0
}

fn dur(s: f64) -> String {
    let s = s.max(0.0) as i64;
    if s < 120 {
        format!("{s} s")
    } else if s < 7200 {
        format!("{} min", s / 60)
    } else if s < 172800 {
        format!("{:.1} h", s as f64 / 3600.0)
    } else {
        format!("{:.1} días", s as f64 / 86400.0)
    }
}

fn hace(p: &Path) -> f64 {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
        .and_then(|t| t.elapsed().ok()).map_or(0.0, |d| d.as_secs_f64())
}

/// Como `vivir.py --estado`: resumen para mirar por SSH (mismo formato).
fn estado(carpeta: &Path) {
    let archivo = carpeta.join("mente.txt");
    if archivo.exists() {
        let edad = cargar(&archivo).map_or("?".to_string(), |a| a.edad.to_string());
        if edad == "?" {
            println!("AVISO: mente.txt no se puede leer");
        }
        let kb = std::fs::metadata(&archivo).map_or(0, |m| m.len()) as f64 / 1024.0;
        println!("Mente: {} ({:.0} KB), edad {edad} al guardar, guardada hace {}",
                 archivo.display(), kb, dur(hace(&archivo)));
    } else {
        println!("No hay mente en {}.", archivo.display());
    }
    let mut resp: Vec<PathBuf> = std::fs::read_dir(carpeta.join("respaldos"))
        .map(|d| d.filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map_or(false, |x| x == "txt")).collect())
        .unwrap_or_default();
    resp.sort_by(|a, b| hace(b).partial_cmp(&hace(a)).unwrap());
    match resp.last() {
        Some(u) => println!("Respaldos: {} (último {})", resp.len(), u.file_name().unwrap().to_string_lossy()),
        None => println!("Respaldos: 0"),
    }
    let log = carpeta.join("vida.csv");
    let Ok(texto) = std::fs::read_to_string(&log) else {
        println!("No hay vida.csv.");
        return;
    };
    let mut renglones = texto.lines();
    let cab: Vec<&str> = renglones.next().unwrap_or("").split(',').collect();
    let filas: Vec<Vec<&str>> = renglones.filter(|l| !l.is_empty()).map(|l| l.split(',').collect()).collect();
    let Some(u) = filas.last() else { return };
    let col = |f: &Vec<&str>, k: &str| cab.iter().position(|c| *c == k).and_then(|i| f.get(i).copied()).unwrap_or("?").to_string();
    println!("Última ventana anotada hace {}: edad {}, {} ticks/seg, RAM {} MB",
             dur(hace(&log)), col(u, "edad"), col(u, "ticks_seg"), col(u, "ram_mb"));
    let cols = ["edad", "malestar", "hambre_pct", "heridas", "comidas", "ticks_seg", "ram_mb"];
    println!("{}", cols.iter().map(|c| format!("{c:>10}")).collect::<Vec<_>>().join("  "));
    for f in &filas[filas.len().saturating_sub(10)..] {
        println!("{}", cols.iter().map(|c| format!("{:>10}", col(f, c))).collect::<Vec<_>>().join("  "));
    }
}
