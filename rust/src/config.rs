//! La config de angri/config.py, que Python manda entera como líneas CLAVE=valor
//! (repr de Python; los dicts van como CLAVE.sub=valor).
use std::collections::HashMap;

pub struct Config {
    pub ancho: i64, pub alto: i64, pub comida_inicial: i64, pub novedades_iniciales: i64,
    pub peligros: i64, pub prob_nueva_comida: f64, pub prob_nueva_novedad: f64,
    pub fuentes_comida: i64, pub radio_fuente: i64, pub comida_maxima: i64,
    pub novedad_maxima: i64, pub estacion: i64, pub prob_mueve_peligro: f64,
    pub bosque: bool, pub ticks_por_segundo: i64, pub bosque_ancho: i64, pub bosque_alto: i64, pub dia: i64,
    pub fraccion_noche: f64, pub estacion_bosque: i64, pub vista_dia: i64, pub vista_noche: i64,
    pub olfato: i64, pub humedad: i64, pub olor_predador: i64, pub oido_predador: i64,
    pub abundancia: f64, pub charcos: i64, pub prob_brote: f64, pub nuez_ticks: i64,
    pub raiz_ticks: i64, pub prob_enferma: f64, pub prob_lluvia: f64, pub prob_crecida: f64,
    pub temp_dia: f64, pub temp_cueva: f64, pub frio_noche: f64, pub frio_lluvia: f64,
    pub predador_dia: f64, pub predador_noche: f64, pub predador_harto: i64,
    pub rastro_decae: f64, pub rastro_dura: i64, pub sueno_noche: f64,
    pub desmayo_donde: String, pub desmayo_integridad: f64, pub desmayo_resto: f64,
    pub malestar_alto: f64,
    pub serotonina: f64, pub aprende_critico: f64, pub aprende_actor: f64, pub huella: f64,
    pub franjas: i64, pub calma: f64, pub aprende_minimo: f64, pub umbral_recuerdo: f64,
    pub recuerdo_decae: f64, pub temperatura_base: f64, pub temperatura_sorpresa: f64,
    pub temperatura_malestar: f64, pub margen_explorar: f64, pub peligro_sin_malestar: bool, pub foco_ciego: u8, pub temperatura_malestar_tope: f64, pub desmayo_final: bool, pub critico_cuerpo: i64, pub estres_cobra: u8, pub invierno: u8, pub vado: bool, pub agua_fria: f64, pub cueva_abriga: bool,
    pub atajo_alivia: bool, pub atajo_comida: bool, pub atajo_veo_comida: bool, pub atajo_lugares: bool, pub critico_propio: u8, pub distancia_fina: bool, pub actor_por_activos: bool, pub decae_actor: f64, pub decae_actor_modo: u8, pub decae_solo_positivos: bool, pub decae_propio: f64, pub ancla_v: f64, pub olvidar_ultimo: bool, pub tope_v_propio: f64,
    pub recompensa_alivio: f64, pub dolor: f64, pub recompensa_estado: f64,
    pub hipocampo: bool, pub capacidad_dia: i64, pub repasos: i64, pub repaso_actor: bool,
    pub aprende_repaso: f64, pub umbral_lugar: f64, pub poda_lugares: f64, pub importancia: bool,
    pub tope_importancia: f64, pub borrar_lugar_vacio: bool, pub prioridad_base: f64,
    pub tope_lugar: f64,
    pub maestro: bool, pub prob_habla: f64, pub lexico: bool, pub aprende_palabra: f64,
    pub piso_palabra: f64, pub ego: bool, pub espera_verificar: i64, pub pasos_dicha: i64,
    pub certeza_dicha: f64, pub confianza_inicial: f64, pub aprende_confianza: f64,
    pub confianza_minima: f64, pub ritmo_maximo: f64, pub sorpresa_ritmo: f64,
    pub maestro_solo_si_pide: bool, pub maestro_inicio: i64, pub prob_espontaneo: f64,
    pub maestro_ciego: bool,
    pub equilibrio: [f64; 7], pub peso: [f64; 7],
    pub gasto_energia_base: f64, pub gasto_energia_mover: f64, pub sube_cansancio_despierto: f64,
    pub sube_curiosidad: f64, pub calma_estres: f64, pub recupera_integridad: f64,
    pub dano_inanicion: f64, pub gasto_agua: f64, pub beber: f64, pub templa: f64,
    pub dano_frio: f64, pub dano_sed: f64,
    pub adenosina_por_actividad: f64, pub adenosina_limpieza_sueno: f64, pub umbral_dormir: f64,
    pub umbral_despertar: f64, pub cortisol_sube: f64, pub cortisol_baja: f64,
    pub malestar_maximo: f64,
    // mente vieja (mente.py)
    pub mente_azar: bool, pub reflejos: bool,
    // corteza (etapa 3)
    pub corteza: bool, pub aprende_corteza: f64, pub escala_error_malestar: f64,
    pub curiosidad_rapido: f64, pub curiosidad_lento: f64, pub peso_curiosidad: f64,
    // tálamo (etapa 5)
    pub talamo: bool, pub ganancia_urgencia: f64, pub temperatura_foco: f64,
    pub inercia_foco: f64, pub aprende_talamo: f64,
    // lenguaje
    pub palabra_suelta: bool, pub frase: bool, pub memoria_palabra: i64, pub maestro_miente: bool,
    pub holistico: bool, pub cambio_significado: i64, pub composicion: i64,
    pub pide: bool, pub pide_azar: bool, pub temperatura_habla: f64, pub aprende_habla: f64,
    pub ventana_habla: i64, pub escucha: bool, pub escucha_cuerpo: bool, pub escucha_pedidos: bool,
    pub aprende_escucha: f64, pub temperatura_escucha: f64, pub umbral_escucha: f64,
    pub escucha_oido: bool, pub escucha_alivio: bool, pub ventana_alivio: i64, pub umbral_alivio: f64,
    // objetos con física (etapa 13b)
    pub objetos: bool, pub objetos_iniciales: i64, pub objetos_maximo: i64, pub prob_objeto: f64,
    pub fuerza: f64, pub calor_fuego: f64, pub corta_viento: f64, pub prob_prender: f64, pub fuego_ticks: i64,
    // mente que arma sus rasgos (etapa 14)
    pub propia: bool, pub rasgos_mano: bool, pub rasgo_alrededor: bool, pub propios: i64, pub nuevos_por_tick: i64,
    pub reemplazo_cada: i64, pub reemplazo_fraccion: f64, pub madurez_propio: i64,
    pub aprende_propio_actor: f64, pub aprende_propio_critico: f64, pub tope_activa: f64, pub ritmo_raro: f64,
    pub repaso_propio: bool, pub nacer_sorpresa: bool, pub repaso_propio_cada: i64, pub predice_propio: f64, pub predice_actor: f64, pub predice_todas: i64, pub critico_cuerpo_propio: i64, pub salida_propia: i64, pub salida_umbral: f64, pub salida_debug: bool, pub salida_lejos: i64, pub salida_lejos_peso: f64, pub critico_vecinos: bool,
}

/// Claves que pueden faltar (configs viejas): valen lo de por defecto.
const OPCIONALES: [(&str, &str); 56] = [
    ("CRITICO_VECINOS", "False"), ("RASGO_ALREDEDOR", "False"),
    ("RITMO_RARO", "1"), ("REPASO_PROPIO", "True"), ("NACER_SORPRESA", "False"), ("REPASO_PROPIO_CADA", "1"), ("PREDICE_PROPIO", "0"), ("PREDICE_ACTOR", "0"), ("PREDICE_TODAS", "0"), ("CRITICO_CUERPO_PROPIO", "0"), ("SALIDA_PROPIA", "0"), ("SALIDA_UMBRAL", "0.03"), ("SALIDA_DEBUG", "False"), ("SALIDA_LEJOS", "0"), ("SALIDA_LEJOS_PESO", "1"),
    ("TOPE_ACTIVA", "0.2"),
    ("RASGOS", "'mano'"), ("PROPIOS", "4096"), ("NUEVOS_POR_TICK", "4"), ("REEMPLAZO_CADA", "100"),
    ("REEMPLAZO_FRACCION", "0.01"), ("MADUREZ_PROPIO", "3000"), ("APRENDE_PROPIO_ACTOR", "0.2"),
    ("APRENDE_PROPIO_CRITICO", "0.5"),
    ("OBJETOS", "False"), ("OBJETOS_INICIALES", "90"), ("OBJETOS_MAXIMO", "160"), ("PROB_OBJETO", "0.01"),
    ("FUERZA", "1.0"), ("CALOR_FUEGO", "0.3"), ("CORTA_VIENTO", "0.04"), ("PROB_PRENDER", "0.15"),
    ("FUEGO_TICKS", "120"), ("FOCO_CIEGO", "'quedarse'"), ("DESMAYO_FINAL", "False"), ("TEMPERATURA_MALESTAR_TOPE", "0"), ("CRITICO_CUERPO", "0"), ("ESTRES_COBRA", "'doble'"), ("INVIERNO", "'viejo'"), ("CRECIDA", "'vieja'"), ("AGUA_FRIA", "0"), ("CUEVA", "'vieja'"),
    ("ATAJO_ALIVIA", "True"), ("ATAJO_COMIDA", "True"), ("ATAJO_VEO_COMIDA", "True"), ("ATAJO_LUGARES", "True"), ("CRITICO_PROPIO", "'ambos'"), ("DISTANCIA_PROPIA", "'vieja'"), ("ACTOR_POR_ACTIVOS", "False"), ("DECAE_ACTOR", "0"), ("DECAE_ACTOR_MODO", "'reloj'"), ("DECAE_SOLO_POSITIVOS", "False"), ("DECAE_PROPIO", "0"), ("ANCLA_V", "0"), ("OLVIDAR_ULTIMO", "False"), ("TOPE_V_PROPIO", "0"),
];

pub const NOMBRES: [&str; 7] = ["energia", "cansancio", "curiosidad", "estres", "integridad", "agua", "calor"];

/// El valor sin el comentario del final (un '#' fuera de comillas).
fn sin_comentario(v: &str) -> &str {
    let mut comilla: Option<char> = None;
    for (i, ch) in v.char_indices() {
        match (comilla, ch) {
            (None, '\'' | '"') => comilla = Some(ch),
            (Some(c), _) if c == ch => comilla = None,
            (None, '#') => return v[..i].trim(),
            _ => {}
        }
    }
    v.trim()
}

/// Una config armada: la de por defecto (o un archivo) con ajustes encima.
pub fn armar(archivo: Option<&str>, ajustes: &[String]) -> (String, Config) {
    let texto = match archivo {
        Some(a) => std::fs::read_to_string(a).unwrap_or_else(|e| {
            eprintln!("angri (rust): no puedo leer {a} ({e})");
            std::process::exit(2)
        }),
        None => Config::texto_por_defecto().to_string(),
    };
    let c = Config::leer_con(&texto, ajustes);
    // el texto con los ajustes, para guardarlo (vivir) y que la vida siga igual
    let mut t = texto;
    if !ajustes.is_empty() {
        t += "\n# ajustes\n";
        for a in ajustes {
            t += a;
            t += "\n";
        }
    }
    (t, c)
}

struct Crudo(HashMap<String, String>);

impl Crudo {
    fn s(&self, k: &str) -> &str {
        self.0.get(k).map(|s| s.as_str()).unwrap_or_else(|| panic!("falta {k} en la config"))
    }
    fn f(&self, k: &str) -> f64 {
        let v = self.s(k);
        match v {
            "True" => 1.0,
            "False" => 0.0,
            _ => v.parse().unwrap_or_else(|_| panic!("{k}={v} no es número")),
        }
    }
    fn i(&self, k: &str) -> i64 {
        let v = self.s(k);
        match v {
            "True" => 1,
            "False" => 0,
            _ => v.parse().unwrap_or_else(|_| panic!("{k}={v} no es entero")),
        }
    }
    fn b(&self, k: &str) -> bool {
        match self.s(k) {
            "True" => true,
            "False" => false,
            v => v.parse::<f64>().map(|x| x != 0.0).unwrap_or(!v.is_empty()),
        }
    }
    fn txt(&self, k: &str) -> String {
        self.s(k).trim_matches(|c| c == '\'' || c == '"').to_string()
    }
}

impl Config {
    /// La config de rust/config.txt (metida en el binario al compilar).
    pub fn texto_por_defecto() -> &'static str {
        include_str!("../config.txt")
    }

    pub fn leer(texto: &str) -> Config {
        Config::leer_con(texto, &[])
    }

    /// Lee la config (líneas `CLAVE = valor  # comentario`) y le aplica ajustes
    /// `CLAVE=valor` encima. Un ajuste con una clave que no existe es un error.
    pub fn leer_con(texto: &str, ajustes: &[String]) -> Config {
        let mut m = HashMap::new();
        for l in texto.lines() {
            if l.trim_start().starts_with('#') {
                continue;
            }
            if let Some((k, v)) = l.split_once('=') {
                m.insert(k.trim().to_string(), sin_comentario(v).to_string());
            }
        }
        for (k, v) in OPCIONALES {
            m.entry(k.to_string()).or_insert_with(|| v.to_string());
        }
        for a in ajustes {
            let Some((k, v)) = a.split_once('=') else {
                eprintln!("angri (rust): ajuste sin '=': {a}");
                std::process::exit(2);
            };
            if !m.contains_key(k.trim()) {
                eprintln!("angri (rust): {} no existe en la config", k.trim());
                std::process::exit(2);
            }
            m.insert(k.trim().to_string(), sin_comentario(v).to_string());
        }
        let c = Crudo(m);
        let rasgos = c.txt("RASGOS");
        if !["mano", "ambos", "propios"].contains(&rasgos.as_str()) {
            eprintln!("angri (rust): RASGOS={rasgos} no existe (mano, ambos o propios)");
            std::process::exit(2);
        }
        let mente = c.txt("MENTE");
        if mente != "aprende" && mente != "azar" {
            eprintln!("angri (rust): MENTE={mente} no existe");
            std::process::exit(2);
        }
        if c.b("PIDE") && c.b("MAESTRO") && c.txt("MUNDO") == "bosque" {
            // (en Python tampoco anda: Habla usa EQUILIBRIO, que no tiene agua ni calor)
            eprintln!("angri (rust): PIDE no funciona en el bosque");
            std::process::exit(2);
        }
        let mut equilibrio = [0.0; 7];
        let mut peso = [0.0; 7];
        for (i, n) in NOMBRES.iter().enumerate() {
            equilibrio[i] = if i < 5 { c.f(&format!("EQUILIBRIO.{n}")) } else { c.f(&format!("EQUILIBRIO_BOSQUE.{n}")) };
            peso[i] = c.f(&format!("PESO_NECESIDAD.{n}"));
        }
        let estres_cobra = match c.txt("ESTRES_COBRA").as_str() { "doble" => 0, "sin_peso" => 1, "solo_golpes" => 2, x => { eprintln!("angri (rust): ESTRES_COBRA={x} no existe"); std::process::exit(2) } };
        if estres_cobra == 1 { peso[3] = 0.0; }
        Config {
            ancho: c.i("ANCHO"), alto: c.i("ALTO"), comida_inicial: c.i("COMIDA_INICIAL"),
            novedades_iniciales: c.i("NOVEDADES_INICIALES"), peligros: c.i("PELIGROS"),
            prob_nueva_comida: c.f("PROB_NUEVA_COMIDA"), prob_nueva_novedad: c.f("PROB_NUEVA_NOVEDAD"),
            fuentes_comida: c.i("FUENTES_COMIDA"), radio_fuente: c.i("RADIO_FUENTE"),
            comida_maxima: c.i("COMIDA_MAXIMA"), novedad_maxima: c.i("NOVEDAD_MAXIMA"),
            estacion: c.i("ESTACION"), prob_mueve_peligro: c.f("PROB_MUEVE_PELIGRO"),
            bosque: c.txt("MUNDO") == "bosque", bosque_ancho: c.i("BOSQUE_ANCHO"),
            ticks_por_segundo: c.i("TICKS_POR_SEGUNDO"),
            bosque_alto: c.i("BOSQUE_ALTO"), dia: c.i("DIA"), fraccion_noche: c.f("FRACCION_NOCHE"),
            estacion_bosque: c.i("ESTACION_BOSQUE"), vista_dia: c.i("VISTA_DIA"),
            vista_noche: c.i("VISTA_NOCHE"), olfato: c.i("OLFATO"), humedad: c.i("HUMEDAD"),
            olor_predador: c.i("OLOR_PREDADOR"), oido_predador: c.i("OIDO_PREDADOR"),
            abundancia: c.f("ABUNDANCIA"), charcos: c.i("CHARCOS"), prob_brote: c.f("PROB_BROTE"),
            nuez_ticks: c.i("NUEZ_TICKS"), raiz_ticks: c.i("RAIZ_TICKS"),
            prob_enferma: c.f("PROB_ENFERMA"), prob_lluvia: c.f("PROB_LLUVIA"),
            prob_crecida: c.f("PROB_CRECIDA"), temp_dia: c.f("TEMP_DIA"), temp_cueva: c.f("TEMP_CUEVA"),
            frio_noche: c.f("FRIO_NOCHE"), frio_lluvia: c.f("FRIO_LLUVIA"),
            predador_dia: c.f("PREDADOR_DIA"), predador_noche: c.f("PREDADOR_NOCHE"),
            predador_harto: c.i("PREDADOR_HARTO"), rastro_decae: c.f("RASTRO_DECAE"),
            rastro_dura: c.i("RASTRO_DURA"), sueno_noche: c.f("SUENO_NOCHE"),
            desmayo_donde: c.txt("DESMAYO_DONDE"), desmayo_integridad: c.f("DESMAYO_INTEGRIDAD"),
            desmayo_resto: c.f("DESMAYO_RESTO"), malestar_alto: c.f("MALESTAR_ALTO"),
            serotonina: c.f("SEROTONINA"), aprende_critico: c.f("APRENDE_CRITICO"),
            aprende_actor: c.f("APRENDE_ACTOR"), huella: c.f("HUELLA"), franjas: c.i("FRANJAS"),
            calma: c.f("CALMA"), aprende_minimo: c.f("APRENDE_MINIMO"),
            umbral_recuerdo: c.f("UMBRAL_RECUERDO"), recuerdo_decae: c.f("RECUERDO_DECAE"),
            temperatura_base: c.f("TEMPERATURA_BASE"), temperatura_sorpresa: c.f("TEMPERATURA_SORPRESA"),
            temperatura_malestar: c.f("TEMPERATURA_MALESTAR"), margen_explorar: c.f("MARGEN_EXPLORAR"),
            peligro_sin_malestar: c.b("PELIGRO_SIN_MALESTAR"), temperatura_malestar_tope: c.f("TEMPERATURA_MALESTAR_TOPE"), foco_ciego: match c.txt("FOCO_CIEGO").as_str() { "quedarse" => 0, "neutro" => 1, "neutro_todo" => 2, x => { eprintln!("angri (rust): FOCO_CIEGO={x} no existe"); std::process::exit(2) } }, desmayo_final: c.b("DESMAYO_FINAL"), critico_cuerpo: c.f("CRITICO_CUERPO") as i64, estres_cobra, invierno: match c.txt("INVIERNO").as_str() { "viejo" => 0, "repartido" => 1, x => { eprintln!("angri (rust): INVIERNO={x} no existe"); std::process::exit(2) } }, agua_fria: c.f("AGUA_FRIA"), cueva_abriga: match c.txt("CUEVA").as_str() { "vieja" => false, "abriga" => true, x => { eprintln!("angri (rust): CUEVA={x} no existe"); std::process::exit(2) } }, vado: match c.txt("CRECIDA").as_str() { "vieja" => false, "vado" => true, x => { eprintln!("angri (rust): CRECIDA={x} no existe"); std::process::exit(2) } },
            atajo_alivia: c.b("ATAJO_ALIVIA"), atajo_comida: c.b("ATAJO_COMIDA"), atajo_veo_comida: c.b("ATAJO_VEO_COMIDA"), atajo_lugares: c.b("ATAJO_LUGARES"), actor_por_activos: c.b("ACTOR_POR_ACTIVOS"), decae_actor: c.f("DECAE_ACTOR"), decae_solo_positivos: c.b("DECAE_SOLO_POSITIVOS"), decae_propio: c.f("DECAE_PROPIO"), ancla_v: c.f("ANCLA_V"), olvidar_ultimo: c.b("OLVIDAR_ULTIMO"), tope_v_propio: c.f("TOPE_V_PROPIO"), decae_actor_modo: match c.txt("DECAE_ACTOR_MODO").as_str() { "reloj" => 0, "presentes" => 1, "huella" => 2, "elegida" => 3, x => { eprintln!("angri (rust): DECAE_ACTOR_MODO={x} no existe"); std::process::exit(2) } },
            critico_propio: match c.txt("CRITICO_PROPIO").as_str() { "ambos" => 0, "tabla" => 1, "unidades" => 2, x => { eprintln!("angri (rust): CRITICO_PROPIO={x} no existe"); std::process::exit(2) } },
            distancia_fina: match c.txt("DISTANCIA_PROPIA").as_str() { "vieja" => false, "fina" => true, x => { eprintln!("angri (rust): DISTANCIA_PROPIA={x} no existe"); std::process::exit(2) } },
            recompensa_alivio: c.f("RECOMPENSA_ALIVIO"), dolor: c.f("DOLOR"),
            recompensa_estado: c.f("RECOMPENSA_ESTADO"),
            hipocampo: c.b("HIPOCAMPO"), capacidad_dia: c.i("CAPACIDAD_DIA"), repasos: c.i("REPASOS"),
            repaso_actor: c.b("REPASO_ACTOR"), aprende_repaso: c.f("APRENDE_REPASO"),
            umbral_lugar: c.f("UMBRAL_LUGAR"), poda_lugares: c.f("PODA_LUGARES"),
            importancia: c.b("IMPORTANCIA"), tope_importancia: c.f("TOPE_IMPORTANCIA"),
            borrar_lugar_vacio: c.b("BORRAR_LUGAR_VACIO"), prioridad_base: c.f("PRIORIDAD_BASE"),
            tope_lugar: c.f("TOPE_LUGAR"),
            maestro: c.b("MAESTRO"), prob_habla: c.f("PROB_HABLA"), lexico: c.b("LEXICO"),
            aprende_palabra: c.f("APRENDE_PALABRA"), piso_palabra: c.f("PISO_PALABRA"), ego: c.b("EGO"),
            espera_verificar: c.i("ESPERA_VERIFICAR"), pasos_dicha: c.i("PASOS_DICHA"),
            certeza_dicha: c.f("CERTEZA_DICHA"), confianza_inicial: c.f("CONFIANZA_INICIAL"),
            aprende_confianza: c.f("APRENDE_CONFIANZA"), confianza_minima: c.f("CONFIANZA_MINIMA"),
            ritmo_maximo: c.f("RITMO_MAXIMO"), sorpresa_ritmo: c.f("SORPRESA_RITMO"),
            maestro_solo_si_pide: c.b("MAESTRO_SOLO_SI_PIDE"), maestro_inicio: c.i("MAESTRO_INICIO"),
            prob_espontaneo: c.f("PROB_ESPONTANEO"), maestro_ciego: c.b("MAESTRO_CIEGO"),
            equilibrio, peso,
            gasto_energia_base: c.f("GASTO_ENERGIA_BASE"), gasto_energia_mover: c.f("GASTO_ENERGIA_MOVER"),
            sube_cansancio_despierto: c.f("SUBE_CANSANCIO_DESPIERTO"), sube_curiosidad: c.f("SUBE_CURIOSIDAD"),
            calma_estres: c.f("CALMA_ESTRES"), recupera_integridad: c.f("RECUPERA_INTEGRIDAD"),
            dano_inanicion: c.f("DANO_INANICION"), gasto_agua: c.f("GASTO_AGUA"), beber: c.f("BEBER"),
            templa: c.f("TEMPLA"), dano_frio: c.f("DANO_FRIO"), dano_sed: c.f("DANO_SED"),
            adenosina_por_actividad: c.f("ADENOSINA_POR_ACTIVIDAD"),
            adenosina_limpieza_sueno: c.f("ADENOSINA_LIMPIEZA_SUENO"), umbral_dormir: c.f("UMBRAL_DORMIR"),
            umbral_despertar: c.f("UMBRAL_DESPERTAR"), cortisol_sube: c.f("CORTISOL_SUBE"),
            cortisol_baja: c.f("CORTISOL_BAJA"), malestar_maximo: c.f("MALESTAR_MAXIMO"),
            mente_azar: mente == "azar", reflejos: c.b("REFLEJOS"),
            corteza: c.b("CORTEZA"), aprende_corteza: c.f("APRENDE_CORTEZA"),
            escala_error_malestar: c.f("ESCALA_ERROR_MALESTAR"), curiosidad_rapido: c.f("CURIOSIDAD_RAPIDO"),
            curiosidad_lento: c.f("CURIOSIDAD_LENTO"), peso_curiosidad: c.f("PESO_CURIOSIDAD"),
            talamo: c.b("TALAMO"), ganancia_urgencia: c.f("GANANCIA_URGENCIA"),
            temperatura_foco: c.f("TEMPERATURA_FOCO"), inercia_foco: c.f("INERCIA_FOCO"),
            aprende_talamo: c.f("APRENDE_TALAMO"),
            palabra_suelta: c.b("PALABRA_SUELTA"), frase: c.b("FRASE"), memoria_palabra: c.i("MEMORIA_PALABRA"),
            maestro_miente: c.b("MAESTRO_MIENTE"), holistico: c.b("HOLISTICO"),
            cambio_significado: c.i("CAMBIO_SIGNIFICADO"), composicion: c.i("COMPOSICION"),
            pide: c.b("PIDE"), pide_azar: c.b("PIDE_AZAR"), temperatura_habla: c.f("TEMPERATURA_HABLA"),
            aprende_habla: c.f("APRENDE_HABLA"), ventana_habla: c.i("VENTANA_HABLA"), escucha: c.b("ESCUCHA"),
            escucha_cuerpo: c.b("ESCUCHA_CUERPO"), escucha_pedidos: c.b("ESCUCHA_PEDIDOS"),
            aprende_escucha: c.f("APRENDE_ESCUCHA"), temperatura_escucha: c.f("TEMPERATURA_ESCUCHA"),
            umbral_escucha: c.f("UMBRAL_ESCUCHA"), escucha_oido: c.b("ESCUCHA_OIDO"),
            escucha_alivio: c.b("ESCUCHA_ALIVIO"), ventana_alivio: c.i("VENTANA_ALIVIO"),
            umbral_alivio: c.f("UMBRAL_ALIVIO"),
            objetos: c.b("OBJETOS") && c.txt("MUNDO") == "bosque", objetos_iniciales: c.i("OBJETOS_INICIALES"),
            objetos_maximo: c.i("OBJETOS_MAXIMO"), prob_objeto: c.f("PROB_OBJETO"), fuerza: c.f("FUERZA"),
            calor_fuego: c.f("CALOR_FUEGO"), corta_viento: c.f("CORTA_VIENTO"),
            prob_prender: c.f("PROB_PRENDER"), fuego_ticks: c.i("FUEGO_TICKS"),
            propia: rasgos != "mano" && mente == "aprende", rasgos_mano: rasgos != "propios", rasgo_alrededor: c.b("RASGO_ALREDEDOR"),
            propios: c.i("PROPIOS"), nuevos_por_tick: c.i("NUEVOS_POR_TICK"),
            reemplazo_cada: c.i("REEMPLAZO_CADA").max(1), reemplazo_fraccion: c.f("REEMPLAZO_FRACCION"),
            madurez_propio: c.i("MADUREZ_PROPIO"), aprende_propio_actor: c.f("APRENDE_PROPIO_ACTOR"),
            aprende_propio_critico: c.f("APRENDE_PROPIO_CRITICO"), tope_activa: c.f("TOPE_ACTIVA"), ritmo_raro: c.f("RITMO_RARO"),
            repaso_propio: c.b("REPASO_PROPIO") && c.b("HIPOCAMPO"), nacer_sorpresa: c.b("NACER_SORPRESA"),
            repaso_propio_cada: c.i("REPASO_PROPIO_CADA"), predice_propio: c.f("PREDICE_PROPIO"), predice_actor: c.f("PREDICE_ACTOR"), predice_todas: c.f("PREDICE_TODAS") as i64, critico_cuerpo_propio: c.f("CRITICO_CUERPO_PROPIO") as i64, salida_propia: c.f("SALIDA_PROPIA") as i64, salida_umbral: c.f("SALIDA_UMBRAL"), salida_debug: c.b("SALIDA_DEBUG"), salida_lejos: c.f("SALIDA_LEJOS") as i64, salida_lejos_peso: c.f("SALIDA_LEJOS_PESO"), critico_vecinos: c.b("CRITICO_VECINOS"),
        }
    }
}
