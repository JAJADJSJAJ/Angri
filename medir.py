"""Vara de medir: corre a Angri sin panel ni disco y saca métricas.

    python medir.py                  -> 50.000 ticks, semillas 0..4
    python medir.py 20000 10         -> 20.000 ticks, 10 semillas
    python medir.py 50000 5 azar     -> mente vieja al azar puro (el piso)
    python medir.py 50000 5 reflejos -> mente vieja con reflejos innatos

Cada Angri nace de cero (no toca datos/). Misma semilla = misma vida exacta,
así dos versiones de la mente se pueden comparar en igualdad de condiciones.
"""
import statistics as st
import sys

sys.stdout.reconfigure(encoding="utf-8")

import angri.config as C
import angri.mente as M
from angri.vida import Angri


def una_vida(semilla, ticks):
    a = Angri(semilla=semilla, cargar=False)
    final = 0.0  # malestar en el último 20 % de la vida (ya aprendió)
    corte = int(ticks * 0.8)
    malestar = heridas = comidas = novedades = 0
    hambre = 0  # ticks con energía < 0.3
    for t in range(ticks):
        a.evento = ""
        a.tick()
        m = a.cuerpo.malestar()
        malestar += m
        if t >= corte:
            final += m
        heridas += a.evento == "se lastima"
        comidas += a.evento == "come"
        novedades += a.evento == "descubre algo nuevo"
        hambre += a.cuerpo.v["energia"] < 0.3
    return {
        "malestar": malestar / ticks,
        "mal. final": final / (ticks - corte),
        "heridas": heridas,
        "comidas": comidas,
        "novedades": novedades,
        "suenos": a.suenos,
        "% hambre": 100 * hambre / ticks,
    }


def main():
    ticks = int(sys.argv[1]) if len(sys.argv) > 1 else 50_000
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 5
    if "azar" in sys.argv or "reflejos" in sys.argv:
        C.MENTE = "azar"
        C.REFLEJOS = M.REFLEJOS = "reflejos" in sys.argv
    vidas = [una_vida(s, ticks) for s in range(n)]
    print(f"{n} vidas de {ticks} ticks, mente: {C.MENTE if C.MENTE == 'aprende' else ('reflejos' if C.REFLEJOS else 'azar puro')}\n")
    print(f"{'métrica':<10} {'promedio':>10} {'desvío':>9}")
    for k in vidas[0]:
        xs = [v[k] for v in vidas]
        print(f"{k:<10} {st.mean(xs):>10.3f} {st.pstdev(xs):>9.3f}")


if __name__ == "__main__":
    main()
