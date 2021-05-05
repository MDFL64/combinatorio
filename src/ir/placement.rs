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

struct Grid {
    cell_map: HashMap<(i32,i32),u32>,
    node_positions: Vec<Option<(i32,i32)>>,
    approx_w: i32
}

impl Grid {
    // POWER
    // I/O
    // ...
    // ...
    fn new(size: usize) -> Self {

        let approx_w = ((size as f32 / 2.0).sqrt() * 2.0).ceil() as i32;

        let mut grid = Self{
            cell_map: HashMap::new(),
            node_positions: Vec::new(),
            approx_w
        };

        grid.node_positions.resize(size, None);

        grid
    }

    fn is_filled(&self, key: (i32,i32)) -> bool {
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
            if !self.is_filled((x,y)) {
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
            if !self.is_filled((x,y)) {
                break;
            }
            x += 1;
        }
        x += 1;
        loop {
            if !self.is_filled((x,y)) {
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
                if !self.is_filled((x,y)) {
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
        let mut grid = Grid::new(self.nodes.len());

        for (i,node) in self.nodes.iter().enumerate() {
            match node {
                IRNode::Input(_) => {
                    grid.add_input(i as u32, self.port_count);
                },
                IRNode::Output(_,arg) => {
                    grid.add_output(i as u32, self.port_count);
                    networks.add_link(arg, i as u32);
                },
                IRNode::BinOp(lhs,_,rhs) => {
                    grid.add_node(i as u32);
                    networks.add_link(lhs, i as u32);
                    networks.add_link(rhs, i as u32);
                },
                _ => panic!("todo network {:?}",node)
            }
        }

        println!("=> {:#?}",networks.list);
        println!("=> {:#?}",grid.node_positions);
    }
}
