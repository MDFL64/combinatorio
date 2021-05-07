use std::collections::HashMap;

use crate::common::ConnectType;

use super::{IRArg, IRModule, IRNode, WireColor};

#[derive(Debug)]
struct WireNet {
    color: WireColor,
    connections: Vec<(u32,ConnectType)>
}

const MAX_DIST: f32 = 9.0;

fn check_dist(a: (f32,f32), b: (f32,f32)) -> bool {
    let x = a.0 - b.0;
    let y = a.1 - b.1;
    let sq_dist = x * x + y * y;
    return sq_dist <= MAX_DIST * MAX_DIST;
}

impl WireNet {
    fn to_links(&self, module: &IRModule, out: &mut Vec<WireLink>) -> bool {
        if self.connections.len() == 2 {
            let a = self.connections[0].clone();
            let b = self.connections[1].clone();
            let pos_a = module.get_true_pos(a.0);
            let pos_b = module.get_true_pos(b.0);
            if !check_dist(pos_a,pos_b) {
                println!("{:?} {:?} / {:?} {:?}",a,b,pos_a,pos_b);
                return false;
            }
            out.push(WireLink{
                color: self.color,
                a,
                b
            });
            true
        } else {
            panic!("todo make this actually work for non-trivial")
        }
    }
}

#[derive(Debug)]
pub struct WireLink {
    pub color: WireColor,
    pub a: (u32,ConnectType),
    pub b: (u32,ConnectType)
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

    // On success, returns a list of links.
    // On failure, returns a list of bad network IDs.
    fn to_links(&self, module: &IRModule) -> Result< Vec<WireLink>, Vec<u32> > {
        let mut out = Vec::new();
        let mut failed = Vec::new();
        for (i, net) in self.list.iter().enumerate() {
            if !net.to_links(module, &mut out) {
                failed.push(i as u32);
            }
        }

        if failed.len() > 0 {
            Err(failed)
        } else {
            Ok(out)
        }
    }
}

#[derive(Default, Debug)]
pub struct Grid {
    cell_map: HashMap<(i32,i32),u32>,
    node_positions: Vec<Option<(i32,i32)>>,
    approx_w: i32
}

impl Grid {
    // POWER
    // I/O
    // ...
    // ...
    fn init(&mut self, size: usize) {

        self.approx_w = ((size as f32 / 2.0).sqrt() * 2.0).ceil() as i32;

        self.node_positions.resize(size, None);
    }

    pub fn get_pos_for(&self, id: u32) -> (i32,i32) {
        self.node_positions[id as usize].unwrap()
    }

    fn is_cell_filled(&self, key: (i32,i32)) -> bool {
        self.cell_map.get(&key).is_some()
    }

    fn set(&mut self, key: (i32,i32), val: u32) {
        if let Some(current_id) = self.cell_map.get(&key) {
            self.node_positions[*current_id as usize] = None;
        }
        if let Some(current_pos) = self.node_positions[val as usize] {
            self.cell_map.remove(&current_pos);
        }
        self.cell_map.insert(key,val);
        self.node_positions[val as usize] = Some(key);
    }

    // Initial layout is very inefficent. We just check every cell each time until we find an empty one.
    fn add_input(&mut self, id: u32, port_count: i32) {
        let mut x = -port_count/2;
        let y = 1;
        loop {
            if !self.is_cell_filled((x,y)) {
                self.set((x,y), id);
                return;
            }
            x += 1;
        }
    }

    fn add_output(&mut self, id: u32, port_count: i32) {
        let mut x = -port_count/2;
        let y = 1;
        loop {
            if !self.is_cell_filled((x,y)) {
                break;
            }
            x += 1;
        }
        x += 1;
        loop {
            if !self.is_cell_filled((x,y)) {
                self.set((x,y), id);
                return;
            }
            x += 1;
        }
    }

    fn add_node(&mut self, id: u32) {
        let base_x = -self.approx_w/2;
        let mut y = 2;

        loop {
            for offset_x in 0..self.approx_w {
                let x = base_x + offset_x;
                if !self.is_cell_filled((x,y)) {
                    self.set((x,y), id);
                    return;
                }
            }
            y += 1;
        }
    }
}

impl IRModule {
    pub fn place_nodes(&mut self) {
        let mut networks: NetRegistry = Default::default();
        self.grid.init(self.nodes.len());

        // Initial placement
        for (i,node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(_) => {
                    self.grid.add_input(i as u32, self.port_count);
                },
                IRNode::Output(_,arg) => {
                    self.grid.add_output(i as u32, self.port_count);
                    networks.add_link(arg, i as u32);
                },
                IRNode::BinOp(lhs,_,rhs) => {
                    self.grid.add_node(i as u32);
                    networks.add_link(lhs, i as u32);
                    networks.add_link(rhs, i as u32);
                },
                IRNode::BinOpSame(arg,_) => {
                    self.grid.add_node(i as u32);
                    networks.add_link(arg, i as u32);
                },
                _ => panic!("todo network {:?}",node)
            }
        }

        // Try to generate links
        let res = networks.to_links(&self);

        self.links = res.expect("there was an error");
    }
}
