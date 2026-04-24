use axum::{
    body::{Body, to_bytes},
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    extract::Request as ExtractRequest,

};
use http::StatusCode;
use tracing;




pub async fn log_raw_body(req: ExtractRequest, next: Next) -> Response {
    // 1. Separamos la request en sus partes (cabeceras, uri, etc) y el cuerpo
    let (parts, body) = req.into_parts();

    // 2. Leemos el cuerpo crudo inmediatamente, ANTES de cualquier extractor.
    // Límite de 2MB. Si mandas algo más grande, fallará aquí.
    let bytes = match to_bytes(body, 2 * 1024 * 1024).await {
        Ok(b) => b,
        Err(err) => {
            println!(">>> Error leyendo bytes crudos: {}", err);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // 3. Imprimimos el cuerpo por consola (usamos println! para ir a lo seguro)
    if let Ok(body_str) = std::str::from_utf8(&bytes) {
        println!(">>> 🚀 RAW BODY LLEGANDO: {}", body_str);
        // Cuando confirmes que funciona, cámbialo por: tracing::info!("RAW BODY: {}", body_str);
    } else {
        println!(">>> 🚀 RAW BODY (Binario/No texto): {:?}", bytes);
    }

    // 4. Reconstruimos la request metiendo los bytes de vuelta.
    // Esto es crucial para que tu Handler (y sus validaciones de tipo) pueda volver a leerlo.
    let req = Request::from_parts(parts, Body::from(bytes));

    // 5. Dejamos que la petición continúe hacia el router y los handlers
    next.run(req).await
}



pub async fn log_body_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    let (parts, body) = req.into_parts();

    // 1. Definimos un límite de tamaño para leer el body (ej: 2 MB)
    let body_limit = 2 * 1024 * 1024;

    // 2. Extraemos los bytes reales usando axum::body::to_bytes
    let bytes = match to_bytes(body, body_limit).await {
        Ok(b) => b,
        Err(err) => {
            tracing::error!("Error leyendo el body o límite excedido: {}", err);
            return axum::http::StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }
    };

    // 3. Lo convertimos a string para hacer el log (si es texto legible)
    if let Ok(body_str) = std::str::from_utf8(&bytes) {
        tracing::info!("Body request: {}", body_str);
    } else {
        tracing::info!("Body request: [Bytes no decodificables como UTF-8]");
    }

    // 4. Reconstruimos la request con un nuevo Body a partir de los bytes clonados
    let req = Request::from_parts(parts, Body::from(bytes));

    // 5. Pasamos la request al siguiente middleware o handler
    next.run(req).await
}

