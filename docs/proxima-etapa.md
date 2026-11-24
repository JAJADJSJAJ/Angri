# Próximas etapas (acordado el 2026-11-24, contrastado con la guía de investigación)

Hoy Angri resuelve su mundo a los ~200.000 ticks (malestar ~0,073, hambre 0 %, heridas 0) y en el i5 vive ~1.000 millones de ticks por día: el límite es el mundo, no la máquina.

## Estado (actualizado el 2026-11-24)
- 13a (bosque, sentidos, visor) hecha. Angri pasó entera a Rust (misma vida bit a bit, ~60× más rápido) y Python se borró.
- En el i5 quedó instalada la versión en Rust (renació) y **apagada a propósito**: no vive ni evoluciona hasta que Angri esté completa. `instalar.sh` no la vuelve a prender sola.

## Orden
1. **13b. Objetos con física, sin recetas.** Propiedades (largo, peso, dureza, flota, arde, flexible) y acciones (agarrar, soltar, apilar, atar, golpear, frotar). El mundo no sabe qué es una casa: techo tapa lluvia, paredes cortan viento, hueco chico frena al predador. Refugio y puerta emergen si se le ocurren. (La guía: una actividad propia que la devuelva al equilibrio, crear, es una válvula de regulación sana.) Va antes que la 14 porque le da a la mente algo que los rasgos hechos a mano no pueden ver: así la 14 se mide contra un problema real.
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
