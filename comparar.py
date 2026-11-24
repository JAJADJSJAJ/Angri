"""Comparación en pares: la misma semilla con la config A y con la B.

    python comparar.py 50000 128 "" "PIDE=True MAESTRO_SOLO_SI_PIDE=True" -j 12

Cada vida tiene su suerte (algunas quedan trabadas con hambre). Restando vida
contra vida con la misma semilla, esa suerte se cancela en buena parte: la
diferencia es mucho más precisa que comparar dos promedios sueltos.
Muestra B − A por métrica, con su error estándar; |dif| > 2 e.e. = diferencia real.
"""
import statistics as st
import sys
from multiprocessing import Pool

sys.stdout.reconfigure(encoding="utf-8")

from medir import _vida


def _ajustes(texto):
    return [a.split("=", 1) for a in texto.split()]


def main():
    args = sys.argv[1:]
    procesos = 1
    if "-j" in args:
        i = args.index("-j")
        procesos = int(args[i + 1])
        del args[i:i + 2]
    ticks, n, a, b = int(args[0]), int(args[1]), _ajustes(args[2]), _ajustes(args[3])
    tareas = [(s, ticks, aj, None) for aj in (a, b) for s in range(n)]
    with Pool(procesos) as pool:
        vidas = pool.map(_vida, tareas, chunksize=1)
    va, vb = vidas[:n], vidas[n:]
    print(f"{n} pares de {ticks} ticks\nA: {args[2] or '(config actual)'}\nB: {args[3]}\n")
    print(f"{'métrica':<12} {'A':>9} {'B':>9} {'B−A':>9} {'e.e.':>8} {'z':>6}")
    for k in va[0]:
        pares = [(x[k], y[k]) for x, y in zip(va, vb) if k in x and k in y]
        if len(pares) < 2:
            continue
        d = [y - x for x, y in pares]
        ee = st.stdev(d) / len(d) ** 0.5
        z = st.mean(d) / ee if ee else 0.0
        print(f"{k:<12} {st.mean(x for x, _ in pares):>9.3f} {st.mean(y for _, y in pares):>9.3f}"
              f" {st.mean(d):>9.3f} {ee:>8.3f} {z:>6.1f}")


if __name__ == "__main__":
    main()
