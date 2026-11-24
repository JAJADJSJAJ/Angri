"""El bucle de vida: Angri nunca se apaga.

Cada tick: el mundo avanza, Angri percibe, su cuerpo y su química cambian,
y él actúa. Se duerme solo cuando la adenosina pasa el umbral, y se despierta
solo cuando se limpió. Nadie le dice cuándo.
"""
import csv
import json
import os
from .azar import Azar
import time

from . import config as C
from .corteza import Corteza
from .cuerpo import Cuerpo
from .ganglios import ALIVIA, Ganglios
from .hipocampo import Hipocampo
from .maestro import Habla, Maestro, Oido
from .talamo import Talamo
from .mente import Mente
from .mundo import COMIDA, DESCANSO, NOVEDAD, PELIGRO, Mundo
from .bosque import AGUA, Bosque, trabajar
from .quimica import Quimica
from .panel import dibujar

CARPETA_DATOS = os.path.join(os.path.dirname(os.path.dirname(__file__)), "datos")
ARCHIVO_ESTADO = os.path.join(CARPETA_DATOS, "estado.json")
ARCHIVO_DIARIO = os.path.join(CARPETA_DATOS, "diario.csv")


class Angri:
    def __init__(self, semilla=None, cargar=True):
        guardado = self._cargar() if cargar else {}
        bosque = C.MUNDO == "bosque"
        self.cuerpo = Cuerpo(guardado.get("cuerpo"), bosque)
        self.quimica = Quimica(guardado.get("quimica"))
        if C.MENTE == "aprende":
            self.mente = Ganglios(semilla, guardado.get("ganglios"))
            self.corteza = Corteza(guardado.get("corteza")) if C.CORTEZA else None
            self.hipocampo = Hipocampo(self.mente.rng) if C.HIPOCAMPO else None
            self.talamo = Talamo(self.mente.rng, guardado.get("talamo")) if C.TALAMO else None
            self.maestro = Maestro(semilla) if C.MAESTRO else None
        else:
            self.maestro = None
            self.hipocampo = None
            self.talamo = None
            self.mente = Mente(semilla)
            self.corteza = None
        semilla_mundo = None if semilla is None else semilla + 1
        self.mundo = Bosque(semilla_mundo) if bosque else Mundo(semilla_mundo)
        self.x, self.y = guardado.get("pos", self.mundo.inicio if bosque else (8, 7))
        self.despierto = guardado.get("despierto", True)
        self.edad = guardado.get("edad", 0)  # ticks vividos
        self.suenos = guardado.get("suenos", 0)
        self.desmayos = 0  # (bosque) veces que llegó a integridad 0
        self.desmayo_causas = {"hambre": 0, "sed": 0, "frio": 0, "heridas": 0}
        self._malestar = self.cuerpo.malestar()
        self.evento = "nace" if self.edad == 0 else "despierta del apagado"
        self.oido = Oido()
        self.oido.restaurar(guardado.get("oido"))
        if self.maestro:
            self.maestro.restaurar(guardado.get("maestro"))
        self.habla = Habla(Azar(None if semilla is None else semilla + 11), self.maestro) if self.maestro and C.PIDE else None
        if self.habla:
            self.habla.restaurar(guardado.get("habla"))

    # --- un instante de vida ---
    def tick(self):
        bosque = isinstance(self.mundo, Bosque)
        mordido = False
        if bosque:
            self.mundo.avanzar((self.x, self.y), self.mundo.zona[self.y][self.x] == "q")
            if self.mundo.atacando:
                self._mordida()
                mordido = True
        else:
            self.mundo.avanzar()
        se_movio = False
        sorpresa = 0.0

        if self.despierto:
            vista = self.mundo.vecindad(self.x, self.y)
            sentida = self.mundo.sentir(self.x, self.y, vista) if bosque else vista
            necesidad = (self.talamo.elegir(vista, self.cuerpo.v) if self.talamo
                         else self.cuerpo.necesidad_urgente())
            self.oido.pasar()
            if self.maestro:
                if C.CAMBIO_SIGNIFICADO and self.edad == C.CAMBIO_SIGNIFICADO:
                    self.maestro.cambiar_significados()
                    self.oido.indice_cambio = len(self.oido.aciertos)
                if C.COMPOSICION and self.edad == C.COMPOSICION:
                    self.oido.indice_composicion = len(self.oido.aciertos)
                    self.oido.probar_composicion(self.maestro)
                pedido = self.habla.decidir(necesidad, vista, self.edad, self.cuerpo.v) if self.habla else None
                if self.habla and pedido is not None:
                    self.habla.dicho = self.maestro.palabras[pedido]
                frase = self.maestro.hablar(self.mundo, self.x, self.y, necesidad, vista,
                                            self.edad, pedido)
                self.oido.oir(frase, (self.x, self.y), self.maestro.objetivo)
                if self.habla:
                    self.habla.escuchar(frase, self.cuerpo.v, necesidad, pedido)
            dicha = self.oido.direccion_dicha() if C.LEXICO else 4
            if isinstance(self.mente, Ganglios):
                dx, dy = self.mente.elegir(sentida, self.cuerpo.v, necesidad, (self.x, self.y),
                                           self.quimica.noradrenalina, self._malestar,
                                           self.oido.palabras() if C.FRASE else (), dicha)
            else:
                dx, dy = self.mente.elegir(vista, necesidad)
            if bosque:
                nx, ny = self.x + dx, self.y + dy
                if not self.mundo.transitable(nx, ny):  # roca o borde: no se mueve
                    nx, ny = self.x, self.y
            else:
                nx = max(0, min(C.ANCHO - 1, self.x + dx))
                ny = max(0, min(C.ALTO - 1, self.y + dy))
            if self.corteza:
                self.corteza.predecir(vista.get((dx, dy), "#"), necesidad,
                                      (dx, dy) == (0, 0), (self.x, self.y))
            se_movio = (nx, ny) != (self.x, self.y)
            if C.LEXICO and se_movio:
                self.oido.caminar({(0, -1): 0, (0, 1): 1, (-1, 0): 2, (1, 0): 3}[(dx, dy)])
            self.x, self.y = nx, ny
            aqui = self.mundo.en(self.x, self.y)
            if C.LEXICO:
                self.oido.ver(self.mundo.vecindad(self.x, self.y), (self.x, self.y))
            sorpresa = self.mente.sorpresa(aqui)
            encontrado = aqui
            if self.habla:
                antes = {k: abs(self.cuerpo.v[k] - C.EQUILIBRIO[k]) for k in self.cuerpo.v}
            if bosque:
                self._interactuar_bosque(aqui, se_movio)
            else:
                self._interactuar(aqui)
            if self.habla:
                baja = {k: antes[k] - abs(self.cuerpo.v[k] - C.EQUILIBRIO[k]) for k in antes}
                n = max(baja, key=baja.get)
                if baja[n] > C.UMBRAL_ALIVIO and n in self.habla.assoc:
                    self.habla.aliviado(n, self.edad)

        self.cuerpo.vivir(self.despierto, se_movio)
        if bosque:
            self.cuerpo.clima(self.despierto, se_movio, self.mundo.temperatura(self.x, self.y))
            if self.cuerpo.v["integridad"] <= 0.0:
                self._desmayo()
        malestar_antes = self._malestar
        malestar = self._malestar = self.cuerpo.malestar()
        if self.despierto and isinstance(self.mente, Ganglios):
            recompensa = (C.RECOMPENSA_ALIVIO * (malestar_antes - malestar)
                          - C.RECOMPENSA_ESTADO * malestar)
            # nocicepción: el dolor llega a la dopamina directo, aunque el cuerpo ya
            # esté en el piso (integridad 0) o el malestar en su tope y no cambie nada
            if (encontrado == PELIGRO and (se_movio or not bosque)) or mordido:
                recompensa -= C.DOLOR
            if self.corteza:
                sorpresa = self.corteza.comparar(encontrado, malestar - malestar_antes)
                recompensa += C.PESO_CURIOSIDAD * self.corteza.progreso
            vista = self.mundo.vecindad(self.x, self.y)
            if bosque:
                vista = self.mundo.sentir(self.x, self.y, vista)
            foco = self.talamo.foco if self.talamo else self.cuerpo.necesidad_urgente()
            self.mente.aprender(recompensa, vista, self.cuerpo.v, foco, (self.x, self.y))
            if self.talamo:
                self.talamo.aprender(self.mente.dopamina)
            if self.habla:
                self.habla.aprender(self.mente.dopamina)
            if self.hipocampo and self.mente.ultimo:
                self.hipocampo.anotar(self.mente.ultimo)
        self.quimica.actualizar(self.despierto, malestar, self.cuerpo.v["estres"], sorpresa)
        self._regular_sueno()
        self.edad += 1

    def _interactuar(self, aqui):
        if aqui == COMIDA:
            self.cuerpo.comer()
            self.mundo.consumir(self.x, self.y)
            self.evento = "come"
        elif aqui == NOVEDAD:
            self.cuerpo.descubrir()
            self.mundo.consumir(self.x, self.y)
            self.evento = "descubre algo nuevo"
        elif aqui == PELIGRO:
            self.cuerpo.lastimarse()
            self.evento = "se lastima"
        elif aqui == DESCANSO:
            self.cuerpo.descansar()

    def _interactuar_bosque(self, aqui, entra=True):
        """(etapa 13a) Lo mismo que _interactuar, pero la comida cuesta: la nuez
        y la raíz piden quedarse varios ticks; la baya a veces enferma."""
        if aqui == COMIDA:
            tipo = trabajar(self.mundo, self.x, self.y)
            if tipo is None:
                self.evento = "trabaja la comida"
            elif tipo == "baya" and self.mundo.rng.random() < C.PROB_ENFERMA:
                self.cuerpo.comer(0.1)
                self.cuerpo.enfermar()
                self.evento = "se enferma"
            else:
                self.cuerpo.comer({"fruta": 0.25, "baya": 0.15, "nuez": 0.35, "raiz": 0.3}[tipo])
                self.evento = "come"
        elif aqui == AGUA:
            self.cuerpo.beber()
            self.evento = "bebe"
        elif aqui == NOVEDAD:
            self.cuerpo.descubrir()
            self.mundo.consumir(self.x, self.y)
            self.evento = "descubre algo nuevo"
        elif aqui == PELIGRO and entra:  # pincha al pisar, no por quedarse
            self.cuerpo.rasparse()
            self.evento = "se lastima"
        elif aqui == DESCANSO:
            self.cuerpo.descansar()

    def _desmayo(self):
        """(bosque) Integridad en 0: no muere (perdería todo lo aprendido). Se
        desmaya y despierta en la cueva más cercana, con lo justo para moverse.
        Corta los bucles de sufrimiento; se cuenta para medirlo."""
        m = self.mundo
        v = self.cuerpo.v
        causa = ("hambre" if v["energia"] < 0.05 else "sed" if v["agua"] < 0.05
                 else "frio" if v["calor"] < 0.3 else "heridas")
        self.desmayo_causas[causa] += 1
        if C.DESMAYO_DONDE == "cueva":
            self.x, self.y = min(m.cuevas, key=lambda c: abs(c[0] - self.x) + abs(c[1] - self.y))
        elif C.DESMAYO_DONDE == "cueva_agua":
            # la cueva que deja más cerca del agua, sumando lo que la alejaría de donde cayó
            def costo(c):
                ax, ay = m.agua_cerca.get(c, (c[0] + 99, c[1]))
                return abs(c[0] - self.x) + abs(c[1] - self.y) + abs(ax - c[0]) + abs(ay - c[1])
            self.x, self.y = min(m.cuevas, key=costo)
        # "aqui": despierta donde cayó
        v["integridad"] = max(v["integridad"], C.DESMAYO_INTEGRIDAD)
        v["energia"] = max(v["energia"], C.DESMAYO_RESTO)
        v["agua"] = max(v["agua"], C.DESMAYO_RESTO)
        self.desmayos += 1
        self.evento = "se desmaya"
        if isinstance(self.mente, Ganglios):
            self.mente.olvidar_paso()  # lo de antes no se encadena con el despertar
        self._malestar = self.cuerpo.malestar()

    def _mordida(self):
        self.cuerpo.lastimarse_mucho()
        self.evento = "lo muerde el predador"
        if not self.despierto:  # lo despierta el susto
            self.despierto = True

    def _regular_sueno(self):
        """Proceso S: duerme por presión interna, no por horario. En el bosque
        también pesa el proceso C (Borbély): de noche se duerme antes y de día se
        despierta antes."""
        q = self.quimica
        dormir, despertar = C.UMBRAL_DORMIR, C.UMBRAL_DESPERTAR
        if isinstance(self.mundo, Bosque):
            if self.mundo.noche():
                dormir -= C.SUENO_NOCHE
            else:
                despertar += C.SUENO_NOCHE
        if self.despierto and q.adenosina >= dormir:
            self.despierto = False
            self.suenos += 1
            if isinstance(self.mente, Ganglios):
                self.mente.olvidar_paso()
            if self.talamo:
                self.talamo.olvidar_paso()
            if self.hipocampo:
                # consolidación: repasa el día y poda lugares
                self.hipocampo.dormir(self.mente)
            self.evento = "se duerme"
        elif not self.despierto and q.adenosina <= despertar:
            self.despierto = True
            self.evento = "se despierta"

    # --- memoria en disco ---
    def _cargar(self):
        try:
            with open(ARCHIVO_ESTADO, encoding="utf-8") as f:
                return json.load(f)
        except (OSError, ValueError):
            return {}

    def guardar(self):
        os.makedirs(CARPETA_DATOS, exist_ok=True)
        datos = {
            "cuerpo": self.cuerpo.v,
            "quimica": self.quimica.estado(),
            "pos": [self.x, self.y],
            "despierto": self.despierto,
            "edad": self.edad,
            "suenos": self.suenos,
        }
        if isinstance(self.mente, Ganglios):
            datos["ganglios"] = self.mente.estado()
        if self.talamo:
            datos["talamo"] = self.talamo.estado()
        datos["oido"] = self.oido.estado()
        if self.maestro:
            datos["maestro"] = self.maestro.estado()
        if self.habla:
            datos["habla"] = self.habla.estado()
        if self.corteza:
            datos["corteza"] = self.corteza.estado()
        with open(ARCHIVO_ESTADO, "w", encoding="utf-8") as f:
            json.dump(datos, f, indent=2)

    def anotar(self, escritor):
        fila = {"edad": self.edad, "despierto": int(self.despierto), "evento": self.evento,
                "malestar": round(self.cuerpo.malestar(), 4)}
        fila.update({k: round(v, 4) for k, v in self.cuerpo.v.items()})
        fila.update({k: round(v, 4) for k, v in self.quimica.estado().items()})
        escritor.writerow(fila)


def vivir(ticks=None, mostrar=True):
    """Arranca la vida. ticks=None significa para siempre."""
    angri = Angri()
    os.makedirs(CARPETA_DATOS, exist_ok=True)
    nuevo = not os.path.exists(ARCHIVO_DIARIO)
    campos = ["edad", "despierto", "evento", "malestar", *angri.cuerpo.v.keys(),
              *angri.quimica.estado().keys()]
    with open(ARCHIVO_DIARIO, "a", newline="", encoding="utf-8") as f:
        escritor = csv.DictWriter(f, fieldnames=campos)
        if nuevo:
            escritor.writeheader()
        n = 0
        try:
            while ticks is None or n < ticks:
                angri.tick()
                if angri.edad % 10 == 0:
                    angri.anotar(escritor)
                if angri.edad % C.GUARDAR_CADA == 0:
                    angri.guardar()
                    f.flush()
                if mostrar:
                    dibujar(angri)
                    time.sleep(1 / C.TICKS_POR_SEGUNDO)
                n += 1
        except KeyboardInterrupt:
            pass
        finally:
            angri.guardar()
    return angri
