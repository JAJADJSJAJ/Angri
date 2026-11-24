"""El cuerpo de Angri (hipotálamo y tronco cerebral).

Variables internas que cambian solas con el tiempo y con lo que hace.
Ningún script le dice qué hacer: solo existen estas reglas, como en un organismo.
"""
from . import config as C


def _clip(x):
    return max(0.0, min(1.0, x))


def equilibrio(k):
    e = C.EQUILIBRIO.get(k)
    return C.EQUILIBRIO_BOSQUE[k] if e is None else e


class Cuerpo:
    def __init__(self, estado=None, bosque=False):
        self.v = dict(C.EQUILIBRIO)
        if bosque:  # (etapa 13a) sed y temperatura corporal
            self.v.update(C.EQUILIBRIO_BOSQUE)
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
        # inanición: sin energía el cuerpo se consume (si no, el hambre extrema
        # sería una meseta sin salida y no habría nada que aprender)
        if v["energia"] < 0.05:
            v["integridad"] -= C.DANO_INANICION
        # hambre y daño generan estrés
        if v["energia"] < 0.3:
            v["estres"] += 0.002
        if v["integridad"] < 0.7:
            v["estres"] += 0.003
        for k in v:
            v[k] = _clip(v[k])

    def clima(self, despierto, se_movio, aire):
        """(bosque) sed y temperatura: el cuerpo tiende al calor del aire; moverse
        abriga un poco. Frío o sed extremos dañan y estresan."""
        v = self.v
        v["agua"] -= C.GASTO_AGUA * (1.0 if despierto else 0.5)
        v["calor"] += C.TEMPLA * (aire + (0.1 if se_movio else 0.0) - v["calor"])
        if v["calor"] < 0.3:
            v["integridad"] -= C.DANO_FRIO
            v["estres"] += 0.002
        # sin comida o sin agua el cuerpo no se cura y se consume: si no, el hambre
        # extrema sería una meseta sin salida (una vida quedó 2 M ticks trabada así)
        if v["agua"] < 0.05 or v["energia"] < 0.05:
            v["integridad"] -= C.RECUPERA_INTEGRIDAD + C.DANO_SED
        if v["agua"] < 0.3:
            v["estres"] += 0.002
        for k in v:
            v[k] = _clip(v[k])

    def beber(self):
        self.v["agua"] = _clip(self.v["agua"] + C.BEBER)

    def enfermar(self):
        self.v["integridad"] = _clip(self.v["integridad"] - 0.1)
        self.v["estres"] = _clip(self.v["estres"] + 0.2)
        self.v["energia"] = _clip(self.v["energia"] - 0.1)

    def lastimarse_mucho(self):
        """(bosque) el predador muerde."""
        self.v["integridad"] = _clip(self.v["integridad"] - 0.3)
        self.v["estres"] = _clip(self.v["estres"] + 0.5)

    def rasparse(self):
        """(bosque) espinas: duele poco."""
        self.v["integridad"] = _clip(self.v["integridad"] - 0.06)
        self.v["estres"] = _clip(self.v["estres"] + 0.15)

    # --- efectos del mundo ---
    def comer(self, cuanto=0.25):
        self.v["energia"] = _clip(self.v["energia"] + cuanto)

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
            C.PESO_NECESIDAD[k] * abs(self.v[k] - equilibrio(k)) for k in self.v
        )
        maximo = sum(C.PESO_NECESIDAD[k] for k in self.v)
        return min(total / maximo, C.MALESTAR_MAXIMO)

    def necesidad_urgente(self):
        """La necesidad más lejos de su equilibrio."""
        return max(
            self.v,
            key=lambda k: C.PESO_NECESIDAD[k] * abs(self.v[k] - equilibrio(k)),
        )
