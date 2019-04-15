
all: c_rust_interop_test.o main.o
	gcc -o c_rust_interop_test c_rust_interop_test.c main.c -ldata_storage_api -lm -L"../target/debug"
