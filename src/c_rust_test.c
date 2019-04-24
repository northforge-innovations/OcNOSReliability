#include <stdio.h>
#include <string.h>
#include "c_rust_test.h"

#define PEER_ENTRY_TEST_INITIAL_PEER 1
#define PEER_ENTRY_TEST_PEER_NUMBER 1000
#define PEER_ENTRY_TEST_INITIAL_IFINDEX 3

int c_rust_peer_entry_test()
{
	int rc = 0;
	PeerEntry entry;

	for (int i = 0; i < PEER_ENTRY_TEST_PEER_NUMBER; i++) {
		unsigned int current_peer_prefix = PEER_ENTRY_TEST_INITIAL_PEER + i;
		entry.prefix = current_peer_prefix ;
		entry.out_ifindex =  PEER_ENTRY_TEST_INITIAL_IFINDEX + 1;
		rc = peer_add(current_peer_prefix, &entry);
		if (rc != 0)
			return rc;
		memset((char *)&entry,0,sizeof(entry));
		rc = peer_lookup(current_peer_prefix,&entry);
		if (rc != 0)
			return rc;
		rc = peer_delete(current_peer_prefix);
		if (rc != 0)
			return rc;
	}
	for (int i = 0; i < PEER_ENTRY_TEST_PEER_NUMBER; i++) {
		unsigned int current_peer_prefix = PEER_ENTRY_TEST_INITIAL_PEER + i;
		entry.prefix = current_peer_prefix;
		entry.out_ifindex =  PEER_ENTRY_TEST_INITIAL_IFINDEX + 1;
		rc = peer_add(current_peer_prefix, &entry);
		if (rc != 0)
			return rc;
		memset((char *)&entry,0,sizeof(entry));
		rc = peer_lookup(current_peer_prefix,&entry);
		if (rc != 0)
			return rc;
	}
	for (int i = 0; i < PEER_ENTRY_TEST_PEER_NUMBER; i++) {
		unsigned int current_peer_prefix = PEER_ENTRY_TEST_INITIAL_PEER + i;
		memset((char *)&entry,0,sizeof(entry));
		rc = peer_lookup(current_peer_prefix,&entry);
		if (rc != 0)
			return rc;
		rc = peer_delete(current_peer_prefix);
		if (rc != 0)
			return rc;
	}
	return rc;
}

#define PEER_ROUTE_ENTRY_TEST1_INITIAL_PEER 1001
#define PEER_ROUTE_ENTRY_TEST1_PEER_NUMBER 1000
#define PEER_ROUTE_ENTRY_TEST1_INITIAL_IFINDEX 3000
#define PEER_ROUTE_ENTRY_TEST1_INITIAL_ROUTE 10000
#define PEER_ROUTE_ENTRY_TEST1_ROUTE_NUMBER 1000

int c_rust_peer_route_entry_test1()
{
	int rc = 0;
	PeerEntry peer_entry;

	for (int i = 0; i < PEER_ROUTE_ENTRY_TEST1_INITIAL_PEER; i++ ) {
		unsigned int current_peer_prefix = PEER_ROUTE_ENTRY_TEST1_INITIAL_PEER + i;
		peer_entry.prefix = current_peer_prefix;
		peer_entry.out_ifindex = PEER_ROUTE_ENTRY_TEST1_INITIAL_IFINDEX + i;
		rc = peer_add(current_peer_prefix, &peer_entry);
		if (rc != 0)
			return rc;
		memset((char *)&peer_entry,0,sizeof(peer_entry));
		rc = peer_lookup(current_peer_prefix,&peer_entry);
		if (rc != 0)
			return rc;
		RouteEntry route_entry;
		for (int j = 0; j < PEER_ROUTE_ENTRY_TEST1_ROUTE_NUMBER; j++) {
			unsigned int current_route_prefix = PEER_ROUTE_ENTRY_TEST1_INITIAL_ROUTE + j;
			route_entry.prefix = current_route_prefix;
			route_entry.mask = 0xFFFFFF00;
			route_entry.next_hop = 10;
			route_entry.out_ifindex = PEER_ROUTE_ENTRY_TEST1_INITIAL_IFINDEX + i;
			rc = peer_route_add(peer_entry.prefix,&route_entry);
			if (rc != 0)
				return rc;
			memset((char *)&route_entry,0,sizeof(route_entry));
			rc = peer_route_lookup(peer_entry.prefix,current_route_prefix, &route_entry);
			if (rc != 0)
				return rc;
		}
	}
	for (int i = 0; i < PEER_ROUTE_ENTRY_TEST1_INITIAL_PEER; i++ ) {
		unsigned int current_peer_prefix = PEER_ROUTE_ENTRY_TEST1_INITIAL_PEER + i;
		for (int j = 0; j < PEER_ROUTE_ENTRY_TEST1_ROUTE_NUMBER; j++) {
			unsigned int current_route_prefix = PEER_ROUTE_ENTRY_TEST1_INITIAL_ROUTE + j;
			rc = peer_route_delete(current_peer_prefix,current_route_prefix);
			if (rc != 0)
				return rc;
		}
		rc = peer_delete(current_peer_prefix);
		if (rc != 0)
			return rc;
	}
	return rc;
}

#define PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER 2001
#define PEER_ROUTE_ENTRY_TEST2_PEER_NUMBER 2
#define PEER_ROUTE_ENTRY_TEST2_INITIAL_IFINDEX 3000
#define PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE 20000
#define PEER_ROUTE_ENTRY_TEST2_ROUTE_NUMBER 1

int c_rust_peer_route_entry_test2()
{
	int rc = 0;
	PeerEntry peer_entry;
	peer_entry.prefix = PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER;
	peer_entry.out_ifindex = 3;
	rc = peer_add(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER, &peer_entry);
	if (rc != 0)
		return rc;
	memset((char *)&peer_entry,0,sizeof(peer_entry));
	rc = peer_lookup(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER,&peer_entry);
	if (rc != 0)
		return rc;
	RouteEntry route_entry;
	route_entry.prefix = PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE;
	route_entry.mask = 0xFFFFFF00;
	route_entry.next_hop = 10;
	route_entry.out_ifindex = PEER_ROUTE_ENTRY_TEST2_INITIAL_IFINDEX;
	rc = peer_route_add(peer_entry.prefix,&route_entry);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = peer_route_lookup(peer_entry.prefix,PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE, &route_entry);
	if (rc != 0)
		return rc;
	// adding one more peer
	peer_entry.prefix = PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER + 1;
	peer_entry.out_ifindex = PEER_ROUTE_ENTRY_TEST2_INITIAL_IFINDEX;
	rc = peer_add(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER + 1, &peer_entry);
	if (rc != 0)
		return rc;
	
	rc = peer_route_add(peer_entry.prefix,&route_entry);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = peer_route_lookup(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER + 1,PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE, &route_entry);
	if (rc != 0)
		return rc;
	rc = peer_route_delete(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER,PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE);
	if (rc != 0)
		return rc;
	rc = peer_delete(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = route_lookup(PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE, &route_entry);
	if (rc != 0)
		return rc;
	rc = peer_delete(PEER_ROUTE_ENTRY_TEST2_INITIAL_PEER + 1);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = route_lookup(PEER_ROUTE_ENTRY_TEST2_INITIAL_ROUTE, &route_entry);
	if (rc == 0)
		return -1;
	return 0;
}

#define PEER_ENTRY_TEST2_INITIAL_PEER 3001
#define PEER_ENTRY_TEST2_PEER_NUMBER 1
#define PEER_ENTRY_TEST2_INITIAL_IFINDEX1 3
#define PEER_ENTRY_TEST2_INITIAL_IFINDEX2 5

int c_rust_peer_entry_test2()
{
	int rc = 0;
	PeerEntry entry;

	for (int i = 0; i < PEER_ENTRY_TEST2_PEER_NUMBER; i++) {
		unsigned int current_peer_prefix = PEER_ENTRY_TEST2_INITIAL_PEER + i;
		entry.prefix = current_peer_prefix ;
		entry.out_ifindex =  PEER_ENTRY_TEST2_INITIAL_IFINDEX1;
		rc = peer_add_modify(current_peer_prefix, &entry);
		if (rc != 0)
			return rc;
		memset((char *)&entry,0,sizeof(entry));
		rc = peer_lookup(current_peer_prefix,&entry);
		if (rc != 0)
			return rc;
		if (entry.out_ifindex != PEER_ENTRY_TEST2_INITIAL_IFINDEX1)
			return -1;
	}
	for (int i = 0; i < PEER_ENTRY_TEST2_PEER_NUMBER; i++) {
		unsigned int current_peer_prefix = PEER_ENTRY_TEST2_INITIAL_PEER + i;
		entry.prefix = current_peer_prefix;
		entry.out_ifindex =  PEER_ENTRY_TEST2_INITIAL_IFINDEX2;
		rc = peer_add_modify(current_peer_prefix, &entry);
		if (rc != 1)
			return -1;
		memset((char *)&entry,0,sizeof(entry));
		rc = peer_lookup(current_peer_prefix,&entry);
		if (rc != 0)
			return rc;
		if (entry.out_ifindex != PEER_ENTRY_TEST2_INITIAL_IFINDEX2)
			return -1;
	}
	for (int i = 0; i < PEER_ENTRY_TEST2_PEER_NUMBER; i++) {
		unsigned int current_peer_prefix = PEER_ENTRY_TEST2_INITIAL_PEER + i;
		memset((char *)&entry,0,sizeof(entry));
		rc = peer_lookup(current_peer_prefix,&entry);
		if (rc != 0)
			return rc;
		rc = peer_delete(current_peer_prefix);
		if (rc != 0)
			return rc;
	}
	return rc;
}

#define PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER 4001
#define PEER_ROUTE_ENTRY_TEST3_PEER_NUMBER 2
#define PEER_ROUTE_ENTRY_TEST3_INITIAL_IFINDEX 3000
#define PEER_ROUTE_ENTRY_TEST3_MODIFIED_IFINDEX 5000
#define PEER_ROUTE_ENTRY_TEST3_INITIAL_NEXT_HOP 10
#define PEER_ROUTE_ENTRY_TEST3_MODIFIED_NEXT_HOP 30
#define PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE 20000
#define PEER_ROUTE_ENTRY_TEST3_ROUTE_NUMBER 1

int c_rust_peer_route_entry_test3()
{
	int rc = 0;
	PeerEntry peer_entry;
	peer_entry.prefix = PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER;
	peer_entry.out_ifindex = 3;
	printf("adding first peer\n");
	rc = peer_add(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER, &peer_entry);
	if (rc != 0)
		return rc;
	printf("done. lookup...\n");
	memset((char *)&peer_entry,0,sizeof(peer_entry));
	rc = peer_lookup(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,&peer_entry);
	if (rc != 0)
		return rc;
	printf("done\n");
	RouteEntry route_entry;
	route_entry.prefix = PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE;
	route_entry.mask = 0xFFFFFF00;
	route_entry.next_hop = PEER_ROUTE_ENTRY_TEST3_INITIAL_NEXT_HOP;
	route_entry.out_ifindex = PEER_ROUTE_ENTRY_TEST3_INITIAL_IFINDEX;
	printf("adding route to first peer\n");
	rc = peer_route_add_modify(peer_entry.prefix,&route_entry);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = peer_route_lookup(peer_entry.prefix,PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE, &route_entry);
	if (rc != 0)
		return rc;
	// adding one more peer
	peer_entry.prefix = PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER + 1;
	peer_entry.out_ifindex = PEER_ROUTE_ENTRY_TEST3_INITIAL_IFINDEX;
	printf("adding second peer\n");
	rc = peer_add(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER + 1, &peer_entry);
	if (rc != 0)
		return rc;
	printf("adding route to second peer\n");
	rc = peer_route_add_modify(peer_entry.prefix,&route_entry);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = peer_route_lookup(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER + 1,PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE, &route_entry);
	if (rc != 0)
		return rc;
	//now modify
	route_entry.prefix = PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE;
	route_entry.mask = 0xFFFFFF00;
	route_entry.next_hop = PEER_ROUTE_ENTRY_TEST3_MODIFIED_NEXT_HOP;
	route_entry.out_ifindex = PEER_ROUTE_ENTRY_TEST3_MODIFIED_IFINDEX;
	printf("modifying route\n");
	rc = peer_route_add_modify(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,&route_entry);
	if (rc != 1)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = peer_route_lookup(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER + 1,PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE, &route_entry);
	if (rc != 0)
		return rc;
	if ((route_entry.out_ifindex != PEER_ROUTE_ENTRY_TEST3_MODIFIED_IFINDEX) ||
	    (route_entry.next_hop != PEER_ROUTE_ENTRY_TEST3_MODIFIED_NEXT_HOP)) {
		printf("either next_hop %d or out_ifindex %d is not reflected\n",route_entry.next_hop,route_entry.out_ifindex);
		memset((char *)&route_entry,0,sizeof(route_entry));
		if( peer_route_lookup(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE, &route_entry) == 0) {
			printf("next_hop %d or out_ifindex %d\n",route_entry.next_hop,route_entry.out_ifindex);
		}
		return -1;
	}
	printf("deleting peer1 route\n");
	rc = peer_route_delete(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER,PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE);
	if (rc != 0)
		return rc;
	printf("deleting peer1\n");
	rc = peer_delete(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	printf("looking up for route\n");
	rc = route_lookup(PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE, &route_entry);
	if (rc != 0)
		return 0;
	printf("deleting peer2\n");
	rc = peer_delete(PEER_ROUTE_ENTRY_TEST3_INITIAL_PEER + 1);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	printf("looking up route\n");
	rc = route_lookup(PEER_ROUTE_ENTRY_TEST3_INITIAL_ROUTE, &route_entry);
	if (rc == 0)
		return -1;
	return 0;
}

