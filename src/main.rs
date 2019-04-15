use data_storage_api::*;
extern {
    fn c_rust_test();
}

fn main() {
    unsafe {
        c_rust_test();
    }
}
