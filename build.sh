cargo build
MYLIB=$(find . -name libcrusttest.a)
ln -s $MYLIB
make -C c_rust_interop_test
