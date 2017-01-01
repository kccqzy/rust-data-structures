use std::cmp::Ordering::*;

#[derive(Debug)]
pub struct BST<T> {
    nodes: Vec<Node<T>>,
    root: Option<Ptr>,
}

#[derive(Debug, Clone, Copy)]
struct Ptr(usize);

#[derive(Debug, Clone, Copy)]
enum Color {Red, Black}

#[derive(Debug)]
struct Node<T> {
    elem: T,
    color: Color,
    left: Option<Ptr>,
    right: Option<Ptr>,
}

impl<T: Ord> Node<T> {
    fn new(elem: T, color: Color) -> Self {
        Node { elem: elem, color: color, left: None, right: None }
    }
}

impl<T: Ord> BST<T> {
    fn deref(&self, i: &Ptr) -> &Node<T> {
        &self.nodes[i.0]
    }

    fn deref_mut(&mut self, i: &Ptr) -> &mut Node<T> {
        &mut self.nodes[i.0]
    }

    pub fn new() -> Self {
        BST{ nodes: Vec::new(), root: None }
    }

    pub fn singleton(elem: T) -> Self {
        BST{ nodes: vec![Node::new(elem, Color::Black)], root: Some(Ptr(0))}
    }

    fn member_impl(&self, ptr: &Option<Ptr>, elem: &T) -> bool {
        match *ptr {
            None => false,
            Some(ref ptr) => {
                let node = self.deref(ptr);
                match node.elem.cmp(elem) {
                    Less => self.member_impl(&node.right, elem),
                    Greater => self.member_impl(&node.left, elem),
                    Equal => true,
                }
            }
        }
    }

    pub fn member(&self, elem: &T) -> bool {
        self.member_impl(&self.root, elem)
    }

    fn is_red(&self, ptr: &Option<Ptr>) -> bool {
        ptr.as_ref().map_or(false, |p| if let Color::Red = self.deref(p).color { true } else { false })
    }

    fn rotate_left(&mut self, h: Ptr) -> Ptr {
        let x : Ptr = {self.deref(&h).right.expect("rotate left on node whose left child is nil")};
        self.deref_mut(&h).right = {self.deref(&x).left};
        self.deref_mut(&x).left = Some(h);
        self.deref_mut(&x).color = {self.deref(&h).color};
        self.deref_mut(&h).color = Color::Red;
        x
        // Note to self: the braces on the right of the assignment are to limit
        // the scope of the immutable borrow, because we are immediately
        // borrowing again mutably.
    }

    fn rotate_right(&mut self, h: Ptr) -> Ptr {
        let x : Ptr = {self.deref(&h).left.expect("rotate right on node whose left child is nil")};
        self.deref_mut(&h).left = {self.deref(&x).right};
        self.deref_mut(&x).right = Some(h);
        self.deref_mut(&x).color = {self.deref(&h).color};
        self.deref_mut(&h).color = Color::Red;
        x
    }

    fn move_red_up(&mut self, h: Ptr) {
        self.deref_mut(&h).color = Color::Red;
        let left : Ptr = {self.deref(&h).left.expect("move red up on node whose left child is nil")};
        self.deref_mut(&left).color = Color::Black;
        let right: Ptr = {self.deref(&h).right.expect("move red up on node whose right child is nil")};
        self.deref_mut(&right).color = Color::Black;
    }

    fn insert_impl(&mut self, node: Option<Ptr>, elem: T) -> Ptr {
        match node {
            None => {
                self.nodes.push(Node::new(elem, Color::Red));
                Ptr(self.nodes.len() - 1)
            },
            Some(mut node) => {
                match self.deref(&node).elem.cmp(&elem) {
                    Less => {
                        let right : Option<Ptr> = self.deref(&node).right;
                        let new_right : Ptr = self.insert_impl(right, elem);
                        self.deref_mut(&node).right = Some(new_right);
                    },
                    Greater => {
                        let left : Option<Ptr> = self.deref(&node).left;
                        let new_left : Ptr = self.insert_impl(left, elem);
                        self.deref_mut(&node).left = Some(new_left);
                    },
                    Equal => self.deref_mut(&node).elem = elem,
                }

                if self.is_red(&self.deref(&node).right) && !self.is_red(&self.deref(&node).left) {
                    node = self.rotate_left(node);
                }
                if self.is_red(&self.deref(&node).left) && self.is_red(&self.deref(&self.deref(&node).left.unwrap()).left) {
                    node = self.rotate_right(node);
                }
                if self.is_red(&self.deref(&node).left) && self.is_red(&self.deref(&node).right) {
                    self.move_red_up(node);
                }

                node
            }
        }
    }

    pub fn insert(&mut self, elem: T) {
        let old_root : Option<Ptr> = self.root;
        let new_root : Ptr = self.insert_impl(old_root, elem);
        self.root = Some(new_root);
        self.deref_mut(&new_root).color = Color::Black;
    }

    fn print_structure_inner(&self, node: Option<Ptr>) {
        match node {
            None => print!("[missing]"),
            Some(node_id) => {
                print!("{{ node ");
                let node = self.deref(&node_id);
                if let Color::Red = node.color {
                    print!("[draw=red]");
                }
                print!("{{{:?}}} ", node_id.0); // Prints order of insertion
                if let Color::Red = node.color {
                    print!("edge from parent[red]");
                }
                print!(" child ");
                self.print_structure_inner(node.left);
                print!(" child ");
                self.print_structure_inner(node.right);
                print!(" }}");
            }
        }
    }

    pub fn print_structure(&self) {
        match self.root {
            None => (),
            Some(ref node_id) => {
                println!("%% Put these in your preamble\n\
                          \\usepackage{{tikz}}\n\
                          \\usetikzlibrary{{graphdrawing}}\n\
                          \\usegdlibrary{{trees}}\n\
                          \\definecolor{{red}}{{RGB}}{{171,50,37}}\n\n\
                          %% Put these in the document body\n\
                          \\tikz [binary tree layout, nodes={{draw,circle}}, font=\\sffamily, semithick] \
                          \\node");
                let node = self.deref(&node_id);
                print!("{{{:?}}} child ", node_id.0); // Prints order of insertion
                self.print_structure_inner(node.left);
                print!(" child ");
                self.print_structure_inner(node.right);
                println!(";");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BST;

    #[test]
    fn it_works() {
        let e: BST<i32> = BST::new();
        let s = BST::singleton(2);

        assert_eq!(false, e.member(&1));
        assert_eq!(false, e.member(&2));
        assert_eq!(false, e.member(&3));

        assert_eq!(false, s.member(&1));
        assert_eq!(true, s.member(&2));
        assert_eq!(false, s.member(&3));

        let mut s = s;
        s.insert(1);
        assert_eq!(true, s.member(&1));
        assert_eq!(true, s.member(&2));
        assert_eq!(false, s.member(&3));

        s.insert(4);
        assert_eq!(true, s.member(&1));
        assert_eq!(true, s.member(&2));
        assert_eq!(false, s.member(&3));
        assert_eq!(true, s.member(&4));

        {
            let mut thousand : BST<i32> = BST::new();
            for i in 0..1000 {
                thousand.insert(i);
            }
            for i in 0..1000 {
                assert_eq!(true, thousand.member(&i));
            }
            assert_eq!(false, thousand.member(&1000));
        }

        {
            let mut thousand : BST<i32> = BST::new();
            for i in 0..1000 {
                if i % 2 == 0 {
                    thousand.insert(i);
                }
            }
            for i in 0..1000 {
                assert_eq!(i % 2 == 0, thousand.member(&i));
            }
            for i in 0..1000 {
                if i % 2 == 1 {
                    thousand.insert(i);
                }
            }
            for i in 0..1000 {
                assert_eq!(true, thousand.member(&i));
            }
        }

        {
            let mut ex : BST<i32> = BST::new();
            for c in 0..64 {
                ex.insert(c);
                ex.print_structure();
                println!("");
            }
        }

        {
            let mut ex : BST<i32> = BST::new();
            let v: [i32; 20] = [14, 9, 12, 6, 2, 10, 1, 18, 16, 5, 8, 17, 13, 3, 11, 15, 7, 19, 4, 20];
            for c in &v {
                ex.insert(*c);
                ex.print_structure();
                println!("");
            }
        }
    }
}
