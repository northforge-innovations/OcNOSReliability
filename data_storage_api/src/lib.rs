#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;

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

pub fn route_add(_prefix: u32, _entry: Box<RouteEntry>) {
    println!("route_add: key: {} prefix: {} next_hop: {} out_ifindex: {}",_prefix,_entry.prefix, _entry.next_hop,_entry.out_ifindex);
    ROUTE_TABLE.lock().unwrap().insert(_prefix, _entry);
}

pub fn route_lookup(_prefix: u32, _entry: &mut RouteEntry) -> Result<i32, i32> {
   if ROUTE_TABLE.lock().unwrap().contains_key(&_prefix) {
        let re = &ROUTE_TABLE.lock().unwrap()[&_prefix];
        _entry.prefix = re.prefix;
        _entry.next_hop = re.next_hop;
        _entry.out_ifindex = re.out_ifindex;
        Ok(0)
    } else {
        Err(-1)
    }
}

pub fn route_delete(_prefix: u32) -> Result<i32, i32> {
    ROUTE_TABLE.lock().unwrap().remove(&_prefix);
    Ok(0)
}
