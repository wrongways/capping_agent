pub mod handlers;
pub mod model;
pub mod response;
pub mod route;
pub mod server;


fn am_root() -> bool {
    match std::env::var("USER") {
        Ok(user) => user == "root",
        Err(_e) => false,
    }
}
