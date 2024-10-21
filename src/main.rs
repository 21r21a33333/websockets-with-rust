mod ws;
mod lobby;
use lobby::Lobby;
mod messages;
mod start_connection;
use start_connection::start_connection as start_connection_route;
use start_connection::notify_poll_id;
use actix::Actor;

use actix_web::{web::Data, App, HttpServer};


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let chat_server = Lobby::default().start(); //create and spin up a lobby

    HttpServer::new(move || {
        App::new()
            .service(start_connection_route) //register our route. rename with "as" import or naming conflict
            .service(notify_poll_id)
            .app_data(Data::new(chat_server.clone())) //register the lobby
    })
    .bind("0.0.0.0:7000")?
    .run()
    .await
}