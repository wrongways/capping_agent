use std::process::Command;
use serde_json::{self, Value};
use crate::am_root;
use std::fs;

use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::{
    model::{BiosInfo, ServerInfo, SystemInfo},
    response::ServerInfoResponse,
};

struct CPUInfo {
    n_online: u64,
    model: String,
    max_mhz: u64,
    min_mhz: u64,
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
    };

    let bios_info = BiosInfo {
        vendor: "vendor".to_string(),
        version: "version".to_string(),
        revision: "revision".to_string(),
        release_date: "release_date".to_string(),
    };

    let server_info = ServerInfo {
        system_info,
        bios_info,
    };

    let response = ServerInfoResponse {
        status: StatusCode::OK.to_string(),
        server_info,
    };

    Json(response)
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
            "CPU min mhz" => min_mhz = rhs.trim().parse().expect("Failed to parse cpu freq"),
            "CPU max mhz" => max_mhz = rhs.trim().parse().expect("Failed to parse cpu freq"),
            _ => continue,
        }
    }

    CPUInfo {
        n_online: cpu_count,
        model,
        min_mhz,
        max_mhz,
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
        let mut model = value["product"].to_string();
        if let Some(index) = model.find('(') {
            model.truncate(index);
        }

        SysInfo {
            vendor: value["vendor"].to_string(),
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

    let os_release = fs::read_to_string("/etc/os-release").expect("Failed to read /ect/os_release");
    for line in os_release.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        assert!(parts.len() == 2);
        let (lhs, rhs) = (parts[0], parts[1]);
        match lhs.to_lowercase().trim() {
            "name" => os_name = rhs.trim().to_string(),
            "version" => os_version = rhs.trim().to_string(),
            _ => continue,
        }
    }

    format!("{os_name} {os_version}")
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_hostname() {
        assert_eq!(hostname(), "iMac");
    }
}
