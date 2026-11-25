# Angri — guía para Claude

Angri es un "cerebro artificial que vive solo". Hablale al usuario en español rioplatense y explicale en
simple el **por qué** de lo que pasa adentro de Angri: es lo que más le gusta.

## Antes de tocar nada, leé

1. `docs/proxima-etapa.md`: estado actual, qué sigue y en qué orden, y el estado del i5.
2. `README.md`: cómo correrlo, la estructura y la historia etapa por etapa (qué se probó, qué se prendió,
   qué se apagó y por qué). Es largo: leé "Cómo correrlo", "Estructura" y la última etapa, y buscá el resto
   cuando lo necesites.
3. `rust/config.txt`: todos los números que se pueden afinar, cada uno con su comentario.

## Reglas (no se negocian)

- **Angri no sabe nada de antemano.** No sabe qué es la comida ni el peligro. Todo lo aprende de su propio
  malestar (la dopamina es el error TD). Nada de scripts ni recetas que le digan qué hacer, y nada de LLM
  externos como maestro.
- **Cada pieza se mide, y si no mejora, se apaga** y se anota en el README con sus números. Se mide en pares
  con `angri comparar` y muchas vidas (lo de 8 vidas ya dio mejoras falsas).
- Tope al malestar (`MALESTAR_MAXIMO`) y registro de su interior, siempre. Vigilar los bucles de sufrimiento.
- Guardar y retomar tienen que dar la misma vida bit a bit: `cargo test --release` antes de cada commit. Si
  cambia la vida, se regenera `rust/pruebas/referencia.txt` a propósito y se dice en el commit.

## Cómo se trabaja

- Todo en Rust, en `rust/`, sin dependencias. Python se borró (está en git hasta `f3ce27d`).
- Al terminar cada paso: actualizar `README.md` (su sección de etapa) y `docs/proxima-etapa.md` (estado y
  orden), y hacer commit en `main` terminado en `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- GitHub: repo público `JAJADJSJAJ/Angri`, con `main` protegida. Nunca force-push.
- i5: la vida que corre ahí está **apagada a propósito** hasta que Angri esté completa. No la prendas sin que
  el usuario lo pida (cómo prenderla está en `docs/proxima-etapa.md`).
