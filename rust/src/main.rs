//! Angri: una simulación de un agente con cerebro bioinspirado. Todo en Rust,
//! sin dependencias. La config está en rust/config.txt (metida en el binario).
//!
//!   angri vivir [--carpeta DIR] ...             -> vida continua (ver vivir.rs)
//!   angri visor [--carpeta DIR] [--puerto P]    -> mirarla desde el navegador (visor.rs)
//!   angri medir [TICKS] [N] [CLAVE=VALOR ...]   -> métricas de N vidas (medir.rs)
//!   angri comparar TICKS N "A" "B" [-j J]       -> pares A/B con la misma semilla
//!   angri juntar CRUDO...                       -> junta porciones de comparar --crudo
//!   angri vida SEMILLA TICKS      [CLAVE=VALOR ...] -> métricas de una vida en JSON
//!   angri traza SEMILLA TICKS     ...              -> una línea por tick (bits exactos)
//!   angri foto SEMILLA TICKS [CADA] ...            -> la foto del visor (o una cada CADA ticks)
//!   angri velocidad SEMILLA TICKS ...              -> ticks/s
//!   angri retomar SEMILLA TICKS   ...              -> cortar, guardar, cargar y seguir da IDÉNTICO
//! En todos, `--config ARCHIVO` usa otra config en vez de la de adentro.
mod azar;
mod config;
mod extra;
mod foto;
mod guardar;
mod lenguaje;
mod mate;
mod medir;
mod mente;
mod mundo;
mod propia;
#[cfg(test)]
mod pruebas;
mod vida;
mod visor;
mod vivir;

use config::Config;
use mente::*;
use mundo::*;
use std::io::Write;
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

/// Celdas a las que llega desde (x, y) sin pisar PELIGRO (con `pasa_peligro`, pisándolo), hasta `tope`.
fn region(b: &Bosque, x: i64, y: i64, pasa_peligro: bool, tope: usize) -> Vec<(i64, i64)> {
    let mut r = vec![(x, y)];
    let mut i = 0;
    while i < r.len() && r.len() <= tope {
        let (cx, cy) = r[i];
        i += 1;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (cx + dx, cy + dy);
            if b.transitable(nx, ny) && (pasa_peligro || b.en(nx, ny) != PELIGRO) && !r.contains(&(nx, ny)) {
                r.push((nx, ny));
            }
        }
    }
    r
}

/// Bolsillo: sin pisar espinas llega a 3 celdas o menos, pero pisándolas sale (la espina es la puerta).
/// Devuelve las celdas del bolsillo.
fn bolsillo(b: &Bosque, x: i64, y: i64) -> Option<Vec<(i64, i64)>> {
    if b.en(x, y) == PELIGRO {
        return None;
    }
    let r = region(b, x, y, false, 3);
    if r.len() > 3 || region(b, x, y, true, 3).len() <= 3 {
        return None;
    }
    Some(r)
}

/// Las métricas de una vida (las de medir.py), como (nombre, número en texto).
fn metricas(semilla: u64, ticks: i64, c: &Config) -> Vec<(String, String)> {
    let mut a = Angri::new(semilla, c);
    let corte = (ticks as f64 * 0.8) as i64;
    let (mut malestar, mut fin) = (0.0f64, 0.0f64);
    let (mut heridas, mut comidas, mut novedades, mut hambre, mut alto) = (0i64, 0i64, 0i64, 0i64, 0i64);
    let bosque = c.bosque;
    let (mut sed, mut frio, mut mordidas, mut enfermo, mut bebidas, mut dormido, mut refugio) = (0i64, 0i64, 0i64, 0i64, 0i64, 0i64, 0i64);
    let (mut racha, mut racha_max) = (0i64, 0i64);
    let (mut techo_dormido, mut lleva, mut pila_max, mut refugio_propio) = (0i64, 0i64, 0i64, 0i64);
    let mut plantada = 0i64;
    // Paso 0 (métricas que no mienten): ventana de hambre, racha, despierta/dormida, crudas, estación, tope
    const VENT: usize = 10_000;
    let mut anillo = vec![false; VENT];
    let (mut en_vent, mut peor_vent, mut racha_h, mut racha_h_max) = (0i64, 0i64, 0i64, 0i64);
    let (mut desp, mut h_desp, mut h_dorm, mut cruda, mut tope) = (0i64, 0i64, 0i64, 0i64, 0i64);
    let (mut h_est, mut t_est) = ([0i64; 4], [0i64; 4]);
    let (mut cerca_pred, mut huye) = (0i64, 0i64);
    // Exposición vs evitación: cuánto estuvo al lado del peligro y cuánto se lastimó estando ahí
    let (mut expuesta, mut heridas_exp, mut pred2, mut mordidas_p2, mut recorrido) = (0i64, 0i64, 0i64, 0i64, 0i64);
    let mut anio = 0.0f64;
    let mut por_anio: Vec<f64> = vec![];
    let largo_anio = 4 * c.estacion_bosque;
    let (mut tardanzas, mut rondas): (Vec<i64>, Vec<i64>) = (vec![], vec![]);
    // [nueva, vieja, ticks desde el cambio, ticks cerca de la vieja]
    let mut pendiente: Option<((i64, i64), (i64, i64), i64, i64)> = None;
    let ventana = if c.estacion != 0 { c.estacion / 2 } else { 0 };
    let r = c.radio_fuente;
    // Diagnóstico (dg): solo miran, no tocan el azar ni el estado. [toda la vida, último 20 %]
    let (mut dg_ent, mut dg_ent_n) = ([0.0f64; 2], [0i64; 2]);
    let (mut dg_quieta, mut dg_desp) = ([0i64; 2], [0i64; 2]);
    let (mut dg_v, mut dg_v_n) = ([0.0f64; 2], [0i64; 2]);
    let (mut dg_dist, mut dg_dist_n) = ([0.0f64; 2], [0i64; 2]);
    // Bolsillo con espina por puerta: ticks adentro, entradas, aperturas (la espina se corre sola y queda salida
    // sin espina), salidas por esa apertura, cruces pisando la espina, comida a la vista adentro, racha máxima
    let mut bol_prev: Option<Vec<(i64, i64)>> = None;
    let mut abierto: Option<(Vec<(i64, i64)>, i64)> = None; // (celdas del bolsillo que se abrió, tick)
    let (mut bol_t, mut bol_entra, mut bol_abre, mut bol_sale_abre, mut bol_cruza) = (0i64, 0i64, 0i64, 0i64, 0i64);
    let (mut bol_racha, mut bol_racha_max, mut bol_comida_vista, mut bol_hambre) = (0i64, 0i64, 0i64, 0i64);
    let (mut bol_espera_abre, mut bol_desm) = (0i64, 0i64);
    for t in 0..ticks {
        a.evento = Evento::Nada;
        let cambios = match &a.mundo { Mundo::G(g) => g.cambios, _ => 0 };
        let mut d0 = 99;
        if let Mundo::B(b) = &a.mundo {
            let (px, py) = b.predador;
            d0 = if b.harto == 0 { (px - a.x).abs() + (py - a.y).abs() } else { 99 };
        }
        let (x0, y0) = (a.x, a.y);
        let (mw, mh) = a.mundo.ancho_alto();
        let junto = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
            let (x, y) = (x0 + dx, y0 + dy);
            x >= 0 && y >= 0 && x < mw && y < mh && a.mundo.en(x, y) == PELIGRO
        });
        a.tick(c);
        recorrido += (a.x - x0).abs() + (a.y - y0).abs();
        {
            let tramos: &[usize] = if t >= corte { &[0, 1] } else { &[0] };
            for &j in tramos {
                if a.despierto {
                    dg_desp[j] += 1;
                    dg_quieta[j] += (a.x == x0 && a.y == y0) as i64;
                    let pr = a.mente.probs_ahora();
                    let na = pr.len();
                    let h: f64 = pr.iter().filter(|&&p| p > 0.0).map(|&p| -p * p.ln()).sum();
                    if na > 1 && h.is_finite() {
                        dg_ent[j] += h / (na as f64).ln();
                        dg_ent_n[j] += 1;
                    }
                }
                if let Some(p) = &a.mente.propia {
                    if p.vs.is_finite() {
                        dg_v[j] += p.vs;
                        dg_v_n[j] += 1;
                    }
                }
                if let (0, Mundo::B(b)) = (t % 100, &a.mundo) {
                    if let Some(d) = b.comida.iter().map(|e| (e.0 .0 - a.x).abs() + (e.0 .1 - a.y).abs()).min() {
                        dg_dist[j] += d as f64;
                        dg_dist_n[j] += 1;
                    }
                }
            }
        }
        if let Mundo::B(b) = &a.mundo {
            let ahora = bolsillo(b, a.x, a.y);
            match (&bol_prev, &ahora) {
                (None, Some(_)) => bol_entra += 1,
                (Some(p), None) => {
                    if b.en(a.x, a.y) == PELIGRO {
                        bol_cruza += 1;
                    } else if p.contains(&(a.x, a.y)) {
                        // sigue donde estaba pero ya no es bolsillo: la espina se corrió (o se fue la pila)
                        bol_abre += 1;
                        abierto = Some((p.clone(), t));
                    }
                }
                _ => {}
            }
            if let Some(cel) = &ahora {
                bol_t += 1;
                bol_racha += 1;
                bol_racha_max = bol_racha_max.max(bol_racha);
                bol_hambre += (a.cuerpo.v[ENERGIA] < 0.3) as i64;
                bol_desm += (a.evento == Evento::Desmaya) as i64;
                if t % 10 == 0 {
                    let vista = (a.x - 3..=a.x + 3).any(|x| (a.y - 3..=a.y + 3).any(|y| {
                        x >= 0 && y >= 0 && x < mw && y < mh && b.en(x, y) == COMIDA && !cel.contains(&(x, y))
                    }));
                    bol_comida_vista += vista as i64;
                }
                abierto = None; // se volvió a cerrar (o entró a otro) antes de salir
            } else {
                bol_racha = 0;
                if let Some((cel, t0)) = &abierto {
                    if cel.iter().all(|&(x, y)| (x - a.x).abs() + (y - a.y).abs() > 3) {
                        bol_sale_abre += 1;
                        bol_espera_abre += t - t0;
                        abierto = None;
                    }
                }
            }
            bol_prev = ahora;
        }
        if junto {
            expuesta += 1;
            heridas_exp += (a.evento == Evento::Lastima) as i64;
        }
        if d0 <= 2 {
            pred2 += 1;
            mordidas_p2 += (a.evento == Evento::Muerde) as i64;
        }
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
        let con_hambre = a.cuerpo.v[ENERGIA] < 0.3;
        hambre += con_hambre as i64;
        let i = (t as usize) % VENT;
        en_vent += con_hambre as i64 - anillo[i] as i64;
        anillo[i] = con_hambre;
        if t as usize + 1 >= VENT.min(ticks as usize) {
            peor_vent = peor_vent.max(en_vent);
        }
        racha_h = if con_hambre { racha_h + 1 } else { 0 };
        racha_h_max = racha_h_max.max(racha_h);
        if a.despierto {
            desp += 1;
            h_desp += con_hambre as i64;
        } else {
            h_dorm += con_hambre as i64;
        }
        // necesidades "de más es mejor" (equilibrio alto): energía, integridad, agua, calor
        let v = &a.cuerpo.v;
        cruda += (0..a.cuerpo.n).any(|k| c.equilibrio[k] >= 0.5 && v[k] < 0.15) as i64;
        tope += (m >= c.malestar_maximo - 1e-9) as i64;
        alto += (m > c.malestar_alto) as i64;
        racha = if m > c.malestar_alto { racha + 1 } else { 0 };
        racha_max = racha_max.max(racha);
        if let Mundo::B(b) = &a.mundo {
            let v = &a.cuerpo.v;
            anio += m;
            let e = b.estacion(c) as usize;
            t_est[e] += 1;
            h_est[e] += con_hambre as i64;
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
            if c.objetos {
                if !a.despierto {
                    techo_dormido += b.cubierto(a.x, a.y) as i64;
                    refugio_propio += (b.refugio(a.x, a.y) && b.zona_en(a.x, a.y) != Z_CUEVA) as i64;
                }
                lleva += a.lleva.is_some() as i64;
                if t % 100 == 0 {
                    pila_max = pila_max.max(b.pilas.iter().map(|p| p.len() as i64).max().unwrap_or(0));
                }
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
    f("peor 10k hambre", num((100 * peor_vent) as f64 / VENT.min(ticks as usize) as f64));
    f("racha hambre", racha_h_max.to_string());
    f("% hambre desp", num((100 * h_desp) as f64 / desp.max(1) as f64));
    f("% hambre dorm", num((100 * h_dorm) as f64 / (ticks - desp).max(1) as f64));
    f("% mal alto", num((100 * alto) as f64 / tk));
    f("% cruda<0.15", num((100 * cruda) as f64 / tk));
    f("% mal tope", num((100 * tope) as f64 / tk));
    f("racha alta", racha_max.to_string());
    f("% expuesta", num((100 * expuesta) as f64 / tk));
    f("heridas/1k exp", num(1000.0 * heridas_exp as f64 / expuesta.max(1) as f64));
    f("recorrido", recorrido.to_string());
    if bosque {
        f("% sed", num((100 * sed) as f64 / tk));
        f("% frío", num((100 * frio) as f64 / tk));
        f("mordidas", mordidas.to_string());
        f("% pred<=2", num((100 * pred2) as f64 / tk));
        f("mordidas/1k pred<=2", num(1000.0 * mordidas_p2 as f64 / pred2.max(1) as f64));
        f("enfermó", enfermo.to_string());
        f("bebe", bebidas.to_string());
        f("% duerme cueva", num((100 * refugio) as f64 / dormido.max(1) as f64));
        f("desmayos", a.desmayos.to_string());
        for (i, k) in ["hambre", "sed", "frio", "heridas"].iter().enumerate() {
            f(&format!("desm. {k}"), a.causas[i].to_string());
        }
        f("% plantada", num((100 * plantada) as f64 / tk));
        for (e, k) in ["prim", "ver", "oto", "inv"].iter().enumerate() {
            if t_est[e] > 0 {
                f(&format!("% hambre {k}"), num((100 * h_est[e]) as f64 / t_est[e] as f64));
            }
        }
        f("% huye", num(if cerca_pred != 0 { (100 * huye) as f64 / cerca_pred as f64 } else { 0.0 }));
        for (i, x) in por_anio.iter().enumerate() {
            f(&format!("mal. año {}", i + 1), num(*x));
        }
        if let (true, Mundo::B(b)) = (c.objetos, &a.mundo) {
            for (i, k) in vida::HECHOS.iter().enumerate() {
                f(k, a.hechos[i].to_string());
            }
            f("% lleva", num((100 * lleva) as f64 / tk));
            f("pila max", pila_max.to_string());
            f("techo final", b.techo.iter().filter(|&&t| t > 0).count().to_string());
            f("% duerme techo", num((100 * techo_dormido) as f64 / dormido.max(1) as f64));
            f("% duerme refugio propio", num((100 * refugio_propio) as f64 / dormido.max(1) as f64));
        }
    } else {
        f("encuentra", num(if tardanzas.is_empty() { 0.0 } else { media_int(&tardanzas) }));
        f("ronda vieja", num(if rondas.is_empty() { 0.0 } else { media_int(&rondas) }));
    }
    {
        let m = |s: f64, n: i64| if n > 0 { s / n as f64 } else { f64::NAN };
        for (j, suf) in ["", " final"].iter().enumerate() {
            f(&format!("dg entropía{suf}"), num(m(dg_ent[j], dg_ent_n[j])));
            f(&format!("dg quieta %{suf}"), num(100.0 * m(dg_quieta[j] as f64, dg_desp[j])));
            if a.mente.propia.is_some() {
                f(&format!("dg V{suf}"), num(m(dg_v[j], dg_v_n[j])));
            }
            if bosque {
                f(&format!("dg dist comida{suf}"), num(m(dg_dist[j], dg_dist_n[j])));
            }
        }
        if bosque {
            f("dg % bolsillo", num((100 * bol_t) as f64 / tk));
            f("dg bols entra", bol_entra.to_string());
            f("dg bols abre", bol_abre.to_string());
            f("dg bols sale abre", bol_sale_abre.to_string());
            f("dg bols espera sale", num(m(bol_espera_abre as f64, bol_sale_abre)));
            f("dg bols cruza", bol_cruza.to_string());
            f("dg bols racha", bol_racha_max.to_string());
            f("dg bols % hambre", num(100.0 * m(bol_hambre as f64, bol_t)));
            f("dg bols % comida vista", num(100.0 * m(bol_comida_vista as f64, (bol_t + 9) / 10)));
            f("dg bols desmayos", bol_desm.to_string());
        }
    }
    if let Some(p) = &a.mente.propia {
        // al final de la vida: el banco (no-átomos) y los átomos solos
        let (atomos, banco) = p.unidades.split_at(propia::N_ATOMOS);
        let vivas: Vec<&propia::Unidad> = banco.iter().filter(|u| u.k > 0).collect();
        let wabs = |u: &propia::Unidad| if u.mover { u.w[0].abs() } else { u.w.iter().map(|x| x.abs()).sum::<f64>() / NACC as f64 };
        let mm = |xs: Vec<f64>| if xs.is_empty() { f64::NAN } else { xs.iter().sum::<f64>() / xs.len() as f64 };
        f("dg unidades", vivas.len().to_string());
        f("dg util", num(mm(vivas.iter().map(|u| u.util).collect())));
        f("dg |w| átomos", num(mm(atomos.iter().map(|u| wabs(u)).collect())));
        f("dg |w| banco", num(mm(vivas.iter().map(|u| wabs(u)).collect())));
        // q[0] = suba de energía predicha, q[1] = baja (PREDICE[0] = ENERGIA): neto = q[0] - q[1]
        let qe: Vec<f64> = p.activas_ahora().iter().filter(|&&(_, mk)| !propia::Propia::es_del_estado(mk))
            .map(|&(u, _)| { let q = &p.unidades[u as usize].q; q[0] - q[1] }).collect();
        f("dg q energía", num(mm(qe)));
        let (vivas, mezclan) = p.resumen();
        f("unidades", vivas.to_string());
        f("mezclan", mezclan.to_string());
        f("reiniciadas", p.reiniciadas.to_string());
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
    o
}

/// Las métricas en JSON, una línea (el formato de siempre: lo usa la referencia).
fn una_vida(semilla: u64, ticks: i64, c: &Config) -> String {
    // Las métricas del paso 0 (revisión del diseño) solo miran la vida: quedan fuera de la referencia
    // para que la huella de la vida siga siendo la misma.
    const PASO0: [&str; 15] = ["peor 10k hambre", "racha hambre", "% hambre desp", "% hambre dorm", "% cruda<0.15",
        "% mal tope", "% hambre prim", "% hambre ver", "% hambre oto", "% hambre inv",
        // exposición vs evitación (también solo miran)
        "% expuesta", "heridas/1k exp", "recorrido", "% pred<=2", "mordidas/1k pred<=2"];
    // las de diagnóstico ("dg ...") también solo miran
    let partes: Vec<String> = metricas(semilla, ticks, c).iter().filter(|(k, _)| !PASO0.contains(&k.as_str()) && !k.starts_with("dg ")).map(|(k, v)| format!("\"{}\": {}", k, v)).collect();
    format!("{{{}}}", partes.join(", "))
}

/// El renglón de la traza de un tick (lo mismo que arma probar_rust.py en Python).
fn linea_traza(a: &Angri, c: &Config) -> String {
    let mut l = format!("{} {} {} {} {:016x}", a.edad, a.x, a.y, a.despierto as i32, a.malestar(c).to_bits());
    for k in 0..a.cuerpo.n {
        l += &format!(" {:016x}", a.cuerpo.v[k].to_bits());
    }
    l += &format!(" {:016x} {:016x}", a.quimica.adenosina.to_bits(), a.mente.dopamina.to_bits());
    l += &format!(" {:016x} {:016x}", a.quimica.noradrenalina.to_bits(), a.oido.confianza.to_bits());
    let foco = a.talamo.as_ref().and_then(|t| t.foco).map_or(-1, |f| f as i64);
    let pedidos = a.habla.as_ref().map_or(0, |h| h.pedidos.len());
    l += &format!(" {} {} {}", foco, pedidos, a.oido.aciertos.len());
    l
}

fn traza(semilla: u64, ticks: i64, c: &Config) {
    let mut a = Angri::new(semilla, c);
    let out = std::io::stdout();
    let mut out = std::io::BufWriter::new(out.lock());
    for _ in 0..ticks {
        a.tick(c);
        writeln!(out, "{}", linea_traza(&a, c)).unwrap();
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
    let resto = if args.len() > 2 { &args[2..] } else { &[] };
    match args.get(1).map(|s| s.as_str()) {
        Some("vivir") => return vivir::vivir(resto),
        Some("medir") => return medir::medir(resto),
        Some("comparar") => return medir::comparar(resto),
        Some("juntar") => return medir::juntar(resto),
        Some("visor") => return visor::visor(resto),
        Some("vida" | "traza" | "foto" | "velocidad" | "retomar") if args.len() >= 4 => {}
        _ => {
            eprintln!("{}", AYUDA);
            std::process::exit(1);
        }
    }
    let semilla: u64 = args[2].parse().expect("semilla");
    let ticks: i64 = args[3].parse().expect("ticks");
    // resto: [CADA] (solo foto), CLAVE=VALOR..., --config ARCHIVO
    let (mut ajustes, mut archivo, mut sueltos) = (vec![], None, vec![]);
    let mut i = 4;
    while i < args.len() {
        if args[i] == "--config" {
            archivo = args.get(i + 1).cloned();
            i += 1;
        } else if args[i].contains('=') {
            ajustes.push(args[i].clone());
        } else {
            sueltos.push(args[i].clone());
        }
        i += 1;
    }
    let (_, c) = config::armar(archivo.as_deref(), &ajustes);
    match args[1].as_str() {
        "vida" => println!("{}", una_vida(semilla, ticks, &c)),
        "traza" => traza(semilla, ticks, &c),
        "foto" => {
            // con CADA, una foto cada CADA ticks
            let cada: i64 = sueltos.first().map_or(ticks.max(1), |x| x.parse().expect("cada"));
            let mut a = Angri::new(semilla, &c);
            for t in 1..=ticks {
                a.evento = Evento::Nada;
                a.tick(&c);
                if t % cada == 0 {
                    println!("{}", foto::foto(&a, &c));
                }
            }
        }
        "retomar" => {
            if !retomar(semilla, ticks, &c) {
                std::process::exit(1);
            }
        }
        _ => {
            let t0 = std::time::Instant::now();
            let mut a = Angri::new(semilla, &c);
            for _ in 0..ticks {
                a.tick(&c);
            }
            let s = t0.elapsed().as_secs_f64();
            println!("{} ticks en {:.3} s: {:.0} ticks/s (malestar final {:.6})", ticks, s, ticks as f64 / s, a.malestar(&c));
        }
    }
}

const AYUDA: &str = "uso: angri vivir [--carpeta DIR] [--ticks N] [--semilla S] [--ventana W] [--guardar G]
                   [--panel] [--foto S] [--estado] [CLAVE=VALOR ...]
     angri visor [--carpeta DIR] [--puerto P] [--host H]
     angri medir [TICKS] [N] [azar|reflejos] [CLAVE=VALOR ...] [-j J]
     angri comparar TICKS N \"AJUSTES A\" \"AJUSTES B\" [-j J] [--desde K] [--crudo SALIDA.tsv]
     angri juntar CRUDO.tsv ...
     angri vida|traza|foto|velocidad|retomar SEMILLA TICKS [CLAVE=VALOR ...]
  (todos aceptan --config ARCHIVO; la de por defecto es rust/config.txt)";
