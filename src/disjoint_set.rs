
enum SetNode {
    Child(usize),
    Root(usize)
}

pub struct DisjointSet {
    data: Vec<SetNode>
}

impl DisjointSet {
    pub fn new(size: usize) -> Self {
        let mut data = Vec::with_capacity(size);
        for i in 0..size {
            data.push(SetNode::Root(i));
        }
        DisjointSet{data}
    }

    pub fn get(&mut self, i: usize) -> usize {
        match &self.data[i] {
            SetNode::Root(id) => *id,
            SetNode::Child(id) => {
                let id = *id; // remove borrow here to prevent conflict
                let correct_id = self.get(id);
                self.data[i] = SetNode::Child(correct_id);
                correct_id
            }
        }
    }

    pub fn merge(&mut self, a: usize, b: usize) {
        self.data[a] = SetNode::Child(b);
    }

    pub fn count_sets(&self) -> u32 {
        let mut count = 0;
        for node in &self.data {
            if let SetNode::Root(_) = node {
                count += 1;
            }
        }
        count
    }
}
