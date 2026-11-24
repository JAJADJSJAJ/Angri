"""Memoria en disco completa (etapa 12): la mente entera, no un resumen.

Recorre todo el grafo de objetos de Angri (ganglios con pesos, V y huellas,
hipocampo, tálamo, léxico, Habla, maestro, cuerpo, mundo y los generadores
al azar) y lo escribe a JSON. Al cargar lo rearma igual, de modo que retomar
da exactamente la misma vida que no haber cortado nunca.

Codificación (JSON legible, Python puro):
  tupla        -> {"$t": [...]}
  set          -> {"$s": [...]}            (ordenado si se puede)
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


def _ordenado(conjunto):
    """Sets en orden fijo: su orden de iteración depende de la historia de
    inserciones y cambia al recargar, aunque el contenido sea el mismo."""
    try:
        return sorted(conjunto)
    except TypeError:
        return list(conjunto)


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
        return {**marca, "$s": [rec(x) for x in _ordenado(obj)]}
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


def _objetos(raiz):
    """Todos los objetos "de clase" (no contenedores ni random) alcanzables."""
    vistos, pila, out = set(), [raiz], []
    while pila:
        o = pila.pop()
        if isinstance(o, _ATOMOS) or isinstance(o, random.Random) or id(o) in vistos:
            continue
        vistos.add(id(o))
        if isinstance(o, dict):
            pila.extend(o.keys())
            pila.extend(o.values())
        elif isinstance(o, (list, tuple, set, frozenset)):
            pila.extend(o)
        elif hasattr(o, "__dict__"):
            out.append(o)
            pila.extend(o.__dict__.values())
    return out


def completar(obj, molde):
    """Compatibilidad frente a cambios de código: compara cada objeto cargado
    con uno de la misma clase en `molde` (una mente recién nacida, construida
    aparte: no toca el random ni nada de la cargada). Atributos que la clase
    tiene ahora y el guardado no trae se copian del molde (valor por defecto);
    los que sobran se dejan. Devuelve los avisos. Si no falta nada, la mente
    queda intacta (retomar sigue siendo bit-idéntico)."""
    import copy
    por_clase = {}
    for m in _objetos(molde):
        por_clase.setdefault(type(m), m)
    avisos, avisado = [], set()
    for o in _objetos(obj):
        m = por_clase.get(type(o))
        if m is None:
            continue
        nombre = type(o).__qualname__
        for k in m.__dict__.keys() - o.__dict__.keys():
            o.__dict__[k] = copy.deepcopy(m.__dict__[k])
            if (nombre, k, "falta") not in avisado:
                avisado.add((nombre, k, "falta"))
                avisos.append(f"{nombre}.{k}: no estaba en el guardado, toma el valor por defecto")
        for k in o.__dict__.keys() - m.__dict__.keys():
            if (nombre, k, "sobra") not in avisado:
                avisado.add((nombre, k, "sobra"))
                avisos.append(f"{nombre}.{k}: está en el guardado pero el código nuevo no lo crea (se deja)")
    return avisos


def molde_angri(ticks=200):
    """Mente de referencia para `completar`: nace aparte y vive un poco, así ya
    tiene los atributos que se crean en el primer tick (no los da por sobrantes)."""
    from angri.vida import Angri
    m = Angri(semilla=12345, cargar=False)
    for _ in range(ticks):
        m.tick()
    return m


def config_actual():
    """Valores en mayúscula de angri/config.py (lo que se puede afinar)."""
    from angri import config as C
    return {k: getattr(C, k) for k in sorted(dir(C))
            if k.isupper() and isinstance(getattr(C, k), (_ATOMOS, list, tuple, dict))}


def comparar_config(guardada):
    """Avisos de valores de config que cambiaron desde que se guardó."""
    if guardada is None:
        return []
    ahora = rearmar(volcar(config_actual()))
    guardada = rearmar(guardada)
    avisos = []
    for k in sorted(set(ahora) | set(guardada)):
        if k not in guardada:
            avisos.append(f"config {k} nuevo: {ahora[k]!r}")
        elif k not in ahora:
            avisos.append(f"config {k} ya no existe (era {guardada[k]!r})")
        elif ahora[k] != guardada[k]:
            avisos.append(f"config {k} cambió: {guardada[k]!r} -> {ahora[k]!r}")
    return avisos


def guardar(obj, archivo, extra=None):
    """Guardado atómico: escribe a un temporal y lo renombra (un corte de luz
    deja el archivo viejo o el nuevo, nunca uno a medias). Guarda también la
    config con que vivía."""
    carpeta = os.path.dirname(os.path.abspath(archivo))
    os.makedirs(carpeta, exist_ok=True)
    tmp = archivo + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump({"version": 12, "extra": extra or {}, "config": volcar(config_actual()),
                   "mente": volcar(obj)}, f, separators=(",", ":"))
        f.flush()
        os.fsync(f.fileno())
    os.replace(tmp, archivo)


def cargar(archivo, molde=None, avisar=print):
    """Devuelve (objeto, extra) o None si no hay guardado completo o válido.
    `molde`: función que crea una mente nueva, para completar atributos que
    falten si cambió el código. Los avisos (atributos, config) van a `avisar`."""
    try:
        with open(archivo, encoding="utf-8") as f:
            datos = json.load(f)
        if datos.get("version") != 12:
            return None
        obj = rearmar(datos["mente"])
    except (OSError, ValueError, KeyError, TypeError, AttributeError, ImportError) as e:
        if os.path.exists(archivo):
            avisar(f"AVISO: {archivo} no carga ({type(e).__name__}: {e})")
        return None
    avisos = comparar_config(datos.get("config"))
    if molde is not None:
        avisos += completar(obj, molde())
    for a in avisos:
        avisar("AVISO: " + a)
    return obj, datos.get("extra", {})
