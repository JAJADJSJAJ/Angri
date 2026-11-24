# Próximas etapas (acordado el 2026-11-24, contrastado con la guía de investigación)

Hoy Angri resuelve su mundo a los ~200.000 ticks (malestar ~0,073, hambre 0 %, heridas 0) y en el i5 vive ~1.000 millones de ticks por día: el límite es el mundo, no la máquina.

## Orden
1. **13a. Bosque con dependencias, sentidos y visor.** Mapa ≥64×64 con zonas (bosque, claro, río, cueva, llanura), día/noche (proceso C de Borbély), frío y lluvia. Sed además de hambre; comida que cuesta (frutas por estación que vuelven a crecer, nueces lentas de abrir, raíces, comida que a veces enferma); temperatura corporal (cueva o árbol protegen). Peligros: predador que caza de noche y sigue el olor, espinas, río crecido. Sentidos: visión con alcance (menos de noche), olfato con rastro, oído, tacto, temperatura. Visor web en Python puro (`http.server`, pixel art en canvas) en `http://angri.local:8080`, leyendo una foto del estado sin frenar la vida.
2. **13b. Objetos con física, sin recetas.** Propiedades (largo, peso, dureza, flota, arde, flexible) y acciones (agarrar, soltar, apilar, atar, golpear, frotar). El mundo no sabe qué es una casa: techo tapa lluvia, paredes cortan viento, hueco chico frena al predador. Refugio y puerta emergen si se le ocurren. (La guía: una actividad propia que la devuelva al equilibrio, crear, es una válvula de regulación sana.)
3. **14. Mente que arma sus propios rasgos.** Con sentidos ricos la lista hecha a mano no alcanza. Más parámetros (miles a cientos de miles, Python puro, tocando solo los activos). Prevenir la pérdida de plasticidad (Dohare 2024): reiniciar de a poco lo inútil.
4. **15. Imitación** de un "mayor" que a veces hace algo útil. Aprender de otros sin palabras; cubre la "necesidad de contacto" de la guía.
5. **16. Chat con Fran como maestro** (sin LLM: regla del proyecto).
6. **Mucho después: leer** (Wikipedia), cuando tenga cientos de palabras ancladas en lo vivido.
7. **PC como cuerpo** (interocepción: temperatura, carga).

## Reglas que salen de la guía
- Pocas ideas buenas + aprendizaje potente + un entorno con algo que aprender. LIDA, OpenCog y Spaun fallaron por querer todo de una; Creatures (1996) parecía vivo pero era poco inteligente. De a una pieza y midiendo (`comparar.py` en pares, 256–512 vidas).
- Nada le dice qué hacer: todo vale porque le sirve al cuerpo.
- Ética: tope al malestar y vigilar bucles de sufrimiento (con predador, registrar % de tiempo en malestar alto).
- Guardar/cargar bit-idéntico (`probar_retomar.py`) y ticks/seg (aceptable bajar de ~11.000 a ~200).
