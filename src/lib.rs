#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

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
    static ref ROUTE_TABLE: Arc<
        ReentrantMutex<
            RefCell<HashMap<Ipv4Addr, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>,
        >,
    > = Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
}

lazy_static! {
    static ref PEER_TABLE: Arc<ReentrantMutex<RefCell<HashMap<Ipv4Addr, Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
}

struct RouteIntEntry {
    prefix: Ipv4Addr,
    mask: Ipv4Addr,
    next_hop: Ipv4Addr,
    out_ifindex: u32,
    peer_table: Arc<
        ReentrantMutex<RefCell<HashMap<Ipv4Addr, Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>>>>,
    >,
    creator: Ipv4Addr,
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
        _prefix: Ipv4Addr,
        _mask: Ipv4Addr,
        _next_hop: Ipv4Addr,
        _out_ifindex: u32,
        _creator: Ipv4Addr,
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
        peer_prefix: Ipv4Addr,
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
    pub fn delete_peer(&self, _peer_prefix: Ipv4Addr) -> i32 {
        if self.peer_table.lock().borrow().contains_key(&_peer_prefix) {
            self.peer_table.lock().borrow_mut().remove(&_peer_prefix);
            0
        } else {
            -1
        }
    }
    pub fn peer_exists(&self, _peer_prefix: Ipv4Addr) -> bool {
        self.peer_table.lock().borrow().contains_key(&_peer_prefix)
    }
    pub fn get_number_of_peers(&self) -> usize {
        self.peer_table.lock().borrow().len()
    }
}

unsafe fn copy_ip_addr_to_user(addr_ptr: *mut u8, addr: &Ipv4Addr) {
    *addr_ptr.wrapping_add(0) = addr.octets()[0];
    *addr_ptr.wrapping_add(1) = addr.octets()[1];
    *addr_ptr.wrapping_add(2) = addr.octets()[2];
    *addr_ptr.wrapping_add(3) = addr.octets()[3];
}

unsafe fn copy_ip_addr_from_user(addr_ptr: *mut u8) -> Ipv4Addr {
    Ipv4Addr::new(
        *addr_ptr,
        *addr_ptr.wrapping_add(1),
        *addr_ptr.wrapping_add(2),
        *addr_ptr.wrapping_add(3),
    )
}

#[no_mangle]
pub extern "C" fn route_lookup(_prefix: &IpAddrC, _entry: *mut RouteEntry) -> i32 {
    let ip_addr: Ipv4Addr;
    unsafe {
        ip_addr = copy_ip_addr_from_user(_prefix.addr);
    }
    if ROUTE_TABLE.lock().borrow().contains_key(&ip_addr) {
        unsafe {
            println!(
                "route_lookup prefix {}, found prefix {} mask {} next_hop {} out_ifindex {}",
                ip_addr,
                ROUTE_TABLE.lock().borrow()[&ip_addr].lock().borrow().prefix,
                ROUTE_TABLE.lock().borrow()[&ip_addr].lock().borrow().mask,
                ROUTE_TABLE.lock().borrow()[&ip_addr]
                    .lock()
                    .borrow()
                    .next_hop,
                ROUTE_TABLE.lock().borrow()[&ip_addr]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            let mut addr_ptr: *mut u8 = (*_entry).prefix.addr;
            copy_ip_addr_to_user(
                addr_ptr,
                &ROUTE_TABLE.lock().borrow()[&ip_addr].lock().borrow().prefix,
            );
            addr_ptr = (*_entry).mask.addr;
            copy_ip_addr_to_user(
                addr_ptr,
                &ROUTE_TABLE.lock().borrow()[&ip_addr].lock().borrow().mask,
            );
            addr_ptr = (*_entry).next_hop.addr;
            copy_ip_addr_to_user(
                addr_ptr,
                &ROUTE_TABLE.lock().borrow()[&ip_addr]
                    .lock()
                    .borrow()
                    .next_hop,
            );
            (*_entry).out_ifindex = ROUTE_TABLE.lock().borrow()[&ip_addr]
                .lock()
                .borrow()
                .out_ifindex;
        }
        0
    } else {
        -1
    }
}

struct PeerIntEntry {
    prefix: Ipv4Addr,
    out_ifindex: u32,
    peer_route_table: Arc<
        ReentrantMutex<
            RefCell<HashMap<Ipv4Addr, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>,
        >,
    >,
}

impl PeerIntEntry {
    pub fn new(_prefix: Ipv4Addr, _out_ifindex: u32) -> PeerIntEntry {
        PeerIntEntry {
            prefix: _prefix,
            out_ifindex: _out_ifindex,
            peer_route_table: Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new()))),
        }
    }
}

#[no_mangle]
pub extern "C" fn peer_add(_prefix: &IpAddrC, _entry: *mut PeerEntry) -> i32 {
    let mut entry: Box<PeerEntry>;
    let ip_addr: Ipv4Addr;
    unsafe {
        entry = Box::from_raw(_entry);
        ip_addr = copy_ip_addr_from_user(_prefix.addr);
    }
    println!(
        "peer_add: key: {} out_ifindex: {}",
        ip_addr, entry.out_ifindex
    );

    if PEER_TABLE.lock().borrow().contains_key(&ip_addr) {
        let _m_entry = Box::into_raw(entry);
        -1
    } else {
        let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
            PeerIntEntry::new(ip_addr, entry.out_ifindex),
        ))));
        let _m_entry = Box::into_raw(entry);
        PEER_TABLE.lock().borrow_mut().insert(ip_addr, new_entry);
        0
    }
}

#[no_mangle]
pub extern "C" fn peer_add_modify(_prefix: &IpAddrC, _entry: *mut PeerEntry) -> i32 {
    let mut entry: Box<PeerEntry>;
    let ip_addr: Ipv4Addr;
    unsafe {
        entry = Box::from_raw(_entry);
        ip_addr = copy_ip_addr_from_user(_prefix.addr);
    }
    println!(
        "peer_add_modify: key: {} out_ifindex: {}",
        ip_addr, entry.out_ifindex
    );
    if PEER_TABLE.lock().borrow().contains_key(&ip_addr) {
        println!(
            "peer_add_modify: existing out_ifindex {} for peer {}",
            PEER_TABLE.lock().borrow()[&ip_addr]
                .lock()
                .borrow()
                .out_ifindex,
            ip_addr
        );
        PEER_TABLE.lock().borrow()[&ip_addr]
            .lock()
            .borrow_mut()
            .out_ifindex = entry.out_ifindex;
        println!(
            "new out_ifindex {} for peer {}",
            PEER_TABLE.lock().borrow()[&ip_addr]
                .lock()
                .borrow()
                .out_ifindex,
            ip_addr
        );
        let _m_entry = Box::into_raw(entry);
        1
    } else {
        let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
            PeerIntEntry::new(ip_addr, entry.out_ifindex),
        ))));
        let _m_entry = Box::into_raw(entry);
        PEER_TABLE.lock().borrow_mut().insert(ip_addr, new_entry);
        0
    }
}

#[no_mangle]
pub extern "C" fn peer_lookup(_prefix: &IpAddrC, _entry: *mut PeerEntry) -> i32 {
    let ip_addr: Ipv4Addr;
    unsafe {
        ip_addr = copy_ip_addr_from_user(_prefix.addr);
    }
    if PEER_TABLE.lock().borrow().contains_key(&ip_addr) {
        unsafe {
            println!(
                "peer_lookup prefix {}, found prefix {} out_ifindex {}",
                ip_addr,
                PEER_TABLE.lock().borrow()[&ip_addr].lock().borrow().prefix,
                PEER_TABLE.lock().borrow()[&ip_addr]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            let addr_ptr: *mut u8 = (*_entry).prefix.addr;
            copy_ip_addr_to_user(
                addr_ptr,
                &PEER_TABLE.lock().borrow()[&ip_addr].lock().borrow().prefix,
            );
            (*_entry).out_ifindex = PEER_TABLE.lock().borrow()[&ip_addr]
                .lock()
                .borrow()
                .out_ifindex;
        }
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn peer_delete(_prefix: &IpAddrC) -> i32 {
    let ip_addr: Ipv4Addr;
    unsafe {
        ip_addr = copy_ip_addr_from_user(_prefix.addr);
    }
    if PEER_TABLE.lock().borrow().contains_key(&ip_addr) {
        {
            for val in PEER_TABLE.lock().borrow()[&ip_addr]
                .lock()
                .borrow()
                .peer_route_table
                .lock()
                .borrow()
                .values()
            {
                val.lock().borrow_mut().delete_peer(ip_addr);
                if val.lock().borrow().get_number_of_peers() == 0 {
                    ROUTE_TABLE
                        .lock()
                        .borrow_mut()
                        .remove(&val.lock().borrow().prefix);
                }
            }
        }
        println!("deleting existing entry");
        PEER_TABLE.lock().borrow_mut().remove(&ip_addr);
        println!("done");
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn peer_route_add(_peer_prefix: &IpAddrC, _entry: *mut RouteEntry) -> i32 {
    let peer_ip_addr: Ipv4Addr;
    let route_prefix: Ipv4Addr;
    let route_mask: Ipv4Addr;
    let next_hop_addr: Ipv4Addr;
    let mut entry: Box<RouteEntry>;
    unsafe {
        entry = Box::from_raw(_entry);
        peer_ip_addr = copy_ip_addr_from_user(_peer_prefix.addr);
        route_prefix = copy_ip_addr_from_user(entry.prefix.addr);
        route_mask = copy_ip_addr_from_user(entry.mask.addr);
        next_hop_addr = copy_ip_addr_from_user(entry.next_hop.addr);
    }
    if PEER_TABLE.lock().borrow().contains_key(&peer_ip_addr) {
        println!(
            "peer_route_add: found peer {} out_ifindex {}",
            PEER_TABLE.lock().borrow()[&peer_ip_addr]
                .lock()
                .borrow()
                .prefix,
            PEER_TABLE.lock().borrow()[&peer_ip_addr]
                .lock()
                .borrow()
                .out_ifindex
        );

        if PEER_TABLE.lock().borrow()[&peer_ip_addr]
            .lock()
            .borrow()
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&route_prefix)
        {
            let _m_entry = Box::into_raw(entry);
            -2
        } else {
            let route_entry: Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>;
            if ROUTE_TABLE.lock().borrow().contains_key(&route_prefix) {
                route_entry = Arc::clone(&ROUTE_TABLE.lock().borrow()[&route_prefix]);
                println!("cloned route entry {} for peer", route_prefix);
            } else {
                let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
                    RouteIntEntry::new(
                        route_prefix,
                        route_mask,
                        next_hop_addr,
                        entry.out_ifindex,
                        peer_ip_addr,
                    ),
                ))));
                route_entry = Arc::clone(&new_entry);
                ROUTE_TABLE
                    .lock()
                    .borrow_mut()
                    .insert(route_prefix, new_entry);
                println!("new route entry {} for peer", route_prefix);
            }
            let route_entry_prefix = route_entry.lock().borrow_mut().prefix;
            PEER_TABLE.lock().borrow()[&peer_ip_addr]
                .lock()
                .borrow()
                .peer_route_table
                .lock()
                .borrow_mut()
                .insert(route_entry_prefix, Arc::clone(&route_entry));

            route_entry.lock().borrow_mut().add_peer(
                PEER_TABLE.lock().borrow()[&peer_ip_addr]
                    .lock()
                    .borrow_mut()
                    .prefix,
                Arc::clone(&PEER_TABLE.lock().borrow()[&peer_ip_addr]),
            );
            let _m_entry = Box::into_raw(entry);
            {
                println!(
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
pub extern "C" fn peer_route_add_modify(_peer_prefix: &IpAddrC, _entry: *mut RouteEntry) -> i32 {
    let mut entry: Box<RouteEntry>;
    let peer_ip_addr: Ipv4Addr;
    let route_prefix: Ipv4Addr;
    let route_mask: Ipv4Addr;
    let next_hop_addr: Ipv4Addr;
    unsafe {
        entry = Box::from_raw(_entry);
        peer_ip_addr = copy_ip_addr_from_user(_peer_prefix.addr);
        route_prefix = copy_ip_addr_from_user(entry.prefix.addr);
        route_mask = copy_ip_addr_from_user(entry.mask.addr);
        next_hop_addr = copy_ip_addr_from_user(entry.next_hop.addr);
    }
    if PEER_TABLE.lock().borrow().contains_key(&peer_ip_addr) {
        println!(
            "peer_route_add_modify: found peer {} out_ifindex {}",
            PEER_TABLE.lock().borrow()[&peer_ip_addr]
                .lock()
                .borrow()
                .prefix,
            PEER_TABLE.lock().borrow()[&peer_ip_addr]
                .lock()
                .borrow()
                .out_ifindex
        );

        if PEER_TABLE.lock().borrow()[&peer_ip_addr]
            .lock()
            .borrow()
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&route_prefix)
        {
            println!(
                "old next_hop {}",
                PEER_TABLE.lock().borrow()[&peer_ip_addr]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&route_prefix]
                    .lock()
                    .borrow()
                    .next_hop
            );
            println!(
                "old out_ifindex {}",
                PEER_TABLE.lock().borrow()[&peer_ip_addr]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&route_prefix]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            PEER_TABLE.lock().borrow()[&peer_ip_addr]
                .lock()
                .borrow()
                .peer_route_table
                .lock()
                .borrow()[&route_prefix]
                .lock()
                .borrow_mut()
                .out_ifindex = entry.out_ifindex;
            PEER_TABLE.lock().borrow()[&peer_ip_addr]
                .lock()
                .borrow()
                .peer_route_table
                .lock()
                .borrow()[&route_prefix]
                .lock()
                .borrow_mut()
                .next_hop = next_hop_addr;
            println!(
                "new next_hop {}",
                PEER_TABLE.lock().borrow()[&peer_ip_addr]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&route_prefix]
                    .lock()
                    .borrow()
                    .next_hop
            );
            println!(
                "new out_ifindex {}",
                PEER_TABLE.lock().borrow()[&peer_ip_addr]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&route_prefix]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            let _m_entry = Box::into_raw(entry);
            1
        } else {
            let route_entry: Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>;

            if ROUTE_TABLE.lock().borrow().contains_key(&route_prefix) {
                route_entry = Arc::clone(&ROUTE_TABLE.lock().borrow()[&route_prefix]);
                println!("cloned route entry for peer");
            } else {
                let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
                    RouteIntEntry::new(
                        route_prefix,
                        route_mask,
                        next_hop_addr,
                        entry.out_ifindex,
                        peer_ip_addr,
                    ),
                ))));
                route_entry = Arc::clone(&new_entry);
                ROUTE_TABLE
                    .lock()
                    .borrow_mut()
                    .insert(route_prefix, new_entry);
                println!("new route entry for peer");
            }
            let _peer_prefix = PEER_TABLE.lock().borrow()[&peer_ip_addr]
                .lock()
                .borrow()
                .prefix;
            route_entry.lock().borrow_mut().add_peer(
                peer_ip_addr,
                Arc::clone(&PEER_TABLE.lock().borrow()[&peer_ip_addr]),
            );
            println!(
                "peer_exists {}",
                route_entry.lock().borrow().peer_exists(peer_ip_addr)
            );
            println!("rt creator {}", route_entry.lock().borrow().creator);
            let _m_entry = Box::into_raw(entry);
            println!(
                "peer_route_add_modify: key: {} prefix: {} out_ifindex: {}",
                route_entry.lock().borrow().prefix,
                route_entry.lock().borrow().prefix,
                route_entry.lock().borrow().out_ifindex
            );
            PEER_TABLE.lock().borrow()[&peer_ip_addr]
                .lock()
                .borrow()
                .peer_route_table
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
pub extern "C" fn peer_route_lookup(
    _peer_prefix: &IpAddrC,
    _route_prefix: &IpAddrC,
    _entry: *mut RouteEntry,
) -> i32 {
    let peer_ip_addr: Ipv4Addr;
    let route_prefix: Ipv4Addr;
    unsafe {
        peer_ip_addr = copy_ip_addr_from_user(_peer_prefix.addr);
        route_prefix = copy_ip_addr_from_user(_route_prefix.addr);
    }
    if PEER_TABLE.lock().borrow().contains_key(&peer_ip_addr) {
        if PEER_TABLE.lock().borrow()[&peer_ip_addr]
            .lock()
            .borrow()
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&route_prefix)
        {
            unsafe {
                println!(
                    "peer_route_lookup peer_prefix {}, found prefix {}",
                    peer_ip_addr, route_prefix
                );
                let mut addr_ptr: *mut u8 = (*_entry).prefix.addr;
                copy_ip_addr_to_user(
                    addr_ptr,
                    &PEER_TABLE.lock().borrow()[&peer_ip_addr]
                        .lock()
                        .borrow()
                        .peer_route_table
                        .lock()
                        .borrow()[&route_prefix]
                        .lock()
                        .borrow()
                        .prefix,
                );
                addr_ptr = (*_entry).mask.addr;
                copy_ip_addr_to_user(
                    addr_ptr,
                    &PEER_TABLE.lock().borrow()[&peer_ip_addr]
                        .lock()
                        .borrow()
                        .peer_route_table
                        .lock()
                        .borrow()[&route_prefix]
                        .lock()
                        .borrow()
                        .mask,
                );
                addr_ptr = (*_entry).next_hop.addr;
                copy_ip_addr_to_user(
                    addr_ptr,
                    &PEER_TABLE.lock().borrow()[&peer_ip_addr]
                        .lock()
                        .borrow()
                        .peer_route_table
                        .lock()
                        .borrow()[&route_prefix]
                        .lock()
                        .borrow()
                        .next_hop,
                );
                (*_entry).out_ifindex = PEER_TABLE.lock().borrow()[&peer_ip_addr]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&route_prefix]
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
pub extern "C" fn peer_route_delete(_peer_prefix: &IpAddrC, _route_prefix: &IpAddrC) -> i32 {
    let peer_ip_addr: Ipv4Addr;
    let route_prefix: Ipv4Addr;
    unsafe {
        peer_ip_addr = copy_ip_addr_from_user(_peer_prefix.addr);
        route_prefix = copy_ip_addr_from_user(_route_prefix.addr);
    }
    if PEER_TABLE.lock().borrow().contains_key(&peer_ip_addr) {
        if PEER_TABLE.lock().borrow()[&peer_ip_addr]
            .lock()
            .borrow()
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&route_prefix)
        {
            println!("found route {} for peer {}", route_prefix, peer_ip_addr);
            if ROUTE_TABLE.lock().borrow().contains_key(&route_prefix) {
                println!("found route entry in global table");
                ROUTE_TABLE.lock().borrow()[&route_prefix]
                    .lock()
                    .borrow_mut()
                    .delete_peer(peer_ip_addr);
                println!("peer is removed from route table entry peer list");
                if ROUTE_TABLE.lock().borrow()[&route_prefix]
                    .lock()
                    .borrow()
                    .get_number_of_peers()
                    == 0
                {
                    println!("removing route from global as it was the last peer");
                    ROUTE_TABLE.lock().borrow_mut().remove(&route_prefix);
                }
            }
            PEER_TABLE.lock().borrow_mut().remove(&route_prefix);
            0
        } else {
            -2
        }
    } else {
        -1
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
