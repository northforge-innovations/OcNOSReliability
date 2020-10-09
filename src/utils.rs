use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[allow(dead_code)]
pub unsafe fn copy_ip_addr_to_user(addr_ptr: *mut u8, addr: &IpAddr) {
    match addr {
        IpAddr::V4(ipv4) => {
            for i in 0..4 {
                *addr_ptr.wrapping_add(i) = ipv4.octets()[i];
            }
        }
        IpAddr::V6(ipv6) => {
            for i in 0..16 {
                *addr_ptr.wrapping_add(i) = ipv6.octets()[i];
            }
        }
    }
}

pub unsafe fn copy_ip_addr_v4_from_user(addr_ptr: *mut u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(
        *addr_ptr,
        *addr_ptr.wrapping_add(1),
        *addr_ptr.wrapping_add(2),
        *addr_ptr.wrapping_add(3),
    ))
}

pub unsafe fn copy_ip_addr_v6_from_user(addr_ptr: *mut u16) -> IpAddr {
    IpAddr::V6(Ipv6Addr::new(
        *addr_ptr,
        *addr_ptr.wrapping_add(1),
        *addr_ptr.wrapping_add(2),
        *addr_ptr.wrapping_add(3),
        *addr_ptr.wrapping_add(4),
        *addr_ptr.wrapping_add(5),
        *addr_ptr.wrapping_add(6),
        *addr_ptr.wrapping_add(7),
    ))
}
