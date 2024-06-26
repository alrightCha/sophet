mod dht;
use dht::node::Node;
use dht::protocol::Protocol;
use dht::utils;

const NET_SIZE: usize = 10;

fn test_big_net() {
	let mut interfaces: Vec<Protocol> = Vec::with_capacity(NET_SIZE);
	let mut base_port = 8000;

	let root = Node::new(utils::get_local_ip().unwrap(), 7999);
	let root_interface = Protocol::new(root.ip.clone(), root.port.clone(), None);
	root_interface.put("MAIN_KEY".to_owned(), "MAIN_VALUE".to_owned());

	for i in 0..(NET_SIZE - 1) {
		let node = Node::new(utils::get_local_ip().unwrap(), base_port);

		interfaces.push(Protocol::new(node.ip, node.port, Some(root.clone())));
		println!(
			"[+] Created interface for index: {} on port: {}",
			i, base_port
		);

		base_port += 1;
	}

	for (index, interface) in interfaces.iter().enumerate() {
		println!("[+] Putting <key, value> pair for index: {}", index);
		interface.put(format!("key_{}", index), format!("value_{}", index));
	}

	for (index, interface) in interfaces.iter().enumerate() {
		let res = interface.get(format!("key_{}", index));
		println!("[*] Looking for key_{}, got {}", index, res.unwrap());
	}
}

fn main() {
	test_big_net();
}