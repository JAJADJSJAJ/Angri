"""Prueba: el núcleo en Rust da la MISMA vida que Python, tick a tick y bit a bit.

    python probar_rust.py [TICKS] [SEMILLAS] [CLAVE=VALOR ...]
    python probar_rust.py 20000 3 MUNDO='"bosque"'

Por cada semilla corre la vida en Python y en Rust (rust/target/release/angri)
y compara, cada tick: posición, si está despierto, malestar, cada variable del
cuerpo, adenosina y dopamina (los bits del float, no un redondeo). Después
compara las métricas de medir.una_vida.
"""
import json
import struct
import sys

from angri import config as C
from angri.ganglios import Ganglios
from angri.vida import Angri
import medir


def _bits(x):
    return format(struct.unpack("<Q", struct.pack("<d", x))[0], "016x")


def traza_python(semilla, ticks):
    a = Angri(semilla=semilla, cargar=False)
    for _ in range(ticks):
        a.tick()
        partes = [str(a.edad), str(a.x), str(a.y), str(int(a.despierto)), _bits(a.cuerpo.malestar())]
        partes += [_bits(v) for v in a.cuerpo.v.values()]
        partes += [_bits(a.quimica.adenosina), _bits(a.mente.dopamina if isinstance(a.mente, Ganglios) else 0.0)]
        yield " ".join(partes)


def main():
    for arg in [x for x in sys.argv[1:] if "=" in x]:
        k, v = arg.split("=", 1)
        setattr(C, k, type(getattr(C, k))(eval(v)))
        sys.argv.remove(arg)
    ticks = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    semillas = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    todo_bien = True
    for s in range(semillas):
        rs = medir.rust("traza", s, ticks).splitlines()
        distinto = None
        for i, (p, r) in enumerate(zip(traza_python(s, ticks), rs)):
            if p != r:
                distinto = (i + 1, p, r)
                break
        if distinto is None and len(rs) != ticks:
            distinto = (len(rs), "(Python sigue)", "(Rust cortó)")
        mp = json.loads(json.dumps(medir.una_vida(s, ticks)))
        mr = medir.una_vida_rust(s, ticks)
        metricas = mp == mr
        if distinto:
            t, p, r = distinto
            print(f"semilla {s}: DISTINTO en el tick {t}\n  python: {p}\n  rust:   {r}")
        print(f"semilla {s}: {ticks} ticks {'IDÉNTICOS' if not distinto else 'con diferencias'}, "
              f"métricas {'IGUALES' if metricas else 'DISTINTAS'}")
        if not metricas:
            for k in mp.keys() | mr.keys():
                if mp.get(k) != mr.get(k):
                    print(f"  {k}: python {mp.get(k)}  rust {mr.get(k)}")
        todo_bien &= not distinto and metricas
    print("IDÉNTICO" if todo_bien else "DISTINTO")
    return todo_bien


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.exit(0 if main() else 1)
