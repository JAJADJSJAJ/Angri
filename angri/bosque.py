"""El bosque (etapa 13a): un mundo grande con dependencias reales.

64×64 con zonas (bosque, claro, río, roca con cuevas, llanura), reloj del
mundo (día/noche y estaciones), temperatura y lluvia. La comida cuesta: frutas
que solo dan en su estación y vuelven a crecer, nueces que tardan en abrirse,
raíces que hay que escarbar, bayas que a veces enferman. Hay agua (río y
charcos), espinas, un predador que caza más de noche siguiendo el olor de
Angri, y el río a veces crece.

El mundo NO sabe para qué sirve nada: la cueva vale porque adentro no llueve
ni hace frío, y el predador no entra porque no cabe. Lo demás lo pone el cuerpo.

Para la mente, cada celda se muestra con el mismo vocabulario de antes
(·  *  ~  ?  X  #) más ≈ (agua). Lo que importa de verdad (qué comida es, si
está en estación, si el río creció) se descubre viviendo.

Sentidos (`sentir`): la vista alcanza 2 casillas de día y 1 de noche; además,
de cada tipo de estímulo que no se ve, el más fuerte que llega por otro
sentido se pone en su lugar real (dirección y distancia): olfato de comida
(6 casillas), humedad del agua (8), olor del predador (5) y su ruido de noche
(8). El tacto y la temperatura llegan por el cuerpo.
"""
import math
from angri.azar import Azar

from . import config as C

VACIO, COMIDA, DESCANSO, NOVEDAD, PELIGRO = "·", "*", "~", "?", "X"
AGUA, ROCA = "≈", "#"

# zonas (terreno fijo)
Z_BOSQUE, Z_CLARO, Z_RIO, Z_ROCA, Z_CUEVA, Z_LLANURA = "b", "c", "r", "k", "q", "l"

ESTACIONES = ["primavera", "verano", "otoño", "invierno"]
# en qué estaciones da cada comida (fruta y baya por estación; nueces de otoño a
# primavera; raíces siempre, pero hay que escarbar). En invierno solo hay comida que cuesta.
DA = {"fruta": (0, 1, 2), "baya": (0, 1), "nuez": (2, 3, 0), "raiz": (0, 1, 2, 3)}
TOPE = {"fruta": 24, "baya": 10, "nuez": 14, "raiz": 14}


class Bosque:
    def __init__(self, semilla=None):
        self.rng = Azar(semilla)
        self.ancho, self.alto = C.BOSQUE_ANCHO, C.BOSQUE_ALTO
        self.tiempo = 0
        self.cambio = None          # (interfaz de Mundo: el bosque no muda fuentes)
        self.lluvia = 0             # ticks de lluvia que quedan
        self.crecida = 0            # ticks de río crecido que quedan
        self.comida = {}            # (x, y) -> [tipo, progreso]
        self.predador = None
        self.harto = 0              # ticks que el predador no caza (después de atacar)
        self.rastro = {}            # (x, y) -> tick en que Angri pasó (lo huele el predador)
        self.atacando = False
        self._generar()

    # --- generación del mapa ---
    def _generar(self):
        r, W, H = self.rng, self.ancho, self.alto
        self.zona = [[Z_BOSQUE] * W for _ in range(H)]
        # llanura al este del río
        self.rio_x = []
        x0 = int(W * 0.6)
        fase = r.random() * 6.28
        for y in range(H):
            rx = x0 + int(3 * math.sin(y / 7 + fase))
            self.rio_x.append(rx)
            for x in range(W):
                if x > rx + 1:
                    self.zona[y][x] = Z_LLANURA
                elif rx - 1 <= x <= rx:
                    self.zona[y][x] = Z_RIO
        # roca al noroeste con cuevas en el borde; un peñasco en la llanura con otra
        self.cuevas = []
        for (cx, cy, rr, n) in ((6, 6, 6, 2), (x0 + 12, H - 12, 3, 1)):
            for y in range(max(0, cy - rr), min(H, cy + rr + 1)):
                for x in range(max(0, cx - rr), min(W, cx + rr + 1)):
                    if (x - cx) ** 2 + (y - cy) ** 2 <= rr * rr and self.zona[y][x] != Z_RIO:
                        self.zona[y][x] = Z_ROCA
            # cuevas: huecos en el borde de la roca, abiertos hacia afuera
            puestos = 0
            for _ in range(200):
                if puestos == n:
                    break
                a = r.random() * 6.28
                x = int(round(cx + (rr - 1) * math.cos(a)))
                y = int(round(cy + (rr - 1) * math.sin(a)))
                if not (0 < x < W - 1 and 0 < y < H - 1):
                    continue
                afuera = [(x + dx, y + dy) for dx, dy in ((0, -1), (0, 1), (-1, 0), (1, 0))
                          if self.zona[y + dy][x + dx] not in (Z_ROCA, Z_CUEVA)]
                if afuera and self.zona[y][x] == Z_ROCA:
                    self.zona[y][x] = Z_CUEVA
                    self.cuevas.append((x, y))
                    puestos += 1
        # claros dentro del bosque
        self.claros = []
        for _ in range(6):
            for _ in range(50):
                cx, cy = r.randrange(4, x0 - 4), r.randrange(4, H - 4)
                if self.zona[cy][cx] == Z_BOSQUE:
                    break
            rr = r.randint(3, 5)
            self.claros.append((cx, cy))
            for y in range(cy - rr, cy + rr + 1):
                for x in range(cx - rr, cx + rr + 1):
                    if (0 <= x < W and 0 <= y < H and (x - cx) ** 2 + (y - cy) ** 2 <= rr * rr
                            and self.zona[y][x] == Z_BOSQUE):
                        self.zona[y][x] = Z_CLARO
        # objetos
        self.grilla = [[VACIO] * W for _ in range(H)]
        for y in range(H):
            for x in range(W):
                z = self.zona[y][x]
                if z == Z_ROCA:
                    self.grilla[y][x] = ROCA
                elif z == Z_CUEVA:
                    self.grilla[y][x] = DESCANSO
                elif z == Z_RIO:
                    self.grilla[y][x] = AGUA
        for cx, cy in self.claros[:C.CHARCOS]:  # charcos: agua aunque el río crezca
            self.grilla[cy][cx] = AGUA
        celdas = lambda z: [(x, y) for y in range(H) for x in range(W)  # noqa: E731
                            if self.zona[y][x] == z and self.grilla[y][x] == VACIO]
        bosque = celdas(Z_BOSQUE)
        self.arboles = {"fruta": r.sample(bosque, 10), "nuez": r.sample(bosque, 8)}
        self.arboles["baya"] = list(self.claros)
        self.arboles["raiz"] = r.sample(celdas(Z_LLANURA), 8)
        # zarzas: espinas en el bosque, más cerca de los frutales
        for _ in range(45):
            fx, fy = r.choice(self.arboles["fruta"]) if r.random() < 0.5 else r.choice(bosque)
            x, y = fx + r.randint(-3, 3), fy + r.randint(-3, 3)
            if 0 <= x < W and 0 <= y < H and self.grilla[y][x] == VACIO and self.zona[y][x] == Z_BOSQUE:
                self.grilla[y][x] = PELIGRO
        self.espinas = sum(f.count(PELIGRO) for f in self.grilla)
        for _ in range(C.NOVEDADES_INICIALES):
            self._novedad()
        for tipo in DA:
            for _ in range(int(TOPE[tipo] * C.ABUNDANCIA) // 2):
                self._brotar(tipo, forzar=True)
        # el predador vive en lo hondo del bosque, lejos del primer claro
        self.cueva_predador = max(bosque, key=lambda p: abs(p[0] - self.claros[0][0]) + abs(p[1] - self.claros[0][1]))
        self.predador = list(self.cueva_predador)
        self.inicio = self.claros[0]
        self._distancias_agua()

    def _novedad(self):
        for _ in range(50):
            x, y = self.rng.randrange(self.ancho), self.rng.randrange(self.alto)
            if self.grilla[y][x] == VACIO:
                self.grilla[y][x] = NOVEDAD
                return

    def _brotar(self, tipo, forzar=False):
        if not forzar and self.estacion() not in DA[tipo]:
            return
        if sum(1 for t, _ in self.comida.values() if t == tipo) >= TOPE[tipo] * C.ABUNDANCIA:
            return
        fx, fy = self.rng.choice(self.arboles[tipo])
        rr = 3 if tipo == "baya" else 2
        x, y = fx + self.rng.randint(-rr, rr), fy + self.rng.randint(-rr, rr)
        if 0 <= x < self.ancho and 0 <= y < self.alto and self.grilla[y][x] == VACIO:
            self.grilla[y][x] = COMIDA
            self.comida[(x, y)] = [tipo, 0]

    def _distancias_agua(self):
        """Para la humedad: celda -> agua más cercana (BFS desde todo el agua)."""
        W, H = self.ancho, self.alto
        cerca = {}
        frente = []
        for y in range(H):
            for x in range(W):
                if self.grilla[y][x] == AGUA:
                    cerca[(x, y)] = (x, y)
                    frente.append((x, y))
        while frente:
            nuevo = []
            for (x, y) in frente:
                for dx, dy in ((0, -1), (0, 1), (-1, 0), (1, 0)):
                    p = (x + dx, y + dy)
                    if 0 <= p[0] < W and 0 <= p[1] < H and p not in cerca:
                        cerca[p] = cerca[(x, y)]
                        nuevo.append(p)
            frente = nuevo
        self.agua_cerca = cerca

    # --- reloj ---
    def hora(self):
        return (self.tiempo % C.DIA) / C.DIA          # 0 = amanecer

    def noche(self):
        return self.hora() >= 1 - C.FRACCION_NOCHE

    def estacion(self):
        return (self.tiempo // C.ESTACION_BOSQUE) % 4

    def temperatura(self, x, y):
        """Temperatura del aire donde está (0 frío .. 1 calor). La cueva la amortigua."""
        if self.zona[y][x] == Z_CUEVA:
            return C.TEMP_CUEVA
        t = (C.TEMP_DIA, C.TEMP_DIA + 0.1, C.TEMP_DIA - 0.05, C.TEMP_DIA - 0.2)[self.estacion()]
        if self.noche():
            t -= C.FRIO_NOCHE
        if self.lluvia and self.zona[y][x] != Z_BOSQUE:
            t -= C.FRIO_LLUVIA        # bajo los árboles se moja menos
        elif self.lluvia:
            t -= C.FRIO_LLUVIA / 2
        return max(0.0, min(1.0, t))

    # --- interfaz de Mundo ---
    def en(self, x, y):
        return self.grilla[y][x]

    def transitable(self, x, y):
        return 0 <= x < self.ancho and 0 <= y < self.alto and self.grilla[y][x] != ROCA

    def consumir(self, x, y):
        if self.grilla[y][x] in (COMIDA, NOVEDAD):
            self.grilla[y][x] = VACIO
        self.comida.pop((x, y), None)

    def vecindad(self, x, y, radio=None):
        """Lo que ve: 2 casillas de día, 1 de noche (siempre al menos las vecinas).
        El predador se ve encima de lo que haya."""
        if radio is None:
            radio = C.VISTA_NOCHE if self.noche() else C.VISTA_DIA
        vista = {}
        W, H, g = self.ancho, self.alto, self.grilla
        for dy in range(-radio, radio + 1):
            ny = y + dy
            if 0 <= ny < H:
                fila = g[ny]
                for dx in range(-radio, radio + 1):
                    nx = x + dx
                    if 0 <= nx < W:
                        vista[(dx, dy)] = fila[nx]
        px, py = self.predador
        if abs(px - x) <= radio and abs(py - y) <= radio and self.harto == 0:
            vista[(px - x, py - y)] = PELIGRO
        return vista

    def sentir(self, x, y, vista):
        """Vista + lo más fuerte de cada tipo que llega por otros sentidos, en su
        lugar real (dirección y distancia). No pisa lo que se ve."""
        s = dict(vista)
        # olfato: la comida más cercana dentro de 6
        mejor, d0 = None, C.OLFATO + 1
        for (cx, cy) in self.comida:
            d = abs(cx - x) + abs(cy - y)
            if d < d0:
                mejor, d0 = (cx - x, cy - y), d
        if mejor is not None and mejor not in s:
            s[mejor] = COMIDA
        # humedad: el agua más cercana dentro de 8
        ax, ay = self.agua_cerca.get((x, y), (x + 99, y))
        if abs(ax - x) + abs(ay - y) <= C.HUMEDAD and (ax - x, ay - y) not in s:
            s[(ax - x, ay - y)] = AGUA
        # el predador: se huele cerca, y de noche se lo oye moverse más lejos
        px, py = self.predador
        d = abs(px - x) + abs(py - y)
        if self.harto == 0 and (d <= C.OLOR_PREDADOR or (self.noche() and d <= C.OIDO_PREDADOR)):
            s.setdefault((px - x, py - y), PELIGRO)
        return s

    # --- el mundo cambia solo ---
    def avanzar(self, angri_pos=None, refugiado=False):
        self.tiempo += 1
        r = self.rng
        # clima
        if self.lluvia:
            self.lluvia -= 1
            if self.lluvia > 0 and not self.crecida and r.random() < C.PROB_CRECIDA:
                self._crecer(True)
        elif r.random() < C.PROB_LLUVIA:
            self.lluvia = r.randint(200, 700)
        if self.crecida:
            self.crecida -= 1
            if self.crecida == 0:
                self._crecer(False)
        # comida que vuelve a crecer (cada una en su estación) y novedades
        if r.random() < C.PROB_BROTE:
            self._brotar(r.choice(("fruta", "fruta", "baya", "nuez", "raiz")))
        if r.random() < C.PROB_NUEVA_NOVEDAD and sum(f.count(NOVEDAD) for f in self.grilla) < C.NOVEDAD_MAXIMA:
            self._novedad()
        # fuera de estación la fruta y la baya se pudren de a poco
        if self.tiempo % 100 == 0:
            for p, (tipo, _) in list(self.comida.items()):
                if self.estacion() not in DA[tipo] and r.random() < 0.1:
                    self.consumir(*p)
        # rastro de Angri (lo que sigue el predador)
        if angri_pos is not None:
            self.rastro[tuple(angri_pos)] = self.tiempo
            if self.tiempo % 200 == 0:
                for p in [p for p, t in self.rastro.items() if self.tiempo - t > C.RASTRO_DURA]:
                    del self.rastro[p]
        self._mover_predador(angri_pos, refugiado)

    def _crecer(self, si):
        """El río crece: el agua del río y sus orillas se vuelven peligro. Los
        charcos siguen siendo agua."""
        self.crecida = self.rng.randint(300, 900) if si else 0
        for y in range(self.alto):
            rx = self.rio_x[y]
            for x in range(max(0, rx - 2), min(self.ancho, rx + 3)):
                if self.zona[y][x] == Z_RIO:
                    self.grilla[y][x] = PELIGRO if si else AGUA
                elif self.grilla[y][x] in (VACIO, PELIGRO) and x in (rx - 2, rx + 1, rx + 2):
                    self.grilla[y][x] = PELIGRO if si else VACIO
        self._distancias_agua()

    def _puede_predador(self, x, y):
        return (0 <= x < self.ancho and 0 <= y < self.alto
                and self.zona[y][x] in (Z_BOSQUE, Z_CLARO, Z_LLANURA)
                and self.grilla[y][x] not in (AGUA, ROCA))

    def _olor(self, x, y):
        """Olor de Angri en (x, y): su rastro decae con el tiempo y se difunde a
        las vecinas (a la mitad)."""
        mejor = 0.0
        for dx, dy, f in ((0, 0, 1.0), (0, -1, 0.5), (0, 1, 0.5), (-1, 0, 0.5), (1, 0, 0.5)):
            t = self.rastro.get((x + dx, y + dy))
            if t is not None:
                v = f * C.RASTRO_DECAE ** (self.tiempo - t)
                if v > mejor:
                    mejor = v
        return mejor

    def _mover_predador(self, angri_pos, refugiado):
        r = self.rng
        self.atacando = False
        px, py = self.predador
        if self.harto:
            # harto: vuelve a su guarida y no caza (el sufrimiento no se encadena)
            self.harto -= 1
            if r.random() < 0.3:
                gx, gy = self.cueva_predador
                paso = (0 if gx == px else (1 if gx > px else -1), 0) if r.random() < 0.5 else \
                       (0, 0 if gy == py else (1 if gy > py else -1))
                if self._puede_predador(px + paso[0], py + paso[1]):
                    self.predador = [px + paso[0], py + paso[1]]
            return
        if r.random() >= (C.PREDADOR_NOCHE if self.noche() else C.PREDADOR_DIA):
            return
        pasos = [(0, -1), (0, 1), (-1, 0), (1, 0)]
        opciones = [(px + dx, py + dy) for dx, dy in pasos if self._puede_predador(px + dx, py + dy)]
        if not opciones:
            return
        ax, ay = angri_pos if angri_pos is not None else (-99, -99)
        cerca = abs(ax - px) + abs(ay - py)
        if cerca <= 2 and not refugiado:
            elegido = min(opciones, key=lambda p: abs(p[0] - ax) + abs(p[1] - ay))
        else:
            olores = [self._olor(*p) for p in opciones]
            m = max(olores)
            if m > 0.05 and r.random() < (0.9 if self.noche() else 0.5):
                elegido = opciones[olores.index(m)]
            else:
                elegido = r.choice(opciones)
        self.predador = list(elegido)
        if tuple(elegido) == (ax, ay) and not refugiado:
            self.atacando = True
            self.harto = C.PREDADOR_HARTO

    def foto(self):
        """Estado chico para el visor (no es la mente: solo lo que se ve)."""
        return {"ancho": self.ancho, "alto": self.alto, "tiempo": self.tiempo,
                "hora": round(self.hora(), 3), "noche": self.noche(),
                "estacion": ESTACIONES[self.estacion()], "lluvia": bool(self.lluvia),
                "crecida": bool(self.crecida), "predador": self.predador,
                "harto": self.harto > 0,
                "grilla": ["".join(f) for f in self.grilla],
                "comida": [[x, y, t] for (x, y), (t, _) in self.comida.items()]}

    def foto_terreno(self):
        return ["".join(f) for f in self.zona]


def _trabajo(tipo):
    return {"nuez": C.NUEZ_TICKS, "raiz": C.RAIZ_TICKS}.get(tipo, 1)


def trabajar(mundo, x, y):
    """Un tick sobre una comida: la fruta y la baya se comen al llegar; la nuez
    y la raíz necesitan quedarse varios ticks (abrir, escarbar). Devuelve el
    tipo cuando se la come, o None si todavía no."""
    c = mundo.comida.get((x, y))
    if c is None:
        return None
    c[1] += 1
    if c[1] >= _trabajo(c[0]):
        mundo.consumir(x, y)
        return c[0]
    return None
