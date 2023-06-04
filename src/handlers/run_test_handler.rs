use axum::{Json, response::IntoResponse, http::StatusCode};
use crate::model::FirestarterParams;
use crate::firestarter::Firestarter;
use crate::rapl::monitor_rapl::monitor_rapl;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use log::trace;

const RAPL_END_DELAY_SECS: u64 = 1;

pub async fn run_test_handler(Json(firestarter_params): Json<FirestarterParams>) -> impl IntoResponse {
    trace!("run_test_handler({firestarter_params:?})");
    // start rapl monitor
    let (rapl_tx, rapl_rx) = mpsc::channel();
    let rapl_thread = thread::spawn(move || monitor_rapl(&rapl_rx, firestarter_params.runtime_secs + RAPL_END_DELAY_SECS));
    trace!("Launching firestarter");

    // start firestarter
    let firestarter = Firestarter::new(firestarter_params);
    firestarter.run();
    trace!("Firestarter finished, signalling rapl monitor");
    thread::sleep(Duration::from_secs(RAPL_END_DELAY_SECS));
    rapl_tx.send(())
        .expect("Failed to send halt message to rapl monitor");

    trace!("Signalled rapl monitor, joining");
    let rapl_stats = rapl_thread.join()
        .expect("Failed to join rapl thread and receive data");
    trace!("Joinined rapl thread");
    println!("RAPL stats: {rapl_stats:?}");
    (StatusCode::OK, Json(rapl_stats))
}
