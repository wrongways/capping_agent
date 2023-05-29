pub mod handlers;
pub mod model;
pub mod response;
pub mod route;
pub mod server;
pub mod rapl;
pub mod firestarter;

use std::os::unix::fs::MetadataExt;


fn am_root() -> bool {
    let uid = std::fs::metadata("/proc/self").map(|m| m.uid())
        .expect("Failed to read /proc/self");
    println!("UID: {uid}");
    uid == 0
}
