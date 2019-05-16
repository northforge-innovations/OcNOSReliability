#[repr(C)]
#[allow(dead_code)]
pub struct IpAddrC {
    pub family: u8,
    pub addr: *mut u8,
}

#[repr(C)]
#[allow(dead_code)]
pub struct RouteEntry {
    pub prefix: IpAddrC,
    pub mask: IpAddrC,
    pub next_hop: IpAddrC,
    pub out_ifindex: u32,
}

#[repr(C)]
#[allow(dead_code)]
pub struct PeerEntry {
    pub prefix: IpAddrC,
    pub out_ifindex: u32,
}

#[repr(C)]
#[allow(dead_code)]
pub struct ForwardingEntry {
    pub next_hop: IpAddrC,
    pub out_ifindex: u32,
}

#[repr(C)]
#[allow(dead_code)]
pub struct IlmAddData {
    pub in_label: u32,
    pub in_iface: u32,
    pub next_hop: IpAddrC,
    pub out_ifindex: u32,
    pub out_label: u32,
    pub ilm_ix: u32,
    pub owner: u32,
}

#[repr(C)]
#[allow(dead_code)]
pub struct IlmDelData {
    pub in_label: u32,
    pub in_iface: u32,
    pub ilm_ix: u32,
    pub owner: u32,
}

#[repr(C)]
#[allow(dead_code)]
pub struct FtnAddData {
    pub fec: IpAddrC,
    pub ftn_ix: u32,
    pub next_hop: IpAddrC,
    pub out_ifindex: u32,
    pub out_label_number: u32,
    pub out_label: *mut u32,
}

#[repr(C)]
#[allow(dead_code)]
pub struct FtnDelData {
    pub fec: IpAddrC,
    pub ftn_ix: u32,
}

#[repr(C)]
#[allow(dead_code)]
pub struct NhAddDel {
    pub addr: IpAddrC,
    pub ifindex: u32,
    pub is_add: bool,
}
