"""Lenguaje (etapa 6): un maestro chico que le habla, y Angri aprende qué significa.

El maestro es un "modelo chico" con reglas: ve TODO el mundo (Angri solo ve 5x5)
y, cuando lo que Angri necesita no está a la vista, a veces le dice dos palabras:
qué es y para dónde queda ("ñam ku" = comida al norte). Las palabras son sílabas
inventadas y mezcladas con la semilla: Angri no nace sabiendo ninguna.

Angri no tiene diccionario. Lo que escucha queda unos ticks en la memoria de
trabajo, y el actor de los ganglios tiene un rasgo por (palabra, movimiento):
si moverse al norte después de oír "ku" sale mejor de lo esperado, la dopamina
lo refuerza. El significado se aprende igual que todo lo demás.

Con MAESTRO_MIENTE=True el maestro dice palabras al azar: es el control para
saber si lo que ayuda es el contenido o solo oír algo.
"""
import random

from . import config as C
from .ganglios import ALIVIA, NECESIDADES
from .mundo import COMIDA, DESCANSO, NOVEDAD

SILABAS = ["ku", "ba", "ti", "mo", "ñam", "ze", "lu", "pi"]
COSAS = [COMIDA, DESCANSO, NOVEDAD]
DIRECCIONES = ["norte", "sur", "oeste", "este"]  # mismo orden que MOVIMIENTOS[0..3]
N_PALABRAS = len(SILABAS)


class Maestro:
    def __init__(self, semilla=None):
        self.rng = random.Random(None if semilla is None else semilla + 7)
        orden = list(range(N_PALABRAS))
        self.rng.shuffle(orden)
        # significado secreto (solo el maestro lo sabe): palabra -> índice
        self.palabra_dir = {d: orden[i] for i, d in enumerate(DIRECCIONES)}
        self.palabra_cosa = {c: orden[4 + i] for i, c in enumerate(COSAS)}
        self.dicho = None  # última frase, para el panel

    def hablar(self, mundo, x, y, necesidad, vista):
        """Devuelve una lista de palabras (índices) o [] si se calla."""
        objetivo = ALIVIA.get(necesidad)
        if objetivo is None or self.rng.random() >= C.PROB_HABLA:
            return []
        if any(c == objetivo for c in vista.values()):
            return []  # ya lo ve: no hace falta decir nada
        lugar = self._cerca(mundo, x, y, objetivo)
        if lugar is None:
            return []
        dx, dy = lugar[0] - x, lugar[1] - y
        if abs(dy) >= abs(dx):
            d = "norte" if dy < 0 else "sur"
        else:
            d = "oeste" if dx < 0 else "este"
        frase = [self.palabra_cosa[objetivo], self.palabra_dir[d]]
        if C.MAESTRO_MIENTE:
            frase = [self.rng.randrange(N_PALABRAS) for _ in frase]
        self.dicho = " ".join(SILABAS[p] for p in frase)
        return frase

    @staticmethod
    def _cerca(mundo, x, y, objetivo):
        mejor, dist = None, 10**9
        for yy, fila in enumerate(mundo.grilla):
            for xx, c in enumerate(fila):
                if c == objetivo:
                    d = abs(xx - x) + abs(yy - y)
                    if d < dist:
                        mejor, dist = (xx, yy), d
        return mejor


class Oido:
    """Memoria de trabajo de lo escuchado: cada palabra se apaga a los pocos ticks."""

    def __init__(self):
        self.eco = {}  # palabra -> ticks que le quedan

    def oir(self, frase):
        for p in frase:
            self.eco[p] = C.MEMORIA_PALABRA

    def pasar(self):
        for p in list(self.eco):
            self.eco[p] -= 1
            if self.eco[p] <= 0:
                del self.eco[p]

    def palabras(self):
        return tuple(sorted(self.eco))
