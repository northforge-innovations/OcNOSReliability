#include <stdio.h>
#include "c_rust_test.h"

int c_rust_test()
{
	int rc = 0;
	RouteEntry entry;
	entry.prefix = 1;
	entry.next_hop = 2;
	entry.out_ifindex = 3;
	printf("%s %d\n",__FILE__,__LINE__);
	rc = route_add(1, &entry);
	if (rc != 0)
		return rc;
	memset(&entry,0,sizeof(entry));
	rc = route_lookup(1,&entry);
	if (rc != 0)
		return rc;
	printf("%s %d %x %x %x\n",__FILE__,__LINE__,entry.prefix,entry.next_hop,entry.out_ifindex);
	rc = route_delete(1);
	return rc;
}
