use std::process::Command;
use serde_json::{self, Value};
use crate::am_root;
use std::fs;

use axum::{response::IntoResponse, Json, http::StatusCode};

use crate::{
    model::{BiosInfo, ServerInfo, SystemInfo},
};

struct CPUInfo {
    n_online: u64,
    model: String,
    max_mhz: u64,
    min_mhz: u64,
    threads_per_core: u64,
    cores_per_socket: u64,
    n_sockets: u64,
}

struct SysInfo {
    vendor: String,
    model: String,
    os: String,
}


pub async fn system_info_handler() -> impl IntoResponse {
    let cpu_info = cpu_info();
    let sys_info = sys_info();
    let system_info = SystemInfo {
        hostname: hostname(),
        model: sys_info.model,
        os: sys_info.os,
        manufacturer: sys_info.vendor,
        cpu_version: cpu_info.model,
        online_cpus: cpu_info.n_online,
        min_mhz: cpu_info.min_mhz,
        max_mhz: cpu_info.max_mhz,
        threads_per_core: cpu_info.threads_per_core,
        cores_per_socket: cpu_info.cores_per_socket,
        n_sockets: cpu_info.n_sockets,
    };

    let bios_info = bios();

    let server_info = ServerInfo {
        system_info,
        bios_info,
    };

    let response = serde_json::json!(server_info);

    (StatusCode::OK, Json(response))
}


fn hostname() -> String {
    String::from_utf8(
        Command::new("hostname")
            .arg("-s")
            .output()
            .expect("failed to get hostname")
            .stdout)
        .expect("Failed to extract hostname")
        .trim()
        .to_string()
}

fn cpu_info() -> CPUInfo {
    let mut cpu_count: u64 = 0;
    let mut model = String::new();
    let mut min_mhz: u64 = 0;
    let mut max_mhz: u64 = 0;
    let mut threads_per_core: u64 = 0;
    let mut cores_per_socket: u64 = 0;
    let mut n_sockets: u64 = 0;

    let cpu_info = String::from_utf8(
        Command::new("lscpu")
            .output()
            .expect("Failed to run lscpu")
            .stdout)
        .expect("Failed to extract cpu info");

    for line in cpu_info.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        assert!(parts.len() == 2);
        let (lhs, rhs) = (parts[0], parts[1]);
        match lhs.trim() {
            "CPU(s)" => cpu_count = rhs.trim().parse().expect("Failed to parse cpu count"),
            "Model name" => model = rhs.trim().to_string(),
            "CPU min MHz" => min_mhz = rhs.trim().parse::<f64>().expect("Failed to parse cpu freq") as u64,
            "CPU max MHz" => max_mhz = rhs.trim().parse::<f64>().expect("Failed to parse cpu freq") as u64,
            "Thread(s) per core" => threads_per_core = rhs.trim().parse().expect("Failed to parse cpu threads"),
            "Core(s) per socket" => cores_per_socket = rhs.trim().parse().expect("Failed to parse cpu cores"),
            "Socket(s)" => n_sockets = rhs.trim().parse().expect("Failed to parse cpu freq"),
            _ => continue,
        }
    }

    CPUInfo {
        n_online: cpu_count,
        model,
        min_mhz,
        max_mhz,
        threads_per_core,
        cores_per_socket,
        n_sockets
    }
}

fn sys_info() -> SysInfo {
    if am_root() {
        let sys_info = String::from_utf8(
            Command::new("lshw")
                .arg("-C")
                .arg("system")
                .arg("-quiet")
                .arg("-json")
                .output()
                .expect("Failed to run lshw")
                .stdout)
            .expect("Failed to extract hw info");

        let value: Value = serde_json::from_str(&sys_info)
            .expect("Failed to parse lshw json");


        // remove anything in parenthesis at the end.
        let mut model = value["product"].to_string().replace('"', "");
        if let Some(index) = model.find('(') {
            model.truncate(index);
        }

        SysInfo {
            vendor: value["vendor"].to_string().replace('"', ""),
            model,
            os: os(),
        }
    } else {
        SysInfo {
            vendor: "Unknown".to_string(),
            model: "Unknown".to_string(),
            os: os(),
        }
    }
}

fn os() -> String {
    let mut os_name = String::new();
    let mut os_version = String::new();

    let os_release = fs::read_to_string("/etc/os-release").expect("Failed to read /etc/os_release");
    for line in os_release.lines() {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            let (lhs, rhs) = (parts[0], parts[1]);
            match lhs.to_lowercase().trim() {
                "name" => os_name = rhs.trim().to_string(),
                "version" => os_version = rhs.trim().to_string(),
                _ => continue,
            }
        }
    }
    println!("os(): {os_name} {os_version}");
    format!("{os_name} {os_version}").replace('"', "")
}

fn bios() -> BiosInfo {
    if am_root() {
        let bios_info = String::from_utf8(
            Command::new("dmidecode")
                .arg("-q")
                .arg("-t")
                .arg("0")
                .output()
                .expect("Failed to run dmidecode")
                .stdout)
            .expect("Failed to extract dmi info");

        let mut vendor = String::new();
        let mut version = String::new();
        let mut revision = String::new();
        let mut release_date = String::new();

        for line in bios_info.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                let (lhs, rhs) = (parts[0], parts[1]);
                match lhs.to_lowercase().trim() {
                    "vendor" => vendor = rhs.trim().to_string(),
                    "version" => version = rhs.trim().to_string(),
                    "bios revision" => revision = rhs.trim().to_string(),
                    "release date" => release_date = rhs.trim().to_string(),
                    _ => continue,
                }
            }
        }

        BiosInfo {vendor, version, revision, release_date}

    } else {
        BiosInfo {
            vendor: "unknown".to_string(),
            version: "unknown".to_string(),
            revision: "unknown".to_string(),
            release_date: "unknown".to_string(),
        }
    }

}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_hostname() {
        assert_eq!(hostname(), "iMac");
    }
}
