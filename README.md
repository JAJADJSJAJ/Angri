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
| `angri/ganglios.py` | Ganglios basales | Etapa 2: actor-crítico que aprende con dopamina (error TD) |
| `angri/mente.py` | — | Mente vieja (azar / reflejos), queda como vara de comparación |
| `medir.py` | — | Vara de medir: vidas reproducibles sin panel |
| `angri/vida.py` | El bucle | Latido continuo; se duerme y se despierta solo por adenosina |
| `angri/panel.py` | — | Muestra el mundo y su interior en la terminal |
| `vivir.py` | — | Punto de entrada |

## Etapa 1 — mundo y cuerpo ✅

Objetivo: verificar que está **vivo**: que se cansa, se duerme solo, se despierta recuperado y mantiene su equilibrio, sin que nada externo le diga cuándo.
En una prueba de 20.000 ticks durmió 21 veces y terminó con un malestar de 0,10.

## Vara de medir

`python medir.py` corre 5 vidas de 50.000 ticks reproducibles (semillas 0–4) sin tocar `datos/`.
Con `azar` o `reflejos` al final se mide la mente vieja. El mundo está calibrado para que sin cerebro **sufra**:
poca comida (tope de 10) y 5 peligros que deambulan.

| Mente (5–8 vidas × 50.000 ticks) | Malestar | Malestar final (último 20 %) | % tiempo con hambre |
|---|---|---|---|
| Azar puro — el piso | 0,56 | 0,58 | ~47 % |
| **Aprende (etapa 2)** | **0,22** | **0,23** | 45 % |
| Reflejos innatos — la meta | 0,18 | 0,19 | ~27 % |

La comida ahora crece alrededor de 3 **fuentes fijas** (como árboles frutales), así que el mundo tiene
estructura que vale la pena aprender.

## Etapa 2 — estado

Funciona: sin saber qué es la comida ni el peligro, aprende solo a esquivar peligros y sufre la mitad que el azar.
Lo que todavía falla:

- **Hambre:** moverse cuesta energía al instante y la comida es rara, así que a veces aprende que
  "quedarse quieto" es mejor. Con hambre extrema (energía en 0) haga lo que haga se siente igual: no hay señal.
- **Olvido en vidas largas:** en 150.000 ticks algunas vidas que comían bien dejan de comer
  (malestar final 0,33 con mucha variación entre vidas). Todavía no se sabe por qué.

Arreglado:

- **Se quedaba parado arriba de un peligro:** había aprendido "la X duele" solo cuando tenía hambre; con
  otra necesidad urgente no lo sabía. Ahora hay rasgos que valen para cualquier necesidad. Heridas: de cientos a ~10.
- **Indefensión aprendida:** si de chico casi no comía, nunca aprendía que comer es bueno y se quedaba
  en un círculo vicioso. Ahora **cuanto peor la pasa, más varía lo que hace** (`TEMPERATURA_MALESTAR`).

Probado y **apagado** porque empeoraba (regla: si no mejora, se saca):

| Idea | Resultado |
|---|---|
| Inanición daña la integridad | Mucho peor |
| Sin noradrenalina (exploración por sorpresa) | Peor: la sorpresa ayuda |
| Calma: aprender más despacio con la experiencia (`CALMA`) | Peor: congela errores de la infancia |
| Memoria de lugares donde se sintió bien (`UMBRAL_RECUERDO`) | Peor: sin consolidación, recuerda ruido |

La memoria de lugares queda en el código para la etapa 4 (hipocampo + sueño), donde tiene más sentido.

## Hoja de ruta

1. ✅ Mundo y cuerpo
2. 🔶 Dopamina: actor-crítico que aprende a mantenerse en equilibrio (funciona, falta estabilizar)
3. Predicción del mundo y curiosidad por progreso de aprendizaje
4. Hipocampo y sueño real: replay del día, poda y medir olvido con y sin dormir
5. Tálamo y espacio global: módulos que compiten por la atención
6. Lenguaje, con un modelo chico como maestro
7. Mundo real: internet, sensores de la PC como "cuerpo" y vida continua

## Reglas del proyecto

- Nada de scripts que le digan qué hacer: las reglas viven dentro del sistema.
- Cada pieza "cerebral" se mide; si no mejora nada, se saca.
- Tope al malestar (`MALESTAR_MAXIMO`) y registro de su interior, siempre.
