#[cfg(test)]
mod tests {
    extern crate data_storage_api;

    extern "C" {
        fn c_rust_peer_entry_test() -> i32;
        fn c_rust_peer_route_entry_test1() -> i32;
        fn c_rust_peer_route_entry_test2() -> i32;
    }

    #[test]
    fn peer_entry_test() {
        unsafe {
            let rc = c_rust_peer_entry_test();
            assert_eq!(rc, 0);
        }
    }
    #[test]
    fn peer_route_entry_test1() {
        unsafe {
            let rc = c_rust_peer_route_entry_test1();
            assert_eq!(rc, 0);
        }
    }
    #[test]
    fn peer_route_entry_test2() {
        unsafe {
            let rc = c_rust_peer_route_entry_test2();
            assert_eq!(rc, 0);
        }
    }
}

extern crate data_storage_api;

extern "C" {
    fn c_rust_peer_entry_test() -> i32;
    fn c_rust_peer_route_entry_test1() -> i32;
    fn c_rust_peer_route_entry_test2() -> i32;
}

fn main() {
    unsafe {
        c_rust_peer_entry_test();
        c_rust_peer_route_entry_test1();
        c_rust_peer_route_entry_test2();
    }
}
