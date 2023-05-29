use log::{error, trace};
use std::fmt::{self, Display, Formatter};
use std::process::Command;
use crate::model::FirestarterParams;

const FIRESTARTER_PATH: &str = "/home_nfs/wainj/local/bin/firestarter";

#[derive(Debug)]
/// Hold the firestarter configuration
pub struct Firestarter {
    path: String,
    runtime_secs: u64,
    load_pct: u64,
    load_period_us: u64,
    n_threads: u64,
}

impl Firestarter {
    #[must_use]
    /// Creates a new firestarter instance ready to run performing basic validation which may cause...
    /// # Panics
    pub fn new(params: FirestarterParams) -> Self {
        assert!(params.load_pct > 0 && params.load_pct <= 100);
        assert!(params.load_period_us == 0 || params.load_pct <= params.load_period_us);
        Self {
            path: FIRESTARTER_PATH.into(),
            runtime_secs: params.runtime_secs,
            load_pct: params.load_pct,
            load_period_us: params.load_period_us,
            n_threads: params.n_threads,
        }
    }

    /// Launches firestarter. This is done on a separate thread.
    // TODO: Might be pertinent to bind threads to processors to see if there's
    //       uneven capping across domains.
    pub fn run(&self) {
        trace!("FIRESTARTER LAUNCHING:\n{self}");
        let firestarter = Command::new(&self.path)
            .arg("--quiet")
            .arg("--timeout")
            .arg(self.runtime_secs.to_string())
            .arg("--load")
            .arg(self.load_pct.to_string())
            .arg("--period")
            .arg(self.load_period_us.to_string())
            .arg("--threads")
            .arg(self.n_threads.to_string())
            .spawn()
            .expect("Firestarter failed to launch");

        match firestarter.wait_with_output() {
            Ok(_) => trace!("FIRESTARTER exited successfully"),
            Err(e) => error!("FIRESTARTER failed: {e:?}"),
        }
    }
}

impl Display for Firestarter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} --timeout {} --load {} --period {} --threads {}",
            self.path, self.runtime_secs, self.load_pct, self.load_period_us, self.n_threads
        )
    }
}
