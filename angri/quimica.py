"""Neuromoduladores: señales globales que cambian cómo funciona todo Angri.

En la etapa 1 solo se calculan y se registran. En la etapa 2 van a controlar
cuánto aprende (dopamina), cuánto explora (noradrenalina), etc.
"""
from . import config as C


def _clip(x):
    return max(0.0, min(1.0, x))


class Quimica:
    def __init__(self, estado=None):
        self.adenosina = 0.0     # presión de sueño
        self.cortisol = 0.0      # estrés lento
        self.dopamina = 0.0      # error de "recompensa": ¿mejoró el equilibrio?
        self.noradrenalina = 0.0  # sorpresa / alerta
        if estado:
            self.__dict__.update(estado)
        self._malestar_previo = None

    def actualizar(self, despierto, malestar, estres, sorpresa):
        # Adenosina: se acumula despierto, se limpia durmiendo (proceso S de Borbély)
        if despierto:
            self.adenosina += C.ADENOSINA_POR_ACTIVIDAD
        else:
            self.adenosina -= C.ADENOSINA_LIMPIEZA_SUENO
        # Cortisol: sigue al estrés, pero lento
        if estres > 0.4:
            self.cortisol += C.CORTISOL_SUBE * (estres - 0.4)
        self.cortisol -= C.CORTISOL_BAJA
        # Dopamina: recompensa homeostática = el malestar bajó (Keramati & Gutkin)
        if self._malestar_previo is None:
            self.dopamina = 0.0
        else:
            self.dopamina = max(-1.0, min(1.0, (self._malestar_previo - malestar) * 50))
        self._malestar_previo = malestar
        # Noradrenalina: salta con la sorpresa y decae
        self.noradrenalina = 0.9 * self.noradrenalina + 0.1 * sorpresa

        self.adenosina = _clip(self.adenosina)
        self.cortisol = _clip(self.cortisol)
        self.noradrenalina = _clip(self.noradrenalina)

    def estado(self):
        return {
            "adenosina": self.adenosina,
            "cortisol": self.cortisol,
            "dopamina": self.dopamina,
            "noradrenalina": self.noradrenalina,
        }
