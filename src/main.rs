#[macro_use]
extern crate actix_web;
extern crate clap;
extern crate memcache;

const APPNAME: &'static str = "memproxy";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DEF_CFG_FN: &'static str = "cfg-memproxy.json";
const DEF_BIND_ADDR: &'static str = "127.0.0.1";
const DEF_BIND_PORT: &'static str = "8080";

use std::sync::{Arc, Mutex};
use std::{env, fs, io};

use actix_web::http::StatusCode;
use actix_web::{guard, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct UpstreamConfig {
    endpoint: String,
}

#[derive(Serialize, Deserialize)]
struct ServerConfig {
    upstream: Vec<UpstreamConfig>,
}

struct ServerState {
    memclient: memcache::Client,
}

// helper function, 404 not found
fn err_not_found() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("application/json")
        .body(
            json!({
          "error": {
             "code" : -404,
              "message": "not found"}})
            .to_string(),
        ))
}

// helper function, server error
fn err_500() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .content_type("application/json")
        .body(
            json!({
          "error": {
             "code" : -500,
              "message": "internal server error"}})
            .to_string(),
        ))
}

// helper function, success + binary response
fn ok_binary(val: Vec<u8>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/octet-stream")
        .body(val))
}

// helper function, success + json response
fn ok_json(jval: serde_json::Value) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/json")
        .body(jval.to_string()))
}

/// simple root index handler, describes our service
#[get("/")]
fn req_index(
    req: HttpRequest,
) -> Result<HttpResponse> {
    println!("{:?}", req);

    ok_json(json!({
        "name": APPNAME,
        "version": VERSION,
    }))
}

/// DELETE data item.  key in URI path.  returned ok as json response
fn req_delete(
    m_state: web::Data<Arc<Mutex<ServerState>>>,
    req: HttpRequest,
    path: web::Path<(String,)>,
) -> Result<HttpResponse> {
    println!("{:?}", req);

    let mut state = m_state.lock().unwrap();

    match state.memclient.delete(&path.0) {
        Ok(optval) => match optval {
	    true => ok_json(json!({ "result": true })),
            false => err_not_found(), // db: value not found
	}
        Err(_e) => err_500(), // db: error
    }
}

/// GET data item.  key in URI path.  returned value as binary response
fn req_get(
    m_state: web::Data<Arc<Mutex<ServerState>>>,
    req: HttpRequest,
    path: web::Path<(String,)>,
) -> Result<HttpResponse> {
    println!("{:?}", req);

    let mut state = m_state.lock().unwrap();

    match state.memclient.get::<Vec<u8>>(&path.0) {
        Ok(optval) => match optval {
            Some(val) => ok_binary(val),
            None => err_not_found(), // db: value not found
        },
        Err(_e) => err_500(), // db: error
    }
}

/// PUT data item.  key in URI path, value in HTTP payload.
fn req_put(
    m_state: web::Data<Arc<Mutex<ServerState>>>,
    req: HttpRequest,
    (path, body): (web::Path<(String,)>, web::Bytes),
) -> Result<HttpResponse> {
    println!("{:?}", req);

    let mut state = m_state.lock().unwrap();

    match state.memclient.set(&path.0, &body[..], 0) {
        Ok(_optval) => ok_json(json!({"result": true})),
        Err(_e) => err_500(), // db: error
    }
}

/// 404 handler
fn p404() -> Result<HttpResponse> {
    err_not_found()
}

fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // parse command line
    let cli_matches = clap::App::new(APPNAME)
        .version(VERSION)
        .about("Microservice REST wrapper for memcached")
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("JSON-FILE")
                .help(&format!(
                    "Sets a custom configuration file (default: {})",
                    DEF_CFG_FN
                ))
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("bind-addr")
                .long("bind-addr")
                .value_name("IP-ADDRESS")
                .help(&format!(
                    "Custom server socket bind address (default: {})",
                    DEF_BIND_ADDR
                ))
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("bind-port")
                .long("bind-port")
                .value_name("PORT")
                .help(&format!(
                    "Custom server socket bind port (default: {})",
                    DEF_BIND_PORT
                ))
                .takes_value(true),
        )
        .get_matches();

    // configure based on CLI options
    let bind_addr = cli_matches.value_of("bind-addr").unwrap_or(DEF_BIND_ADDR);
    let bind_port = cli_matches.value_of("bind-port").unwrap_or(DEF_BIND_PORT);
    let bind_pair = format!("{}:{}", bind_addr, bind_port);
    let server_hdr = format!("{}/{}", APPNAME, VERSION);

    // read JSON configuration file
    let cfg_fn = cli_matches.value_of("config").unwrap_or(DEF_CFG_FN);
    let cfg_text = fs::read_to_string(cfg_fn)?;
    let server_cfg: ServerConfig = serde_json::from_str(&cfg_text)?;

    // we only support 1 upstream, for now
    assert_eq!(server_cfg.upstream.len(), 1);
    let memhost = format!("memcache://{}", server_cfg.upstream[0].endpoint);

    let srv_state = Arc::new(Mutex::new(ServerState {
        memclient: memcache::Client::connect(memhost).unwrap(),
    }));

    // configure web server
    let sys = actix_rt::System::new(APPNAME);

    HttpServer::new(move || {
        App::new()
            // pass application state to each handler
            .data(Arc::clone(&srv_state))
            // apply default headers
            .wrap(middleware::DefaultHeaders::new().header("Server", server_hdr.to_string()))
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // register our routes
            .service(req_index)
            .service(
                web::resource("/cache/{key}")
                    .route(web::get().to(req_get))
                    .route(web::put().to(req_put))
                    .route(web::delete().to(req_delete)),
            )
            // default
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET` -- redundant?
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind(bind_pair.to_string())?
    .start();

    println!("Starting http server: {}", bind_pair);
    sys.run()
}
