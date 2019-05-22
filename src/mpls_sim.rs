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
pub struct FecEntry {
    ftn_list: FtnList,
    dependent_ftn_down_list: FtnList,
    dependent_ilm_down_list: IlmList,
}
type FtnTable = Arc<ReentrantMutex<RefCell<PatriciaMap<FecEntry>>>>;
type IlmTable = Arc<ReentrantMutex<RefCell<HashMap<IlmKey, IlmList>>>>;
type XcTable = Arc<ReentrantMutex<RefCell<HashMap<XcKey, XcEntryWrapped>>>>;
type NhlfeEntryWrapped = Arc<ReentrantMutex<RefCell<Box<NhlfeEntry>>>>;
type NhlfeTable = Arc<ReentrantMutex<RefCell<HashMap<NhlfeKey, NhlfeEntryWrapped>>>>;
type IdTable = Arc<ReentrantMutex<RefCell<Box<IdMap>>>>;
type NhTable = Arc<ReentrantMutex<RefCell<PatriciaMap<u32>>>>;

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
    pub static ref ILM_ID_TABLE: IdTable =
        Arc::new(ReentrantMutex::new(RefCell::new(Box::new(IdMap {
            ids: [false; 1024]
        }))));
    pub static ref NH_TABLE4: NhTable =
        Arc::new(ReentrantMutex::new(RefCell::new(PatriciaMap::new())));
    pub static ref NH_TABLE6: NhTable =
        Arc::new(ReentrantMutex::new(RefCell::new(PatriciaMap::new())));
}

pub struct IdMap {
    ids: [bool; 1024],
}

impl IdMap {
    fn get_free(&mut self) -> u32 {
        let mut i = 0;
        while i < 1024 {
            if !self.ids[i] {
                self.ids[i] = true;
                return (i + 1) as u32;
            }
            i = i + 1;
        }
        0
    }
    fn put_free(&mut self, ix: usize) {
        if ix > 1024 || ix == 0 {
            trace!("idx is out of range {}", ix);
            return;
        }
        self.ids[ix - 1] = false;
    }
}

pub enum NhTableGen {
    V4(&'static NH_TABLE4),
    V6(&'static NH_TABLE6),
}

macro_rules! nhtable_insert_version {
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $entry:ident, $table:ident) => {
        match $key {
            $IpVerGood(ipv) => {
                write_val!(&$table).insert(ipv.octets(), $entry);
            }
            $IpVerBad(_) => {
                trace!("IPvX is unexpected here");
            }
        }
    };
}

macro_rules! nhtable_insert {
    ($self:ident, $key:ident, $entry:ident) => {
        match $self {
            NhTableGen::V4(_) => {
                nhtable_insert_version!(IpAddr::V4, IpAddr::V6, $key, $entry, NH_TABLE4)
            }
            NhTableGen::V6(_) => {
                nhtable_insert_version!(IpAddr::V6, IpAddr::V4, $key, $entry, NH_TABLE6)
            }
        }
    };
}

macro_rules! nhtable_lookup_version {
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $table:ident) => {
        match $key {
            $IpVerGood(ipv) => {
                if read_val!(&$table).contains_key(ipv.octets()) {
                    return Ok(*read_val!(&$table).get(ipv.octets()).unwrap());
                }
            }
            $IpVerBad(_) => {
                trace!("IPVx is unexpected here");
            }
        }
    };
}

macro_rules! nhtable_lookup {
    ($self:ident, $key:ident) => {
        match $self {
            NhTableGen::V4(_) => nhtable_lookup_version!(IpAddr::V4, IpAddr::V6, $key, NH_TABLE4),
            NhTableGen::V6(_) => nhtable_lookup_version!(IpAddr::V6, IpAddr::V4, $key, NH_TABLE6),
        }
    };
}

macro_rules! nhtable_remove_version {
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $table:ident) => {
        match $key {
            $IpVerGood(ipv) => {
                write_val!(&$table).remove(ipv.octets());
            }
            $IpVerBad(_) => {
                trace!("IPVx is unexpected here");
            }
        }
    };
}

macro_rules! nhtable_remove {
    ($self:ident, $key:ident) => {
        match $self {
            NhTableGen::V4(_) => nhtable_remove_version!(IpAddr::V4, IpAddr::V6, $key, NH_TABLE4),
            NhTableGen::V6(_) => nhtable_remove_version!(IpAddr::V6, IpAddr::V4, $key, NH_TABLE6),
        }
    };
}

impl NhTableGen {
    fn insert(&self, key: IpAddr, entry: u32) {
        nhtable_insert!(self, key, entry);
    }
    fn lookup(&self, key: IpAddr) -> Result<u32, i32> {
        nhtable_lookup!(self, key);
        Err(-1)
    }
    fn remove(&self, key: IpAddr) {
        nhtable_remove!(self, key);
    }
}

pub enum FtnTableGen {
    V4(&'static FTN_TABLE4),
    V6(&'static FTN_TABLE6),
}

fn insert_list<E>(list: &mut Vec<E>, entry: E) {
    trace!("insert_list");
    list.push(entry);
}
fn list_is_empty<E>(list: &Vec<E>) -> bool {
    trace!("list_is_empty");
    list.len() == 0
}

macro_rules! process_dependent_entry_version {
    ($add_up_list:ident, $down_list:ident, $entry:ident, $ftn_table:ident, $ipv:ident) => {
        if read_val!($ftn_table).contains_key($ipv.octets()) {
            match FtnTableGen::lookup_list(
                &read_val!($ftn_table).get($ipv.octets()).unwrap().ftn_list,
                &|ie| true == read_val!(ie).state,
            ) {
                Some(parent_ftn) => {
                    write_val!(parent_ftn).$add_up_list(Arc::clone(&$entry));
                }
                None => {
                    insert_list(
                        &mut write_val!($ftn_table)
                            .get_mut($ipv.octets())
                            .unwrap()
                            .$down_list,
                        Arc::clone(&$entry),
                    );
                }
            }
        }
    };
}

macro_rules! process_dependent_entry {
    ($add_up_list:ident, $down_list:ident, $self:ident, $fec:ident, $entry:ident) => {
        match $self.$fec {
            IpAddr::V4(ipv) => {
                process_dependent_entry_version!($add_up_list, $down_list, $entry, FTN_TABLE4, ipv)
            }
            IpAddr::V6(ipv) => {
                process_dependent_entry_version!($add_up_list, $down_list, $entry, FTN_TABLE6, ipv)
            }
        }
    };
}

macro_rules! ftn_table_gen_lookup_version {
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $ftn_ix:ident, $table:ident) => {
        match $key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    $IpVerGood(ipv) => {
                        if read_val!($table).contains_key(ipv.octets()) {
                            return FtnTableGen::lookup_list(
                                &read_val!($table).get(ipv.octets()).unwrap().ftn_list,
                                &|ie| $ftn_ix == read_val!(ie).ftn_ix,
                            );
                        }
                    }
                    $IpVerBad(_) => {
                        trace!("IPvx is not expected here!");
                    }
                },
            }
    };
}

macro_rules! ftn_table_gen_lookup {
    ($self:ident, $key:ident, $ftn_ix:ident) => {
        match $self {
            FtnTableGen::V4(_) => {
                ftn_table_gen_lookup_version!(IpAddr::V4, IpAddr::V6, $key, $ftn_ix, FTN_TABLE4);
            }
            FtnTableGen::V6(_) => {
                ftn_table_gen_lookup_version!(IpAddr::V4, IpAddr::V6, $key, $ftn_ix, FTN_TABLE4);
            }
        }
    };
}

macro_rules! ftn_table_gen_insert_version {
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $entry:ident, $table:ident) => {
        match $key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    $IpVerGood(ipv) => {
                        if !read_val!($table).contains_key(ipv.octets()) {
                            trace!("not found, create new");
                            write_val!($table).insert(
                                ipv.octets(),
                                FecEntry {
                                    ftn_list: Vec::new(),
                                    dependent_ftn_down_list: Vec::new(),
                                    dependent_ilm_down_list: Vec::new(),
                                },
                            );
                        }
                        insert_list(
                            &mut write_val!($table)
                                .get_mut(ipv.octets())
                                .unwrap()
                                .ftn_list,
                            $entry,
                        );
                    }
                    $IpVerBad(_) => {
                        trace!("IPvx is not expected here!");
                    }
                },
            }
    };
}

macro_rules! ftn_table_gen_insert {
    ($self:ident, $key:ident, $entry:ident) => {
        match $self {
            FtnTableGen::V4(_) => {
                ftn_table_gen_insert_version!(IpAddr::V4, IpAddr::V6, $key, $entry, FTN_TABLE4);
            }
            FtnTableGen::V6(_) => {
                ftn_table_gen_insert_version!(IpAddr::V4, IpAddr::V6, $key, $entry, FTN_TABLE4);
            }
        }
    };
}

macro_rules! ftn_table_gen_remove_version {
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $ftn_ix:ident, $table:ident) => {
        match $key {
                FtnKey::IP(key_ip) => match key_ip.prefix {
                    $IpVerGood(ipv) => {
                        let removed_ftn = FtnTableGen::remove_from_list(
                            &mut write_val!($table)
                                .get_mut(ipv.octets())
                                .unwrap()
                                .ftn_list,
                            $ftn_ix,
                        );
                        if removed_ftn.is_some() {
                            let removed_ftn_unwarp = Arc::clone(&removed_ftn.unwrap());
                            write_val!(removed_ftn_unwarp).clean_ftn_up_list();
                            write_val!(removed_ftn_unwarp).clean_ilm_up_list();
                        }
                        if list_is_empty(
                            &read_val!($table).get(ipv.octets()).unwrap().ftn_list,
                        ) && list_is_empty(
                            &read_val!($table)
                                .get(ipv.octets())
                                .unwrap()
                                .dependent_ftn_down_list,
                        ) && list_is_empty(
                            &read_val!($table)
                                .get(ipv.octets())
                                .unwrap()
                                .dependent_ilm_down_list,
                        ) {
                            write_val!($table).remove(ipv.octets());
                        }
                    }
                    $IpVerBad(_) => {
                        trace!("IPvx is not expected here!");
                    }
                },
            }
    };
}

macro_rules! ftn_table_gen_remove {
    ($self:ident, $key:ident, $ftn_ix:ident) => {
        match $self {
            FtnTableGen::V4(_) => {
                ftn_table_gen_remove_version!(IpAddr::V4, IpAddr::V6, $key, $ftn_ix, FTN_TABLE4);
            }
            FtnTableGen::V6(_) => {
                ftn_table_gen_remove_version!(IpAddr::V4, IpAddr::V6, $key, $ftn_ix, FTN_TABLE4);
            }
        }
    };
}

impl FtnTableGen {
    fn lookup_list(
        ftn_list: &FtnList,
        on_element: &Fn(&FtnEntryWrapped) -> bool,
    ) -> Option<FtnEntryWrapped> {
        trace!("FtnTableGen::lookup_list");
        let len: usize = ftn_list.len();
        let mut i = 0;
        while i < len {
            if on_element(&ftn_list[i]) {
                return Some(Arc::clone(&ftn_list[i]));
            }
            i += 1;
        }
        None
    }
    fn remove_from_list(ftn_list: &mut FtnList, ftn_ix: u32) -> Option<FtnEntryWrapped> {
        trace!("FtnTableGen::remove_from_list");
        let len: usize = ftn_list.len();
        let mut i = 0;
        while i < len {
            if read_val!(ftn_list[i]).ftn_ix == ftn_ix {
                let removed_ftn = Arc::clone(&ftn_list[i]);
                ftn_list.remove(i);
                return Some(removed_ftn);
            }
            i += 1;
        }
        None
    }
    fn lookup(&self, key: &FtnKey, ftn_ix: u32) -> Option<FtnEntryWrapped> {
        trace!("FtnTableGen::lookup");
        ftn_table_gen_lookup!(self, key, ftn_ix);
        None
    }
    fn insert(&self, key: FtnKey, entry: FtnEntryWrapped) {
        trace!("FtnTableGen::insert");
        ftn_table_gen_insert!(self, key, entry);
    }
    fn remove(&self, key: &FtnKey, ftn_ix: u32) {
        trace!("FtnTableGen::remove");
        ftn_table_gen_remove!(self, key, ftn_ix);
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

trait MplsEntry {
    fn get_xc_list(&mut self) -> &mut XcList;
    fn add_xc_entry(&mut self, entry: XcEntryWrapped) {
        trace!("add_xc_entry");
        self.get_xc_list().push(entry);
    }
    fn free_xc_list(&mut self) {
        trace!("freeing xc_list");
        let xc_list = self.get_xc_list();
        let len: usize = xc_list.len();
        let mut i = 0;
        while i < len {
            XcTableGen::XC(&XC_TABLE).remove(&write_val!(xc_list[i]).xc_key);
            i += 1;
        }
    }
    fn iterate_xc_list(&mut self, on_xc: &Fn(&XcEntryWrapped) -> bool) -> Option<XcEntryWrapped> {
        trace!("iterating xc_list");
        let xc_list = self.get_xc_list();
        let len: usize = xc_list.len();
        let mut i = 0;
        while i < len {
            if !on_xc(&xc_list[i]) {
                return Some(Arc::clone(&xc_list[i]));
            }
            i += 1;
        }
        None
    }
}

pub struct FtnEntry {
    fec: IpAddr,
    ftn_ix: u32,
    xc_list: XcList,
    is_dependent: bool,
    dependent_ftn_up_list: FtnList,
    dependent_ilm_up_list: IlmList,
    state: bool,
}

macro_rules! clear_entry_list {
    ($self:ident, $up_list:ident, $down_list:ident, $add_up_list:ident) => {
        loop {
            match $self.$up_list.pop() {
                Some(dep_ftn) => {
                    write_val!(dep_ftn).down();
                    process_dependent_entry!(
                        $add_up_list,
                        $down_list,
                        $self,
                        fec,
                        dep_ftn
                    );
                }
                None => {
                    break;
                }
            }
        }
    }
}

impl FtnEntry {
    fn new(fec: IpAddr, idx: u32, dependent: bool) -> FtnEntry {
        FtnEntry {
            fec: fec,
            ftn_ix: idx,
            xc_list: Vec::new(),
            is_dependent: dependent,
            dependent_ftn_up_list: Vec::new(),
            dependent_ilm_up_list: Vec::new(),
            state: false,
        }
    }
    fn up(&mut self) {
        self.state = true;
    }
    fn down(&mut self) {
        self.state = false;
        self.clean_ftn_up_list();
        self.clean_ilm_up_list();
    }
    fn add_to_ftn_up_list(&mut self, dep_ftn: FtnEntryWrapped) {
        write_val!(dep_ftn).up();
        self.dependent_ftn_up_list.push(dep_ftn);
    }
    fn add_to_ilm_up_list(&mut self, dep_ilm: IlmEntryWrapped) {
        write_val!(dep_ilm).up();
        self.dependent_ilm_up_list.push(dep_ilm);
    }
    fn clean_ftn_up_list(&mut self) {
        clear_entry_list!(self, dependent_ftn_up_list, dependent_ftn_down_list, add_to_ftn_up_list);
    }
    fn clean_ilm_up_list(&mut self) {
        clear_entry_list!(self, dependent_ilm_up_list, dependent_ilm_down_list, add_to_ilm_up_list);
    }
}

impl MplsEntry for FtnEntry {
    fn get_xc_list(&mut self) -> &mut XcList {
        &mut self.xc_list
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

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
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
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum IlmKey {
    PKT(IlmKeyPkt),
}

pub struct IlmEntry {
    ilm_key: IlmKey,
    ilm_ix: u32,
    xc_list: XcList,
    owner: u32,
    state: bool,
}

impl IlmEntry {
    fn new(ilm_key: IlmKey, ilm_ix: u32, owner: u32) -> IlmEntry {
        IlmEntry {
            ilm_key: ilm_key,
            ilm_ix: ilm_ix,
            xc_list: Vec::new(),
            owner: owner,
            state: false,
        }
    }
    fn up(&mut self) {
        self.state = true;
    }
    fn down(&mut self) {
        self.state = false;
    }
}

impl MplsEntry for IlmEntry {
    fn get_xc_list(&mut self) -> &mut XcList {
        &mut self.xc_list
    }
}

pub enum IlmTableGen {
    ILM(&'static ILM_TABLE),
}

impl IlmTableGen {
    fn lookup_list(
        &self,
        ilm_list: &IlmList,
        compare: &Fn(&IlmEntryWrapped) -> bool,
    ) -> Option<IlmEntryWrapped> {
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
            return self.lookup_list(&read_val!(&ILM_TABLE)[ilm_key], &|ie| {
                owner == read_val!(ie).owner
            });
        }
        None
    }
    fn lookup_by_ix(&self, ilm_key: &IlmKey, ilm_ix: u32) -> Option<IlmEntryWrapped> {
        if read_val!(&ILM_TABLE).contains_key(ilm_key) {
            return self.lookup_list(&read_val!(&ILM_TABLE)[ilm_key], &|ie| {
                ilm_ix == read_val!(ie).ilm_ix
            });
        }
        None
    }
    fn insert(&mut self, ilm_key: IlmKey, ilm_entry: IlmEntryWrapped) {
        trace!("IlmTableGen::insert");
        if !read_val!(&ILM_TABLE).contains_key(&ilm_key) {
            trace!("creating new entry");
            let ilm_list: IlmList = Vec::new();
            write_val!(&ILM_TABLE).insert(ilm_key, ilm_list);
        }
        trace!("insetion to list");
        match write_val!(&ILM_TABLE).get_mut(&ilm_key) {
            Some(ll) => {
                insert_list(ll, ilm_entry);
            }
            None => {
                trace!("expected to find linked list on key {:?}", ilm_key);
            }
        }
    }
    fn remove(&mut self, ilm_key: &IlmKey) {
        write_val!(&ILM_TABLE).remove(ilm_key);
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

fn _create_nhlfe_and_xc(
    nhlfe_k: NhlfeKey,
    old_xc_ix: u32,
    old_nhlfe_ix: u32,
) -> Option<XcEntryWrapped> {
    let xc_entry: XcEntryWrapped;
    let nhlfe: Option<NhlfeEntryWrapped>;
    let create_xc_entry = |xc_key: XcKey| {
        let new_xc_entry: XcEntryWrapped = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
            XcEntry::new(&xc_key, None),
        ))));
        trace!("insert new XcEntry to XC_TABLE");
        XcTableGen::XC(&XC_TABLE).insert(&xc_key, Arc::clone(&new_xc_entry));
        new_xc_entry
    };

    match &nhlfe_k {
        NhlfeKey::IP(nhlfe_k_ip) => {
            if nhlfe_k_ip.next_hop.is_ipv4() {
                nhlfe = NhlfeTableGen::V4(&NHLFE_TABLE4).lookup(&nhlfe_k);
            } else {
                nhlfe = NhlfeTableGen::V6(&NHLFE_TABLE6).lookup(&nhlfe_k);
            }
        }
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
            let xc_ix;
            let nhlfe_ix;
            if old_xc_ix > 0 {
                xc_ix = old_xc_ix;
            } else {
                xc_ix = write_val!(XC_ID_TABLE).get_free();
            }
            if old_nhlfe_ix > 0 {
                nhlfe_ix = old_nhlfe_ix;
            } else {
                nhlfe_ix = write_val!(NHLFE_ID_TABLE).get_free();
            }
            if xc_ix == 0 {
                trace!("cannot allocate xc ix");
                write_val!(NHLFE_ID_TABLE).put_free(nhlfe_ix as usize);
                return None;
            }
            if nhlfe_ix == 0 {
                trace!("cannot allocate nhlfe_ix");
                write_val!(XC_ID_TABLE).put_free(xc_ix as usize);
                return None;
            }
            xc_key = XcKey {
                in_iface: 0,
                gen_label: GenLabel { label: 0 },
                xc_ix: xc_ix as u32,
                nhlfe_ix: nhlfe_ix as u32,
            };
            xc_entry = create_xc_entry(xc_key);
            let nhlfe_entry: NhlfeEntryWrapped;
            match &nhlfe_k {
                NhlfeKey::IP(nhlfe_k_ip) => {
                    nhlfe_entry = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
                        NhlfeEntry::new(
                            nhlfe_k_ip.next_hop,
                            nhlfe_k_ip.out_label,
                            nhlfe_k_ip.out_iface,
                            0,
                            0,
                            nhlfe_k_ip.ingress,
                            nhlfe_k_ip.egress,
                            xc_ix,
                            nhlfe_ix,
                        ),
                    ))));
                }
            }
            write_val!(xc_entry).set_nhlfe(Some(Arc::clone(&nhlfe_entry)));
            match &nhlfe_k {
                NhlfeKey::IP(nhlfe_k_ip) => {
                    if nhlfe_k_ip.next_hop.is_ipv4() {
                        NhlfeTableGen::V4(&NHLFE_TABLE4).insert(nhlfe_k, nhlfe_entry);
                    } else {
                        NhlfeTableGen::V6(&NHLFE_TABLE6).insert(nhlfe_k, nhlfe_entry);
                    }
                }
            }
        }
    }
    Some(xc_entry)
}

fn _ftn_add(ftn_add_data_int: &FtnAddDataInt) -> i32 {
    let xc_entry: XcEntryWrapped;
    let nhlfe_k: NhlfeKey = NhlfeKey::IP(NhlfeKeyIp {
        next_hop: ftn_add_data_int.next_hop,
        out_label: ftn_add_data_int.out_label,
        out_iface: ftn_add_data_int.out_ifindex,
        trunk_id: 0,
        lsp_id: 0,
        ingress: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        egress: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
    });

    match _create_nhlfe_and_xc(nhlfe_k, 0, 0) {
        Some(ret_xc_entry) => {
            xc_entry = ret_xc_entry;
        }
        None => {
            trace!("cannot create XC entry!");
            return -1;
        }
    }
    let mut is_dependent: bool = false;
    match ftn_add_data_int.next_hop {
        IpAddr::V4(_) => match NhTableGen::V4(&NH_TABLE4).lookup(ftn_add_data_int.next_hop) {
            Ok(_) => {}
            Err(_) => {
                is_dependent = true;
            }
        },
        IpAddr::V6(_) => match NhTableGen::V6(&NH_TABLE6).lookup(ftn_add_data_int.next_hop) {
            Ok(_) => {}
            Err(_) => {
                is_dependent = true;
            }
        },
    }
    trace!("FTN entry is dependent {}", is_dependent);
    let ftn_entry: FtnEntryWrapped = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
        FtnEntry::new(ftn_add_data_int.fec, ftn_add_data_int.ftn_ix, is_dependent),
    ))));
    if !is_dependent {
        write_val!(ftn_entry).up();
    }
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
    pub ilm_ix: u32,
    pub owner: u32,
}

impl IlmAddDataInt {
    fn new(
        in_label: u32,
        in_iface: u32,
        next_hop: IpAddr,
        out_ifindex: u32,
        out_label: u32,
        ilm_ix: u32,
        owner: u32,
    ) -> IlmAddDataInt {
        IlmAddDataInt {
            in_label: in_label,
            in_iface: in_iface,
            next_hop: next_hop,
            out_ifindex: out_ifindex,
            out_label: out_label,
            ilm_ix: ilm_ix,
            owner: owner,
        }
    }
}

pub struct IlmDelDataInt {
    pub in_label: u32,
    pub in_iface: u32,
    pub ilm_ix: u32,
    pub owner: u32,
}

impl IlmDelDataInt {
    fn new(in_label: u32, in_iface: u32, ilm_ix: u32, owner: u32) -> IlmDelDataInt {
        IlmDelDataInt {
            in_label: in_label,
            in_iface: in_iface,
            ilm_ix: ilm_ix,
            owner: owner,
        }
    }
}

fn _ilm_add(
    ilm_add_int: &mut IlmAddDataInt,
    ilm_key: IlmKey,
    old_xc_ix: u32,
    old_nhlfe_ix: u32,
) -> i32 {
    trace!("_ilm_add");
    let xc_entry: XcEntryWrapped;
    let nhlfe_k: NhlfeKey = NhlfeKey::IP(NhlfeKeyIp {
        next_hop: ilm_add_int.next_hop,
        out_label: ilm_add_int.out_label,
        out_iface: ilm_add_int.out_ifindex,
        trunk_id: 0,
        lsp_id: 0,
        ingress: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        egress: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
    });

    match _create_nhlfe_and_xc(nhlfe_k, old_xc_ix, old_nhlfe_ix) {
        Some(ret_xc_entry) => {
            xc_entry = ret_xc_entry;
        }
        None => {
            trace!("Cannot create XC entry!");
            return -1;
        }
    }
    let ilm_entry: IlmEntryWrapped = Arc::new(ReentrantMutex::new(RefCell::new(Box::new(
        IlmEntry::new(ilm_key, ilm_add_int.ilm_ix, ilm_add_int.owner),
    ))));
    write_val!(ilm_entry).add_xc_entry(xc_entry);
    IlmTableGen::ILM(&ILM_TABLE).insert(ilm_key, ilm_entry);
    0
}

fn _ilm_update(
    ilm_add_int: &mut IlmAddDataInt,
    existing_ilm: IlmEntryWrapped,
    ilm_key: IlmKey,
) -> i32 {
    trace!("_ilm_update");
    let xc_entry: Option<XcEntryWrapped> = write_val!(existing_ilm).iterate_xc_list(&|xc| {
        match &read_val!(xc).nhlfe {
            Some(nhlfe) => match &read_val!(nhlfe).nhlfe_key {
                NhlfeKey::IP(nhlfe_k_ip) => {
                    if nhlfe_k_ip.next_hop == ilm_add_int.next_hop {
                        trace!("next hops equal");
                        return false;
                    }
                }
            },
            None => {
                trace!("Expected to find nhlfe");
            }
        }
        true
    });
    match xc_entry {
        None => {
            _ilm_add(ilm_add_int, ilm_key, 0, 0);
        }
        Some(existing_xc_entry) => {
            trace!("updating ILM!");
        }
    }
    0
}

fn _ilm_add_update(ilm_add_int: &mut IlmAddDataInt) -> i32 {
    let ilm_key = IlmKey::PKT(IlmKeyPkt::new(ilm_add_int.in_label, ilm_add_int.in_iface));
    if ilm_add_int.ilm_ix > 0 {
        match IlmTableGen::ILM(&ILM_TABLE).lookup_by_ix(&ilm_key, ilm_add_int.ilm_ix) {
            Some(existing_ilm) => {
                trace!("ILM entry already exists");
                return _ilm_update(ilm_add_int, existing_ilm, ilm_key);
            }
            None => {
                trace!("creating ILM entry");
            }
        }
    } else {
        match write_val!(ILM_ID_TABLE).get_free() {
            0 => {
                trace!("Cannot allocate ILM IX");
                return -1;
            }
            allocated_ix => {
                ilm_add_int.ilm_ix = allocated_ix;
            }
        }
    }
    match IlmTableGen::ILM(&ILM_TABLE).lookup_by_owner(&ilm_key, ilm_add_int.owner) {
        Some(_) => {
            trace!("ILM entry already exists");
            return -1;
        }
        None => {
            trace!("creating ILM entry");
        }
    }
    _ilm_add(ilm_add_int, ilm_key, 0, 0)
}

#[no_mangle]
pub extern "C" fn ilm_add(ilm_add_data: *mut IlmAddData) -> i32 {
    let mut ilm_add_int: IlmAddDataInt;
    trace!("ilm_add");
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
    _ilm_add_update(&mut ilm_add_int)
}

fn _ilm_del(ilm_del_int: &IlmDelDataInt) -> i32 {
    let ilm_key = IlmKey::PKT(IlmKeyPkt::new(ilm_del_int.in_label, ilm_del_int.in_iface));
    let ilm_entry;
    if ilm_del_int.ilm_ix > 0 {
        ilm_entry = IlmTableGen::ILM(&ILM_TABLE).lookup_by_ix(&ilm_key, ilm_del_int.ilm_ix);
    } else {
        ilm_entry = IlmTableGen::ILM(&ILM_TABLE).lookup_by_owner(&ilm_key, ilm_del_int.owner);
    }
    match ilm_entry {
        None => {
            trace!("ILM entry is not found");
            return -1;
        }
        Some(existing_ilm) => {
            trace!("_ilm_del: freeing xc list");
            write_val!(existing_ilm).free_xc_list();
            trace!("_ilm_del: removing ilm entry");
            IlmTableGen::ILM(&ILM_TABLE).remove(&ilm_key);
        }
    }
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

#[no_mangle]
pub extern "C" fn nh_add_del(nh_add_del_data: *mut NhAddDel) -> i32 {
    let addr: IpAddr;
    let ifindex: u32;
    let is_add: bool;
    trace!("nh_add_del");
    unsafe {
        let mut addr_ptr: *mut u8 = (*nh_add_del_data).addr.addr;
        if (*nh_add_del_data).addr.family == 1 {
            addr = copy_ip_addr_v4_from_user(addr_ptr);
        } else {
            addr = copy_ip_addr_v6_from_user(addr_ptr as *mut u16);
        }
        ifindex = (*nh_add_del_data).ifindex;
        is_add = (*nh_add_del_data).is_add;
    }
    match addr {
        IpAddr::V4(_) => {
            if is_add {
                NhTableGen::V4(&NH_TABLE4).insert(addr, ifindex);
            } else {
                NhTableGen::V4(&NH_TABLE4).remove(addr);
            }
        }
        IpAddr::V6(_) => {
            if is_add {
                NhTableGen::V6(&NH_TABLE6).insert(addr, ifindex);
            } else {
                NhTableGen::V6(&NH_TABLE6).remove(addr);
            }
        }
    }
    0
}
