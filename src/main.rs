use data_storage_api::*;

fn main() {
    let entry = Box::new(RouteEntry::new(1,2,3));
    route_add(1, entry);
    println!("Hello, world!");
    let mut retrieved_route = Box::new(RouteEntry::new(0,0,0));
    match route_lookup(1, &mut retrieved_route) {
        Ok(_o) => println!("retrieved route: prefix {}, next_hop {}, out_ifindex {} ",retrieved_route.get_prefix(),retrieved_route.get_next_hop(),retrieved_route.get_out_ifindex()),
        Err(_e) => println!("cannot find route")
    }
    match route_lookup(2, &mut retrieved_route) {
        Ok(_o) => println!("retrieved route: prefix {}, next_hop {}, out_ifindex {} ",retrieved_route.get_prefix(),retrieved_route.get_next_hop(),retrieved_route.get_out_ifindex()),
        Err(_e) => println!("cannot find route"),
    }
}
