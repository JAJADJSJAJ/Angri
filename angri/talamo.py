"""Tálamo y espacio global (etapa 5): los módulos compiten por la atención.

Cada necesidad es un módulo que grita por el foco. Su saliencia es:
- lo urgente que está (distancia ponderada a su equilibrio, normalizada),
- más un sesgo aprendido según el contexto: si lo que la alivia está a la vista
  (o, para integridad y estrés, si hay un peligro a la vista).
Gana uno por softmax, y el ganador se "difunde" a todo Angri: es la necesidad
con la que los ganglios eligen y aprenden. El foco tiene inercia (ignición): solo
cambia si otro módulo lo supera por un margen, para no picotear entre metas.

El tálamo aprende con la misma dopamina que los ganglios: si atender a X salió
mejor de lo esperado, la próxima vez X se vuelve más saliente en ese contexto.
"""
import math

from . import config as C
from .ganglios import ALIVIA, NECESIDADES_BASE as NECESIDADES, _hacia
from .mundo import PELIGRO


class Talamo:
    def __init__(self, rng, estado=None):
        self.rng = rng
        # sesgo[n][visible]: cuánto suma ser n con/sin su objetivo a la vista
        self.sesgo = {n: [0.0, 0.0] for n in NECESIDADES}
        if estado:
            for n, v in estado.items():
                self.sesgo[n] = list(v)
        self.foco = None
        self._ctx = None
        self._probs = None

    def _visible(self, vista, n):
        objetivo = ALIVIA.get(n, PELIGRO)
        return int(vista.get((0, 0)) == objetivo or _hacia(vista, objetivo) != 4)

    def elegir(self, vista, cuerpo):
        urg = {n: C.PESO_NECESIDAD[n] * abs(cuerpo[n] - C.EQUILIBRIO[n]) for n in NECESIDADES}
        tope = max(urg.values()) or 1.0
        ctx = {n: self._visible(vista, n) for n in NECESIDADES}
        sal = {n: C.GANANCIA_URGENCIA * urg[n] / tope + self.sesgo[n][ctx[n]]
               for n in NECESIDADES}
        if self.foco:
            sal[self.foco] += C.INERCIA_FOCO
        m = max(sal.values())
        e = {n: math.exp((s - m) / C.TEMPERATURA_FOCO) for n, s in sal.items()}
        z = sum(e.values())
        self._probs = {n: x / z for n, x in e.items()}
        self.foco = self.rng.choices(NECESIDADES, [self._probs[n] for n in NECESIDADES])[0]
        self._ctx = ctx
        return self.foco

    def aprender(self, dopamina):
        if self._probs is None:
            return
        for n, p in self._probs.items():
            g = (1.0 if n == self.foco else 0.0) - p
            self.sesgo[n][self._ctx[n]] += C.APRENDE_TALAMO * dopamina * g

    def olvidar_paso(self):
        self._probs = None
        self.foco = None

    def estado(self):
        return self.sesgo
