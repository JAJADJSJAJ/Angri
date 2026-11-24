"""Prueba: la foto del visor (angri/foto.py) que arma Rust es la MISMA que la de
Python, byte a byte, para la misma vida.

    python probar_foto.py [SEMILLAS] [TICKS] [CADA] [CLAVE=VALOR ...]
    python probar_foto.py 3 20000 7 MUNDO='"bosque"'

Compara una foto cada CADA ticks (y cuenta qué eventos aparecieron).
"""
import json
import sys

from angri import config as C
from angri import foto as F
import angri.mente as M
from angri.vida import Angri
import medir


def fotos_python(semilla, ticks, cada):
    a = Angri(semilla=semilla, cargar=False)
    for t in range(1, ticks + 1):
        a.evento = ""
        a.tick()
        if t % cada == 0:
            yield json.dumps(F.foto(a), ensure_ascii=False, separators=(",", ":"))


def main():
    for arg in [x for x in sys.argv[1:] if "=" in x]:
        k, v = arg.split("=", 1)
        setattr(C, k, type(getattr(C, k))(eval(v)))
        sys.argv.remove(arg)
    M.REFLEJOS = C.REFLEJOS
    semillas = int(sys.argv[1]) if len(sys.argv) > 1 else 3
    ticks = int(sys.argv[2]) if len(sys.argv) > 2 else 5000
    cada = int(sys.argv[3]) if len(sys.argv) > 3 else 7
    bien = True
    eventos = set()
    for s in range(semillas):
        rs = medir.rust("foto", s, ticks, cada).splitlines()
        n = 0
        for n, (p, r) in enumerate(zip(fotos_python(s, ticks, cada), rs), 1):
            eventos.add(json.loads(p)["evento"])
            if p != r:
                i = next((i for i, (x, y) in enumerate(zip(p, r)) if x != y), min(len(p), len(r)))
                print(f"semilla {s}: foto {n} DISTINTA")
                print(f"  python: ...{p[max(0, i - 60):i + 60]}")
                print(f"  rust:   ...{r[max(0, i - 60):i + 60]}")
                bien = False
                break
        else:
            bien &= n == len(rs) == ticks // cada
        print(f"semilla {s}: {n} fotos comparadas")
    print("eventos vistos:", sorted(eventos))
    print("IDÉNTICO" if bien else "DISTINTO")
    return bien


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.exit(0 if main() else 1)
