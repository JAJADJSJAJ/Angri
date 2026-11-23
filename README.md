# Angri — un cerebro artificial que vive solo

Guía completa de investigación y diseño: https://claude.ai/artifact/2q9F7WwrnzrdKVjJkUAwjg

## Cómo correrlo

Solo necesita Python 3.10 o más nuevo. No usa librerías externas.

```
python vivir.py              # vive para siempre, con panel en vivo
python vivir.py --rapido 20000   # vive 20.000 ticks sin panel (para probar)
```

Con Ctrl+C se pausa: guarda su estado en `datos/estado.json` y la próxima vez sigue desde ahí.
Todo lo que vive queda anotado en `datos/diario.csv` (cuerpo, química y eventos cada 10 ticks).
Para que "renazca" desde cero, borrá la carpeta `datos/`.

## Estructura (una región por archivo)

| Archivo | Región | Qué hace |
|---|---|---|
| `angri/config.py` | — | Todos los números que se pueden afinar |
| `angri/cuerpo.py` | Hipotálamo / tronco | Energía, cansancio, curiosidad, estrés, integridad; malestar = distancia al equilibrio |
| `angri/quimica.py` | Neuromoduladores | Adenosina (sueño), cortisol, dopamina (¿mejoró el equilibrio?), noradrenalina (sorpresa) |
| `angri/mundo.py` | El entorno | Grilla 2D con comida `*`, descanso `~`, novedades `?` y peligros `X` |
| `angri/mente.py` | Corteza (provisoria) | Por ahora: azar + reflejos simples. Se reemplaza en la etapa 2 |
| `angri/vida.py` | El bucle | Latido continuo; se duerme y se despierta solo por adenosina |
| `angri/panel.py` | — | Muestra el mundo y su interior en la terminal |
| `vivir.py` | — | Punto de entrada |

## Etapa actual: 1 — mundo y cuerpo

Objetivo: verificar que está **vivo**: que se cansa, se duerme solo, se despierta recuperado y mantiene su equilibrio, sin que nada externo le diga cuándo.
En una prueba de 20.000 ticks durmió 21 veces y terminó con un malestar de 0,10.

## Hoja de ruta

1. ✅ Mundo y cuerpo
2. Dopamina: actor-crítico que aprende a mantenerse en equilibrio (reemplaza `mente.py`)
3. Predicción del mundo y curiosidad por progreso de aprendizaje
4. Hipocampo y sueño real: replay del día, poda y medir olvido con y sin dormir
5. Tálamo y espacio global: módulos que compiten por la atención
6. Lenguaje, con un modelo chico como maestro
7. Mundo real: internet, sensores de la PC como "cuerpo" y vida continua

## Reglas del proyecto

- Nada de scripts que le digan qué hacer: las reglas viven dentro del sistema.
- Cada pieza "cerebral" se mide; si no mejora nada, se saca.
- Tope al malestar (`MALESTAR_MAXIMO`) y registro de su interior, siempre.
