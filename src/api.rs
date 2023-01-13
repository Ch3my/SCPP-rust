use tracing_subscriber::fmt::format;

use crate::Documento;

// Para comunicar el componente padre al hijo tenemos que usar messages. La herencia
// de variables como en react no funciona porque Rust no acepta mezclar el Scope
// de las variables.

// Definimos el propio state de este "componente"
pub struct ApiState {
    pub docs: Vec<Documento>,
}

pub fn get_docs(
    tx: &mut std::sync::mpsc::Sender<ApiState>,
    api_prefix: String,
    selected_tipo_doc: String,
) {
    // Para concatenar un string es necesario que Rust lo guarde en memoria
    // por eso lo concatenamos (se guarda en memoria) y luego lo usamos
    // NOTA: no se puede usar .as_str() en addr porque estaria saltandose un paso
    // que Rust necesita (guardar en memoria luego transformar)
    let addr = format!(
        "{}{}&fk_tipoDoc={}",
        api_prefix, "/documentos?sessionHash=p0j13h6oockrrou5jfxlf", selected_tipo_doc
    );

    let body = ureq::get(addr.as_str()).call();

    // Creamos datos dummy para pruebas
    let mut state = ApiState { docs: Vec::new() };

    // ureq entiende que al convertir a JSON tiene que apuntar a este objeto
    // y tratar de mapear las propiedades
    match body {
        Ok(res) => state.docs = res.into_json().unwrap(),
        _ => print!("Error al consumir API 1"),
    }

    // Enviamos datos al Padre
    tx.send(state).unwrap();
}
