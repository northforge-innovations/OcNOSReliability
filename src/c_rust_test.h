#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct {
  uint32_t prefix;
  uint32_t out_ifindex;
} PeerEntry;

typedef struct {
  uint32_t prefix;
  uint32_t mask;
  uint32_t next_hop;
  uint32_t out_ifindex;
} RouteEntry;

int32_t peer_add(uint32_t _prefix, PeerEntry *_entry);

int32_t peer_delete(uint32_t _prefix);

int32_t peer_lookup(uint32_t _prefix, PeerEntry *_entry);

int32_t peer_route_add(uint32_t _peer_prefix, RouteEntry *_entry);

int32_t peer_route_delete(uint32_t _peer_prefix, uint32_t _route_prefix);

int32_t peer_route_lookup(uint32_t _peer_prefix, uint32_t _route_prefix, RouteEntry *_entry);

int32_t route_lookup(uint32_t _prefix, RouteEntry *_entry);
