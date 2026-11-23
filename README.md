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
| `angri/corteza.py` | Corteza | Etapa 3: predice qué va a encontrar y cómo cambia su malestar; curiosidad por progreso de aprendizaje (apagada) |
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
Con `azar` o `reflejos` al final se mide la mente vieja. Para ablaciones: `python medir.py 50000 16 PESO_CURIOSIDAD=0`. El mundo está calibrado para que sin cerebro **sufra**:
poca comida (tope de 10) y 5 peligros que deambulan.

| Mente (5–8 vidas × 50.000 ticks) | Malestar | Malestar final (último 20 %) | % tiempo con hambre |
|---|---|---|---|
| Azar puro — el piso | 0,57 | 0,58 | ~47 % |
| **Aprende (etapa 2)** | **0,19** | **0,16** | 36 % |
| **Aprende, vida larga (4 × 150.000)** | **0,14** | **0,13** | 17 % |
| Reflejos innatos — la meta | 0,17 | 0,17 | ~27 % |

La comida ahora crece alrededor de 3 **fuentes fijas** (como árboles frutales), así que el mundo tiene
estructura que vale la pena aprender.

## Etapa 2 — estado

Funciona: sin saber qué es la comida ni el peligro, aprende solo y **en vidas largas supera a los reflejos innatos**.
Lo que todavía se puede mejorar:

- **Hambre:** moverse cuesta energía al instante y la comida es rara, así que a veces aprende que
  "quedarse quieto" es mejor. Con hambre extrema (energía en 0) haga lo que haga se siente igual: no hay señal.
- **Infancia desigual:** en 50.000 ticks algunas vidas tardan mucho en aprender a comer (mucha variación entre vidas).

Arreglado:

- **Se quedaba parado arriba de un peligro:** había aprendido "la X duele" solo cuando tenía hambre; con
  otra necesidad urgente no lo sabía. Ahora hay rasgos que valen para cualquier necesidad. Heridas: de cientos a ~10.
- **Indefensión aprendida:** si de chico casi no comía, nunca aprendía que comer es bueno y se quedaba
  en un círculo vicioso. Ahora **cuanto peor la pasa, más varía lo que hace** (`TEMPERATURA_MALESTAR`).
- **"Olvido" en vidas largas que no era olvido:** aprendía a esquivar las novedades `?` (le bajan la curiosidad
  de más), nadie las consumía y llenaban el mundo hasta encerrar la comida. Ahora tienen tope (`NOVEDAD_MAXIMA`).

Probado y **apagado** porque empeoraba (regla: si no mejora, se saca):

| Idea | Resultado |
|---|---|
| Inanición daña la integridad | Mucho peor |
| Sin noradrenalina (exploración por sorpresa) | Peor: la sorpresa ayuda |
| Calma: aprender más despacio con la experiencia (`CALMA`) | Peor: congela errores de la infancia |
| Memoria de lugares donde se sintió bien (`UMBRAL_RECUERDO`) | Peor: sin consolidación, recuerda ruido |
| Corteza predictiva como fuente de sorpresa (`CORTEZA`, curiosidad en 0) | Igual: 16 vidas 0,190 vs 0,190; 150k 0,149 vs 0,143 |
| Curiosidad por progreso de aprendizaje (`PESO_CURIOSIDAD` 0,1 / 0,3 / 0,5 / 1 / 2 / 5) | No robusta: con 8 vidas peso 1 parecía ganar (0,170 vs 0,190), con 16 vidas pierde (0,221); peso alto lo distrae y pasa hambre |

La memoria de lugares queda en el código para la etapa 4 (hipocampo + sueño), donde tiene más sentido.

## Etapa 3 — estado (probada, apagada)

`angri/corteza.py` antes de cada paso adivina **qué va a encontrar** y **cuánto le va a cambiar el malestar**,
con lo que ve, la necesidad urgente y la zona del mundo. El error de esa adivinanza es la sorpresa real
(noradrenalina), y la **curiosidad** premia que el error *esté bajando* en un tipo de situación (no el error
en sí, para no engancharse con el ruido de los peligros que se mueven al azar). Se guarda en `estado.json`.

Medido honestamente no mejora: el mundo es tan chico que lo que hay en la casilla de al lado ya lo ve, y lo
poco que queda por predecir es casi ruido; la diferencia entre vidas (desvío ~0,07) tapa cualquier efecto.
Lección: medir con 16 vidas, no 8 (con 8 apareció una mejora falsa). Queda en el código para cuando el mundo
tenga cosas que valga la pena predecir (lugares fuera de la vista, ciclos de comida) o para la etapa 4.

## Etapa 4 — estado (probada, apagada)

**Mundo con estaciones** (queda prendido): cada `ESTACION` = 5000 ticks una fuente de comida se seca y brota
lejos. Lo aprendido de memoria deja de valer; el hambre sube (12 % → 26 % con 2 vidas cortas), así que el
mundo ahora sí tiene algo que olvidar y reaprender.

`angri/hipocampo.py` anota cada momento del día y al dormirse: marca **lugares** donde sintió un gran alivio
(se refuerzan si se repiten, la **poda** los debilita noche a noche), y hace **replay** del día priorizando lo
intenso, reentrenando a los ganglios sin vivirlo. Al despertar borra el día.

Medido con 16 vidas de 50.000 ticks (con estaciones):

| Variante | Malestar | Mal. final | % hambre |
|---|---|---|---|
| Sin hipocampo | **0,190** | **0,154** | **37,9** |
| Solo lugares + poda (`REPASOS=0`) | 0,232 | 0,228 | 47,3 |
| Replay solo del crítico (`REPASO_ACTOR=False`) | 0,213 | 0,184 | 40,6 |
| Replay crítico + actor (`APRENDE_REPASO=0.1`) | 0,280 | 0,246 | 62,0 |

Ninguna mejora. El replay del actor es lo peor: repasa acciones elegidas con otra temperatura y con lo que
sabía antes (aprendizaje *off-policy* sin corrección), y la primera noche lo deja trabado (pesos congelados,
hambre 93 % con el repaso al 0,5). Los lugares recordados lo arrastran a fuentes que ya se secaron.
### Segunda vuelta: replay corregido

Dos arreglos: **corrección por importancia** (cada recuerdo pesa ρ = π_hoy/π_entonces, con tope 1, usando la
temperatura con la que eligió) y **borrar un lugar** al llegar y no ver lo que necesitaba
(`BORRAR_LUGAR_VACIO`). Con 32 vidas de 50.000 ticks:

| Variante | Malestar | Mal. final | % hambre | Heridas |
|---|---|---|---|---|
| Sin hipocampo | 0,206 | 0,165 ± 0,082 | 41,5 | 14 ± 14 |
| Replay corregido (`APRENDE_REPASO=0.5`) | **0,181** | **0,128 ± 0,027** | **25,5** | 175 ± 546 |
| Replay corregido (`APRENDE_REPASO=0.1`) | 0,184 | 0,141 | 28,0 | 70 ± 324 |
| Replay sin corrección | 0,396 | 0,396 | 94,8 | 104 |

La corrección es lo que lo hace funcionar: el sueño ahora **mejora** el final de la vida y el hambre, y las
vidas se parecen más entre sí. Pero 6 de 32 vidas caen temprano en una **trampa de piso**: integridad en 0,
quieto sobre una X, cientos de heridas (semilla 25: 3031). Con la integridad ya en 0 lastimarse no cambia
nada, no hay señal que lo saque, y el repaso refuerza el quedarse. Después lo aprende y sale (malestar final
bajo), pero hasta que se resuelva esa trampa queda **apagado**.

### Tercera vuelta: dolor que no se satura (prendido)

`DOLOR` = 3: pisar una X le resta directo a la recompensa (nocicepción), sin pasar por el malestar, así que
se siente aunque la integridad ya esté en 0 o el malestar en su tope. Con 32 vidas de 50.000 ticks:

| Variante | Malestar | Mal. final | % hambre | Heridas |
|---|---|---|---|---|
| Sin dolor, sin sueño (antes) | 0,206 | 0,165 | 41,5 | 14 ± 14 |
| Dolor, sin sueño | 0,194 | 0,156 | 37,4 | 8 ± 4 |
| **Dolor + sueño (replay corregido)** | **0,179** | **0,142** | **28,9** | **7 ± 3** |
| Dolor ×10 + sueño | 0,184 | 0,150 | 30,2 | 4 ± 1 |

La trampa desaparece y el sueño mejora todo, aunque por poco (≈1–2 desvíos estándar de la media): queda
**prendido**. Falta medir el olvido en sí (cuánto tarda en encontrar una fuente nueva tras cada estación).

## Hoja de ruta

1. ✅ Mundo y cuerpo
2. ✅ Dopamina: actor-crítico que aprende a mantenerse en equilibrio (supera a los reflejos en vidas largas)
3. 🔶 Predicción del mundo y curiosidad por progreso de aprendizaje (hecha y medida; apagada porque no mejora en este mundo)
4. ✅ Hipocampo y sueño real: estaciones, replay corregido, poda de lugares y dolor (mejora modesta; falta medir el olvido tras cada estación)
5. Tálamo y espacio global: módulos que compiten por la atención
6. Lenguaje, con un modelo chico como maestro
7. Mundo real: internet, sensores de la PC como "cuerpo" y vida continua

## Reglas del proyecto

- Nada de scripts que le digan qué hacer: las reglas viven dentro del sistema.
- Cada pieza "cerebral" se mide; si no mejora nada, se saca.
- Tope al malestar (`MALESTAR_MAXIMO`) y registro de su interior, siempre.
