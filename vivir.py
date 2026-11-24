"""Vida continua de Angri (etapa 12): vive sin fin en esta PC.

    python vivir.py                  -> vive para siempre sin panel, guarda y anota
    python vivir.py --panel          -> igual, pero con panel a 10 ticks/seg
    python vivir.py --ticks N        -> vive N ticks más y se detiene (guarda al final)
    python vivir.py --carpeta DIR    -> dónde está su mente (por defecto datos/)
    python vivir.py --semilla S      -> semilla si nace (si ya hay mente, la retoma)
    python vivir.py --ventana W      -> ticks por renglón del log (por defecto 10.000)
    python vivir.py --guardar G      -> cada cuántos ticks guarda (por defecto 50.000)

Si existe DIR/mente.json retoma exactamente donde estaba (mismo random, mismas
huellas: la vida sigue como si no se hubiera cortado). Ctrl+C guarda y sale.
Log liviano en DIR/vida.csv: un renglón por ventana con malestar, % hambre,
heridas, comidas, ticks/seg, tamaño de las estructuras que crecen y RAM.
"""
import argparse
import csv
import os
import sys
import time

sys.stdout.reconfigure(encoding="utf-8")

from angri import memoria
from angri.vida import Angri

CAMPOS = ["edad", "malestar", "hambre_pct", "heridas", "comidas", "suenos",
          "ticks_seg", "elementos", "mayor", "mayor_n", "mente_kb", "ram_mb"]


def contar(obj, ruta="angri", vistos=None, tam=None):
    """Cuenta elementos de todos los contenedores; devuelve total y el mayor por ruta."""
    if vistos is None:
        vistos, tam = set(), {}
    if isinstance(obj, (type(None), bool, int, float, str)) or id(obj) in vistos:
        return tam
    vistos.add(id(obj))
    if isinstance(obj, (list, tuple, set, dict)):
        tam[ruta] = len(obj)
        hijos = obj.values() if isinstance(obj, dict) else obj
        if len(obj) <= 64:  # no baja a cada elemento de listas largas (son átomos o tuplas)
            for i, h in enumerate(hijos):
                contar(h, f"{ruta}[{i}]", vistos, tam)
        return tam
    if hasattr(obj, "__dict__"):
        for k, v in vars(obj).items():
            contar(v, f"{ruta}.{k}", vistos, tam)
    return tam


def ram_mb():
    """Memoria del proceso (Windows: ctypes sobre psapi; si no, 0)."""
    try:
        import ctypes
        from ctypes import wintypes

        class PMC(ctypes.Structure):
            _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
                (n, ctypes.c_size_t) for n in ("PeakWorkingSetSize", "WorkingSetSize",
                                               "QuotaPeakPagedPoolUsage", "QuotaPagedPoolUsage",
                                               "QuotaPeakNonPagedPoolUsage",
                                               "QuotaNonPagedPoolUsage", "PagefileUsage",
                                               "PeakPagefileUsage")]
        c = PMC()
        c.cb = ctypes.sizeof(c)
        k32 = ctypes.windll.kernel32
        k32.GetCurrentProcess.restype = wintypes.HANDLE
        f = ctypes.windll.psapi.GetProcessMemoryInfo
        f.argtypes = [wintypes.HANDLE, ctypes.POINTER(PMC), wintypes.DWORD]
        if not f(k32.GetCurrentProcess(), ctypes.byref(c), c.cb):
            raise OSError
        return c.WorkingSetSize / 2**20
    except Exception:
        try:
            import resource
            return resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024
        except Exception:
            return 0.0


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--carpeta", default=os.path.join(os.path.dirname(__file__), "datos"))
    p.add_argument("--ticks", type=int, default=None)
    p.add_argument("--semilla", type=int, default=None)
    p.add_argument("--ventana", type=int, default=10_000)
    p.add_argument("--guardar", type=int, default=50_000)
    p.add_argument("--panel", action="store_true")
    o = p.parse_args()

    archivo = os.path.join(o.carpeta, "mente.json")
    retomado = memoria.cargar(archivo)
    if retomado:
        angri, _ = retomado
        print(f"Retoma a Angri con {angri.edad} ticks vividos.")
    else:
        angri = Angri(semilla=o.semilla, cargar=False)
        print("Nace Angri.")
    log = os.path.join(o.carpeta, "vida.csv")
    os.makedirs(o.carpeta, exist_ok=True)
    nuevo = not os.path.exists(log)

    if o.panel:
        from angri import config as C
        from angri.panel import dibujar

    def guardar():
        memoria.guardar(angri, archivo)

    with open(log, "a", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=CAMPOS)
        if nuevo:
            w.writeheader()
        n = malestar = hambre = heridas = comidas = 0
        t0 = time.perf_counter()
        hechos = 0
        try:
            while o.ticks is None or hechos < o.ticks:
                angri.evento = ""
                angri.tick()
                hechos += 1
                n += 1
                malestar += angri.cuerpo.malestar()
                hambre += angri.cuerpo.v["energia"] < 0.3
                heridas += angri.evento == "se lastima"
                comidas += angri.evento == "come"
                if o.panel:
                    dibujar(angri)
                    time.sleep(1 / C.TICKS_POR_SEGUNDO)
                if angri.edad % o.ventana == 0:
                    dt = time.perf_counter() - t0
                    tam = contar(angri)
                    mayor = max(tam, key=tam.get)
                    kb = os.path.getsize(archivo) / 1024 if os.path.exists(archivo) else 0
                    fila = {"edad": angri.edad, "malestar": round(malestar / n, 4),
                            "hambre_pct": round(100 * hambre / n, 2), "heridas": heridas,
                            "comidas": comidas, "suenos": angri.suenos,
                            "ticks_seg": round(n / dt), "elementos": sum(tam.values()),
                            "mayor": mayor, "mayor_n": tam[mayor], "mente_kb": round(kb),
                            "ram_mb": round(ram_mb(), 1)}
                    w.writerow(fila)
                    f.flush()
                    if not o.panel:
                        print(" ".join(f"{k}={v}" for k, v in fila.items()), flush=True)
                    n = malestar = hambre = heridas = comidas = 0
                    t0 = time.perf_counter()
                if angri.edad % o.guardar == 0:
                    guardar()
        except KeyboardInterrupt:
            print("\nCtrl+C: guarda y duerme.")
        finally:
            guardar()
    print(f"Guardado en {archivo} ({angri.edad} ticks vividos).")


if __name__ == "__main__":
    main()
