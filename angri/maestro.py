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
        self.dichas = 0

    def cambiar_significados(self):
        """Prueba del ego: las palabras de dirección pasan a significar otra cosa."""
        ps = [self.palabra_dir[d] for d in DIRECCIONES]
        ps = ps[1:] + ps[:1]
        self.palabra_dir = dict(zip(DIRECCIONES, ps))

    def estado(self):
        return {"dir": self.palabra_dir, "cosa": self.palabra_cosa}

    def restaurar(self, d):
        if d:
            self.palabra_dir = dict(d["dir"])
            self.palabra_cosa = dict(d["cosa"])

    def hablar(self, mundo, x, y, necesidad, vista, edad=0):
        self.edad = edad
        """Devuelve una lista de palabras (índices) o [] si se calla."""
        objetivo = ALIVIA.get(necesidad)
        if objetivo is None or self.rng.random() >= C.PROB_HABLA:
            return []
        if C.COMPOSICION and objetivo != (COMIDA if self.edad < C.COMPOSICION else DESCANSO):
            return []  # prueba de composición: primero solo comida, después solo cama
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
        self.dichas += 1
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
    """Memoria de trabajo de lo escuchado, y el léxico: qué significa cada palabra.

    El significado se aprende por SORPRESA (Rescorla-Wagner): al oír una frase
    apuesta para dónde va a aparecer lo que busca; cuando lo ve, la diferencia
    entre lo que apostaba y lo que pasó corrige cada palabra. Una palabra que
    aparece con todas las direcciones ("comida") termina sin dirección; "norte"
    se queda con la suya. Así compone: entiende una frase que nunca oyó.

    Nunca deja de aprender del todo (piso): si un significado cambia, la
    sorpresa vuelve y reaprende. Con EGO=True aprende cada vez menos con la
    experiencia ("ya lo sé todo"), para medir qué pasa.

    Confianza en el maestro: arranca alta (como un chico) y sigue cuánto le
    sirvió lo que le dijeron. Si le mienten, baja y deja de hacerle caso.
    """

    def __init__(self):
        self.eco = {}  # palabra -> ticks que le quedan
        self.sentido = [[0.25] * 4 for _ in range(N_PALABRAS)]  # palabra -> P(dirección)
        self.veces = [0] * N_PALABRAS
        self.confianza = C.CONFIANZA_INICIAL
        self.pendiente = None  # (pos donde oyó, objetivo, frase, ticks, apuesta, era nueva)
        self.frases = set()
        self.aciertos = []  # (acertó, frase nueva)
        self.sorpresa = 0.0
        self.indice_cambio = None
        self.sorpresa_media = 0.0
        self.indice_composicion = None
        self.examen = []
        self.holistico = {}  # control HOLISTICO: frase entera -> P(dirección)
        self.pasos = 0  # para medir: dónde empezó el cambio de significado

    def estado(self):
        return {"sentido": self.sentido, "veces": self.veces, "confianza": self.confianza,
                "frases": [list(f) for f in self.frases],
                "sorpresa_media": self.sorpresa_media}

    def restaurar(self, d):
        if not d:
            return
        self.sentido = [list(s) for s in d["sentido"]]
        self.veces = list(d["veces"])
        self.confianza = d["confianza"]
        self.frases = {tuple(f) for f in d["frases"]}
        self.sorpresa_media = d.get("sorpresa_media", 0.0)

    def probar_composicion(self, maestro):
        """Examen sin aprender: las 4 frases "cama + dirección" (nunca oídas).
        ¿Apuesta bien la dirección? Guarda un acierto (bool) por frase."""
        cama = maestro.palabra_cosa[DESCANSO]
        self.examen = [max(range(4), key=self.apuesta((cama, maestro.palabra_dir[d])).__getitem__) == i
                       for i, d in enumerate(DIRECCIONES)]

    def apuesta(self, frase):
        """Para dónde cree que está, combinando lo que significa cada palabra."""
        if C.HOLISTICO:
            return list(self.holistico.get(tuple(frase), [0.25] * 4))
        prob = [1.0] * 4
        for p in frase:
            prob = [a * b for a, b in zip(prob, self.sentido[p])]
        z = sum(prob) or 1.0
        return [x / z for x in prob]

    def direccion_dicha(self):
        """La dirección que le dijeron (0..3), o 4 si no cree saberla."""
        if (not self.pendiente or self.confianza < C.CONFIANZA_MINIMA
                or self.pasos > C.PASOS_DICHA):
            return 4  # (pasos: ya caminó bastante para ese lado; no seguir de largo)
        ap = self.pendiente[4]
        d = max(range(4), key=ap.__getitem__)
        return d if ap[d] >= C.CERTEZA_DICHA else 4

    def oir(self, frase, pos=None, objetivo=None):
        for p in frase:
            self.eco[p] = C.MEMORIA_PALABRA
        if frase and pos is not None:
            f = tuple(frase)
            self.pendiente = (pos, objetivo, f, C.ESPERA_VERIFICAR, self.apuesta(f),
                              f not in self.frases)
            self.frases.add(f)
            self.pasos = 0

    def caminar(self, d):
        """Cuenta cuántos pasos dio hacia donde le dijeron."""
        if self.pendiente and d == self.direccion_dicha_cruda():
            self.pasos += 1

    def direccion_dicha_cruda(self):
        ap = self.pendiente[4]
        return max(range(4), key=ap.__getitem__)

    def ver(self, vista, pos):
        """Si aparece lo que le dijeron, compara con lo que apostaba y aprende."""
        if not self.pendiente:
            return
        (ox, oy), objetivo, frase, quedan, ap, nueva = self.pendiente
        visto = [(abs(pos[0] + dx - ox) + abs(pos[1] + dy - oy), pos[0] + dx, pos[1] + dy)
                 for (dx, dy), c in vista.items() if c == objetivo]
        if not visto:
            self.pendiente = None if quedan <= 1 else (*self.pendiente[:3], quedan - 1, ap, nueva)
            return
        _, x, y = min(visto)
        dx, dy = x - ox, y - oy
        if dx == 0 and dy == 0:
            self.pendiente = None
            return
        d = (0 if dy < 0 else 1) if abs(dy) >= abs(dx) else (2 if dx < 0 else 3)
        self.sorpresa = 1.0 - ap[d]
        self.sorpresa_media += 0.05 * (self.sorpresa - self.sorpresa_media)
        self.aciertos.append((max(range(4), key=ap.__getitem__) == d, nueva))
        # confianza: ¿lo que me dijo me sirvió más que adivinar?
        util = (ap[d] - 0.25) / 0.75
        self.confianza += C.APRENDE_CONFIANZA * (max(0.0, util) - self.confianza)
        if C.HOLISTICO:
            s = self.holistico.setdefault(frase, [0.25] * 4)
            for i in range(4):
                s[i] = max(s[i] + C.APRENDE_PALABRA * ((1.0 if i == d else 0.0) - ap[i]), 0.02)
            z = sum(s)
            self.holistico[frase] = [v / z for v in s]
            self.pendiente = None
            return
        # Rescorla-Wagner: cada palabra se corrige según el error de la frase entera
        for p in frase:
            self.veces[p] += 1
            ritmo = C.APRENDE_PALABRA
            if C.EGO:
                ritmo = ritmo / (1 + self.veces[p] / 20)
            ritmo = max(ritmo, C.APRENDE_PALABRA * C.PISO_PALABRA)
            ritmo = min(C.RITMO_MAXIMO, ritmo * (1 + C.SORPRESA_RITMO * self.sorpresa_media))
            s = self.sentido[p]
            for i in range(4):
                s[i] += ritmo * ((1.0 if i == d else 0.0) - ap[i])
                s[i] = max(s[i], 0.02)
            z = sum(s)
            self.sentido[p] = [v / z for v in s]
        self.pendiente = None

    def pasar(self):
        for p in list(self.eco):
            self.eco[p] -= 1
            if self.eco[p] <= 0:
                del self.eco[p]

    def palabras(self):
        return tuple(sorted(self.eco))
