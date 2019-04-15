#include <stdio.h>
#include "c_rust_test.h"

void c_rust_test()
{
	RouteEntry entry;
	entry.prefix = 1;
	entry.next_hop = 2;
	entry.out_ifindex = 3;
	printf("%s %d\n",__FILE__,__LINE__);
	route_add(1, &entry);
	memset(&entry,0,sizeof(entry));
	route_lookup(1,&entry);
	printf("%s %d %x %x %x\n",__FILE__,__LINE__,entry.prefix,entry.next_hop,entry.out_ifindex);
	route_delete(1);
}
