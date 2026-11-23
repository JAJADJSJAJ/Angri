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
PELIGROS = 5
PROB_NUEVA_COMIDA = 0.02      # por tick
PROB_NUEVA_NOVEDAD = 0.01     # por tick
FUENTES_COMIDA = 3            # lugares fijos donde crece la comida (como árboles)
RADIO_FUENTE = 2              # la comida aparece a esta distancia de una fuente
COMIDA_MAXIMA = 10
NOVEDAD_MAXIMA = 6            # tope de novedades (sin tope, si las esquiva, tapan el mundo)            # tope de comida en el mundo a la vez
PROB_MUEVE_PELIGRO = 0.02     # por tick: un peligro da un paso

# --- Mente ---
MENTE = "aprende"             # "aprende" (etapa 2), o "azar" (vieja mente.py)
REFLEJOS = False              # solo para MENTE="azar": False = azar puro (el piso)

# --- Ganglios basales (etapa 2) ---
SEROTONINA = 0.95             # γ: cuánto mira al futuro
APRENDE_CRITICO = 0.1
APRENDE_ACTOR = 0.06
HUELLA = 0.8                  # λ: cuánto tiempo atrás llega el crédito de la dopamina
CALMA = 1e12                  # experiencia a partir de la cual aprende más despacio (1e12 = apagado: probado con 2000, empeora)
APRENDE_MINIMO = 0.1          # nunca aprende menos que esta fracción (el mundo cambia)
UMBRAL_RECUERDO = 1e9         # alivio para recordar un lugar (1e9 = apagado: probado con 2.0, empeora)
RECUERDO_DECAE = 0.9995       # por tick: los recuerdos se desvanecen
TEMPERATURA_BASE = 0.3        # exploración de fondo
TEMPERATURA_SORPRESA = 0.5    # la noradrenalina sube la exploración
TEMPERATURA_MALESTAR = 2.0    # cuanto peor la pasa, más explora
RECOMPENSA_ALIVIO = 50.0      # peso de "el malestar bajó"
RECOMPENSA_ESTADO = 1.0       # peso de "estar mal" (castigo continuo)

# --- Corteza (etapa 3): modelo predictivo y curiosidad ---
CORTEZA = False               # (apagada: medida, no mejora de forma robusta) predice qué encuentra y cómo cambia su malestar
APRENDE_CORTEZA = 0.2
ESCALA_ERROR_MALESTAR = 10.0  # Δmalestar es chico: se agranda para compararlo con el de celdas
CURIOSIDAD_RAPIDO = 0.1       # promedio del error reciente
CURIOSIDAD_LENTO = 0.01       # promedio del error de antes
PESO_CURIOSIDAD = 1.0         # (con CORTEZA) recompensa por progreso de aprendizaje (0 = sin curiosidad)

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
DANO_INANICION = 0.0          # sin energía, se consume (0 = apagado: probado, empeora)

# --- Química ---
ADENOSINA_POR_ACTIVIDAD = 0.0009  # se acumula estando despierto
ADENOSINA_LIMPIEZA_SUENO = 0.004  # se limpia durmiendo
UMBRAL_DORMIR = 0.75              # presión de sueño para dormirse
UMBRAL_DESPERTAR = 0.08
CORTISOL_SUBE = 0.02
CORTISOL_BAJA = 0.002

# --- Límites éticos ---
MALESTAR_MAXIMO = 0.85  # tope duro: el malestar nunca pasa de acá
