#[repr(C)]
pub struct IpAddrC {
    pub family: u8,
    pub addr: *mut u8,
}

#[repr(C)]
pub struct RouteEntry {
    pub prefix: IpAddrC,
    pub mask: IpAddrC,
    pub next_hop: IpAddrC,
    pub out_ifindex: u32,
}

#[repr(C)]
pub struct PeerEntry {
    pub prefix: IpAddrC,
    pub out_ifindex: u32,
}

#[repr(C)]
pub struct ForwardingEntry {
    pub next_hop: IpAddrC,
    pub out_ifindex: u32,
}

#[repr(C)]
pub struct IlmAddData {
    pub in_label: u32,
    pub in_iface: u32,
    pub next_hop: IpAddrC,
    pub out_ifindex: u32,
    pub out_label: u32,
    pub ilm_idx: u32,
}

#[repr(C)]
pub struct IlmDelData {
    pub in_label: u32,
    pub in_iface: u32,
    pub ilm_idx: u32,
}

#[repr(C)]
pub struct FtnAddData {
    pub fec: IpAddrC,
    pub ftn_ix: u32,
    pub next_hop: IpAddrC,
    pub out_ifindex: u32,
    pub out_label_number: u32,
    pub out_label: *mut u32,
}

#[repr(C)]
pub struct FtnDelData {
    pub fec: IpAddrC,
    pub ftn_ix: u32,
}
