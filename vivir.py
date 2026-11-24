"""Vida continua de Angri (etapa 12): vive sin fin en esta PC.

    python vivir.py                  -> vive para siempre sin panel, guarda y anota
    python vivir.py --panel          -> igual, pero con panel a 10 ticks/seg
    python vivir.py --ticks N        -> vive N ticks más y se detiene (guarda al final)
    python vivir.py --carpeta DIR    -> dónde está su mente (por defecto datos/)
    python vivir.py --semilla S      -> semilla si nace (si ya hay mente, la retoma)
    python vivir.py --ventana W      -> ticks por renglón del log (por defecto 10.000)
    python vivir.py --guardar G      -> cada cuántos ticks guarda (por defecto 50.000)

    python vivir.py --estado         -> cómo anda (edad, últimas ventanas, RAM, último guardado)

Si existe DIR/mente.json retoma exactamente donde estaba (mismo random, mismas
huellas: la vida sigue como si no se hubiera cortado). Ctrl+C o SIGTERM (lo que
manda systemd al parar) guardan y salen. Cada guardado deja además una copia en
DIR/respaldos/ (las últimas 5 + una por día, 30 días); si mente.json no carga,
retoma de la copia válida más reciente y lo avisa.
Log liviano en DIR/vida.csv: un renglón por ventana con malestar, % hambre,
heridas, comidas, ticks/seg, tamaño de las estructuras que crecen y RAM.
"""
import argparse
import csv
import glob
import json
import os
import shutil
import signal
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


RESPALDOS_ULTIMOS = 5
RESPALDOS_DIAS = 30


_PARAR = []  # SIGTERM levanta la bandera; el bucle termina el tick en curso y sale


def _sigterm(signum, frame):
    _PARAR.append(signum)


def respaldar(archivo, edad):
    """Copia mente.json a respaldos/: rotativa (últimas N) y una por día."""
    carpeta = os.path.join(os.path.dirname(archivo), "respaldos")
    os.makedirs(carpeta, exist_ok=True)
    destinos = [os.path.join(carpeta, f"mente-{edad:012d}.json"),
                os.path.join(carpeta, f"dia-{time.strftime('%Y-%m-%d')}.json")]
    for d in destinos:
        shutil.copyfile(archivo, d + ".tmp")
        os.replace(d + ".tmp", d)
    for patron, quedan in (("mente-*.json", RESPALDOS_ULTIMOS), ("dia-*.json", RESPALDOS_DIAS)):
        for viejo in sorted(glob.glob(os.path.join(carpeta, patron)))[:-quedan]:
            os.remove(viejo)


def retomar(archivo, molde):
    """Carga mente.json; si no carga, la copia válida más reciente de respaldos/."""
    r = memoria.cargar(archivo, molde)
    if r or not os.path.exists(archivo):
        return r
    copias = glob.glob(os.path.join(os.path.dirname(archivo), "respaldos", "*.json"))
    for c in sorted(copias, key=os.path.getmtime, reverse=True):
        r = memoria.cargar(c, molde)
        if r:
            print(f"AVISO: {archivo} no cargó; retoma del respaldo {c} "
                  f"({r[0].edad} ticks).", flush=True)
            return r
    raise SystemExit(f"{archivo} no carga y no hay respaldo válido: no hago nacer "
                     "otra Angri encima. Revisalo a mano.")


def estado(carpeta):
    """Resumen para mirar por SSH."""
    archivo = os.path.join(carpeta, "mente.json")
    log = os.path.join(carpeta, "vida.csv")
    if os.path.exists(archivo):
        hace = time.time() - os.path.getmtime(archivo)
        extra = {}
        try:
            with open(archivo, encoding="utf-8") as f:
                extra = json.load(f).get("extra", {})
        except (OSError, ValueError):
            print("AVISO: mente.json no se puede leer")
        print(f"Mente: {archivo} ({os.path.getsize(archivo) / 1024:.0f} KB), "
              f"edad {extra.get('edad', '?')} al guardar, guardada hace {_dur(hace)}")
    else:
        print(f"No hay mente en {archivo}.")
    resp = sorted(glob.glob(os.path.join(carpeta, "respaldos", "*.json")))
    print(f"Respaldos: {len(resp)}" + (f" (último {os.path.basename(max(resp, key=os.path.getmtime))})" if resp else ""))
    if not os.path.exists(log):
        print("No hay vida.csv.")
        return
    with open(log, encoding="utf-8") as f:
        filas = list(csv.DictReader(f))
    if not filas:
        return
    u = filas[-1]
    print(f"Última ventana anotada hace {_dur(time.time() - os.path.getmtime(log))}: "
          f"edad {u['edad']}, {u['ticks_seg']} ticks/seg, RAM {u['ram_mb']} MB")
    cols = ["edad", "malestar", "hambre_pct", "heridas", "comidas", "ticks_seg", "ram_mb"]
    print("  ".join(f"{c:>10}" for c in cols))
    for fila in filas[-10:]:
        print("  ".join(f"{fila[c]:>10}" for c in cols))


def _dur(s):
    s = int(s)
    if s < 120:
        return f"{s} s"
    if s < 7200:
        return f"{s // 60} min"
    if s < 172800:
        return f"{s / 3600:.1f} h"
    return f"{s / 86400:.1f} días"


def ram_mb():
    """RAM actual del proceso: Linux /proc/self/status (VmRSS), Windows psapi,
    si no resource (pico)."""
    try:
        with open("/proc/self/status") as f:
            for renglon in f:
                if renglon.startswith("VmRSS:"):
                    return int(renglon.split()[1]) / 1024
    except OSError:
        pass
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


def main(argv=None):
    p = argparse.ArgumentParser()
    p.add_argument("--carpeta", default=os.path.join(os.path.dirname(__file__), "datos"))
    p.add_argument("--ticks", type=int, default=None)
    p.add_argument("--semilla", type=int, default=None)
    p.add_argument("--ventana", type=int, default=10_000)
    p.add_argument("--guardar", type=int, default=50_000)
    p.add_argument("--panel", action="store_true")
    p.add_argument("--estado", action="store_true")
    o = p.parse_args(argv)
    if o.estado:
        return estado(o.carpeta)
    _PARAR.clear()
    signal.signal(signal.SIGTERM, _sigterm)

    archivo = os.path.join(o.carpeta, "mente.json")
    retomado = retomar(archivo, memoria.molde_angri)
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
        memoria.guardar(angri, archivo, {"edad": angri.edad, "cuando": time.time()})
        try:
            respaldar(archivo, angri.edad)
        except OSError as e:
            print(f"AVISO: no pude respaldar ({e})", flush=True)

    with open(log, "a", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=CAMPOS)
        if nuevo:
            w.writeheader()
        n = malestar = hambre = heridas = comidas = 0
        t0 = time.perf_counter()
        hechos = 0
        try:
            while (o.ticks is None or hechos < o.ticks) and not _PARAR:
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
