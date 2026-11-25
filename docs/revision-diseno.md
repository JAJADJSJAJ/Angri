# Revisión del diseño (2026-11-25)

Antes de seguir con la etapa 14 revisamos cómo está armada Angri por dentro. Lo encontramos con 3 revisiones
(cuerpo, mente y mundo) después de ver que las vidas del bosque se trababan con hambre. Ya arreglado:
`DESMAYO_DONDE = 'aqui'` (el desmayo la llevaba a la misma cueva una y otra vez).

Regla para todo lo que sigue: de a un cambio, `angri comparar` con 256 pares y `--desde` distinto, y si no mejora se apaga.

## 0. Métricas que no mientan (no cambian la vida) — HECHO (ver README, «Paso 0: métricas»)
- `% trabadas` es todo o nada (más del 60 % de la vida con hambre): una vida con 59 % no cuenta. Reemplazar por la
  peor ventana de 10.000 ticks y la racha de hambre más larga.
- `% hambre` cuenta también el sueño: separar despierta y dormida.
- `% mal alto` depende de los pesos: sumar "alguna necesidad cruda por debajo de 0,15", que no depende de los pesos.
- Hambre por estación (el invierno no se ve), y % del tiempo con el malestar clavado en el tope (0,85).

## 1. Trampas de la mente (cambios chicos)
- **Foco ciego que la deja quieta**: con el foco en integridad (o estrés) `alivia` da "nada" y `hacia` devuelve
  "quedarse" (`mente.rs:268`): el rasgo "voy hacia lo que alivia" empuja a no moverse mientras se muere de hambre.
  **Hecho (paso 1a)**: `FOCO_CIEGO = 'neutro_todo'` (sin alivio, o sin alivio a la vista, el rasgo no empuja); mano: cruda<0,15 −4 puntos (z −3,6), ver README.
- **Explora más cuanto peor está**: la temperatura es `0,3 + 0,5·nora + 8·malestar` (`mente.rs:508`); con malestar
  0,3 elige casi al azar. Cuanta más hambre, menos va derecho a la comida.
  **Medido (paso 1b), apagado**: sacarla o acotarla empeora todo (sin término: cruda<0,15 +45 puntos en la mano); el ruido es lo que la destraba hoy. Ver README.
- **El desmayo es un borrón**: `olvidar_paso` antes de aprender (`vida.rs:459`), así lo que la llevó a caer nunca
  recibe el golpe. Hacerlo un final de episodio (V(s2) = 0 con el malestar de antes).
  **Medido (paso 1c), apagado** (`DESMAYO_FINAL`): mano −2 puntos de cruda<0,15 (z −1,6), propia +2,7 (z 3,3). Ver README.

## 2. El cuerpo y lo que siente
- **El crítico está partido por el foco** (la clave `s` lleva la necesidad del foco): cada salto de foco cambia de
  tabla y ensucia el crédito. Y **no ve agua, calor, cansancio ni estrés**. Que vea el cuerpo entero (franjas) y no
  el foco: V = "cuánto malestar total viene" (la idea de Fran: estar bien es que no duela nada).
  **Hecho (paso 2a)**: `CRITICO_CUERPO = 2` (cada necesidad bien/lejos, sin foco), solo con la mano: cruda<0,15 −5,4
  (z −5,4), hambre −7, trabadas a la mitad (512 pares). Con la propia empeora (+2,2, z 3,5): se ignora. Ver README.
- **El estrés cobra doble**: sube con hambre, sed, frío y heridas, y además pesa por su cuenta; nada lo alivia directo.
  **Hecho (paso 2b)**: `ESTRES_COBRA = 'sin_peso'` (sigue moviendo el cortisol, no pesa en el malestar): cruda<0,15
  −4,2 mano / −4,3 propia (z −4,9 / −6,5, 512 pares); a cambio, algo más de mordidas (mano) y heridas (propia).
  `'solo_golpes'` no cambia nada. Ver README.
- (latente) el tálamo solo mira 5 necesidades, no agua ni calor. Anotado: está apagado (`TALAMO = False`), no afecta hoy.

## 3. El mundo
- **Invierno**: no brotan fruta ni baya y lo que había se borra en ~2.000 ticks; quedan nueces (hay que golpearlas)
  y raíces (todas del otro lado del río). 3 de cada 5 brotes son fruta o baya y en invierno no hacen nada.
  **Medido (paso 3a), apagado** (`INVIERNO = 'repartido'`): neutro en las dos mentes; el hambre de invierno no es mayor que la de verano.
- **La crecida corta el mapa** (río más orillas en peligro 300–900 ticks).
  **Medido (paso 3b), apagado** (`CRECIDA = 'vado'`): neutro en las dos mentes (la crecida es rara).
- **Plantarse en el agua es gratis** (bebe cada tick sin costo): 22 % del tiempo plantada.
  **Medido (paso 3c), apagado** (`AGUA_FRIA = 0.2`): mano plantada −2,9 puntos pero frío +4,3 y nada mejora; propia algo peor.
- **La cueva no abriga** (0,6, menos que el día) y descansar casi no hace nada.
- **El desmayo regala** 0,35 de energía y agua: medio plato gratis.

## 4. Atajos hechos a mano (la etapa 14 de verdad)
`alivia()` (hambre→comida, sed→agua, frío→descanso), el rasgo fijo "voy hacia la comida" (`dc`), el bit "veo comida"
en el crítico y los lugares por necesidad. Sacarlos de a uno en la mano mide cuánto "regala la receta": esa es la vara
para la propia. En la propia: un solo crítico (hoy suma la V de la tabla y la de las unidades, que persiguen el mismo
error), distancia más fina y peso por "me acerco / me alejo", huellas más largas (λ 0,9, γ 0,97: hoy ~4 pasos).

## Orden
0 → 1 → 2 → 3 → 4. Primero que vea y mida bien; después que no se trabe; después que el mundo sea justo; recién
ahí sacarle las recetas y ver cómo aprende sola.
