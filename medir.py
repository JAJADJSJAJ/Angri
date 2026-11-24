"""Vara de medir: corre a Angri sin panel ni disco y saca métricas.

    python medir.py                  -> 50.000 ticks, semillas 0..4
    python medir.py 20000 10         -> 20.000 ticks, 10 semillas
    python medir.py 50000 5 azar     -> mente vieja al azar puro (el piso)
    python medir.py 50000 5 reflejos -> mente vieja con reflejos innatos
    python medir.py 50000 8 PESO_CURIOSIDAD=0 -> cambia un número de config (ablación)
    python medir.py 50000 32 -j 12   -> reparte las vidas en 12 procesos (mismo resultado)

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
    alto = 0    # ticks con malestar alto (ética: que no haya bucles de sufrimiento)
    bosque = C.MUNDO == "bosque"
    sed = frio = mordidas = enfermo = bebidas = dormido = refugio = 0
    # olvido: tras cada estación, cuánto tarda en comer en la fuente nueva y
    # cuántos ticks sigue rondando la vieja (perseveración) en la ventana siguiente
    tardanzas, rondas = [], []
    pendiente = None  # [nueva, vieja, ticks desde el cambio, ticks cerca de la vieja]
    ventana = C.ESTACION // 2 if C.ESTACION else 0

    def cerca(p):
        return abs(a.x - p[0]) <= C.RADIO_FUENTE and abs(a.y - p[1]) <= C.RADIO_FUENTE

    for t in range(ticks):
        a.evento = ""
        cambio = a.mundo.cambio
        a.tick()
        if a.mundo.cambio is not cambio:
            if pendiente:
                tardanzas.append(ventana)
                rondas.append(pendiente[3])
            vieja, nueva = a.mundo.cambio
            pendiente = [nueva, vieja, 0, 0]
        if pendiente:
            pendiente[2] += 1
            pendiente[3] += cerca(pendiente[1]) and a.despierto
            if a.evento == "come" and cerca(pendiente[0]):
                tardanzas.append(pendiente[2])
                rondas.append(pendiente[3])
                pendiente = None
            elif pendiente[2] >= ventana:
                tardanzas.append(ventana)
                rondas.append(pendiente[3])
                pendiente = None
        m = a.cuerpo.malestar()
        malestar += m
        if t >= corte:
            final += m
        heridas += a.evento == "se lastima"
        comidas += a.evento == "come"
        novedades += a.evento == "descubre algo nuevo"
        hambre += a.cuerpo.v["energia"] < 0.3
        alto += m > C.MALESTAR_ALTO
        if bosque:
            v = a.cuerpo.v
            sed += v["agua"] < 0.3
            frio += v["calor"] < 0.4
            mordidas += a.evento == "lo muerde el predador"
            enfermo += a.evento == "se enferma"
            bebidas += a.evento == "bebe"
            if not a.despierto:
                dormido += 1
                refugio += a.mundo.zona[a.y][a.x] == "q"
    extra = {}
    if bosque:
        extra = {"% sed": 100 * sed / ticks, "% frío": 100 * frio / ticks,
                 "mordidas": mordidas, "enfermó": enfermo, "bebe": bebidas,
                 "% duerme cueva": 100 * refugio / max(1, dormido),
                 "desmayos": a.desmayos}
    return {
        "malestar": malestar / ticks,
        "mal. final": final / (ticks - corte),
        "heridas": heridas,
        "comidas": comidas,
        "novedades": novedades,
        "suenos": a.suenos,
        "% hambre": 100 * hambre / ticks,
        # vida trabada: pasa más del 60 % con hambre (queda encerrada lejos de la comida)
        "% trabadas": 100.0 * (hambre / ticks > 0.6),
        "% mal alto": 100 * alto / ticks,
        **extra,
        "encuentra": st.mean(tardanzas) if tardanzas else 0.0,
        "ronda vieja": st.mean(rondas) if rondas else 0.0,
        **_lenguaje(a, ticks),
    }


def _lenguaje(a, ticks):
    """Cuánto entiende: % de veces que apostó bien para dónde estaba lo que le dijeron."""
    ac = a.oido.aciertos
    if not ac:
        return {}
    tramo = ac[int(len(ac) * 0.8):]
    nuevas = [x for x, n in ac[len(ac) // 5:] if n]
    out = {}
    if getattr(a, "habla", None) and a.habla.pedidos:
        ped = a.habla.pedidos
        tramo_p = [ok for ok, e in ped if e >= ticks * 0.8] or [ok for ok, _ in ped]
        out["pide bien %"] = 100 * st.mean(tramo_p)
        out["pedidos"] = len(ped)
    out |= {"entiende %": 100 * st.mean(x for x, _ in tramo),
           "confianza": a.oido.confianza}
    if nuevas:
        out["frase nueva %"] = 100 * st.mean(nuevas)
    if C.COMPOSICION and a.oido.indice_composicion is not None:
        # primeras 20 apuestas con frases de cama nunca oídas (las palabras de
        # dirección solo las aprendió con comida)
        # (también suma aciertos y casos, para el % juntando todas las vidas)
        nuevas = [x for x, n in ac[a.oido.indice_composicion:] if n][:20]
        out["examen %"] = 100 * st.mean(a.oido.examen)
        out["compone n"] = len(nuevas)
        out["compone ok"] = sum(nuevas)
    if C.CAMBIO_SIGNIFICADO:
        # tras el cambio: % de aciertos en las 100 apuestas siguientes
        i = a.oido.indice_cambio
        if i is not None:
            out["reaprende %"] = 100 * st.mean(x for x, _ in ac[i:i + 100] or [(0, 0)])
    return out


_ORIGINAL = {k: v for k, v in vars(C).items() if k.isupper()}


def _ajustar(ajustes, modo):
    # (un proceso del Pool corre varias vidas: siempre parte de la config original)
    for k, v in _ORIGINAL.items():
        setattr(C, k, v)
    M.REFLEJOS = C.REFLEJOS
    for k, v in ajustes:
        setattr(C, k, type(getattr(C, k))(eval(v)))
    if modo:
        C.MENTE = "azar"
        C.REFLEJOS = M.REFLEJOS = modo == "reflejos"


def _vida(tarea):
    """Una vida en otro proceso: hay que repetir los ajustes (Windows arranca de cero)."""
    semilla, ticks, ajustes, modo = tarea
    _ajustar(ajustes, modo)
    return una_vida(semilla, ticks)


def main():
    # ajustes sueltos para ablaciones: python medir.py 50000 8 PESO_CURIOSIDAD=0
    ajustes = []
    for arg in [a for a in sys.argv[1:] if "=" in a]:
        ajustes.append(arg.split("=", 1))
        sys.argv.remove(arg)
    procesos = 1
    if "-j" in sys.argv:
        i = sys.argv.index("-j")
        procesos = int(sys.argv[i + 1])
        del sys.argv[i:i + 2]
    modo = next((m for m in ("azar", "reflejos") if m in sys.argv), None)
    _ajustar(ajustes, modo)
    ticks = int(sys.argv[1]) if len(sys.argv) > 1 else 50_000
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 5
    if procesos > 1:
        from multiprocessing import Pool
        with Pool(procesos) as pool:
            vidas = pool.map(_vida, [(s, ticks, ajustes, modo) for s in range(n)], chunksize=1)
    else:
        vidas = [una_vida(s, ticks) for s in range(n)]
    print(f"{n} vidas de {ticks} ticks, mente: {C.MENTE if C.MENTE == 'aprende' else ('reflejos' if C.REFLEJOS else 'azar puro')}\n")
    # e.e. = error estándar del promedio (desvío/√n): diferencias menores a ~2 e.e. son ruido
    print(f"{'métrica':<10} {'promedio':>10} {'desvío':>9} {'e.e.':>8} {'mediana':>9}")
    for k in max(vidas, key=len):
        xs = [v[k] for v in vidas if k in v]
        print(f"{k:<10} {st.mean(xs):>10.3f} {st.pstdev(xs):>9.3f} {st.pstdev(xs) / len(xs) ** 0.5:>8.3f} {st.median(xs):>9.3f}")


if __name__ == "__main__":
    main()
