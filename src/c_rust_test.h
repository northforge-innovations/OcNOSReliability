#ifndef _C_RUST_TEST_H_
#define _C_RUST_TEST_H_
#include<stdint.h>

struct route_entry {
	uint32_t prefix;
	uint32_t next_hop;
	uint32_t out_ifindex;
};

extern int route_add(uint32_t _prefix, struct route_entry *entry);
extern int route_lookup(uint32_t _prefix, struct route_entry *entry);
extern int route_delete(uint32_t _prefix);

#endif
