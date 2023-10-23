use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::env;
use std::cmp::max;

struct ScapegoatNode{
    key: String,
    ban_list: Vec<u16>,
    ban_date: u32,
    left: Option<Box<ScapegoatNode>>,
    right: Option<Box<ScapegoatNode>>,
    parent: Option<Box<ScapegoatNode>>,
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
    root: Option<Box<ScapegoatNode>>,
    alpha: f64,
    size: u32,
}
impl ScapegoatTree{
    pub fn new(alpha: f64) -> Self{
        Self{
            root: None,
            alpha,
            size: 0,
        }
    }

    pub fn insert(&mut self, user: &str, server: u16, date: u32){
        let node = Some(Box::new(Self::new_node(user, server, date)));
        let height = self.insert_key(node);
        // return the root node and the new node if it was inserted
        // then use parent refs to update size if the new_node is not None
        let size = self.size as f64;
        if height > (size.log(1.0 / self.alpha).floor() as u32){

        }
    }

    fn insert_key(&mut self, mut new_node: Option<Box<ScapegoatNode>>) -> u32 {
        let mut current_node = &self.root;
        if current_node.is_none(){ // Special case to handle first node
            self.root = new_node;
            self.size += 1;
            return 0;
        }

        let mut inserting = true;
        let mut depth = 0;

        while inserting{
            // awkward because nodes are Options rather than ScapegoatNodes directly
            // we take a reference to the Option and then unwrap it to get to the
            // actual ScapegoatNode. Did not find a way to clean this up while allowing
            // updating which Option is being looked at to traverse the tree
            if new_node.as_ref().unwrap().key < current_node.as_ref().unwrap().key{
                match current_node.as_ref().unwrap().left{
                    None => {current_node.as_ref().unwrap().left = new_node; 
                            new_node.as_mut().unwrap().parent = *current_node; 
                            inserting = false;
                            self.size += 1;
                        },
                    Some(_) => {current_node = &current_node.as_ref().unwrap().left},
                }
            } else if new_node.as_ref().unwrap().key > current_node.as_ref().unwrap().key{

            } else{

            }
        }

        return depth;
    }

    fn new_node(user: &str, server: u16, date: u32) -> ScapegoatNode{
        let mut new_node = ScapegoatNode::new();
        new_node.key = user.to_string();
        new_node.ban_list.push(server);
        new_node.ban_date = date;
        return new_node;
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut working = true;
    let mut curr_val = Some(5);

    while working{
        match curr_val {
            Some(mut val) => {
                val *= 2;
                println!("{:?}", curr_val.unwrap());
                println!("{:?}", val);
            },
            None=>{working = false;}
        }
    }

    println!("done");

    let _ = match args[1].as_str(){
        "scapegoat"=>scapegoat(&args[2]),
        "btree"=>todo!(),
        "rbt"=>todo!(),
        "avl"=>todo!(),
        _=>todo!(),
    };

    Ok(())
}

fn scapegoat(griefer_file_name: &String) -> io::Result<()>{
    let griefer_file = File::open(griefer_file_name).unwrap();
    let reader = BufReader::new(griefer_file);
    let mut lines = reader.lines();
    let mut tree = ScapegoatTree::new(0.66);

    while let Some(line) = lines.next(){
        // let info: Vec<&str> = line.unwrap().split_whitespace().collect();
        let parser = line.unwrap();
        let mut info = parser.split_whitespace();
        let user = info.next().unwrap();
        let server = info.next().unwrap().parse::<u16>().unwrap();
        let date = info.next().unwrap().parse::<u32>().unwrap();
        tree.insert(user, server, date);
    }

    Ok(())
}


            if &self.arena[current_index].as_ref().unwrap().key == "mqmaclrlqzb"{
                if !self.arena[current_index].as_ref().unwrap().left.is_none(){
                    println!("left child of mqmaclrlqzb is {}", &self.arena[self.arena[current_index].as_ref().unwrap().left.unwrap()].as_ref().unwrap().key); 
                }
            }
            if &self.arena[current_index].as_ref().unwrap().key == "mqlckikcx@w"{
                if !self.arena[current_index].as_ref().unwrap().right.is_none(){
                    println!("right child of mqlckikcx@w is {}", &self.arena[self.arena[current_index].as_ref().unwrap().right.unwrap()].as_ref().unwrap().key); 
                }
            }
            if &self.arena[current_index].as_ref().unwrap().key == "mqrplxclrq"{
                if !self.arena[current_index].as_ref().unwrap().left.is_none(){
                    println!("left child of mqrplxclrq is {}", &self.arena[self.arena[current_index].as_ref().unwrap().left.unwrap()].as_ref().unwrap().key); 
                }
            }


