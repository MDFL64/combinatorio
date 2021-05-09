use std::collections::HashMap;
use rand::Rng;

use crate::{common::ConnectType, disjoint_set::DisjointSet};

use super::{IRArg, IRModule, IRNode, WireColor};

#[derive(Debug)]
struct WireNet {
    color: WireColor,
    connections: Vec<(u32,ConnectType)>
}

const MAX_DIST: f32 = 9.0;

fn square_dist(a: (f32,f32), b: (f32,f32)) -> f32 {
    let x = a.0 - b.0;
    let y = a.1 - b.1;
    x * x + y * y
}

fn check_dist(sq_dist: f32) -> bool {
    return sq_dist <= MAX_DIST * MAX_DIST;
}

impl WireNet {
    fn to_links(&self, module: &IRModule, out: &mut Vec<WireLink>) -> bool {
        
        let mut subnet_ids = DisjointSet::new(self.connections.len());

        loop {
            let mut link_count = 0;
            for id_a in 0..self.connections.len() {
                let net_id_a = subnet_ids.get(id_a);

                for id_b in (id_a+1)..self.connections.len() {
                    let net_id_b = subnet_ids.get(id_b);

                    // Don't merge matching nets
                    if net_id_a == net_id_b {
                        continue;
                    }

                    let pos_a = module.get_true_pos(self.connections[id_a].0);
                    let pos_b = module.get_true_pos(self.connections[id_b].0);

                    if !check_dist(square_dist(pos_a,pos_b)) {
                        continue;
                    }

                    link_count += 1;

                    subnet_ids.merge(net_id_a, net_id_b);

                    out.push(WireLink{
                        color: self.color,
                        a: self.connections[id_a].clone(),
                        b: self.connections[id_b].clone()
                    });
                    break;
                }
            }

            if subnet_ids.count_sets() == 1 {
                return true;
            }

            if link_count == 0 {
                return false;
            }
        }
    }

    fn correct(&self, module: &mut IRModule) {
        const MIN_FRACTION: f32 = 0.1;
        const MAX_FRACTION: f32 = 0.9;

        fn lerp_pos(start: (i32,i32), end: (i32,i32), f: f32) -> (i32,i32) {
            let x = start.0 + ((end.0 - start.0) as f32 * f).round() as i32;
            let y = start.1 + ((end.1 - start.1) as f32 * f).round() as i32;
            (x,y)
        }

        // Determine midpoint.
        let mut x_sum = 0.0;
        let mut y_sum = 0.0;
        for (id,_) in &self.connections {
            let pos = module.grid.get_pos_for(*id);
            x_sum += pos.0 as f32;
            y_sum += pos.1 as f32;
        }
        let mid_pos = (
            (x_sum / self.connections.len() as f32).round() as i32,
            (y_sum / self.connections.len() as f32).round() as i32
        );

        let mut rng = rand::thread_rng();

        // Get better positions.
        for (id,_) in &self.connections {
            if !module.can_move(*id) {
                continue;
            }

            let base_pos = module.grid.get_pos_for(*id);
            if base_pos == mid_pos {
                continue;
            }

            let fraction = MIN_FRACTION + (MAX_FRACTION - MIN_FRACTION) * rng.gen::<f32>();
            //println!("f = {}",fraction);

            let new_pos = lerp_pos(base_pos, mid_pos, fraction);
            if new_pos == mid_pos {
                continue;
            }

            if module.grid.is_cell_reserved(new_pos) {
                continue;
            }

            // Swap
            let old = module.grid.get_id_at(new_pos);
            if let Some(old_id) = old {
                if !module.can_move(old_id) {
                    continue;
                }
                module.grid.set(base_pos, old_id);
            }
            module.grid.set(new_pos, *id);
            //println!(" - swapped {} {:?}",id,old);
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
    fn add_link(&mut self, src_arg: &IRArg, dest_id: u32, module: &IRModule) {
        if let IRArg::Link(src_id,color) = src_arg {            
            if let IRNode::MultiDriver(args) = &module.nodes[*src_id as usize] {
                for arg in args {
                    let fixed_arg = if let IRArg::Link(src_id,_) = arg {
                        IRArg::Link(*src_id, *color)
                    } else {
                        panic!("raw constants not permitted in multi-driver");
                    };
                    self.add_link(&fixed_arg, dest_id, module);
                }
                return;
            }

            let src_key = (*src_id,ConnectType::Out,*color);
            let dest_key = (dest_id,ConnectType::In,*color);
            let src_net_exists = self.map.contains_key(&src_key);
            let dest_net_exists = self.map.contains_key(&dest_key);
    
            if src_net_exists && dest_net_exists {
                panic!("both exist");
            } else if src_net_exists {
                let net_id = *self.map.get(&src_key).unwrap();
                let net = &mut self.list[net_id];
                net.connections.push((dest_id,ConnectType::In));
                self.map.insert(dest_key, net_id);
            } else if dest_net_exists {
                let net_id = *self.map.get(&dest_key).unwrap();
                let net = &mut self.list[net_id];
                net.connections.push((*src_id,ConnectType::Out));
                self.map.insert(src_key, net_id);
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
    fn to_links(&self, module: &IRModule, priority_check_list: &Vec<u32>) -> Result< Vec<WireLink>, Vec<u32> > {
        let mut failed = Vec::new();
        let mut out = Vec::new();

        // check priority list first
        for i in priority_check_list {
            let net = &self.list[*i as usize];
            if !net.to_links(module, &mut out) {
                failed.push(*i);
            }
        }

        // quick exit
        if failed.len() > 0 {
            return Err(failed);
        }
        out.clear();

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

    fn is_cell_reserved(&self, key: (i32,i32)) -> bool {
        let x = key.0.rem_euclid(18);
        let y = key.1.rem_euclid(9);
        y == 0 && x <= 1
    }

    fn get_id_at(&self, key: (i32,i32)) -> Option<u32> {
        self.cell_map.get(&key).map(|x| *x)
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
            let wind_dir = (y & 1) == 1;
            for offset_x in 0..self.approx_w {
                let x = if wind_dir { base_x + offset_x } else { -base_x - offset_x };
                if !self.is_cell_filled((x,y)) && !self.is_cell_reserved((x,y)) {
                    self.set((x,y), id);
                    return;
                }
            }
            y += 1;
        }
    }
}

impl IRModule {
    pub fn layout_nodes(&mut self) {
        let mut networks: NetRegistry = Default::default();
        self.grid.init(self.nodes.len());

        print!("Layout... ");
        // Initial placement
        for (i,node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(_) => {
                    self.grid.add_input(i as u32, self.port_count);
                },
                IRNode::Constant(_) => {
                    self.grid.add_node(i as u32);
                },
                IRNode::Output(_,arg) => {
                    self.grid.add_output(i as u32, self.port_count);
                    networks.add_link(arg, i as u32, self);
                },
                IRNode::BinOp(lhs,_,rhs) | 
                IRNode::BinOpCmp(lhs,_,rhs) => {
                    self.grid.add_node(i as u32);
                    networks.add_link(lhs, i as u32, self);
                    networks.add_link(rhs, i as u32, self);
                },
                IRNode::BinOpCmpGate(lhs,_,_,gated) => {
                    self.grid.add_node(i as u32);
                    networks.add_link(lhs, i as u32, self);
                    networks.add_link(gated, i as u32, self);
                },
                IRNode::BinOpSame(arg,_) => {
                    self.grid.add_node(i as u32);
                    networks.add_link(arg, i as u32, self);
                },
                IRNode::MultiDriver(_) => (), // actual networking is handled in add_link
                _ => panic!("todo network {:?}",node)
            }
        }

        let mut pass_n = 1;
        let mut priority_list = Vec::new();
        loop {
            // Try to generate links
            let res = networks.to_links(&self, &priority_list);
            //let err_count = res.as_ref().err().map(|list| list.len()).unwrap_or(0);

            if let Err(bad_nets) = res {
                for net_id in &bad_nets {
                    networks.list[*net_id as usize].correct(self);
                }
                priority_list = bad_nets;
            } else {
                self.links = res.unwrap();
                break;
            }
            pass_n += 1;
        }
        println!("Done in {} passes.",pass_n);
    }

    fn can_move(&self, id: u32) -> bool {
        match self.nodes[id as usize] {
            IRNode::Input(..) |
            IRNode::Output(..) => false,
            _ => true
        }
    }
}
