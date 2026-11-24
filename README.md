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
| `angri/maestro.py` | Lenguaje | Etapas 6–7: un maestro le dice con palabras (en español, sin significado de nacimiento) dónde está lo que necesita; el significado se aprende con dopamina |
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
Con `azar` o `reflejos` al final se mide la mente vieja. Para ablaciones: `python medir.py 50000 16 PESO_CURIOSIDAD=0`. Con `-j 12` reparte las vidas en 12 procesos (da idéntico). `python comparar.py 50000 512 "" "PIDE=True" -j 12` compara en pares (misma semilla con A y con B) y da B−A con su error estándar y z. El mundo está calibrado para que sin cerebro **sufra**:
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
**prendido**.

### Olvido tras cada estación

`medir.py` ahora mide, después de cada cambio de estación, **encuentra** (ticks hasta comer en la fuente
nueva; tope 2500 si no la encuentra) y **ronda vieja** (ticks despierto cerca de la fuente que se secó).
32 vidas de 50.000 ticks:

| Variante | Encuentra | Ronda vieja | % hambre |
|---|---|---|---|
| Sin sueño | 1163 ± 465 | 61 ± 45 | 37,4 |
| **Sueño completo** | 1096 ± 427 | 75 ± 42 | **28,9** |
| Solo lugares, sin replay (`REPASOS=0`) | 1213 | 82 | 32,8 |
| Sueño sin borrar lugares vacíos | 1337 | **184** | 44,9 |

- El sueño **no acelera** el reaprendizaje tras un cambio (1096 vs 1163 está dentro del ruido): lo que gana
  viene de consolidar lo que ya sirve, no de olvidar más rápido.
- Lo que evita la perseveración es **borrar al instante un lugar que llega y encuentra vacío**: sin eso
  ronda la fuente seca 3 veces más y es la peor variante. La poda nocturna sola es demasiado lenta.

## Etapa 5 — estado (probada, apagada)

`angri/talamo.py`: cada necesidad es un módulo que compite por el **foco**. Saliencia = urgencia normalizada
× `GANANCIA_URGENCIA` + un sesgo aprendido según si lo que la alivia (o un peligro) está a la vista. Gana uno
por softmax, con **inercia** (el foco actual tiene ventaja, para no picotear), y el ganador se difunde: es la
necesidad con la que los ganglios eligen y aprenden. El tálamo aprende con la misma dopamina.

32 vidas de 50.000 ticks (con sueño y estaciones):

| Variante | Malestar | Mal. final | % hambre | Encuentra |
|---|---|---|---|---|
| Sin tálamo (foco = necesidad más desviada) | 0,179 | **0,142** | 28,9 | **1096** |
| Tálamo (`GANANCIA_URGENCIA=3`, inercia 0,5) | 0,186 | 0,155 | 33,5 | 1229 |
| Tálamo sin inercia | 0,183 | 0,161 | 30,7 | 1255 |
| Tálamo con urgencia ×10 | **0,173** | 0,143 | **24,8** | 1329 |

No mejora de forma clara: cuanto más manda la urgencia, más se parece a no tener tálamo. Lo que aprende tiene
sentido: atender a la **energía cuando no ve comida** (sesgo +1,0) y no cuando la ve (−1,2), porque ir a la
comida visible ya lo hacen los ganglios con cualquier foco. Es decir, el foco sirve para *buscar*, no para
*agarrar*. Pero en este mundo hay una sola cosa urgente a la vez casi siempre (coincide con la más desviada
78–92 % del tiempo), así que no hay mucho por lo que competir. Queda en el código para cuando haya módulos
de verdad distintos (corteza, recuerdos, lenguaje) y no solo necesidades.

## Etapa 6 — estado (prendida)

`angri/maestro.py`: un maestro chico (con reglas, sin librerías) ve todo el mundo; cuando lo que Angri necesita
no está a la vista, con probabilidad `PROB_HABLA` le dice dos palabras: qué es y para dónde queda. Las palabras
son sílabas mezcladas por semilla: Angri no nace sabiendo ninguna. Lo oído suena `MEMORIA_PALABRA` ticks en la
memoria de trabajo y el actor tiene rasgos (palabras oídas, movimiento) que aprende con la misma dopamina.

32 vidas de 50.000 ticks (con sueño y estaciones):

| Variante | Malestar | Mal. final | % hambre | Encuentra |
|---|---|---|---|---|
| Sin maestro (etapa 5) | 0,179 | 0,142 | 28,9 | 1096 |
| Palabras sueltas | 0,217 | 0,182 | 41,7 | 1352 |
| Palabras sueltas, habla ×5 | 0,266 | 0,224 | 58,1 | 1629 |
| Palabras sueltas, maestro que miente | 0,193 | 0,165 | 31,3 | 1122 |
| **Frase entera** | **0,156** | **0,130** | **18,1** | **935** |
| Frase entera + sueltas | 0,180 | 0,142 | 27,9 | 1031 |
| Frase entera, maestro que miente | 0,187 | 0,140 | 28,3 | 1232 |

- Con **palabras sueltas empeora**, y más cuanto más habla: la palabra de la *cosa* ("comida") aparece en cada
  frase, se queda con el crédito y se vuelve una manía de dirección fija (sirve para la fuente de esta estación
  y después estorba), tapando a las palabras de dirección.
- Con **la frase entera como unidad** mejora en todas las métricas (hambre 29 → 18 %), y el control con un
  maestro que miente no mejora: lo que ayuda es el *contenido*, no oír algo.
- Todavía no compone (no sabe que "norte" significa lo mismo en "comida norte" y "cama norte") ni habla él.
  Eso es lo siguiente: separar significado de palabra sin la manía, y producción.

### Etapa 6b — palabras con significado propio, aprendido por sorpresa

`Oido` (en `angri/maestro.py`): al oír una frase, Angri **apuesta** para dónde va a aparecer lo que busca,
combinando lo que cree que significa cada palabra. Cuando lo ve, la diferencia entre la apuesta y lo que pasó
(la **sorpresa**) corrige cada palabra (Rescorla-Wagner). "Comida" aparece con todas las direcciones y queda sin
dirección; "norte" se queda con la suya. Tiene un **piso** de aprendizaje (nunca deja de corregirse) y
**confianza en el maestro**: arranca creyéndole todo, como un chico, y baja si lo que le dice no le sirve.
Los ganglios tienen un rasgo nuevo: "voy adonde me dijeron".

32 vidas de 50.000 ticks:

| Variante | Malestar | % hambre | Entiende | Frase nunca oída | Confianza | Reaprende tras cambio |
|---|---|---|---|---|---|---|
| Sin maestro | 0,179 | 28,9 | — | — | — | — |
| Frase entera (6a) | 0,156 | 18,1 | — | — | — | — |
| Solo léxico | 0,197 | 33,4 | 88 % | **93 %** | 0,85 | — |
| **Léxico + frase entera** | **0,154** | **16,3** | **95 %** | 50 % | 0,86 | — |
| Léxico, maestro que miente | 0,204 | 38,3 | 33 % | — | **0,33** | — |
| Léxico, cambia el sentido a mitad | 0,223 | 42,7 | 80 % | — | 0,78 | **68 %** |
| Ídem con "ego" (aprende cada vez menos) | 0,227 | 46,1 | 80 % | — | 0,69 | 51 % |
| Frase entera, cambia el sentido | 0,182 | 26,6 | — | — | — | — |

- **Entiende y compone:** con el léxico acierta 88–95 % de las veces para dónde está lo que le dijeron, y
  93 % en frases que **nunca había oído** (aprendió "norte" con la comida y lo usa con la cama).
- **El maestro que miente se detecta solo:** la confianza baja de 0,86 a 0,33.
- **El "ego" se paga:** si aprende cada vez menos con la experiencia, tras un cambio de significado reaprende
  51 % contra 68 % con el piso. Se queda con lo viejo.
- **Pero entender no se traduce todavía en vivir mejor:** el léxico solo empeora el malestar, y junto con la
  frase entera queda igual que la frase sola (0,154 vs 0,156, dentro del ruido). Sospecha: sigue una dirección
  dicha desde donde la oyó y, al caminar, se pasa de largo. Queda prendido porque no empeora y es lo que le da
  palabras con significado propio (la base para hablar en español).

### Etapa 6c — ajuste de parámetros (prendido: léxico solo)

Barrido de a un parámetro (16 vidas, léxico solo) y confirmación con 32. Lo que movió la aguja fue aprender
las palabras **más despacio** (`APRENDE_PALABRA` 0,2 → 0,1: una sola experiencia rara no le cambia el
significado) y **esperar más para comprobar** (`ESPERA_VERIFICAR` 30 → 60 ticks: si no lo ve enseguida, igual
cuenta como prueba). Limitar los pasos que sigue una dirección (`PASOS_DICHA`), la certeza para seguirla y cuánto
habla el maestro no cambiaron nada fuera del ruido.

| Variante (32 vidas) | Malestar | Mal. final | % hambre | Encuentra | Entiende |
|---|---|---|---|---|---|
| Sin maestro | 0,179 | 0,142 | 28,9 | 1096 | — |
| Frase entera (6a) | 0,156 | 0,130 | 18,1 | 935 | — |
| **Léxico ajustado** | **0,137** | **0,098** | **12,6** | **708** | 97,5 % |
| Léxico ajustado + frase entera | 0,143 | 0,106 | 13,5 | 878 | 94,6 % |
| Léxico ajustado, maestro que miente | 0,188 | 0,153 | 32,5 | 1154 | 35 % (confianza 0,26) |
| Léxico ajustado, cambia el sentido a mitad | 0,186 | 0,176 | 30,5 | 1201 | 80 % |

- Ahora **entender sí le mejora la vida**: es la mejor versión de todas (hambre 29 → 13 %), y sin la frase
  entera, que queda apagada (`FRASE=False`): con palabras que significan algo por sí solas no hace falta.
- Con un maestro que miente queda casi como sin maestro: la confianza baja y el daño se contiene.

## Hoja de ruta

1. ✅ Mundo y cuerpo
2. ✅ Dopamina: actor-crítico que aprende a mantenerse en equilibrio (supera a los reflejos en vidas largas)
3. 🔶 Predicción del mundo y curiosidad por progreso de aprendizaje (hecha y medida; apagada porque no mejora en este mundo)
4. ✅ Hipocampo y sueño real: estaciones, replay corregido, poda de lugares y dolor (mejora modesta; el sueño consolida pero no acelera el reaprendizaje)
5. 🔶 Tálamo y espacio global: necesidades compiten por el foco (hecho y medido; apagado porque no mejora)
6. 🔶 Lenguaje: entiende frases (mejora) y compone palabras aprendidas por sorpresa (ajustado: mejor versión, hambre 29 → 13 %); ya en español (etapa 7)
7. 🔶 Hablar: Angri pide con una palabra. Por dopamina no aprende qué pedir (34–39 %); **escuchando sí (100 %)**, pero pedir todavía no mejora la vida (etapa 8, apagado)
8. 🔶 Maestro ciego (etapa 9): no sabe qué necesita Angri; pedir no mejora (apagado)
9. ✅ Vidas trabadas (etapa 10): explorar más cuando la pasa mal las saca (malestar 0,179 → 0,144)
10. ✅ Vida continua (etapa 12): guarda la mente entera y retoma bit a bit; vive 1.000.000 de ticks sin degradarse ni crecer en RAM
11. Mundo más rico
12. Mundo real: internet, sensores de la PC como "cuerpo"

## Etapa 6d: cierre del lenguaje (composición, cambio de significado, barrido, memoria)

Mejor versión sin cambios: `LEXICO=True, FRASE=False, APRENDE_PALABRA=0.1, ESPERA_VERIFICAR=60`
→ **malestar 0,137 · hambre 12,6 %** (32 vidas). Nada nuevo la superó; todo lo agregado queda apagado.

**Composición** (`COMPOSICION=t`: hasta el tick t el maestro solo habla de comida, después solo de la cama).
En el tick t se toma un examen sin aprender: las 4 frases "cama + dirección", que nunca oyó.
Control `HOLISTICO=True`: aprende cada frase entera, no puede componer.

| 32 vidas | examen (128 frases nuevas) | aciertos en vida (pocas: 8 y 4 casos) | malestar |
|---|---|---|---|
| léxico, cambio de tema a 25.000 | **97,7 %** | 8/8 | 0,169 |
| holístico, cambio a 25.000 | 25,0 % (azar) | 2/4 | 0,194 |
| léxico, cambio a 5.000 | 81,3 % | 3/4 | 0,182 |
| holístico, cambio a 5.000 | 25,0 % | 3/6 | 0,175 |
| holístico, maestro normal | – | – | 0,188 (vs 0,137) |

Compone: entiende frases que nunca oyó. Y componer importa: sin eso el maestro ayuda mucho menos (0,188).
Los aciertos "en vida" con frases nuevas son muy pocos (la cama casi nunca hace falta lejos de la vista); el examen es la medida confiable.

**Cambio de significado** (`CAMBIO_SIGNIFICADO=25000`, 32 vidas):

| variante | malestar | hambre % | reaprende % (100 apuestas) |
|---|---|---|---|
| sin ajuste | 0,186 | 30,5 | 58,2 |
| `EGO=True` | 0,194 | 32,1 | 27,2 |
| ritmo según sorpresa (`SORPRESA_RITMO=1, RITMO_MAXIMO=0.2`) | 0,199 | 34,9 | 67,4 |
| confianza más sensible (`CONFIANZA_MINIMA=0.5, APRENDE_CONFIANZA=0.05`) | **0,178** | 27,7 | 57,0 |
| ambas | 0,192 | 34,6 | 64,8 |
| ego + confianza sensible | 0,186 | 33,2 | 41,6 |

El ritmo por sorpresa reaprende más rápido (58→67 %) pero vive peor, y sin cambio también empeora (0,151 con 16 vidas):
apura el aprendizaje al principio, cuando todo sorprende. La confianza sensible ayuda tras el cambio (deja de
seguir al maestro mientras no le entiende) pero sin cambio empeora: **0,165 / 23,1 %** (32 vidas). Ninguno se prende.

**Barrido fino** (16 vidas; la base da 0,142 / 13,3 % con esas 16):

| parámetro | valores | malestar |
|---|---|---|
| `PISO_PALABRA` | 0 · 0,5 · 1 | 0,142 (sin efecto: solo actúa con `EGO`) |
| `MEMORIA_PALABRA` | 3 · 12 | 0,142 (sin efecto: solo lo usan `FRASE`/`PALABRA_SUELTA`, apagados) |
| `APRENDE_CONFIANZA` | 0 · 0,003 · 0,03 | 0,142 (la confianza nunca baja de la mínima) |
| `CONFIANZA_INICIAL` | 0,5 · 0,7 | 0,142 |
| `CONFIANZA_MINIMA` | 0 · 0,3 · 0,9 | 0,142 · 0,142 · 0,164 |
| `PROB_HABLA` | 0,05 · 0,07 · 0,15 · 0,2 · 0,4 | 0,207 · 0,193 · 0,153 · 0,154 · 0,169 |
| `EGO` (piso 0,25 / 0) | | 0,158 / 0,157 |

0,1 sigue siendo lo mejor para `PROB_HABLA`: hablarle menos ayuda menos; hablarle más también empeora.

**Memoria**: `guardar`/`_cargar` ahora conservan el léxico, la confianza y las frases oídas del `Oido`,
y el significado secreto del maestro (antes, sin semilla, el maestro cambiaba de idioma en cada arranque).
El panel muestra la confianza en el maestro.

## Etapa 7: palabras en español y Angri pide (probada; pedir queda apagado)

**Palabras en español.** El maestro ahora dice "comida", "cama", "novedad", "norte", "sur", "este", "oeste"
(y "che", que nunca usa). Es solo la etiqueta: el índice de cada palabra sigue mezclado por semilla y Angri
nace sin saber qué significa ninguna. Mismo sorteo que antes, así que da idéntico (32 vidas):

| 32 vidas | malestar | hambre % | entiende % | examen de composición |
|---|---|---|---|---|
| sílabas (etapa 6d) | 0,137 | 12,6 | – | 97,7 % |
| **español** | **0,137** | **12,6** | 97,5 | **97,7 %** (`COMPOSICION=25000`, malestar 0,169 como antes) |

**Angri pide** (`PIDE=True`, en `angri/maestro.py` clase `Habla`). Cuando lo que necesita no está a la vista,
con la misma probabilidad con que habla el maestro (0,1) elige entre decir "comida", "cama", "novedad" o
callarse. Nace eligiendo al azar; un actor con un peso por (necesidad × palabra) aprende con la dopamina de
los ganglios sumada en los `VENTANA_HABLA` ticks siguientes. El maestro contesta por lo que le piden, aunque
no sea lo que Angri más necesita. `MAESTRO_SOLO_SI_PIDE=True`: el maestro solo habla si le piden.
`PIDE_AZAR=True`: control, pide al azar y no aprende. "Pide bien" = energía→comida, cansancio→cama,
curiosidad→novedad (último 20 % de la vida; azar = 33 %).

| 32 vidas | malestar | hambre % | pide bien % |
|---|---|---|---|
| actual (no pide, maestro habla solo) | **0,137** | **12,6** | – |
| pide + maestro habla solo | 0,181 | 27,9 | 33,9 |
| pide al azar + maestro habla solo | 0,187 | 30,4 | 33,3 |
| pide, maestro solo si pide | 0,179 | 28,3 | 39,1 |
| pide al azar, maestro solo si pide | 0,187 | 33,5 | 32,8 |

Ajustes (16 vidas, maestro solo si pide):

| variante | malestar | hambre % | pide bien % |
|---|---|---|---|
| `APRENDE_HABLA=0.2` | 0,167 | 25,0 | 48,3 (desvío 25) |
| `APRENDE_HABLA=0.2, VENTANA_HABLA=100` | 0,224 | 44,6 | 28,8 |
| `APRENDE_HABLA=1, VENTANA_HABLA=100, TEMPERATURA_HABLA=0.5` | 0,200 | 37,2 | 25,7 |

- **No funcionó**: aprender qué pedir solo con dopamina casi no supera al azar (34–39 % vs 33 %; el mejor
  ajuste llega a 48 % con mucha variación). La dopamina de los ticks siguientes mezcla la respuesta del
  maestro con todo lo demás que le pasa: el crédito es demasiado ruidoso.
- Pedir **empeora** frente a la versión actual (0,137 → 0,18): cuando pide mal, el maestro lo manda a buscar
  lo que no necesita, y gasta el turno en que el maestro igual le habría dicho lo correcto.
- Que pida y aprenda (39 %) le gana un poco al control al azar (33 %) y vive apenas mejor (0,179 vs 0,187),
  así que algo aprende, pero no alcanza. Todo queda apagado (`PIDE=False`).

El panel muestra lo que dice Angri (`dice:`) y lo que oye (`oye:`). `guardar`/`cargar` conservan los pesos del habla.

Config final (sin cambios de comportamiento): `MAESTRO=True, LEXICO=True, PROB_HABLA=0.1, PIDE=False,
MAESTRO_SOLO_SI_PIDE=False` → malestar 0,137, hambre 12,6 %.

## Etapa 8: aprende a pedir escuchando (pide bien 100 %; pedir sigue apagado)

Comprensión antes que producción. `Habla.escuchar` (en `angri/maestro.py`): cada vez que el maestro dice algo,
Angri asocia lo que siente con las palabras de cosa que oyó, por sorpresa (Rescorla-Wagner, igual que el
léxico): `assoc[necesidad][palabra] += APRENDE_ESCUCHA · x · (oyó − esperaba)`. Al pedir elige con softmax
(`TEMPERATURA_ESCUCHA`) la palabra más asociada a lo que siente; si ninguna pasa `UMBRAL_ESCUCHA`, se calla.
Nadie le dice qué palabra va con qué necesidad: la correlación sale de que el maestro habla de lo que le hace
falta. "Lo que siente": `ESCUCHA_CUERPO=False` = el foco (necesidad urgente); `True` = el déficit de cada
necesidad del cuerpo. `ESCUCHA_PEDIDOS`: también aprende de lo que le contestan cuando pidió.
Arranque con `MAESTRO_SOLO_SI_PIDE=True`: `MAESTRO_INICIO` (ticks del principio en que habla solo) o
`PROB_ESPONTANEO` (fracción de las veces que igual habla por su cuenta). La dopamina ya no elige la palabra.

**Corrección de medida:** `Habla` usaba el rng de la mente, así que pedir cambiaba toda la vida. Ahora tiene
rng propio. Ese solo cambio movió el "pide por dopamina" de 0,181 a 0,148: **el ruido entre corridas de 32
vidas es ~0,03** (desvío por vida ~0,07). Las tablas de la etapa 7 están afectadas por eso.

Tanda 1 (rng compartido, 32 vidas):

| variante | malestar | hambre % | pide bien % |
|---|---|---|---|
| actual, no pide | **0,137** | **12,6** | – |
| dopamina / azar (maestro habla solo) | 0,181 / 0,187 | 27,9 / 30,4 | 33,9 / 33,3 |
| escucha, cuerpo | 0,184 | 31,3 | 90,5 |
| escucha, foco | 0,155 | 20,2 | 100,0 |
| solo si pide: dopamina / azar | 0,179 / 0,187 | 28,3 / 33,5 | 39,1 / 32,8 |
| solo si pide, escucha, sin arranque | 0,181 | 28,1 | – (nunca oye, nunca pide) |
| solo si pide, escucha, `MAESTRO_INICIO=10000` | 0,176 | 27,5 | 62,1 |
| ídem + `ESCUCHA_PEDIDOS` | 0,179 | 29,5 | 55,0 |
| solo si pide, escucha, `PROB_ESPONTANEO=0.2` | 0,194 | 34,9 | 85,4 |

Tanda 2 (rng propio, 32 vidas):

| variante | malestar | hambre % | pide bien % |
|---|---|---|---|
| dopamina / azar (maestro habla solo) | 0,148 / 0,161 | 18,6 / 21,4 | 33,8 / 34,2 |
| escucha, cuerpo / foco | 0,144 / 0,154 | 17,2 / 18,1 | 89,6 / 100,0 |
| solo si pide: dopamina / azar | 0,181 / 0,173 | 29,5 / 29,3 | 36,4 / 33,8 |
| solo si pide, escucha foco, sin arranque | 0,204 | 36,3 | – |
| … `MAESTRO_INICIO=10000` (± `ESCUCHA_PEDIDOS`) | 0,160 / 0,159 | 20,2 / 20,1 | 99,8 / 100,0 |
| … `PROB_ESPONTANEO=0.2` | 0,149 | 18,0 | 100,0 |

Barrido (16 vidas, solo si pide + espontáneo 0,2, foco; referencia 0,161, no pide 0,142):
`PROB_ESPONTANEO` 0,05 → 0,193 · 0,5 → 0,178; `APRENDE_ESCUCHA` 0,03 → 0,174 · **0,3 → 0,145**;
`UMBRAL_ESCUCHA` 0,05 → 0,150 · 0,5 → 0,185; `TEMPERATURA_ESCUCHA=1` → 0,155 (pide bien 57 %).

Confirmación (32 vidas, `APRENDE_ESCUCHA=0.3`): solo si pide + espontáneo 0,2 → **0,142 · hambre 14,8 % ·
pide bien 100 %**; maestro habla solo → 0,156 · 19,6 % · 100 %. (`UMBRAL_ESCUCHA=0.05` da idéntico.)

- **Funcionó**: aprender a pedir escuchando. Pide bien 100 % (foco) / 90 % (cuerpo) contra 34–39 % por
  dopamina y 33 % al azar. Con el estado del cuerpo rinde menos porque estrés e integridad diluyen la señal.
- Con el maestro que solo contesta, pedir bien también **vive mejor que pedir por dopamina o al azar**
  (0,142–0,149 vs 0,173–0,181), y casi igual que la versión actual.
- **No funcionó**: ninguna variante supera a no pedir (0,137 · 12,6 %); la diferencia está dentro del ruido
  (~0,03). Sin arranque (`MAESTRO_SOLO_SI_PIDE` puro) nunca oye nada y nunca pide (0,18–0,20). Aprender de las
  respuestas a sus propios pedidos no aporta. Pedir no agrega información: el maestro ya hablaba de lo que
  Angri necesitaba.
- Queda `PIDE=False`. Cuando se prende, `Habla` usa la escucha por defecto (`ESCUCHA=True, ESCUCHA_CUERPO=False,
  APRENDE_ESCUCHA=0.3, PROB_ESPONTANEO=0.2`): le gana claramente a la dopamina. Para que pedir sirva, hace
  falta un maestro que **no** sepa lo que Angri necesita (que solo se entere si le piden).

Config final: igual que la etapa 7 (`PIDE=False`) → malestar 0,137, hambre 12,6 %.

## Etapa 9: maestro ciego, medición corregida y velocidad

**Error de medición grande.** Con 128 vidas, la versión actual da **malestar 0,179 · hambre 28 %**, no 0,137 ·
12,6 %. Las vidas son bimodales (algunas quedan trabadas con hambre; mediana 0,153) y las semillas 0–31 tenían
suerte. Con 32 vidas el error estándar es ~0,013 y el ruido entre corridas llega a ~0,04: **los resultados de las
etapas 6–8 con 32 vidas tienen que leerse con eso en mente**. Cambios desde acá:
- `medir.py` muestra error estándar (e.e.) y mediana; `-j N` reparte las vidas en N procesos.
- `comparar.py`: comparación en pares con la misma semilla, B−A ± e.e. y z (|z| > 2 = diferencia real). Las vidas
  divergen rápido, así que el emparejamiento ayuda poco: lo que manda es la cantidad de vidas (512 → e.e. ~0,005).
- El maestro hace siempre los mismos sorteos por tick (pedir ya no cambia el resto de la vida), y `Habla` tiene
  rng propio (etapa 8).

**Velocidad (resultados idénticos, cifra por cifra):** el actor solo actualiza los rasgos que alguna vez usó
(antes recorría los 422 en cada tick, ~320 nunca se usan), el repaso del sueño también, y `_ritmo` se guarda
en caché. Una vida de 50.000 ticks pasa de ~13 s a ~5 s; con `-j 12`, 128 vidas tardan ~1 min.
PyPy (probado, 3.12 v8.0.1) da solo 1,4× más, y no reproduce las cifras de CPython (suma de decimales distinta):
CPython sigue siendo la referencia.

**Maestro ciego** (`MAESTRO_CIEGO=True`): cuando habla por su cuenta, nombra una cosa al azar; no sabe qué necesita
Angri. Solo pidiendo se entera. Como lo que oye ya no está correlacionado con lo que siente, se agregó aprender
**por alivio** (`ESCUCHA_ALIVIO`): si oyó una palabra y en los `VENTANA_ALIVIO` ticks siguientes se le alivia una
necesidad (baja del déficit > `UMBRAL_ALIVIO`; dormir alivia de a 0,01 por tick, de ahí el umbral 0,005), asocia
esa necesidad con esa palabra.

128 pares contra la base (malestar 0,179; negativo = mejor):

| B | malestar | B−A | z | pide bien % |
|---|---|---|---|---|
| sabe, contesta solo si pide, escucha | 0,160 | −0,019 | −2,1 | 100 |
| sabe, habla solo, escucha | 0,162 | −0,017 | −1,9 | 100 |
| sabe, habla solo, dopamina | 0,161 | −0,018 | −2,0 | 37 |
| sabe, contesta solo si pide, dopamina | 0,163 | −0,016 | −1,9 | 37 |
| sabe, contesta solo si pide, al azar | 0,189 | +0,010 | +1,1 | 33 |
| ciego, no pide | 0,185 | +0,006 | +0,6 | – |
| ciego, contesta solo si pide, alivio (umbral 0,005 / 0,1 / ventana 200) | 0,182 / 0,185 / 0,184 | ≈ +0,005 | ≈ +0,5 | 41 |
| ciego, contesta solo si pide, al azar | 0,183 | +0,004 | +0,5 | 33 |
| ciego, habla solo, alivio | 0,164 | −0,015 | −1,7 | 33 |

512 pares (el control que decide):

| B | malestar B−A | z | hambre B−A | z |
|---|---|---|---|---|
| **maestro habla el doble** (`PROB_HABLA=0.2`), no pide | −0,010 | −2,1 | −4,4 | −2,5 |
| pide escuchando, contesta solo si pide | −0,008 | −1,9 | −2,5 | −1,5 |
| pide por dopamina, maestro habla solo | −0,013 | −2,8 | −4,2 | −2,6 |

- **Lo que ayuda es la cantidad de palabras, no pedir bien.** Pedir por dopamina (37 % de aciertos) mejora lo
  mismo que pedir escuchando (100 %), y que el maestro hable el doble sin que Angri pida mejora igual. Pedir no
  agrega nada propio: las diferencias entre las tres están dentro del ruido.
- **El maestro ciego no funcionó:** aprender por alivio llega a 41 % (azar 33 %), porque lo que se le alivia
  después de oír depende de mucho más que de la palabra; pedirle a un maestro ciego no mejora nada.
- Nada se prende: las mejoras son ~2σ y ninguna es de pedir. Que el maestro hable más es un cambio del mundo,
  no del cerebro; queda documentado (`PROB_HABLA=0.2`: −0,010).

Config final sin cambios de comportamiento: `PIDE=False, MAESTRO_CIEGO=False, PROB_HABLA=0.1` →
**malestar 0,179 · hambre 28 % (128 vidas)**.

## Etapa 10: vidas trabadas (prendido: malestar 0,179 → 0,144)

**El problema.** Con 128 vidas, el 17 % pasa más del 60 % del tiempo con hambre (mediana del malestar 0,153,
promedio 0,179: esas vidas arrastran el promedio). Nueva métrica en `medir.py`: `% trabadas`.

**Qué les pasa** (mirando semillas una por una):
- No es mala suerte del arranque: de las 22 trabadas, 13 comieron en los primeros 150 ticks. Solo 2 no comieron
  nunca y quedaron quietas (la energía llega a 0 en unos 350 ticks si no encuentra comida).
- Comen al principio y después pasan **decenas de miles de ticks sin comer**, aunque se mueven: quedan
  **encerradas en un rincón** (las 4 columnas del borde, la cama) y **nunca ven comida** (0 veces en 15.000
  ticks). En 10.000 ticks visitan 26–54 de 336 celdas, cuando una caminata al azar recorrería todo.
- Saben ir a la comida cuando la ven (peso +1,2), pero en 14 de las 22 trabadas el peso de "voy adonde me dijeron"
  es **negativo** (en las sanas, positivo), y lo mismo con "voy adonde una vez me sentí bien". Aprendieron a
  alejarse de lo que las llevaría a la comida. Con la energía en el piso el malestar casi no cambia: no hay señal
  para desaprenderlo, y con poca exploración nunca salen de ahí.

**Arreglos probados** (128 pares contra la base; negativo = mejor):

| cambio | malestar B−A | z | hambre B−A | trabadas % |
|---|---|---|---|---|
| `FRANJAS=8` (crítico ve la energía más fino) | −0,008 | −0,9 | −4,2 | 17 → 8,6 |
| `FRANJAS=16` | +0,005 | +0,5 | +1,6 | 17 → 20 |
| `HUELLA=0.9` (crédito más largo) | +0,010 | +1,0 | +1,8 | 17 → 16 |
| `HUELLA=0.95` | **+0,063** | +5,2 | +19,4 | 17 → 39 |
| `TEMPERATURA_MALESTAR=3` | −0,025 | −2,7 | −9,8 | 17 → 8,6 |
| `TEMPERATURA_MALESTAR=4` | −0,033 | −3,9 | −12,7 | 17 → 5,5 |
| `TEMPERATURA_MALESTAR=4, FRANJAS=8` | −0,024 | −2,7 | −9,3 | 17 → 6,3 |
| `TEMPERATURA_MALESTAR=4, TEMPERATURA_BASE=0.2` | −0,021 | −2,5 | −7,9 | 17 → 10,9 |
| `TEMPERATURA_MALESTAR=6` | −0,029 | −3,4 | −10,8 | 17 → 6,3 |
| **`TEMPERATURA_MALESTAR=8`** | **−0,040** | **−5,3** | **−16,5** | **17 → 1,6** |
| `TEMPERATURA_MALESTAR=12` | −0,037 | −4,9 | −15,7 | 17 → 3,9 (heridas 6,5 → 11,3) |
| `TEMPERATURA_MALESTAR=16` | −0,038 | −4,8 | −18,2 | 17 → 2,3 (heridas → 13,5) |
| `TEMPERATURA_MALESTAR=24` | −0,035 | −4,4 | −17,1 | 17 → 1,6 (heridas → 17,3) |

Confirmación, **512 pares, `TEMPERATURA_MALESTAR=8`**: malestar 0,179 → **0,144** (−0,035 ± 0,004, z −8,7);
malestar final 0,132 → 0,109; hambre 27,4 → **13,3 %**; trabadas 15,6 → **3,9 %**; comidas +19 %; encuentra
la fuente nueva 22 % antes. Costo: se lastima más (heridas 6,7 → 10,0 por vida).

- **Funcionó:** cuando la pasa mal, explorar mucho más. Es lo que ya decía la idea original ("si lo de siempre no
  funciona, probar otra cosa", antídoto contra la indefensión aprendida), pero con 2 era demasiado poco para
  sacarla del rincón. Arriba de 8 ya no mejora y se lastima más.
- **No funcionó:** que el crítico vea más fino (8 o 16 franjas) ni alargar la huella del crédito (0,95 empeora
  mucho: el crédito se reparte entre demasiados pasos).
- Queda prendido `TEMPERATURA_MALESTAR=8`. Nueva base: **malestar 0,144 · hambre 13,3 % (512 vidas)**.

## Etapa 11: chequeo general y heridas (prendido: malestar 0,144 → 0,122)

Antes de seguir, re-medí todo con la base nueva (256 pares de 50.000 ticks, en pares). Un agente revisó el código para ver de dónde salían las heridas.

**Re-medición (vs. base de la etapa 10):**

| pieza | Δ malestar | z | qué pasa |
|---|---|---|---|
| pedir (escucha) | −0,005 | −1,1 | dentro del ruido |
| pedir (dopamina) | −0,005 | −1,2 | dentro del ruido |
| maestro habla el doble | −0,006 | −1,3 | dentro del ruido |
| sin maestro | +0,011 | +3,2 | el maestro sí sirve |
| recordar lugares | −0,008 | −2,1 | candidato |
| corteza | +0,022 | +4,2 | empeora: sigue apagada |
| tálamo | +0,002 | +0,4 | nada |
| pedir + escuchar el cuerpo | −0,010 | −2,6 | candidato |
| pedir + escuchar pedidos | −0,005 | −1,2 | nada |
| DOLOR 6 | −0,003 | −0,7 | heridas 9,9 → 7,2 (z −11,5) |
| DOLOR 12 | +0,004 | +0,8 | heridas 5,4 pero más trabadas |
| menos exploración (base o sorpresa) | ≈ −0,004 | ≈ −1 | nada |

**Por qué se lastimaba más desde la etapa 10.** Ya sabía que la X duele: el peso de "voy a una X" estaba entre −3,5 y −6. Pero la temperatura por malestar aplana el softmax para todas las acciones. Cuando la pasa mal, pisar una X que está viendo tenía probabilidad 0,09–0,13, contra menos de 0,01 con la temperatura base. No le faltaba aprender: la desesperación borraba lo aprendido.

**Arreglo: `PELIGRO_SIN_MALESTAR`.** Moverse sobre un peligro visible usa solo la temperatura base + sorpresa. El resto de las acciones sigue explorando con la desesperación. Es como un miedo innato: el malestar empuja a probar cosas nuevas, pero no a meterse en el fuego.

| 512 pares | antes | ahora | z |
|---|---|---|---|
| malestar | 0,144 | **0,122** | −8,8 |
| heridas | 10,0 | 8,3 | −8,6 |
| % hambre | 13,3 | **7,2** | −7,1 |
| % trabadas | 3,9 | 0,2 | −4,2 |
| comidas | 591 | 638 | +4,8 |

Mejora el hambre, no solo las heridas: lastimarse sube el estrés y el malestar, y eso sube la temperatura. Era un círculo vicioso.

**Encima de eso, `DOLOR` 3 → 6** (512 pares): heridas 8,3 → **6,8** (z −12,7), malestar igual (z −0,1), hambre 7,2 → 8,0 (z +1,3, dentro del ruido).

**Probado y apagado:**
- `MARGEN_EXPLORAR`: no aplicar la temperatura alta a las acciones que considera claramente peores. Es un desastre: traba entre el 73 y el 88 % de las vidas. Salir de un rincón requiere justamente probar lo que cree malo.
- Con el peligro arreglado, ya no suman nada: recordar lugares, escuchar el cuerpo, que el maestro hable el doble, y bajar o subir TEMPERATURA_MALESTAR a 4 o a 12. Todo da |z| < 2, salvo 4, que empeora (z +2,7).

Nueva base: **malestar 0,121, hambre 8 %, heridas 6,8, trabadas 0,4 %**.

## Etapa 12: vida continua (prendida)

Angri ya no vive vidas de 50.000 ticks que se pierden: vive sin fin en la PC.

    python vivir.py                 # vive para siempre, guarda en datos/mente.json, log en datos/vida.csv
    python vivir.py --panel         # lo mismo, mirándolo
    python vivir.py --ticks 100000  # vive 100.000 ticks más y para
    python probar_retomar.py 20000 0

**Memoria completa (`angri/memoria.py`).** En vez de guardar a mano un resumen (antes se perdían las huellas, el hipocampo, el mundo y el random), recorre todo el grafo de objetos de Angri y lo escribe a JSON legible: tuplas, sets, diccionarios con claves raras, generadores al azar y **referencias compartidas** (ganglios, hipocampo y tálamo usan el mismo `random`; si al cargar fueran dos, la vida divergiría). Guardado atómico: escribe a `.tmp`, `fsync` y renombra. Cada 50.000 ticks y al cortar con Ctrl+C. Pesa ~360 KB.

**Retomar es bit-idéntico.** `probar_retomar.py` corre N ticks de un tirón contra N/2 + guardar + cargar + N/2 y compara la mente entera serializada: idéntica con 20.000 (semillas 0 y 1) y 33.333 ticks (semilla 7), y también guardando en un proceso y cargando en otro.

**Vida larga: 1.000.000 de ticks** (semilla 0, un hilo del i5, ~1 min 55 s):

| ticks | malestar | % hambre | heridas | comidas | ticks/seg | RAM MB | `oido.aciertos` |
|---|---|---|---|---|---|---|---|
| 100k | 0,091 | 0,95 | 7 | 1672 | 9962 | 24,8 | 1192 |
| 300k | 0,075 | 0 | 0 | 1881 | 9125 | 26,4 | 4410 |
| 500k | 0,103 | 0,61 | 3 | 1503 | 8915 | 26,6 | 6664 |
| 700k | 0,075 | 0,03 | 0 | 1795 | 8903 | 26,6 | 9245 |
| 1M | 0,072 | 0,01 | 0 | 1949 | 8906 | 27,6 | 13360 |

(ventanas de 100.000 ticks)

- **El aprendizaje no se degrada**: el malestar baja de 0,091 a ~0,073 y queda; los repuntes (500k) son estaciones duras y se recupera solo.
- **Velocidad estable**: ~8.900 ticks/seg desde los 200k (el primer tramo es 10 % más rápido).
- **Fuga encontrada y arreglada**: la caché `_RITMOS` de los ganglios guardaba un renglón por cada valor de experiencia (≈1 por tick): la RAM subía de 27 a 177 MB en 1M ticks. Ahora solo cachea experiencias < 20.000 y calcula directo las otras: **mismo número exacto** (malestar y pesos idénticos a 200k ticks), RAM plana en ~27 MB. No es un tope a la mente: es una caché.
- **Lo único que crece sin tope**: `oido.aciertos`, el registro de apuestas del léxico que usa `medir.py` para el % de acierto: ~13.400 tuplas por millón de ticks (≈1 MB de RAM por millón). Decidido: se deja crecer, sin tope.

### Etapa 12b: vivir en Linux

Para que viva sola en un i5 con Linux sin entorno gráfico, manejada por SSH. Solo hace falta `python3` (nada de Windows; probado en Linux: `probar_retomar.py` IDÉNTICO y `kill -TERM` guarda y retoma).

**Instalar** (como usuario normal, no root):

    git clone <repo> ~/Angri          # o copiar la carpeta (con datos/ si ya tiene mente)
    mkdir -p ~/.config/systemd/user
    cp ~/Angri/deploy/angri.service ~/.config/systemd/user/
    systemctl --user daemon-reload
    systemctl --user enable --now angri
    sudo loginctl enable-linger $USER  # que siga viviendo sin sesión abierta y arranque al prender

Si el repo no está en `~/Angri`, cambiar `WorkingDirectory` en el `.service`. Como servicio de sistema: copiarlo a `/etc/systemd/system/`, poner la ruta absoluta en `WorkingDirectory`, descomentar `User=` con el usuario normal, `WantedBy=multi-user.target`, y `sudo systemctl enable --now angri` (sin `--user` ni linger).

**Mirarla por SSH:**

    cd ~/Angri && python3 vivir.py --estado   # edad, últimas 10 ventanas, ticks/seg, RAM, último guardado, respaldos
    systemctl --user status angri
    journalctl --user -u angri -f             # el log en vivo (avisos incluidos)
    systemctl --user restart angri            # tras un git pull

- **Parar es seguro**: systemd manda SIGTERM; `vivir.py` termina el tick en curso (no corta a mitad: la mente quedaría a medias), guarda y sale. Un segundo SIGTERM durante el guardado se ignora. `Restart=always` la levanta si se cae.
- **Respaldos** en `datos/respaldos/`: cada guardado (cada 50.000 ticks y al salir) copia `mente.json` como `mente-<edad>.json` (quedan las últimas 5) y `dia-<fecha>.json` (una por día, 30 días). Si `mente.json` no carga, retoma de la copia válida más reciente y lo avisa; si ninguna carga, no hace nacer otra encima: sale con error.
- **Cambios de código**: al cargar, cada objeto se compara con una mente de referencia nueva; los atributos que la clase tiene ahora y el guardado no trae se crean con su valor por defecto, y los que sobran se dejan; todo con `AVISO:` en el log. Si no falta nada la mente no se toca (sigue bit-idéntica).
- **Config**: `mente.json` guarda los valores en mayúscula de `angri/config.py`; al retomar avisa cuáles cambiaron (`AVISO: config X cambió: a -> b`).
- **RAM** en el log: en Linux `VmRSS` de `/proc/self/status` (la actual, no el pico).

## Reglas del proyecto

- Nada de scripts que le digan qué hacer: las reglas viven dentro del sistema.
- Cada pieza "cerebral" se mide; si no mejora nada, se saca.
- Tope al malestar (`MALESTAR_MAXIMO`) y registro de su interior, siempre.
