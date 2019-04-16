#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;

#[repr(C)]
pub struct RouteEntry {
    prefix:  u32,
    next_hop: u32,
    out_ifindex: u32
}

impl RouteEntry {
    pub fn new(_prefix: u32, _next_hop: u32, _out_ifindex: u32) -> RouteEntry {
        RouteEntry { prefix: _prefix, next_hop: _next_hop, out_ifindex: _out_ifindex }
    }
    pub fn get_prefix(&self) -> u32 {
        self.prefix
    }
    pub fn get_next_hop(&self) -> u32 {
        self.next_hop
    }
    pub fn get_out_ifindex(&self) -> u32 {
        self.out_ifindex
    }
}

lazy_static! {
    static ref ROUTE_TABLE: Mutex<HashMap<u32, Box<RouteEntry>>> = Mutex::new(HashMap::new());
}

#[no_mangle]
pub extern "C" fn route_add(_prefix: u32, _entry: *mut RouteEntry) -> i32 {
    let mut entry: Box<RouteEntry>;
    unsafe {
        entry = Box::from_raw(_entry); 
    }
    println!("route_add: key: {} prefix: {} next_hop: {} out_ifindex: {}",_prefix,entry.prefix, entry.next_hop,entry.out_ifindex);
    if ROUTE_TABLE.lock().unwrap().contains_key(&_prefix) {
        -1
    } else {
        let new_entry = Box::new(RouteEntry::new(entry.prefix,entry.next_hop,entry.out_ifindex));
        let _m_entry = Box::into_raw(entry);
        ROUTE_TABLE.lock().unwrap().insert(_prefix, new_entry);
        0
    }
}

#[no_mangle]
pub extern "C" fn route_lookup(_prefix: u32, _entry: *mut RouteEntry) -> i32 {
   if ROUTE_TABLE.lock().unwrap().contains_key(&_prefix) {
        let re = &ROUTE_TABLE.lock().unwrap()[&_prefix];
        unsafe {
            println!("route_lookup prefix {}, found prefix {} next_hop {} out_ifindex {}",_prefix,re.prefix,re.next_hop,re.out_ifindex);
            (*_entry).prefix = re.prefix;
            (*_entry).next_hop = re.next_hop;
            (*_entry).out_ifindex = re.out_ifindex;
        }
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn route_delete(_prefix: u32) -> i32 {
    if ROUTE_TABLE.lock().unwrap().contains_key(&_prefix) {
        println!("deleting existing entry");
        ROUTE_TABLE.lock().unwrap().remove(&_prefix);
        println!("done");
        0
    } else {
        -1
    }
}
