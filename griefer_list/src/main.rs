use std::io::{self, BufRead, BufReader};
use std::collections::VecDeque;
use std::fs::File;
use std::env;
use std::cmp::max;
use std::time;

pub type TreeIndex = usize;

struct ScapegoatNode{
    key: String,
    ban_list: Vec<u16>,
    ban_date: u32,
    left: Option<TreeIndex>,
    right: Option<TreeIndex>,
    parent: Option<TreeIndex>,
}
impl ScapegoatNode{
    pub fn new() -> Self{
        Self{
        key: String::new(),
        ban_list: Vec::<u16>::new(),
        ban_date: 0,
        left: None,
        right: None,
        parent: None,
        }
    }
}

struct ScapegoatTree{
    root: Option<TreeIndex>,
    arena: Vec<Option<Box<ScapegoatNode>>>,
    // arena: Vec<Option<ScapegoatNode>>,
    alpha: f64,
    size: u32,
}
impl ScapegoatTree{
    pub fn new(alpha: f64) -> Self{
        Self{
            root: None,
            arena: Vec::new(),
            alpha,
            size: 0,
        }
    }

    fn new_node(user: &str, server: u16, date: u32) -> Box<ScapegoatNode>{
        let mut new_node = Box::new(ScapegoatNode::new());
        // let mut new_node = ScapegoatNode::new();
        new_node.key = user.to_string();
        new_node.ban_list.push(server);
        new_node.ban_date = date;
        return new_node;
    }

    fn alpha_height(&self, n: f64) -> u32{
        n.log(1.0/self.alpha).floor() as u32
    }

    // because the node pointers are Options rather than direct indices we must
    // take a reference to the Option and then unwrap it to get to the actual index,
    // did not find a way to clean this up while allowing tree to function

    fn size_iterative(&self, mut index:Option<TreeIndex>)->u32{
        let mut stack = Vec::new();
        let mut size = 0;

        while !index.is_none(){
            size += 1;
            if !self.arena[index.unwrap()].as_ref().unwrap().right.is_none(){
                stack.push(index);
            }
            index = self.arena[index.unwrap()].as_ref().unwrap().left;
        }

        size
    }

    pub fn insert(&mut self, user: &str, server: u16, date: u32)-> bool{
        let height = self.insert_key(user, server, date);
        if height.is_none(){
            return false;

        } else if height.unwrap() > self.alpha_height(self.size.into()){
            let scapegoat = self.find_scapegoat(self.arena.len()-1);
            
            if self.arena[scapegoat].as_ref().unwrap().parent.is_none(){    // in this case the scapegoat is the root of the tree
                self.root = self.rebuild_tree(scapegoat);
                self.arena[self.root.unwrap()].as_mut().unwrap().parent = None;
            } else {
                let scapegoat_parent = self.arena[scapegoat].as_ref().unwrap().parent.unwrap();
                let rebuilt_node = self.rebuild_tree(scapegoat).unwrap();
                
                self.arena[rebuilt_node].as_mut().unwrap().parent = Some(scapegoat_parent);

                if self.arena[rebuilt_node].as_ref().unwrap().key < self.arena[scapegoat_parent].as_ref().unwrap().key{
                    self.arena[scapegoat_parent].as_mut().unwrap().left = Some(rebuilt_node);
                }
                else{
                    self.arena[scapegoat_parent].as_mut().unwrap().right = Some(rebuilt_node);
                }
            }
        }
        return true;
    }

    fn insert_key(&mut self, user: &str, server: u16, date: u32)->Option<u32>{
        if self.root.is_none(){     // base case for first node
            self.root = Some(0);
            self.arena.push(Some(Self::new_node(user, server, date)));
            self.size += 1;
            return Some(0);
        }

        let mut inserting = true;
        let mut depth = 0;
        let mut current_index = self.root.unwrap();

        while inserting{
            if user < &self.arena[current_index].as_ref().unwrap().key{     // 1st immutable ref to current node
                match self.arena[current_index].as_ref().unwrap().left{     // 2nd immutable ref to current node
                    None=>{
                        let new_node = Self::new_node(user, server, date);
                        self.arena.push(Some(new_node));
                        let next_index = self.arena.len()-1;
                        self.arena[current_index].as_mut().unwrap().left = Some(next_index);    // 1st mutable ref to current node, immutables are stale
                        self.arena[next_index].as_mut().unwrap().parent = Some(current_index);  // 1st mutable ref to next node
                        self.size += 1;
                        inserting = false;
                    },
                    Some(_)=>{
                        current_index = self.arena[current_index].as_ref().unwrap().left.unwrap();
                    },
                }
            } else if user > &self.arena[current_index].as_ref().unwrap().key{
                match self.arena[current_index].as_ref().unwrap().right{
                    None=>{
                        let new_node = Self::new_node(user,server,date);
                        self.arena.push(Some(new_node));
                        let next_index = self.arena.len()-1;
                        self.arena[current_index].as_mut().unwrap().right = Some(next_index);
                        self.arena[next_index].as_mut().unwrap().parent = Some(current_index);
                        self.size += 1;
                        inserting = false;
                    },
                    Some(_)=>{
                        current_index = self.arena[current_index].as_ref().unwrap().right.unwrap();
                    }
                }
            } else {
                self.arena[current_index].as_mut().unwrap().ban_date = max(self.arena[current_index].as_ref().unwrap().ban_date, date);
                if !self.arena[current_index].as_ref().unwrap().ban_list.contains(&server){
                    self.arena[current_index].as_mut().unwrap().ban_list.push(server);
                }
                return None;
            }
            depth += 1;
        }

        return Some(depth);
    }

    fn find_scapegoat(&self, mut index: TreeIndex)->TreeIndex{
        let mut height = 0;
        while !self.arena[index].as_ref().unwrap().parent.is_none(){    // the only way this doesn't happen is if we are root
            height += 1;
            let parent_index = self.arena[index].as_ref().unwrap().parent;
            let size = self.size_iterative(parent_index);
            if height > self.alpha_height(size.into()){
                return parent_index.unwrap();
            }
            index = parent_index.unwrap();
        }
        self.root.unwrap()
    }

    pub fn search(&self, key: &str)->Option<TreeIndex>{
        let mut current_index = self.root;

        while !current_index.is_none(){   
            if key == &self.arena[current_index.unwrap()].as_ref().unwrap().key{
                return current_index;
            } else if key < &self.arena[current_index.unwrap()].as_ref().unwrap().key{
                current_index = self.arena[current_index.unwrap()].as_ref().unwrap().left;
            }
            else if key > &self.arena[current_index.unwrap()].as_ref().unwrap().key{
                current_index = self.arena[current_index.unwrap()].as_ref().unwrap().right;
            }
        }
        None
    }

    fn rebuild_tree(&mut self, scapegoat: TreeIndex)->Option<TreeIndex>{
        let flattened_tree = self.flatten_tree(scapegoat);
        self.arena[scapegoat].as_mut().unwrap().parent = None;
        return self.build_balanced_tree(flattened_tree);
    }

    fn flatten_tree(&self, index: TreeIndex)->VecDeque<TreeIndex>{
        let mut in_order_list = VecDeque::new();
        let left = self.arena[index].as_ref().unwrap().left;
        let right = self.arena[index].as_ref().unwrap().right;

        if !left.is_none(){
            in_order_list.append(&mut self.flatten_tree(left.unwrap()));
        }

        in_order_list.push_back(index);
        
        if !right.is_none(){
            in_order_list.append(&mut self.flatten_tree(right.unwrap()));
        }        
        in_order_list
    }

    fn build_balanced_tree(&mut self, mut in_order_list: VecDeque<TreeIndex>)->Option<TreeIndex>{
        if in_order_list.len() == 0{
            return None;
        }
        let median_index = in_order_list.len() / 2;
        let median = in_order_list[median_index];
        let mut upper_half = in_order_list.split_off(median_index);
        upper_half.pop_front();
        self.arena[median].as_mut().unwrap().left = self.build_balanced_tree(in_order_list);
        self.arena[median].as_mut().unwrap().right = self.build_balanced_tree(upper_half);

        Some(median)
    }

    // Deprecated functions for posterity

    // fn size(&self, index: Option<TreeIndex>) -> u32{
    //     // The size of a node is 1 + size(left) + size(right)
    //     // using the Option is helpful here because it is easy
    //     // to detect if there is no child
    //     if index.is_none(){
    //         return 0;
    //     }
    //     let left = self.arena[index.unwrap()].as_ref().unwrap().left;
    //     let right = self.arena[index.unwrap()].as_ref().unwrap().right;
    //     return 1 + self.size(left) + self.size(right);
    // }
    // fn build_balanced_tree_iterative(&mut self, mut in_order_list: VecDeque<TreeIndex>)->Option<TreeIndex>{
    //     let mut median_index = in_order_list.len() / 2;
    //     let size = in_order_list.len();
    //     let new_root = in_order_list[median_index];
    //     let stack = Vec::new();
    //     let mut left_end = 0;

    //     stack.push(new_root);

    //     while !stack.is_empty(){
    //         // handle left side
    //         while median_index != left_end{
    //             median_index = (median_index - left_end) / 2;
    //             stack.push(in_order_list[median_index])
    //         }
    //         // now at the left-most
    //         // handle right side
    //     }

    // }

    // fn lookup(&self, index: TreeIndex, key: &str)->TreeIndex{
    //     if self.arena[index].is_none() || self.arena[index].as_ref().unwrap().key == key{
    //         return index;
    //     } else {
            
    //         if key < &self.arena[index].as_ref().unwrap().key{
    //             let left = self.arena[index].as_ref().unwrap().left.unwrap();
    //             return self.lookup(left, key);
    //         } else {
    //             let right = self.arena[index].as_ref().unwrap().right.unwrap();
    //             return self.lookup(right, key);
    //         }
    //     }
    // }
}

fn main() -> io::Result<()> {
    let now = time::Instant::now();
    let args: Vec<String> = env::args().collect();

    let _ = match args[1].as_str(){
        "scapegoat"=>scapegoat(&args[2]),
        "btree"=>todo!(),
        "rbt"=>todo!(),
        "avl"=>todo!(),
        _=>todo!(),
    };

    let microseconds = now.elapsed();
    println!("time taken: {}\n", microseconds.as_micros());

    Ok(())
}

fn scapegoat(griefer_file_name: &String) -> io::Result<()>{
    let griefer_file = File::open(griefer_file_name).unwrap();
    let reader = BufReader::new(griefer_file);
    let mut lines = reader.lines();

    // for some reason at other alpha values than .75 it gives the wrong output
    // either missing inserting some nodes or else losing references to
    // them somehow and then when searching returns that a user hasn't been banned
    // when they should be
    let mut tree = ScapegoatTree::new(0.75);

    while let Some(line) = lines.next(){
        let parser = line.unwrap();
        let mut info = parser.split_whitespace();
        let user = info.next().unwrap();
        let server = info.next().unwrap().parse::<u16>().unwrap();
        let date = info.next().unwrap().parse::<u32>().unwrap();
        tree.insert(user, server, date);
    }

    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let mut stdin_buffer = String::new();

    while stdin_reader.read_line(&mut stdin_buffer).unwrap() > 0{
        let located_index = tree.search(&stdin_buffer.strip_suffix("\r\n").unwrap());
        match located_index {
            None=>{
                println!("{} is not currently banned from any servers.", stdin_buffer.strip_suffix("\r\n").unwrap());
            },
            Some(_)=>{
                let name = &tree.arena[located_index.unwrap()].as_ref().unwrap().key;
                let num = tree.arena[located_index.unwrap()].as_ref().unwrap().ban_list.len();
                let time = tree.arena[located_index.unwrap()].as_ref().unwrap().ban_date;
                println!("{} was banned from {} servers. most recently on: {}", name, num, time);
            }
        }
        
        stdin_buffer.clear();
    }


    Ok(())
}