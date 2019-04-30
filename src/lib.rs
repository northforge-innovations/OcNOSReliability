#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
extern crate log;

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

lazy_static! {
    pub static ref ROUTE_TABLE_V4: Arc<ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref ROUTE_TABLE_V6: Arc<ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
}

/*type RouteTableV4 = Arc<ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>>>;
type RouteTableV6 = Arc<ReentrantMutex<RefCell<HashMap<IpAddr, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>>>;*/

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
                PEER_TABLE_V4.lock().borrow_mut().remove(ip_addr);
            }
            PeerTable::V6(_) => {
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
            -1
        } else {
            self.peer_table
                .lock()
                .borrow_mut()
                .insert(peer_prefix, Arc::clone(&peer));
            0
        }
    }
    pub fn delete_peer(&self, _peer_prefix: IpAddr) -> i32 {
        if self.peer_table.lock().borrow().contains_key(&_peer_prefix) {
            self.peer_table.lock().borrow_mut().remove(&_peer_prefix);
            0
        } else {
            -1
        }
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
    if route_table.contains_key(&ip_addr) {
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
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn route_lookup(_prefix: &IpAddrC, _entry: *mut RouteEntry) -> i32 {
    let ip_addr;
    //let route_table: &RouteTable;
    unsafe {
        if _prefix.family == 1 {
            ip_addr = copy_ip_addr_v4_from_user(_prefix.addr);
            //          route_table = ;
            _route_lookup(&ip_addr, &RouteTable::V4(&ROUTE_TABLE_V4), _entry)
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            //            route_table = &mut RouteTable::V6(ROUTE_TABLE_V6);
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

impl PeerIntEntry {
    pub fn new(_prefix: IpAddr, _out_ifindex: u32) -> PeerIntEntry {
        PeerIntEntry {
            prefix: _prefix,
            out_ifindex: _out_ifindex,
            peer_route_table: Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new()))),
        }
    }
}

fn _peer_add(ip_addr: &IpAddr, peer_table: &PeerTable, _entry: *mut PeerEntry) -> i32 {
    let mut entry: Box<PeerEntry>;
    unsafe {
        entry = Box::from_raw(_entry);
    }
    trace!(
        "peer_add: key: {} out_ifindex: {}",
        ip_addr,
        entry.out_ifindex
    );

    if peer_table.contains_key(&ip_addr) {
        let _m_entry = Box::into_raw(entry);
        -1
    } else {
        let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
            PeerIntEntry::new(*ip_addr, entry.out_ifindex),
        ))));
        let _m_entry = Box::into_raw(entry);
        peer_table.insert(&ip_addr, new_entry);
        0
    }
}

#[no_mangle]
pub extern "C" fn peer_add(_prefix: &IpAddrC, _entry: *mut PeerEntry) -> i32 {
    let ip_addr: IpAddr;

    unsafe {
        if _prefix.family == 1 {
            ip_addr = copy_ip_addr_v4_from_user(_prefix.addr);
            _peer_add(&ip_addr, &PeerTable::V4(&PEER_TABLE_V4), _entry)
        } else {
            ip_addr = copy_ip_addr_v6_from_user(_prefix.addr as *mut u16);
            _peer_add(&ip_addr, &PeerTable::V6(&PEER_TABLE_V6), _entry)
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
        trace!(
            "peer_add_modify: existing out_ifindex {} for peer {}",
            pe.out_ifindex,
            ip_addr
        );

        pe.out_ifindex = entry.out_ifindex;
        trace!("new out_ifindex {} for peer {}", pe.out_ifindex, ip_addr);

        let _m_entry = Box::into_raw(entry);
        1
    } else {
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
    if peer_table.contains_key(&ip_addr) {
        unsafe {
            let pe = &*peer_table.get(&ip_addr);
            trace!(
                "peer_lookup prefix {}, found prefix {} out_ifindex {}",
                ip_addr,
                pe.prefix,
                pe.out_ifindex
            );
            let addr_ptr: *mut u8 = (*_entry).prefix.addr;
            copy_ip_addr_to_user(addr_ptr, &pe.prefix);
            (*_entry).out_ifindex = pe.out_ifindex;
        }
        0
    } else {
        -1
    }
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
    if peer_table.contains_key(&ip_addr) {
        {
            let pe: &Box<PeerIntEntry>;
            unsafe {
                pe = &mut *peer_table.get_mut(&ip_addr);
            }
            for val in pe.peer_route_table.lock().borrow().values() {
                val.lock().borrow_mut().delete_peer(*ip_addr);
                if val.lock().borrow().get_number_of_peers() == 0 {
                    route_table.remove(&val.lock().borrow().prefix);
                }
            }
        }
        trace!("deleting existing entry");
        peer_table.remove(&ip_addr);
        trace!("done");
        0
    } else {
        -1
    }
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

fn _peer_route_add(
    peer_ip_addr: &IpAddr,
    route_prefix: &IpAddr,
    route_mask: &IpAddr,
    next_hop_addr: &IpAddr,
    peer_table: &PeerTable,
    route_table: &RouteTable,
    entry: Box<RouteEntry>,
) -> i32 {
    if peer_table.contains_key(&peer_ip_addr) {
        let pe: &Box<PeerIntEntry>;
        unsafe {
            pe = &mut *peer_table.get_mut(&peer_ip_addr);
        }
        trace!(
            "peer_route_add: found peer {} out_ifindex {}",
            pe.prefix,
            pe.out_ifindex
        );

        if pe
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&route_prefix)
        {
            let _m_entry = Box::into_raw(entry);
            -2
        } else {
            let route_entry: Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>;
            if route_table.contains_key(&route_prefix) {
                route_entry = route_table.clone(&route_prefix);
                trace!("cloned route entry {} for peer", route_prefix);
            } else {
                let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
                    RouteIntEntry::new(
                        *route_prefix,
                        *route_mask,
                        *next_hop_addr,
                        entry.out_ifindex,
                        *peer_ip_addr,
                    ),
                ))));
                route_entry = Arc::clone(&new_entry);
                route_table.insert(&route_prefix, new_entry);
                trace!("new route entry {} for peer", route_prefix);
            }
            let route_entry_prefix = route_entry.lock().borrow_mut().prefix;
            pe.peer_route_table
                .lock()
                .borrow_mut()
                .insert(route_entry_prefix, Arc::clone(&route_entry));

            route_entry
                .lock()
                .borrow_mut()
                .add_peer(pe.prefix, peer_table.clone(&peer_ip_addr));
            let _m_entry = Box::into_raw(entry);
            {
                trace!(
                    "peer_route_add: key: {} prefix: {} out_ifindex: {}",
                    route_entry.lock().borrow().prefix,
                    route_entry.lock().borrow().prefix,
                    route_entry.lock().borrow().out_ifindex
                );
            }
            0
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn peer_route_add(_peer_prefix: &IpAddrC, _entry: *mut RouteEntry) -> i32 {
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
            _peer_route_add(
                &peer_ip_addr,
                &route_prefix,
                &route_mask,
                &next_hop_addr,
                &PeerTable::V4(&PEER_TABLE_V4),
                &RouteTable::V4(&ROUTE_TABLE_V4),
                entry,
            )
        } else {
            peer_ip_addr = copy_ip_addr_v6_from_user(_peer_prefix.addr as *mut u16);
            route_prefix = copy_ip_addr_v6_from_user(entry.prefix.addr as *mut u16);
            route_mask = copy_ip_addr_v6_from_user(entry.mask.addr as *mut u16);
            next_hop_addr = copy_ip_addr_v6_from_user(entry.next_hop.addr as *mut u16);
            _peer_route_add(
                &peer_ip_addr,
                &route_prefix,
                &route_mask,
                &next_hop_addr,
                &PeerTable::V6(&PEER_TABLE_V6),
                &RouteTable::V6(&ROUTE_TABLE_V6),
                entry,
            )
        }
    }
}

fn _peer_route_add_modify(
    peer_ip_addr: &IpAddr,
    route_prefix: &IpAddr,
    route_mask: &IpAddr,
    next_hop_addr: &IpAddr,
    peer_table: &PeerTable,
    route_table: &RouteTable,
    entry: Box<RouteEntry>,
) -> i32 {
    if peer_table.contains_key(&peer_ip_addr) {
        let pe: &Box<PeerIntEntry>;
        unsafe {
            pe = &mut *peer_table.get_mut(&peer_ip_addr);
        }
        trace!(
            "peer_route_add_modify: found peer {} out_ifindex {}",
            pe.prefix,
            pe.out_ifindex
        );

        if pe
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&route_prefix)
        {
            trace!(
                "old next_hop {}",
                pe.peer_route_table.lock().borrow()[&route_prefix]
                    .lock()
                    .borrow()
                    .next_hop
            );
            trace!(
                "old out_ifindex {}",
                pe.peer_route_table.lock().borrow()[&route_prefix]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            pe.peer_route_table.lock().borrow()[&route_prefix]
                .lock()
                .borrow_mut()
                .out_ifindex = entry.out_ifindex;
            pe.peer_route_table.lock().borrow()[&route_prefix]
                .lock()
                .borrow_mut()
                .next_hop = *next_hop_addr;
            trace!(
                "new next_hop {}",
                pe.peer_route_table.lock().borrow()[&route_prefix]
                    .lock()
                    .borrow()
                    .next_hop
            );
            trace!(
                "new out_ifindex {}",
                pe.peer_route_table.lock().borrow()[&route_prefix]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            let _m_entry = Box::into_raw(entry);
            1
        } else {
            let route_entry: Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>;

            if route_table.contains_key(&route_prefix) {
                route_entry = Arc::clone(&route_table.clone(&route_prefix));
                trace!("cloned route entry for peer");
            } else {
                let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
                    RouteIntEntry::new(
                        *route_prefix,
                        *route_mask,
                        *next_hop_addr,
                        entry.out_ifindex,
                        *peer_ip_addr,
                    ),
                ))));
                route_entry = Arc::clone(&new_entry);
                route_table.insert(&route_prefix, new_entry);
                trace!("new route entry for peer");
            }
            let _peer_prefix = pe.prefix;
            route_entry
                .lock()
                .borrow_mut()
                .add_peer(*peer_ip_addr, peer_table.clone(&peer_ip_addr));
            trace!(
                "peer_exists {}",
                route_entry.lock().borrow().peer_exists(*peer_ip_addr)
            );
            trace!("rt creator {}", route_entry.lock().borrow().creator);
            let _m_entry = Box::into_raw(entry);
            trace!(
                "peer_route_add_modify: key: {} prefix: {} out_ifindex: {}",
                route_entry.lock().borrow().prefix,
                route_entry.lock().borrow().prefix,
                route_entry.lock().borrow().out_ifindex
            );
            pe.peer_route_table
                .lock()
                .borrow_mut()
                .insert(route_entry.lock().borrow().prefix, Arc::clone(&route_entry));
            0
        }
    } else {
        -1
    }
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
                &PeerTable::V4(&PEER_TABLE_V4),
                &RouteTable::V4(&ROUTE_TABLE_V4),
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
                &PeerTable::V6(&PEER_TABLE_V6),
                &RouteTable::V6(&ROUTE_TABLE_V6),
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
    if peer_table.contains_key(&peer_ip_addr) {
        let pe: &Box<PeerIntEntry>;
        unsafe {
            pe = &mut *peer_table.get_mut(&peer_ip_addr);
        }
        if pe
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&route_prefix)
        {
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
        } else {
            -2
        }
    } else {
        -1
    }
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
    if peer_table.contains_key(&peer_ip_addr) {
        let pe: &Box<PeerIntEntry>;
        unsafe {
            pe = &*peer_table.get(&peer_ip_addr);
        }
        if pe
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&route_prefix)
        {
            trace!("found route {} for peer {}", route_prefix, peer_ip_addr);
            if route_table.contains_key(&route_prefix) {
                let re: &Box<RouteIntEntry>;
                unsafe {
                    re = &*route_table.get(&route_prefix);
                }
                trace!("found route entry in global table");
                re.delete_peer(*peer_ip_addr);
                trace!("peer is removed from route table entry peer list");
                if re.get_number_of_peers() == 0 {
                    trace!("removing route from global as it was the last peer");
                    route_table.remove(&route_prefix);
                }
            }
            peer_table.remove(&route_prefix);
            0
        } else {
            -2
        }
    } else {
        -1
    }
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
