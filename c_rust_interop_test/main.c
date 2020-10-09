#include <stdio.h>

int c_rust_peer_entry_test();
int c_rust_peer_entry_test2();
int c_rust_peer_route_entry_test1();
int c_rust_peer_route_entry_test2();
int c_rust_peer_route_entry_test3();
int c_rust_peer_route_entry_iteration_test1();
int c_rust_prefix_tree_test1();
int c_rust_ftn_test1();
int c_rust_ilm_test1();

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
	if (c_rust_peer_route_entry_iteration_test1() != 0) {
		printf("peer_route_entry_iteration_test1 is failed!\n");
	}
	if (c_rust_prefix_tree_test1() != 0) {
		printf("prefix_tree_test1 is failed!\n");
	}
	if (c_rust_ftn_test1() != 0) {
		printf("ftn test1 is failed!\n");
	}
	if (c_rust_ilm_test1() != 0) {
		printf("ilm test1 is failed!\n");
	}
	return 0;
}
