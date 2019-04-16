cargo build --lib
cargo build --bin poc
cargo test
MYLIB=$(find . -name libcrusttest.a)
rm libcrusttest.a
ln -s $MYLIB
make -C c_rust_interop_test
