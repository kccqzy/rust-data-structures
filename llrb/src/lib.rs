use std::cmp::Ordering;
use std::ops::Not;

#[derive(Debug, Clone)]
pub struct BST<T> {
    nodes: Vec<Option<Node<T>>>,
    root: Option<Ptr>,
    deleted_indices: Vec<Ptr>
}

#[derive(Debug, Clone, Copy)]
struct Ptr(usize);

#[derive(Debug, Clone, Copy)]
enum Color {Red, Black}

#[derive(Debug, Clone)]
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

impl Not for Color {
    type Output = Color;
    fn not(self) -> Self {
        match self {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        }
    }
}

impl<T: Ord> BST<T> {
    fn deref(&self, i: &Ptr) -> &Node<T> {
        self.nodes[i.0].as_ref().expect("deref encounters a reference to a deleted node")
    }

    fn deref_mut(&mut self, i: &Ptr) -> &mut Node<T> {
        self.nodes[i.0].as_mut().expect("deref_mut encounters a reference to a deleted node")
    }

    pub fn new() -> Self {
        BST{ nodes: Vec::new(), root: None, deleted_indices: Vec::new() }
    }

    pub fn singleton(elem: T) -> Self {
        BST{ nodes: vec![Some(Node::new(elem, Color::Black))], root: Some(Ptr(0)), deleted_indices: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.nodes.len() - self.deleted_indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    fn member_impl(&self, ptr: &Option<Ptr>, elem: &T) -> bool {
        match *ptr {
            None => false,
            Some(ref ptr) => {
                let node = self.deref(ptr);
                match node.elem.cmp(elem) {
                    Ordering::Less => self.member_impl(&node.right, elem),
                    Ordering::Greater => self.member_impl(&node.left, elem),
                    Ordering::Equal => true,
                }
            }
        }
    }

    pub fn member(&self, elem: &T) -> bool {
        self.member_impl(&self.root, elem)
    }

    fn is_red(&self, ptr: &Option<Ptr>) -> bool {
        ptr.as_ref().map_or(false, |p| match self.deref(p).color { Color::Red => true, Color::Black => false })
    }

    fn rotate_left(&mut self, h: Ptr) -> Ptr {
        let x : Ptr = self.deref(&h).right.expect("rotate left on node whose left child is nil");
        self.deref_mut(&h).right = self.deref(&x).left;
        self.deref_mut(&x).left = Some(h);
        self.deref_mut(&x).color = self.deref(&h).color;
        self.deref_mut(&h).color = Color::Red;
        x
    }

    fn rotate_right(&mut self, h: Ptr) -> Ptr {
        let x : Ptr = self.deref(&h).left.expect("rotate right on node whose left child is nil");
        self.deref_mut(&h).left = self.deref(&x).right;
        self.deref_mut(&x).right = Some(h);
        self.deref_mut(&x).color = self.deref(&h).color;
        self.deref_mut(&h).color = Color::Red;
        x
    }

    fn move_red_up_or_down(&mut self, h: Ptr) {
        self.deref_mut(&h).color = !self.deref(&h).color;
        let left : Ptr = self.deref(&h).left.expect("move red up/down on node whose left child is nil");
        self.deref_mut(&left).color = !self.deref(&left).color;
        let right: Ptr = self.deref(&h).right.expect("move red up/down on node whose right child is nil");
        self.deref_mut(&right).color = !self.deref(&right).color;
    }

    fn fixup(&mut self, mut node: Ptr) -> Ptr {
        if self.is_red(&self.deref(&node).right) && !self.is_red(&self.deref(&node).left) {
            node = self.rotate_left(node);
        }
        if self.is_red(&self.deref(&node).left) && self.is_red(&self.deref(&self.deref(&node).left.unwrap()).left) {
            node = self.rotate_right(node);
        }
        if self.is_red(&self.deref(&node).left) && self.is_red(&self.deref(&node).right) {
            self.move_red_up_or_down(node);
        }
        node
    }

    fn insert_impl(&mut self, node: Option<Ptr>, elem: T) -> Ptr {
        match node {
            None => {
                let new = Some(Node::new(elem, Color::Red));
                if let Some(index) = self.deleted_indices.pop() {
                    self.nodes[index.0] = new;
                    index
                } else {
                    self.nodes.push(new);
                    Ptr(self.nodes.len() - 1)
                }
            },
            Some(node) => {
                match self.deref(&node).elem.cmp(&elem) {
                    Ordering::Less => {
                        let right : Option<Ptr> = self.deref(&node).right;
                        let new_right : Ptr = self.insert_impl(right, elem);
                        self.deref_mut(&node).right = Some(new_right);
                    },
                    Ordering::Greater => {
                        let left : Option<Ptr> = self.deref(&node).left;
                        let new_left : Ptr = self.insert_impl(left, elem);
                        self.deref_mut(&node).left = Some(new_left);
                    },
                    Ordering::Equal => self.deref_mut(&node).elem = elem,
                }
                self.fixup(node)
            }
        }
    }

    pub fn insert(&mut self, elem: T) {
        let old_root : Option<Ptr> = self.root;
        let new_root : Ptr = self.insert_impl(old_root, elem);
        self.root = Some(new_root);
        self.deref_mut(&new_root).color = Color::Black;
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.nodes.clear();
        self.deleted_indices.clear();
    }

    fn move_red_left(&mut self, mut h: Ptr) -> Ptr {
        self.move_red_up_or_down(h);
        if self.is_red(&self.deref(&self.deref(&h).right.unwrap()).left) {
            self.deref_mut(&h).right = self.deref(&h).right.map(|right| self.rotate_right(right));
            h = self.rotate_left(h);
            self.move_red_up_or_down(h);
        }
        h
    }

    fn take_min_impl(&mut self, mut node: Ptr) -> (T, Option<Ptr>) {
        match self.deref(&node).left {
            None => {
                // The current node is the minimum in the tree.
                self.deleted_indices.push(node);
                (self.nodes[node.0].take().expect("take_min_impl: leftmost node is already deleted").elem, None)
            },
            Some(left) => {
                // We need to make sure the next node is not a 2-node.
                // Making the next node not a 2-node means either it or
                // its left child is red (or both, in the case of a 4-node).
                // This checks if this is violated.
                if !self.is_red(&Some(left)) && !self.is_red(&self.deref(&left).left) {
                    node = self.move_red_left(node);
                }
                let left = self.deref_mut(&node).left.unwrap();
                let (min, new_left) = self.take_min_impl(left);
                self.deref_mut(&node).left = new_left;
                (min, Some(self.fixup(node)))
            }
        }
    }

    pub fn take_min(&mut self) -> Option<T> {
        self.root.map(
            |root|
            if self.deref(&root).left.is_none() {
                // The tree has only one element.
                let rv = self.nodes.swap_remove(root.0).unwrap().elem;
                self.root = None;
                self.deleted_indices.clear();
                self.nodes.clear();
                rv
            } else {
                // The tree has more than one element.
                let (min, new_root) = self.take_min_impl(root);
                self.root = new_root;
                self.deref_mut(&new_root.unwrap()).color = Color::Black;
                min
            })
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
    fn basics() {
        let e: BST<i32> = BST::new();
        let s = BST::singleton(2);

        assert_eq!(false, e.member(&1));
        assert_eq!(false, e.member(&2));
        assert_eq!(false, e.member(&3));

        assert_eq!(false, s.member(&1));
        assert_eq!(true, s.member(&2));
        assert_eq!(false, s.member(&3));
    }

    #[test]
    fn insertion() {
        let mut s = BST::singleton(2);
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

    #[test]
    fn taking_minimum() {
        {
            let mut tree: BST<i32> = BST::new();
            tree.insert(2);
            tree.insert(3);
            tree.insert(5);
            assert_eq!(tree.take_min(), Some(2)); // [3,5]
            tree.insert(1);                       // [1,3,5]
            assert_eq!(tree.take_min(), Some(1)); // [3,5]
            assert_eq!(tree.take_min(), Some(3)); // [5]
            tree.insert(2);                       // [2,5]
            tree.insert(1);                       // [1,2,5]
            assert_eq!(tree.take_min(), Some(1));
            assert_eq!(tree.take_min(), Some(2));
            assert_eq!(tree.take_min(), Some(5));
            assert_eq!(tree.take_min(), None);
            println!("Passed: take_min for a manual test case with interleaved insertions and deletions")
        }

        {
            let mut tree : BST<i32> = BST::new();
            for size in 0..10 {
                for i in 0..size {
                    tree.insert(i);
                }
                for i in 0..size {
                    assert_eq!(tree.take_min(), Some(i));
                }
                assert_eq!(tree.take_min(), None);
                println!("Passed: take_min for tree with {} element(s)", size);
            }
        }
    }
}
