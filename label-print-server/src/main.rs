use actix_web::{middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, Serialize)]
struct PrintRequest {
    // The user wrote `grogery`, but also mentioned `grocery` elsewhere; accept both.
    #[serde(alias = "grocery")]
    grocery: String,
    subtitle: Option<String>,
    // optionally allow small-title override
    #[serde(default)]
    small_title: Option<bool>,
}

#[derive(Debug, Serialize)]
struct TemplateData {
    grocery: String,
    subtitle: Option<String>,
    #[serde(rename = "small-title")]
    small_title: bool,
}

#[post("/print")]
async fn print_label(payload: web::Json<PrintRequest>) -> impl Responder {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let typst_file = base_dir.join("99012.typ");
    let yaml_file = base_dir.join("data.yml");
    let output_pdf = base_dir
        .parent()
        .unwrap_or(base_dir)
        .join("99012.pdf");

    // Prepare YAML content from request
    let data = TemplateData {
        grocery: payload.grocery.clone(),
        subtitle: payload
            .subtitle
            .as_ref()
            .and_then(|s| if s.trim().is_empty() { None } else { Some(s.clone()) }),
        small_title: payload.small_title.unwrap_or(true),
    };

    if let Err(e) = serde_yaml::to_string(&data)
        .map_err(|e| e.to_string())
        .and_then(|s| fs::write(&yaml_file, s).map_err(|e| e.to_string()))
    {
        return HttpResponse::InternalServerError().body(format!("Failed to write YAML: {e}"));
    }

    // Call typst to compile 99012.typ. Ensure typst is installed on PATH.
    let typst_status = Command::new("typst")
        .current_dir(base_dir)
        .arg("compile")
        .arg(&typst_file)
        .arg(&output_pdf)
        .status();

    match typst_status {
        Ok(status) if status.success() => {}
        Ok(status) => {
            return HttpResponse::InternalServerError()
                .body(format!("typst exited with status: {status}"));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Failed to run typst: {e}"));
        }
    }

    // Print using lp -d <printer> <pdf>
    let printer_name = env::var("PRINTER_NAME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "DYMO_LabelWriter_450".to_string());

    let lp_status = Command::new("lp")
        .arg("-d")
        .arg(&printer_name)
        .arg(&output_pdf)
        .status();

    match lp_status {
        Ok(status) if status.success() => HttpResponse::Ok().body("queued"),
        Ok(status) => HttpResponse::InternalServerError()
            .body(format!("lp exited with status: {status}")),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to run lp: {e}")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting label print server.");
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);

    println!("Starting server on http://{}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(print_label)
    })
    .bind((host, port))?
    .run()
    .await
}
