use axum::{response::IntoResponse, Json};
use serde::Serialize;
use serde_json::Value;


#[derive(Debug, Serialize)]
pub struct Rc {
    pub wally: String,
}


pub async fn run_test_handler(Json(body): Json<Value>) -> Json<Rc> {

    println!("{body:?}");
    // start rapl monitor


    // start firestarter



    Json(Rc{wally: String::from("was here")})
}
