#[macro_use]
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
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
}
impl FecEntry {
    fn new(ftn_list: FtnList) -> FecEntry {
        FecEntry { ftn_list: ftn_list }
    }
}
type FecEntryWrapped = Arc<ReentrantMutex<RefCell<Box<FecEntry>>>>;
pub struct NhEvent {
    nh_addr: IpAddr,
    state: bool,
}
pub struct NhTableBase {
    tree: PatriciaMap<NhEntryWrapped>,
    events: VecDeque<NhEvent>,
}
pub struct NhEntry {
    connected: bool,
    physical: bool,
    dependent_ftn_list: FtnList,
    dependent_ilm_list: IlmList,
}
impl NhEntry {
    fn new(connected: bool, physical: bool) -> NhEntry {
        NhEntry {
            connected: connected,
            physical: physical,
            dependent_ftn_list: Vec::new(),
            dependent_ilm_list: Vec::new(),
        }
    }
}
type FtnTable = Arc<ReentrantMutex<RefCell<PatriciaMap<FecEntryWrapped>>>>;
type IlmTable = Arc<ReentrantMutex<RefCell<HashMap<IlmKey, IlmList>>>>;
type XcTable = Arc<ReentrantMutex<RefCell<HashMap<XcKey, XcEntryWrapped>>>>;
type NhlfeEntryWrapped = Arc<ReentrantMutex<RefCell<Box<NhlfeEntry>>>>;
type NhEntryWrapped = Arc<ReentrantMutex<RefCell<Box<NhEntry>>>>;
type NhlfeTable = Arc<ReentrantMutex<RefCell<HashMap<NhlfeKey, NhlfeEntryWrapped>>>>;
type IdTable = Arc<ReentrantMutex<RefCell<Box<IdMap>>>>;
type NhTable = Arc<ReentrantMutex<RefCell<NhTableBase>>>;

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
    pub static ref NH_TABLE4: NhTable = Arc::new(ReentrantMutex::new(RefCell::new(NhTableBase {
        tree: PatriciaMap::new(),
        events: VecDeque::new()
    })));
    pub static ref NH_TABLE6: NhTable = Arc::new(ReentrantMutex::new(RefCell::new(NhTableBase {
        tree: PatriciaMap::new(),
        events: VecDeque::new()
    })));
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
            i += 1;
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
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $table:ident, $entry:ident) => {
        match $key {
            $IpVerGood(ipv) => {
                write_val!(&$table).tree.insert(ipv.octets(), $entry);
            }
            $IpVerBad(_) => {
                trace!("wrong IPversion");
            }
        }
    };
}

macro_rules! nhtable_insert {
    ($self:ident, $key:ident, $entry:ident) => {
        match $self {
            NhTableGen::V4(_) => {
                nhtable_insert_version!(IpAddr::V4, IpAddr::V6, $key, NH_TABLE4, $entry)
            }
            NhTableGen::V6(_) => {
                nhtable_insert_version!(IpAddr::V6, IpAddr::V4, $key, NH_TABLE6, $entry)
            }
        }
    };
}

macro_rules! nhtable_lookup_version {
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $table:ident) => {
        match $key {
            $IpVerGood(ipv) => {
                if read_val!(&$table).tree.contains_key(ipv.octets()) {
                    return Ok(Arc::clone(
                        read_val!($table).tree.get(ipv.octets()).unwrap(),
                    ));
                }
            }
            $IpVerBad(_) => {
                trace!("wrong IPversion");
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
                write_val!(&$table).tree.remove(ipv.octets());
            }
            $IpVerBad(_) => {
                trace!("wrong IPversion");
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

macro_rules! bring_entry_up {
    ($entry:expr, $nh_entry:expr) => {
        let succeeded = read_val!($entry).try_up($nh_entry);
        if succeeded {
            {
                write_val!($entry).up();
            }
            read_val!($entry).attach_to_parent();
        } else {
            {
                write_val!($entry).down();
            }
            //read_val!($entry).detach_from_parent();
        }
    };
}

macro_rules! process_depdendents {
    ($nh_entry:expr) => {
        trace!("process_dependents");
        let try_ftn_up = |e: &FtnEntryWrapped| {
            trace!("bringing up FTN with FEC {}", read_val!(e).fec);
            bring_entry_up!(e, $nh_entry);
            false
        };
        let try_ilm_up = |e: &IlmEntryWrapped| {
            bring_entry_up!(e, $nh_entry);
            false
        };
        iterate_list(&read_val!($nh_entry).dependent_ftn_list, &try_ftn_up);
        iterate_list(&read_val!($nh_entry).dependent_ilm_list, &try_ilm_up);
    };
}

macro_rules! add_entry_to_list {
    ($self:ident, $key:ident, $entry:expr, $type:ty, $ix:ident, $list:ident) => {
        trace!("add_entry_to_list");
        let mut exists: bool = false;
        let mut check_if_entry_exists = |e: &$type| {
            if read_val!(e).$ix == read_val!($entry).$ix {
                exists = true;
                return true;
            }
            false
        };
        match $self.lookup($key) {
            Ok(nh_entry) => {
                iterate_list_mut(&read_val!(nh_entry).$list, &mut check_if_entry_exists);
                if exists {
                    return;
                }
                insert_list(&mut write_val!(nh_entry).$list, $entry);
            }
            Err(_) => {}
        }
    };
}

impl NhTableGen {
    fn create_modify_entry(
        &self,
        key: IpAddr,
        exists: bool,
        physical: bool,
    ) -> Option<NhEntryWrapped> {
        trace!("NhTableGen::insert {} {}", key, exists);
        let mut entry_to_return: Option<NhEntryWrapped> = None;
        match NhTableGen::lookup(self, key) {
            Ok(existing_entry) => {
                write_val!(existing_entry).connected = exists;
                write_val!(existing_entry).physical = physical;
            }
            Err(_) => {
                let nh_entry: NhEntryWrapped = Arc::new(ReentrantMutex::new(RefCell::new(
                    Box::new(NhEntry::new(exists, physical)),
                )));
                entry_to_return = Some(Arc::clone(&nh_entry));
                nhtable_insert!(self, key, nh_entry);
            }
        }
        match NhTableGen::lookup(self, key) {
            Ok(existing_entry) => {
                entry_to_return = Some(Arc::clone(&existing_entry));
                process_depdendents!(&existing_entry);
            }
            Err(_) => {
                trace!("cannot find nh entry, should not get here!");
                return None;
            }
        }
        entry_to_return
    }
    fn lookup(&self, key: IpAddr) -> Result<NhEntryWrapped, i32> {
        nhtable_lookup!(self, key);
        Err(-1)
    }
    fn add_ftn_entry_to_list(&mut self, key: IpAddr, entry: FtnEntryWrapped) {
        add_entry_to_list!(self, key, entry, FtnEntryWrapped, ix, dependent_ftn_list);
    }
    fn add_ilm_entry_to_list(&mut self, key: IpAddr, entry: IlmEntryWrapped) {
        add_entry_to_list!(self, key, entry, IlmEntryWrapped, ix, dependent_ilm_list);
    }
    fn remove(&self, key: IpAddr) {
        nhtable_remove!(self, key);
    }
    fn enqueue(&mut self, nh_addr: IpAddr, state: bool) {
        match self {
            NhTableGen::V4(_) => write_val!(NH_TABLE4).events.push_back(NhEvent {
                nh_addr: nh_addr,
                state: state,
            }),
            NhTableGen::V6(_) => write_val!(NH_TABLE6).events.push_back(NhEvent {
                nh_addr: nh_addr,
                state: state,
            }),
        }
    }
    fn dequeue(&mut self) -> Option<NhEvent> {
        match self {
            NhTableGen::V4(_) => write_val!(NH_TABLE4).events.pop_front(),
            NhTableGen::V6(_) => write_val!(NH_TABLE6).events.pop_front(),
        }
    }
    fn process_events(&mut self) {
        loop {
            match self.dequeue() {
                Some(ev) => {
                    self.create_modify_entry(ev.nh_addr, ev.state, false);
                }
                None => {
                    return;
                }
            }
        }
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
fn iterate_list<E>(list: &Vec<E>, on_element: &Fn(&E) -> bool) {
    trace!("iterate_list");
    let mut i = 0;
    let len = list.len();
    while i < len {
        if on_element(&list[i]) {
            break;
        }
        i += 1;
    }
}

fn iterate_list_mut<E>(list: &Vec<E>, on_element: &mut FnMut(&E) -> bool) {
    trace!("iterate_list");
    let mut i = 0;
    let len = list.len();
    while i < len {
        if on_element(&list[i]) {
            break;
        }
        i += 1;
    }
}

fn iterate_ftn_list(
    list: &Vec<FtnEntryWrapped>,
    on_element: &Fn(&FtnEntryWrapped) -> bool,
) -> Option<FtnEntryWrapped> {
    trace!("iterate_list");
    let mut i = 0;
    let len = list.len();
    while i < len {
        if on_element(&list[i]) {
            return Some(Arc::clone(&list[i]));
        }
        i += 1;
    }
    return None;
}

macro_rules! ftn_table_gen_lookup_version {
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $ix:ident, $table:ident) => {
        match $key {
            FtnKey::IP(key_ip) => match key_ip.prefix {
                $IpVerGood(ipv) => {
                    if read_val!($table).contains_key(ipv.octets()) {
                        return iterate_ftn_list(
                            &read_val!(read_val!($table).get(ipv.octets()).unwrap()).ftn_list,
                            &|ie| $ix == read_val!(ie).ix,
                        );
                    }
                }
                $IpVerBad(_) => {
                    trace!("wrong IPversion");
                }
            },
        }
    };
}

macro_rules! ftn_table_gen_lookup {
    ($self:ident, $key:ident, $ix:ident) => {
        match $self {
            FtnTableGen::V4(_) => {
                ftn_table_gen_lookup_version!(IpAddr::V4, IpAddr::V6, $key, $ix, FTN_TABLE4);
            }
            FtnTableGen::V6(_) => {
                ftn_table_gen_lookup_version!(IpAddr::V4, IpAddr::V6, $key, $ix, FTN_TABLE4);
            }
        }
    };
}

macro_rules! ftn_table_gen_lookup_fec_version {
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $table:ident) => {
        match $key {
            FtnKey::IP(key_ip) => match key_ip.prefix {
                $IpVerGood(ipv) => {
                    if read_val!($table).contains_key(ipv.octets()) {
                        return iterate_ftn_list(
                            &read_val!(read_val!($table).get(ipv.octets()).unwrap()).ftn_list,
                            &|ie| read_val!(ie).is_up(),
                        );
                    }
                }
                $IpVerBad(_) => {
                    trace!("wrong IPversion");
                }
            },
        }
    };
}

macro_rules! ftn_table_gen_lookup_fec {
    ($self:ident, $key:ident) => {
        match $self {
            FtnTableGen::V4(_) => {
                ftn_table_gen_lookup_fec_version!(IpAddr::V4, IpAddr::V6, $key, FTN_TABLE4);
            }
            FtnTableGen::V6(_) => {
                ftn_table_gen_lookup_fec_version!(IpAddr::V4, IpAddr::V6, $key, FTN_TABLE4);
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
                            Arc::new(ReentrantMutex::new(RefCell::new(Box::new(FecEntry::new(
                                Vec::new(),
                            ))))),
                        );
                    }
                    insert_list(
                        &mut write_val!(write_val!($table).get_mut(ipv.octets()).unwrap()).ftn_list,
                        $entry,
                    );
                }
                $IpVerBad(_) => {
                    trace!("wrong IPversion");
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
    ($IpVerGood:path, $IpVerBad:path, $key:ident, $ix:ident, $table:ident) => {
        match $key {
            FtnKey::IP(key_ip) => match key_ip.prefix {
                $IpVerGood(ipv) => {
                    let removed_ftn = FtnTableGen::remove_from_list(
                        &mut write_val!(write_val!($table).get_mut(ipv.octets()).unwrap()).ftn_list,
                        $ix,
                    );
                    if removed_ftn.is_some() {
                        trace!("removed FTN, cleaning up dependent FTNs/ILMs");
                        let removed_ftn_unwarp = Arc::clone(&removed_ftn.unwrap());
                        write_val!(removed_ftn_unwarp).down();
                    }
                    if list_is_empty(
                        &read_val!(read_val!($table).get(ipv.octets()).unwrap()).ftn_list,
                    ) {
                        trace!("no FTNs, removing FEC entry");
                        write_val!($table).remove(ipv.octets());
                    }
                }
                $IpVerBad(_) => {
                    trace!("wrong IPversion");
                }
            },
        }
    };
}

macro_rules! ftn_table_gen_remove {
    ($self:ident, $key:ident, $ix:ident) => {
        match $self {
            FtnTableGen::V4(_) => {
                ftn_table_gen_remove_version!(IpAddr::V4, IpAddr::V6, $key, $ix, FTN_TABLE4);
            }
            FtnTableGen::V6(_) => {
                ftn_table_gen_remove_version!(IpAddr::V4, IpAddr::V6, $key, $ix, FTN_TABLE4);
            }
        }
    };
}

fn get_nhlfe_key(list: &XcList) -> Option<NhlfeKey> {
    if list.len() == 0 {
        trace!("xc_list is empty");
        return None;
    }
    if read_val!(list[0]).nhlfe.is_none() {
        trace!("no NHLFE for first XC");
        return None;
    }
    let xc_entry = Arc::clone(&list[0]);
    let nhlfe_entry = Arc::clone(&read_val!(xc_entry).nhlfe.as_ref().unwrap());
    let nhlfe_key = read_val!(nhlfe_entry).nhlfe_key;
    Some(nhlfe_key)
}

macro_rules! link_entry_to_nh_bring_up_and_dependent_version2 {
    ($table:ident, $ver:ident, $nh:ident, $entry:ident, $nh_entry:ident, $func:ident) => {
        NhTableGen::$ver(&$table).$func($nh.next_hop, Arc::clone(&$entry));
        bring_entry_up!(Arc::clone(&$entry), &$nh_entry);
        NhTableGen::$ver(&$table).process_events();
    };
}

macro_rules! link_entry_to_nh_bring_up_and_dependent_version1 {
    ($table:ident, $ver:ident, $nh:ident, $entry:ident, $func:ident) => {
        match NhTableGen::$ver(&$table).lookup($nh.next_hop) {
            Ok(nh_entry) => {
                trace!("found NH");
                link_entry_to_nh_bring_up_and_dependent_version2!(
                    $table, $ver, $nh, $entry, nh_entry, $func
                );
            }
            Err(_) => {
                trace!("NH is not found");
                let nh_entry =
                    NhTableGen::$ver(&$table).create_modify_entry($nh.next_hop, false, false);
                if nh_entry.is_some() {
                    let nh_entry_unwrapped = nh_entry.unwrap();
                    link_entry_to_nh_bring_up_and_dependent_version2!(
                        $table,
                        $ver,
                        $nh,
                        $entry,
                        nh_entry_unwrapped,
                        $func
                    );
                }
            }
        }
    };
}

macro_rules! link_entry_to_nh_bring_up_and_dependent {
    ($nh:ident, $entry:ident, $func:ident) => {
        match $nh.next_hop {
            IpAddr::V4(_) => {
                link_entry_to_nh_bring_up_and_dependent_version1!(
                    NH_TABLE4, V4, $nh, $entry, $func
                );
            }
            IpAddr::V6(_) => {
                link_entry_to_nh_bring_up_and_dependent_version1!(
                    NH_TABLE6, V6, $nh, $entry, $func
                );
            }
        }
    };
}

impl FtnTableGen {
    fn remove_from_list(ftn_list: &mut FtnList, ix: u32) -> Option<FtnEntryWrapped> {
        trace!("FtnTableGen::remove_from_list");
        let len: usize = ftn_list.len();
        let mut i = 0;
        while i < len {
            if read_val!(ftn_list[i]).ix == ix {
                let removed_ftn = Arc::clone(&ftn_list[i]);
                ftn_list.remove(i);
                return Some(removed_ftn);
            }
            i += 1;
        }
        None
    }
    fn lookup_by_ix(&self, key: &FtnKey, ix: u32) -> Option<FtnEntryWrapped> {
        trace!("FtnTableGen::lookup");
        ftn_table_gen_lookup!(self, key, ix);
        None
    }
    fn lookup_by_ftn_fec(&self, key: &FtnKey) -> Option<FtnEntryWrapped> {
        trace!("FtnTableGen::lookup");
        ftn_table_gen_lookup_fec!(self, key);
        None
    }
    fn insert(&self, key: FtnKey, entry: FtnEntryWrapped) {
        trace!("FtnTableGen::insert");
        let entry_clone = Arc::clone(&entry);
        let entry_clone2 = Arc::clone(&entry_clone);
        let entry_clone3 = Arc::clone(&entry_clone2);
        ftn_table_gen_insert!(self, key, entry);
        let nhlfe_k: Option<NhlfeKey> = get_nhlfe_key(&*read_val!(entry_clone3).get_xc_list());

        match nhlfe_k {
            Some(nhlfe_key) => match nhlfe_key {
                NhlfeKey::IP(nhlfe_key_ip) => {
                    link_entry_to_nh_bring_up_and_dependent!(
                        nhlfe_key_ip,
                        entry_clone,
                        add_ftn_entry_to_list
                    );
                }
            },
            None => {
                trace!("Nhlfe is not found");
            }
        }
    }
    fn remove(&self, key: &FtnKey, ix: u32) {
        trace!("FtnTableGen::remove");
        ftn_table_gen_remove!(self, key, ix);
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
    fn get_xc_list(&self) -> &XcList;
    fn get_xc_list_mut(&mut self) -> &mut XcList;
    fn attach_to_parent(&self);
    fn detach_from_parent(&self);
    fn up(&mut self);
    fn down(&mut self);
    fn is_up(&self) -> bool;
    fn add_xc_entry(&mut self, entry: XcEntryWrapped) {
        trace!("add_xc_entry");
        self.get_xc_list_mut().push(entry);
    }
    fn free_xc_list(&mut self) {
        trace!("freeing xc_list");
        let xc_list = self.get_xc_list_mut();
        loop {
            match xc_list.pop() {
                Some(xc_entry) => {
                    write_val!(xc_entry).cleanup();
                    XcTableGen::XC(&XC_TABLE).remove(&read_val!(xc_entry).xc_key);
                }
                None => {
                    return;
                }
            }
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
    fn try_up(&self, nh_entry: &NhEntryWrapped) -> bool {
        let is_up: bool;
        let xc_list = self.get_xc_list();
        trace!("MplsEntry::try_up");
        if xc_list.len() == 0 {
            trace!("xc_list is empty");
            return false;
        }
        if read_val!(xc_list[0]).nhlfe.is_none() {
            trace!("no NHLFE for first XC");
            return false;
        }
        match &read_val!(xc_list[0]).nhlfe {
            Some(nhlfe_entry) => match &read_val!(nhlfe_entry).nhlfe_key {
                NhlfeKey::IP(nhlfe_key_ip) => match nhlfe_key_ip.next_hop {
                    IpAddr::V4(_) => {
                        trace!("found NH");
                        if read_val!(nh_entry).connected {
                            trace!("NH is connected");
                            is_up = true;
                        } else {
                            trace!("NH is not connected");
                            is_up = false;
                        }
                    }
                    IpAddr::V6(_) => {
                        trace!("found NH");
                        if read_val!(nh_entry).connected {
                            trace!("NH is connected");
                            is_up = true;
                        } else {
                            trace!("NH is not connected");
                            is_up = false;
                        }
                    }
                },
            },
            None => {
                is_up = false;
            }
        }
        return is_up;
    }
}

pub struct FtnEntry {
    fec: IpAddr,
    ix: u32,
    xc_list: XcList,
    dependent_ftn_up_list: FtnList,
    dependent_ilm_up_list: IlmList,
    state: bool,
}

macro_rules! clear_entry_list {
    ($self:ident, $up_list:ident) => {
        loop {
            match $self.$up_list.pop() {
                Some(dep) => {
                    trace!("clear_entry_list: found dependent");
                    write_val!(dep).down();
                }
                None => {
                    break;
                }
            }
        }
    };
}

impl FtnEntry {
    fn new(fec: IpAddr, idx: u32) -> FtnEntry {
        FtnEntry {
            fec: fec,
            ix: idx,
            xc_list: Vec::new(),
            dependent_ftn_up_list: Vec::new(),
            dependent_ilm_up_list: Vec::new(),
            state: false,
        }
    }
    fn add_to_ftn_up_list(&mut self, dep_ftn: FtnEntryWrapped) {
        trace!("FtnEntry::add_to_ftn_up_list");
        //write_val!(dep_ftn).up();
        self.dependent_ftn_up_list.push(dep_ftn);
    }
    fn add_to_ilm_up_list(&mut self, dep_ilm: IlmEntryWrapped) {
        trace!("FtnEntry::add_to_ilm_up_list");
        //write_val!(dep_ilm).up();
        self.dependent_ilm_up_list.push(dep_ilm);
    }
    fn clean_ftn_up_list(&mut self) {
        trace!("FtnEntry::clean_ftn_up_list");
        clear_entry_list!(self, dependent_ftn_up_list);
    }
    fn clean_ilm_up_list(&mut self) {
        trace!("FtnEntry::clean_ilm_up_list");
        clear_entry_list!(self, dependent_ilm_up_list);
    }
}

macro_rules! attach_to_parent2 {
    ($ver:ident, $nhlfe_key_ip:ident, $dep:ident, $table:ident, $func:ident, $rc:ident) => {
        trace!("attach_to_parent2 {}", $nhlfe_key_ip.next_hop);
        match FtnTableGen::$ver(&$table)
            .lookup_by_ftn_fec(&FtnKey::IP(FtnKeyIp::new($nhlfe_key_ip.next_hop)))
        {
            Some(parent_entry) => {
                {
                    trace!("found FEC {}", read_val!(parent_entry).fec);
                }
                {
                    write_val!(parent_entry).$func(Arc::clone(&$dep));
                    $rc = true;
                }
            }
            None => {
                trace!("FEC {} is not found", $nhlfe_key_ip.next_hop);
            }
        }
    };
}

macro_rules! attach_to_parent1 {
    ($self:ident, $ver:ident, $ver_other:ident, $entry_table:expr, $key:expr, $ftn_table:ident, $ftn_table_other:ident, $nh_table:ident, $func:ident, $rc:ident) => {
        trace!("attach_to_parent1");
        match $entry_table.lookup_by_ix($key, $self.ix) {
            Some(dep) => match get_nhlfe_key(&$self.get_xc_list()) {
                Some(nhlfe_key) => match nhlfe_key {
                    NhlfeKey::IP(nhlfe_key_ip) => match nhlfe_key_ip.next_hop {
                        IpAddr::$ver(_) => {
                            attach_to_parent2!($ver, nhlfe_key_ip, dep, $ftn_table, $func, $rc);
                        }
                        IpAddr::$ver_other(_) => {
                            attach_to_parent2!(
                                $ver_other,
                                nhlfe_key_ip,
                                dep,
                                $ftn_table_other,
                                $func,
                                $rc
                            );
                        }
                    },
                },
                None => {
                    trace!("Nhlfe is not found");
                }
            },
            None => {
                trace!("cannot find entry with id {}", $self.ix);
            }
        }
        //NhTableGen::$ver(&$nh_table).create_modify_entry($self.fec, true, false);
    };
}

impl MplsEntry for FtnEntry {
    fn get_xc_list(&self) -> &XcList {
        &self.xc_list
    }
    fn get_xc_list_mut(&mut self) -> &mut XcList {
        &mut self.xc_list
    }
    fn is_up(&self) -> bool {
        self.state
    }
    fn attach_to_parent(&self) {
        trace!("attach_to_parent");
        let mut is_attached: bool = false;
        match self.fec {
            IpAddr::V4(_) => {
                attach_to_parent1!(
                    self,
                    V4,
                    V6,
                    FtnTableGen::V4(&FTN_TABLE4),
                    &FtnKey::IP(FtnKeyIp::new(self.fec)),
                    FTN_TABLE4,
                    FTN_TABLE6,
                    NH_TABLE4,
                    add_to_ftn_up_list,
                    is_attached
                );
                if !is_attached {
                    match get_nhlfe_key(&self.get_xc_list()) {
                        Some(nhlfe_key) => match nhlfe_key {
                            NhlfeKey::IP(nhlfe_key_ip) => {
                                match NhTableGen::V4(&NH_TABLE4).lookup(nhlfe_key_ip.next_hop) {
                                    Ok(nh) => {
                                        is_attached = read_val!(nh).connected;
                                    }
                                    Err(_) => {}
                                }
                            }
                        },
                        None => {
                            trace!("cannot retrieve NH");
                        }
                    }
                }
                if is_attached {
                    NhTableGen::V4(&NH_TABLE4).enqueue(self.fec, true);
                }
            }
            IpAddr::V6(_) => {
                attach_to_parent1!(
                    self,
                    V6,
                    V4,
                    FtnTableGen::V6(&FTN_TABLE6),
                    &FtnKey::IP(FtnKeyIp::new(self.fec)),
                    FTN_TABLE6,
                    FTN_TABLE4,
                    NH_TABLE6,
                    add_to_ftn_up_list,
                    is_attached
                );
                if !is_attached {
                    match get_nhlfe_key(&self.get_xc_list()) {
                        Some(nhlfe_key) => match nhlfe_key {
                            NhlfeKey::IP(nhlfe_key_ip) => {
                                match NhTableGen::V4(&NH_TABLE4).lookup(nhlfe_key_ip.next_hop) {
                                    Ok(nh) => {
                                        is_attached = read_val!(nh).connected;
                                    }
                                    Err(_) => {}
                                }
                            }
                        },
                        None => {
                            trace!("cannot retrieve NH");
                        }
                    }
                }
                if is_attached {
                    NhTableGen::V6(&NH_TABLE6).enqueue(self.fec, true);
                }
            }
        }
    }
    fn detach_from_parent(&self) {
        trace!("detach_from_parent");
        match self.fec {
            IpAddr::V4(_) => {
                NhTableGen::V4(&NH_TABLE4).enqueue(self.fec, false);
            }
            IpAddr::V6(_) => {
                NhTableGen::V6(&NH_TABLE6).enqueue(self.fec, false);
            }
        }
    }
    fn up(&mut self) {
        trace!("FtnEntry::up {}", self.fec);
        self.state = true;
    }
    fn down(&mut self) {
        trace!("FtnEntry::down {}", self.fec);
        self.state = false;
        self.clean_ftn_up_list();
        self.clean_ilm_up_list();
        self.detach_from_parent();
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct NhlfeKeyIp {
    next_hop: IpAddr,
    out_label: u32,
    out_iface: u32,
    trunk_id: u16,
    lsp_id: u16,
    ingress: IpAddr,
    egress: IpAddr,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
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
        self.set_nhlfe(None);
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
    ix: u32,
    xc_list: XcList,
    owner: u32,
    state: bool,
}

impl IlmEntry {
    fn new(ilm_key: IlmKey, ix: u32, owner: u32) -> IlmEntry {
        IlmEntry {
            ilm_key: ilm_key,
            ix: ix,
            xc_list: Vec::new(),
            owner: owner,
            state: false,
        }
    }
}

impl MplsEntry for IlmEntry {
    fn get_xc_list(&self) -> &XcList {
        &self.xc_list
    }
    fn get_xc_list_mut(&mut self) -> &mut XcList {
        &mut self.xc_list
    }
    fn is_up(&self) -> bool {
        self.state
    }
    fn attach_to_parent(&self) {
        trace!("attach_to_parent");
        let mut _is_attached: bool = false;
        match &self.ilm_key {
            IlmKey::PKT(pkt_key) => {
                attach_to_parent1!(
                    self,
                    V4,
                    V6,
                    IlmTableGen::ILM(&ILM_TABLE),
                    &IlmKey::PKT(IlmKeyPkt::new(pkt_key.in_label, pkt_key.in_iface)),
                    FTN_TABLE4,
                    FTN_TABLE6,
                    NH_TABLE4,
                    add_to_ilm_up_list,
                    _is_attached
                );
            }
        }
    }
    fn detach_from_parent(&self) {
        trace!("not implemented");
    }
    fn up(&mut self) {
        trace!("IlmEntry::up {}", self.ix);
        self.state = true;
    }
    fn down(&mut self) {
        trace!("IlmEntry::down {}", self.ix);
        self.state = false;
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
    fn lookup_by_ix(&self, ilm_key: &IlmKey, ix: u32) -> Option<IlmEntryWrapped> {
        if read_val!(&ILM_TABLE).contains_key(ilm_key) {
            return self.lookup_list(&read_val!(&ILM_TABLE)[ilm_key], &|ie| {
                ix == read_val!(ie).ix
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
                insert_list(ll, Arc::clone(&ilm_entry));
            }
            None => {
                trace!("expected to find linked list on key {:?}", ilm_key);
            }
        }
        let nhlfe_k: Option<NhlfeKey> = get_nhlfe_key(&*read_val!(ilm_entry).get_xc_list());

        match nhlfe_k {
            Some(nhlfe_key) => match nhlfe_key {
                NhlfeKey::IP(nhlfe_key_ip) => {
                    link_entry_to_nh_bring_up_and_dependent!(
                        nhlfe_key_ip,
                        ilm_entry,
                        add_ilm_entry_to_list
                    );
                }
            },
            None => {
                trace!("Nhlfe is not found");
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
            match FtnTableGen::V4(&FTN_TABLE4).lookup_by_ix(
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
            match FtnTableGen::V6(&FTN_TABLE6).lookup_by_ix(
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
        Some(_) => {
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
                NhTableGen::V4(&NH_TABLE4).create_modify_entry(addr, true, true);
            } else {
                NhTableGen::V4(&NH_TABLE4).create_modify_entry(addr, false, true);
            }
            NhTableGen::V4(&NH_TABLE4).process_events();
        }
        IpAddr::V6(_) => {
            if is_add {
                NhTableGen::V6(&NH_TABLE6).create_modify_entry(addr, true, true);
            } else {
                NhTableGen::V6(&NH_TABLE6).create_modify_entry(addr, false, true);
            }
            NhTableGen::V6(&NH_TABLE6).process_events();
        }
    }
    0
}
