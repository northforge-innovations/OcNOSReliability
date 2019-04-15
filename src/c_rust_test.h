#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct {
  uint32_t prefix;
  uint32_t next_hop;
  uint32_t out_ifindex;
} RouteEntry;

int32_t route_add(uint32_t _prefix, RouteEntry *_entry);

int32_t route_delete(uint32_t _prefix);

int32_t route_lookup(uint32_t _prefix, RouteEntry *_entry);
