#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[repr(C)]
pub struct RouteEntry {
    prefix: u32,
    mask: u32,
    next_hop: u32,
    out_ifindex: u32,
}

#[repr(C)]
pub struct PeerEntry {
    prefix: u32,
    out_ifindex: u32,
}

lazy_static! {
    static ref ROUTE_TABLE: Mutex<HashMap<u32, Arc<Box<RouteIntEntry>>>> =
        Mutex::new(HashMap::new());
}

lazy_static! {
    static ref PEER_TABLE: Mutex<HashMap<u32, Arc<Box<PeerIntEntry>>>> = Mutex::new(HashMap::new());
}

struct RouteIntEntry {
    prefix: u32,
    mask: u32,
    next_hop: u32,
    out_ifindex: u32,
    peer_table: Mutex<HashMap<u32, Arc<Box<PeerIntEntry>>>>,
}

impl RouteIntEntry {
    pub fn new(_prefix: u32, _mask: u32, _next_hop: u32, _out_ifindex: u32) -> RouteIntEntry {
        RouteIntEntry {
            prefix: _prefix,
            mask: _mask,
            next_hop: _next_hop,
            out_ifindex: _out_ifindex,
            peer_table: Mutex::new(HashMap::new()),
        }
    }
    pub fn AddPeer(&self, peer: Arc<Box<PeerIntEntry>>) -> i32 {
        if self.peer_table.lock().unwrap().contains_key(&peer.prefix) {
            -1
        } else {
            self.peer_table.lock().unwrap().insert(peer.prefix, peer);
            0
        }
    }
    pub fn DeletePeer(&self, _peer_prefix: u32) -> i32 {
        if self.peer_table.lock().unwrap().contains_key(&_peer_prefix) {
            self.peer_table.lock().unwrap().remove(&_peer_prefix);
            0
        } else {
            -1
        }
    }
    pub fn PeerLookup(&self, _peer_prefix: u32, peer: &mut Arc<Box<PeerIntEntry>>) -> i32 {
        if self.peer_table.lock().unwrap().contains_key(&_peer_prefix) {
            *peer = Arc::clone(&self.peer_table.lock().unwrap()[&_peer_prefix]);
            0
        } else {
            -1
        }
    }
    pub fn GetNumberOfPeers(&self) -> usize {
        self.peer_table.lock().unwrap().len()
    }
}

#[no_mangle]
pub extern "C" fn route_lookup(_prefix: u32, _entry: *mut RouteEntry) -> i32 {
    if ROUTE_TABLE.lock().unwrap().contains_key(&_prefix) {
        let re = &ROUTE_TABLE.lock().unwrap()[&_prefix];
        unsafe {
            println!(
                "route_lookup prefix {}, found prefix {} mask {} next_hop {} out_ifindex {}",
                _prefix, re.prefix, re.mask, re.next_hop, re.out_ifindex
            );
            (*_entry).prefix = re.prefix;
            (*_entry).mask = re.mask;
            (*_entry).next_hop = re.next_hop;
            (*_entry).out_ifindex = re.out_ifindex;
        }
        0
    } else {
        -1
    }
}

struct PeerIntEntry {
    prefix: u32,
    out_ifindex: u32,
    peer_route_table: Mutex<HashMap<u32, Arc<Box<RouteIntEntry>>>>,
}

impl PeerIntEntry {
    pub fn new(_prefix: u32, _out_ifindex: u32) -> PeerIntEntry {
        PeerIntEntry {
            prefix: _prefix,
            out_ifindex: _out_ifindex,
            peer_route_table: Mutex::new(HashMap::new()),
        }
    }
    pub fn get_prefix(&self) -> u32 {
        self.prefix
    }
    pub fn get_out_ifindex(&self) -> u32 {
        self.out_ifindex
    }
}

#[no_mangle]
pub extern "C" fn peer_add(_prefix: u32, _entry: *mut PeerEntry) -> i32 {
    let mut entry: Box<PeerEntry>;
    unsafe {
        entry = Box::from_raw(_entry);
    }
    println!(
        "peer_add: key: {} prefix: {} out_ifindex: {}",
        _prefix, entry.prefix, entry.out_ifindex
    );
    if PEER_TABLE.lock().unwrap().contains_key(&_prefix) {
        let _m_entry = Box::into_raw(entry);
        -1
    } else {
        let new_entry = Arc::new(Box::new(PeerIntEntry::new(entry.prefix, entry.out_ifindex)));
        let _m_entry = Box::into_raw(entry);
        PEER_TABLE.lock().unwrap().insert(_prefix, new_entry);
        0
    }
}

#[no_mangle]
pub extern "C" fn peer_lookup(_prefix: u32, _entry: *mut PeerEntry) -> i32 {
    if PEER_TABLE.lock().unwrap().contains_key(&_prefix) {
        let re = &PEER_TABLE.lock().unwrap()[&_prefix];
        unsafe {
            println!(
                "peer_lookup prefix {}, found prefix {} out_ifindex {}",
                _prefix, re.prefix, re.out_ifindex
            );
            (*_entry).prefix = re.prefix;
            (*_entry).out_ifindex = re.out_ifindex;
        }
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn peer_delete(_prefix: u32) -> i32 {
    let mut peer_gt = PEER_TABLE.lock().unwrap();
    if peer_gt.contains_key(&_prefix) {
        {
            let mut peer_rt = peer_gt[&_prefix].peer_route_table.lock().unwrap();
            let mut global_rt = ROUTE_TABLE.lock().unwrap();
            for val in peer_rt.values() {
                val.DeletePeer(_prefix);
                if val.GetNumberOfPeers() == 0 {
                    global_rt.remove(&val.prefix);
                }
            }
        }
        println!("deleting existing entry");
        peer_gt.remove(&_prefix);
        println!("done");
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn peer_route_add(_peer_prefix: u32, _entry: *mut RouteEntry) -> i32 {
    if PEER_TABLE.lock().unwrap().contains_key(&_peer_prefix) {
        let re = &PEER_TABLE.lock().unwrap()[&_peer_prefix];
        println!(
            "peer_route_add: found prefix {} out_ifindex {}",
            re.prefix, re.out_ifindex
        );
        let mut entry: Box<RouteEntry>;
        unsafe {
            entry = Box::from_raw(_entry);
        }
        if re
            .peer_route_table
            .lock()
            .unwrap()
            .contains_key(&entry.prefix)
        {
            let _m_entry = Box::into_raw(entry);
            -2
        } else {
            let route_entry: Arc<Box<RouteIntEntry>>;
            if ROUTE_TABLE.lock().unwrap().contains_key(&entry.prefix) {
                let existing_entry: &Arc<Box<RouteIntEntry>> =
                    &ROUTE_TABLE.lock().unwrap()[&entry.prefix];
                route_entry = Arc::clone(&existing_entry);
                println!("cloned route entry for peer");
            } else {
                let new_entry: Arc<Box<RouteIntEntry>> = Arc::new(Box::new(RouteIntEntry::new(
                    entry.prefix,
                    entry.mask,
                    entry.next_hop,
                    entry.out_ifindex,
                )));
                route_entry = Arc::clone(&new_entry);
                ROUTE_TABLE.lock().unwrap().insert(entry.prefix, new_entry);
                println!("new route entry for peer");
            }
            route_entry.AddPeer(Arc::clone(re));
            let _m_entry = Box::into_raw(entry);
            println!(
                "peer_route_add: key: {} prefix: {} out_ifindex: {}",
                route_entry.prefix, route_entry.prefix, route_entry.out_ifindex
            );
            re.peer_route_table
                .lock()
                .unwrap()
                .insert(route_entry.prefix, route_entry);
            0
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn peer_route_lookup(
    _peer_prefix: u32,
    _route_prefix: u32,
    _entry: *mut RouteEntry,
) -> i32 {
    if PEER_TABLE.lock().unwrap().contains_key(&_peer_prefix) {
        let pe = &PEER_TABLE.lock().unwrap()[&_peer_prefix];
        if pe
            .peer_route_table
            .lock()
            .unwrap()
            .contains_key(&_route_prefix)
        {
            let re = &pe.peer_route_table.lock().unwrap()[&_route_prefix];
            unsafe {
                println!(
                    "peer_route_lookup peer_prefix {}, found prefix {}",
                    _peer_prefix, _route_prefix
                );
                (*_entry).prefix = re.prefix;
                (*_entry).mask = re.mask;
                (*_entry).next_hop = re.next_hop;
                (*_entry).out_ifindex = re.out_ifindex;
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
pub extern "C" fn peer_route_delete(_peer_prefix: u32, _route_prefix: u32) -> i32 {
    let mut peer_t = PEER_TABLE.lock().unwrap();
    if peer_t.contains_key(&_peer_prefix) {
        let re = &peer_t[&_peer_prefix];
        let mut peer_rt = re.peer_route_table.lock().unwrap();
        if peer_rt.contains_key(&_route_prefix) {
            println!("found route {} for peer {}", _route_prefix, _peer_prefix);
            let route_entry: Arc<Box<RouteIntEntry>>;
            let mut global_rt = ROUTE_TABLE.lock().unwrap();
            if global_rt.contains_key(&_route_prefix) {
                println!("found route entry in global table");
                let existing_entry: &Arc<Box<RouteIntEntry>> = &global_rt[&_route_prefix];
                existing_entry.DeletePeer(_peer_prefix);
                println!("peer is removed from route table entry peer list");
                if existing_entry.GetNumberOfPeers() == 0 {
                    println!("removing route from global as it was the last peer");
                    global_rt.remove(&_route_prefix);
                }
            }
            peer_rt.remove(&_route_prefix);
            0
        } else {
            -2
        }
    } else {
        -1
    }
}

use std::alloc::{GlobalAlloc, System, Layout};

extern "C" {
    fn pool_alloc(size: usize) -> *mut u8;
    fn pool_free(ptr: *mut u8);
}

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        //System.alloc(layout)
	pool_alloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        //System.dealloc(ptr, layout)
	pool_free(ptr);
    }
}

#[global_allocator]
static GLOBAL: MyAllocator = MyAllocator;
