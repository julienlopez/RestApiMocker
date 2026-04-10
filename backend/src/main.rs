mod internal_routes;
use crate::internal_routes::{
    delete_all_mocks, delete_mock, delete_mocks_by_pattern, get_config, get_history, get_mocks,
    okapi_add_operation_for_delete_all_mocks_, okapi_add_operation_for_delete_mock_,
    okapi_add_operation_for_delete_mocks_by_pattern_, okapi_add_operation_for_get_config_,
    okapi_add_operation_for_get_history_, okapi_add_operation_for_get_mocks_,
    okapi_add_operation_for_set_mock_, set_mock,
};

mod mock_route;
use crate::mock_route::mock_catcher;
mod types;
use crate::types::{MockRegistry, RequestHistory};

use rocket::fairing::AdHoc;
use rocket::fs::{FileServer, relative};
use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::{SwaggerUIConfig, make_swagger_ui};

#[tokio::main]
async fn main() {
    let public_port: u16 = std::env::var("PUBLIC_PORT")
        .unwrap_or_else(|_| "9090".to_string())
        .parse()
        .expect("Invalid port number for public server");
    let private_port: u16 = std::env::var("PRIVATE_PORT")
        .unwrap_or_else(|_| "8090".to_string())
        .parse()
        .expect("Invalid port number for internal server");

    let conf = libcommon::Config {
        public_port,
        private_port,
    };
    println!("Public server running on port: {}", conf.public_port);
    println!("Internal server running on port: {}", conf.private_port);

    let history: RequestHistory = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mocks: MockRegistry = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    let public = rocket::build()
        .configure(rocket::Config {
            address: std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
            port: conf.public_port,
            ..Default::default()
        })
        .manage(history.clone())
        .manage(mocks.clone())
        .attach(AdHoc::on_request("Record request", |req, _| {
            println!("Received request: {} {}", req.method(), req.uri());
            Box::pin(async move {
                if let Some(history) = req.rocket().state::<RequestHistory>() {
                    let record = libcommon::RequestRecord {
                        method: req.method().to_string(),
                        path: req.uri().path().to_string(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    };
                    history.lock().unwrap().push(record);
                }
            })
        }))
        .register("/", rocket::catchers![mock_catcher]);

    let internal = rocket::build()
        .configure(rocket::Config {
            address: std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
            port: conf.private_port,
            ..Default::default()
        })
        .manage(conf)
        .manage(history)
        .manage(mocks)
        .mount(
            "/internal",
            openapi_get_routes![
                get_config,
                get_history,
                get_mocks,
                set_mock,
                delete_mock,
                delete_all_mocks,
                delete_mocks_by_pattern
            ],
        )
        .mount(
            "/internal/docs",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/internal/openapi.json".to_string(),
                ..Default::default()
            }),
        );
    let ui = rocket::build()
        .configure(rocket::Config {
            address: std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
            port: 80,
            ..Default::default()
        })
        .mount("/", FileServer::from(relative!("public")));

    tokio::join!(public.launch(), internal.launch(), ui.launch())
        .0
        .expect("Failed to launch servers");
}
