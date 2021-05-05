use std::collections::HashMap;

use super::{IRArg, IRModule, IRNode, WireColor};

#[derive(Debug,Hash,PartialEq,Eq)]
enum ConnectType {
    In,
    Out
}

#[derive(Debug)]
struct WireNet {
    color: WireColor,
    connections: Vec<(u32,ConnectType)>
}

#[derive(Default)]
struct NetRegistry {
    map: HashMap<(u32,ConnectType,WireColor),usize>,
    list: Vec<WireNet>
}

impl NetRegistry {
    fn add_link(&mut self, src_arg: &IRArg, dest_id: u32) {
        if let IRArg::Link(src_id,color) = src_arg {
            let src_key = (*src_id,ConnectType::Out,*color);
            let dest_key = (dest_id,ConnectType::In,*color);
            let src_net_exists = self.map.contains_key(&src_key);
            let dest_net_exists = self.map.contains_key(&dest_key);
    
            if src_net_exists && dest_net_exists {
                panic!("both exist");
            } else if src_net_exists {
                panic!("src exists");
            } else if dest_net_exists {
                panic!("dest exists");
            } else {
                let net = WireNet{
                    color: *color,
                    connections: vec!((*src_id,ConnectType::Out),(dest_id,ConnectType::In))
                };
                let net_id = self.list.len();
                self.list.push(net);

                self.map.insert(src_key, net_id);
                self.map.insert(dest_key, net_id);
            }
        }
    }
}

impl IRModule {
    pub fn place_nodes(&mut self) {
        let mut networks: NetRegistry = Default::default();

        for (i,node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(_) => (),
                IRNode::Output(_,arg) => {
                    networks.add_link(arg, i as u32);
                },
                IRNode::BinOp(lhs,_,rhs) => {
                    networks.add_link(lhs, i as u32);
                    networks.add_link(rhs, i as u32);
                },
                _ => panic!("todo network {:?}",node)
            }
        }

        println!("=> {:#?}",networks.list);
    }
}
