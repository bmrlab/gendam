use std::net::{TcpListener, UdpSocket};

pub(crate) fn get_available_port(start: u16, end: u16) -> Option<u16> {
    (start..end).find(|port| port_is_available(*port))
}

fn port_is_available(port: u16) -> bool {
    // make sure to use 0.0.0.0 to match
    match (
        TcpListener::bind(("0.0.0.0", port)),
        UdpSocket::bind(("0.0.0.0", port)),
    ) {
        (Ok(_), Ok(_)) => true,
        _ => false,
    }
}
