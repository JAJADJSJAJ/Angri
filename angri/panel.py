"""Panel en la terminal: el mundo y el interior de Angri en vivo."""
import os

from . import config as C


def _barra(valor, ancho=20):
    llenos = int(round(max(0.0, min(1.0, valor)) * ancho))
    return "█" * llenos + "░" * (ancho - llenos)


def dibujar(angri):
    filas = []
    for y in range(C.ALTO):
        fila = ""
        for x in range(C.ANCHO):
            if (x, y) == (angri.x, angri.y):
                fila += "A" if angri.despierto else "z"
            else:
                fila += angri.mundo.en(x, y)
        filas.append(fila)

    v = angri.cuerpo.v
    q = angri.quimica
    estado = "DESPIERTO" if angri.despierto else "DURMIENDO"
    lineas = [
        f"ANGRI · edad {angri.edad} ticks · {estado} · sueños: {angri.suenos}",
        f"último evento: {angri.evento}",
        "",
        *filas,
        "",
        "CUERPO",
        *(f"  {k:<11}{_barra(val)} {val:.2f}" for k, val in v.items()),
        "QUÍMICA",
        f"  {'adenosina':<11}{_barra(q.adenosina)} {q.adenosina:.2f}  (dormir ≥ {C.UMBRAL_DORMIR})",
        f"  {'cortisol':<11}{_barra(q.cortisol)} {q.cortisol:.2f}",
        f"  {'noradren.':<11}{_barra(q.noradrenalina)} {q.noradrenalina:.2f}",
        f"  {'dopamina':<11}{q.dopamina:+.2f}",
        "",
        f"malestar: {_barra(angri.cuerpo.malestar())} {angri.cuerpo.malestar():.2f}"
        f" · necesidad urgente: {angri.cuerpo.necesidad_urgente()}"
        + (f" · foco: {angri.talamo.foco}" if getattr(angri, "talamo", None) else ""),
        *([f"oye: {angri.maestro.dicho}"] if getattr(angri, "maestro", None) and angri.maestro.dicho else []),
        "",
        "Ctrl+C para pausar (guarda su estado y sigue desde ahí la próxima vez)",
    ]
    os.system("cls" if os.name == "nt" else "clear")
    print("\n".join(lineas))
