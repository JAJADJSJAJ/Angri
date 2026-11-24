"""La mente de Angri.

ETAPA 1: todavía no aprende. Se mueve al azar, con un sesgo mínimo
tipo "reflejo" (como un bebé: se acerca a lo que alivia su necesidad urgente
si lo tiene justo al lado). Esto es lo que se reemplaza en la etapa 2 por un
actor-crítico guiado por dopamina.
"""
from angri.azar import Azar

from .config import REFLEJOS
from .mundo import COMIDA, DESCANSO, NOVEDAD, PELIGRO

MOVIMIENTOS = [(0, -1), (0, 1), (-1, 0), (1, 0), (0, 0)]

# qué alivia cada necesidad
ALIVIA = {
    "energia": COMIDA,
    "cansancio": DESCANSO,
    "curiosidad": NOVEDAD,
}


class Mente:
    def __init__(self, semilla=None):
        self.rng = Azar(semilla)
        self.prediccion = None  # qué espera encontrar en el próximo paso

    def elegir(self, vista, necesidad):
        if not REFLEJOS:
            return self.rng.choice(MOVIMIENTOS)
        objetivo = ALIVIA.get(necesidad)
        # reflejo: si lo que necesita está pegado, ir ahí
        if objetivo:
            for mov in MOVIMIENTOS[:4]:
                if vista.get(mov) == objetivo:
                    return mov
        # reflejo de evitación del peligro
        seguros = [m for m in MOVIMIENTOS if vista.get(m) not in (PELIGRO, None)]
        mov = self.rng.choice(seguros or MOVIMIENTOS)
        return mov

    def sorpresa(self, encontrado):
        """Error de predicción muy simple: ¿encontró algo distinto de lo esperado?"""
        esperado = self.prediccion
        self.prediccion = encontrado  # predice que el mundo sigue igual
        if esperado is None:
            return 0.0
        return 0.0 if encontrado == esperado else 1.0
