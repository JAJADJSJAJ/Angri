# Angri — un cerebro artificial que vive solo

Guía completa de investigación y diseño: https://claude.ai/artifact/2q9F7WwrnzrdKVjJkUAwjg

## Cómo correrlo

Todo en Rust (`rust/`), sin dependencias. Hace falta `cargo` (Rust 1.85 o más nuevo).

```
cd rust
cargo build --release                       # el binario queda en rust/target/release/angri
cargo test --release                        # pruebas (unos 7 s)
target/release/angri vivir --panel          # vive para siempre, con panel en la terminal
target/release/angri vivir --ticks 20000    # vive 20.000 ticks sin panel
target/release/angri visor                  # http://localhost:8080, la mira desde el navegador
target/release/angri medir 50000 8 -j 8     # vara de medir
```

`angri vivir` guarda la mente en `datos-rust/` (o `--carpeta DIR`, o `ANGRI_DATOS`) y la
próxima vez sigue desde ahí; con Ctrl+C termina el tick, guarda y sale. Para que
"renazca" desde cero, borrá la carpeta. Sin argumentos, `angri` muestra la ayuda.

**Historia:** Angri se escribió primero en Python (`angri/*.py`, `vivir.py`,
`medir.py`...), se pasó entera a Rust dando la misma vida bit a bit, y Python se
borró (paso 3; está en la historia de git hasta el commit `f3ce27d`). Las
etapas de abajo cuentan los comandos de entonces: `python medir.py ...` es hoy
`angri medir ...`, `python comparar.py ...` es `angri comparar ...`,
`python vivir.py` es `angri vivir`, `python visor.py` es `angri visor`, y los
ajustes `CLAVE=VALOR` son los mismos (sin las comillas de Python:
`MUNDO=bosque`). Los dicts de la config se ajustan como `PESO_NECESIDAD.curiosidad=0`.

## Estructura

| Archivo | Región | Qué hace |
|---|---|---|
| `rust/config.txt` | — | Todos los números que se pueden afinar (va metido en el binario; `--config ARCHIVO` usa otro) |
| `rust/src/config.rs` | — | Lee la config y los ajustes `CLAVE=VALOR` |
| `rust/src/azar.rs` | — | Azar propio: PCG32 sembrado con splitmix64 |
| `rust/src/mente.rs` | Hipotálamo, neuromoduladores, ganglios basales | Cuerpo y malestar; química (adenosina, cortisol, dopamina, noradrenalina); actor-crítico que aprende con dopamina |
| `rust/src/extra.rs` | Corteza, hipocampo, tálamo, mente vieja | Predicción (apagada), repaso al dormir, foco (apagado), azar/reflejos como vara |
| `rust/src/lenguaje.rs` | Lenguaje | Maestro, oído (léxico por sorpresa) y habla (pedir) |
| `rust/src/mundo.rs` | El entorno | Grilla 24×14 y bosque 64×64 (zonas, día/noche, estaciones, lluvia, río, comida que cuesta, agua, predador) |
| `rust/src/vida.rs` | El bucle | El tick: se duerme y se despierta solo por adenosina |
| `rust/src/guardar.rs` | — | Guardar/cargar la mente entera como texto (floats con sus bits) |
| `rust/src/vivir.rs` | — | Vida continua, respaldos, panel, foto, `--estado` |
| `rust/src/foto.rs` | — | Foto para el visor y panel de la terminal |
| `rust/src/visor.rs`, `visor.html` | — | Visor web (HTTP mínimo; el pixel art lo dibuja el navegador) |
| `rust/src/medir.rs` | — | `angri medir` y `angri comparar` |
| `rust/src/main.rs` | — | Punto de entrada y modos de una vida (`vida`, `traza`, `foto`, `velocidad`, `retomar`) |
| `rust/src/pruebas.rs`, `rust/pruebas/` | — | `cargo test`: azar, determinismo, retomar, vida de referencia |
| `deploy/` | — | Instalación en el i5 |

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

Para que viva sola en un i5 con Linux sin entorno gráfico, manejada por SSH. (Desde el paso 3 corre el binario de Rust: ver "Angri en Rust" más abajo; lo que sigue se escribió para la versión en Python.)

**Dónde vive**: el i5-4570 (Debian 13, 4 núcleos sin hyperthreading, 16 GB, sin pantalla; `angri.local`). Reemplaza al proyecto viejo de esa PC (llama.cpp/Qwen) y tiene toda la máquina: sin `Nice` ni topes de CPU/RAM. Python puro usa 1 núcleo; los otros 3 quedan libres para mediciones (`comparar.py -j 3`) o etapas futuras.

| qué | dónde |
|---|---|
| usuario de sistema (sin sudo) | `angri` |
| programa | `/opt/angri/angri` (root, solo lectura; antes `/opt/angri/codigo` en Python) |
| mente, respaldos y log | `/var/lib/angri` (`ANGRI_DATOS`; `--carpeta` lo pisa) |
| servicio | `/etc/systemd/system/angri.service` (de sistema: no hace falta linger) |
| comando | `/usr/local/bin/angri [estado|log [N]|vivo|parar|seguir|reiniciar]` |

El servicio corre `/opt/angri/angri vivir` con `Restart=always`, `TimeoutStopSec=120` y la misma jaula que usaba la PC (`NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome`, `PrivateTmp`, solo escribe en `/var/lib/angri`).

**Mudanza** (una vez, como `fran` por SSH):

    # desde esta PC: copiar el repo a ~/despliegue/Angri en el i5
    scp -r Angri fran@angri.local:despliegue/
    ssh fran@angri.local
    sh ~/despliegue/Angri/deploy/desinstalar-viejo.sh   # borra el proyecto viejo entero
    sh ~/despliegue/Angri/deploy/instalar.sh            # instala y la hace nacer

`desinstalar-viejo.sh` no guarda nada viejo: borra las units `angri-ciclo/angri-sueno` (servicio y timer), `angri-web`, `llama-charla/emb/dia` (y sus drop-ins), `/opt/angri` entero, el contenido de `/var/lib/angri`, `/usr/local/bin/angri`, `/etc/ld.so.conf.d/angri-llama.conf` (+ `ldconfig`), `~/llama.cpp` y `~/modelos`, e imprime lo que borró. **No toca** el usuario `angri` (se reutiliza), las actualizaciones de seguridad (unattended-upgrades y el reinicio de las 05:30: por eso el guardado en SIGTERM es imprescindible) ni la regla udev del wifi Mercusys.

**Actualizar** tras cambiar el código: copiar de nuevo a `~/despliegue/Angri` y correr `instalar.sh` (para el servicio, así guarda; recopia el código, sin `__pycache__`; la mente queda y retoma, con avisos si cambiaron clases o config).

**Mirarla por SSH:**

    angri                      # = angri estado: edad, últimas 10 ventanas, ticks/seg, RAM, último guardado, respaldos
    angri log 50               # journalctl -u angri (avisos incluidos)
    angri vivo                 # el log en vivo
    angri parar / angri seguir

Para probarla en otra PC sin instalar nada: `python3 vivir.py` (usa `datos/` del repo).

- **Parar es seguro**: systemd manda SIGTERM; `vivir.py` termina el tick en curso (no corta a mitad: la mente quedaría a medias), guarda y sale. Un segundo SIGTERM durante el guardado se ignora. `Restart=always` la levanta si se cae.
- **Respaldos** en `datos/respaldos/`: cada guardado (cada 50.000 ticks y al salir) copia `mente.json` como `mente-<edad>.json` (quedan las últimas 5) y `dia-<fecha>.json` (una por día, 30 días). Si `mente.json` no carga, retoma de la copia válida más reciente y lo avisa; si ninguna carga, no hace nacer otra encima: sale con error.
- **Cambios de código**: al cargar, cada objeto se compara con una mente de referencia nueva; los atributos que la clase tiene ahora y el guardado no trae se crean con su valor por defecto, y los que sobran se dejan; todo con `AVISO:` en el log. Si no falta nada la mente no se toca (sigue bit-idéntica).
- **Config**: `mente.json` guarda los valores en mayúscula de `angri/config.py`; al retomar avisa cuáles cambiaron (`AVISO: config X cambió: a -> b`).
- **RAM** en el log: en Linux `VmRSS` de `/proc/self/status` (la actual, no el pico).

## Etapa 13a: el bosque (probado; apagado: `MUNDO = "grilla"`)

La grilla de 24×14 se resolvía a los ~200.000 ticks (malestar ~0,073, hambre 0 %). Para que vivir tenga algo que aprender hay un mundo nuevo, configurable (`MUNDO = "bosque"`); el viejo queda igual, bit a bit, para comparar.

- **Mapa 64×64** con zonas: bosque, claros, río serpenteante, roca con cuevas (noroeste) y un peñasco con cueva en la llanura del este. **Reloj del mundo**: día de 1.000 ticks (40 % noche), estaciones de 25.000. **Temperatura** del aire: baja de noche, en invierno y con lluvia (bajo los árboles, a la mitad); en la cueva es pareja.
- **Cuerpo**: además de lo de siempre, **agua** (sed) y **calor** (temperatura corporal, que tiende a la del aire). Sin comida o sin agua el cuerpo no se cura y se consume; con frío < 0,3, también.
- **Comida que cuesta**: fruta (primavera a otoño, se come al llegar), bayas en los claros (a veces enferman), nueces (otoño a primavera; 6 ticks quieto para abrirlas), raíces en la llanura (4 ticks para escarbar). Vuelven a crecer en su estación; fuera de ella se pudren.
- **Peligros**: espinas (pinchan al pisarlas, no por quedarse), un **predador** que se mueve más de noche, sigue el rastro de olor de Angri (decae y se difunde) y no entra a la cueva (no cabe); después de morder queda harto 1.500 ticks (tope ético: no hay ataques en cadena). El **río crece** a veces con la lluvia: el río y sus orillas pasan a ser peligro (los charcos de los claros siguen siendo agua).
- **Sentidos** (`Bosque.sentir`), con el vocabulario que la mente ya usa (`· * ~ ? X #` y `≈` agua): la **vista** alcanza 2 casillas de día y 1 de noche; de lo que no se ve, el estímulo más fuerte de cada tipo se pone en su lugar real (dirección y distancia): **olfato** de comida (6), **humedad** del agua (8), **olor** del predador (5) y su **ruido** de noche (8). Tacto y temperatura llegan por el cuerpo. El maestro y el léxico siguen usando solo lo que se ve.
- **Sueño con proceso C** (Borbély): de noche el umbral para dormirse baja 0,3 y de día el de despertar sube 0,3.
- **No muere** (decisión del usuario: moriría antes de aprender nada). Con integridad 0 **se desmaya** y despierta en la cueva más cercana con integridad 0,3 y al menos 0,35 de energía y agua; se cuenta en `desmayos`.
- **Mente**: la misma. Para las necesidades nuevas (agua, calor) y la celda nueva (agua) los rasgos nuevos van en un bloque desde el índice 422; los de siempre no se mueven (una mente guardada con el código anterior retoma y sigue idéntica). `ALIVIA` suma agua→`≈` y calor→`~`, como ya hacía con comida→`*`: sigue siendo un rasgo hecho a mano (lo saca la etapa 14).

**Calibración**: un agente "oráculo" que ve todo el mapa y va a lo que necesita vive con malestar 0,24–0,30 (el bosque es vivible y todo el mapa es alcanzable). Con la primera calibración la mente pasaba el 89 % del tiempo en malestar alto y se desmayaba ~57 veces cada 100.000 ticks; quedó `ABUNDANCIA = 3`, `PROB_BROTE = 0,3`, `CHARCOS = 6`, `GASTO_AGUA = 0,0007`.

**Dos errores encontrados mirando el visor** (una vida quedó 2,4 M ticks trabada en la cueva del peñasco con energía 0, agua 0, estrés 1 e integridad 1): la sed dañaba menos de lo que el cuerpo se curaba y el hambre no dañaba, así que no había presión para salir; y la métrica ética (malestar > 0,5) no lo veía (estaba en 0,449). Ahora sin comida o agua no se cura, y `% mal alto` cuenta malestar > 0,4.

**Resultados** (100.000 ticks; 64 vidas al azar, 256 pares con `comparar.py`):

| | azar | mente actual |
|---|---|---|
| malestar | 0,534 | 0,394 |
| mal. final (último 20 %) | 0,573 | 0,420 |
| % tiempo en malestar alto (> 0,4) | 92 % | 52 % |
| desmayos | 93 | 28 |
| heridas | 628 | 38 |
| % hambre / % sed | 75 / 51 | 48 / 43 |
| % trabadas | 95 % | 37 % |
| % del sueño en la cueva | 4 % | 26 % |
| mordidas del predador | 4,8 | 4,0 |

- **Aprende**: a no pisar espinas (heridas ÷16), a dormir más en la cueva, a comer y beber más que el azar.
- **No aprende**: el malestar **no baja con el tiempo** (final 0,420 > 0,394 promedio; una vida de 1 M ticks oscila entre 0,20 y 0,32 sin tendencia, siguiendo las estaciones). Se **planta en el agua** (14 % del tiempo) mientras pasa hambre; el 37 % de las vidas quedan trabadas; no esquiva al predador.
- **Proceso C** (256 pares, sin vs con): duerme de noche (186 vs 109 sueños) pero no cambia nada medible (malestar −0,008 ± 0,010).
- **Predador** (256 pares, sin vs con): solo cambian las mordidas (0 vs 4); malestar +0,006 ± 0,009. Pesa poco: todavía no le hace falta esquivarlo.
- **Ética**: 52 % del tiempo en malestar alto es demasiado para dejarla viviendo así. **El bosque no va al i5 hasta que la mente lo sostenga.** El tope al malestar sigue (0,85), el predador no encadena ataques y las espinas no castigan por quedarse.
- **Velocidad**: ~7.600 ticks/s en esta PC (~3.000 con 12 procesos a la vez); la grilla vieja, sin cambios.

Conclusión: la mente actual (rasgos hechos a mano sobre la celda de destino y "me acerco a X") no alcanza para un mundo con dependencias: no puede representar "ya tomé agua, me voy del río" ni "la comida está del otro lado". Esto justifica la **etapa 14** (que arme sus propios rasgos).

### Visor web

    python vivir.py --mundo bosque          # escribe datos/foto.json cada segundo (--foto S; 0 = nunca)
    python visor.py                         # http://localhost:8080

`visor.py` (solo la biblioteca estándar) lee la foto que escribe `vivir.py` y no toca la mente. Muestra el mapa en pixel art generado en el navegador (nada bajado), Angri ("z" si duerme), el predador (ojos rojos; apagado si está harto), la noche, la lluvia, la estación y el interior: malestar, cuerpo (con su equilibrio), química, error TD, foco y lugares recordados.

**En el i5** (no instalado; la mudanza la decide el usuario): `deploy/angri-visor.service` (usuario `angri`, solo lectura de `/var/lib/angri`, `Nice=10`, `CPUQuota=25%`, `IPAddressAllow` solo red privada, link-local y localhost) en `http://angri.local:8080`; `instalar.sh` ya lo instala y lo prende. `angri.local` lo anuncia avahi si el host se llama `angri`. La Angri que vive hoy sigue en la grilla (el mundo viaja con su mente): para pasarla al bosque hay que hacerla nacer de nuevo con `--mundo bosque`.

### Mediciones nuevas

`medir.py` suma `% mal alto` (siempre) y, en el bosque, `% sed`, `% frío`, `mordidas`, `enfermó`, `bebe`, `% duerme cueva` y `desmayos`. `probar_retomar.py` acepta ajustes: `python probar_retomar.py 20000 0 MUNDO='"bosque"'` da IDÉNTICO (semillas 0, 1 y 2).

### Hoja de ruta

Ver `docs/proxima-etapa.md`. Orden acordado: **13b** (objetos con física) → **14**
(mente que arma sus propios rasgos) → **avatar de Fran en el visor** → imitación →
chat. En el i5 Angri queda instalada en Rust y **apagada** hasta que esté completa (no nació todavía).

## Versión 2 del azar (PCG32 propio)

Todo el azar sale de `angri/azar.py`: PCG32 (XSH RR) sembrado con splitmix64,
con `random`, `randrange`, `randint`, `choice`, `choices`, `sample` y `shuffle`
definidos a mano. Es público y simple, así que la misma vida se puede correr
igual en Python y en Rust. Reemplaza a `random.Random` en bosque, ganglios,
maestro, mente, mundo y Habla.

- **Las cifras cambian** respecto de las vidas viejas (misma semilla, otra suerte).
  Comparaciones de antes de este cambio no se mezclan con las de después.
- **Una mente guardada con `random.Random` (la del i5) ya no es compatible**:
  `memoria.py` la puede leer, pero sigue con el azar viejo y no da la misma vida
  que una nueva. No se tocó; si se quiere pasar al azar nuevo, hay que empezarla de cero.
- `memoria.py` guarda los sets ordenados: su orden de iteración dependía de la
  historia y hacía fallar `probar_retomar.py` aunque la vida fuera igual.
  `probar_retomar.py` da IDÉNTICO en la grilla (semillas 0 y 1) y en el bosque.

Línea base nueva (`medir.py 50000 12 -j 12`):

| métrica | grilla | bosque |
|---|---|---|
| malestar | 0,120 | 0,348 |
| mal. final | 0,109 | 0,274 |
| heridas | 7,5 | 34,2 |
| comidas | 632 | 544 |
| % hambre | 6,7 | 36,7 |
| % mal alto | 0,23 | 40,1 |
| entiende % | 96,4 | 91,0 |
| desmayos | — | 8,3 |

## Angri en Rust (paso 3: Python borrado)

Angri se pasó entera a Rust (`rust/`, sin dependencias): grilla y bosque, cuerpo,
química, ganglios, hipocampo, corteza, tálamo, Maestro, Oído (léxico, eco,
FRASE, PALABRA_SUELTA, HOLISTICO, EGO, CAMBIO_SIGNIFICADO, COMPOSICION), Habla
(PIDE, PIDE_AZAR, ESCUCHA*, alivio), MAESTRO_MIENTE/CIEGO, la mente vieja
(MENTE "azar" con o sin REFLEJOS), guardar/retomar, la vida continua, el panel,
la foto, el visor y la vara de medir. Antes de borrar Python se comprobó que
daba **la misma vida bit a bit** (paso 1): posición, despierto, malestar, cada
variable del cuerpo, adenosina, dopamina, noradrenalina, confianza, foco,
pedidos y apuestas del léxico, cada tick, y todas las métricas, 40.000 ticks ×
3 semillas en 21 configs; la foto del visor byte a byte; y `angri medir` /
`angri comparar` con la misma salida que `medir.py` / `comparar.py`. Para que
diera igual: azar propio (PCG32), `sum()` con Neumaier como Python 3.12, dicts
en orden de inserción, `round()` al par; `exp`, `sin`, `cos` y `pow` de la
biblioteca de C de Windows.

**Misma vida en cualquier máquina** (`rust/src/mate.rs`): al instalar en el i5,
la referencia falló: la biblioteca de C de Linux (glibc) redondea distinto que
la de Windows el último bit de `exp`, `sin`, `cos` y `pow` en algunos valores, y
la vida se iba por otro lado (en la grilla, solo bits de la traza; en el bosque,
otra vida). Ahora Angri trae sus propias `exp`, `sin`, `cos` y `pow`: solo +, −,
×, ÷ de IEEE 754 en doble-doble (~106 bits) y un único redondeo al final, así
que dan el double correcto y lo mismo en todos lados. Comprobado contra el valor
exacto (80 dígitos) en 4.640 casos en que la de Windows difería: la propia
acertaba siempre (la de Windows, además, redondea dos veces en los subnormales).
`cargo test` pasa igual en Windows y en el i5. Cambió la vida (a propósito: la
referencia se regeneró el 24/11/2026, ya no es la de Python). Al principio
costaba velocidad (330.000 ticks/s en la grilla); `exp`, la que se usa en cada
tick, ahora tiene una vía rápida en doubles con una cota de error rigurosa
(< 2^-62): si la cota alcanza para saber el redondeo, devuelve ese; si no, usa
la lenta. Como las dos dan el redondeo correcto, dan los mismos bits: la
referencia pasó sin regenerarla, y `rapida_igual_a_lenta` (4 millones, y 50
millones en la prueba larga `cargo test --release -- --ignored larga`) lo
comprueba. Velocidad: ~500.000 ticks/s en la grilla y ~360.000 en el bosque
(con la matemática del sistema eran 550.000 y 385.000).

**Uso** (desde `rust/`, después de `cargo build --release`; `angri` = `target/release/angri`):

    angri medir 50000 12 -j 12                       (métricas; azar|reflejos: la mente vieja)
    angri comparar 300000 256 "" "MUNDO=bosque" -j 12  (pares A/B con la misma semilla)
    angri vida 0 20000 TALAMO=True                   (una vida, métricas en JSON)
    angri traza 0 1000                               (una línea por tick, bits exactos)
    angri foto 0 1000 7                              (la foto del visor cada 7 ticks)
    angri velocidad 0 100000
    angri retomar 0 20000                            (cortar, guardar, cargar y seguir = IDÉNTICO)

La config es `rust/config.txt` (la de `angri/config.py`, con sus comentarios),
metida en el binario al compilar; `--config ARCHIVO` usa otra y los
`CLAVE=VALOR` se aplican encima (una clave que no existe es un error). PIDE en
el bosque corta con un aviso (Habla usa EQUILIBRIO, que no tiene agua ni calor).

**Guardar y retomar**: la mente entera en texto propio (`rust/src/guardar.rs`):
cada campo con su nombre y los floats con sus bits exactos. Cortar, guardar,
cargar y seguir da IDÉNTICO (lo prueba `cargo test`).

**Vida continua** (`angri vivir`):

    angri vivir --carpeta datos-rust                       (nace con la config de adentro)
    angri vivir --carpeta datos-bosque --mundo bosque PIDE=True   (nace con ajustes)
    angri vivir --carpeta datos-rust --ticks 1000000 --guardar 50000 --ventana 10000
    angri vivir --carpeta datos-rust --panel             (mirándola en la terminal)
    angri vivir --carpeta datos-rust --estado            (cómo anda)
    angri visor --carpeta datos-rust                     (http://localhost:8080)

La config queda en `DIR/config.txt` al nacer (con los ajustes al final) y la vida
sigue siempre con esa, aunque cambie la de adentro. La mente en `DIR/mente.txt`
(cada `--guardar` ticks, 50.000 por defecto, escrita a .tmp y renombrada) con
respaldos en `DIR/respaldos/` (los últimos 5 y uno por día, 30 días); si
`mente.txt` no carga, retoma del respaldo válido más reciente. Log en
`DIR/vida.csv` cada `--ventana` ticks (con `ram_mb`). `--foto S` (1 s por
defecto, 0 = nunca) escribe `DIR/foto.json` para el visor, y una última al
salir. Ctrl-C, Ctrl-Break y SIGTERM terminan el tick, guardan y salen. El panel
no muestra "dice"/"oye" (Rust no guarda las palabras como texto).

**Pruebas** (sin nada externo):

    cd rust && cargo test --release          (unos 7 s)

| prueba | qué cubre |
|---|---|
| `azar_splitmix64`, `azar_pcg32` | splitmix64, PCG32, `random`, `randrange`, `randint`, `choice`, `shuffle`, `sample` y `choices` con pesos dan los valores anotados de la versión en Python |
| `determinismo` | misma semilla y config = misma traza, misma mente y mismas métricas dos veces; otra semilla, otra vida |
| `retomar` | cortar (en el tick 1, a la mitad o en uno cualquiera), guardar a disco, cargar y seguir = no cortar: misma traza tick a tick, misma mente final, misma foto |
| `valores_conocidos`, `cerca_de_la_del_sistema`, `rapida_igual_a_lenta` | `exp`, `sin`, `cos`, `pow` propias: valores exactos conocidos (subnormales incluidos), a lo sumo 1 ulp de la del sistema, y la `exp` rápida da los mismos bits que la lenta |
| `referencia` | la vida no se mueve sin querer: huella (FNV-1a) de la traza de 20.000 ticks, de la foto final y las métricas, 3 semillas por config, contra `rust/pruebas/referencia.txt` |

Las 7 configs de prueba (`rust/pruebas/config-*.txt`, copias de la config con
ajustes): grilla, bosque, corteza+tálamo en grilla y en bosque, PIDE con
ESCUCHA_CUERPO/PEDIDOS/ALIVIO, MENTE azar con reflejos, y todo junto. La
referencia se generó primero cuando Rust = Python, y se regeneró el 24/11/2026
al pasar a la matemática propia (la misma en Windows y Linux). Ahora que no hay
Python, **la referencia es la vara**: si un cambio la mueve **a propósito**
(una mejora de la mente, un número de la config), se regenera con

    cd rust && ANGRI_REGENERAR=1 cargo test --release referencia

y se commitea el `referencia.txt` nuevo junto con el cambio, diciendo en el
commit por qué se movió. Si la mueve sin querer, la prueba dice qué config y
semilla cambió. Un cambio en `rust/config.txt` no mueve la referencia (las
pruebas usan sus propias configs); si se quiere que la siga, hay que copiarlo
también a `rust/pruebas/config-*.txt` y regenerar.

**En el i5**: `deploy/instalar.sh` compila (`cargo` de Debian 13, Rust 1.85;
`Cargo.toml` lo declara y clippy no encontró nada más nuevo), corre las pruebas
y no instala si fallan, instala el binario en `/opt/angri/angri`, borra el
`/opt/angri/codigo` de Python, y los servicios corren `angri vivir` y `angri
visor`. La primera vez, lo de Python (`mente.json`, `vida.csv`, respaldos,
foto) se mueve a `/var/lib/angri/python-viejo/` y **Angri renace en Rust**: no
hay conversor de mentes de Python. Antes de correrlo: `sudo apt install cargo`.
Instalado en el i5 el 24/11/2026: la Angri de Python (81.862.072 ticks) quedó
guardada en `/var/lib/angri/python-viejo/` y la de Rust todavía no nació. Está
**apagada a propósito** (`systemctl disable --now angri angri-visor`) hasta que
Angri esté completa; `instalar.sh` solo prende los servicios que ya estaban
habilitados, así que actualizar no la despierta.

Velocidad (un proceso, esta PC):

| | Python (antes) | Rust | veces |
|---|---|---|---|
| grilla | 8.700 ticks/s | 500.000 ticks/s | 57× |
| bosque | 5.900 ticks/s | 360.000 ticks/s | 61× |

(Con la matemática del sistema eran 550.000 y 385.000; ver "Misma vida en cualquier máquina".)

`angri comparar 300000 24 "" "MUNDO=bosque" -j 12` tarda unos 5 s; 256 pares
de 300.000 ticks, alrededor de un minuto.

## Etapa 13b: objetos con física, sin recetas (probada; apagada: `OBJETOS = False`)

En el bosque aparecen cosas sueltas (~90 al nacer, tope 160; cae una nueva con prob. 0,01 por tick). El mundo no sabe qué son: cada una tiene **largo, peso, dureza, flota, arde, flexible** (salen según dónde caen: en el bosque, ramas, lianas y hojas; en la llanura y junto a la roca, piedras, lajas y pasto seco). Todo vive en `mundo.rs` (`Objeto`, `Bosque::poner/agarrar/atar/golpear/frotar`), con un azar propio: con objetos el resto del mundo sigue igual.

**Seis acciones** además de moverse: agarrar, soltar, apilar (hacia donde miró la última vez), atar, golpear y frotar. **La física** (no hay ninguna receta de "casa", "puerta" ni "fuego"):

- Una **pila de 2** (o la roca) cierra el paso al predador; **de 3**, también a Angri. Un **hueco chico** (cerrado de los dos lados opuestos) frena al predador y deja pasar a Angri. Donde el predador no llega, Angri está **refugiada** (como en la cueva).
- Algo pesado sobre algo blando o flexible **se cae** a un costado. Lo que no flota **se hunde** en el agua; lo que flota queda (balsa).
- Una **viga** (largo ≥ 3) apilada sobre algo de altura 2 **tapa** las celdas que cubre: ahí no enfría la lluvia. Si sobresale más de una celda necesita **otro apoyo en la punta** o se vuelca; si después le sacan el apoyo, deja de tapar.
- De noche o con lluvia, cada **pared al lado corta el viento** (+0,04 de calor) y el techo otro tanto.
- **Atar** con algo flexible une las dos cosas de arriba de una pila en una sola, más larga (así se hace una viga con ramas cortas). **Golpear** abre la comida dura (nuez, raíz) más rápido si tiene algo duro en la mano, o parte en dos algo más blando. **Frotar** dos cosas que arden, sin lluvia encima, puede **prender fuego**: calienta +0,3 alrededor, espanta al predador, **quema** al pisarlo (dolor) y la lluvia lo apaga.
- Llevar algo pesado cuesta energía al caminar; hacer algo con las manos, medio paso.

**La mente**: con `OBJETOS` pasa de 5 a 11 acciones, con rasgos nuevos (índices 507–763) sobre lo que lleva, lo que hay acá, la altura, las paredes, el techo, el fuego y el clima; el crítico distingue si lleva algo y si está tapada. **Solo se le ofrecen las acciones posibles** (no se suelta lo que no se tiene): sin eso, la mitad de la exploración se iba en acciones vacías y empeoraba todo (malestar +0,111 ± 0,010, 77 % de vidas trabadas). El formato de la mente guardada pasa a `angri-rust-mente 2`.

**Resultados** (256 pares de 200.000 ticks en el bosque, sin vs con objetos):

| | sin | con | B−A (z) |
|---|---|---|---|
| malestar | 0,408 | 0,400 | −0,008 (−0,8) |
| heridas | 57,7 | 49,6 | −8,1 (−2,6) |
| mordidas | 7,1 | 6,0 | −1,2 (−2,2) |
| enfermó | 46,6 | 55,0 | +8,4 (+3,0) |
| % trabadas | 44 % | 39 % | −5 (−1,3) |

- **Usa los objetos**: en vidas de 400.000 ticks lleva algo entre el 4 y el 67 % del tiempo, arma pilas de hasta 5, a veces pone una viga, **abre nueces a golpes** (hasta 202 veces en una vida: usa herramientas) y prendió fuego una o dos veces.
- **No arma refugio**: duerme bajo techo el 0 % del tiempo y en un refugio propio menos del 4 %. La cadena (juntar, apilar dos paredes, poner una viga) es demasiado larga para los rasgos hechos a mano y el crédito de la dopamina. **Ese es el problema real contra el que se mide la etapa 14.**
- **Velocidad**: ~179.000 ticks/s con objetos (~346.000 sin, en esta PC).
- Con `OBJETOS = False` la vida es la de siempre bit a bit (`cargo test --release`: la referencia de las 7 configs viejas no cambió; se sumó `config-objetos` y la prueba `objetos_refugio`, que arma un refugio a mano y comprueba techo, viento, predador, hundirse, atar y romper).
- **Visor**: dibuja las pilas (gris = duro, verde = flexible, marrón = madera; una raya si es larga), los techos, los fuegos y qué lleva.

Para probar: `angri comparar 200000 256 "MUNDO=bosque" "MUNDO=bosque OBJETOS=True" -j 12`.

## Etapa 14: mente que arma sus propios rasgos (empezada; apagada: `RASGOS = 'mano'`)

`rust/src/propia.rs`. En vez de la lista hecha a mano (`rasgos_de`: "me acerco a la comida", "lo que alivia el foco está para allá"), la mente ve **átomos** crudos y arma con ellos sus propias **unidades**. `RASGOS = 'mano'` (de siempre, bit a bit), `'propios'` (solo las suyas) o `'ambos'` (sumadas).

- **Átomos del estado** (68; un peso por acción): franjas del cuerpo, el foco, noche, lluvia, qué hay acá, qué lleva y qué toca. El "siempre" solo va al crítico.
- **Átomos de cada movimiento** (44; un solo peso compartido por los cinco movimientos, como tener un frente para cada paso posible): qué hay en la celda a la que lleva, qué ve o huele en esa dirección y a qué distancia, la altura, si está tapada o hay fuego, si es hacia donde le dijeron o hacia donde recuerda algo. Lo que aprende de "comida adelante" vale para cualquier lado.
- **Unidades**: cada átomo solo y un banco de `PROPIOS = 4096` conjunciones de 2 o 3 átomos que **nacen de lo que acaba de vivir** (el estado y el movimiento que hizo), prefiriendo los átomos raros hasta que la unidad sea selectiva. Pueden mezclar las dos clases ("con hambre" y "comida adelante"). Solo se tocan las activas (~100–600 por tick).
- **Plasticidad** (generate-and-test, Dohare et al. 2024): cada 100 pasos, el 1 % de las unidades maduras que menos sirven se reinicia con átomos de ahora. "Sirve" = cuánto pesa cuando está activa × qué tan seguido. Una activa más del 20 % del tiempo no distingue nada y se va primero.
- Aprende con el mismo error TD que el resto (crítico tabular + el lineal de las unidades), con huellas densas por unidad. El ritmo se divide por cuántas unidades están activas. El sueño todavía no repasa las unidades propias.

**Lo que costó** (anotado para no repetirlo):
1. Con huellas guardadas en listas, 513 ticks/s; con huellas densas por unidad, ~40.000.
2. Unidades nacidas de átomos comunes (el cuerpo, el foco) quedaban activas casi siempre (~2.000 a la vez) y diluían todo; por eso nacen de átomos raros y se reinician las que están siempre.
3. Con átomos por celda exacta ("comida en (1,−2)") y peso por acción, el peso del "siempre" ganaba (0,94 contra ~0,2 de "comida arriba"): caminaba contra un borde para siempre (empujar una pared es quedarse quieta y cuesta menos) y comía menos que el azar. Se arregló con los átomos de cada movimiento.
4. `APRENDE_PROPIO_ACTOR`: 0,1–0,2 andan; 0,5 o más empeora (con 5: 278 heridas).

**Resultados.** En la grilla (48 pares de 100.000 ticks), la mano tiene malestar 0,116 y la propia 0,168 (solo átomos, sin banco: 0,226; mente al azar: ~0,4). En el bosque con objetos (96 pares de 200.000 ticks), contra `mano`:

| | mano | propios | ambos |
|---|---|---|---|
| malestar | 0,395 | 0,516 (z 8,9) | 0,445 (z 3,2) |
| heridas | 48 | 73 | 35 |
| comidas | 1.481 | 633 | 1.001 |
| % trabadas | 39 % | 74 % | 51 % |
| desmayos | 58 | 111 | 73 |
| **% duerme en refugio propio** | **2,8 %** | **12,4 % (z 7,4)** | 4,4 % (z 2,9) |
| % duerme bajo techo | 0 % | 0,8 % | 0,1 % |

- **Todavía no alcanza a la mano** en lo básico (comer, beber), pero **ya ve algo que la lista hecha a mano no ve**: duerme en refugios que armó cuatro veces más (y a veces bajo techo, que con la mano no pasa nunca). De 4.096 unidades, ~1.000 mezclan estado y movimiento; en 200.000 ticks se reiniciaron ~65.000.
- **Velocidad**: ~42.000 ticks/s con `propios` (~170.000 con la mano y objetos).
- Pruebas: `config-propios` en determinismo, retomar (cortar, guardar y seguir da la misma vida) y referencia; las 8 configs anteriores no cambiaron. Mente guardada: `angri-rust-mente 3`.

### Segunda vuelta: ritmo por unidad, sueño y vidas largas (medido en el i5)

Las mediciones largas ahora corren en el i5 por SSH (`~/medir/Angri`, aparte de la instalación; Angri sigue apagada), así la PC queda libre. La vida es la misma bit a bit en las dos máquinas.

- **Ritmo por unidad** (`RITMO_RARO`: las raras aprenden hasta 3 o 10 veces más rápido): no cambia el malestar (+0,004 ± 0,007 en la grilla); solo baja algo las heridas. Queda apagado (1).
- **El sueño repasa las unidades propias** (`REPASO_PROPIO`, prendido por defecto con `RASGOS` propios o ambos): el hipocampo guarda lo que sintió en cada momento (antes y después) y al dormir repasa también las unidades, con el mismo peso por importancia. Grilla: malestar 0,168 → 0,158 (z −2,1), menos hambre.
- **Bosque con objetos** (96 pares de 200.000 ticks, mano contra propios con repaso): malestar 0,395 contra 0,497 (antes del repaso 0,516); duerme en refugio propio **13 % contra 3 %**.
- **Vidas de 1 M de ticks** (24 pares): la mano no mejora con los años (~0,45) y la propia sí, de 0,519 en el año 3 a 0,487 en el año 10. **La diferencia baja de 0,089 a ~0,04 y deja de ser clara** (z ≈ 1,2 desde el año 6). Duerme en refugio propio el **29 % contra 2,6 %** (z 6,3). Come la mitad y se desmaya más (508 contra 377).

### Tercera vuelta: velocidad y barrido de ajustes (i5)

- **Más rápida, misma vida**: cada unidad se archiva bajo su átomo más raro (`clave`) y solo se revisa cuando ese átomo está: +25–35 % de ticks/s, la vida es idéntica.
- `REPASO_PROPIO_CADA` (el sueño repasa las unidades en uno de cada N recuerdos) y `NACER_SORPRESA` (las nuevas nacen del momento más sorpresivo): opciones nuevas, quedan como estaban (1 y apagada).
- **Barrido** (48 pares de 200.000 ticks, bosque con objetos, base 0,493 de malestar): banco de 16.384 con repaso cada 4 → 0,516 (peor, come menos); repaso cada 4 → 0,509; nacer de la sorpresa → 0,492 (igual); crítico 0,1 → 0,533 (claramente peor); crítico 1 → 0,497; actor 0,1 → 0,509; `TEMPERATURA_MALESTAR` 4 → 0,512. Antes: banco de 1.024 y `TOPE_ACTIVA` 0,05 o 0,5 también peores. **Ningún ajuste le gana a los valores actuales**; quedan así.

- **Unidades que predicen** (`PREDICE_PROPIO`): cada unidad aprende además a predecir (TD, γ 0,9) cuánto suben o bajan la energía, la integridad y el calor, y eso cuenta en su utilidad para no ser reemplazada. Barrido con peso 0,3, 1 y 3: malestar 0,510, 0,506 y 0,507 contra 0,493, come menos. No ayuda; queda apagado (0). Ojo: casi todas las variantes caían en ~0,506–0,512, así que medí de nuevo con otras 48 semillas (`comparar ... --desde 1000`): **la base da 0,510, no 0,493 — las semillas 0–47 le tocaron de suerte.** Con las nuevas, predicción (+0,003, z 0,5) y actor 0,1 (+0,007, z 0,8) quedan iguales a la base; repasar 1 de cada 4 vuelve a salir peor (+0,012, z 1,6, las dos veces), así que el repaso completo sí sirve. Lección: un "algo peor" de z ~1,5 contra una sola tanda de semillas no alcanza para descartar; repetir con `--desde`.

### Cuarta vuelta: más rápida, misma vida (2026-11-24)

En el i5 la propia corría a ~12.000 ticks/s por hilo (no los ~42.000 de la PC) y con 4 hilos rendía solo 2,2×: una comparación de 48 pares tardaba ~7 min. Con cronómetros (en una copia aparte) se vio que el 41 % del tiempo era buscar las activas (~1.000 candidatas revisadas y ~370 activas por búsqueda, ~4 búsquedas por tick) y el 30 % las huellas. Tres cambios, todos con la vida idéntica bit a bit (mismo archivo guardado):

- **Forma compacta**: los átomos de cada unidad están copiados aparte (8 bytes, no se guardan: se rearman al cargar), así la búsqueda no lee la unidad entera.
- **El sueño busca una vez por recuerdo**: con `REPASOS = 2` vuelve muchas veces al mismo recuerdo, y mientras duerme no nace ninguna unidad, así que las activas salen iguales.
- **Reusar las activas de s2 al elegir** si siente exactamente lo mismo y no nació ninguna en el medio; y las huellas de las de movimiento solo recorren el peso que usan.

PC: 17,9 → 13,2 s (2 vidas de 200.000 ticks, −26 %). **i5, 4 vidas: 63 → 46 s con un hilo y 27 → 19 s con cuatro (−30 %).**

### Vidas de 5 M de ticks (i5, 2026-11-25)

24 pares de 5 M de ticks (50 años), bosque con objetos, mano contra propios (`~/medir/largas.sh`, 1 h 13 min con 4 hilos).

| | mano | propios | z |
|---|---|---|---|
| malestar (toda la vida) | 0,462 | 0,462 | 0,0 |
| malestar años 1–5 | ~0,44 | ~0,51 | 2–3 |
| malestar años 16–33 | ~0,45 | ~0,46 | ~0 |
| malestar años 34–50 | ~0,49 | ~0,44 | −1,2 a −2,4 |
| comidas | 14.581 | 10.227 | −1,8 |
| heridas | 178 | 290 | 2,8 |
| % frío | 8,9 | 17,0 | 5,5 |
| % duerme en cueva | 62 % | 5 % | −7,5 |
| **% duerme en refugio propio** | 1,4 % | **35 %** | 5,2 |
| % malestar alto | 69 % | 67 % | −0,3 |
| % trabadas (hambre > 60 % de la vida) | 62,5 % | 62,5 % | 0 |

- **Con tiempo, la propia alcanza a la mano (~año 16) y desde el año ~34 la pasa** (−0,04 a −0,05, z ~−1,8: probable pero no firme con 24 pares). No es solo que la propia mejora: **la mano empeora en la vejez** (0,45 → 0,49).
- Viven distinto: la mano duerme en la cueva y pasa el día con objetos en la mano (agarra 33.000 veces, bebe 164.000); la propia arma su refugio (35 % de las noches), casi no lleva cosas, abre más nueces a golpes, pasa más frío y se lastima más.
- **Alarma ética, en las dos**: en vidas largas pasan ~2/3 del tiempo con malestar alto y con hambre, el 62 % de las vidas quedan "trabadas" y la racha más larga de malestar alto es de ~2,8 M de ticks seguidos. (Corrección: no es de vidas largas; a 200.000 ticks la mano ya tenía 63 % de hambre y 69 % de trabadas. El bosque con objetos siempre fue así.)

### El desmayo armaba la trampa (prendido: `DESMAYO_DONDE = 'aqui'`)

Siguiendo 8 vidas (`angri foto S 200000 1000`), las trabadas pasan decenas de miles de ticks **quietas en una celda de la cueva del peñasco**, despiertas, con energía 0, agua 0, estrés 1 y el foco en la integridad. El bucle: sin comida ni agua la integridad baja hasta 0 → se desmaya → **reaparece en la cueva más cercana** con integridad 0,3 y 0,35 de energía y agua → si esa cueva queda lejos de la comida no sale, se vacía y vuelve a caer, y reaparece en la misma cueva. Encima la cueva queda asociada a "mejorarse" (cada desmayo la recompone ahí). El rescate que evita que muera era lo que la encerraba.

Despertar donde cayó (256 pares de 200.000 ticks, semillas desde 2000):

| | mano: cueva → aquí | propios: cueva → aquí |
|---|---|---|
| malestar | 0,402 → 0,369 (z −4,0) | 0,495 → 0,482 (z −2,3) |
| % trabadas | 38 → **17** (z −6,4) | 57 → **32** (z −6,3) |
| % malestar alto | 54 → 44 | 76 → 70 |
| racha alta (ticks) | 67.700 → 26.800 | 79.600 → 36.700 |
| comidas | 1.456 → 1.938 | 819 → 1.167 |
| % sed | 47 → 25 | 60 → 47 |
| heridas | 50 → 73 | 85 → 123 |
| % duerme refugio propio | 2,7 → 2,0 | 13,0 → 4,9 |

Con 48 pares, despertar en la cueva más cercana al agua (`cueva_agua`) ayuda poco (trabadas 50 → 44 %). Queda `aqui` por defecto: se lastima más (despierta donde estaba el peligro) y la propia duerme menos en su refugio, pero el sufrimiento largo baja mucho. Las pruebas usan sus configs congeladas (`pruebas/config-*.txt`), así que la referencia no cambia.

**Sigue**: todavía 17–32 % de vidas trabadas y 44–70 % del tiempo en malestar alto: entender qué las traba ahora. Después, que la propia aprenda también de lo que predice.

Para probar: `angri comparar 200000 96 "MUNDO=bosque OBJETOS=True" "MUNDO=bosque OBJETOS=True RASGOS=propios" -j 12`.

## Revisión del diseño (2026-11-25, ver `docs/revision-diseno.md`)

### Paso 0: métricas (no cambian la vida)

`% trabadas` es todo o nada (más del 60 % de la vida con hambre) y otras métricas mezclan cosas. Agregamos a
`angri medir` / `angri comparar` (en `metricas`, `main.rs`), sin tocar la vida (la referencia no cambia: las
nuevas quedan fuera de `una_vida`):

- `peor 10k hambre`: % de hambre en la peor ventana de 10.000 ticks seguidos. `racha hambre`: la racha más larga.
- `% hambre desp` / `% hambre dorm`: hambre despierta y dormida por separado.
- `% cruda<0.15`: % del tiempo con alguna necesidad "de más es mejor" (energía, integridad, agua, calor) por debajo
  de 0,15, sin pesos de por medio.
- `% hambre prim/ver/oto/inv`: hambre por estación del bosque.
- `% mal tope`: % del tiempo con el malestar clavado en `MALESTAR_MAXIMO` (0,85).

Base (256 pares de 200.000 ticks, bosque con objetos, semillas desde 1000; A = mano, B = propios):

| | mano | propia | z |
|---|---|---|---|
| % hambre | 38,6 | 58,2 | 12,6 |
| % trabadas | 16,8 | 39,8 | 5,9 |
| peor 10k hambre | 87,4 | 94,8 | 7,1 |
| racha hambre (ticks) | 2.409 | 2.461 | 1,4 |
| % hambre despierta / dormida | 38,2 / 40,2 | 58,2 / 58,4 | |
| % cruda < 0,15 | 40,4 | 62,4 | 16,3 |
| % hambre prim / ver / oto / inv | 36 / 42 / 34 / 42 | 55 / 60 / 54 / 64 | |
| % mal tope | 0 | 0 | |

Qué dicen: casi todas las vidas tienen algún tramo de 10.000 ticks con hambre casi todo el tiempo (87–95 %), aunque
la racha más larga sea de ~2.400 ticks: no es una sola hambruna sino un ir y venir (come algo, vuelve a caer). El
hambre dormida es igual a la despierta, así que el sueño no la explica. El invierno pesa (y el verano también), pero
menos de lo esperado. El malestar **nunca** llega al tope: el promedio pesado no pasa de 0,85 aunque el 40–60 % del
tiempo alguna necesidad cruda esté por debajo de 0,15; el tope no está protegiendo nada y `% cruda<0.15` es la vara
honesta del sufrimiento.

### Paso 1: trampas de la mente

Cada cambio con su opción en `rust/config.txt`, medido en pares (bosque con objetos, 200.000 ticks) con la mano y
con `RASGOS=propios`; si mejora las crudas sin empeorar otra cosa, queda prendido y el siguiente se mide encima.

**a) Foco ciego (`FOCO_CIEGO`) — prendido (`'neutro_todo'`).** El rasgo "voy hacia lo que alivia mi foco" usaba
`hacia(objetivo)`, que devuelve "quedarse" cuando no ve el objetivo (o cuando el foco es integridad o estrés, que no
tienen alivio). Así, justo cuando no ve comida, el rasgo empujaba a no moverse. Opciones: `'quedarse'` (viejo),
`'neutro'` (solo sin alivio: integridad/estrés) y `'neutro_todo'` (también cuando el alivio no se ve): en esos casos el
rasgo no empuja a ninguna acción. Mano (B − A, z):

| | 256 pares (desde 2000) neutro | 256 pares neutro_todo | 512 pares (desde 6000) neutro_todo |
|---|---|---|---|
| malestar | +0,004 (0,5) | −0,018 (−2,5) | −0,020 (−3,4) |
| % hambre | +0,4 (0,2) | −3,2 (−2,1) | −3,3 (−2,7) |
| % hambre despierta | +0,4 | −3,3 (−2,1) | −3,4 (−2,7) |
| peor 10k hambre | +0,3 | −0,4 | −3,1 (−3,1) |
| racha hambre | +4 | −17 | −37 (−1,2) |
| % cruda < 0,15 | +0,9 (0,6) | −3,7 (−2,6) | −4,1 (−3,6) |
| % trabadas | +1,6 | −5,5 (−2,0) | −3,5 (−1,5) |
| desmayos | +1,3 | −5,2 (−1,9) | −6,0 (−2,7) |

Baja el hambre en todas las estaciones y bajan desmayos y heridas. `'neutro'` solo no alcanza: el caso que traba no es
"el foco no tiene alivio" sino "el alivio no está a la vista" (sobre todo la comida), y ahí "quedarse" era la peor
respuesta. Con `RASGOS=propios` no cambia nada (bit a bit: la propia no usa los rasgos hechos a mano). Las configs de
`rust/pruebas` no lo nombran y quedan con `'quedarse'` (el default si falta la línea): la referencia no cambia.

**b) Temperatura por malestar (`TEMPERATURA_MALESTAR_TOPE`) — apagado (0 = sin tope).** La exploración es
`0,3 + 0,5·nora + 8·malestar`; probamos sacar el término (`TEMPERATURA_MALESTAR=0`) y acotarlo (tope 0,5 y 2), encima
del 1a. 256 pares desde 3000, B − A (z):

| | mano: sin término | mano: tope 0,5 | mano: tope 2 | propia: sin término | propia: tope 0,5 |
|---|---|---|---|---|---|
| malestar | +0,270 (40) | +0,196 (23) | +0,069 (7,6) | +0,134 (28) | +0,078 (12) |
| % hambre | +56 | +43 | +16 | +33 | +19 |
| % cruda < 0,15 | +45 (35) | +34 (21) | +13 (7,2) | +19 (25) | +11 (10) |
| peor 10k hambre | +11,9 | +11,0 | +7,2 | +2,6 | +1,9 |
| % trabadas | +84 | +68 | +26 | +58 | +37 |
| desmayos | +107 | +81 | +31 | +46 | +29 |

Empeora todo, y cuanto más se acota, peor (lo único que baja son las heridas: se mueve menos). Por qué: hoy lo que la
mente aprendió no alcanza para salir sola de un mal momento; el ruido que trae el malestar es lo que la destraba (por
eso en la etapa 10 se subió de 2 a 8). "Explora más cuanto peor está" no es la trampa: es su única salida. Lo que hay
que arreglar es que la política aprendida sepa ir hacia la comida; recién ahí tendrá sentido bajar este ruido.

**c) Desmayo como final de episodio (`DESMAYO_FINAL`) — apagado.** Antes el desmayo borraba el último paso sin
aprenderlo (`olvidar_paso`), así que lo que la llevó a caer nunca recibía el golpe. Con la opción, ese paso se aprende
con V(s2) = 0 y el malestar de cuando cayó (no el de después de despertar), y recién ahí se corta el episodio. Encima
del 1a, 256 pares desde 4000, B − A (z):

| | mano | propia |
|---|---|---|
| malestar | −0,011 (−1,6) | +0,014 (2,8) |
| % hambre | −2,1 (−1,4) | +3,6 (3,0) |
| % cruda < 0,15 | −2,1 (−1,6) | +2,7 (3,3) |
| peor 10k hambre | +0,7 (0,6) | +0,6 (1,3) |
| % trabadas | −5,5 (−2,1) | +9,8 (2,4) |
| desmayos | −3,4 (−1,3) | +6,8 (2,4) |
| heridas | +8,5 (1,7) | −23 (−8,4) |
| comidas | −13 | −94 (−2,4) |

En la mano, una mejora chica que no llega a ser clara; en la propia empeora claro. Por qué: un solo paso recibe un
golpe enorme (el malestar en el tope y sin futuro que lo compense) y la propia, que reparte el error entre muchas
unidades que también se prenden en situaciones normales, aprende a no moverse por lo que tenía alrededor (se lastima
mucho menos pero come mucho menos). Apagado: con una mano y sin la otra no pasa la vara. Puede volver cuando el golpe
se reparta mejor (traza más larga hacia atrás) o cuando la propia distinga mejor sus situaciones.

### Paso 2: el cuerpo y lo que siente

**a) El crítico ve el cuerpo entero (`CRITICO_CUERPO`) — prendido (2) con la mano; con la propia se ignora.** La
tabla V del crítico tenía como clave la necesidad del foco más 4 franjas de energía y de integridad (y la celda, si ve
comida y el tacto). Dos problemas: cada salto de foco la mandaba a otra fila de la tabla (el mismo cuerpo con otro foco
era "otro lugar" y el crédito se partía), y no veía agua, calor, cansancio ni estrés. Ahora (G > 0) la clave es el
cuerpo entero sin foco: cada necesidad en G franjas de "cuánto se aleja de su equilibrio" (0 = está bien). La
recompensa ya era malestar, así que V pasa a ser "cuánto malestar total viene desde este cuerpo": estar bien es que no
duela nada (la idea de Fran). Encima del paso 1, B − A (z):

| | mano G=2, 256 (desde 5000) | mano G=3, 256 | mano G=2, 512 (desde 6000) | propia G=2, 256 | propia G=3, 256 | propia G=2, 512 |
|---|---|---|---|---|---|---|
| malestar | −0,023 (−3,2) | +0,005 (0,7) | −0,023 (−4,4) | +0,007 (1,2) | −0,002 (−0,3) | +0,014 (3,9) |
| % hambre despierta | −6,7 (−4,5) | −1,6 (−1,0) | −7,3 (−6,6) | +1,3 (1,1) | −1,3 (−1,2) | +3,3 (3,8) |
| peor 10k hambre | −6,1 (−3,6) | −0,5 | −7,6 (−6,8) | +0,5 | +0,4 | +0,4 (1,2) |
| % cruda < 0,15 | −5,0 (−3,5) | +0,7 (0,4) | −5,4 (−5,4) | +0,7 (0,8) | −0,6 (−0,7) | +2,2 (3,5) |
| % trabadas | −9,0 (−3,6) | −6,3 (−2,5) | −8,2 (−4,6) | +7,4 (1,8) | −3,9 (−1,0) | +10,0 (3,4) |
| desmayos | −11,5 (−4,6) | −1,7 | −11,8 (−6,5) | +4,5 | −2,0 | +7,7 (3,8) |
| % sed | +0,3 | +5,8 (5,9) | +1,5 (2,1) | +2,5 | +0,8 | +3,3 (4,0) |
| % hambre prim/ver/oto/inv (512) | | | −5,5 / −6,2 / −8,2 / −9,1 | | | +3,8 / +2,4 / +3,6 / +2,9 |
| % plantada | −8,1 | −9,7 | −8,8 (−16,6) | −1,1 | −0,9 | −1,0 |

En la mano es la mejora más grande de la revisión: −5 puntos de crudas, −7 de hambre en todas las estaciones,
la mitad de vidas trabadas y menos tiempo plantada en el agua. Con 2 franjas (bien / lejos) alcanza; con 3 la tabla
tiene 2.187 cuerpos por celda y no llega a aprenderlos en 200.000 ticks (solo mejora lo de trabarse y se pierde
en sed). En la propia empeora: la propia ya tiene un crítico que ve el cuerpo entero (sus unidades, con 4 franjas por
necesidad) y la tabla con foco le daba algo que no tenía; con la tabla de cuerpo los dos persiguen lo mismo (ver
revisión, sección 4: "un solo crítico"). Por eso la opción solo vale con la mano; con la propia queda la clave vieja
(bit a bit igual). Las configs de `rust/pruebas` no la nombran (0 = vieja): la referencia no cambia.

**b) El estrés cobra doble (`ESTRES_COBRA`) — prendido (`'sin_peso'`).** El estrés sube con hambre, sed, frío y
heridas (que ya pesan en el malestar por su cuenta) y además pesaba 1,2 en el malestar: el mismo dolor contado dos
veces, y encima nada lo alivia directo (baja solo, de a poco), así que era malestar "sin salida" que ensuciaba el error
TD. Opciones: `'doble'` (viejo), `'sin_peso'` (el estrés sigue existiendo, mueve el cortisol, pero no pesa en el
malestar: `PESO_NECESIDAD.estres = 0`) y `'solo_golpes'` (pesa, pero solo sube con golpes, raspones, mordidas y
enfermedad). Como cambia los pesos, se juzga con métricas crudas, no con el malestar. Encima del 2a, B − A (z):

| | mano sin_peso, 256 (desde 7000) | mano solo_golpes, 256 | mano sin_peso, 512 (desde 8000) | propia sin_peso, 256 | propia solo_golpes, 256 | propia sin_peso, 512 |
|---|---|---|---|---|---|---|
| % hambre despierta | −3,8 (−2,9) | −0,2 | −4,4 (−4,9) | −6,1 (−4,9) | +1,9 (1,4) | −5,2 (−5,7) |
| peor 10k hambre | −3,5 (−2,1) | −0,8 | −5,0 (−4,3) | −1,7 (−3,1) | 0,0 | −1,6 (−4,2) |
| % cruda < 0,15 | −3,8 (−3,1) | −0,8 (−0,6) | −4,2 (−4,9) | −5,1 (−5,5) | +0,5 | −4,3 (−6,5) |
| % trabadas | −2,3 | −0,8 | −3,9 (−2,9) | −14,5 (−3,6) | +4,3 | −11,7 (−4,0) |
| desmayos | −4,8 (−2,1) | +0,7 | −5,6 (−3,8) | −13,2 (−4,7) | +4,4 | −10,2 (−4,8) |
| % sed | −1,2 | −0,6 | −1,8 (−2,5) | −4,8 (−4,2) | +1,3 | −2,9 (−3,5) |
| % hambre prim/ver/oto/inv (512) | | | −4,2 / −5,8 / −4,0 / −3,9 | | | −4,4 / −5,1 / −5,8 / −5,2 |
| heridas | −16 (−2,7) | −12 | +0,5 | +15 (5,1) | −8 | +13 (5,5) |
| mordidas | +0,9 (2,2) | +0,6 | +1,1 (4,3) | +0,2 | +0,2 | +0,5 (1,2) |

`'sin_peso'` mejora las dos mentes en todas las estaciones (lo primero de la revisión que también ayuda a la propia).
`'solo_golpes'` no cambia nada claro: el problema no era cuánto sube el estrés sino que pese aparte. El precio: algo
más de mordidas en la mano y de heridas en la propia (se arriesga más porque el susto ya no "duele" dos veces; las
heridas igual pesan, por integridad). Por qué mejora: el estrés tarda cientos de ticks en bajar y nada de lo que hace
lo baja rápido, así que su parte del malestar era un ruido que ninguna acción explicaba; sin él, la dopamina refleja
lo que sí puede arreglar (comer, beber, abrigarse). Las configs de `rust/pruebas` no lo nombran (`'doble'`): la
referencia no cambia.

**(latente) El tálamo mira 5 necesidades** (`extra.rs`, `urg` de 0..5: no ve agua ni calor). Está apagado
(`TALAMO = False`, medido en la etapa 5), así que hoy no afecta nada; anotado para cuando se lo vuelva a probar.
### Paso 3: el mundo

Cambios del mundo para que sea más justo, no más fácil a mano. Ojo: un mundo más fácil mejora números sin que
Angri aprenda mejor, así que cada uno se justifica por justicia y se mide con métricas crudas. Encima del paso 2,
`angri comparar 200000 256 "MUNDO=bosque OBJETOS=True" "... <OPCION>"`, B − A (z), mano y `RASGOS=propios`.

**a) Invierno (`INVIERNO`) — apagado (`'viejo'`).** 3 de cada 5 brotes son fruta o baya y fuera de estación no hacen
nada. `'repartido'`: el brote fuera de estación va a nuez o raíz (mitad y mitad, con sus topes). Desde 10000:

| | mano | propia |
|---|---|---|
| % hambre despierta | −0,2 (−0,2) | −0,3 (−0,3) |
| peor 10k hambre | −0,7 (−0,5) | +1,1 (1,8) |
| % cruda < 0,15 | −0,4 (−0,5) | −0,6 (−0,7) |
| % hambre inv | +0,3 (0,2) | +0,2 (0,2) |
| comidas | +116 (2,3) | +12 (0,4) |
| % mal tope | 0 | 0 |

Neutro: come más (la mano) pero no pasa menos hambre, ni siquiera en invierno. El hambre de invierno no es mayor que
la de verano (mano 27,7 contra 29,6): lo que falta no es comida en el mundo sino ir a buscarla. Como solo agrega
comida sin arreglar nada medible, queda apagado (el mundo más simple).

**b) La crecida corta el mapa (`CRECIDA`) — apagado (`'vieja'`).** Con lluvia el río puede crecer 300–900 ticks: el
río y sus orillas pasan a peligro de punta a punta. `'vado'`: 3 filas en el medio quedan como estaban. Desde 11000:

| | mano | propia |
|---|---|---|
| % hambre despierta | +0,1 (0,0) | −0,2 (−0,2) |
| peor 10k hambre | +0,1 (0,0) | +0,3 (0,7) |
| % cruda < 0,15 | −0,7 (−0,7) | −0,5 (−0,8) |
| % sed | −1,1 (−1,3) | −0,8 (−1,0) |
| heridas | −9,9 (−1,8) | −1,3 (−0,5) |
| % mal tope | 0 | 0 |

Neutro: la crecida es rara (pocas veces por vida) y cruzar el peligro raspa pero no traba. Queda apagado.

**c) Plantarse en el agua es gratis (`AGUA_FRIA`) — apagado (0).** Parada en el agua bebe cada tick sin costo. Con
`AGUA_FRIA = 0,2` el agua enfría (el aire ahí está 0,2 más frío). Desde 12000:

| | mano | propia |
|---|---|---|
| % plantada | −2,9 (−6,4) | −0,2 (−1,6) |
| % frío | +4,3 (14,4) | +1,6 (8,8) |
| % hambre despierta | +0,4 (0,3) | +1,6 (1,2) |
| peor 10k hambre | −1,2 (−0,7) | +0,8 (1,5) |
| % cruda < 0,15 | +0,8 (0,7) | +1,2 (1,2) |
| % trabadas | −0,4 | +7,0 (1,9) |
| % mal tope | 0 | 0 |

Se planta menos (la mano), pero solo cambia una cosa por otra: pasa más frío y no come mejor; la propia tiende a
empeorar en todo. Plantarse no era una trampa que le quitara tiempo de comer: el costo solo agrega sufrimiento.
Queda apagado.

**d) La cueva no abriga (`CUEVA`) — apagado (`'vieja'`).** La cueva está a 0,6 (menos que el día) y descansar solo
baja el cansancio. `'abriga'`: la cueva está a 0,7 todo el año y descansar despierta ahí baja la adenosina (a la
mitad de lo que la baja dormir). Desde 13000:

| | mano | propia |
|---|---|---|
| % hambre despierta | +0,2 (0,2) | +2,7 (2,2) |
| peor 10k hambre | +0,2 (0,2) | +0,6 (1,0) |
| % cruda < 0,15 | +0,1 (0,1) | +1,8 (2,0) |
| % trabadas | −0,8 | +6,6 (1,9) |
| desmayos | +0,9 | +5,4 (2,1) |
| sueños | −19 (−19,6) | −12 (−10,6) |
| % duerme en cueva | −2,6 (−8,1) | −0,7 (−5,5) |
| % mal tope | 0 | 0 |

En la mano no cambia nada; en la propia empeora (z ~2). Descansar despierta en la cueva le saca el sueño: duerme
menos y casi nunca en la cueva (descansa ahí y después duerme afuera). El abrigo no hacía falta: el frío no es lo
que la traba. Queda apagado.

**e) El desmayo regala (`DESMAYO_RESTO`) — se queda en 0,35.** Al desmayarse despierta con al menos 0,35 de energía y
agua: medio plato gratis. Probado `DESMAYO_RESTO = 0,1` (apenas lo justo para no volver a caer enseguida). Desde 14000:

| | mano | propia |
|---|---|---|
| % hambre despierta | +4,1 (2,8) | +4,8 (3,6) |
| peor 10k hambre | +7,8 (5,2) | +3,9 (7,5) |
| racha de hambre | 2.269 → 18.641 (7,9) | 2.424 → 27.158 (12,0) |
| % cruda < 0,15 | +7,2 (4,9) | +10,4 (9,5) |
| desmayos | 29 → 55 (7,4) | 77 → 128 (12,6) |
| % trabadas | +1,2 | +14,5 (3,6) |
| % mal alto | +5,1 (3,1) | +6,0 (4,3) |
| % mal tope | 0 | 0 |

Sin el regalo arma justo el bucle de sufrimiento que hay que vigilar: despierta casi vacía, no llega a la comida,
vuelve a caer, y la racha de hambre se multiplica por 8–11. El regalo no es una trampa que tape un problema de
aprendizaje: es lo que corta el bucle (decisión del usuario: no muere). Queda en 0,35; no hay código nuevo.

**Paso 3 terminado: nada prendido.** Ninguno de los cuatro cambios del mundo mejora; el último empeora mucho. Lo que
traba a Angri no es el mundo (en invierno no pasa más hambre que en verano, plantarse no le quita comida, el frío no
la traba): es la mente. Sigue el paso 4 (los atajos hechos a mano).

## Reglas del proyecto

- Nada de scripts que le digan qué hacer: las reglas viven dentro del sistema.
- Cada pieza "cerebral" se mide; si no mejora nada, se saca.
- Tope al malestar (`MALESTAR_MAXIMO`) y registro de su interior, siempre.
