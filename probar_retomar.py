"""Prueba de la etapa 12: retomar da la misma vida, bit a bit.

    python probar_retomar.py [TICKS] [SEMILLA] [CLAVE=VALOR ...]
    python probar_retomar.py 20000 0 MUNDO='"bosque"'   -> con el bosque (etapa 13a)

Corre TICKS de un tirón; aparte corre TICKS/2, guarda a disco, carga en un
objeto nuevo y corre la otra mitad. Compara la mente entera serializada
(pesos, V, huellas, hipocampo, léxico, cuerpo, mundo, random): debe ser igual.
"""
import json
import os
import sys
import tempfile

from angri import memoria
from angri.vida import Angri


def main():
    from angri import config as C
    for arg in [x for x in sys.argv[1:] if "=" in x]:
        k, v = arg.split("=", 1)
        setattr(C, k, type(getattr(C, k))(eval(v)))
        sys.argv.remove(arg)
    ticks = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    semilla = int(sys.argv[2]) if len(sys.argv) > 2 else 0
    a = Angri(semilla=semilla, cargar=False)
    for _ in range(ticks):
        a.tick()
    b = Angri(semilla=semilla, cargar=False)
    for _ in range(ticks // 2):
        b.tick()
    archivo = os.path.join(tempfile.mkdtemp(), "mente.json")
    memoria.guardar(b, archivo)
    del b
    c, _ = memoria.cargar(archivo, memoria.molde_angri)
    for _ in range(ticks - ticks // 2):
        c.tick()
    ja = json.dumps(memoria.volcar(a), sort_keys=True)
    jc = json.dumps(memoria.volcar(c), sort_keys=True)
    print(f"{ticks} ticks, semilla {semilla}: edad {a.edad}/{c.edad}, "
          f"malestar {a.cuerpo.malestar():.6f}/{c.cuerpo.malestar():.6f}, "
          f"mente {len(ja)} bytes")
    print("IDÉNTICO" if ja == jc else "DISTINTO")
    return ja == jc


if __name__ == "__main__":
    sys.exit(0 if main() else 1)
