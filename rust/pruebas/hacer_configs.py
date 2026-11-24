"""Arma las configs de las pruebas de Rust (rust/pruebas/config-*.txt) desde
angri/config.py. Solo hace falta correrlo si cambia la config de Python y se
quiere que las pruebas usen la nueva (después regenerar la referencia).

    python rust/pruebas/hacer_configs.py
"""
import importlib
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
from angri import config as C  # noqa: E402
import medir  # noqa: E402

CONFIGS = {
    "grilla": {},
    "bosque": {"MUNDO": "bosque"},
    "corteza-talamo": {"CORTEZA": True, "TALAMO": True},
    "corteza-talamo-bosque": {"MUNDO": "bosque", "CORTEZA": True, "TALAMO": True},
    "pide": {"PIDE": True, "ESCUCHA_CUERPO": True, "ESCUCHA_PEDIDOS": True, "ESCUCHA_ALIVIO": True},
    "azar": {"MENTE": "azar", "REFLEJOS": True},
    "todo": {"CORTEZA": True, "TALAMO": True, "PIDE": True, "FRASE": True, "PALABRA_SUELTA": True,
             "COMPOSICION": 8000, "EGO": True, "CAMBIO_SIGNIFICADO": 12000},
}

if __name__ == "__main__":
    aqui = os.path.dirname(os.path.abspath(__file__))
    for nombre, ajustes in CONFIGS.items():
        importlib.reload(C)
        for k, v in ajustes.items():
            assert hasattr(C, k), k
            setattr(C, k, type(getattr(C, k))(v))
        with open(os.path.join(aqui, f"config-{nombre}.txt"), "w", encoding="utf-8", newline="\n") as f:
            f.write(medir.config_texto())
        print(nombre)
