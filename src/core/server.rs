use crate::core::engine::{Engine, Options};
use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::path::PathBuf;

async fn handle_zenith(req: HttpRequest) -> impl Responder {
    let mut path = req.path().to_string();
    if path == "/" {
        path = "/index.zen".to_string();
    }

    if !path.ends_with(".zen") {
        let file_path = PathBuf::from(".").join(&path[1..]);
        if file_path.exists() {
            return match NamedFile::open(file_path) {
                Ok(f) => f.into_response(&req),
                Err(_) => HttpResponse::InternalServerError().body("Error opening file"),
            };
        }
        return HttpResponse::NotFound().body("Not Found");
    }

    let full_path = format!(".{}", path);
    let engine = Engine::new(Options {
        allow_read: true,
        allow_net: true,
        allow_env: true,
    });

    match engine.transpile(&full_path) {
        Ok(php_code) => {
            let ctx_init = format!(
                "\n$ctx = new Context();\n$ctx->path = \"{}\";\n$ctx->query = (object)$_GET;\n$ctx->body = (object)$_POST;\n$db = null;\n",
                path
            );
            let final_php = php_code.replace(
                "$file = new ZenithFile();",
                &format!("$file = new ZenithFile();{}", ctx_init),
            );

            match engine.execute(&final_php) {
                Ok(output) => HttpResponse::Ok().content_type("text/html").body(output),
                Err(e) => {
                    HttpResponse::InternalServerError().body(format!("Execution Error: {}", e))
                }
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Transpilation Error: {}", e)),
    }
}

pub async fn start(port: &str) -> std::io::Result<()> {
    let addr = if port.starts_with(':') {
        format!("127.0.0.1{}", port)
    } else {
        format!("127.0.0.1:{}", port)
    };

    println!("Server starting on http://{}", addr);

    HttpServer::new(|| App::new().default_service(web::to(handle_zenith)))
        .bind(addr)?
        .run()
        .await
}
