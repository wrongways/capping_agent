use axum::Json;
use crate::model::FirestarterParams;
use crate::firestarter::Firestarter;
use crate::rapl::monitor_rapl::monitor_rapl;
use crate::model::RaplRecord;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const RAPL_END_DELAY_SECS: u64 = 2;

pub async fn run_test_handler(Json(firestarter_params): Json<FirestarterParams>) -> Json<Vec<RaplRecord>> {

    println!("{firestarter_params:?}");
    // start rapl monitor
    let (rapl_tx, rapl_rx) = mpsc::channel();
    let rapl_thread = thread::spawn(move || monitor_rapl(&rapl_rx, firestarter_params.runtime_secs + RAPL_END_DELAY_SECS));

    // start firestarter
    let firestarter = Firestarter::new(firestarter_params);
    firestarter.run();
    thread::sleep(Duration::from_secs(RAPL_END_DELAY_SECS));
    rapl_tx.send(())
        .expect("Failed to send halt message to rapl monitor");

    let rapl_stats = rapl_thread.join()
        .expect("Failed to join rapl thread and receive data");

    println!("RAPL stats: {rapl_stats:?}");
    Json(rapl_stats)
}
