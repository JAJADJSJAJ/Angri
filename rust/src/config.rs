//! La config de angri/config.py, que Python manda entera como líneas CLAVE=valor
//! (repr de Python; los dicts van como CLAVE.sub=valor).
use std::collections::HashMap;

pub struct Config {
    pub ancho: i64, pub alto: i64, pub comida_inicial: i64, pub novedades_iniciales: i64,
    pub peligros: i64, pub prob_nueva_comida: f64, pub prob_nueva_novedad: f64,
    pub fuentes_comida: i64, pub radio_fuente: i64, pub comida_maxima: i64,
    pub novedad_maxima: i64, pub estacion: i64, pub prob_mueve_peligro: f64,
    pub bosque: bool, pub bosque_ancho: i64, pub bosque_alto: i64, pub dia: i64,
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
    pub temperatura_malestar: f64, pub margen_explorar: f64, pub peligro_sin_malestar: bool,
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
}

pub const NOMBRES: [&str; 7] = ["energia", "cansancio", "curiosidad", "estres", "integridad", "agua", "calor"];

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
    pub fn leer(texto: &str) -> Config {
        let mut m = HashMap::new();
        for l in texto.lines() {
            if let Some((k, v)) = l.split_once('=') {
                m.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        let c = Crudo(m);
        // lo que el núcleo en Rust no tiene: tiene que estar apagado
        for k in ["CORTEZA", "TALAMO", "PIDE", "PALABRA_SUELTA", "FRASE", "MAESTRO_MIENTE",
                  "HOLISTICO", "CAMBIO_SIGNIFICADO", "COMPOSICION"] {
            if c.b(k) {
                eprintln!("angri (rust): {k} no está portado a Rust; corré sin --rust");
                std::process::exit(2);
            }
        }
        if c.txt("MENTE") != "aprende" {
            eprintln!("angri (rust): solo MENTE='aprende' está portada");
            std::process::exit(2);
        }
        let mut equilibrio = [0.0; 7];
        let mut peso = [0.0; 7];
        for (i, n) in NOMBRES.iter().enumerate() {
            equilibrio[i] = if i < 5 { c.f(&format!("EQUILIBRIO.{n}")) } else { c.f(&format!("EQUILIBRIO_BOSQUE.{n}")) };
            peso[i] = c.f(&format!("PESO_NECESIDAD.{n}"));
        }
        Config {
            ancho: c.i("ANCHO"), alto: c.i("ALTO"), comida_inicial: c.i("COMIDA_INICIAL"),
            novedades_iniciales: c.i("NOVEDADES_INICIALES"), peligros: c.i("PELIGROS"),
            prob_nueva_comida: c.f("PROB_NUEVA_COMIDA"), prob_nueva_novedad: c.f("PROB_NUEVA_NOVEDAD"),
            fuentes_comida: c.i("FUENTES_COMIDA"), radio_fuente: c.i("RADIO_FUENTE"),
            comida_maxima: c.i("COMIDA_MAXIMA"), novedad_maxima: c.i("NOVEDAD_MAXIMA"),
            estacion: c.i("ESTACION"), prob_mueve_peligro: c.f("PROB_MUEVE_PELIGRO"),
            bosque: c.txt("MUNDO") == "bosque", bosque_ancho: c.i("BOSQUE_ANCHO"),
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
            peligro_sin_malestar: c.b("PELIGRO_SIN_MALESTAR"),
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
        }
    }
}
