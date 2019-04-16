extern void c_rust_test();

int main(int argc, char**argv)
{
	c_rust_route_entry_test();
	c_rust_peer_entry_test();
	c_rust_peer_route_entry_test();
	return 0;
}
