# Etapa 2: que aprenda con dopamina

Qué hay que hacer:

- Reemplazar la elección al azar de `angri/mente.py` por un **actor-crítico**:
  - **Crítico:** aprende a predecir cuánto malestar le espera desde cada situación.
  - **Actor:** aprende qué movimiento conviene en cada situación.
- La **dopamina** es el error TD (lo que mejoró el equilibrio menos lo que esperaba) y controla cuánto aprenden ambos. `quimica.py` ya la calcula de forma simple.
- La **noradrenalina** sube la exploración cuando hay sorpresa.
- La **serotonina** (nueva) define cuánto mira al futuro: el factor de descuento.

Para empezar alcanza con tablas (Q-learning tabular) o una red chiquita en NumPy. Todavía no hace falta PyTorch.

Cómo medir que funciona: comparar el malestar promedio y las veces que se lastima con la etapa 1 (mente al azar) en 50.000 ticks. Tiene que mejorar.
