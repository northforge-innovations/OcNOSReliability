#include <stdio.h>
#include "c_rust_test.h"

int c_rust_route_entry_test()
{
	int rc = 0;
	RouteEntry entry;
	entry.prefix = 1;
	entry.mask = 0xFFFFFF00;
	entry.next_hop = 2;
	entry.out_ifindex = 3;
	rc = route_add(1, &entry);
	if (rc != 0)
		return rc;
	memset((char *)&entry,0,sizeof(entry));
	rc = route_lookup(1,&entry);
	if (rc != 0)
		return rc;
	printf("%s %d %x %x %x %x\n",__FILE__,__LINE__,entry.prefix,entry.mask,entry.next_hop,entry.out_ifindex);
	rc = route_delete(1);
	return rc;
}

int c_rust_peer_entry_test()
{
	int rc = 0;
	PeerEntry entry;
	entry.prefix = 1;
	entry.out_ifindex = 3;
	rc = peer_add(1, &entry);
	if (rc != 0)
		return rc;
	memset((char *)&entry,0,sizeof(entry));
	rc = peer_lookup(1,&entry);
	if (rc != 0)
		return rc;
	printf("%s %d %x %x %x\n",__FILE__,__LINE__,entry.prefix,entry.out_ifindex);
	rc = peer_delete(1);
	return rc;
}

int c_rust_peer_route_entry_test()
{
	int rc = 0;
	PeerEntry peer_entry;
	peer_entry.prefix = 1;
	peer_entry.out_ifindex = 3;
	rc = peer_add(1, &peer_entry);
	if (rc != 0)
		return rc;
	memset((char *)&peer_entry,0,sizeof(peer_entry));
	rc = peer_lookup(1,&peer_entry);
	if (rc != 0)
		return rc;
	printf("%s %d %x %x %x\n",__FILE__,__LINE__,peer_entry.prefix,peer_entry.out_ifindex);
	RouteEntry route_entry;
	route_entry.prefix = 5;
	route_entry.mask = 0xFFFFFF00;
	route_entry.next_hop = 10;
	route_entry.out_ifindex = 30;
	rc = peer_route_add(peer_entry.prefix,&route_entry);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = peer_route_lookup(peer_entry.prefix,5, &route_entry);
	if (rc != 0)
		return rc;
	printf("%s %d %x %x %x %x\n",__FILE__,__LINE__,route_entry.prefix,route_entry.mask,route_entry.next_hop,route_entry.out_ifindex);
	// adding one more peer
	peer_entry.prefix = 2;
	peer_entry.out_ifindex = 4;
	rc = peer_add(2, &peer_entry);
	if (rc != 0)
		return rc;
	
	rc = peer_route_add(peer_entry.prefix,&route_entry);
	if (rc != 0)
		return rc;
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = peer_route_lookup(1,5, &route_entry);
	if (rc != 0)
		return rc;
	printf("%s %d %x %x %x %x\n",__FILE__,__LINE__,route_entry.prefix,route_entry.mask,route_entry.next_hop,route_entry.out_ifindex);
	memset((char *)&route_entry,0,sizeof(route_entry));
	rc = peer_route_lookup(2,5, &route_entry);
	if (rc != 0)
		return rc;
	printf("%s %d %x %x %x %x\n",__FILE__,__LINE__,route_entry.prefix,route_entry.mask,route_entry.next_hop,route_entry.out_ifindex);
	rc = peer_route_delete(1,5);
	if (rc != 0)
		return rc;
	rc = peer_route_delete(2,5);
	if (rc != 0)
		return rc;
	rc = peer_delete(1);
	if (rc != 0)
		return rc;
	rc = peer_delete(2);
	return rc;
}
