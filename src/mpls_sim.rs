#[macro_use]
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
use std::collections::LinkedList;
#[path = "external_types.rs"]
mod external_types;
use external_types::*;
use log::*;
#[path = "utils.rs"]
mod utils;
use utils::*;
#[path = "macros.rs"]
#[macro_use]
mod macros;

type XcList = LinkedList<Arc<ReentrantMutex<RefCell<Box<XcEntry>>>>>;
type FtnList = LinkedList<Arc<ReentrantMutex<RefCell<Box<FtnEntry>>>>>;
type IlmList = LinkedList<Arc<ReentrantMutex<RefCell<Box<IlmEntry>>>>>;
type OwnerList = LinkedList<Arc<ReentrantMutex<RefCell<Box<u32>>>>>;
type OutIfList = LinkedList<Arc<ReentrantMutex<RefCell<Box<u32>>>>>;
type FtnTable = Arc<ReentrantMutex<RefCell<PatriciaMap<FtnList>>>>;
type IlmTable = Arc<ReentrantMutex<RefCell<HashMap<IlmKey, IlmList>>>>;
type XcTable = Arc<ReentrantMutex<RefCell<HashMap<XcKey, XcList>>>>;
type NhlfeEntryWrapped = Arc<ReentrantMutex<RefCell<Box<NhlfeEntry>>>>;
type NhlfeTable = Arc<ReentrantMutex<RefCell<HashMap<NhlfeKey, NhlfeEntryWrapped>>>>;

lazy_static! {
    pub static ref FTN_TABLE4: FtnTable =
        Arc::new(ReentrantMutex::new(RefCell::new(PatriciaMap::new())));
    pub static ref FTN_TABLE6: FtnTable =
        Arc::new(ReentrantMutex::new(RefCell::new(PatriciaMap::new())));
    pub static ref ILM_TABLE: IlmTable =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref XC_TABLE: XcTable = Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref NHLFE_TABLE4: NhlfeTable =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
    pub static ref NHLFE_TABLE6: NhlfeTable =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
}

pub enum Nhlfe_Table {
    V4(&'static NHLFE_TABLE4),
    V6(&'static NHLFE_TABLE6),
}

impl Nhlfe_Table {
    fn lookup(&self, key: &NhlfeKey) -> Option<NhlfeEntryWrapped> {
        match self {
            Nhlfe_Table::V4(_) => {
                if read_val!(NHLFE_TABLE4).contains_key(key) {
                    return Some(Arc::clone(&read_val!(NHLFE_TABLE4)[key]));
                }
            }
            Nhlfe_Table::V6(_) => {
                if read_val!(NHLFE_TABLE6).contains_key(key) {
                    return Some(Arc::clone(&read_val!(NHLFE_TABLE6)[key]));
                }
            }
        }
        None
    }
    fn insert(&self, key: NhlfeKey, entry: NhlfeEntryWrapped) {
        match self {
            Nhlfe_Table::V4(_) => {
                write_val!(NHLFE_TABLE4).insert(key, Arc::clone(&entry));
            }
            Nhlfe_Table::V6(_) => {
                write_val!(NHLFE_TABLE6).insert(key, Arc::clone(&entry));
            }
        }
    }
    fn remove(&self, key: &NhlfeKey) {
        match self {
            Nhlfe_Table::V4(_) => {
                write_val!(NHLFE_TABLE4).remove(key);
            }
            Nhlfe_Table::V6(_) => {
                write_val!(NHLFE_TABLE6).remove(key);
            }
        }
    }
}

pub struct FtnKeyIp {
    prefix: IpAddr,
}

pub enum FtnKey {
    IP(FtnKeyIp),
}

pub struct FtnEntry {
    ftn_key: FtnKey,
    ftn_idx: u32,
    xc_list: XcList,
    owner_list: OwnerList,
    out_if_list: OutIfList,
}

#[derive(PartialEq, Eq, Hash)]
pub struct NhlfeKeyIp {
    next_hop: IpAddr,
    out_label: u32,
    out_iface: u32,
    trunk_id: u16,
    lsp_id: u16,
    ingress: IpAddr,
    egress: IpAddr,
}

#[derive(PartialEq, Eq, Hash)]
pub enum NhlfeKey {
    IP(NhlfeKeyIp),
}

pub struct NhlfeEntry {
    nhlfe_key: NhlfeKey,
}

impl NhlfeEntry {
    fn new(
        next_hop: IpAddr,
        out_label: u32,
        out_iface: u32,
        trunk_id: u16,
        lsp_id: u16,
        ingress: IpAddr,
        egress: IpAddr,
    ) -> NhlfeEntry {
        let nhlfe_k: NhlfeKey = NhlfeKey::IP(NhlfeKeyIp {
            next_hop: next_hop,
            out_label: out_label,
            out_iface: out_iface,
            trunk_id: trunk_id,
            lsp_id: lsp_id,
            ingress: ingress,
            egress: egress,
        });
        NhlfeEntry { nhlfe_key: nhlfe_k }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct GenLabel {
    label: u32,
}
#[derive(PartialEq, Eq, Hash)]
pub struct XcKey {
    in_iface: u32,
    gen_label: GenLabel,
}

pub struct XcEntry {
    xc_key: XcKey,
    nhlfe: Arc<ReentrantMutex<RefCell<Box<NhlfeEntry>>>>,
}

#[derive(PartialEq, Eq, Hash)]
pub struct IlmKeyPkt {
    in_iface: u32,
    in_label: u32,
}
#[derive(PartialEq, Eq, Hash)]
pub enum IlmKey {
    PKT(IlmKeyPkt),
}

pub struct IlmEntry {
    ilm_key: IlmKey,
    ilm_idx: u32,
    xc_list: XcList,
    owner_list: OwnerList,
    out_if_list: OutIfList,
}

struct FtnAddDataInt {
    fec: IpAddr,
    next_hop: IpAddr,
    out_ifindex: u32,
    out_label: u32,
    ftn_idx: u32,
}

impl FtnAddDataInt {
    pub fn new(
        _fec: IpAddr,
        _next_hop: IpAddr,
        _out_ifindex: u32,
        _out_label: u32,
        _ftn_idx: u32,
    ) -> FtnAddDataInt {
        FtnAddDataInt {
            fec: _fec,
            next_hop: _next_hop,
            out_ifindex: _out_ifindex,
            out_label: _out_label,
            ftn_idx: _ftn_idx,
        }
    }
}

unsafe fn convert_ftn_add_to_internal(ftn_add_data: *mut FtnAddData) -> Result<FtnAddDataInt, i32> {
    let ftn_add_int: FtnAddDataInt;
    let fec: IpAddr;
    let next_hop: IpAddr;

    unsafe {
        let mut addr_ptr: *mut u8 = (*ftn_add_data).fec.addr;
        if (*ftn_add_data).fec.family == 1 {
            fec = copy_ip_addr_v4_from_user(addr_ptr);
        } else {
            fec = copy_ip_addr_v6_from_user(addr_ptr as *mut u16);
        }
        addr_ptr = (*ftn_add_data).next_hop.addr;
        if (*ftn_add_data).next_hop.family == 1 {
            next_hop = copy_ip_addr_v4_from_user(addr_ptr);
        } else {
            next_hop = copy_ip_addr_v6_from_user(addr_ptr as *mut u16);
        }
        ftn_add_int = FtnAddDataInt::new(
            fec,
            next_hop,
            (*ftn_add_data).out_ifindex,
            *(*ftn_add_data).out_label,
            (*ftn_add_data).ftn_idx,
        );
    }
    Ok(ftn_add_int)
}

fn _ftn_add(ftn_add_data_int: &FtnAddDataInt) -> i32 {
    let nhlfe: Option<NhlfeEntryWrapped>;
    let nhlfe_k: NhlfeKey = NhlfeKey::IP(NhlfeKeyIp {
            next_hop: ftn_add_data_int.next_hop,
            out_label: ftn_add_data_int.out_label,
            out_iface: ftn_add_data_int.out_ifindex,
            trunk_id: 0,
            lsp_id: 0,
            ingress: IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
            egress: IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
        });
    if ftn_add_data_int.next_hop.is_ipv4() {
        nhlfe = Nhlfe_Table::V4(&NHLFE_TABLE4).lookup(&nhlfe_k);
    } else {
        nhlfe = Nhlfe_Table::V6(&NHLFE_TABLE6).lookup(&nhlfe_k);
    }
    match nhlfe {
        Some(nhlfe_exists) => {},
        None => {
            let nhlfe_entry: NhlfeEntryWrapped = Arc::new(ReentrantMutex::new(RefCell::new(
                Box::new(NhlfeEntry::new(
                    ftn_add_data_int.next_hop,
                    ftn_add_data_int.out_label,
                    ftn_add_data_int.out_ifindex,
                    0,
                    0,
                    IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
                    IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
                )),
            )));
            if ftn_add_data_int.next_hop.is_ipv4() {
                Nhlfe_Table::V4(&NHLFE_TABLE4).insert(nhlfe_k, nhlfe_entry);
            } else {
                Nhlfe_Table::V6(&NHLFE_TABLE6).insert(nhlfe_k, nhlfe_entry);
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn ftn_add(ftn_add_data: *mut FtnAddData) -> i32 {
    let mut ftn_add_int: FtnAddDataInt;
    unsafe {
        match convert_ftn_add_to_internal(ftn_add_data) {
            Ok(ret_val) => {
                ftn_add_int = ret_val;
            }
            Err(_) => {
                return -1;
            }
        }
    }
    _ftn_add(&ftn_add_int)
}

#[no_mangle]
pub extern "C" fn ftn_del(ftn_add_data: *mut FtnDelData) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn ilm_add(ftn_add_data: *mut IlmAddData) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn ilm_del(ftn_add_data: *mut IlmDelData) -> i32 {
    0
}
