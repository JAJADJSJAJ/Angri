"""El bucle de vida: Angri nunca se apaga.

Cada tick: el mundo avanza, Angri percibe, su cuerpo y su química cambian,
y él actúa. Se duerme solo cuando la adenosina pasa el umbral, y se despierta
solo cuando se limpió. Nadie le dice cuándo.
"""
import csv
import json
import os
import time

from . import config as C
from .corteza import Corteza
from .cuerpo import Cuerpo
from .ganglios import Ganglios
from .hipocampo import Hipocampo
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
        else:
            self.hipocampo = None
            self.mente = Mente(semilla)
            self.corteza = None
        self.mundo = Mundo(None if semilla is None else semilla + 1)
        self.x, self.y = guardado.get("pos", (8, 7))
        self.despierto = guardado.get("despierto", True)
        self.edad = guardado.get("edad", 0)  # ticks vividos
        self.suenos = guardado.get("suenos", 0)
        self._malestar = self.cuerpo.malestar()
        self.evento = "nace" if self.edad == 0 else "despierta del apagado"

    # --- un instante de vida ---
    def tick(self):
        self.mundo.avanzar()
        se_movio = False
        sorpresa = 0.0

        if self.despierto:
            vista = self.mundo.vecindad(self.x, self.y)
            necesidad = self.cuerpo.necesidad_urgente()
            if isinstance(self.mente, Ganglios):
                dx, dy = self.mente.elegir(vista, self.cuerpo.v, necesidad, (self.x, self.y),
                                           self.quimica.noradrenalina, self._malestar)
            else:
                dx, dy = self.mente.elegir(vista, necesidad)
            nx = max(0, min(C.ANCHO - 1, self.x + dx))
            ny = max(0, min(C.ALTO - 1, self.y + dy))
            if self.corteza:
                self.corteza.predecir(vista.get((dx, dy), "#"), necesidad,
                                      (dx, dy) == (0, 0), (self.x, self.y))
            se_movio = (nx, ny) != (self.x, self.y)
            self.x, self.y = nx, ny
            aqui = self.mundo.en(self.x, self.y)
            sorpresa = self.mente.sorpresa(aqui)
            encontrado = aqui
            self._interactuar(aqui)

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
            self.mente.aprender(recompensa, vista, self.cuerpo.v,
                                self.cuerpo.necesidad_urgente(), (self.x, self.y))
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
