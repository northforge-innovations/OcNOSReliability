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

//#[link(name = "c_callbacks")]
extern "C" {
    fn on_peer(ip_addr: &IpAddrC) -> i32;
}

#[repr(C)]
pub struct IpAddrC {
    family: u8,
    addr: *mut u8,
}

#[repr(C)]
pub struct RouteEntry {
    prefix: IpAddrC,
    mask: IpAddrC,
    next_hop: IpAddrC,
    out_ifindex: u32,
}

#[repr(C)]
pub struct PeerEntry {
    prefix: IpAddrC,
    out_ifindex: u32,
}

#[repr(C)]
pub struct ForwardingEntry {
    next_hop: IpAddrC,
    out_ifindex: u32,
}

lazy_static! {
    pub static ref ROUTE_TABLE_V4: Arc<ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref ROUTE_TABLE_V6: Arc<ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref PREFIX_TREE4: Arc<ReentrantMutex<RefCell<PatriciaMap<Arc<ReentrantMutex<RefCell<Box<ForwardingEntryInt>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(PatriciaMap::new())));
    pub static ref PREFIX_TREE6: Arc<ReentrantMutex<RefCell<PatriciaMap<Arc<ReentrantMutex<RefCell<Box<ForwardingEntryInt>>>>>>>> =
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
    fn insert(
        &self,
        key: IpAddr,
        fwd_entry: Arc<ReentrantMutex<RefCell<Box<ForwardingEntryInt>>>>,
    ) -> i32 {
        match self {
            PrefixTree::V4(_) => match key {
                IpAddr::V4(ipv4) => {
                    PREFIX_TREE4
                        .lock()
                        .borrow_mut()
                        .insert(ipv4.octets(), fwd_entry);
                    0
                }
                IpAddr::V6(_) => {
                    trace!("wrong argument type, expected ipv4");
                    -1
                }
            },

            PrefixTree::V6(_) => match key {
                IpAddr::V6(ipv6) => {
                    PREFIX_TREE6
                        .lock()
                        .borrow_mut()
                        .insert(ipv6.octets(), fwd_entry);
                    0
                }
                IpAddr::V4(_) => {
                    trace!("wrong argument type, expected ipv6");
                    -1
                }
            },
        }
    }

    fn get_longest_common_prefix(
        &self,
        key: IpAddr,
    ) -> Option<Arc<ReentrantMutex<RefCell<Box<ForwardingEntryInt>>>>> {
        match self {
            PrefixTree::V4(_) => match key {
                IpAddr::V4(ipv4) => match PREFIX_TREE4
                    .lock()
                    .borrow()
                    .get_longest_common_prefix(&ipv4.octets())
                {
                    Some((k, v)) => Some(Arc::clone(v)),
                    None => None,
                },
                IpAddr::V6(_) => {
                    trace!("wrong argument type, expected ipv4");
                    None
                }
            },

            PrefixTree::V6(_) => match key {
                IpAddr::V6(ipv6) => match PREFIX_TREE6
                    .lock()
                    .borrow()
                    .get_longest_common_prefix(&ipv6.octets())
                {
                    Some((k, v)) => Some(Arc::clone(v)),
                    None => None,
                },
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
                IpAddr::V4(ipv4) => match PREFIX_TREE4.lock().borrow_mut().remove(ipv4.octets()) {
                    Some(rc) => 0,
                    None => -1,
                },
                IpAddr::V6(_) => {
                    trace!("wrong argument type, expected ipv4");
                    -1
                }
            },

            PrefixTree::V6(_) => match key {
                IpAddr::V6(ipv6) => match PREFIX_TREE6.lock().borrow_mut().remove(ipv6.octets()) {
                    Some(rc) => 0,
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
            RouteTable::V4(_) => ROUTE_TABLE_V4.lock().borrow().contains_key(ip_addr),
            RouteTable::V6(_) => ROUTE_TABLE_V6.lock().borrow().contains_key(ip_addr),
        }
    }
    #[inline]
    fn get(&self, ip_addr: &IpAddr) -> *const Box<RouteIntEntry> {
        match self {
            RouteTable::V4(_) => &(*ROUTE_TABLE_V4.lock().borrow()[ip_addr].lock().borrow()),
            RouteTable::V6(_) => &(*ROUTE_TABLE_V6.lock().borrow()[ip_addr].lock().borrow()),
        }
    }
    fn get_mut(&self, ip_addr: &IpAddr) -> *mut Box<RouteIntEntry> {
        match self {
            RouteTable::V4(_) => {
                &mut (*ROUTE_TABLE_V4.lock().borrow()[ip_addr].lock().borrow_mut())
            }
            RouteTable::V6(_) => {
                &mut (*ROUTE_TABLE_V6.lock().borrow()[ip_addr].lock().borrow_mut())
            }
        }
    }
    fn insert(&self, ip_addr: &IpAddr, entry: Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>) {
        match self {
            RouteTable::V4(_) => {
                ROUTE_TABLE_V4.lock().borrow_mut().insert(*ip_addr, entry);
            }
            RouteTable::V6(_) => {
                ROUTE_TABLE_V6.lock().borrow_mut().insert(*ip_addr, entry);
            }
        }
    }
    fn remove(&self, ip_addr: &IpAddr) {
        match self {
            RouteTable::V4(_) => {
                ROUTE_TABLE_V4.lock().borrow_mut().remove(ip_addr);
            }
            RouteTable::V6(_) => {
                ROUTE_TABLE_V6.lock().borrow_mut().remove(ip_addr);
            }
        }
    }
    fn clone(&self, ip_addr: &IpAddr) -> Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>> {
        match self {
            RouteTable::V4(_) => Arc::clone(&ROUTE_TABLE_V4.lock().borrow()[ip_addr]),
            RouteTable::V6(_) => Arc::clone(&ROUTE_TABLE_V6.lock().borrow()[ip_addr]),
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
    ) -> Option<Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>> {
        let route_entry: Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>;
        if self.contains_key(route_prefix) {
            let re: &mut Box<RouteIntEntry>;
            unsafe {
                re = &mut *self.get_mut(&route_prefix);
            }
            re.next_hop = *next_hop_addr;
            re.out_ifindex = out_ifindex;
            re.mask = *route_mask;
            re.creator = *peer_ip_addr;
            if !re.peer_exists(*peer_ip_addr) {
                trace!(
                    "RouteTable::add_modify: peer {} is not present in peer list, adding",
                    peer_ip_addr
                );
                re.add_peer(*peer_ip_addr, peer_table.clone(peer_ip_addr));
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
            route_entry
                .lock()
                .borrow_mut()
                .add_peer(*peer_ip_addr, peer_table.clone(peer_ip_addr));
            trace!(
                "RouteTable::add_modify: new route entry prefix {} for peer {}",
                route_prefix,
                peer_ip_addr
            );
        }
        Some(route_entry)
    }
}

lazy_static! {
    pub static ref PEER_TABLE_V4: Arc<ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref PEER_TABLE_V6: Arc<ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
}

pub enum PeerTable {
    V4(&'static PEER_TABLE_V4),
    V6(&'static PEER_TABLE_V6),
}

impl PeerTable {
    fn contains_key(&self, ip_addr: &IpAddr) -> bool {
        match self {
            PeerTable::V4(_) => PEER_TABLE_V4.lock().borrow().contains_key(ip_addr),
            PeerTable::V6(_) => PEER_TABLE_V6.lock().borrow().contains_key(ip_addr),
        }
    }
    fn get(&self, ip_addr: &IpAddr) -> *const Box<PeerIntEntry> {
        match self {
            PeerTable::V4(_) => &(*PEER_TABLE_V4.lock().borrow()[ip_addr].lock().borrow()),
            PeerTable::V6(_) => &(*PEER_TABLE_V6.lock().borrow()[ip_addr].lock().borrow()),
        }
    }
    fn get_mut(&self, ip_addr: &IpAddr) -> *mut Box<PeerIntEntry> {
        match self {
            PeerTable::V4(_) => &mut (*PEER_TABLE_V4.lock().borrow()[ip_addr].lock().borrow_mut()),
            PeerTable::V6(_) => &mut (*PEER_TABLE_V6.lock().borrow()[ip_addr].lock().borrow_mut()),
        }
    }
    fn insert(&self, ip_addr: &IpAddr, entry: Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>) {
        match self {
            PeerTable::V4(_) => {
                PEER_TABLE_V4.lock().borrow_mut().insert(*ip_addr, entry);
            }
            PeerTable::V6(_) => {
                PEER_TABLE_V6.lock().borrow_mut().insert(*ip_addr, entry);
            }
        }
    }
    fn remove(&self, ip_addr: &IpAddr) {
        match self {
            PeerTable::V4(_) => {
                if self.contains_key(ip_addr) {
                    PEER_TABLE_V4.lock().borrow()[ip_addr]
                        .lock()
                        .borrow_mut()
                        .cleanup(&RouteTable::V4(&ROUTE_TABLE_V4));
                }
                PEER_TABLE_V4.lock().borrow_mut().remove(ip_addr);
            }
            PeerTable::V6(_) => {
                if self.contains_key(ip_addr) {
                    PEER_TABLE_V6.lock().borrow()[ip_addr]
                        .lock()
                        .borrow_mut()
                        .cleanup(&RouteTable::V6(&ROUTE_TABLE_V6));
                }
                PEER_TABLE_V6.lock().borrow_mut().remove(ip_addr);
            }
        }
    }
    fn clone(&self, ip_addr: &IpAddr) -> Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>> {
        match self {
            PeerTable::V4(_) => Arc::clone(&PEER_TABLE_V4.lock().borrow()[ip_addr]),
            PeerTable::V6(_) => Arc::clone(&PEER_TABLE_V6.lock().borrow()[ip_addr]),
        }
    }
    fn _iterate(&self, keys_vals: Iter<IpAddr, Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>>) {
        for (key, val) in keys_vals {
            println!("key: {} val: {}", key, val.lock().borrow().prefix);
            unsafe {
                let mut ip: IpAddrC = IpAddrC {
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
            PeerTable::V4(_) => self._iterate(PEER_TABLE_V4.lock().borrow().iter()),
            PeerTable::V6(_) => self._iterate(PEER_TABLE_V6.lock().borrow().iter()),
        }
    }
}

pub struct RouteIntEntry {
    prefix: IpAddr,
    mask: IpAddr,
    next_hop: IpAddr,
    out_ifindex: u32,
    peer_table: Arc<
        ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>>>>,
    >,
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
    pub fn add_peer(
        &mut self,
        peer_prefix: IpAddr,
        peer: Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>,
    ) -> i32 {
        if self.peer_table.lock().borrow().contains_key(&peer_prefix) {
            trace!("RouteIntEntry::add_peer {} already exists", peer_prefix);
            -1
        } else {
            trace!("RouteIntEntry::add_peer {}", peer_prefix);
            self.peer_table
                .lock()
                .borrow_mut()
                .insert(peer_prefix, Arc::clone(&peer));
            0
        }
    }
    pub fn delete_peer(&self, peer_prefix: IpAddr, route_table: &RouteTable) -> i32 {
        trace!("RouteIntEntry::delete_peer {}", peer_prefix);
        let rc: i32;
        if self.peer_table.lock().borrow().contains_key(&peer_prefix) {
            self.peer_table.lock().borrow_mut().remove(&peer_prefix);
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
        self.peer_table.lock().borrow().contains_key(&_peer_prefix)
    }
    pub fn get_number_of_peers(&self) -> usize {
        self.peer_table.lock().borrow().len()
    }
}

unsafe fn copy_ip_addr_to_user(addr_ptr: *mut u8, addr: &IpAddr) {
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

unsafe fn copy_ip_addr_v4_from_user(addr_ptr: *mut u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(
        *addr_ptr,
        *addr_ptr.wrapping_add(1),
        *addr_ptr.wrapping_add(2),
        *addr_ptr.wrapping_add(3),
    ))
}

unsafe fn copy_ip_addr_v6_from_user(addr_ptr: *mut u16) -> IpAddr {
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

fn _route_lookup(ip_addr: &IpAddr, route_table: &RouteTable, _entry: *mut RouteEntry) -> i32 {
    if !route_table.contains_key(&ip_addr) {
        trace!("route_lookup: cannot find prefix {}", ip_addr);
        return -1;
    }
    unsafe {
        let re: &Box<RouteIntEntry> = &*route_table.get(&ip_addr);
        trace!(
            "route_lookup prefix {}, found prefix {} mask {} next_hop {} out_ifindex {}",
            ip_addr,
            re.prefix,
            re.mask,
            re.next_hop,
            re.out_ifindex
        );
        let mut addr_ptr: *mut u8 = (*_entry).prefix.addr;
        copy_ip_addr_to_user(addr_ptr, &re.prefix);
        addr_ptr = (*_entry).mask.addr;
        copy_ip_addr_to_user(addr_ptr, &re.mask);
        addr_ptr = (*_entry).next_hop.addr;
        copy_ip_addr_to_user(addr_ptr, &re.next_hop);
        (*_entry).out_ifindex = re.out_ifindex;
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
    peer_route_table: Arc<
        ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>>,
    >,
}

impl Drop for PeerIntEntry {
    fn drop(&mut self) {
        trace!("PeerIntEntry::Drop {}", self.prefix);
        let route_table: &RouteTable;
        match self.prefix {
            IpAddr::V4(ipv4) => {
                self.cleanup(&RouteTable::V4(&ROUTE_TABLE_V4));
            }
            IpAddr::V6(ipv6) => {
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
        for val in self.peer_route_table.lock().borrow().values() {
            val.lock()
                .borrow_mut()
                .delete_peer(self.prefix, route_table);
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
        let route_entry: Option<Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>;
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

        if !self
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&route_prefix)
        {
            trace!(
                "PeerIntEntry::route_add_modify prefix {} not found in peer rt. adding",
                route_prefix
            );
            match route_entry {
                Some(re) => {
                    self.peer_route_table
                        .lock()
                        .borrow_mut()
                        .insert(re.lock().borrow().prefix, Arc::clone(&re));
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
        let mut pe: &mut Box<PeerIntEntry>;
        unsafe {
            pe = &mut *peer_table.get_mut(&ip_addr);
        }
        trace!("found existing. modifying");
        pe.out_ifindex = entry.out_ifindex;

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
    unsafe {
        let pe = &*peer_table.get(&ip_addr);
        let addr_ptr: *mut u8 = (*_entry).prefix.addr;
        copy_ip_addr_to_user(addr_ptr, &pe.prefix);
        (*_entry).out_ifindex = pe.out_ifindex;
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

fn _peer_delete(ip_addr: &IpAddr, peer_table: &PeerTable, route_table: &RouteTable) -> i32 {
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
            _peer_delete(
                &ip_addr,
                &PeerTable::V4(&PEER_TABLE_V4),
                &RouteTable::V4(&ROUTE_TABLE_V4),
            )
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            _peer_delete(
                &ip_addr,
                &PeerTable::V6(&PEER_TABLE_V6),
                &RouteTable::V6(&ROUTE_TABLE_V6),
            )
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
    let pe: &mut Box<PeerIntEntry>;
    unsafe {
        pe = &mut *peer_table.get_mut(&peer_ip_addr);
    }
    let rc = pe.route_add_modify(
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
    let pe: &Box<PeerIntEntry>;
    unsafe {
        pe = &mut *peer_table.get_mut(&peer_ip_addr);
    }
    if !pe
        .peer_route_table
        .lock()
        .borrow()
        .contains_key(&route_prefix)
    {
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
            &pe.peer_route_table.lock().borrow()[&route_prefix]
                .lock()
                .borrow()
                .prefix,
        );
        addr_ptr = (*_entry).mask.addr;
        copy_ip_addr_to_user(
            addr_ptr,
            &pe.peer_route_table.lock().borrow()[&route_prefix]
                .lock()
                .borrow()
                .mask,
        );
        addr_ptr = (*_entry).next_hop.addr;
        copy_ip_addr_to_user(
            addr_ptr,
            &pe.peer_route_table.lock().borrow()[&route_prefix]
                .lock()
                .borrow()
                .next_hop,
        );
        (*_entry).out_ifindex = pe.peer_route_table.lock().borrow()[&route_prefix]
            .lock()
            .borrow()
            .out_ifindex;
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

fn _peer_route_delete(
    peer_ip_addr: &IpAddr,
    route_prefix: &IpAddr,
    peer_table: &PeerTable,
    route_table: &RouteTable,
) -> i32 {
    trace!(
        "peer_route_delete peer {} prefix {}",
        peer_ip_addr,
        route_prefix
    );
    if !peer_table.contains_key(&peer_ip_addr) {
        trace!("peer not found");
        return -1;
    }
    let pe: &Box<PeerIntEntry>;
    unsafe {
        pe = &*peer_table.get(&peer_ip_addr);
    }
    if !pe
        .peer_route_table
        .lock()
        .borrow()
        .contains_key(&route_prefix)
    {
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
            _peer_route_delete(
                &peer_ip_addr,
                &route_prefix,
                &PeerTable::V4(&PEER_TABLE_V4),
                &RouteTable::V4(&ROUTE_TABLE_V4),
            )
        } else {
            peer_ip_addr = copy_ip_addr_v6_from_user(_peer_prefix.addr as *mut u16);
            route_prefix = copy_ip_addr_v6_from_user(_route_prefix.addr as *mut u16);
            _peer_route_delete(
                &peer_ip_addr,
                &route_prefix,
                &PeerTable::V6(&PEER_TABLE_V6),
                &RouteTable::V6(&ROUTE_TABLE_V6),
            )
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
    unsafe {
        match prefix_tree.get_longest_common_prefix(*ip_addr) {
            Some(fe) => {
                trace!(
                    "longest_match_lookup prefix {}, found next_hop {} out_ifindex {}",
                    ip_addr,
                    fe.lock().borrow().next_hop,
                    fe.lock().borrow().out_ifindex
                );
                let mut addr_ptr: *mut u8 = (*_entry).next_hop.addr;
                copy_ip_addr_to_user(addr_ptr, &fe.lock().borrow().next_hop);
                (*_entry).out_ifindex = fe.lock().borrow().out_ifindex;
                0
            }
            None => -1,
        }
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
    unsafe {
        let new_fe: Arc<ReentrantMutex<RefCell<Box<ForwardingEntryInt>>>>;
        new_fe = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(fe))));
        prefix_tree.insert(*ip_addr, new_fe)
    }
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

use log::*;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

#[no_mangle]
pub extern "C" fn init_logger() {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace));
}
