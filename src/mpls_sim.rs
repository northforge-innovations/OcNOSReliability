#[macro_use]
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
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
type IlmEntryWrapped = Arc<ReentrantMutex<RefCell<Box<IlmEntry>>>>;
type XcList = Vec<XcEntryWrapped>;
type FtnList = Vec<FtnEntryWrapped>;
type IlmList = Vec<IlmEntryWrapped>;
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
        let i = 0;
        while i < 1024 {
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

pub enum FtnTableGen {
    V4(&'static FTN_TABLE4),
    V6(&'static FTN_TABLE6),
}

impl FtnTableGen {
    fn lookup_list(ftn_list: &FtnList, ftn_ix: u32) -> Option<FtnEntryWrapped> {
        trace!("FtnTableGen::lookup_list");
        let len: usize = ftn_list.len();
        let mut i = 0;
        while i < len {
            if read_val!(ftn_list[i]).ftn_ix == ftn_ix {
                return Some(Arc::clone(&ftn_list[i]));
            }
            i += 1;
        }
        None
    }
    fn insert_list(ftn_list: &mut FtnList, entry: FtnEntryWrapped) {
        trace!("FtnTableGen::insert_list");
        ftn_list.push(entry);
    }
    fn list_is_empty(ftn_list: &FtnList) -> bool {
        trace!("FtnTableGen::list_is_empty");
        ftn_list.len() == 0
    }
    fn remove_from_list(ftn_list: &mut FtnList, ftn_ix: u32) {
        trace!("FtnTableGen::remove_from_list");
        let len: usize = ftn_list.len();
        let mut i = 0;
        while i < len {
            if read_val!(ftn_list[i]).ftn_ix == ftn_ix {
                ftn_list.remove(i);
                break;
            }
            i += 1;
        }
    }
    fn lookup(&self, key: &FtnKey, ftn_ix: u32) -> Option<FtnEntryWrapped> {
        trace!("FtnTableGen::lookup");
        match self {
            FtnTableGen::V4(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V4(ipv4) => {
                        if read_val!(FTN_TABLE4).contains_key(ipv4.octets()) {
                            return FtnTableGen::lookup_list(
                                &read_val!(FTN_TABLE4).get(ipv4.octets()).unwrap(),
                                ftn_ix,
                            );
                        }
                    }
                    IpAddr::V6(_) => {
                        trace!("IPv6 is not expected here!");
                    }
                },
            },
            FtnTableGen::V6(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V6(ipv6) => {
                        if read_val!(FTN_TABLE6).contains_key(ipv6.octets()) {
                            return FtnTableGen::lookup_list(
                                &read_val!(FTN_TABLE6).get(ipv6.octets()).unwrap(),
                                ftn_ix,
                            );
                        }
                    }
                    IpAddr::V4(_) => {
                        trace!("IPv4 is not expected here!");
                    }
                },
            },
        }
        None
    }
    fn insert(&self, key: FtnKey, entry: FtnEntryWrapped) {
        trace!("FtnTableGen::insert");
        match self {
            FtnTableGen::V4(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V4(ipv4) => {
                        if !read_val!(FTN_TABLE4).contains_key(ipv4.octets()) {
                            trace!("not found, create new");
                            write_val!(FTN_TABLE4).insert(ipv4.octets(), Vec::new());
                        }
                        FtnTableGen::insert_list(
                            write_val!(FTN_TABLE4).get_mut(ipv4.octets()).unwrap(),
                            entry,
                        );
                    }
                    IpAddr::V6(_) => {
                        trace!("IPv6 is not expected here!");
                    }
                },
            },
            FtnTableGen::V6(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V6(ipv6) => {
                        if !read_val!(FTN_TABLE6).contains_key(ipv6.octets()) {
                            trace!("not found, create new");
                            write_val!(FTN_TABLE6).insert(ipv6.octets(), Vec::new());
                        }
                        FtnTableGen::insert_list(
                            write_val!(FTN_TABLE6).get_mut(ipv6.octets()).unwrap(),
                            entry,
                        );
                    }
                    IpAddr::V4(_) => {
                        trace!("IPv4 is not expected here!");
                    }
                },
            },
        }
    }
    fn remove(&self, key: &FtnKey, ftn_ix: u32) {
        trace!("FtnTableGen::remove");
        match self {
            FtnTableGen::V4(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V4(ipv4) => {
                        FtnTableGen::remove_from_list(
                            write_val!(FTN_TABLE4).get_mut(ipv4.octets()).unwrap(),
                            ftn_ix,
                        );
                        if FtnTableGen::list_is_empty(
                            read_val!(FTN_TABLE4).get(ipv4.octets()).unwrap(),
                        ) {
                            write_val!(FTN_TABLE4).remove(ipv4.octets());
                        }
                    }
                    IpAddr::V6(_) => {
                        trace!("IPv6 is not expected here!");
                    }
                },
            },
            FtnTableGen::V6(_) => match key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    IpAddr::V6(ipv6) => {
                        FtnTableGen::remove_from_list(
                            write_val!(FTN_TABLE6).get_mut(ipv6.octets()).unwrap(),
                            ftn_ix,
                        );
                        if FtnTableGen::list_is_empty(
                            read_val!(FTN_TABLE6).get(ipv6.octets()).unwrap(),
                        ) {
                            write_val!(FTN_TABLE6).remove(ipv6.octets());
                        }
                    }
                    IpAddr::V4(_) => {
                        trace!("IPv4 is not expected here!");
                    }
                },
            },
        }
    }
}

pub enum NhlfeTableGen {
    V4(&'static NHLFE_TABLE4),
    V6(&'static NHLFE_TABLE6),
}

impl NhlfeTableGen {
    fn lookup(&self, key: &NhlfeKey) -> Option<NhlfeEntryWrapped> {
        trace!("NhlfeTableGen::lookup");
        match self {
            NhlfeTableGen::V4(_) => {
                if read_val!(NHLFE_TABLE4).contains_key(key) {
                    return Some(Arc::clone(&read_val!(NHLFE_TABLE4)[key]));
                }
            }
            NhlfeTableGen::V6(_) => {
                if read_val!(NHLFE_TABLE6).contains_key(key) {
                    return Some(Arc::clone(&read_val!(NHLFE_TABLE6)[key]));
                }
            }
        }
        None
    }
    fn insert(&self, key: NhlfeKey, entry: NhlfeEntryWrapped) {
        trace!("NhlfeTableGen::insert");
        match self {
            NhlfeTableGen::V4(_) => {
                write_val!(NHLFE_TABLE4).insert(key, Arc::clone(&entry));
            }
            NhlfeTableGen::V6(_) => {
                write_val!(NHLFE_TABLE6).insert(key, Arc::clone(&entry));
            }
        }
    }
    fn remove(&self, key: &NhlfeKey) {
        trace!("NhlfeTableGen::remove");
        match self {
            NhlfeTableGen::V4(_) => {
                write_val!(NHLFE_TABLE4).remove(key);
            }
            NhlfeTableGen::V6(_) => {
                write_val!(NHLFE_TABLE6).remove(key);
            }
        }
    }
}

pub enum XcTableGen {
    XC(&'static XC_TABLE),
}

impl XcTableGen {
    fn lookup(&self, key: &XcKey) -> Option<XcEntryWrapped> {
        trace!("XcTableGen::lookup");
        if read_val!(XC_TABLE).contains_key(key) {
            trace!("found");
            return Some(Arc::clone(&read_val!(XC_TABLE)[key]));
        }
        None
    }
    fn insert(&self, key: &XcKey, entry: XcEntryWrapped) {
        trace!("XcTableGen::insert");
        write_val!(XC_TABLE).insert(*key, Arc::clone(&entry));
    }
    fn remove(&self, key: &XcKey) {
        trace!("XcTableGen::remove");
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
    fec: IpAddr,
    ftn_ix: u32,
    xc_list: XcList,
}

impl FtnEntry {
    fn new(fec: IpAddr, idx: u32) -> FtnEntry {
        FtnEntry {
            fec: fec,
            ftn_ix: idx,
            xc_list: Vec::new(),
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
        while i < len {
            XcTableGen::XC(&XC_TABLE).remove(&write_val!(self.xc_list[i]).xc_key);
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
                        NhlfeTableGen::V4(&NHLFE_TABLE4).remove(&read_val!(n).nhlfe_key);
                    }
                    IpAddr::V6(_) => {
                        NhlfeTableGen::V6(&NHLFE_TABLE6).remove(&read_val!(n).nhlfe_key);
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

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct IlmKeyPkt {
    in_iface: u32,
    in_label: u32,
}

impl IlmKeyPkt {
    fn new(in_label: u32, in_iface: u32) -> IlmKeyPkt {
        IlmKeyPkt {
            in_iface: in_iface,
            in_label: in_label,
        }
    }
}
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum IlmKey {
    PKT(IlmKeyPkt),
}

pub struct IlmEntry {
    ilm_key: IlmKey,
    ilm_ix: u32,
    xc_list: XcList,
    owner: u32,
}

impl IlmEntry {
    fn new(ilm_key: IlmKey, ilm_ix: u32, owner: u32) -> IlmEntry {
        IlmEntry {
            ilm_key: ilm_key,
            ilm_ix: ilm_ix,
            xc_list: Vec::new(),
            owner: owner,
        }
    }
}

pub enum IlmTableGen {
    ILM(&'static ILM_TABLE),
}

impl IlmTableGen {
    fn lookup_list(&self, ilm_list: &IlmList, compare: &Fn(&IlmEntryWrapped)->bool) -> Option<IlmEntryWrapped> {
        trace!("IlmTableGen::lookup_list");
        let len: usize = ilm_list.len();
        let mut i = 0;
        while i < len {
            if compare(&ilm_list[i]) {
                return Some(Arc::clone(&ilm_list[i]));
            }
            i += 1;
        }
        None
    }
    fn lookup_by_owner(&self, ilm_key: &IlmKey, owner: u32) -> Option<IlmEntryWrapped> {
        if read_val!(&ILM_TABLE).contains_key(ilm_key) {
            return self.lookup_list(&read_val!(&ILM_TABLE)[ilm_key], &|ie| owner == read_val!(ie).owner);
        }
        None
    }
    fn lookup_by_ix(&self, ilm_key: &IlmKey, ilm_ix: u32) -> Option<IlmEntryWrapped> {
        if read_val!(&ILM_TABLE).contains_key(ilm_key) {
            return self.lookup_list(&read_val!(&ILM_TABLE)[ilm_key], &|ie| ilm_ix == read_val!(ie).ilm_ix);
        }
        None
    }
    fn insert_list(ilm_list: &mut IlmList, ilm_entry: IlmEntryWrapped) {
        ilm_list.push(ilm_entry);
    }
    fn insert(&mut self, ilm_key: IlmKey, ilm_entry: IlmEntryWrapped) {
        if read_val!(&ILM_TABLE).contains_key(&ilm_key) {
            let ilm_list: IlmList = Vec::new();
            write_val!(&ILM_TABLE).insert(ilm_key, ilm_list);
        }
        IlmTableGen::insert_list(
            &mut write_val!(&ILM_TABLE).get_mut(&ilm_key).unwrap(),
            ilm_entry,
        );
    }
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
        XcTableGen::XC(&XC_TABLE).insert(&xc_key, Arc::clone(&new_xc_entry));
        new_xc_entry
    };

    if ftn_add_data_int.next_hop.is_ipv4() {
        nhlfe = NhlfeTableGen::V4(&NHLFE_TABLE4).lookup(&nhlfe_k);
    } else {
        nhlfe = NhlfeTableGen::V6(&NHLFE_TABLE6).lookup(&nhlfe_k);
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
            let xc_entry_o = XcTableGen::XC(&XC_TABLE).lookup(&xc_key);
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
                NhlfeTableGen::V4(&NHLFE_TABLE4).insert(nhlfe_k, nhlfe_entry);
            } else {
                NhlfeTableGen::V6(&NHLFE_TABLE6).insert(nhlfe_k, nhlfe_entry);
            }
        }
    }
    let ftn_entry: FtnEntryWrapped = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
        FtnEntry::new(ftn_add_data_int.fec, ftn_add_data_int.ftn_ix),
    ))));
    write_val!(ftn_entry).add_xc_entry(xc_entry);
    match ftn_add_data_int.fec {
        IpAddr::V4(_) => {
            FtnTableGen::V4(&FTN_TABLE4)
                .insert(FtnKey::IP(FtnKeyIp::new(ftn_add_data_int.fec)), ftn_entry);
        }
        IpAddr::V6(_) => {
            FtnTableGen::V6(&FTN_TABLE6)
                .insert(FtnKey::IP(FtnKeyIp::new(ftn_add_data_int.fec)), ftn_entry);
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn ftn_add(ftn_add_data: *mut FtnAddData) -> i32 {
    let ftn_add_int: FtnAddDataInt;
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

    let addr_ptr: *mut u8 = (*ftn_del_data).fec.addr;
    if (*ftn_del_data).fec.family == 1 {
        fec = copy_ip_addr_v4_from_user(addr_ptr);
    } else {
        fec = copy_ip_addr_v6_from_user(addr_ptr as *mut u16);
    }
    ftn_del_int = FtnDelDataInt::new(fec, (*ftn_del_data).ftn_ix);
    Ok(ftn_del_int)
}

fn _ftn_del(ftn_del_data_int: &FtnDelDataInt) -> i32 {
    match ftn_del_data_int.fec {
        IpAddr::V4(_) => {
            match FtnTableGen::V4(&FTN_TABLE4).lookup(
                &FtnKey::IP(FtnKeyIp::new(ftn_del_data_int.fec)),
                ftn_del_data_int.ftn_ix,
            ) {
                Some(e) => {
                    write_val!(e).free_xc_list();
                    FtnTableGen::V4(&FTN_TABLE4).remove(
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
        IpAddr::V6(_) => {
            match FtnTableGen::V6(&FTN_TABLE6).lookup(
                &FtnKey::IP(FtnKeyIp::new(ftn_del_data_int.fec)),
                ftn_del_data_int.ftn_ix,
            ) {
                Some(e) => {
                    write_val!(e).free_xc_list();
                    FtnTableGen::V6(&FTN_TABLE6).remove(
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
    let ftn_del_int: FtnDelDataInt;
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

unsafe fn convert_ilm_add_to_internal(ilm_add_data: *mut IlmAddData) -> Result<IlmAddDataInt, i32> {
    let ilm_add_int: IlmAddDataInt;
    let next_hop: IpAddr;

    let addr_ptr: *mut u8 = (*ilm_add_data).next_hop.addr;
    if (*ilm_add_data).next_hop.family == 1 {
        next_hop = copy_ip_addr_v4_from_user(addr_ptr);
    } else {
        next_hop = copy_ip_addr_v6_from_user(addr_ptr as *mut u16);
    }
    ilm_add_int = IlmAddDataInt::new(
        (*ilm_add_data).in_label,
        (*ilm_add_data).in_iface,
        next_hop,
        (*ilm_add_data).out_ifindex,
        (*ilm_add_data).out_label,
        (*ilm_add_data).ilm_ix,
        (*ilm_add_data).owner,
    );
    Ok(ilm_add_int)
}

unsafe fn convert_ilm_del_to_internal(ilm_del_data: *mut IlmDelData) -> Result<IlmDelDataInt, i32> {
    let ilm_del_int: IlmDelDataInt;

    ilm_del_int = IlmDelDataInt::new(
        (*ilm_del_data).in_label,
        (*ilm_del_data).in_iface,
        (*ilm_del_data).ilm_ix,
        (*ilm_del_data).owner,
    );
    Ok(ilm_del_int)
}

pub struct IlmAddDataInt {
    pub in_label: u32,
    pub in_iface: u32,
    pub next_hop: IpAddr,
    pub out_ifindex: u32,
    pub out_label: u32,
    pub ilm_idx: u32,
    pub owner: u32,
}

impl IlmAddDataInt {
    fn new(
        in_label: u32,
        in_iface: u32,
        next_hop: IpAddr,
        out_ifindex: u32,
        out_label: u32,
        ilm_idx: u32,
        owner: u32,
    ) -> IlmAddDataInt {
        IlmAddDataInt {
            in_label: in_label,
            in_iface: in_iface,
            next_hop: next_hop,
            out_ifindex: out_ifindex,
            out_label: out_label,
            ilm_idx: ilm_idx,
            owner: owner,
        }
    }
}

pub struct IlmDelDataInt {
    pub in_label: u32,
    pub in_iface: u32,
    pub ilm_idx: u32,
    pub owner: u32,
}

impl IlmDelDataInt {
    fn new(in_label: u32, in_iface: u32, ilm_idx: u32, owner: u32) -> IlmDelDataInt {
        IlmDelDataInt {
            in_label: in_label,
            in_iface: in_iface,
            ilm_idx: ilm_idx,
            owner: owner,
        }
    }
}

fn _ilm_add(ilm_add_int: &IlmAddDataInt) -> i32 {
    let ilm_key = IlmKey::PKT(IlmKeyPkt::new(ilm_add_int.in_label, ilm_add_int.in_iface));
    match IlmTableGen::ILM(&ILM_TABLE).lookup_by_owner(&ilm_key, ilm_add_int.owner) {
        Some(_) => {
            trace!("ILM entry already exists");
            return -1;
        }
        None => {
            trace!("creating ILM entry");
        }
    }
    let ilm_entry: IlmEntryWrapped = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
        IlmEntry::new(ilm_key, ilm_add_int.ilm_idx, ilm_add_int.owner),
    ))));
    IlmTableGen::ILM(&ILM_TABLE).insert(ilm_key, ilm_entry);
    0
}

#[no_mangle]
pub extern "C" fn ilm_add(ilm_add_data: *mut IlmAddData) -> i32 {
    let ilm_add_int: IlmAddDataInt;
    trace!("ilm_del");
    unsafe {
        match convert_ilm_add_to_internal(ilm_add_data) {
            Ok(ret_val) => {
                ilm_add_int = ret_val;
            }
            Err(_) => {
                trace!("cannot convert ilm_add to internal");
                return -1;
            }
        }
    }
    _ilm_add(&ilm_add_int)
}

fn _ilm_del(ilm_del_int: &IlmDelDataInt) -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn ilm_del(ilm_del_data: *mut IlmDelData) -> i32 {
    let ilm_del_int: IlmDelDataInt;
    trace!("ilm_del");
    unsafe {
        match convert_ilm_del_to_internal(ilm_del_data) {
            Ok(ret_val) => {
                ilm_del_int = ret_val;
            }
            Err(_) => {
                trace!("cannot convert ilm_del to internal");
                return -1;
            }
        }
    }
    _ilm_del(&ilm_del_int)
}
