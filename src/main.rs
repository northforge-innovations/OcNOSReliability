#[cfg(test)]
mod tests {
extern crate data_storage_api;

extern {
    fn c_rust_test() -> i32;
}

#[test]
fn main() {
    unsafe {
        let rc = c_rust_test();
	assert_eq!(rc, 0);
    }
}
}
