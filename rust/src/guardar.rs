//! Memoria en disco del núcleo en Rust: la mente entera, en un archivo de texto.
//!
//! Formato propio, simple: tokens separados por espacios, en el orden de los
//! campos de cada estructura (un campo por línea, con su nombre delante para
//! poder leerlo). Los floats van como sus 16 dígitos hexadecimales (bits exactos:
//! retomar da la misma vida), los mapas con las claves ordenadas.
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

pub const CABECERA: &str = "angri-rust-mente 3"; // 2: etapa 13b (objetos); 3: etapa 14 (rasgos propios)

pub struct Escritor {
    pub s: String,
}

impl Escritor {
    pub fn new() -> Escritor {
        Escritor { s: String::with_capacity(1 << 20) }
    }
    pub fn tok(&mut self, t: &str) {
        self.s.push_str(t);
        self.s.push(' ');
    }
    pub fn campo(&mut self, nombre: &str) {
        self.s.push('\n');
        self.s.push_str(nombre);
        self.s.push(' ');
    }
}

pub struct Lector<'a> {
    it: std::str::SplitAsciiWhitespace<'a>,
}

impl<'a> Lector<'a> {
    pub fn new(s: &'a str) -> Lector<'a> {
        Lector { it: s.split_ascii_whitespace() }
    }
    pub fn tok(&mut self) -> &'a str {
        self.it.next().expect("la mente guardada está cortada")
    }
    pub fn campo(&mut self, nombre: &str) {
        let t = self.tok();
        assert_eq!(t, nombre, "la mente guardada no coincide con esta versión (esperaba {nombre}, hay {t})");
    }
}

pub trait Pieza: Sized {
    fn guardar(&self, e: &mut Escritor);
    fn cargar(l: &mut Lector) -> Self;
}

macro_rules! entero {
    ($($t:ty),*) => {$(
        impl Pieza for $t {
            fn guardar(&self, e: &mut Escritor) { e.tok(&self.to_string()); }
            fn cargar(l: &mut Lector) -> Self { l.tok().parse().expect("entero") }
        }
    )*};
}
entero!(i64, u64, u32, u16, u8, usize);

impl Pieza for f64 {
    fn guardar(&self, e: &mut Escritor) {
        e.tok(&format!("{:016x}", self.to_bits()));
    }
    fn cargar(l: &mut Lector) -> Self {
        f64::from_bits(u64::from_str_radix(l.tok(), 16).expect("float"))
    }
}

impl Pieza for bool {
    fn guardar(&self, e: &mut Escritor) {
        e.tok(if *self { "1" } else { "0" });
    }
    fn cargar(l: &mut Lector) -> Self {
        l.tok() == "1"
    }
}

impl Pieza for String {
    fn guardar(&self, e: &mut Escritor) {
        e.tok(&self.len().to_string());
        e.tok(&self.bytes().map(|b| format!("{b:02x}")).collect::<String>());
    }
    fn cargar(l: &mut Lector) -> Self {
        let n: usize = l.tok().parse().unwrap();
        let h = if n == 0 { "" } else { l.tok() };
        let b: Vec<u8> = (0..n).map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap()).collect();
        String::from_utf8(b).unwrap()
    }
}

impl<A: Pieza, B: Pieza> Pieza for (A, B) {
    fn guardar(&self, e: &mut Escritor) {
        self.0.guardar(e);
        self.1.guardar(e);
    }
    fn cargar(l: &mut Lector) -> Self {
        let a = A::cargar(l);
        (a, B::cargar(l))
    }
}

impl<A: Pieza, B: Pieza, C: Pieza> Pieza for (A, B, C) {
    fn guardar(&self, e: &mut Escritor) {
        self.0.guardar(e);
        self.1.guardar(e);
        self.2.guardar(e);
    }
    fn cargar(l: &mut Lector) -> Self {
        let a = A::cargar(l);
        let b = B::cargar(l);
        (a, b, C::cargar(l))
    }
}

impl<T: Pieza, const N: usize> Pieza for [T; N] {
    fn guardar(&self, e: &mut Escritor) {
        for x in self {
            x.guardar(e);
        }
    }
    fn cargar(l: &mut Lector) -> Self {
        std::array::from_fn(|_| T::cargar(l))
    }
}

impl<T: Pieza> Pieza for Vec<T> {
    fn guardar(&self, e: &mut Escritor) {
        e.tok(&self.len().to_string());
        for x in self {
            x.guardar(e);
        }
    }
    fn cargar(l: &mut Lector) -> Self {
        let n: usize = l.tok().parse().unwrap();
        (0..n).map(|_| T::cargar(l)).collect()
    }
}

impl<T: Pieza> Pieza for VecDeque<T> {
    fn guardar(&self, e: &mut Escritor) {
        e.tok(&self.len().to_string());
        for x in self {
            x.guardar(e);
        }
    }
    fn cargar(l: &mut Lector) -> Self {
        let n: usize = l.tok().parse().unwrap();
        (0..n).map(|_| T::cargar(l)).collect()
    }
}

impl<T: Pieza> Pieza for Option<T> {
    fn guardar(&self, e: &mut Escritor) {
        match self {
            None => e.tok("-"),
            Some(x) => {
                e.tok("+");
                x.guardar(e);
            }
        }
    }
    fn cargar(l: &mut Lector) -> Self {
        if l.tok() == "+" { Some(T::cargar(l)) } else { None }
    }
}

impl<K: Pieza + Ord + Hash + Eq + Clone, V: Pieza> Pieza for HashMap<K, V> {
    fn guardar(&self, e: &mut Escritor) {
        let mut ks: Vec<&K> = self.keys().collect();
        ks.sort();
        e.tok(&ks.len().to_string());
        for k in ks {
            k.guardar(e);
            self[k].guardar(e);
        }
    }
    fn cargar(l: &mut Lector) -> Self {
        let n: usize = l.tok().parse().unwrap();
        (0..n).map(|_| {
            let k = K::cargar(l);
            (k, V::cargar(l))
        }).collect()
    }
}

impl<K: Pieza + Ord + Hash + Eq + Clone> Pieza for HashSet<K> {
    fn guardar(&self, e: &mut Escritor) {
        let mut ks: Vec<&K> = self.iter().collect();
        ks.sort();
        e.tok(&ks.len().to_string());
        for k in ks {
            k.guardar(e);
        }
    }
    fn cargar(l: &mut Lector) -> Self {
        let n: usize = l.tok().parse().unwrap();
        (0..n).map(|_| K::cargar(l)).collect()
    }
}

/// pieza!(Tipo { campo, campo, ... }): guarda y carga los campos en ese orden.
#[macro_export]
macro_rules! pieza {
    ($t:ident { $($f:ident),* $(,)? }) => {
        impl $crate::guardar::Pieza for $t {
            fn guardar(&self, e: &mut $crate::guardar::Escritor) {
                $( e.campo(stringify!($f)); $crate::guardar::Pieza::guardar(&self.$f, e); )*
            }
            fn cargar(l: &mut $crate::guardar::Lector) -> Self {
                $t { $( $f: { l.campo(stringify!($f)); $crate::guardar::Pieza::cargar(l) }, )* }
            }
        }
    };
}

/// Escribe a un archivo de forma segura: primero a .tmp y después renombra.
pub fn escribir_archivo(ruta: &std::path::Path, texto: &str) -> std::io::Result<()> {
    let tmp = ruta.with_extension("tmp");
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(texto.as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, ruta)
}
