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
COMIDA_MAXIMA = 10            # tope de comida en el mundo a la vez
NOVEDAD_MAXIMA = 6            # tope de novedades (sin tope, si las esquiva, tapan el mundo)
ESTACION = 5000               # cada cuántos ticks una fuente se seca y brota en otro lado (0 = fijas)
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
DOLOR = 3.0                   # castigo directo por lastimarse (no pasa por el malestar: no se satura)
RECOMPENSA_ESTADO = 1.0       # peso de "estar mal" (castigo continuo)

# --- Corteza (etapa 3): modelo predictivo y curiosidad ---
CORTEZA = False               # (apagada: medida, no mejora de forma robusta) predice qué encuentra y cómo cambia su malestar
APRENDE_CORTEZA = 0.2
ESCALA_ERROR_MALESTAR = 10.0  # Δmalestar es chico: se agranda para compararlo con el de celdas
CURIOSIDAD_RAPIDO = 0.1       # promedio del error reciente
CURIOSIDAD_LENTO = 0.01       # promedio del error de antes
PESO_CURIOSIDAD = 1.0         # (con CORTEZA) recompensa por progreso de aprendizaje (0 = sin curiosidad)

# --- Hipocampo (etapa 4): memoria del día y sueño ---
HIPOCAMPO = True              # guarda el día y lo repasa al dormir
CAPACIDAD_DIA = 3000          # cuántos momentos del día entran (los más viejos se van)
REPASOS = 2                   # cuántas veces repasa el día por noche
REPASO_ACTOR = True           # el repaso también entrena al actor (no solo al crítico)
APRENDE_REPASO = 0.5          # fracción del ritmo de aprendizaje despierto
UMBRAL_LUGAR = 2.0            # alivio para que un momento marque un lugar
PODA_LUGARES = 0.6            # al dormir, los lugares que no se reconfirmaron se debilitan así
IMPORTANCIA = True            # el repaso pesa cada recuerdo por π_hoy/π_entonces
TOPE_IMPORTANCIA = 1.0        # tope de ese peso (evita que un recuerdo raro explote)
BORRAR_LUGAR_VACIO = True     # llegar a un lugar recordado y no ver nada lo borra
PRIORIDAD_BASE = 0.1          # cuánto pesa en el repaso un momento sin alivio ni dolor
TOPE_LUGAR = 3.0              # fuerza máxima de un lugar recordado

# --- Tálamo (etapa 5): los módulos compiten por la atención ---
TALAMO = False                # (apagado: medido, no mejora) False = atiende siempre a la necesidad más desviada (etapa 4)
GANANCIA_URGENCIA = 3.0       # cuánto pesa lo urgente en la saliencia
TEMPERATURA_FOCO = 0.3        # exploración de la atención
INERCIA_FOCO = 0.5            # ventaja del foco actual (ignición: no picotear entre metas)
APRENDE_TALAMO = 0.02

# --- Lenguaje (etapa 6): un maestro le habla ---
MAESTRO = True                # un maestro que ve todo el mundo le dice dónde está lo que necesita
PROB_HABLA = 0.1              # por tick, si lo que necesita no está a la vista
MEMORIA_PALABRA = 6           # ticks que una palabra oída sigue sonando
MAESTRO_MIENTE = False        # control: dice palabras al azar
PALABRA_SUELTA = False        # rasgo por (palabra, movimiento) (apagado: la palabra de la cosa se vuelve una manía de dirección y empeora)
LEXICO = True                 # significado de cada palabra aprendido por sorpresa (compone)
APRENDE_PALABRA = 0.1 
PISO_PALABRA = 0.25           # nunca aprende menos que esta fracción (anti-ego)
EGO = False                   # prueba: aprende cada vez menos con la experiencia
ESPERA_VERIFICAR = 60         # ticks que espera ver lo que le dijeron para comprobarlo
PASOS_DICHA = 99              # cuántos pasos sigue una dirección dicha antes de soltarla
CERTEZA_DICHA = 0.4           # cuánto tiene que creer en una dirección para seguirla
CONFIANZA_INICIAL = 1.0       # de chico le cree todo al maestro
APRENDE_CONFIANZA = 0.01
CONFIANZA_MINIMA = 0.15       # por debajo deja de hacerle caso
CAMBIO_SIGNIFICADO = 0        # tick en que las palabras de dirección cambian de sentido (0 = nunca)
COMPOSICION = 0               # prueba: hasta este tick el maestro solo habla de comida, después solo de la cama (0 = apagado)
HOLISTICO = False             # control: aprende cada frase entera como un todo (no puede componer)
RITMO_MAXIMO = 0.9            # tope del ritmo de aprendizaje de una palabra
SORPRESA_RITMO = 0.0          # aprende más rápido cuando viene sorprendido (x(1 + k·sorpresa reciente))
# --- Etapa 7: Angri pide ---
PIDE = False                  # Angri puede decir una palabra cuando no ve lo que necesita
PIDE_AZAR = False             # control: elige la palabra al azar y no aprende
MAESTRO_SOLO_SI_PIDE = False  # el maestro solo habla cuando Angri le pide
TEMPERATURA_HABLA = 1.0
APRENDE_HABLA = 0.02
VENTANA_HABLA = 30            # ticks de dopamina que se le acreditan a lo que dijo
FRASE = False                 # rasgo por (par de palabras oídas juntas, movimiento)

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
