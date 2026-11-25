# Próximas etapas (acordado el 2026-11-24, contrastado con la guía de investigación)

Hoy Angri resuelve su mundo a los ~200.000 ticks (malestar ~0,073, hambre 0 %, heridas 0) y en el i5 vive ~1.000 millones de ticks por día: el límite es el mundo, no la máquina.

## Estado (actualizado el 2026-11-24)
- 13a (bosque, sentidos, visor) hecha.
- **13b (objetos con física) hecha** (2026-11-24, ver el README): con `OBJETOS = True` usa los objetos (lleva cosas, apila, abre nueces a golpes, alguna viga, algún fuego) sin empeorar (malestar −0,008 ± 0,010; heridas y mordidas bajan), pero **no arma refugio**: la cadena es larga para la mente de hoy. Queda apagada por defecto; es el problema real para la 14.
- **14 (mente que arma sus rasgos) empezada** (2026-11-24, ver el README, `rust/src/propia.rs`, `RASGOS = 'propios'`): átomos crudos del estado y de cada movimiento, un banco de 4.096 conjunciones que nacen de lo vivido y se reinician de a poco. Aprende (grilla: 0,168 contra 0,116 de la mano) y en el bosque con objetos **duerme en refugios propios 12 % contra 3 %**, pero come mucho menos (malestar 0,516 contra 0,395). Segunda vuelta (medida en el i5 por SSH): el sueño repasa las unidades (ayuda; prendido); el ritmo por unidad no cambia nada (apagado). **En vidas de 1 M de ticks la propia mejora con los años y la mano no: la diferencia baja de 0,089 a ~0,04 (ya no clara) y duerme en refugio propio 29 % contra 3 %.** Tercera vuelta: índice por átomo más raro (+25–35 % más rápida, misma vida); barrido de 7 ajustes en el i5, ninguno mejora (el banco de 16.384 empeora). Cuarta vuelta: −30 % de tiempo en el i5 con la misma vida (forma compacta, el sueño busca una vez por recuerdo, reusa las activas de s2); con 4 hilos, 4 vidas de 200.000 ticks tardan 19 s. Siguiente: vidas de 3–5 M de ticks en el i5. Falta: que coma como la mano (aprender de lo que predice, vidas de varios millones). Angri pasó entera a Rust (misma vida bit a bit, ~60× más rápido) y Python se borró.
- En el i5 quedó **apagada a propósito** (`systemctl disable --now angri`): no vive ni evoluciona hasta que Angri esté completa. Ya tiene la versión en Rust instalada (`/opt/angri/angri`), sin nacer; la de Python (81.862.072 ticks) quedó en `/var/lib/angri/python-viejo/`. `instalar.sh` no la prende sola; para prenderla: `sudo systemctl enable --now angri angri-visor`.

## Orden
1. ~~**13b. Objetos con física, sin recetas.**~~ (hecha) Propiedades (largo, peso, dureza, flota, arde, flexible) y acciones (agarrar, soltar, apilar, atar, golpear, frotar). El mundo no sabe qué es una casa: techo tapa lluvia, paredes cortan viento, hueco chico frena al predador. Refugio y puerta emergen si se le ocurren. (La guía: una actividad propia que la devuelva al equilibrio, crear, es una válvula de regulación sana.) Va antes que la 14 porque le da a la mente algo que los rasgos hechos a mano no pueden ver: así la 14 se mide contra un problema real.
2. **14. Mente que arma sus propios rasgos.** Con sentidos ricos y objetos la lista hecha a mano no alcanza. Más parámetros (miles a cientos de miles; con Rust entran), tocando solo los activos. Prevenir la pérdida de plasticidad (Dohare 2024): reiniciar de a poco lo inútil.
3. **El mapa de Fran: avatar en el visor.** Moverse por el mapa, darle comida, decirle palabras: el maestro pasa a ser una persona, sin modelos externos (en `visor.rs`/`visor.html` y un canal de entrada a `angri vivir`).
4. **15. Imitación** de un "mayor" que a veces hace algo útil. Aprender de otros sin palabras; cubre la "necesidad de contacto" de la guía.
5. **16. Chat con Fran como maestro** (sin LLM: regla del proyecto).
6. Mundo más grande (256×256, más animales).
7. **Mucho después: leer** (Wikipedia), cuando tenga cientos de palabras ancladas en lo vivido.
8. **PC como cuerpo** (interocepción: temperatura, carga).

## Reglas que salen de la guía
- Pocas ideas buenas + aprendizaje potente + un entorno con algo que aprender. LIDA, OpenCog y Spaun fallaron por querer todo de una; Creatures (1996) parecía vivo pero era poco inteligente. De a una pieza y midiendo (`angri comparar` en pares, 256–512 vidas).
- Nada le dice qué hacer: todo vale porque le sirve al cuerpo.
- Ética: tope al malestar y vigilar bucles de sufrimiento (con predador, registrar % de tiempo en malestar alto).
- Guardar/cargar bit-idéntico (`cargo test --release`: retomar y referencia) y ticks/seg (hoy ~540.000; aceptable bajar mucho si la mente lo pide).
