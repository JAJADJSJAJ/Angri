"""El mundo de Angri: una grilla 2D simple.

  ·  vacío       *  comida      ~  zona de descanso
  ?  algo nuevo  X  peligro     A  Angri
"""
import random

from . import config as C

VACIO, COMIDA, DESCANSO, NOVEDAD, PELIGRO = "·", "*", "~", "?", "X"


class Mundo:
    def __init__(self, semilla=None):
        self.rng = random.Random(semilla)
        self.fuentes = []
        self.grilla = [[VACIO] * C.ANCHO for _ in range(C.ALTO)]
        # una zona fija de descanso, como una "cama"
        for y in range(1, 4):
            for x in range(1, 5):
                self.grilla[y][x] = DESCANSO
        # fuentes de comida: lugares fijos donde crece (como árboles frutales),
        # así recordar dónde comió sirve de algo
        self.fuentes = [(self.rng.randrange(C.ANCHO), self.rng.randrange(5, C.ALTO))
                        for _ in range(C.FUENTES_COMIDA)]
        for _ in range(C.COMIDA_INICIAL):
            self._poner(COMIDA)
        for _ in range(C.NOVEDADES_INICIALES):
            self._poner(NOVEDAD)
        for _ in range(C.PELIGROS):
            self._poner(PELIGRO)

    def _poner(self, cosa):
        for _ in range(100):
            if cosa == COMIDA and self.fuentes:
                fx, fy = self.rng.choice(self.fuentes)
                x = fx + self.rng.randint(-C.RADIO_FUENTE, C.RADIO_FUENTE)
                y = fy + self.rng.randint(-C.RADIO_FUENTE, C.RADIO_FUENTE)
                if not (0 <= x < C.ANCHO and 0 <= y < C.ALTO):
                    continue
            else:
                x, y = self.rng.randrange(C.ANCHO), self.rng.randrange(C.ALTO)
            if self.grilla[y][x] == VACIO:
                self.grilla[y][x] = cosa
                return

    def en(self, x, y):
        return self.grilla[y][x]

    def consumir(self, x, y):
        if self.grilla[y][x] in (COMIDA, NOVEDAD):
            self.grilla[y][x] = VACIO

    def avanzar(self):
        """El mundo también cambia solo."""
        if self.rng.random() < C.PROB_NUEVA_COMIDA and self._contar(COMIDA) < C.COMIDA_MAXIMA:
            self._poner(COMIDA)
        if self.rng.random() < C.PROB_MUEVE_PELIGRO:
            self._mover_un_peligro()
        if self.rng.random() < C.PROB_NUEVA_NOVEDAD:
            self._poner(NOVEDAD)

    def _contar(self, cosa):
        return sum(fila.count(cosa) for fila in self.grilla)

    def _mover_un_peligro(self):
        """Los peligros deambulan: lo aprendido sobre ellos no sirve para siempre."""
        peligros = [(x, y) for y, fila in enumerate(self.grilla)
                    for x, c in enumerate(fila) if c == PELIGRO]
        if not peligros:
            return
        x, y = self.rng.choice(peligros)
        dx, dy = self.rng.choice([(0, -1), (0, 1), (-1, 0), (1, 0)])
        nx, ny = x + dx, y + dy
        if 0 <= nx < C.ANCHO and 0 <= ny < C.ALTO and self.grilla[ny][nx] == VACIO:
            self.grilla[y][x], self.grilla[ny][nx] = VACIO, PELIGRO

    def vecindad(self, x, y, radio=2):
        """Lo que Angri 've' a su alrededor."""
        vista = {}
        for dy in range(-radio, radio + 1):
            for dx in range(-radio, radio + 1):
                nx, ny = x + dx, y + dy
                if 0 <= nx < C.ANCHO and 0 <= ny < C.ALTO:
                    vista[(dx, dy)] = self.grilla[ny][nx]
        return vista
