"""El cuerpo de Angri (hipotálamo y tronco cerebral).

Variables internas que cambian solas con el tiempo y con lo que hace.
Ningún script le dice qué hacer: solo existen estas reglas, como en un organismo.
"""
from . import config as C


def _clip(x):
    return max(0.0, min(1.0, x))


class Cuerpo:
    def __init__(self, estado=None):
        self.v = dict(C.EQUILIBRIO)
        if estado:
            self.v.update(estado)

    # --- dinámica propia ---
    def vivir(self, despierto, se_movio):
        v = self.v
        v["energia"] -= C.GASTO_ENERGIA_BASE + (C.GASTO_ENERGIA_MOVER if se_movio else 0)
        if despierto:
            v["cansancio"] += C.SUBE_CANSANCIO_DESPIERTO
            v["curiosidad"] += C.SUBE_CURIOSIDAD
        else:
            v["cansancio"] -= C.SUBE_CANSANCIO_DESPIERTO * 4
            v["energia"] += C.GASTO_ENERGIA_BASE * 0.5  # durmiendo gasta menos
        v["estres"] -= C.CALMA_ESTRES * v["estres"]
        v["integridad"] += C.RECUPERA_INTEGRIDAD
        # hambre y daño generan estrés
        if v["energia"] < 0.3:
            v["estres"] += 0.002
        if v["integridad"] < 0.7:
            v["estres"] += 0.003
        for k in v:
            v[k] = _clip(v[k])

    # --- efectos del mundo ---
    def comer(self):
        self.v["energia"] = _clip(self.v["energia"] + 0.25)

    def descansar(self):
        self.v["cansancio"] = _clip(self.v["cansancio"] - 0.01)

    def descubrir(self):
        self.v["curiosidad"] = _clip(self.v["curiosidad"] - 0.3)

    def lastimarse(self):
        self.v["integridad"] = _clip(self.v["integridad"] - 0.15)
        self.v["estres"] = _clip(self.v["estres"] + 0.3)

    # --- cómo se siente ---
    def malestar(self):
        """Distancia ponderada al equilibrio (0 = perfecto). Es el 'afecto' de Solms."""
        total = sum(
            C.PESO_NECESIDAD[k] * abs(self.v[k] - C.EQUILIBRIO[k]) for k in self.v
        )
        maximo = sum(C.PESO_NECESIDAD.values())
        return min(total / maximo, C.MALESTAR_MAXIMO)

    def necesidad_urgente(self):
        """La necesidad más lejos de su equilibrio."""
        return max(
            self.v,
            key=lambda k: C.PESO_NECESIDAD[k] * abs(self.v[k] - C.EQUILIBRIO[k]),
        )
