"""Memoria en disco completa (etapa 12): la mente entera, no un resumen.

Recorre todo el grafo de objetos de Angri (ganglios con pesos, V y huellas,
hipocampo, tálamo, léxico, Habla, maestro, cuerpo, mundo y los generadores
al azar) y lo escribe a JSON. Al cargar lo rearma igual, de modo que retomar
da exactamente la misma vida que no haber cortado nunca.

Codificación (JSON legible, Python puro):
  tupla        -> {"$t": [...]}
  set          -> {"$s": [...]}            (en su orden de iteración)
  dict raro    -> {"$d": [[k, v], ...]}     (claves no-str o que empiezan con $)
  random       -> {"$rng": estado}
  objeto       -> {"$o": "modulo.Clase", "a": {atributos}}
  compartido   -> {"$id": n, ...} la primera vez, {"$ref": n} las siguientes
Las referencias compartidas importan: ganglios, hipocampo y tálamo usan el
MISMO random; si al cargar fueran dos, la vida divergiría.
"""
import importlib
import json
import os
import random

_ATOMOS = (type(None), bool, int, float, str)


def _contar(obj, vistos):
    """Primera pasada: cuántas veces aparece cada objeto mutable."""
    if isinstance(obj, _ATOMOS):
        return
    i = id(obj)
    if i in vistos:
        vistos[i] += 1
        return
    vistos[i] = 1
    if isinstance(obj, (list, tuple, set, frozenset)):
        for x in obj:
            _contar(x, vistos)
    elif isinstance(obj, dict):
        for k, v in obj.items():
            _contar(k, vistos)
            _contar(v, vistos)
    elif isinstance(obj, random.Random):
        pass
    else:
        _contar(_atributos(obj), vistos)


def _atributos(obj):
    if not hasattr(obj, "__dict__"):
        raise TypeError(f"no sé guardar {type(obj).__name__}")
    return obj.__dict__


def _a_json(obj, vistos, ids):
    if isinstance(obj, _ATOMOS):
        return obj
    i = id(obj)
    if i in ids:
        return {"$ref": ids[i]}
    marca = {}
    if vistos.get(i, 0) > 1 and not isinstance(obj, tuple):
        ids[i] = len(ids)
        marca = {"$id": ids[i]}
    rec = lambda x: _a_json(x, vistos, ids)  # noqa: E731
    if isinstance(obj, list):
        return {**marca, "$l": [rec(x) for x in obj]} if marca else [rec(x) for x in obj]
    if isinstance(obj, tuple):
        return {"$t": [rec(x) for x in obj]}
    if isinstance(obj, (set, frozenset)):
        return {**marca, "$s": [rec(x) for x in obj]}
    if isinstance(obj, dict):
        if not marca and all(isinstance(k, str) and not k.startswith("$") for k in obj):
            return {k: rec(v) for k, v in obj.items()}
        return {**marca, "$d": [[rec(k), rec(v)] for k, v in obj.items()]}
    if isinstance(obj, random.Random):
        return {**marca, "$rng": rec(obj.getstate())}
    cls = type(obj)
    return {**marca, "$o": f"{cls.__module__}.{cls.__qualname__}",
            "a": {k: rec(v) for k, v in _atributos(obj).items()}}


def _de_json(x, objs):
    if isinstance(x, list):
        return [_de_json(v, objs) for v in x]
    if not isinstance(x, dict):
        return x
    if "$ref" in x:
        return objs[x["$ref"]]
    n = x.get("$id")

    def registrar(o):
        if n is not None:
            objs[n] = o
        return o

    if "$t" in x:
        return tuple(_de_json(v, objs) for v in x["$t"])
    if "$l" in x:
        o = registrar([])
        o.extend(_de_json(v, objs) for v in x["$l"])
        return o
    if "$s" in x:
        o = registrar(set())
        o.update(_de_json(v, objs) for v in x["$s"])
        return o
    if "$d" in x:
        o = registrar({})
        for k, v in x["$d"]:
            o[_de_json(k, objs)] = _de_json(v, objs)
        return o
    if "$rng" in x:
        o = registrar(random.Random())
        o.setstate(_de_json(x["$rng"], objs))
        return o
    if "$o" in x:
        modulo, _, clase = x["$o"].rpartition(".")
        cls = getattr(importlib.import_module(modulo), clase)
        o = registrar(cls.__new__(cls))
        o.__dict__.update({k: _de_json(v, objs) for k, v in x["a"].items()})
        return o
    return {k: _de_json(v, objs) for k, v in x.items()}


def volcar(obj):
    """Objeto -> estructura JSON."""
    vistos = {}
    _contar(obj, vistos)
    return _a_json(obj, vistos, {})


def rearmar(datos):
    """Estructura JSON -> objeto (con sus referencias compartidas)."""
    return _de_json(datos, {})


def guardar(obj, archivo, extra=None):
    """Guardado atómico: escribe a un temporal y lo renombra (un corte de luz
    deja el archivo viejo o el nuevo, nunca uno a medias)."""
    carpeta = os.path.dirname(os.path.abspath(archivo))
    os.makedirs(carpeta, exist_ok=True)
    tmp = archivo + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump({"version": 12, "extra": extra or {}, "mente": volcar(obj)}, f,
                  separators=(",", ":"))
        f.flush()
        os.fsync(f.fileno())
    os.replace(tmp, archivo)


def cargar(archivo):
    """Devuelve (objeto, extra) o None si no hay guardado completo."""
    try:
        with open(archivo, encoding="utf-8") as f:
            datos = json.load(f)
    except (OSError, ValueError):
        return None
    if datos.get("version") != 12:
        return None
    return rearmar(datos["mente"]), datos.get("extra", {})
