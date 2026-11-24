"""Ganglios basales: la mente que aprende (etapa 2).

Actor-crítico tabular guiado por dopamina:

- Actor (estriado dorsal): pesos sobre rasgos del movimiento (qué hay en la
  casilla destino según la necesidad urgente, si acerca a comida o a lo que
  necesita). Generaliza: lo que aprende de una X vale para todas.
- Crítico (estriado ventral): V[s] sobre una situación resumida (necesidad,
  energía, integridad, dónde está parado, si ve comida).
- Calma: aprende mucho de lo nuevo y poco de lo que ya vivió mil veces.
- Memoria de lugares: recuerda DÓNDE sintió un gran alivio (no qué lo causó) y
  el actor puede aprender a volver ahí. Los recuerdos se desvanecen.
- Dopamina = error TD: δ = r + γ·V(s') − V(s). Si salió mejor de lo esperado,
  refuerza lo que hizo; si salió peor, lo debilita. Nadie le dice qué está bien.
- Serotonina = γ: cuánto mira al futuro.
- Noradrenalina: la sorpresa sube la temperatura → explora más.
"""
import ast
import math
from angri.azar import Azar

from . import config as C
from .mundo import COMIDA, DESCANSO, NOVEDAD, PELIGRO

AGUA = "≈"
MOVIMIENTOS = [(0, -1), (0, 1), (-1, 0), (1, 0), (0, 0)]
NECESIDADES_BASE = ["energia", "cansancio", "curiosidad", "estres", "integridad"]
# (etapa 13a) sed y temperatura corporal: solo existen en el bosque. Sus rasgos
# van en un bloque aparte, así los índices de siempre no se mueven.
NECESIDADES = NECESIDADES_BASE + ["agua", "calor"]
ALIVIA = {"energia": COMIDA, "cansancio": DESCANSO, "curiosidad": NOVEDAD,
          "agua": AGUA, "calor": DESCANSO}


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


CELDAS = ["·", COMIDA, DESCANSO, NOVEDAD, PELIGRO, "#", AGUA]  # "#" = borde o roca


def _rasgos(vista, necesidad, dir_comida, dir_objetivo, dir_recuerdo, a, palabras=(), dir_dicha=4):
    """Qué 'significa' el movimiento a en esta situación (índices de rasgos activos).

    El actor no memoriza situaciones exactas: aprende el valor de rasgos simples
    ("ir a una X cuando lo urgente es la energía"), así lo aprendido en un lugar
    sirve en todos.
    """
    n = NECESIDADES.index(necesidad)
    destino = CELDAS.index(vista.get(MOVIMIENTOS[a], "#"))
    if n >= 5 or destino >= 6:
        return _rasgos_bosque(n, destino, a, dir_comida, dir_objetivo, dir_recuerdo, dir_dicha)
    r = [destino * 5 + n]                       # 0..29: qué hay adonde voy
    if a == dir_comida:
        r.append(30 + n)                        # 30..34: me acerco a comida
    if a == dir_objetivo:
        r.append(35 + n)                        # 35..39: me acerco a lo que necesito
    if a == 4:
        r.append(40 + n)                        # 40..44: me quedo quieto
    if a == dir_recuerdo:
        r.append(45 + n)                        # 45..49: voy a donde una vez me sentí bien
    r.append(50 + destino)                      # 50..55: qué hay adonde voy, sea cual sea
    if a == 4:                                  #         la necesidad ("la X duele siempre")
        r.append(56)                            # 56: me quedo quieto, sea cual sea
    if C.PALABRA_SUELTA:
        for p in palabras:                      # 57..96: oí la palabra p y hago a
            r.append(57 + p * 5 + a)            #         (etapa 6: el significado se aprende)
    if C.FRASE:
        for i, p in enumerate(palabras):        # 97..: oí p y q juntas y hago a
            for q in palabras[i + 1:]:          #       (la frase entera significa)
                r.append(97 + (p * N_PALABRAS + q) * 5 + a)
    if a == dir_dicha:
        r.append(417 + n)                       # 417..421: voy adonde me dijeron
    return r


def _rasgos_bosque(n, destino, a, dc, do, dr, dd):
    """(etapa 13a) los mismos rasgos cuando hay una necesidad nueva (agua, calor)
    o una celda nueva (agua). Lo que no depende de lo nuevo usa el índice de
    siempre (así "acercarse a comida con hambre" es el mismo rasgo aunque el
    paso caiga en el agua); lo nuevo va en un bloque desde 422."""
    r = [destino * 5 + n if destino < 6 and n < 5 else 422 + destino * 7 + n]  # 422..470
    if a == dc:
        r.append(30 + n if n < 5 else 471 + n)  # 476..477
    if a == do:
        r.append(35 + n if n < 5 else 478 + n)  # 483..484
    if a == 4:
        r.append(40 + n if n < 5 else 485 + n)  # 490..491
    if a == dr:
        r.append(45 + n if n < 5 else 492 + n)  # 497..498
    r.append(50 + destino if destino < 6 else 506)
    if a == 4:
        r.append(56)
    if a == dd:
        r.append(417 + n if n < 5 else 499 + n)  # 504..505
    return r


N_PALABRAS = 8
_RITMOS = {}  # caché de _ritmo
N_RASGOS = 507  # (hasta 422 los de siempre; 422..506 los del bosque)


def _direccion(dx, dy):
    if dx == 0 and dy == 0:
        return 4
    if abs(dy) >= abs(dx):
        return 0 if dy < 0 else 1
    return 2 if dx < 0 else 3


def _franja(x):
    return min(int(x * C.FRANJAS), C.FRANJAS - 1)


class Ganglios:
    def __init__(self, semilla=None, estado=None):
        self.rng = Azar(semilla)
        self.V = {}                    # crítico: situación resumida -> valor esperado
        self.w = [0.0] * N_RASGOS      # actor: peso de cada rasgo
        self.exp_V = {}                # experiencia: cuántas veces vivió cada situación
        self.exp_w = [0] * N_RASGOS    # y cuántas veces usó cada rasgo
        # memoria de lugares: necesidad -> {(x, y): fuerza del recuerdo}
        self.lugares = {n: {} for n in NECESIDADES}
        if estado:
            self.V = {ast.literal_eval(k): v for k, v in estado["V"].items()}
            self.w = estado["w"] + [0.0] * (N_RASGOS - len(estado["w"]))
            self.exp_V = {ast.literal_eval(k): v for k, v in estado.get("exp_V", {}).items()}
            self.exp_w = estado.get("exp_w", [])
            self.exp_w = self.exp_w + [0] * (N_RASGOS - len(self.exp_w))
            for n, lugares in estado.get("lugares", {}).items():
                self.lugares[n] = {ast.literal_eval(k): f for k, f in lugares.items()}
        self.huella_V = {}             # huellas de elegibilidad: lo reciente
        self.huella_w = [0.0] * N_RASGOS  # se lleva crédito por lo que vino después
        self._activos = set()  # rasgos que alguna vez tuvieron huella
        self.prediccion = None
        self._s = None
        self._a = None
        self._rasgos = None
        self._probs = None
        self.dopamina = 0.0  # último error TD
        self.ultimo = None   # última transición vivida

    def _resumen(self, vista, cuerpo, necesidad):
        """Lo que el crítico mira para estimar el futuro."""
        return (necesidad, _franja(cuerpo["energia"]), _franja(cuerpo["integridad"]),
                vista.get((0, 0)), _hacia(vista, COMIDA) != 4)

    def _ritmo(self, base, experiencia):
        """Cuánto aprende: mucho de lo nuevo, poco de lo que ya vivió mil veces
        (pero nunca cero: el mundo cambia)."""
        if experiencia >= 20000:  # cuenta directa (mismo número): la caché no crece sin fin
            return max(base * C.APRENDE_MINIMO, base / math.sqrt(1 + experiencia / C.CALMA))
        clave = (base, experiencia)
        r = _RITMOS.get(clave)
        if r is None:  # se calcula una vez por (ritmo, experiencia) y se guarda
            r = _RITMOS[clave] = max(base * C.APRENDE_MINIMO,
                                     base / math.sqrt(1 + experiencia / C.CALMA))
        return r

    def _recuerdo(self, necesidad, pos):
        """Dirección al lugar recordado más atractivo (fuerte y cerca) para esta necesidad."""
        mejor, valor = 4, 0.0
        x, y = pos
        for (lx, ly), f in self.lugares[necesidad].items():
            v = f / (1 + abs(lx - x) + abs(ly - y))
            if v > valor:
                mejor, valor = _direccion(lx - x, ly - y), v
        return mejor

    # --- decidir ---
    def elegir(self, vista, cuerpo, necesidad, pos, noradrenalina=0.0, malestar=0.0, palabras=(), dir_dicha=4):
        dc = _hacia(vista, COMIDA)
        do = _hacia(vista, ALIVIA.get(necesidad))
        dr = self._recuerdo(necesidad, pos)
        rasgos = [_rasgos(vista, necesidad, dc, do, dr, a, palabras, dir_dicha) for a in range(len(MOVIMIENTOS))]
        h = [sum(self.w[i] for i in r) for r in rasgos]
        # cuanto peor la pasa, más varía lo que hace: si lo de siempre no funciona,
        # probar otra cosa (antídoto contra la indefensión aprendida)
        temp = (C.TEMPERATURA_BASE + C.TEMPERATURA_SORPRESA * noradrenalina
                + C.TEMPERATURA_MALESTAR * malestar)
        m = max(h)
        if C.MARGEN_EXPLORAR or C.PELIGRO_SIN_MALESTAR:
            # la exploración por malestar no alcanza a lo que ya sabe que está mal
            t0 = C.TEMPERATURA_BASE + C.TEMPERATURA_SORPRESA * noradrenalina
            temp = [t0 if ((C.MARGEN_EXPLORAR and x < m - C.MARGEN_EXPLORAR)
                           or (C.PELIGRO_SIN_MALESTAR and vista.get(MOVIMIENTOS[b]) == PELIGRO))
                    else temp for b, x in enumerate(h)]
            e = [math.exp((x - m) / t) for x, t in zip(h, temp)]
        else:
            e = [math.exp((x - m) / temp) for x in h]
        z = sum(e)
        probs = [x / z for x in e]
        a = self.rng.choices(range(len(MOVIMIENTOS)), probs)[0]
        self._s = self._resumen(vista, cuerpo, necesidad)
        self._a, self._rasgos, self._probs, self._temp = a, rasgos, probs, temp
        self._necesidad = necesidad
        return MOVIMIENTOS[a]

    # --- aprender ---
    def aprender(self, recompensa, vista, cuerpo, necesidad, pos):
        """Se llama después de actuar, con lo que pasó y la nueva situación."""
        if self._s is None:
            return
        # memoria de lugares: los recuerdos se desvanecen; un gran alivio deja uno nuevo.
        # No sabe QUÉ lo alivió: solo recuerda DÓNDE se sintió mucho mejor.
        for lugares in self.lugares.values():
            for k in list(lugares):
                lugares[k] *= C.RECUERDO_DECAE
                if lugares[k] < 0.05:
                    del lugares[k]
        if recompensa > C.UMBRAL_RECUERDO:
            self.lugares[self._necesidad][tuple(pos)] = 1.0
        s = self._s
        s2 = self._resumen(vista, cuerpo, necesidad)
        # lo vivido, para que el hipocampo lo guarde y lo repase de noche
        self.ultimo = (s, self._rasgos, self._a, recompensa, s2, tuple(pos), self._necesidad,
                       self._probs[self._a], self._temp)
        # un lugar recordado al que llega y no hay nada de lo que necesitaba: se borra ya
        if C.BORRAR_LUGAR_VACIO:
            objetivo = ALIVIA.get(necesidad)
            lugares = self.lugares[necesidad]
            if (tuple(pos) in lugares and objetivo
                    and vista.get((0, 0)) != objetivo and _hacia(vista, objetivo) == 4):
                del lugares[tuple(pos)]
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
        self.exp_V[s] = self.exp_V.get(s, 0) + 1
        for k, e in self.huella_V.items():
            ritmo = self._ritmo(C.APRENDE_CRITICO, self.exp_V.get(k, 0))
            self.V[k] = self.V.get(k, 0.0) + ritmo * delta * e
        # actor: gradiente de softmax (sube lo que hizo, baja lo esperado), con huella
        # (solo recorre los rasgos que alguna vez se usaron: los demás tienen huella 0
        #  exacta y no cambian; da idéntico y es mucho más rápido)
        hw = self.huella_w
        activos = self._activos
        for i in activos:
            hw[i] *= decae
        for i in self._rasgos[self._a]:
            hw[i] += 1.0
            self.exp_w[i] += 1
        for b, p in enumerate(self._probs):
            for i in self._rasgos[b]:
                hw[i] -= p
                activos.add(i)
        for i in activos:
            self.w[i] += self._ritmo(C.APRENDE_ACTOR, self.exp_w[i]) * delta * hw[i]

    def repasar(self, momento, fraccion):
        """Replay: vuelve a aprender de un momento guardado, sin vivirlo.

        Sin huellas (cada momento suelto) y con las probabilidades de HOY: si ya
        aprendió algo nuevo, el recuerdo se reinterpreta con lo que sabe ahora.
        """
        s, rasgos, a, r, s2 = momento[:5]
        p_entonces, temp = momento[7], momento[8]
        h = [sum(self.w[i] for i in rs) for rs in rasgos]
        m = max(h)
        if isinstance(temp, list):
            e = [math.exp((x - m) / t) for x, t in zip(h, temp)]
        else:
            e = [math.exp((x - m) / temp) for x in h]
        z = sum(e)
        # corrección por importancia: si hoy ya no haría eso, el recuerdo pesa menos
        rho = min(C.TOPE_IMPORTANCIA, (e[a] / z) / p_entonces) if C.IMPORTANCIA else 1.0
        delta = r + C.SEROTONINA * self.V.get(s2, 0.0) - self.V.get(s, 0.0)
        self.V[s] = self.V.get(s, 0.0) + fraccion * rho * self._ritmo(
            C.APRENDE_CRITICO, self.exp_V.get(s, 0)) * delta
        if not C.REPASO_ACTOR:
            return delta
        grad = {}  # solo los rasgos que tocan (mismas cuentas, en el mismo orden)
        for i in rasgos[a]:
            grad[i] = grad.get(i, 0.0) + 1.0
        for b, x in enumerate(e):
            for i in rasgos[b]:
                grad[i] = grad.get(i, 0.0) - x / z
        for i, g in grad.items():
            if g:
                self.w[i] += fraccion * rho * self._ritmo(C.APRENDE_ACTOR, self.exp_w[i]) * delta * g
        return delta

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
        return {"V": {repr(k): v for k, v in self.V.items()}, "w": self.w,
                "exp_V": {repr(k): v for k, v in self.exp_V.items()}, "exp_w": self.exp_w,
                "lugares": {n: {repr(k): f for k, f in l.items()}
                            for n, l in self.lugares.items()}}
