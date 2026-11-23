"""Corteza: el modelo predictivo del mundo (etapa 3).

Antes de cada paso, Angri adivina dos cosas:

- QUÉ va a encontrar en la casilla adonde va (comida, cama, novedad, peligro, nada).
- CUÁNTO va a cambiar su malestar.

Lo adivina con lo que ve (qué había en el destino antes de moverse), la necesidad
urgente y la zona del mundo donde está. Nadie le dice qué es cada cosa: solo
compara lo que esperaba con lo que pasó y corrige (regla delta).

- Error de predicción = sorpresa real → alimenta la noradrenalina.
- Curiosidad por progreso de aprendizaje (Oudeyer / Schmidhuber): para cada tipo
  de situación guarda el error promedio reciente (rápido) y el de antes (lento).
  Si el error está BAJANDO ahí, está aprendiendo algo: eso se siente bien.
  El error en sí no premia: el ruido (un peligro que se mueve al azar) es siempre
  impredecible, el error no baja y por lo tanto no engancha.
"""
import math

from . import config as C

CELDAS = ["·", "*", "~", "?", "X", "#"]
N_ZONAS = 9  # el mundo dividido en 3×3 regiones


def _zona(pos):
    x, y = pos
    return min(3 * x // C.ANCHO, 2) * 3 + min(3 * y // C.ALTO, 2)


class Corteza:
    def __init__(self, estado=None):
        n = len(CELDAS)
        # pesos: rasgo -> [logit por celda] y rasgo -> Δmalestar esperado
        self.wc = {}
        self.wm = {}
        self.rapido = {}   # situación -> error reciente
        self.lento = {}    # situación -> error de antes
        if estado:
            self.wc = estado["wc"]
            self.wm = estado["wm"]
            self.rapido = estado["rapido"]
            self.lento = estado["lento"]
        self._n = n
        self._rasgos = None
        self._clave = None
        self._pc = None
        self._pm = 0.0
        self.error = 0.0
        self.progreso = 0.0

    def _activos(self, visto, necesidad, quieto, pos):
        return [f"v{visto}", f"v{visto}|{necesidad}", f"q{int(quieto)}|{necesidad}",
                f"z{_zona(pos)}|{visto}", "sesgo"]

    def predecir(self, visto, necesidad, quieto, pos):
        r = self._rasgos = self._activos(visto, necesidad, quieto, pos)
        self._clave = f"{visto}|{necesidad}"
        logits = [0.0] * self._n
        for k in r:
            for i, w in enumerate(self.wc.get(k, ())):
                logits[i] += w
        m = max(logits)
        e = [math.exp(l - m) for l in logits]
        z = sum(e)
        self._pc = [x / z for x in e]
        self._pm = sum(self.wm.get(k, 0.0) for k in r)

    def comparar(self, encontrado, delta_malestar):
        """Después del paso: cuánto se equivocó, corrige, y mide si está mejorando."""
        if self._rasgos is None:
            return 0.0
        j = CELDAS.index(encontrado) if encontrado in CELDAS else 0
        err_c = 1.0 - self._pc[j]
        err_m = delta_malestar - self._pm
        a = C.APRENDE_CORTEZA / len(self._rasgos)
        for k in self._rasgos:
            wc = self.wc.setdefault(k, [0.0] * self._n)
            for i in range(self._n):
                wc[i] += a * ((i == j) - self._pc[i])
            self.wm[k] = self.wm.get(k, 0.0) + a * err_m * C.ESCALA_ERROR_MALESTAR
        self.error = min(1.0, err_c + C.ESCALA_ERROR_MALESTAR * abs(err_m))
        # progreso de aprendizaje en ESTA situación
        c = self._clave
        r = self.rapido.get(c, self.error)
        l = self.lento.get(c, self.error)
        r += C.CURIOSIDAD_RAPIDO * (self.error - r)
        l += C.CURIOSIDAD_LENTO * (self.error - l)
        self.rapido[c], self.lento[c] = r, l
        self.progreso = max(0.0, l - r)
        self._rasgos = None
        return self.error

    def estado(self):
        return {"wc": self.wc, "wm": self.wm, "rapido": self.rapido, "lento": self.lento}
