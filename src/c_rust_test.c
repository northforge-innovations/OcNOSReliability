#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include "c_rust_test.h"

static void build_ip_addr(char *ip_addr_str, int increment, unsigned int *prefix)
{
	struct in_addr in;
	inet_aton(ip_addr_str, &in);
	*prefix = htonl(ntohl(in.s_addr) + increment);
}

static void setup_ip_addr(IpAddrC *addr, unsigned int *prefix)
{
	addr->family = 1;
	addr->addr = (uint8_t *)prefix;
}

static void setup_peer_entry(PeerEntry *peer_entry, unsigned int *prefix, unsigned int out_ifindex)
{
	peer_entry->prefix.family = 1;
	peer_entry->prefix.addr = (uint8_t *)prefix;
	peer_entry->out_ifindex = out_ifindex;
}

static void setup_route_entry(RouteEntry *route_entry, unsigned int *prefix, unsigned int *mask, unsigned int *next_hop, unsigned int out_ifindex)
{
	route_entry->prefix.family = 1;
	route_entry->prefix.addr = (uint8_t *)prefix;
	route_entry->mask.family = 1;
	route_entry->mask.addr = (uint8_t *)mask;
	route_entry->next_hop.family = 1;
	route_entry->next_hop.addr = (uint8_t *)next_hop;
	route_entry->out_ifindex = out_ifindex;
}

#define PEER_ENTRY_TEST_INITIAL_PEER "1.1.1.1"
#define PEER_ENTRY_TEST_PEER_NUMBER 1000
#define PEER_ENTRY_TEST_INITIAL_IFINDEX 3

int c_rust_peer_entry_test()
{
	int rc = 0;
	PeerEntry entry;
	IpAddrC ip_addr;
	unsigned int current_peer_prefix;
	unsigned int dummy = 0;

	init_logger();

	for (int i = 0; i < PEER_ENTRY_TEST_PEER_NUMBER; i++) {
		build_ip_addr(PEER_ENTRY_TEST_INITIAL_PEER,i, &current_peer_prefix);
		setup_peer_entry(&entry, &current_peer_prefix, PEER_ENTRY_TEST_INITIAL_IFINDEX + 1);
		rc = peer_add(&entry.prefix, &entry);
		if (rc != 0)
			return rc;
		setup_peer_entry(&entry, &dummy, 0);
		setup_ip_addr(&ip_addr, &current_peer_prefix);
		rc = peer_lookup(&ip_addr,&entry);
		if (rc != 0)
			return rc;
		rc = peer_delete(&ip_addr);
		if (rc != 0)
			return rc;
	}
	for (int i = 0; i < PEER_ENTRY_TEST_PEER_NUMBER; i++) {
		build_ip_addr(PEER_ENTRY_TEST_INITIAL_PEER,i, &current_peer_prefix);
		setup_ip_addr(&ip_addr, &current_peer_prefix);
		setup_peer_entry(&entry, &current_peer_prefix, PEER_ENTRY_TEST_INITIAL_IFINDEX + 1);
		rc = peer_add(&ip_addr, &entry);
		if (rc != 0)
			return rc;
		setup_peer_entry(&entry, &dummy, 0);
		rc = peer_lookup(&ip_addr,&entry);
		if (rc != 0)
			return rc;
	}
	for (int i = 0; i < PEER_ENTRY_TEST_PEER_NUMBER; i++) {
		build_ip_addr(PEER_ENTRY_TEST_INITIAL_PEER,i, &current_peer_prefix);
		setup_ip_addr(&ip_addr, &current_peer_prefix);
		setup_peer_entry(&entry, &dummy, 0);
		rc = peer_lookup(&ip_addr,&entry);
		if (rc != 0)
			return rc;
		rc = peer_delete(&ip_addr);
		if (rc != 0)
			return rc;
	}
	return rc;
}

#define PEER_ROUTE_ENTRY_TEST1_INITIAL_PEER "10.0.0.1"
#define PEER_ROUTE_ENTRY_TEST1_PEER_NUMBER 1000
#define PEER_ROUTE_ENTRY_TEST1_INITIAL_IFINDEX 3000
#define PEER_ROUTE_ENTRY_TEST1_INITIAL_ROUTE "10.10.0.1"
#define PEER_ROUTE_ENTRY_TEST1_ROUTE_NUMBER 1000

int c_rust_peer_route_entry_test1()
{
	int rc = 0;
	PeerEntry peer_entry;
	IpAddrC ip_addr;
	unsigned int current_peer_prefix;
	unsigned int current_route_prefix;
	unsigned int dummy = 0;

	init_logger();

	for (int i = 0; i < PEER_ROUTE_ENTRY_TEST1_PEER_NUMBER; i++ ) {
		build_ip_addr(PEER_ROUTE_ENTRY_TEST1_INITIAL_PEER,i, &current_peer_prefix);
		setup_ip_addr(&ip_addr, &current_peer_prefix);
		setup_peer_entry(&peer_entry, &current_peer_prefix, PEER_ROUTE_ENTRY_TEST1_INITIAL_IFINDEX + i);
		rc = peer_add(&ip_addr, &peer_entry);
		if (rc != 0)
			return rc;
		setup_peer_entry(&peer_entry, &dummy, 0);
		rc = peer_lookup(&ip_addr,&peer_entry);
		if (rc != 0)
			return rc;
		RouteEntry route_entry;
		for (int j = 0; j < PEER_ROUTE_ENTRY_TEST1_ROUTE_NUMBER; j++) {
			unsigned int mask = 0xFFFFFF00;
			unsigned int next_hop = 10;
			build_ip_addr(PEER_ROUTE_ENTRY_TEST1_INITIAL_ROUTE,j, &current_route_prefix);
			setup_route_entry(&route_entry, &current_route_prefix, &mask, &next_hop, PEER_ROUTE_ENTRY_TEST1_INITIAL_IFINDEX + i);
			rc = peer_route_add(&peer_entry.prefix,&route_entry);
			if (rc != 0)
				return rc;
			next_hop = 0;
			setup_route_entry(&route_entry, &current_route_prefix, &mask, &next_hop, 0);
			rc = peer_route_lookup(&peer_entry.prefix,&route_entry.prefix, &route_entry);
			if (rc != 0)
				return rc;
		}
	}
	for (int i = 0; i < PEER_ROUTE_ENTRY_TEST1_PEER_NUMBER; i++ ) {
		build_ip_addr(PEER_ROUTE_ENTRY_TEST1_INITIAL_PEER,i, &current_peer_prefix);
		setup_ip_addr(&ip_addr, &current_peer_prefix);
		for (int j = 0; j < PEER_ROUTE_ENTRY_TEST1_ROUTE_NUMBER; j++) {
			unsigned int mask = 0xFFFFFF00;
			unsigned int next_hop = 10;
			RouteEntry route_entry;
			build_ip_addr(PEER_ROUTE_ENTRY_TEST1_INITIAL_ROUTE,j, &current_route_prefix);
			setup_route_entry(&route_entry, &current_route_prefix, &mask, &next_hop, 0);
			rc = peer_route_delete(&ip_addr,&route_entry.prefix);
			if (rc != 0)
				return rc;
		}
		rc = peer_delete(&ip_addr);
		if (rc != 0)
			return rc;
	}
	return rc;
}

#define PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER "2.0.0.1"
#define PEER_ROUTE_ENTRY_TEST2_PEER_NUMBER 2
#define PEER_ROUTE_ENTRY_TEST2_INITIAL_IFINDEX 3000
#define PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE "20.0.0.1"
#define PEER_ROUTE_ENTRY_TEST2_ROUTE_NUMBER 1

int c_rust_peer_route_entry_test2()
{
	int rc = 0;
	PeerEntry peer_entry;
	IpAddrC ip_addr;
	unsigned int dummy = 0;
	unsigned int current_peer_prefix;

	init_logger();

	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER,0, &current_peer_prefix);
	setup_ip_addr(&ip_addr, &current_peer_prefix);
	setup_peer_entry(&peer_entry, &current_peer_prefix, 3);
	rc = peer_add(&ip_addr, &peer_entry);
	if (rc != 0)
		return rc;
	setup_peer_entry(&peer_entry, &dummy, 0);

	rc = peer_lookup(&ip_addr,&peer_entry);
	if (rc != 0)
		return rc;
	RouteEntry route_entry;
	unsigned int current_route_prefix;
	unsigned int mask = 0xFFFFFF00;
	unsigned int next_hop = 10;
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE,0, &current_route_prefix);
	setup_route_entry(&route_entry, &current_route_prefix, &mask, &next_hop, PEER_ROUTE_ENTRY_TEST2_INITIAL_IFINDEX);
	
	rc = peer_route_add(&ip_addr,&route_entry);
	if (rc != 0)
		return rc;
	rc = peer_route_lookup(&ip_addr,&route_entry.prefix, &route_entry);
	if (rc != 0)
		return rc;
	// adding one more peer
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER,1, &current_peer_prefix);
	setup_ip_addr(&ip_addr, &current_peer_prefix);
	setup_peer_entry(&peer_entry, &current_peer_prefix, PEER_ROUTE_ENTRY_TEST2_INITIAL_IFINDEX);
	rc = peer_add(&ip_addr, &peer_entry);
	if (rc != 0)
		return rc;
	rc = peer_route_add(&ip_addr,&route_entry);
	if (rc != 0)
		return rc;
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER,1, &current_peer_prefix);
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE,0, &current_route_prefix);
	rc = peer_route_lookup(&peer_entry.prefix,&route_entry.prefix, &route_entry);
	if (rc != 0)
		return rc;
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER,0, &current_peer_prefix);
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE,0, &current_route_prefix);
	rc = peer_route_delete(&ip_addr,&route_entry.prefix);
	if (rc != 0)
		return rc;
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER,0, &current_peer_prefix);
	rc = peer_delete(&ip_addr);
	if (rc != 0)
		return rc;
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE,0, &current_route_prefix);
	rc = route_lookup(&route_entry.prefix, &route_entry);
	if (rc != 0)
		return rc;
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER,1, &current_peer_prefix);
	rc = peer_delete(&ip_addr);
	if (rc != 0)
		return rc;
	build_ip_addr(PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE,0, &current_route_prefix);
	rc = route_lookup(&route_entry.prefix, &route_entry);
	if (rc == 0)
		return -1;
	return 0;
}

#define PEER_ENTRY_TEST2_INITIAL_PEER "30.0.0.1"
#define PEER_ENTRY_TEST2_PEER_NUMBER 1
#define PEER_ENTRY_TEST2_INITIAL_IFINDEX1 3
#define PEER_ENTRY_TEST2_INITIAL_IFINDEX2 5

int c_rust_peer_entry_test2()
{
	int rc = 0;
	PeerEntry peer_entry;
	IpAddrC ip_addr;
	unsigned int current_peer_prefix;
	unsigned int dummy = 0;

	init_logger();

	for (int i = 0; i < PEER_ENTRY_TEST2_PEER_NUMBER; i++) {
		build_ip_addr(PEER_ENTRY_TEST2_INITIAL_PEER,i, &current_peer_prefix);
		setup_ip_addr(&ip_addr, &current_peer_prefix);
		setup_peer_entry(&peer_entry, &current_peer_prefix, PEER_ENTRY_TEST2_INITIAL_IFINDEX1);
		rc = peer_add_modify(&ip_addr, &peer_entry);
		if (rc != 0)
			return rc;

		rc = peer_lookup(&ip_addr,&peer_entry);
		if (rc != 0)
			return rc;
		if (peer_entry.out_ifindex != PEER_ENTRY_TEST2_INITIAL_IFINDEX1)
			return -1;
	}
	for (int i = 0; i < PEER_ENTRY_TEST2_PEER_NUMBER; i++) {
		build_ip_addr(PEER_ENTRY_TEST2_INITIAL_PEER,i, &current_peer_prefix);
		setup_ip_addr(&ip_addr, &current_peer_prefix);
		setup_peer_entry(&peer_entry, &current_peer_prefix, PEER_ENTRY_TEST2_INITIAL_IFINDEX2);
		rc = peer_add_modify(&ip_addr, &peer_entry);
		if (rc != 1)
			return -1;

		rc = peer_lookup(&ip_addr,&peer_entry);
		if (rc != 0)
			return rc;
		if (peer_entry.out_ifindex != PEER_ENTRY_TEST2_INITIAL_IFINDEX2)
			return -1;
	}
	for (int i = 0; i < PEER_ENTRY_TEST2_PEER_NUMBER; i++) {
		build_ip_addr(PEER_ENTRY_TEST2_INITIAL_PEER,i, &current_peer_prefix);
		setup_ip_addr(&ip_addr, &current_peer_prefix);
		rc = peer_lookup(&ip_addr,&peer_entry);
		if (rc != 0)
			return rc;
		rc = peer_delete(&ip_addr);
		if (rc != 0)
			return rc;
	}
	return rc;
}

#define PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER "40.0.0.1"
#define PEER_ROUTE_ENTRY_TEST3_PEER_NUMBER 2
#define PEER_ROUTE_ENTRY_TEST3_INITIAL_IFINDEX 3000
#define PEER_ROUTE_ENTRY_TEST3_MODIFIED_IFINDEX 5000
#define PEER_ROUTE_ENTRY_TEST3_INITIAL_NEXT_HOP 10
#define PEER_ROUTE_ENTRY_TEST3_MODIFIED_NEXT_HOP 30
#define PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE "200.0.0.1"
#define PEER_ROUTE_ENTRY_TEST3_ROUTE_NUMBER 1

int c_rust_peer_route_entry_test3()
{
	int rc = 0;
	PeerEntry peer_entry;
	IpAddrC ip_addr;
	unsigned int current_peer_prefix;

	init_logger();

	build_ip_addr(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,0, &current_peer_prefix);
	setup_ip_addr(&ip_addr, &current_peer_prefix);
	setup_peer_entry(&peer_entry, &current_peer_prefix, 3);
	printf("adding first peer\n");
	rc = peer_add(&ip_addr, &peer_entry);
	if (rc != 0)
		return rc;
	printf("done. lookup...\n");
	
	rc = peer_lookup(&ip_addr,&peer_entry);
	if (rc != 0)
		return rc;
	printf("done\n");
	RouteEntry route_entry;
	unsigned int current_route_prefix;
	unsigned int mask = 0xFFFFFF00;
	unsigned int next_hop = PEER_ROUTE_ENTRY_TEST3_INITIAL_NEXT_HOP;
	build_ip_addr(PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE,0, &current_route_prefix);
	setup_route_entry(&route_entry, &current_route_prefix, &mask, &next_hop, PEER_ROUTE_ENTRY_TEST3_INITIAL_IFINDEX);
	printf("adding route to first peer\n");
	rc = peer_route_add_modify(&ip_addr,&route_entry);
	if (rc != 0)
		return rc;

	rc = peer_route_lookup(&ip_addr,&route_entry.prefix, &route_entry);
	if (rc != 0)
		return rc;
	// adding one more peer
	build_ip_addr(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,1, &current_peer_prefix);
	printf("adding second peer\n");
	rc = peer_add(&ip_addr, &peer_entry);
	if (rc != 0)
		return rc;
	printf("adding route to second peer\n");
	rc = peer_route_add_modify(&ip_addr,&route_entry);
	if (rc != 0)
		return rc;

	rc = peer_route_lookup(&ip_addr,&route_entry.prefix, &route_entry);
	if (rc != 0)
		return rc;
	//now modify
	next_hop = PEER_ROUTE_ENTRY_TEST3_MODIFIED_NEXT_HOP;
	setup_route_entry(&route_entry, &current_route_prefix, &mask, &next_hop, PEER_ROUTE_ENTRY_TEST3_MODIFIED_IFINDEX);
	printf("modifying route\n");
	build_ip_addr(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,0, &current_peer_prefix);
	rc = peer_route_add_modify(&ip_addr,&route_entry);
	if (rc != 1)
		return rc;
	build_ip_addr(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,1, &current_peer_prefix);
	rc = peer_route_lookup(&ip_addr,&route_entry.prefix, &route_entry);
	if (rc != 0)
		return rc;
	next_hop = *((unsigned int *)route_entry.next_hop.addr);
	if ((route_entry.out_ifindex != PEER_ROUTE_ENTRY_TEST3_MODIFIED_IFINDEX) ||
	    (next_hop != PEER_ROUTE_ENTRY_TEST3_MODIFIED_NEXT_HOP)) {
		printf("either next_hop %d or out_ifindex %d is not reflected\n",next_hop,route_entry.out_ifindex);
		return -1;
	}
	printf("deleting peer1 route\n");
	build_ip_addr(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,0, &current_peer_prefix);
	rc = peer_route_delete(&ip_addr,&route_entry.prefix);
	if (rc != 0)
		return rc;
	printf("deleting peer1\n");
	rc = peer_delete(&ip_addr);
	if (rc != 0)
		return rc;

	printf("looking up for route\n");
	rc = route_lookup(&route_entry.prefix, &route_entry);
	if (rc != 0)
		return 0;
	printf("deleting peer2\n");
	build_ip_addr(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,1, &current_peer_prefix);
	rc = peer_delete(&ip_addr);
	if (rc != 0)
		return rc;

	printf("looking up route\n");
	rc = route_lookup(&route_entry.prefix, &route_entry);
	if (rc == 0)
		return -1;
	return 0;
}

