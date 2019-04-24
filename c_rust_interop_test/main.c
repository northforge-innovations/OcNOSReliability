#include <stdio.h>

int c_rust_peer_entry_test();
int c_rust_peer_entry_test2();
int c_rust_peer_route_entry_test1();
int c_rust_peer_route_entry_test2();
int c_rust_peer_route_entry_test3();

int main(int argc, char**argv)
{
	if (c_rust_peer_entry_test() != 0) {
		printf("peer_entry_test is failed!\n");
	}
	if (c_rust_peer_route_entry_test1() != 0) {
		printf("peer_route_entry_test1 is failed!\n");
	}
	if (c_rust_peer_route_entry_test2() != 0) {
		printf("peer_route_entry_test2 is failed!\n");
	}
	if (c_rust_peer_entry_test2() != 0) {
		printf("peer_entry_test2 is failed!\n");
	}
	if (c_rust_peer_route_entry_test3() != 0) {
		printf("peer_route_entry_test3 is failed!\n");
	}
	return 0;
}
