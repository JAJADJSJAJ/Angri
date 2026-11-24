"""Lenguaje (etapa 6): un maestro chico que le habla, y Angri aprende qué significa.

El maestro es un "modelo chico" con reglas: ve TODO el mundo (Angri solo ve 5x5)
y, cuando lo que Angri necesita no está a la vista, a veces le dice dos palabras:
qué es y para dónde queda ("comida norte"). Desde la etapa 7 las palabras son
en español, pero el índice de cada una está mezclado con la semilla y Angri no
nace sabiendo ninguna: el significado lo aprende.

Angri no tiene diccionario. Lo que escucha queda unos ticks en la memoria de
trabajo, y el actor de los ganglios tiene un rasgo por (palabra, movimiento):
si moverse al norte después de oír "ku" sale mejor de lo esperado, la dopamina
lo refuerza. El significado se aprende igual que todo lo demás.

Con MAESTRO_MIENTE=True el maestro dice palabras al azar: es el control para
saber si lo que ayuda es el contenido o solo oír algo.
"""
import math
from angri.azar import Azar

from . import config as C
from .ganglios import ALIVIA, NECESIDADES_BASE as NECESIDADES
from .mundo import COMIDA, DESCANSO, NOVEDAD

COSAS = [COMIDA, DESCANSO, NOVEDAD]
DIRECCIONES = ["norte", "sur", "oeste", "este"]  # mismo orden que MOVIMIENTOS[0..3]
# Etapa 7: palabras reales en español. Son solo la etiqueta que se muestra: el
# índice de cada palabra sigue mezclado por semilla y Angri no sabe qué significa.
NOMBRE_COSA = {COMIDA: "comida", DESCANSO: "cama", NOVEDAD: "novedad"}
N_PALABRAS = 8  # 4 direcciones + 3 cosas + 1 que nunca se usa ("che")


class Maestro:
    def __init__(self, semilla=None):
        self.rng = Azar(None if semilla is None else semilla + 7)
        orden = list(range(N_PALABRAS))
        self.rng.shuffle(orden)
        # significado secreto (solo el maestro lo sabe): palabra -> índice
        self.palabra_dir = {d: orden[i] for i, d in enumerate(DIRECCIONES)}
        self.palabra_cosa = {c: orden[4 + i] for i, c in enumerate(COSAS)}
        self.palabras = ["che"] * N_PALABRAS  # índice -> cómo suena
        for d, p in self.palabra_dir.items():
            self.palabras[p] = d
        for c, p in self.palabra_cosa.items():
            self.palabras[p] = NOMBRE_COSA[c]
        self.objetivo = None
        self.dicho = None  # última frase, para el panel
        self.dichas = 0

    def cambiar_significados(self):
        """Prueba del ego: las palabras de dirección pasan a significar otra cosa."""
        ps = [self.palabra_dir[d] for d in DIRECCIONES]
        ps = ps[1:] + ps[:1]
        self.palabra_dir = dict(zip(DIRECCIONES, ps))
        for d, p in self.palabra_dir.items():
            self.palabras[p] = d

    def estado(self):
        return {"dir": self.palabra_dir, "cosa": self.palabra_cosa, "palabras": self.palabras}

    def restaurar(self, d):
        if d:
            self.palabra_dir = dict(d["dir"])
            self.palabra_cosa = dict(d["cosa"])
            self.palabras = list(d.get("palabras", self.palabras))

    def hablar(self, mundo, x, y, necesidad, vista, edad=0, pedido=None):
        """Devuelve una lista de palabras (índices) o [] si se calla.
        pedido: la palabra que dijo Angri (índice); si es una cosa, contesta
        por esa cosa aunque no sea lo que Angri más necesita."""
        self.edad = edad
        self.objetivo = None
        # siempre los mismos sorteos: así pedir o no pedir no cambia el resto de la vida
        u_esp, u_habla, u_cosa = self.rng.random(), self.rng.random(), self.rng.randrange(3)
        cosa = {p: c for c, p in self.palabra_cosa.items()}.get(pedido)
        if cosa is not None:
            objetivo = cosa
        elif C.MAESTRO_SOLO_SI_PIDE and not (
                self.edad < C.MAESTRO_INICIO or u_esp < C.PROB_ESPONTANEO):
            return []  # (salvo al principio o de vez en cuando: así Angri oye antes de pedir)
        else:
            # maestro ciego: no sabe qué necesita Angri, habla de cualquier cosa
            objetivo = COSAS[u_cosa] if C.MAESTRO_CIEGO else ALIVIA.get(necesidad)
            if objetivo is None or objetivo not in self.palabra_cosa or u_habla >= C.PROB_HABLA:
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
        self.objetivo = objetivo
        self.dichas += 1
        if C.MAESTRO_MIENTE:
            frase = [self.rng.randrange(N_PALABRAS) for _ in frase]
        self.dicho = " ".join(self.palabras[p] for p in frase)
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


class Habla:
    """Etapa 7: Angri pide. Cuando lo que necesita no está a la vista, a veces
    (PROB_HABLA, el mismo ritmo con que habla el maestro) elige entre decir una
    de las 3 palabras de cosas o callarse. Nace eligiendo al azar: un actor con
    un rasgo por (necesidad, palabra) que aprende con la misma dopamina de los
    ganglios, sumada en los ticks que siguen a lo que dijo.
    Con PIDE_AZAR=True elige al azar siempre (control)."""

    def __init__(self, rng, maestro):
        self.rng = rng
        self.opciones = [maestro.palabra_cosa[c] for c in COSAS] + [None]  # None = callarse
        self.w = {n: [0.0] * len(self.opciones) for n in NECESIDADES}
        self.huella = None  # [necesidad, opción, ticks que quedan]
        # Etapa 8: lo que oyó. assoc[necesidad][opción]: cuánto espera oír esa
        # palabra cuando siente eso (Rescorla-Wagner, como el léxico)
        self.assoc = {n: [0.0] * 3 for n in NECESIDADES}
        self.oidas = 0
        self.edad = 0
        self.ultima = None  # (palabras de cosa oídas, edad): para aprender por alivio
        self.dicho = None
        self.pedidos = []  # (pidió lo correcto para su necesidad, edad)

    def estado(self):
        return {"w": self.w, "assoc": self.assoc}

    def restaurar(self, d):
        if d:
            self.w = {k: list(v) for k, v in d["w"].items()}
            if "assoc" in d:
                self.assoc = {k: list(v) for k, v in d["assoc"].items()}

    @staticmethod
    def _sentir(cuerpo, foco):
        """Lo que siente: el foco (uno) o el estado del cuerpo (déficit de cada necesidad)."""
        if not C.ESCUCHA_CUERPO or cuerpo is None:
            return {foco: 1.0} if foco in NECESIDADES else {}
        x = {k: C.PESO_NECESIDAD[k] * abs(cuerpo[k] - C.EQUILIBRIO[k]) for k in NECESIDADES}
        z = sum(x.values()) or 1.0
        return {k: v / z for k, v in x.items()}

    def escuchar(self, frase, cuerpo, foco, pedido):
        """El maestro habló: refuerza necesidad -> palabra de cosa que oyó (sorpresa)."""
        if frase:
            self.ultima = ([p in frase for p in self.opciones[:3]], self.edad)
        if (not C.ESCUCHA or not C.ESCUCHA_OIDO or not frase
                or (pedido is not None and not C.ESCUCHA_PEDIDOS)):
            return
        x = self._sentir(cuerpo, foco)
        oyo = [1.0 if p in frase else 0.0 for p in self.opciones[:3]]
        for i in range(3):
            err = oyo[i] - sum(v * self.assoc[n][i] for n, v in x.items())
            for n, v in x.items():
                self.assoc[n][i] += C.APRENDE_ESCUCHA * v * err
        self.oidas += 1

    def aliviado(self, necesidad, edad):
        """Se le alivió una necesidad (comió, durmió, descubrió). Si hace poco oyó una
        palabra de cosa, esa palabra queda asociada con lo que se le alivió."""
        if not (C.ESCUCHA and C.ESCUCHA_ALIVIO) or self.ultima is None:
            return
        oyo, cuando = self.ultima
        if edad - cuando > C.VENTANA_ALIVIO:
            return
        a = self.assoc[necesidad]
        for i in range(3):
            a[i] += C.APRENDE_ESCUCHA * ((1.0 if oyo[i] else 0.0) - a[i])
        self.ultima = None

    def decidir(self, necesidad, vista, edad, cuerpo=None):
        """Devuelve el índice de la palabra que dice, o None."""
        self.edad = edad
        objetivo = ALIVIA.get(necesidad)
        if (objetivo is None or self.huella is not None
                or any(c == objetivo for c in vista.values())
                or self.rng.random() >= C.PROB_HABLA):
            return None
        w = self.w[necesidad]
        if C.PIDE_AZAR:
            i = self.rng.randrange(len(self.opciones))
        elif C.ESCUCHA:
            x = self._sentir(cuerpo, necesidad)
            a = [sum(v * self.assoc[n][j] for n, v in x.items()) for j in range(3)]
            if max(a) < C.UMBRAL_ESCUCHA:
                return None  # todavía no sabe cómo se dice: se calla
            m = max(a)
            ps = [math.exp((v - m) / C.TEMPERATURA_ESCUCHA) for v in a]
            r, i = self.rng.random() * sum(ps), 0
            while i < 2 and r > ps[i]:
                r -= ps[i]
                i += 1
        else:
            m = max(w)
            ps = [math.exp((x - m) / C.TEMPERATURA_HABLA) for x in w]
            r, i = self.rng.random() * sum(ps), 0
            while i < len(ps) - 1 and r > ps[i]:
                r -= ps[i]
                i += 1
        self.huella = [necesidad, i, C.VENTANA_HABLA]
        p = self.opciones[i]
        if p is not None:
            self.pedidos.append((i == COSAS.index(objetivo), edad))
        return p

    def aprender(self, dopamina):
        if self.huella is None:
            return
        n, i, quedan = self.huella
        if not C.PIDE_AZAR and not C.ESCUCHA:
            self.w[n][i] += C.APRENDE_HABLA * dopamina
        self.huella = None if quedan <= 1 else [n, i, quedan - 1]


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
