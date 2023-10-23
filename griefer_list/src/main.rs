use std::cmp::max;
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::time;

pub type TreeIndex = usize;

struct ScapegoatNode {
    key: String,
    ban_list: Vec<u16>,
    ban_date: u32,
    left: Option<TreeIndex>,
    right: Option<TreeIndex>,
    parent: Option<TreeIndex>,
}
impl ScapegoatNode {
    pub fn new() -> Self {
        Self {
            key: String::new(),
            ban_list: Vec::<u16>::new(),
            ban_date: 0,
            left: None,
            right: None,
            parent: None,
        }
    }
}

struct ScapegoatTree {
    root: Option<TreeIndex>,
    // arena: Vec<Option<Box<ScapegoatNode>>>,
    arena: Vec<Option<ScapegoatNode>>,
    alpha: f64,
    size: u32,
}
impl ScapegoatTree {
    pub fn new(alpha: f64) -> Self {
        Self {
            root: None,
            arena: Vec::new(),
            alpha,
            size: 0,
        }
    }

    fn new_node(user: &str, server: u16, date: u32) -> ScapegoatNode {
        // let mut new_node = Box::new(ScapegoatNode::new());
        let mut new_node = ScapegoatNode::new();
        new_node.key = user.to_string();
        new_node.ban_list.push(server);
        new_node.ban_date = date;
        return new_node;
    }

    fn alpha_height(&self, n: f64) -> u32 {
        n.log(1.0 / self.alpha).floor() as u32
    }

    // because the node pointers are Options rather than direct indices we must
    // take a reference to the Option and then unwrap it to get to the actual index,
    // did not come up with a way to clean this up while allowing tree to function and this
    // was the best way I found to avoid null reference errors

    fn size_iterative(&self, mut index: Option<TreeIndex>) -> u32 {
        let mut stack = Vec::new();
        let mut size = 0;

        while !stack.is_empty() || !index.is_none(){
            while !index.is_none(){
                stack.push(index);
                index = self.arena[index.unwrap()].as_ref().unwrap().left;
            }

            let next_index = stack.pop().unwrap();

            if !next_index.is_none(){
                size += 1;
                index = self.arena[next_index.unwrap()].as_ref().unwrap().right;
            }
        }

        size
    }

    pub fn insert(&mut self, user: &str, server: u16, date: u32) -> bool {
        let height = self.insert_key(user, server, date);
        if height.is_none() {
            return false;
        } else if height.unwrap() > self.alpha_height(self.size.into()) {
            // println!("need rebuild");
            let scapegoat = self.find_scapegoat(self.arena.len() - 1);

            if self.arena[scapegoat].as_ref().unwrap().parent.is_none() {
                // in this case the scapegoat is the root of the tree
                self.root = self.rebuild_tree(scapegoat);
                self.arena[self.root.unwrap()].as_mut().unwrap().parent = None;
            } else {
                let scapegoat_parent = self.arena[scapegoat].as_ref().unwrap().parent.unwrap();
                let rebuilt_node = self.rebuild_tree(scapegoat).unwrap();

                self.arena[rebuilt_node].as_mut().unwrap().parent = Some(scapegoat_parent);

                if self.arena[rebuilt_node].as_ref().unwrap().key
                    < self.arena[scapegoat_parent].as_ref().unwrap().key
                {
                    self.arena[scapegoat_parent].as_mut().unwrap().left = Some(rebuilt_node);
                } else {
                    self.arena[scapegoat_parent].as_mut().unwrap().right = Some(rebuilt_node);
                }
            }
        }
        // println!("height: {}, alpha_height: {}", height.unwrap(), self.alpha_height(self.size.into()));
        return true;
    }

    fn insert_key(&mut self, user: &str, server: u16, date: u32) -> Option<u32> {
        if self.root.is_none() {
            // base case for first node
            self.root = Some(0);
            self.arena.push(Some(Self::new_node(user, server, date)));
            self.size += 1;
            return Some(0);
        }

        let mut inserting = true;
        let mut depth = 0;
        let mut current_index = self.root.unwrap();

        while inserting {
            if user < &self.arena[current_index].as_ref().unwrap().key {
                // println!("inserting {} left of {}", user, &self.arena[current_index].as_ref().unwrap().key);
                match self.arena[current_index].as_ref().unwrap().left {
                    None => {
                        let new_node = Self::new_node(user, server, date);
                        self.arena.push(Some(new_node));
                        let next_index = self.arena.len() - 1;
                        self.arena[current_index].as_mut().unwrap().left = Some(next_index);
                        self.arena[next_index].as_mut().unwrap().parent = Some(current_index);
                        self.size += 1;
                        inserting = false;
                    }
                    Some(_) => {
                        current_index = self.arena[current_index].as_ref().unwrap().left.unwrap();
                    }
                }

            } else if user > &self.arena[current_index].as_ref().unwrap().key {
                // println!("inserting {} right of {}", user, &self.arena[current_index].as_ref().unwrap().key);
                match self.arena[current_index].as_ref().unwrap().right {
                    None => {
                        let new_node = Self::new_node(user, server, date);
                        self.arena.push(Some(new_node));
                        let next_index = self.arena.len() - 1;
                        self.arena[current_index].as_mut().unwrap().right = Some(next_index);
                        self.arena[next_index].as_mut().unwrap().parent = Some(current_index);
                        self.size += 1;
                        inserting = false;
                    }
                    Some(_) => {
                        current_index = self.arena[current_index].as_ref().unwrap().right.unwrap();
                    }
                }

            } else {
                // println!("found dup");
                self.arena[current_index].as_mut().unwrap().ban_date =
                    max(self.arena[current_index].as_ref().unwrap().ban_date, date);
                if !self.arena[current_index]
                    .as_ref()
                    .unwrap()
                    .ban_list
                    .contains(&server)
                {
                    self.arena[current_index]
                        .as_mut()
                        .unwrap()
                        .ban_list
                        .push(server);
                }
                return None;
            }
            depth += 1;
        }

        return Some(depth);
    }

    fn find_scapegoat(&self, mut index: TreeIndex) -> TreeIndex {
        // println!("finding scapegoat");
        let mut height = 0;
        while !self.arena[index].as_ref().unwrap().parent.is_none() {
            // the only way this doesn't happen is if we are root
            height += 1;
            let parent_index = self.arena[index].as_ref().unwrap().parent;
            let size = self.size_iterative(parent_index);
            if height > self.alpha_height(size.into()) {
                return parent_index.unwrap();
            }
            index = parent_index.unwrap();
        }
        self.root.unwrap()
    }

    pub fn search(&self, key: &str) -> Option<TreeIndex> {
        let mut current_index = self.root;

        while !current_index.is_none() {
            if key == &self.arena[current_index.unwrap()].as_ref().unwrap().key {
                return current_index;
            } else if key < &self.arena[current_index.unwrap()].as_ref().unwrap().key {
                current_index = self.arena[current_index.unwrap()].as_ref().unwrap().left;
            } else if key > &self.arena[current_index.unwrap()].as_ref().unwrap().key {
                current_index = self.arena[current_index.unwrap()].as_ref().unwrap().right;
            }
        }
        None
    }

    fn rebuild_tree(&mut self, scapegoat: TreeIndex) -> Option<TreeIndex> {
        let flattened_tree = self.flatten_tree(Some(scapegoat));
        self.arena[scapegoat].as_mut().unwrap().parent = None;
        return self.build_balanced_tree(flattened_tree);
    }

    fn flatten_tree(&self, mut index: Option<TreeIndex>) -> VecDeque<TreeIndex> {
        let mut in_order_list = VecDeque::new();
        let mut stack = Vec::new();

        while !stack.is_empty() || !index.is_none(){
            while !index.is_none(){
                stack.push(index);
                index = self.arena[index.unwrap()].as_ref().unwrap().left;
            }

            let next_index = stack.pop().unwrap();
            in_order_list.push_back(next_index.unwrap());

            if !next_index.is_none(){
                index = self.arena[next_index.unwrap()].as_ref().unwrap().right;
            }
        }

        in_order_list
    }

    fn build_balanced_tree(&mut self, mut in_order_list: VecDeque<TreeIndex>) -> Option<TreeIndex> {
        if in_order_list.len() == 0 {
            return None;
        }
        let median_index = in_order_list.len() / 2;
        let median = in_order_list[median_index];
        let mut upper_half = in_order_list.split_off(median_index);
        upper_half.pop_front();

        // Protip: you gotta update parent references when you rebuild the tree
        // or else it breaks

        let left = self.build_balanced_tree(in_order_list);
        self.arena[median].as_mut().unwrap().left = left;
        if !left.is_none(){
            self.arena[left.unwrap()].as_mut().unwrap().parent = Some(median);
        }
        
        let right = self.build_balanced_tree(upper_half);
        self.arena[median].as_mut().unwrap().right = right;
        if !right.is_none(){
            self.arena[right.unwrap()].as_mut().unwrap().parent = Some(median);
        }

        Some(median)
    }
}

fn main() -> io::Result<()> {
    let now = time::Instant::now();
    let args: Vec<String> = env::args().collect();

    let _ = match args[1].as_str() {
        "scapegoat" => scapegoat(&args[2]),
        "btree" => todo!(),
        "rbt" => todo!(),
        "avl" => todo!(),
        _ => todo!(),
    };

    let microseconds = now.elapsed();
    println!("time taken: {}\n", microseconds.as_micros());

    Ok(())
}

fn scapegoat(griefer_file_name: &String) -> io::Result<()> {
    let griefer_file = File::open(griefer_file_name).unwrap();
    let reader = BufReader::new(griefer_file);
    let mut lines = reader.lines();
    let mut tree = ScapegoatTree::new(2.0/3.0);

    while let Some(line) = lines.next() {
        let parser = line.unwrap();
        let mut info = parser.split_whitespace();
        let user = info.next().unwrap();
        let server = info.next().unwrap().parse::<u16>().unwrap();
        let date = info.next().unwrap().parse::<u32>().unwrap();
        tree.insert(user, server, date);
    }

    let stdin = io::stdin();
    let stdin_reader = stdin.lock();
    let mut stdin_buffer = BufReader::new(stdin_reader);
    let mut stdin_string = String::new();
    let stdout = io::stdout();
    let stdout_writer = stdout.lock();
    let mut stdout_buffer = BufWriter::new(stdout_writer);

    while stdin_buffer.read_line(&mut stdin_string).unwrap() > 0 {
        let located_index = tree.search(&stdin_string.trim());
        match located_index {
            None => {
                write!(
                    &mut stdout_buffer,
                    "{}  is not currently banned from any servers.\n",
                    &stdin_string.trim()
                )?;
            }
            Some(_) => {
                let name = &tree.arena[located_index.unwrap()].as_ref().unwrap().key;
                let num = tree.arena[located_index.unwrap()]
                    .as_ref()
                    .unwrap()
                    .ban_list
                    .len();
                let time = tree.arena[located_index.unwrap()]
                    .as_ref()
                    .unwrap()
                    .ban_date;
                write!(&mut stdout_buffer, "{} was banned from {} servers. most recently on {}\n", name, num, time)?;
            }
        }

        let _ = stdout_buffer.flush();
        stdin_string.clear();
    }

    Ok(())
}
