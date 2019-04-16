#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;

#[repr(C)]
pub struct RouteEntry {
    prefix:  u32,
    mask:    u32,
    next_hop: u32,
    out_ifindex: u32
}

#[repr(C)]
pub struct PeerEntry {
    prefix:  u32,
    out_ifindex: u32
}

impl RouteEntry {
    pub fn new(_prefix: u32, _mask: u32, _next_hop: u32, _out_ifindex: u32) -> RouteEntry {
        RouteEntry { prefix: _prefix, mask: _mask, next_hop: _next_hop, out_ifindex: _out_ifindex }
    }
    pub fn get_prefix(&self) -> u32 {
        self.prefix
    }
    pub fn get_mask(&self) -> u32 {
	self.mask
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

lazy_static! {
    static ref PEER_TABLE: Mutex<HashMap<u32, Box<PeerIntEntry>>> = Mutex::new(HashMap::new());
}

#[no_mangle]
pub extern "C" fn route_add(_prefix: u32,_entry: *mut RouteEntry) -> i32 {
    let mut entry: Box<RouteEntry>;
    unsafe {
        entry = Box::from_raw(_entry); 
    }
    println!("route_add: key: {} prefix: {} mask: {} next_hop: {} out_ifindex: {}",_prefix,entry.prefix, entry.mask,entry.next_hop,entry.out_ifindex);
    if ROUTE_TABLE.lock().unwrap().contains_key(&_prefix) {
        -1
    } else {
        let new_entry = Box::new(RouteEntry::new(entry.prefix,entry.mask,entry.next_hop,entry.out_ifindex));
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
            println!("route_lookup prefix {}, found prefix {} mask {} next_hop {} out_ifindex {}",_prefix,re.prefix,re.mask,re.next_hop,re.out_ifindex);
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

struct PeerIntEntry {
	prefix: u32,
	out_ifindex: u32,
	peer_route_table: Mutex<HashMap<u32, Box<RouteEntry>>>,
}

impl PeerIntEntry {
    pub fn new(_prefix: u32, _out_ifindex: u32) -> PeerIntEntry {
        PeerIntEntry { prefix: _prefix, out_ifindex: _out_ifindex, peer_route_table: Mutex::new(HashMap::new()) }
    }
    pub fn get_prefix(&self) -> u32 {
        self.prefix
    }
    pub fn get_out_ifindex(&self) -> u32 {
        self.out_ifindex
    }
}

#[no_mangle]
pub extern "C" fn peer_add(_prefix: u32,_entry: *mut PeerEntry) -> i32 {
    let mut entry: Box<PeerEntry>;
    unsafe {
        entry = Box::from_raw(_entry); 
    }
    println!("peer_add: key: {} prefix: {} out_ifindex: {}",_prefix,entry.prefix, entry.out_ifindex);
    if PEER_TABLE.lock().unwrap().contains_key(&_prefix) {
        -1
    } else {
        let new_entry = Box::new(PeerIntEntry::new(entry.prefix,entry.out_ifindex));
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
            println!("peer_lookup prefix {}, found prefix {} out_ifindex {}",_prefix,re.prefix,re.out_ifindex);
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
    if PEER_TABLE.lock().unwrap().contains_key(&_prefix) {
        println!("deleting existing entry");
        PEER_TABLE.lock().unwrap().remove(&_prefix);
        println!("done");
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn peer_route_add(_peer_prefix: u32,_entry: *mut RouteEntry) -> i32 {
    if PEER_TABLE.lock().unwrap().contains_key(&_peer_prefix) {
        let re = &PEER_TABLE.lock().unwrap()[&_peer_prefix];
        println!("peer_route_add: found prefix {} out_ifindex {}",re.prefix,re.out_ifindex);
        let mut entry: Box<RouteEntry>;
	unsafe {
        	entry = Box::from_raw(_entry); 
    	}
        if re.peer_route_table.lock().unwrap().contains_key(&entry.prefix) {
	    -2
        } else {
	    let new_entry = Box::new(RouteEntry::new(entry.prefix,entry.mask,entry.next_hop,entry.out_ifindex));
            let _m_entry = Box::into_raw(entry);
            println!("peer_route_add: key: {} prefix: {} out_ifindex: {}",new_entry.prefix,new_entry.prefix, new_entry.out_ifindex);
            re.peer_route_table.lock().unwrap().insert(new_entry.prefix, new_entry); 
            0
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn peer_route_lookup(_peer_prefix: u32, _route_prefix: u32,_entry: *mut RouteEntry) -> i32 {
   if PEER_TABLE.lock().unwrap().contains_key(&_peer_prefix) {
        let pe = &PEER_TABLE.lock().unwrap()[&_peer_prefix];
        if pe.peer_route_table.lock().unwrap().contains_key(&_route_prefix) {
	    let re = &pe.peer_route_table.lock().unwrap()[&_route_prefix];
	    unsafe {
        	    println!("peer_route_lookup peer_prefix {}, found prefix {}",_peer_prefix,_route_prefix);
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
   if PEER_TABLE.lock().unwrap().contains_key(&_peer_prefix) {
        let re = &PEER_TABLE.lock().unwrap()[&_peer_prefix];
        if re.peer_route_table.lock().unwrap().contains_key(&_route_prefix) {
	    re.peer_route_table.lock().unwrap().remove(&_route_prefix);
	    0
	} else {
	    -2
	}
    } else {
        -1
    }
}
