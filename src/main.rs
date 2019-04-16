#[cfg(test)]
mod tests {
extern crate data_storage_api;

extern {
    fn c_rust_route_entry_test() -> i32;
    fn c_rust_peer_entry_test() -> i32;
    fn c_rust_peer_route_entry_test() -> i32;
}

#[test]
fn route_entry_test() {
    unsafe {
        let rc = c_rust_route_entry_test();
	assert_eq!(rc, 0);
    }
}

#[test]
fn peer_entry_test() {
    unsafe {
        let rc = c_rust_peer_entry_test();
	assert_eq!(rc, 0);
    }
}
#[test]
fn peer_route_entry_test() {
    unsafe {
        let rc = c_rust_peer_route_entry_test();
	assert_eq!(rc, 0);
    }
}
}
