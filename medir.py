"""Vara de medir: corre a Angri sin panel ni disco y saca métricas.

    python medir.py                  -> 50.000 ticks, semillas 0..4
    python medir.py 20000 10         -> 20.000 ticks, 10 semillas
    python medir.py 50000 5 reflejos -> con los reflejos innatos prendidos

Cada Angri nace de cero (no toca datos/). Misma semilla = misma vida exacta,
así dos versiones de la mente se pueden comparar en igualdad de condiciones.
"""
import statistics as st
import sys

import angri.config as C
import angri.mente as M
from angri.vida import Angri


def una_vida(semilla, ticks):
    a = Angri(semilla=semilla, cargar=False)
    malestar = heridas = comidas = novedades = 0
    hambre = 0  # ticks con energía < 0.3
    for _ in range(ticks):
        a.evento = ""
        a.tick()
        malestar += a.cuerpo.malestar()
        heridas += a.evento == "se lastima"
        comidas += a.evento == "come"
        novedades += a.evento == "descubre algo nuevo"
        hambre += a.cuerpo.v["energia"] < 0.3
    return {
        "malestar": malestar / ticks,
        "heridas": heridas,
        "comidas": comidas,
        "novedades": novedades,
        "suenos": a.suenos,
        "% hambre": 100 * hambre / ticks,
    }


def main():
    ticks = int(sys.argv[1]) if len(sys.argv) > 1 else 50_000
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 5
    if "reflejos" in sys.argv:
        C.REFLEJOS = M.REFLEJOS = True
    vidas = [una_vida(s, ticks) for s in range(n)]
    print(f"{n} vidas de {ticks} ticks, reflejos: {C.REFLEJOS}\n")
    print(f"{'métrica':<10} {'promedio':>10} {'desvío':>9}")
    for k in vidas[0]:
        xs = [v[k] for v in vidas]
        print(f"{k:<10} {st.mean(xs):>10.3f} {st.pstdev(xs):>9.3f}")


if __name__ == "__main__":
    main()
