#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

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
    static ref ROUTE_TABLE: Arc<ReentrantMutex<RefCell<HashMap<u32, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
}

lazy_static! {
    static ref PEER_TABLE: Arc<ReentrantMutex<RefCell<HashMap<u32, Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
}

struct RouteIntEntry {
    prefix: u32,
    mask: u32,
    next_hop: u32,
    out_ifindex: u32,
    peer_table:
        Arc<ReentrantMutex<RefCell<HashMap<u32, Arc<ReentrantMutex<RefCell<Box<PeerIntEntry>>>>>>>>,
    creator: u32,
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
        _prefix: u32,
        _mask: u32,
        _next_hop: u32,
        _out_ifindex: u32,
        _creator: u32,
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
        peer_prefix: u32,
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
    pub fn delete_peer(&self, _peer_prefix: u32) -> i32 {
        if self.peer_table.lock().borrow().contains_key(&_peer_prefix) {
            self.peer_table.lock().borrow_mut().remove(&_peer_prefix);
            0
        } else {
            -1
        }
    }
    pub fn peer_exists(&self, _peer_prefix: u32) -> bool {
        self.peer_table.lock().borrow().contains_key(&_peer_prefix)
    }
    pub fn get_number_of_peers(&self) -> usize {
        self.peer_table.lock().borrow().len()
    }
}

#[no_mangle]
pub extern "C" fn route_lookup(_prefix: u32, _entry: *mut RouteEntry) -> i32 {
    if ROUTE_TABLE.lock().borrow().contains_key(&_prefix) {
        unsafe {
            println!(
                "route_lookup prefix {}, found prefix {} mask {} next_hop {} out_ifindex {}",
                _prefix,
                ROUTE_TABLE.lock().borrow()[&_prefix].lock().borrow().prefix,
                ROUTE_TABLE.lock().borrow()[&_prefix].lock().borrow().mask,
                ROUTE_TABLE.lock().borrow()[&_prefix]
                    .lock()
                    .borrow()
                    .next_hop,
                ROUTE_TABLE.lock().borrow()[&_prefix]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            (*_entry).prefix = ROUTE_TABLE.lock().borrow()[&_prefix].lock().borrow().prefix;
            (*_entry).mask = ROUTE_TABLE.lock().borrow()[&_prefix].lock().borrow().mask;
            (*_entry).next_hop = ROUTE_TABLE.lock().borrow()[&_prefix]
                .lock()
                .borrow()
                .next_hop;
            (*_entry).out_ifindex = ROUTE_TABLE.lock().borrow()[&_prefix]
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
    prefix: u32,
    out_ifindex: u32,
    peer_route_table: Arc<
        ReentrantMutex<RefCell<HashMap<u32, Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>>>>,
    >,
}

impl PeerIntEntry {
    pub fn new(_prefix: u32, _out_ifindex: u32) -> PeerIntEntry {
        PeerIntEntry {
            prefix: _prefix,
            out_ifindex: _out_ifindex,
            peer_route_table: Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new()))),
        }
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
    //let mut peer_tbl = PEER_TABLE.lock();
    if PEER_TABLE.lock().borrow().contains_key(&_prefix) {
        let _m_entry = Box::into_raw(entry);
        -1
    } else {
        let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
            PeerIntEntry::new(entry.prefix, entry.out_ifindex),
        ))));
        let _m_entry = Box::into_raw(entry);
        PEER_TABLE.lock().borrow_mut().insert(_prefix, new_entry);
        0
    }
}

#[no_mangle]
pub extern "C" fn peer_add_modify(_prefix: u32, _entry: *mut PeerEntry) -> i32 {
    let mut entry: Box<PeerEntry>;
    unsafe {
        entry = Box::from_raw(_entry);
    }
    println!(
        "peer_add_modify: key: {} prefix: {} out_ifindex: {}",
        _prefix, entry.prefix, entry.out_ifindex
    );
    if PEER_TABLE.lock().borrow().contains_key(&_prefix) {
        println!(
            "peer_add_modify: existing out_ifindex {} for peer {}",
            PEER_TABLE.lock().borrow()[&_prefix]
                .lock()
                .borrow()
                .out_ifindex,
            _prefix
        );
        /*        Arc::get_mut(&mut peer_tbl.get_mut(&_prefix).unwrap())
        .unwrap()
        .out_ifindex = entry.out_ifindex;*/
        PEER_TABLE.lock().borrow()[&_prefix]
            .lock()
            .borrow_mut()
            .out_ifindex = entry.out_ifindex;
        println!(
            "new out_ifindex {} for peer {}",
            PEER_TABLE.lock().borrow()[&_prefix]
                .lock()
                .borrow()
                .out_ifindex,
            _prefix
        );
        let _m_entry = Box::into_raw(entry);
        1
    } else {
        let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
            PeerIntEntry::new(entry.prefix, entry.out_ifindex),
        ))));
        let _m_entry = Box::into_raw(entry);
        PEER_TABLE.lock().borrow_mut().insert(_prefix, new_entry);
        0
    }
}

#[no_mangle]
pub extern "C" fn peer_lookup(_prefix: u32, _entry: *mut PeerEntry) -> i32 {
    if PEER_TABLE.lock().borrow().contains_key(&_prefix) {
        unsafe {
            println!(
                "peer_lookup prefix {}, found prefix {} out_ifindex {}",
                _prefix,
                PEER_TABLE.lock().borrow()[&_prefix].lock().borrow().prefix,
                PEER_TABLE.lock().borrow()[&_prefix]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            (*_entry).prefix = PEER_TABLE.lock().borrow()[&_prefix].lock().borrow().prefix;
            (*_entry).out_ifindex = PEER_TABLE.lock().borrow()[&_prefix]
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
pub extern "C" fn peer_delete(_prefix: u32) -> i32 {
    if PEER_TABLE.lock().borrow().contains_key(&_prefix) {
        {
            for val in PEER_TABLE.lock().borrow()[&_prefix]
                .lock()
                .borrow()
                .peer_route_table
                .lock()
                .borrow()
                .values()
            {
                val.lock().borrow_mut().delete_peer(_prefix);
                if val.lock().borrow().get_number_of_peers() == 0 {
                    ROUTE_TABLE
                        .lock()
                        .borrow_mut()
                        .remove(&val.lock().borrow().prefix);
                }
            }
        }
        println!("deleting existing entry");
        PEER_TABLE.lock().borrow_mut().remove(&_prefix);
        println!("done");
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn peer_route_add(_peer_prefix: u32, _entry: *mut RouteEntry) -> i32 {
    if PEER_TABLE.lock().borrow().contains_key(&_peer_prefix) {
        println!(
            "peer_route_add: found peer {} out_ifindex {}",
            PEER_TABLE.lock().borrow()[&_peer_prefix]
                .lock()
                .borrow()
                .prefix,
            PEER_TABLE.lock().borrow()[&_peer_prefix]
                .lock()
                .borrow()
                .out_ifindex
        );
        let mut entry: Box<RouteEntry>;
        unsafe {
            entry = Box::from_raw(_entry);
        }

        if PEER_TABLE.lock().borrow()[&_peer_prefix]
            .lock()
            .borrow()
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&entry.prefix)
        {
            let _m_entry = Box::into_raw(entry);
            -2
        } else {
            let route_entry: Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>;
            if ROUTE_TABLE.lock().borrow().contains_key(&entry.prefix) {
                route_entry = Arc::clone(&ROUTE_TABLE.lock().borrow()[&entry.prefix]);
                println!("cloned route entry {} for peer", entry.prefix);
            } else {
                let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
                    RouteIntEntry::new(
                        entry.prefix,
                        entry.mask,
                        entry.next_hop,
                        entry.out_ifindex,
                        _peer_prefix,
                    ),
                ))));
                route_entry = Arc::clone(&new_entry);
                ROUTE_TABLE
                    .lock()
                    .borrow_mut()
                    .insert(entry.prefix, new_entry);
                println!("new route entry {} for peer", entry.prefix);
            }
            let route_entry_prefix = route_entry.lock().borrow_mut().prefix;
            PEER_TABLE.lock().borrow()[&_peer_prefix]
                .lock()
                .borrow()
                .peer_route_table
                .lock()
                .borrow_mut()
                .insert(route_entry_prefix, Arc::clone(&route_entry));

            route_entry.lock().borrow_mut().add_peer(
                PEER_TABLE.lock().borrow()[&_peer_prefix]
                    .lock()
                    .borrow_mut()
                    .prefix,
                Arc::clone(&PEER_TABLE.lock().borrow()[&_peer_prefix]),
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
pub extern "C" fn peer_route_add_modify(_peer_prefix: u32, _entry: *mut RouteEntry) -> i32 {
    if PEER_TABLE.lock().borrow().contains_key(&_peer_prefix) {
        println!(
            "peer_route_add_modify: found peer {} out_ifindex {}",
            PEER_TABLE.lock().borrow()[&_peer_prefix]
                .lock()
                .borrow()
                .prefix,
            PEER_TABLE.lock().borrow()[&_peer_prefix]
                .lock()
                .borrow()
                .out_ifindex
        );
        let mut entry: Box<RouteEntry>;
        unsafe {
            entry = Box::from_raw(_entry);
        }
        if PEER_TABLE.lock().borrow()[&_peer_prefix]
            .lock()
            .borrow()
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&entry.prefix)
        {
            println!(
                "old next_hop {}",
                PEER_TABLE.lock().borrow()[&_peer_prefix]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&entry.prefix]
                    .lock()
                    .borrow()
                    .next_hop
            );
            println!(
                "old out_ifindex {}",
                PEER_TABLE.lock().borrow()[&_peer_prefix]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&entry.prefix]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            PEER_TABLE.lock().borrow()[&_peer_prefix]
                .lock()
                .borrow()
                .peer_route_table
                .lock()
                .borrow()[&entry.prefix]
                .lock()
                .borrow_mut()
                .out_ifindex = entry.out_ifindex;
            PEER_TABLE.lock().borrow()[&_peer_prefix]
                .lock()
                .borrow()
                .peer_route_table
                .lock()
                .borrow()[&entry.prefix]
                .lock()
                .borrow_mut()
                .next_hop = entry.next_hop;
            println!(
                "new next_hop {}",
                PEER_TABLE.lock().borrow()[&_peer_prefix]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&entry.prefix]
                    .lock()
                    .borrow()
                    .next_hop
            );
            println!(
                "new out_ifindex {}",
                PEER_TABLE.lock().borrow()[&_peer_prefix]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&entry.prefix]
                    .lock()
                    .borrow()
                    .out_ifindex
            );
            let _m_entry = Box::into_raw(entry);
            1
        } else {
            let route_entry: Arc<ReentrantMutex<RefCell<Box<RouteIntEntry>>>>;

            if ROUTE_TABLE.lock().borrow().contains_key(&entry.prefix) {
                route_entry = Arc::clone(&ROUTE_TABLE.lock().borrow()[&entry.prefix]);
                println!("cloned route entry for peer");
            } else {
                let new_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
                    RouteIntEntry::new(
                        entry.prefix,
                        entry.mask,
                        entry.next_hop,
                        entry.out_ifindex,
                        _peer_prefix,
                    ),
                ))));
                route_entry = Arc::clone(&new_entry);
                ROUTE_TABLE
                    .lock()
                    .borrow_mut()
                    .insert(entry.prefix, new_entry);
                println!("new route entry for peer");
            }
            let _peer_prefix = PEER_TABLE.lock().borrow()[&_peer_prefix]
                .lock()
                .borrow()
                .prefix;
            route_entry.lock().borrow_mut().add_peer(
                _peer_prefix,
                Arc::clone(&PEER_TABLE.lock().borrow()[&_peer_prefix]),
            );
            println!(
                "peer_exists {}",
                route_entry.lock().borrow().peer_exists(_peer_prefix)
            );
            println!("rt creator {}", route_entry.lock().borrow().creator);
            let _m_entry = Box::into_raw(entry);
            println!(
                "peer_route_add_modify: key: {} prefix: {} out_ifindex: {}",
                route_entry.lock().borrow().prefix,
                route_entry.lock().borrow().prefix,
                route_entry.lock().borrow().out_ifindex
            );
            PEER_TABLE.lock().borrow()[&_peer_prefix]
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
    _peer_prefix: u32,
    _route_prefix: u32,
    _entry: *mut RouteEntry,
) -> i32 {
    if PEER_TABLE.lock().borrow().contains_key(&_peer_prefix) {
        if PEER_TABLE.lock().borrow()[&_peer_prefix]
            .lock()
            .borrow()
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&_route_prefix)
        {
            unsafe {
                println!(
                    "peer_route_lookup peer_prefix {}, found prefix {}",
                    _peer_prefix, _route_prefix
                );
                (*_entry).prefix = PEER_TABLE.lock().borrow()[&_peer_prefix]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&_route_prefix]
                    .lock()
                    .borrow()
                    .prefix;
                (*_entry).mask = PEER_TABLE.lock().borrow()[&_peer_prefix]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&_route_prefix]
                    .lock()
                    .borrow()
                    .mask;
                (*_entry).next_hop = PEER_TABLE.lock().borrow()[&_peer_prefix]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&_route_prefix]
                    .lock()
                    .borrow()
                    .next_hop;
                (*_entry).out_ifindex = PEER_TABLE.lock().borrow()[&_peer_prefix]
                    .lock()
                    .borrow()
                    .peer_route_table
                    .lock()
                    .borrow()[&_route_prefix]
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
pub extern "C" fn peer_route_delete(_peer_prefix: u32, _route_prefix: u32) -> i32 {
    if PEER_TABLE.lock().borrow().contains_key(&_peer_prefix) {
        if PEER_TABLE.lock().borrow()[&_peer_prefix]
            .lock()
            .borrow()
            .peer_route_table
            .lock()
            .borrow()
            .contains_key(&_route_prefix)
        {
            println!("found route {} for peer {}", _route_prefix, _peer_prefix);
            if ROUTE_TABLE.lock().borrow().contains_key(&_route_prefix) {
                println!("found route entry in global table");
                ROUTE_TABLE.lock().borrow()[&_route_prefix]
                    .lock()
                    .borrow_mut()
                    .delete_peer(_peer_prefix);
                println!("peer is removed from route table entry peer list");
                if ROUTE_TABLE.lock().borrow()[&_route_prefix]
                    .lock()
                    .borrow()
                    .get_number_of_peers()
                    == 0
                {
                    println!("removing route from global as it was the last peer");
                    ROUTE_TABLE.lock().borrow_mut().remove(&_route_prefix);
                }
            }
            PEER_TABLE.lock().borrow_mut().remove(&_route_prefix);
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
