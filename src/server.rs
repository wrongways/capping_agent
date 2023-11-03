use crate::route::create_router;
use axum;
use std::fmt;
use std::net::{SocketAddr, ToSocketAddrs};


pub struct Server {
    pub listen_address: SocketAddr,
}


impl Server {
    pub fn new(listen_address: &str) -> Self {
        Self {
            listen_address: listen_address.to_socket_addrs()
                .expect("Failed to parse listend address {listen_address}")
                .next()
                .expect("Failed to get first socket address"),
        }
    }

    pub async fn run(&self) {
        println!("🚀 Server starting on {}", self);
        axum::Server::bind(&self.to_string().parse().unwrap())
            .serve(create_router().into_make_service())
            .await
            .unwrap();
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.listen_address)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let server = Server::new("localhost:8080");
        let expected = "[::1]:8080".to_string();
        assert_eq!(expected, format!("{server}"));

        let server = Server::new("www.ibm.com:80");
        let expected = "[2a02:26f0:2b00:3a7::1e89]:80".to_string();
        assert_eq!(expected, format!("{server}"));
    }
}
