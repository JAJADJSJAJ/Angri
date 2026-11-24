"""Foto del estado para el visor (etapa 13a): lo que se ve, no la mente entera.

vivir.py la escribe cada tanto (por tiempo de reloj, no por ticks) a
DIR/foto.json, de forma atómica; el visor solo la lee. Escribirla cuesta unos
milisegundos, así que no frena la vida.
"""
import json
import os

from .bosque import Bosque


def foto(angri):
    m = angri.mundo
    if isinstance(m, Bosque):
        mundo = m.foto()
        terreno = m.foto_terreno()
    else:
        mundo = {"ancho": len(m.grilla[0]), "alto": len(m.grilla), "tiempo": m.tiempo,
                 "hora": 0.25, "noche": False, "estacion": "-", "lluvia": False,
                 "crecida": False, "predador": None, "harto": False,
                 "grilla": ["".join(f) for f in m.grilla], "comida": []}
        terreno = ["l" * mundo["ancho"]] * mundo["alto"]
    c = angri.cuerpo
    mente = angri.mente
    return {
        "edad": angri.edad, "suenos": angri.suenos, "despierto": angri.despierto,
        "evento": angri.evento, "pos": [angri.x, angri.y],
        "malestar": round(c.malestar(), 4), "foco": c.necesidad_urgente(),
        "cuerpo": {k: round(v, 3) for k, v in c.v.items()},
        "quimica": {k: round(v, 3) for k, v in angri.quimica.estado().items()},
        "dopamina_td": round(getattr(mente, "dopamina", 0.0), 3),
        "lugares": {n: len(l) for n, l in getattr(mente, "lugares", {}).items()},
        "mundo": mundo, "terreno": terreno,
    }


def escribir(angri, archivo):
    tmp = archivo + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(foto(angri), f, ensure_ascii=False, separators=(",", ":"))
    os.replace(tmp, archivo)
