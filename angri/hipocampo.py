"""Hipocampo (etapa 4): guarda el día y lo repasa al dormir.

Despierto: anota cada momento vivido (situación, qué hizo, cómo le fue).
Dormido:
- Replay: repasa el día en desorden, los momentos más intensos primero, y los
  ganglios vuelven a aprender de ellos sin tener que vivirlos otra vez.
- Lugares: donde sintió un gran alivio queda un lugar recordado (para esa
  necesidad). Si vuelve a pasar, el recuerdo se refuerza; si no, la poda lo
  debilita noche a noche. Así olvida las fuentes de comida que se secaron.
- Al despertar el día se borra: lo que valía la pena ya pasó a los ganglios.
"""
from . import config as C


class Hipocampo:
    def __init__(self, rng):
        self.rng = rng
        self.dia = []
        self.ultima_noche = {"repasos": 0, "lugares": 0, "dopamina": 0.0}

    def anotar(self, momento):
        self.dia.append(momento)
        if len(self.dia) > C.CAPACIDAD_DIA:
            self.dia.pop(0)

    def dormir(self, ganglios):
        # poda: lo que no se reconfirma hoy se va borrando
        for lugares in ganglios.lugares.values():
            for k in list(lugares):
                lugares[k] *= C.PODA_LUGARES
                if lugares[k] < 0.05:
                    del lugares[k]
        # lugares del día: dónde se sintió mucho mejor
        nuevos = 0
        for s, _, _, r, _, pos, necesidad, *_ in self.dia:
            if r > C.UMBRAL_LUGAR:
                l = ganglios.lugares[necesidad]
                l[pos] = min(C.TOPE_LUGAR, l.get(pos, 0.0) + 1.0)
                nuevos += 1
        # replay: lo intenso pesa más (como las ondas agudas, que priorizan lo saliente)
        total = 0.0
        n = 0
        if self.dia:
            pesos = [C.PRIORIDAD_BASE + abs(m[3]) for m in self.dia]
            k = C.REPASOS * len(self.dia)
            for m in self.rng.choices(self.dia, pesos, k=k):
                total += abs(ganglios.repasar(m, C.APRENDE_REPASO))
                n += 1
        self.ultima_noche = {"repasos": n, "lugares": nuevos,
                             "dopamina": total / n if n else 0.0}
        self.dia = []
