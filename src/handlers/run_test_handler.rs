use axum::{response::IntoResponse, Json};
use serde::Serialize;


use crate::model::FirestarterParams;

#[derive(Debug, Serialize)]
pub struct Rc {
    pub wally: String,
}


pub async fn run_test_handler(Json(body): Json<FirestarterParams>) -> Json<Rc> {

    println!("{body:?}");
    // start rapl monitor


    // start firestarter



    Json(Rc{wally: String::from("was here")})
}
