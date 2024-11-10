use std::net::Ipv4Addr;
use std::sync::OnceLock;

static IP_ADDRESSES: OnceLock<Vec<Ipv4Addr>> = OnceLock::new();

fn initialize_ips() -> Vec<Ipv4Addr> {
    vec![
        Ipv4Addr::new(192, 168, 0, 1),
        Ipv4Addr::new(192, 168, 0, 2),
        Ipv4Addr::new(192, 168, 0, 3),
    ]
}

fn get_ip_addresses() -> &'static Vec<Ipv4Addr> {
    IP_ADDRESSES.get_or_init(|| initialize_ips())
}

fn main() {
    // Accesăm variabila IP_ADDRESSES, care va fi inițializată la primul acces
    let ips = get_ip_addresses();
    println!("IP addresses: {:?}", ips);
}
