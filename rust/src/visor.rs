//! Visor web (antes visor.py): mirar a Angri vivir desde el navegador.
//!
//!   angri visor [--carpeta DIR] [--puerto P] [--host H]   -> http://H:P (127.0.0.1:8080)
//!
//! No toca la mente: solo lee DIR/foto.json, que `angri vivir` escribe cada
//! tanto. Si la vida está parada, muestra la última foto. El dibujo (pixel art)
//! lo hace el navegador con código (visor.html, metido en el binario): no se
//! baja nada de afuera. HTTP mínimo con la biblioteca estándar, un hilo por pedido.
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

const PAGINA: &str = include_str!("visor.html");

fn responder(mut s: TcpStream, foto: &Path) {
    let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(5)));
    let mut buf = [0u8; 4096];
    let mut pedido = Vec::new();
    while !pedido.windows(4).any(|w| w == b"\r\n\r\n") && pedido.len() < 16384 {
        match s.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => pedido.extend_from_slice(&buf[..n]),
        }
    }
    let texto = String::from_utf8_lossy(&pedido);
    let mut partes = texto.split_whitespace();
    let (metodo, ruta) = (partes.next().unwrap_or(""), partes.next().unwrap_or(""));
    let ruta = ruta.split('?').next().unwrap_or("");
    let (estado, tipo, cuerpo): (&str, &str, Vec<u8>) = if metodo != "GET" {
        ("405 Method Not Allowed", "text/plain; charset=utf-8", b"solo GET".to_vec())
    } else if ruta == "/" || ruta == "/index.html" {
        ("200 OK", "text/html; charset=utf-8", PAGINA.as_bytes().to_vec())
    } else if ruta == "/foto.json" {
        match std::fs::read(foto) {
            Ok(b) => ("200 OK", "application/json; charset=utf-8", b),
            Err(_) => ("404 Not Found", "text/plain; charset=utf-8", "todavía no hay foto".as_bytes().to_vec()),
        }
    } else {
        ("404 Not Found", "text/plain; charset=utf-8", b"no existe".to_vec())
    };
    let cab = format!("HTTP/1.1 {estado}\r\nContent-Type: {tipo}\r\nContent-Length: {}\r\n\
                       Cache-Control: no-store\r\nConnection: close\r\n\r\n", cuerpo.len());
    let _ = s.write_all(cab.as_bytes());
    let _ = s.write_all(&cuerpo);
}

pub fn visor(args: &[String]) {
    let mut carpeta = PathBuf::from(std::env::var("ANGRI_DATOS").unwrap_or_else(|_| "datos-rust".into()));
    let (mut puerto, mut host) = (8080u16, "127.0.0.1".to_string());
    let mut i = 0;
    while i < args.len() {
        let v = args.get(i + 1).cloned().unwrap_or_default();
        match args[i].as_str() {
            "--carpeta" => carpeta = PathBuf::from(v),
            "--puerto" => puerto = v.parse().expect("--puerto"),
            "--host" => host = v,
            o => {
                eprintln!("opción desconocida: {o}");
                std::process::exit(1);
            }
        }
        i += 2;
    }
    let foto = carpeta.join("foto.json");
    let l = TcpListener::bind((host.as_str(), puerto)).unwrap_or_else(|e| {
        eprintln!("no puedo escuchar en {host}:{puerto} ({e})");
        std::process::exit(1)
    });
    println!("Visor en http://{host}:{puerto} leyendo {}", foto.display());
    for s in l.incoming().flatten() {
        let f = foto.clone();
        std::thread::spawn(move || responder(s, &f));
    }
}
