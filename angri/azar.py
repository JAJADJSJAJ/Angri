"""Azar propio y público (versión 2 del azar): PCG32 sembrado con splitmix64.

Reemplaza a random.Random para que la misma vida se pueda correr igual en
Python y en Rust (rust/src/azar.rs implementa exactamente lo mismo). Todo es
aritmética entera de 64 bits y una sola división en punto flotante exacta, así
que no depende de la plataforma.

PCG32 = "XSH RR" de M. O'Neill (pcg-random.org), estado e incremento de 64 bits.
splitmix64 (S. Vigna) arma el estado y el incremento a partir de la semilla.

Operaciones (definidas acá, no copian a las de random):
  random()            53 bits: (a<<26 | b) / 2**53, a = u32>>5, b = u32>>6
  _debajo(n)          entero uniforme en [0, n): máscara de bits y rechazo
  randrange, randint, choice, shuffle (Fisher-Yates de atrás hacia adelante),
  sample (Fisher-Yates parcial sobre una copia),
  choices (sin pesos: piso(random()*n); con pesos: acumulados + bisect_right,
           sumados en orden de izquierda a derecha).
Se guarda con angri/memoria.py como cualquier objeto: dos enteros.
"""
import bisect
import os

M64 = (1 << 64) - 1
MULT = 6364136223846793005


def splitmix64(x):
    """Devuelve (nuevo_estado, salida)."""
    x = (x + 0x9E3779B97F4A7C15) & M64
    z = x
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & M64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & M64
    return x, z ^ (z >> 31)


class Azar:
    def __init__(self, semilla=None):
        if semilla is None:
            semilla = int.from_bytes(os.urandom(8), "little")
        s = semilla & M64
        s, a = splitmix64(s)
        s, b = splitmix64(s)
        self.inc = (b << 1 | 1) & M64
        self.estado = 0
        self._u32()
        self.estado = (self.estado + a) & M64
        self._u32()

    def _u32(self):
        viejo = self.estado
        self.estado = (viejo * MULT + self.inc) & M64
        xs = (((viejo >> 18) ^ viejo) >> 27) & 0xFFFFFFFF
        rot = viejo >> 59
        return ((xs >> rot) | (xs << ((-rot) & 31))) & 0xFFFFFFFF

    def random(self):
        a = self._u32() >> 5
        b = self._u32() >> 6
        return (a * 67108864 + b) / 9007199254740992.0

    def _debajo(self, n):
        if n <= 0:
            raise ValueError("rango vacío")
        if n > 1 << 32:
            raise ValueError("rango demasiado grande para Azar")
        k = (n - 1).bit_length()
        if k == 0:
            return 0
        while True:
            r = self._u32() >> (32 - k)
            if r < n:
                return r

    def randrange(self, a, b=None):
        if b is None:
            a, b = 0, a
        return a + self._debajo(b - a)

    def randint(self, a, b):
        return a + self._debajo(b - a + 1)

    def choice(self, seq):
        return seq[self._debajo(len(seq))]

    def shuffle(self, x):
        for i in range(len(x) - 1, 0, -1):
            j = self._debajo(i + 1)
            x[i], x[j] = x[j], x[i]

    def sample(self, poblacion, k):
        pool = list(poblacion)
        n = len(pool)
        if k > n:
            raise ValueError("muestra más grande que la población")
        res = []
        for i in range(k):
            j = self._debajo(n - i)
            res.append(pool[j])
            pool[j] = pool[n - i - 1]
        return res

    def choices(self, poblacion, weights=None, k=1):
        n = len(poblacion)
        if weights is None:
            return [poblacion[int(self.random() * n)] for _ in range(k)]
        acum, t = [], 0.0
        for w in weights:
            t += w
            acum.append(t)
        return [poblacion[bisect.bisect_right(acum, self.random() * t, 0, n - 1)]
                for _ in range(k)]
