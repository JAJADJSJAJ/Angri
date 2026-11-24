"""El bucle de vida: Angri nunca se apaga.

Cada tick: el mundo avanza, Angri percibe, su cuerpo y su química cambian,
y él actúa. Se duerme solo cuando la adenosina pasa el umbral, y se despierta
solo cuando se limpió. Nadie le dice cuándo.
"""
import csv
import json
import os
import random
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
from .quimica import Quimica
from .panel import dibujar

CARPETA_DATOS = os.path.join(os.path.dirname(os.path.dirname(__file__)), "datos")
ARCHIVO_ESTADO = os.path.join(CARPETA_DATOS, "estado.json")
ARCHIVO_DIARIO = os.path.join(CARPETA_DATOS, "diario.csv")


class Angri:
    def __init__(self, semilla=None, cargar=True):
        guardado = self._cargar() if cargar else {}
        self.cuerpo = Cuerpo(guardado.get("cuerpo"))
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
        self.mundo = Mundo(None if semilla is None else semilla + 1)
        self.x, self.y = guardado.get("pos", (8, 7))
        self.despierto = guardado.get("despierto", True)
        self.edad = guardado.get("edad", 0)  # ticks vividos
        self.suenos = guardado.get("suenos", 0)
        self._malestar = self.cuerpo.malestar()
        self.evento = "nace" if self.edad == 0 else "despierta del apagado"
        self.oido = Oido()
        self.oido.restaurar(guardado.get("oido"))
        if self.maestro:
            self.maestro.restaurar(guardado.get("maestro"))
        self.habla = Habla(random.Random(None if semilla is None else semilla + 11), self.maestro) if self.maestro and C.PIDE else None
        if self.habla:
            self.habla.restaurar(guardado.get("habla"))

    # --- un instante de vida ---
    def tick(self):
        self.mundo.avanzar()
        se_movio = False
        sorpresa = 0.0

        if self.despierto:
            vista = self.mundo.vecindad(self.x, self.y)
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
                dx, dy = self.mente.elegir(vista, self.cuerpo.v, necesidad, (self.x, self.y),
                                           self.quimica.noradrenalina, self._malestar,
                                           self.oido.palabras() if C.FRASE else (), dicha)
            else:
                dx, dy = self.mente.elegir(vista, necesidad)
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
            self._interactuar(aqui)
            if self.habla:
                baja = {k: antes[k] - abs(self.cuerpo.v[k] - C.EQUILIBRIO[k]) for k in antes}
                n = max(baja, key=baja.get)
                if baja[n] > C.UMBRAL_ALIVIO and n in self.habla.assoc:
                    self.habla.aliviado(n, self.edad)

        self.cuerpo.vivir(self.despierto, se_movio)
        malestar_antes = self._malestar
        malestar = self._malestar = self.cuerpo.malestar()
        if self.despierto and isinstance(self.mente, Ganglios):
            recompensa = (C.RECOMPENSA_ALIVIO * (malestar_antes - malestar)
                          - C.RECOMPENSA_ESTADO * malestar)
            # nocicepción: el dolor llega a la dopamina directo, aunque el cuerpo ya
            # esté en el piso (integridad 0) o el malestar en su tope y no cambie nada
            if encontrado == PELIGRO:
                recompensa -= C.DOLOR
            if self.corteza:
                sorpresa = self.corteza.comparar(encontrado, malestar - malestar_antes)
                recompensa += C.PESO_CURIOSIDAD * self.corteza.progreso
            vista = self.mundo.vecindad(self.x, self.y)
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

    def _regular_sueno(self):
        """Proceso S: duerme por presión interna, no por horario."""
        q = self.quimica
        if self.despierto and q.adenosina >= C.UMBRAL_DORMIR:
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
        elif not self.despierto and q.adenosina <= C.UMBRAL_DESPERTAR:
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
