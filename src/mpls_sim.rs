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

type XcEntryWrapped = Arc<ReentrantMutex<RefCell<Box<XcEntry>>>>;
type FtnEntryWrapped = Arc<ReentrantMutex<RefCell<Box<FtnEntry>>>>;
type XcList = Vec<XcEntryWrapped>;
type FtnList = Vec<Arc<ReentrantMutex<RefCell<Box<FtnEntry>>>>>;
type IlmList = Vec<Arc<ReentrantMutex<RefCell<Box<IlmEntry>>>>>;
type OwnerList = Vec<Arc<ReentrantMutex<RefCell<Box<u32>>>>>;
type OutIfList = Vec<Arc<ReentrantMutex<RefCell<Box<u32>>>>>;
type FtnTable = Arc<ReentrantMutex<RefCell<PatriciaMap<FtnList>>>>;
type IlmTable = Arc<ReentrantMutex<RefCell<HashMap<IlmKey, IlmList>>>>;
type XcTable = Arc<ReentrantMutex<RefCell<HashMap<XcKey, XcEntryWrapped>>>>;
type NhlfeEntryWrapped = Arc<ReentrantMutex<RefCell<Box<NhlfeEntry>>>>;
type NhlfeTable = Arc<ReentrantMutex<RefCell<HashMap<NhlfeKey, NhlfeEntryWrapped>>>>;
type IdTable = Arc<ReentrantMutex<RefCell<Box<IdMap>>>>;

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
    pub static ref XC_ID_TABLE: IdTable =
        Arc::new(ReentrantMutex::new(RefCell::new(Box::new(IdMap {
            ids: [false; 1024]
        }))));
    pub static ref NHLFE_ID_TABLE: IdTable =
        Arc::new(ReentrantMutex::new(RefCell::new(Box::new(IdMap {
            ids: [false; 1024]
        }))));
}

pub struct IdMap {
    ids: [bool; 1024],
}

impl IdMap {
    fn get_free(&mut self) -> i32 {
        let mut i = 0;
        while (i < 1024) {
            if !self.ids[i] {
                self.ids[i] = true;
                return i as i32;
            }
        }
        -1
    }
    fn put_free(&mut self, idx: usize) {
        if idx >= 1024 {
            trace!("idx is too large {}", idx);
            return;
        }
        self.ids[idx] = false;
    }
}

pub enum Ftn_Table {
    V4(&'static FTN_TABLE4),
    V6(&'static FTN_TABLE6),
}

impl Ftn_Table {
    fn lookup_list(ftn_list: &FtnList, ftn_ix: u32) -> Option<FtnEntryWrapped> {
        trace!("Ftn_Table::lookup_list");
        let len: usize = ftn_list.len();
        let mut i = 0;
        while (i < len) {
            if read_val!(ftn_list[i]).ftn_ix == ftn_ix {
                return Some(Arc::clone(&ftn_list[i]));
            }
            i += 1;
        }
        None
    }
    fn insert_list(ftn_list: &mut FtnList, ftn_ix: u32, entry: FtnEntryWrapped) {
        trace!("Ftn_Table::insert_list");
        ftn_list.push(entry);
    }
    fn list_is_empty(ftn_list: &FtnList) -> bool {
        trace!("Ftn_Table::list_is_empty");
        ftn_list.len() == 0
    }
    fn remove_from_list(ftn_list: &mut FtnList, ftn_ix: u32) {
        trace!("Ftn_Table::remove_from_list");
        let len: usize = ftn_list.len();
        let mut i = 0;
        while (i < len) {
            if read_val!(ftn_list[i]).ftn_ix == ftn_ix {
                ftn_list.remove(i);
                break;
            }
            i += 1;
        }
    }
    fn lookup(&self, key: &FtnKey, ftn_ix: u32) -> Option<FtnEntryWrapped> {
        trace!("Ftn_Table::lookup");
        match self {
            Ftn_Table::V4(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V4(ipv4) => {
                        if read_val!(FTN_TABLE4).contains_key(ipv4.octets()) {
                            return Ftn_Table::lookup_list(
                                &read_val!(FTN_TABLE4).get(ipv4.octets()).unwrap(),
                                ftn_ix,
                            );
                        }
                    }
                    IpAddr::V6(ipv6) => {
                        trace!("IPv6 is not expected here!");
                    }
                },
            },
            Ftn_Table::V6(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V6(ipv6) => {
                        if read_val!(FTN_TABLE6).contains_key(ipv6.octets()) {
                            return Ftn_Table::lookup_list(
                                &read_val!(FTN_TABLE6).get(ipv6.octets()).unwrap(),
                                ftn_ix,
                            );
                        }
                    }
                    IpAddr::V4(ipv4) => {
                        trace!("IPv4 is not expected here!");
                    }
                },
            },
        }
        None
    }
    fn insert(&self, key: FtnKey, ftn_ix: u32, entry: FtnEntryWrapped) {
        trace!("Ftn_Table::insert");
        match self {
            Ftn_Table::V4(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V4(ipv4) => {
                        if !read_val!(FTN_TABLE4).contains_key(ipv4.octets()) {
                            trace!("not found, create new");
                            write_val!(FTN_TABLE4).insert(ipv4.octets(), Vec::new());
                        }
                        Ftn_Table::insert_list(
                            write_val!(FTN_TABLE4).get_mut(ipv4.octets()).unwrap(),
                            ftn_ix,
                            entry,
                        );
                    }
                    IpAddr::V6(ipv6) => {
                        trace!("IPv6 is not expected here!");
                    }
                },
            },
            Ftn_Table::V6(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V6(ipv6) => {
                        if !read_val!(FTN_TABLE6).contains_key(ipv6.octets()) {
                            trace!("not found, create new");
                            write_val!(FTN_TABLE6).insert(ipv6.octets(), Vec::new());
                        }
                        Ftn_Table::insert_list(
                            write_val!(FTN_TABLE6).get_mut(ipv6.octets()).unwrap(),
                            ftn_ix,
                            entry,
                        );
                    }
                    IpAddr::V4(ipv4) => {
                        trace!("IPv4 is not expected here!");
                    }
                },
            },
        }
    }
    fn remove(&self, key: &FtnKey, ftn_ix: u32) {
        trace!("Ftn_Table::remove");
        match self {
            Ftn_Table::V4(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V4(ipv4) => {
                        Ftn_Table::remove_from_list(
                            write_val!(FTN_TABLE4).get_mut(ipv4.octets()).unwrap(),
                            ftn_ix,
                        );
                        if Ftn_Table::list_is_empty(
                            read_val!(FTN_TABLE4).get(ipv4.octets()).unwrap(),
                        ) {
                            write_val!(FTN_TABLE4).remove(ipv4.octets());
                        }
                    }
                    IpAddr::V6(ipv6) => {
                        trace!("IPv6 is not expected here!");
                    }
                },
            },
            Ftn_Table::V6(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V6(ipv6) => {
                        Ftn_Table::remove_from_list(
                            write_val!(FTN_TABLE6).get_mut(ipv6.octets()).unwrap(),
                            ftn_ix,
                        );
                        if Ftn_Table::list_is_empty(
                            read_val!(FTN_TABLE6).get(ipv6.octets()).unwrap(),
                        ) {
                            write_val!(FTN_TABLE6).remove(ipv6.octets());
                        }
                    }
                    IpAddr::V4(ipv4) => {
                        trace!("IPv4 is not expected here!");
                    }
                },
            },
        }
    }
}

pub enum Nhlfe_Table {
    V4(&'static NHLFE_TABLE4),
    V6(&'static NHLFE_TABLE6),
}

impl Nhlfe_Table {
    fn lookup(&self, key: &NhlfeKey) -> Option<NhlfeEntryWrapped> {
        trace!("Nhlfe_Table::lookup");
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
        trace!("Nhlfe_Table::insert");
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
        trace!("Nhlfe_Table::remove");
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

pub enum Xc_Table {
    XC(&'static XC_TABLE),
}

impl Xc_Table {
    fn lookup(&self, key: &XcKey) -> Option<XcEntryWrapped> {
        trace!("Xc_Table::lookup");
        if read_val!(XC_TABLE).contains_key(key) {
            trace!("found");
            return Some(Arc::clone(&read_val!(XC_TABLE)[key]));
        }
        None
    }
    fn insert(&self, key: &XcKey, entry: XcEntryWrapped) {
        trace!("Xc_Table::insert");
        write_val!(XC_TABLE).insert(*key, Arc::clone(&entry));
    }
    fn remove(&self, key: &XcKey) {
        trace!("Xc_Table::remove");
        write_val!(XC_TABLE).remove(key);
    }
}

pub struct FtnKeyIp {
    prefix: IpAddr,
}

impl FtnKeyIp {
    fn new(prefix: IpAddr) -> FtnKeyIp {
        FtnKeyIp { prefix: prefix }
    }
}

pub enum FtnKey {
    IP(FtnKeyIp),
}

pub struct FtnEntry {
    ftn_key: FtnKey,
    ftn_ix: u32,
    xc_list: XcList,
    owner_list: OwnerList,
    out_if_list: OutIfList,
}

impl FtnEntry {
    fn new(key: FtnKey, idx: u32) -> FtnEntry {
        FtnEntry {
            ftn_key: key,
            ftn_ix: idx,
            xc_list: Vec::new(),
            owner_list: Vec::new(),
            out_if_list: Vec::new(),
        }
    }
    fn add_xc_entry(&mut self, entry: XcEntryWrapped) {
        trace!("add_xc_entry");
        self.xc_list.push(entry);
    }
    fn free_xc_list(&mut self) {
        trace!("freeing xc_list");
        let len: usize = self.xc_list.len();
        let mut i = 0;
        while (i < len) {
            Xc_Table::XC(&XC_TABLE).remove(&write_val!(self.xc_list[i]).xc_key);
            i += 1;
        }
    }
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
    nhlfe_ix: u32,
    xc_ix: u32,
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
        _xc_ix: u32,
        _nhlfe_ix: u32,
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
        NhlfeEntry {
            nhlfe_key: nhlfe_k,
            xc_ix: _xc_ix,
            nhlfe_ix: _nhlfe_ix,
        }
    }
}

impl Drop for NhlfeEntry {
    fn drop(&mut self) {
        trace!("Drop for NhlfeEntry");
        write_val!(NHLFE_ID_TABLE).put_free(self.nhlfe_ix as usize);
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct GenLabel {
    label: u32,
}
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct XcKey {
    in_iface: u32,
    gen_label: GenLabel,
    xc_ix: u32,
    nhlfe_ix: u32,
}

pub struct XcEntry {
    xc_key: XcKey,
    nhlfe: Option<NhlfeEntryWrapped>,
}

impl XcEntry {
    fn new(key: &XcKey, entry: Option<NhlfeEntryWrapped>) -> XcEntry {
        XcEntry {
            xc_key: *key,
            nhlfe: entry,
        }
    }
    fn set_nhlfe(&mut self, entry: Option<NhlfeEntryWrapped>) {
        trace!("setting NHLFE reference for XcEntry");
        self.nhlfe = entry;
    }
    fn cleanup(&mut self) {
        trace!("XcEntry::cleanup");
        match &self.nhlfe {
            Some(n) => match &read_val!(n).nhlfe_key {
                NhlfeKey::IP(nhlfe_ip) => match nhlfe_ip.next_hop {
                    IpAddr::V4(_) => {
                        Nhlfe_Table::V4(&NHLFE_TABLE4).remove(&read_val!(n).nhlfe_key);
                    }
                    IpAddr::V6(_) => {
                        Nhlfe_Table::V6(&NHLFE_TABLE6).remove(&read_val!(n).nhlfe_key);
                    }
                },
            },
            None => {
                trace!("No NHLFE for XC entry");
            }
        }
    }
}

impl Drop for XcEntry {
    fn drop(&mut self) {
        trace!("Drop for XcEntry");
        write_val!(XC_ID_TABLE).put_free(self.xc_key.xc_ix as usize);
        self.cleanup();
    }
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
    ftn_ix: u32,
}

impl FtnAddDataInt {
    pub fn new(
        _fec: IpAddr,
        _next_hop: IpAddr,
        _out_ifindex: u32,
        _out_label: u32,
        _ftn_ix: u32,
    ) -> FtnAddDataInt {
        FtnAddDataInt {
            fec: _fec,
            next_hop: _next_hop,
            out_ifindex: _out_ifindex,
            out_label: _out_label,
            ftn_ix: _ftn_ix,
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
            (*ftn_add_data).ftn_ix,
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
        ingress: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        egress: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
    });
    let xc_entry: XcEntryWrapped;
    let create_xc_entry = |xc_key: XcKey| {
        let new_xc_entry: XcEntryWrapped = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
            XcEntry::new(&xc_key, None),
        ))));
        trace!("insert new XcEntry to XC_TABLE");
        Xc_Table::XC(&XC_TABLE).insert(&xc_key, Arc::clone(&new_xc_entry));
        new_xc_entry
    };

    if ftn_add_data_int.next_hop.is_ipv4() {
        nhlfe = Nhlfe_Table::V4(&NHLFE_TABLE4).lookup(&nhlfe_k);
    } else {
        nhlfe = Nhlfe_Table::V6(&NHLFE_TABLE6).lookup(&nhlfe_k);
    }
    let xc_key: XcKey;
    match nhlfe {
        Some(nhlfe_exists) => {
            trace!("found NHLFE");
            xc_key = XcKey {
                in_iface: 0,
                gen_label: GenLabel { label: 0 },
                xc_ix: read_val!(nhlfe_exists).xc_ix,
                nhlfe_ix: read_val!(nhlfe_exists).nhlfe_ix,
            };
            let xc_entry_o = Xc_Table::XC(&XC_TABLE).lookup(&xc_key);
            match xc_entry_o {
                Some(o) => {
                    trace!("found XC for NHLFE");
                    xc_entry = o;
                }
                None => {
                    xc_entry = create_xc_entry(xc_key);
                }
            }
        }
        None => {
            let xc_ix = write_val!(XC_ID_TABLE).get_free();
            let nhlfe_ix = write_val!(NHLFE_ID_TABLE).get_free();
            if xc_ix == -1 {
                trace!("cannot allocate xc ix");
                write_val!(NHLFE_ID_TABLE).put_free(nhlfe_ix as usize);
                return -1;
            }
            if nhlfe_ix == -1 {
                trace!("cannot allocate nhlfe_ix");
                write_val!(XC_ID_TABLE).put_free(xc_ix as usize);
                return -1;
            }
            xc_key = XcKey {
                in_iface: 0,
                gen_label: GenLabel { label: 0 },
                xc_ix: xc_ix as u32,
                nhlfe_ix: nhlfe_ix as u32,
            };
            xc_entry = create_xc_entry(xc_key);
            let nhlfe_entry: NhlfeEntryWrapped = Arc::new(ReentrantMutex::new(RefCell::new(
                Box::new(NhlfeEntry::new(
                    ftn_add_data_int.next_hop,
                    ftn_add_data_int.out_label,
                    ftn_add_data_int.out_ifindex,
                    0,
                    0,
                    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    xc_ix as u32,
                    nhlfe_ix as u32,
                )),
            )));
            write_val!(xc_entry).set_nhlfe(Some(Arc::clone(&nhlfe_entry)));
            if ftn_add_data_int.next_hop.is_ipv4() {
                Nhlfe_Table::V4(&NHLFE_TABLE4).insert(nhlfe_k, nhlfe_entry);
            } else {
                Nhlfe_Table::V6(&NHLFE_TABLE6).insert(nhlfe_k, nhlfe_entry);
            }
        }
    }
    let ftn_entry: FtnEntryWrapped =
        Arc::new(ReentrantMutex::new(RefCell::new(Box::new(FtnEntry::new(
            FtnKey::IP(FtnKeyIp::new(ftn_add_data_int.fec)),
            ftn_add_data_int.ftn_ix,
        )))));
    write_val!(ftn_entry).add_xc_entry(xc_entry);
    match ftn_add_data_int.fec {
        IpAddr::V4(ipv4) => {
            Ftn_Table::V4(&FTN_TABLE4).insert(
                FtnKey::IP(FtnKeyIp::new(ftn_add_data_int.fec)),
                ftn_add_data_int.ftn_ix,
                ftn_entry,
            );
        }
        IpAddr::V6(ipv6) => {
            Ftn_Table::V6(&FTN_TABLE6).insert(
                FtnKey::IP(FtnKeyIp::new(ftn_add_data_int.fec)),
                ftn_add_data_int.ftn_ix,
                ftn_entry,
            );
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn ftn_add(ftn_add_data: *mut FtnAddData) -> i32 {
    let mut ftn_add_int: FtnAddDataInt;
    trace!("ftn_add");
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

struct FtnDelDataInt {
    fec: IpAddr,
    ftn_ix: u32,
}

impl FtnDelDataInt {
    pub fn new(_fec: IpAddr, _ftn_ix: u32) -> FtnDelDataInt {
        FtnDelDataInt {
            fec: _fec,
            ftn_ix: _ftn_ix,
        }
    }
}

unsafe fn convert_ftn_del_to_internal(ftn_del_data: *mut FtnDelData) -> Result<FtnDelDataInt, i32> {
    let ftn_del_int: FtnDelDataInt;
    let fec: IpAddr;

    unsafe {
        let mut addr_ptr: *mut u8 = (*ftn_del_data).fec.addr;
        if (*ftn_del_data).fec.family == 1 {
            fec = copy_ip_addr_v4_from_user(addr_ptr);
        } else {
            fec = copy_ip_addr_v6_from_user(addr_ptr as *mut u16);
        }
        ftn_del_int = FtnDelDataInt::new(fec, (*ftn_del_data).ftn_ix);
    }
    Ok(ftn_del_int)
}

fn _ftn_del(ftn_del_data_int: &FtnDelDataInt) -> i32 {
    let ftn_entry: FtnEntryWrapped;
    match ftn_del_data_int.fec {
        IpAddr::V4(ipv4) => {
            match Ftn_Table::V4(&FTN_TABLE4).lookup(
                &FtnKey::IP(FtnKeyIp::new(ftn_del_data_int.fec)),
                ftn_del_data_int.ftn_ix,
            ) {
                Some(e) => {
                    write_val!(e).free_xc_list();
                    Ftn_Table::V4(&FTN_TABLE4).remove(
                        &FtnKey::IP(FtnKeyIp::new(ftn_del_data_int.fec)),
                        ftn_del_data_int.ftn_ix,
                    );
                }
                None => {
                    trace!("cannot find FTN entry");
                    return -1;
                }
            }
        }
        IpAddr::V6(ipv6) => {
            match Ftn_Table::V6(&FTN_TABLE6).lookup(
                &FtnKey::IP(FtnKeyIp::new(ftn_del_data_int.fec)),
                ftn_del_data_int.ftn_ix,
            ) {
                Some(e) => {
                    write_val!(e).free_xc_list();
                    Ftn_Table::V6(&FTN_TABLE6).remove(
                        &FtnKey::IP(FtnKeyIp::new(ftn_del_data_int.fec)),
                        ftn_del_data_int.ftn_ix,
                    );
                }
                None => {
                    trace!("cannot find FTN entry");
                    return -1;
                }
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn ftn_del(ftn_del_data: *mut FtnDelData) -> i32 {
    let mut ftn_del_int: FtnDelDataInt;
    trace!("ftn_del");
    unsafe {
        match convert_ftn_del_to_internal(ftn_del_data) {
            Ok(ret_val) => {
                ftn_del_int = ret_val;
            }
            Err(_) => {
                trace!("cannot convert ftn_del to internal");
                return -1;
            }
        }
    }
    _ftn_del(&ftn_del_int)
}

#[no_mangle]
pub extern "C" fn ilm_add(ftn_add_data: *mut IlmAddData) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn ilm_del(ftn_add_data: *mut IlmDelData) -> i32 {
    0
}
