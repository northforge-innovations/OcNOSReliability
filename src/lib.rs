#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;
use std::sync::Arc;
extern crate log;
extern crate patricia_tree;
use patricia_tree::*;
mod external_types;
use external_types::*;
mod simple_logger;
use log::*;
#[macro_use]
mod macros;
mod mpls_sim;
mod utils;
use utils::*;

//#[link(name = "c_callbacks")]
extern "C" {
    fn on_peer(ip_addr: &IpAddrC) -> i32;
}

type ROUTE_INT_ENTRY = Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>;

type PEER_INT_ENTRY = Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>;

type FORWARDING_INT_ENTRY = Arc<ReentrantMutex<RefCell<Box<ForwardingEntryInt>>>>;

type ROUTE_TABLE = Arc<ReentrantMutex<RefCell<HashMap<IpAddr, ROUTE_INT_ENTRY>>>>;
type PREFIX_TREE = Arc<ReentrantMutex<RefCell<PatriciaMap<FORWARDING_INT_ENTRY>>>>;

type PEER_TABLE = Arc<ReentrantMutex<RefCell<HashMap<IpAddr, PEER_INT_ENTRY>>>>;

lazy_static! {
    pub static ref ROUTE_TABLE_V4: ROUTE_TABLE =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref ROUTE_TABLE_V6: ROUTE_TABLE =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref PEER_TABLE_V4: PEER_TABLE =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref PEER_TABLE_V6: PEER_TABLE =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref PREFIX_TREE4: PREFIX_TREE =
        Arc::new(ReentrantMutex::new(RefCell::new(PatriciaMap::new())));
    pub static ref PREFIX_TREE6: PREFIX_TREE =
        Arc::new(ReentrantMutex::new(RefCell::new(PatriciaMap::new())));
}

pub enum PrefixTree {
    V4(&'static PREFIX_TREE4),
    V6(&'static PREFIX_TREE6),
}

pub struct ForwardingEntryInt {
    next_hop: IpAddr,
    out_ifindex: u32,
}

impl PrefixTree {
    fn insert(&self, key: IpAddr, fwd_entry: FORWARDING_INT_ENTRY) -> i32 {
        match self {
            PrefixTree::V4(_) => match key {
                IpAddr::V4(ipv4) => {
                    write_val!(PREFIX_TREE4).insert(ipv4.octets(), fwd_entry);
                    0
                }
                IpAddr::V6(_) => {
                    trace!("wrong argument type, expected ipv4");
                    -1
                }
            },

            PrefixTree::V6(_) => match key {
                IpAddr::V6(ipv6) => {
                    write_val!(PREFIX_TREE6).insert(ipv6.octets(), fwd_entry);
                    0
                }
                IpAddr::V4(_) => {
                    trace!("wrong argument type, expected ipv6");
                    -1
                }
            },
        }
    }

    fn get_longest_common_prefix(&self, key: IpAddr) -> Option<FORWARDING_INT_ENTRY> {
        match self {
            PrefixTree::V4(_) => match key {
                IpAddr::V4(ipv4) => {
                    match read_val!(PREFIX_TREE4).get_longest_common_prefix(&ipv4.octets()) {
                        Some((_k, v)) => Some(Arc::clone(v)),
                        None => None,
                    }
                }
                IpAddr::V6(_) => {
                    trace!("wrong argument type, expected ipv4");
                    None
                }
            },

            PrefixTree::V6(_) => match key {
                IpAddr::V6(ipv6) => {
                    match read_val!(PREFIX_TREE6).get_longest_common_prefix(&ipv6.octets()) {
                        Some((_k, v)) => Some(Arc::clone(v)),
                        None => None,
                    }
                }
                IpAddr::V4(_) => {
                    trace!("wrong argument type, expected ipv6");
                    None
                }
            },
        }
    }

    fn remove(&self, key: IpAddr) -> i32 {
        match self {
            PrefixTree::V4(_) => match key {
                IpAddr::V4(ipv4) => match write_val!(PREFIX_TREE4).remove(ipv4.octets()) {
                    Some(_rc) => 0,
                    None => -1,
                },
                IpAddr::V6(_) => {
                    trace!("wrong argument type, expected ipv4");
                    -1
                }
            },

            PrefixTree::V6(_) => match key {
                IpAddr::V6(ipv6) => match write_val!(PREFIX_TREE6).remove(ipv6.octets()) {
                    Some(_rc) => 0,
                    None => -1,
                },
                IpAddr::V4(_) => {
                    trace!("wrong argument type, expected ipv6");
                    -1
                }
            },
        }
    }
}

pub enum RouteTable {
    V4(&'static ROUTE_TABLE_V4),
    V6(&'static ROUTE_TABLE_V6),
}

impl RouteTable {
    fn contains_key(&self, ip_addr: &IpAddr) -> bool {
        match self {
            RouteTable::V4(_) => read_val!(ROUTE_TABLE_V4).contains_key(ip_addr),
            RouteTable::V6(_) => read_val!(ROUTE_TABLE_V6).contains_key(ip_addr),
        }
    }
    #[inline]
    fn get(&self, ip_addr: &IpAddr) -> ROUTE_INT_ENTRY {
        match self {
            RouteTable::V4(_) => Arc::clone(&read_val!(ROUTE_TABLE_V4)[ip_addr]),
            RouteTable::V6(_) => Arc::clone(&read_val!(ROUTE_TABLE_V6)[ip_addr]),
        }
    }

    fn insert(&self, ip_addr: &IpAddr, entry: ROUTE_INT_ENTRY) {
        match self {
            RouteTable::V4(_) => {
                write_val!(ROUTE_TABLE_V4).insert(*ip_addr, entry);
            }
            RouteTable::V6(_) => {
                write_val!(ROUTE_TABLE_V6).insert(*ip_addr, entry);
            }
        }
    }
    fn remove(&self, ip_addr: &IpAddr) {
        match self {
            RouteTable::V4(_) => {
                write_val!(ROUTE_TABLE_V4).remove(ip_addr);
            }
            RouteTable::V6(_) => {
                write_val!(ROUTE_TABLE_V6).remove(ip_addr);
            }
        }
    }
    fn clone(&self, ip_addr: &IpAddr) -> ROUTE_INT_ENTRY {
        match self {
            RouteTable::V4(_) => Arc::clone(&read_val!(ROUTE_TABLE_V4)[ip_addr]),
            RouteTable::V6(_) => Arc::clone(&read_val!(ROUTE_TABLE_V6)[ip_addr]),
        }
    }
    fn add_modify(
        &mut self,
        route_prefix: &IpAddr,
        route_mask: &IpAddr,
        next_hop_addr: &IpAddr,
        out_ifindex: u32,
        peer_ip_addr: &IpAddr,
        peer_table: &PeerTable,
    ) -> Option<ROUTE_INT_ENTRY> {
        let route_entry: ROUTE_INT_ENTRY;
        if self.contains_key(route_prefix) {
            let re: ROUTE_INT_ENTRY = self.get(&route_prefix);
            write_val!(re).next_hop = *next_hop_addr;
            write_val!(re).out_ifindex = out_ifindex;
            write_val!(re).mask = *route_mask;
            write_val!(re).creator = *peer_ip_addr;
            if !read_val!(re).peer_exists(*peer_ip_addr) {
                trace!(
                    "RouteTable::add_modify: peer {} is not present in peer list, adding",
                    peer_ip_addr
                );
                write_val!(re).add_peer(*peer_ip_addr, peer_table.clone(peer_ip_addr));
            }
            trace!(
                "RouteTable::add_modify: cloned route entry prefix {} for peer {}",
                route_prefix,
                peer_ip_addr
            );
            return Some(self.clone(route_prefix));
        } else {
            let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
                RouteIntEntry::new(
                    *route_prefix,
                    *route_mask,
                    *next_hop_addr,
                    out_ifindex,
                    *peer_ip_addr,
                ),
            ))));
            route_entry = Arc::clone(&new_entry);
            self.insert(&route_prefix, new_entry);
            write_val!(route_entry).add_peer(*peer_ip_addr, peer_table.clone(peer_ip_addr));
            trace!(
                "RouteTable::add_modify: new route entry prefix {} for peer {}",
                route_prefix,
                peer_ip_addr
            );
        }
        Some(route_entry)
    }
}

pub enum PeerTable {
    V4(&'static PEER_TABLE_V4),
    V6(&'static PEER_TABLE_V6),
}

impl PeerTable {
    fn contains_key(&self, ip_addr: &IpAddr) -> bool {
        match self {
            PeerTable::V4(_) => read_val!(PEER_TABLE_V4).contains_key(ip_addr),
            PeerTable::V6(_) => read_val!(PEER_TABLE_V6).contains_key(ip_addr),
        }
    }
    fn get(&self, ip_addr: &IpAddr) -> PEER_INT_ENTRY {
        match self {
            PeerTable::V4(_) => Arc::clone(&read_val!(PEER_TABLE_V4)[ip_addr]),
            PeerTable::V6(_) => Arc::clone(&read_val!(PEER_TABLE_V6)[ip_addr]),
        }
    }

    fn insert(&self, ip_addr: &IpAddr, entry: PEER_INT_ENTRY) {
        match self {
            PeerTable::V4(_) => {
                write_val!(PEER_TABLE_V4).insert(*ip_addr, entry);
            }
            PeerTable::V6(_) => {
                write_val!(PEER_TABLE_V6).insert(*ip_addr, entry);
            }
        }
    }
    fn remove(&self, ip_addr: &IpAddr) {
        match self {
            PeerTable::V4(_) => {
                if self.contains_key(ip_addr) {
                    write_val!(read_val!(PEER_TABLE_V4)[ip_addr])
                        .cleanup(&RouteTable::V4(&ROUTE_TABLE_V4));
                }
                write_val!(PEER_TABLE_V4).remove(ip_addr);
            }
            PeerTable::V6(_) => {
                if self.contains_key(ip_addr) {
                    write_val!(read_val!(PEER_TABLE_V6)[ip_addr])
                        .cleanup(&RouteTable::V6(&ROUTE_TABLE_V6));
                }
                write_val!(PEER_TABLE_V6).remove(ip_addr);
            }
        }
    }
    fn clone(&self, ip_addr: &IpAddr) -> PEER_INT_ENTRY {
        match self {
            PeerTable::V4(_) => Arc::clone(&read_val!(PEER_TABLE_V4)[ip_addr]),
            PeerTable::V6(_) => Arc::clone(&read_val!(PEER_TABLE_V6)[ip_addr]),
        }
    }
    fn _iterate(&self, keys_vals: Iter<IpAddr, PEER_INT_ENTRY>) {
        for (key, val) in keys_vals {
            println!("key: {} val: {}", key, read_val!(val).prefix);
            unsafe {
                let ip: IpAddrC = IpAddrC {
                    family: 1,
                    addr: ptr::null_mut(),
                };
                if on_peer(&ip) == 1 {
                    break;
                }
            }
        }
    }
    fn iterate(&mut self) {
        match self {
            PeerTable::V4(_) => self._iterate(read_val!(PEER_TABLE_V4).iter()),
            PeerTable::V6(_) => self._iterate(read_val!(PEER_TABLE_V6).iter()),
        }
    }
}

pub struct RouteIntEntry {
    prefix: IpAddr,
    mask: IpAddr,
    next_hop: IpAddr,
    out_ifindex: u32,
    peer_table: PEER_TABLE,
    creator: IpAddr,
}

impl Clone for RouteIntEntry {
    fn clone(&self) -> Self {
        RouteIntEntry {
            prefix: self.prefix,
            mask: self.mask,
            next_hop: self.next_hop,
            out_ifindex: self.out_ifindex,
            peer_table: Arc::clone(&self.peer_table),
            creator: self.creator,
        }
    }
}

impl RouteIntEntry {
    pub fn new(
        _prefix: IpAddr,
        _mask: IpAddr,
        _next_hop: IpAddr,
        _out_ifindex: u32,
        _creator: IpAddr,
    ) -> RouteIntEntry {
        RouteIntEntry {
            prefix: _prefix,
            mask: _mask,
            next_hop: _next_hop,
            out_ifindex: _out_ifindex,
            peer_table: Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new()))),
            creator: _creator,
        }
    }
    pub fn add_peer(&mut self, peer_prefix: IpAddr, peer: PEER_INT_ENTRY) -> i32 {
        if read_val!(self.peer_table).contains_key(&peer_prefix) {
            trace!("RouteIntEntry::add_peer {} already exists", peer_prefix);
            -1
        } else {
            trace!("RouteIntEntry::add_peer {}", peer_prefix);
            write_val!(self.peer_table).insert(peer_prefix, Arc::clone(&peer));
            0
        }
    }
    pub fn delete_peer(&self, peer_prefix: IpAddr, route_table: &RouteTable) -> i32 {
        trace!("RouteIntEntry::delete_peer {}", peer_prefix);
        let rc: i32;
        if read_val!(self.peer_table).contains_key(&peer_prefix) {
            write_val!(self.peer_table).remove(&peer_prefix);
            rc = 0;
        } else {
            trace!("RouteIntEntry::delete_peer {} does not exist", peer_prefix);
            rc = -1;
        }
        if self.get_number_of_peers() == 0 {
            trace!("RouteIntEntry::delete_peer {} last gone", peer_prefix);
            route_table.remove(&self.prefix);
        }
        return rc;
    }
    pub fn peer_exists(&self, _peer_prefix: IpAddr) -> bool {
        read_val!(self.peer_table).contains_key(&_peer_prefix)
    }
    pub fn get_number_of_peers(&self) -> usize {
        read_val!(self.peer_table).len()
    }
}

fn _route_lookup(ip_addr: &IpAddr, route_table: &RouteTable, _entry: *mut RouteEntry) -> i32 {
    if !route_table.contains_key(&ip_addr) {
        trace!("route_lookup: cannot find prefix {}", ip_addr);
        return -1;
    }
    let re: ROUTE_INT_ENTRY = route_table.get(&ip_addr);

    trace!(
        "route_lookup prefix {}, found prefix {} mask {} next_hop {} out_ifindex {}",
        ip_addr,
        read_val!(re).prefix,
        read_val!(re).mask,
        read_val!(re).next_hop,
        read_val!(re).out_ifindex
    );
    unsafe {
        let mut addr_ptr: *mut u8 = (*_entry).prefix.addr;
        copy_ip_addr_to_user(addr_ptr, &read_val!(re).prefix);
        addr_ptr = (*_entry).mask.addr;
        copy_ip_addr_to_user(addr_ptr, &read_val!(re).mask);
        addr_ptr = (*_entry).next_hop.addr;
        copy_ip_addr_to_user(addr_ptr, &read_val!(re).next_hop);
        (*_entry).out_ifindex = read_val!(re).out_ifindex;
    }
    0
}

#[no_mangle]
pub extern "C" fn route_lookup(_prefix: &IpAddrC, _entry: *mut RouteEntry) -> i32 {
    let ip_addr;

    unsafe {
        if _prefix.family == 1 {
            ip_addr = copy_ip_addr_v4_from_user(_prefix.addr);
            _route_lookup(&ip_addr, &RouteTable::V4(&ROUTE_TABLE_V4), _entry)
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            _route_lookup(&ip_addr, &RouteTable::V6(&ROUTE_TABLE_V6), _entry)
        }
    }
}

pub struct PeerIntEntry {
    prefix: IpAddr,
    out_ifindex: u32,
    peer_route_table: ROUTE_TABLE,
}

impl Drop for PeerIntEntry {
    fn drop(&mut self) {
        trace!("PeerIntEntry::Drop {}", self.prefix);
        match self.prefix {
            IpAddr::V4(_ipv4) => {
                self.cleanup(&RouteTable::V4(&ROUTE_TABLE_V4));
            }
            IpAddr::V6(_ipv6) => {
                self.cleanup(&RouteTable::V6(&ROUTE_TABLE_V6));
            }
        }
    }
}

impl PeerIntEntry {
    pub fn new(_prefix: IpAddr, _out_ifindex: u32) -> PeerIntEntry {
        PeerIntEntry {
            prefix: _prefix,
            out_ifindex: _out_ifindex,
            peer_route_table: Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new()))),
        }
    }
    fn cleanup(&mut self, route_table: &RouteTable) {
        trace!("PeerIntEntry::cleanup");
        for val in read_val!(self.peer_route_table).values() {
            write_val!(val).delete_peer(self.prefix, route_table);
        }
    }
    pub fn route_add_modify(
        &mut self,
        route_prefix: &IpAddr,
        route_mask: &IpAddr,
        next_hop_addr: &IpAddr,
        out_ifindex: u32,
        peer_ip_addr: &IpAddr,
        route_table: &mut RouteTable,
        peer_table: &mut PeerTable,
    ) -> i32 {
        let route_entry: Option<ROUTE_INT_ENTRY>;
        trace!(
            "PeerIntEntry::route_add_modify prefix {} peer {}",
            route_prefix,
            peer_ip_addr
        );
        route_entry = route_table.add_modify(
            route_prefix,
            route_mask,
            next_hop_addr,
            out_ifindex,
            peer_ip_addr,
            peer_table,
        );

        if !read_val!(self.peer_route_table).contains_key(&route_prefix) {
            trace!(
                "PeerIntEntry::route_add_modify prefix {} not found in peer rt. adding",
                route_prefix
            );
            match route_entry {
                Some(re) => {
                    write_val!(self.peer_route_table).insert(read_val!(re).prefix, Arc::clone(&re));
                    return 0;
                }
                None => {
                    trace!("expected route_entry, found nothing!");
                    return -1;
                }
            }
        } else {
            return 1;
        }
    }
}

fn _peer_add_modify(ip_addr: &IpAddr, peer_table: &PeerTable, _entry: *mut PeerEntry) -> i32 {
    let mut entry: Box<PeerEntry>;

    unsafe {
        entry = Box::from_raw(_entry);
    }
    trace!(
        "peer_add_modify: key: {} out_ifindex: {}",
        ip_addr,
        entry.out_ifindex
    );
    if peer_table.contains_key(&ip_addr) {
        let pe: PEER_INT_ENTRY = peer_table.get(&ip_addr);
        trace!("found existing. modifying");
        write_val!(pe).out_ifindex = entry.out_ifindex;

        let _m_entry = Box::into_raw(entry);
        1
    } else {
        trace!("creating new");
        let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
            PeerIntEntry::new(*ip_addr, entry.out_ifindex),
        ))));
        let _m_entry = Box::into_raw(entry);
        peer_table.insert(&ip_addr, new_entry);
        0
    }
}

#[no_mangle]
pub extern "C" fn peer_add_modify(_prefix: &IpAddrC, _entry: *mut PeerEntry) -> i32 {
    let ip_addr: IpAddr;

    unsafe {
        if _prefix.family == 1 {
            ip_addr = copy_ip_addr_v4_from_user(_prefix.addr);
            _peer_add_modify(&ip_addr, &PeerTable::V4(&PEER_TABLE_V4), _entry)
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            _peer_add_modify(&ip_addr, &PeerTable::V6(&PEER_TABLE_V6), _entry)
        }
    }
}

fn _peer_lookup(ip_addr: &IpAddr, peer_table: &PeerTable, _entry: *mut PeerEntry) -> i32 {
    trace!("peer_lookup {}", ip_addr);
    if !peer_table.contains_key(&ip_addr) {
        trace!("not found");
        return -1;
    }
    let pe = peer_table.get(&ip_addr);
    unsafe {
        let addr_ptr: *mut u8 = (*_entry).prefix.addr;
        copy_ip_addr_to_user(addr_ptr, &read_val!(pe).prefix);
        (*_entry).out_ifindex = read_val!(pe).out_ifindex;
    }
    0
}

#[no_mangle]
pub extern "C" fn peer_lookup(_prefix: &IpAddrC, _entry: *mut PeerEntry) -> i32 {
    let ip_addr: IpAddr;

    unsafe {
        if _prefix.family == 1 {
            ip_addr = copy_ip_addr_v4_from_user(_prefix.addr);
            _peer_lookup(&ip_addr, &PeerTable::V4(&PEER_TABLE_V4), _entry)
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            _peer_lookup(&ip_addr, &PeerTable::V6(&PEER_TABLE_V6), _entry)
        }
    }
}

fn _peer_delete(ip_addr: &IpAddr, peer_table: &PeerTable) -> i32 {
    trace!("peer_delete {}", ip_addr);
    if !peer_table.contains_key(&ip_addr) {
        trace!("not found");
        return -1;
    }
    peer_table.remove(&ip_addr);
    0
}

#[no_mangle]
pub extern "C" fn peer_delete(_prefix: &IpAddrC) -> i32 {
    let ip_addr: IpAddr;
    unsafe {
        if _prefix.family == 1 {
            ip_addr = copy_ip_addr_v4_from_user(_prefix.addr);
            _peer_delete(&ip_addr, &PeerTable::V4(&PEER_TABLE_V4))
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            _peer_delete(&ip_addr, &PeerTable::V6(&PEER_TABLE_V6))
        }
    }
}

fn _peer_route_add_modify(
    peer_ip_addr: &IpAddr,
    route_prefix: &IpAddr,
    route_mask: &IpAddr,
    next_hop_addr: &IpAddr,
    peer_table: &mut PeerTable,
    route_table: &mut RouteTable,
    entry: Box<RouteEntry>,
) -> i32 {
    trace!(
        "peer_route_add_modify: peer {} prefix {}",
        peer_ip_addr,
        route_prefix
    );
    if !peer_table.contains_key(&peer_ip_addr) {
        trace!("not found");
        return -1;
    }
    let pe: PEER_INT_ENTRY = peer_table.get(&peer_ip_addr);
    let rc = write_val!(pe).route_add_modify(
        route_prefix,
        route_mask,
        next_hop_addr,
        entry.out_ifindex,
        peer_ip_addr,
        route_table,
        peer_table,
    );
    let _m_entry = Box::into_raw(entry);
    rc
}

#[no_mangle]
pub extern "C" fn peer_route_add_modify(_peer_prefix: &IpAddrC, _entry: *mut RouteEntry) -> i32 {
    let peer_ip_addr: IpAddr;
    let route_prefix: IpAddr;
    let route_mask: IpAddr;
    let next_hop_addr: IpAddr;
    let mut entry: Box<RouteEntry>;

    unsafe {
        entry = Box::from_raw(_entry);
        if _peer_prefix.family == 1 {
            peer_ip_addr = copy_ip_addr_v4_from_user(_peer_prefix.addr);
            route_prefix = copy_ip_addr_v4_from_user(entry.prefix.addr);
            route_mask = copy_ip_addr_v4_from_user(entry.mask.addr);
            next_hop_addr = copy_ip_addr_v4_from_user(entry.next_hop.addr);
            _peer_route_add_modify(
                &peer_ip_addr,
                &route_prefix,
                &route_mask,
                &next_hop_addr,
                &mut PeerTable::V4(&PEER_TABLE_V4),
                &mut RouteTable::V4(&ROUTE_TABLE_V4),
                entry,
            )
        } else {
            peer_ip_addr = copy_ip_addr_v6_from_user(_peer_prefix.addr as *mut u16);
            route_prefix = copy_ip_addr_v6_from_user(entry.prefix.addr as *mut u16);
            route_mask = copy_ip_addr_v6_from_user(entry.mask.addr as *mut u16);
            next_hop_addr = copy_ip_addr_v6_from_user(entry.next_hop.addr as *mut u16);
            _peer_route_add_modify(
                &peer_ip_addr,
                &route_prefix,
                &route_mask,
                &next_hop_addr,
                &mut PeerTable::V6(&PEER_TABLE_V6),
                &mut RouteTable::V6(&ROUTE_TABLE_V6),
                entry,
            )
        }
    }
}

fn _peer_route_lookup(
    peer_ip_addr: &IpAddr,
    route_prefix: &IpAddr,
    peer_table: &PeerTable,
    _entry: *mut RouteEntry,
) -> i32 {
    trace!(
        "peer_route_lookup: peer {} prefix {}",
        peer_ip_addr,
        route_prefix
    );
    if !peer_table.contains_key(&peer_ip_addr) {
        trace!("peer not found");
        return -1;
    }
    let pe: PEER_INT_ENTRY = peer_table.get(&peer_ip_addr);
    if !read_val!(read_val!(pe).peer_route_table).contains_key(&route_prefix) {
        trace!("prefix not found in peer rt");
        return -2;
    }
    unsafe {
        trace!(
            "peer_route_lookup peer_prefix {}, found prefix {}",
            peer_ip_addr,
            route_prefix
        );
        let mut addr_ptr: *mut u8 = (*_entry).prefix.addr;
        copy_ip_addr_to_user(
            addr_ptr,
            &read_val!(read_val!(read_val!(pe).peer_route_table)[&route_prefix]).prefix,
        );
        addr_ptr = (*_entry).mask.addr;
        copy_ip_addr_to_user(
            addr_ptr,
            &read_val!(read_val!(read_val!(pe).peer_route_table)[&route_prefix]).mask,
        );
        addr_ptr = (*_entry).next_hop.addr;
        copy_ip_addr_to_user(
            addr_ptr,
            &read_val!(read_val!(read_val!(pe).peer_route_table)[&route_prefix]).next_hop,
        );
        (*_entry).out_ifindex =
            read_val!(read_val!(read_val!(pe).peer_route_table)[&route_prefix]).out_ifindex;
    }
    0
}

#[no_mangle]
pub extern "C" fn peer_route_lookup(
    _peer_prefix: &IpAddrC,
    _route_prefix: &IpAddrC,
    _entry: *mut RouteEntry,
) -> i32 {
    let peer_ip_addr: IpAddr;
    let route_prefix: IpAddr;

    unsafe {
        if _peer_prefix.family == 1 {
            peer_ip_addr = copy_ip_addr_v4_from_user(_peer_prefix.addr);
            route_prefix = copy_ip_addr_v4_from_user(_route_prefix.addr);
            _peer_route_lookup(
                &peer_ip_addr,
                &route_prefix,
                &PeerTable::V4(&PEER_TABLE_V4),
                _entry,
            )
        } else {
            peer_ip_addr = copy_ip_addr_v6_from_user(_peer_prefix.addr as *mut u16);
            route_prefix = copy_ip_addr_v6_from_user(_route_prefix.addr as *mut u16);
            _peer_route_lookup(
                &peer_ip_addr,
                &route_prefix,
                &PeerTable::V6(&PEER_TABLE_V6),
                _entry,
            )
        }
    }
}

fn _peer_route_delete(peer_ip_addr: &IpAddr, route_prefix: &IpAddr, peer_table: &PeerTable) -> i32 {
    trace!(
        "peer_route_delete peer {} prefix {}",
        peer_ip_addr,
        route_prefix
    );
    if !peer_table.contains_key(&peer_ip_addr) {
        trace!("peer not found");
        return -1;
    }
    let pe: PEER_INT_ENTRY = peer_table.get(&peer_ip_addr);

    if !read_val!(read_val!(pe).peer_route_table).contains_key(&route_prefix) {
        trace!(
            "cannot find route {} for peer {}",
            route_prefix,
            peer_ip_addr
        );
        return -2;
    }
    peer_table.remove(&route_prefix);
    0
}

#[no_mangle]
pub extern "C" fn peer_route_delete(_peer_prefix: &IpAddrC, _route_prefix: &IpAddrC) -> i32 {
    let peer_ip_addr: IpAddr;
    let route_prefix: IpAddr;

    unsafe {
        if _peer_prefix.family == 1 {
            peer_ip_addr = copy_ip_addr_v4_from_user(_peer_prefix.addr);
            route_prefix = copy_ip_addr_v4_from_user(_route_prefix.addr);
            _peer_route_delete(&peer_ip_addr, &route_prefix, &PeerTable::V4(&PEER_TABLE_V4))
        } else {
            peer_ip_addr = copy_ip_addr_v6_from_user(_peer_prefix.addr as *mut u16);
            route_prefix = copy_ip_addr_v6_from_user(_route_prefix.addr as *mut u16);
            _peer_route_delete(&peer_ip_addr, &route_prefix, &PeerTable::V6(&PEER_TABLE_V6))
        }
    }
}

fn _peer_iterate(peer_table: &mut PeerTable, route_table: &RouteTable) {
    peer_table.iterate();
}

#[no_mangle]
pub extern "C" fn peer_iterate(address_family: u32) {
    if address_family == 1 {
        _peer_iterate(
            &mut PeerTable::V4(&PEER_TABLE_V4),
            &RouteTable::V4(&ROUTE_TABLE_V4),
        );
    } else {
        _peer_iterate(
            &mut PeerTable::V6(&PEER_TABLE_V6),
            &RouteTable::V6(&ROUTE_TABLE_V6),
        );
    }
}

fn _longest_match_lookup(
    ip_addr: &IpAddr,
    prefix_tree: &PrefixTree,
    _entry: *mut ForwardingEntry,
) -> i32 {
    match prefix_tree.get_longest_common_prefix(*ip_addr) {
        Some(fe) => {
            trace!(
                "longest_match_lookup prefix {}, found next_hop {} out_ifindex {}",
                ip_addr,
                read_val!(fe).next_hop,
                read_val!(fe).out_ifindex
            );
            unsafe {
                let addr_ptr: *mut u8 = (*_entry).next_hop.addr;
                copy_ip_addr_to_user(addr_ptr, &read_val!(fe).next_hop);
                (*_entry).out_ifindex = read_val!(fe).out_ifindex;
            }
            0
        }
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn longest_match_lookup(_prefix: &IpAddrC, _entry: *mut ForwardingEntry) -> i32 {
    let ip_addr;

    unsafe {
        if _prefix.family == 1 {
            ip_addr = copy_ip_addr_v4_from_user(_prefix.addr);
            _longest_match_lookup(&ip_addr, &PrefixTree::V4(&PREFIX_TREE4), _entry)
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            _longest_match_lookup(&ip_addr, &PrefixTree::V6(&PREFIX_TREE6), _entry)
        }
    }
}

fn _longest_match_add(ip_addr: &IpAddr, prefix_tree: &PrefixTree, fe: ForwardingEntryInt) -> i32 {
    let new_fe: FORWARDING_INT_ENTRY;
    new_fe = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(fe))));
    prefix_tree.insert(*ip_addr, new_fe)
}

#[no_mangle]
pub extern "C" fn longest_match_add(_prefix: &IpAddrC, _entry: *mut ForwardingEntry) -> i32 {
    let ip_addr;
    let rc;

    unsafe {
        let user_fe: Box<ForwardingEntry> = Box::from_raw(_entry);
        if _prefix.family == 1 {
            ip_addr = copy_ip_addr_v4_from_user(_prefix.addr);
            let fe: ForwardingEntryInt = ForwardingEntryInt {
                next_hop: copy_ip_addr_v4_from_user(user_fe.next_hop.addr),
                out_ifindex: user_fe.out_ifindex,
            };
            rc = _longest_match_add(&ip_addr, &PrefixTree::V4(&PREFIX_TREE4), fe);
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            let fe: ForwardingEntryInt = ForwardingEntryInt {
                next_hop: copy_ip_addr_v6_from_user(user_fe.next_hop.addr as *mut u16),
                out_ifindex: user_fe.out_ifindex,
            };
            rc = _longest_match_add(&ip_addr, &PrefixTree::V6(&PREFIX_TREE6), fe);
        }
        Box::into_raw(user_fe);
    }
    return rc;
}

fn _longest_match_delete(ip_addr: &IpAddr, prefix_tree: &PrefixTree) -> i32 {
    prefix_tree.remove(*ip_addr)
}

#[no_mangle]
pub extern "C" fn longest_match_delete(_prefix: &IpAddrC) -> i32 {
    let ip_addr;
    let rc;

    unsafe {
        if _prefix.family == 1 {
            ip_addr = copy_ip_addr_v4_from_user(_prefix.addr);
            rc = _longest_match_delete(&ip_addr, &PrefixTree::V4(&PREFIX_TREE4));
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            rc = _longest_match_delete(&ip_addr, &PrefixTree::V6(&PREFIX_TREE6));
        }
    }
    return rc;
}

use std::alloc::{GlobalAlloc, Layout};

extern "C" {
    fn pool_alloc(size: usize) -> *mut u8;
    fn pool_free(ptr: *mut u8);
}

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        //System.alloc(layout)
        pool_alloc(_layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        //System.dealloc(ptr, layout)
        pool_free(ptr);
    }
}

#[global_allocator]
static GLOBAL: MyAllocator = MyAllocator;
