//! La foto para el visor (como angri/foto.py) y el panel de la terminal (como
//! angri/panel.py). La foto es el mismo JSON que escribe Python, byte a byte,
//! así `visor.py` puede mirar una vida de Rust sin saber de qué lenguaje viene.
use crate::config::Config;
use crate::guardar::escribir_archivo;
use crate::mundo::*;
use crate::vida::{Angri, Evento};

pub const NOMBRES: [&str; 7] = ["energia", "cansancio", "curiosidad", "estres", "integridad", "agua", "calor"];
const CELDAS: [&str; 7] = ["·", "*", "~", "?", "X", "#", "≈"];
const ESTACIONES: [&str; 4] = ["primavera", "verano", "otoño", "invierno"];
const TIPOS: [&str; 4] = ["fruta", "baya", "nuez", "raiz"];

pub fn evento_texto(e: Evento) -> &'static str {
    use Evento::*;
    match e {
        Nada => "",
        Come => "come",
        Descubre => "descubre algo nuevo",
        Lastima => "se lastima",
        Muerde => "lo muerde el predador",
        Enferma => "se enferma",
        Bebe => "bebe",
        Desmaya => "se desmaya",
        Otro => "",
        Trabaja => "trabaja la comida",
        Duerme => "se duerme",
        Despierta => "se despierta",
        Nace => "nace",
        Apagado => "despierta del apagado",
    }
}

/// round(x, n) de Python y su repr (el formato de Rust redondea el valor
/// binario exacto al par, igual que Python).
fn redondo(x: f64, n: usize) -> String {
    let r: f64 = format!("{:.n$}", x).parse().unwrap();
    format!("{:?}", r)
}

fn texto(s: &str) -> String {
    let mut o = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => o += "\\\"",
            '\\' => o += "\\\\",
            c if (c as u32) < 0x20 => o += &format!("\\u{:04x}", c as u32),
            c => o.push(c),
        }
    }
    o + "\""
}

fn filas(celdas: &[u8], w: i64, f: impl Fn(u8) -> String) -> Vec<String> {
    celdas.chunks(w as usize).map(|fila| fila.iter().map(|&c| f(c)).collect()).collect()
}

fn lista(xs: &[String]) -> String {
    format!("[{}]", xs.join(","))
}

/// El JSON de angri/foto.py: foto(angri), con separadores "," y ":".
pub fn foto(a: &Angri, c: &Config) -> String {
    let (w, h) = a.mundo.ancho_alto();
    let celda = |x: u8| CELDAS[x as usize].to_string();
    let grilla = lista(&filas(a.mundo.celdas(), w, celda).iter().map(|s| texto(s)).collect::<Vec<_>>());
    let (mundo, terreno) = match &a.mundo {
        Mundo::B(b) => {
            let hora = b.tiempo.rem_euclid(c.dia) as f64 / c.dia as f64;
            let comida: Vec<String> = b.comida.iter()
                .map(|((x, y), t, _)| format!("[{x},{y},{}]", texto(TIPOS[*t as usize]))).collect();
            let m = format!(
                "{{\"ancho\":{w},\"alto\":{h},\"tiempo\":{},\"hora\":{},\"noche\":{},\"estacion\":{},\
                 \"lluvia\":{},\"crecida\":{},\"predador\":[{},{}],\"harto\":{},\"grilla\":{grilla},\"comida\":{}}}",
                b.tiempo, redondo(hora, 3), b.noche(c), texto(ESTACIONES[b.estacion(c) as usize]),
                b.lluvia != 0, b.crecida != 0, b.predador.0, b.predador.1, b.harto > 0, lista(&comida));
            let t = filas(&b.zona, w, |z| (z as char).to_string());
            (m, lista(&t.iter().map(|s| texto(s)).collect::<Vec<_>>()))
        }
        Mundo::G(g) => {
            let m = format!(
                "{{\"ancho\":{w},\"alto\":{h},\"tiempo\":{},\"hora\":0.25,\"noche\":false,\"estacion\":\"-\",\
                 \"lluvia\":false,\"crecida\":false,\"predador\":null,\"harto\":false,\"grilla\":{grilla},\"comida\":[]}}",
                g.tiempo);
            let fila = texto(&"l".repeat(w as usize));
            (m, lista(&vec![fila; h as usize]))
        }
    };
    let cu = &a.cuerpo;
    let cuerpo: Vec<String> = (0..cu.n).map(|k| format!("\"{}\":{}", NOMBRES[k], redondo(cu.v[k], 3))).collect();
    let q = &a.quimica;
    let quimica = format!("{{\"adenosina\":{},\"cortisol\":{},\"dopamina\":{},\"noradrenalina\":{}}}",
                          redondo(q.adenosina, 3), redondo(q.cortisol, 3), redondo(q.dopamina, 3),
                          redondo(q.noradrenalina, 3));
    let (td, lugares) = if a.vieja.is_some() {
        (0.0, String::new())
    } else {
        let l: Vec<String> = a.mente.lugares.iter().enumerate()
            .map(|(k, l)| format!("\"{}\":{}", NOMBRES[k], l.v.len())).collect();
        (a.mente.dopamina, l.join(","))
    };
    format!(
        "{{\"edad\":{},\"suenos\":{},\"despierto\":{},\"evento\":{},\"pos\":[{},{}],\"malestar\":{},\
         \"foco\":{},\"cuerpo\":{{{}}},\"quimica\":{quimica},\"dopamina_td\":{},\"lugares\":{{{lugares}}},\
         \"mundo\":{mundo},\"terreno\":{terreno}}}",
        a.edad, a.suenos, a.despierto, texto(evento_texto(a.evento)), a.x, a.y, redondo(cu.malestar(c), 4),
        texto(NOMBRES[cu.necesidad_urgente(c)]), cuerpo.join(","), redondo(td, 3))
}

pub fn escribir(a: &Angri, c: &Config, archivo: &std::path::Path) -> std::io::Result<()> {
    escribir_archivo(archivo, &foto(a, c))
}

fn barra(valor: f64) -> String {
    let llenos = (valor.clamp(0.0, 1.0) * 20.0).round_ties_even() as usize;
    "█".repeat(llenos) + &"░".repeat(20 - llenos)
}

/// El panel de angri/panel.py (sin "dice"/"oye": Rust no guarda las palabras).
pub fn panel(a: &Angri, c: &Config) -> String {
    let (w, h) = a.mundo.ancho_alto();
    let mut l: Vec<String> = vec![];
    let estado = if a.despierto { "DESPIERTO" } else { "DURMIENDO" };
    l.push(format!("ANGRI · edad {} ticks · {estado} · sueños: {}", a.edad, a.suenos));
    l.push(format!("último evento: {}", evento_texto(a.evento)));
    l.push(String::new());
    for y in 0..h {
        let mut fila = String::new();
        for x in 0..w {
            if (x, y) == (a.x, a.y) {
                fila += if a.despierto { "A" } else { "z" };
            } else {
                fila += CELDAS[a.mundo.en(x, y) as usize];
            }
        }
        l.push(fila);
    }
    l.push(String::new());
    l.push("CUERPO".into());
    for k in 0..a.cuerpo.n {
        let v = a.cuerpo.v[k];
        l.push(format!("  {:<11}{} {:.2}", NOMBRES[k], barra(v), v));
    }
    let q = &a.quimica;
    l.push("QUÍMICA".into());
    l.push(format!("  {:<11}{} {:.2}  (dormir ≥ {:?})", "adenosina", barra(q.adenosina), q.adenosina, c.umbral_dormir));
    l.push(format!("  {:<11}{} {:.2}", "cortisol", barra(q.cortisol), q.cortisol));
    l.push(format!("  {:<11}{} {:.2}", "noradren.", barra(q.noradrenalina), q.noradrenalina));
    l.push(format!("  {:<11}{:+.2}", "dopamina", q.dopamina));
    l.push(String::new());
    let m = a.cuerpo.malestar(c);
    let mut s = format!("malestar: {} {:.2} · necesidad urgente: {}", barra(m), m, NOMBRES[a.cuerpo.necesidad_urgente(c)]);
    if let Some(t) = &a.talamo {
        s += &format!(" · foco: {}", t.foco.map_or("None", |f| NOMBRES[f]));
    }
    l.push(s);
    if a.maestro.is_some() {
        l.push(format!("confianza en el maestro: {:.2}", a.oido.confianza));
    }
    l.push(String::new());
    l.push("Ctrl+C para pausar (guarda su estado y sigue desde ahí la próxima vez)".into());
    // borrar la pantalla con ANSI (Windows 10+ y Linux)
    format!("\x1b[2J\x1b[H{}", l.join("\n"))
}
