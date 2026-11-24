//! Vida continua (como vivir.py, sin panel): vive sin fin, guarda la mente cada
//! tanto con respaldos rotativos y termina bien con Ctrl-C o SIGTERM.
//!
//!   angri vivir [--carpeta DIR] [--ticks N] [--semilla S] [--ventana W] [--guardar G]
//!
//! La config: DIR/config.txt (líneas CLAVE=valor). Si no existe, se lee de stdin
//! y se guarda ahí, así la vida sigue siempre con la misma config:
//!   python -c "import medir; print(medir.config_texto(), end='')" | angri vivir --carpeta DIR
//! Si existe DIR/mente.txt la retoma exactamente donde estaba; si no carga, retoma
//! del respaldo válido más reciente (DIR/respaldos/: los últimos 5 y uno por día, 30 días).
//! Log liviano en DIR/vida.csv, un renglón por ventana.
use crate::config::Config;
use crate::guardar::escribir_archivo;
use crate::mente::ENERGIA;
use crate::vida::{Angri, Evento};
use std::io::{Read, Write};
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
    let mut i = 0;
    while i < args.len() {
        let v = args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--carpeta" => carpeta = PathBuf::from(v),
            "--ticks" => ticks = Some(v.parse::<i64>().expect("--ticks")),
            "--semilla" => semilla = Some(v.parse::<u64>().expect("--semilla")),
            "--ventana" => ventana = v.parse().expect("--ventana"),
            "--guardar" => guardar_cada = v.parse().expect("--guardar"),
            o => {
                eprintln!("opción desconocida: {o}");
                std::process::exit(1);
            }
        }
        i += 2;
    }
    std::fs::create_dir_all(&carpeta).expect("no puedo crear la carpeta");
    let archivo_config = carpeta.join("config.txt");
    let texto_config = match std::fs::read_to_string(&archivo_config) {
        Ok(t) => t,
        Err(_) => {
            let mut t = String::new();
            std::io::stdin().read_to_string(&mut t).unwrap();
            escribir_archivo(&archivo_config, &t).expect("no puedo guardar config.txt");
            t
        }
    };
    let c = Config::leer(&texto_config);
    escuchar_senales();
    let archivo = carpeta.join("mente.txt");
    let mut a = match retomar(&carpeta) {
        Some(a) => {
            println!("Retoma a Angri con {} ticks vividos.", a.edad);
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
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&log).expect("vida.csv");
    if nuevo {
        writeln!(f, "edad,malestar,hambre_pct,heridas,comidas,suenos,ticks_seg").unwrap();
    }
    let (mut n, mut malestar, mut hambre, mut heridas, mut comidas) = (0i64, 0.0f64, 0i64, 0i64, 0i64);
    let mut t0 = std::time::Instant::now();
    let mut hechos = 0i64;
    while ticks.map_or(true, |t| hechos < t) && !PARAR.load(Ordering::SeqCst) {
        a.evento = Evento::Nada;
        a.tick(&c);
        hechos += 1;
        n += 1;
        malestar += a.malestar(&c);
        hambre += (a.cuerpo.v[ENERGIA] < 0.3) as i64;
        heridas += (a.evento == Evento::Lastima) as i64;
        comidas += (a.evento == Evento::Come) as i64;
        if a.edad % ventana == 0 {
            let dt = t0.elapsed().as_secs_f64();
            let fila = format!("{},{:.4},{:.2},{},{},{},{:.0}", a.edad, malestar / n as f64,
                               100.0 * hambre as f64 / n as f64, heridas, comidas, a.suenos, n as f64 / dt);
            writeln!(f, "{fila}").unwrap();
            f.flush().unwrap();
            println!("edad={} malestar={:.4} hambre_pct={:.2} heridas={} comidas={} suenos={} ticks_seg={:.0}",
                     a.edad, malestar / n as f64, 100.0 * hambre as f64 / n as f64, heridas, comidas, a.suenos,
                     n as f64 / dt);
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
    println!("Guardado en {} ({} ticks vividos).", archivo.display(), a.edad);
}
