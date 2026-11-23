"""Ganglios basales: la mente que aprende (etapa 2).

Actor-crítico tabular guiado por dopamina:

- Actor (estriado dorsal): pesos sobre rasgos del movimiento (qué hay en la
  casilla destino según la necesidad urgente, si acerca a comida o a lo que
  necesita). Generaliza: lo que aprende de una X vale para todas.
- Crítico (estriado ventral): V[s] sobre una situación resumida (necesidad,
  energía, integridad, dónde está parado, si ve comida).
- Dopamina = error TD: δ = r + γ·V(s') − V(s). Si salió mejor de lo esperado,
  refuerza lo que hizo; si salió peor, lo debilita. Nadie le dice qué está bien.
- Serotonina = γ: cuánto mira al futuro.
- Noradrenalina: la sorpresa sube la temperatura → explora más.
"""
import ast
import math
import random

from . import config as C
from .mundo import COMIDA, DESCANSO, NOVEDAD, PELIGRO

MOVIMIENTOS = [(0, -1), (0, 1), (-1, 0), (1, 0), (0, 0)]
NECESIDADES = ["energia", "cansancio", "curiosidad", "estres", "integridad"]
ALIVIA = {"energia": COMIDA, "cansancio": DESCANSO, "curiosidad": NOVEDAD}


def _hacia(vista, objetivo):
    """Dirección (índice de MOVIMIENTOS, o 4 = no se ve) al objetivo más cercano."""
    if objetivo is None:
        return 4
    mejor, dist = 4, 99
    for (dx, dy), c in vista.items():
        if c == objetivo and (dx, dy) != (0, 0):
            d = abs(dx) + abs(dy)
            if d < dist:
                dist = d
                if abs(dy) >= abs(dx):
                    mejor = 0 if dy < 0 else 1
                else:
                    mejor = 2 if dx < 0 else 3
    return mejor


CELDAS = ["·", COMIDA, DESCANSO, NOVEDAD, PELIGRO, "#"]  # "#" = borde


def _rasgos(vista, necesidad, dir_comida, dir_objetivo, a):
    """Qué 'significa' el movimiento a en esta situación (índices de rasgos activos).

    El actor no memoriza situaciones exactas: aprende el valor de rasgos simples
    ("ir a una X cuando lo urgente es la energía"), así lo aprendido en un lugar
    sirve en todos.
    """
    n = NECESIDADES.index(necesidad)
    destino = CELDAS.index(vista.get(MOVIMIENTOS[a], "#"))
    r = [destino * 5 + n]                       # 0..29: qué hay adonde voy
    if a == dir_comida:
        r.append(30 + n)                        # 30..34: me acerco a comida
    if a == dir_objetivo:
        r.append(35 + n)                        # 35..39: me acerco a lo que necesito
    if a == 4:
        r.append(40 + n)                        # 40..44: me quedo quieto
    return r


N_RASGOS = 45


def _franja(x):
    return min(int(x * 4), 3)


class Ganglios:
    def __init__(self, semilla=None, estado=None):
        self.rng = random.Random(semilla)
        self.V = {}                    # crítico: situación resumida -> valor esperado
        self.w = [0.0] * N_RASGOS      # actor: peso de cada rasgo
        if estado:
            self.V = {ast.literal_eval(k): v for k, v in estado["V"].items()}
            self.w = estado["w"]
        self.huella_V = {}             # huellas de elegibilidad: lo reciente
        self.huella_w = [0.0] * N_RASGOS  # se lleva crédito por lo que vino después
        self.prediccion = None
        self._s = None
        self._a = None
        self._rasgos = None
        self._probs = None
        self.dopamina = 0.0  # último error TD

    def _resumen(self, vista, cuerpo, necesidad):
        """Lo que el crítico mira para estimar el futuro."""
        return (necesidad, _franja(cuerpo["energia"]), _franja(cuerpo["integridad"]),
                vista.get((0, 0)), _hacia(vista, COMIDA) != 4)

    # --- decidir ---
    def elegir(self, vista, cuerpo, necesidad, noradrenalina=0.0):
        dc = _hacia(vista, COMIDA)
        do = _hacia(vista, ALIVIA.get(necesidad))
        rasgos = [_rasgos(vista, necesidad, dc, do, a) for a in range(len(MOVIMIENTOS))]
        h = [sum(self.w[i] for i in r) for r in rasgos]
        temp = C.TEMPERATURA_BASE + C.TEMPERATURA_SORPRESA * noradrenalina
        m = max(h)
        e = [math.exp((x - m) / temp) for x in h]
        z = sum(e)
        probs = [x / z for x in e]
        a = self.rng.choices(range(len(MOVIMIENTOS)), probs)[0]
        self._s = self._resumen(vista, cuerpo, necesidad)
        self._a, self._rasgos, self._probs = a, rasgos, probs
        return MOVIMIENTOS[a]

    # --- aprender ---
    def aprender(self, recompensa, vista, cuerpo, necesidad):
        """Se llama después de actuar, con lo que pasó y la nueva situación."""
        if self._s is None:
            return
        s = self._s
        s2 = self._resumen(vista, cuerpo, necesidad)
        v = self.V.get(s, 0.0)
        delta = recompensa + C.SEROTONINA * self.V.get(s2, 0.0) - v
        self.dopamina = delta
        decae = C.SEROTONINA * C.HUELLA
        # crítico con huellas
        for k in list(self.huella_V):
            self.huella_V[k] *= decae
            if self.huella_V[k] < 0.01:
                del self.huella_V[k]
        self.huella_V[s] = 1.0
        for k, e in self.huella_V.items():
            self.V[k] = self.V.get(k, 0.0) + C.APRENDE_CRITICO * delta * e
        # actor: gradiente de softmax (sube lo que hizo, baja lo esperado), con huella
        hw = self.huella_w
        for i in range(N_RASGOS):
            hw[i] *= decae
        for i in self._rasgos[self._a]:
            hw[i] += 1.0
        for b, p in enumerate(self._probs):
            for i in self._rasgos[b]:
                hw[i] -= p
        paso = C.APRENDE_ACTOR * delta
        for i in range(N_RASGOS):
            self.w[i] += paso * hw[i]

    def olvidar_paso(self):
        """Al dormirse: lo último que hizo no se encadena con lo que viene al despertar."""
        self._s = None
        self.huella_V.clear()
        self.huella_w = [0.0] * N_RASGOS

    def sorpresa(self, encontrado):
        esperado = self.prediccion
        self.prediccion = encontrado
        if esperado is None:
            return 0.0
        return 0.0 if encontrado == esperado else 1.0

    def estado(self):
        return {"V": {repr(k): v for k, v in self.V.items()}, "w": self.w}
