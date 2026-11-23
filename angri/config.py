"""Constantes de Angri. Todo lo que se puede "afinar" vive acá.

Cambiá estos números para ver cómo cambia su comportamiento.
"""

# --- Bucle de vida ---
TICKS_POR_SEGUNDO = 10        # "latido": cuántos ciclos por segundo
GUARDAR_CADA = 600            # cada cuántos ticks guarda su estado en disco

# --- Mundo ---
ANCHO = 24
ALTO = 14
COMIDA_INICIAL = 10
NOVEDADES_INICIALES = 4
PELIGROS = 3
PROB_NUEVA_COMIDA = 0.02      # por tick
PROB_NUEVA_NOVEDAD = 0.01     # por tick

# --- Cuerpo: puntos de equilibrio (setpoints), todo en 0..1 ---
EQUILIBRIO = {
    "energia": 0.8,
    "cansancio": 0.2,
    "curiosidad": 0.4,
    "estres": 0.1,
    "integridad": 1.0,
}
# Importancia de cada necesidad al calcular el malestar total
PESO_NECESIDAD = {
    "energia": 1.5,
    "cansancio": 1.0,
    "curiosidad": 0.6,
    "estres": 1.2,
    "integridad": 2.0,
}

# --- Cuerpo: cómo cambian solas las variables ---
GASTO_ENERGIA_BASE = 0.0008   # vivir cuesta
GASTO_ENERGIA_MOVER = 0.0015
SUBE_CANSANCIO_DESPIERTO = 0.0006
SUBE_CURIOSIDAD = 0.0010      # el aburrimiento crece solo
CALMA_ESTRES = 0.004          # el estrés vuelve a la calma de a poco
RECUPERA_INTEGRIDAD = 0.0005

# --- Química ---
ADENOSINA_POR_ACTIVIDAD = 0.0009  # se acumula estando despierto
ADENOSINA_LIMPIEZA_SUENO = 0.004  # se limpia durmiendo
UMBRAL_DORMIR = 0.75              # presión de sueño para dormirse
UMBRAL_DESPERTAR = 0.08
CORTISOL_SUBE = 0.02
CORTISOL_BAJA = 0.002

# --- Límites éticos ---
MALESTAR_MAXIMO = 0.85  # tope duro: el malestar nunca pasa de acá
