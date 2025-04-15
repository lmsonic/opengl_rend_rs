use std::{cell::RefCell, rc::Rc};

pub struct Node<T: Clone + Copy> {
    children: Vec<Rc<RefCell<Node<T>>>>,
    value: T,
    index: usize,
}

#[macro_export]
macro_rules! node {
    ($elem:expr) => {
        Node::new($elem)
    };
}

impl<T: Clone + Copy> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            children: vec![],
            value,
            index: 0,
        }
    }

    pub fn leaf(mut self, child: T) -> Self {
        self.children.push(Rc::new(RefCell::new(Node::new(child))));
        self
    }
    pub fn leaves<const N: usize>(mut self, children: [T; N]) -> Self {
        self.children.append(
            &mut children
                .into_iter()
                .map(|node| Rc::new(RefCell::new(Node::new(node))))
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn node(mut self, child: Node<T>) -> Self {
        self.children.push(Rc::new(RefCell::new(child)));
        self
    }

    pub fn nodes<const N: usize>(mut self, children: [Node<T>; N]) -> Self {
        self.children.append(
            &mut children
                .into_iter()
                .map(|node| Rc::new(RefCell::new(node)))
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn add_leaf<const N: usize>(&mut self, child: T) {
        self.children.push(Rc::new(RefCell::new(Node::new(child))));
    }
    pub fn add_leaves<const N: usize>(&mut self, children: [T; N]) {
        self.children.append(
            &mut children
                .into_iter()
                .map(|node| Rc::new(RefCell::new(Node::new(node))))
                .collect::<Vec<_>>(),
        );
    }

    pub fn add_node(&mut self, child: Node<T>) {
        self.children.push(Rc::new(RefCell::new(child)));
    }

    pub fn add_nodes<const N: usize>(&mut self, children: [Node<T>; N]) {
        self.children.append(
            &mut children
                .into_iter()
                .map(|node| Rc::new(RefCell::new(node)))
                .collect::<Vec<_>>(),
        );
    }

    pub fn rebuild_indices(&mut self) {
        let depth = 0;
        for child in &self.children {
            todo!()
        }
    }

    pub fn visit(&self, f: &mut impl FnMut(usize, T)) {
        for child in &self.children {
            let value = child.borrow().value;
            let index = child.borrow().index;
            f(index, value);
            // DFS
            child.borrow_mut().visit(f);
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::Node;
    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Transform {
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
    }

    impl Transform {
        const fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
            Self {
                position,
                rotation,
                scale,
            }
        }
    }
    #[test]
    fn test_node_tree_api() {
        // Base
        let base_pos = Vec3::new(3.0, -5.0, -40.0);
        let base_ang = -45.0;
        let base = Transform::new(base_pos, Vec3::Y * base_ang, Vec3::ONE);

        let base_scale_z = 3.0;
        let base_scale = Vec3::new(1.0, 1.0, base_scale_z);
        let base_left_pos = Vec3::new(2.0, 0.0, 0.0);
        let base_right_pos = Vec3::new(-2.0, 0.0, 0.0);
        let left_base = Transform::new(base_left_pos, Vec3::ZERO, base_scale);
        let right_base = Transform::new(base_right_pos, Vec3::ZERO, base_scale);

        // Upper arm
        let upper_arm_ang = -50.0;
        let upper_arm_base = Transform::new(Vec3::ZERO, Vec3::X * upper_arm_ang, Vec3::ONE);

        let upper_arm_size = 9.0;
        let upper_arm_pos = Vec3::Z * (upper_arm_size / 2.0 - 1.0);
        let upper_arm_scale = Vec3::new(1.0, 1.0, upper_arm_size / 2.0);

        let upper_arm = Transform::new(upper_arm_pos, Vec3::ZERO, upper_arm_scale);
        // Lower arm

        let lower_arm_pos = Vec3::new(0.0, 0.0, 8.0);
        let lower_arm_ang = 60.0;
        let lower_arm_base = Transform::new(lower_arm_pos, Vec3::X * lower_arm_ang, Vec3::ONE);

        let lower_arm_len = 5.0;
        let lower_arm_width = 1.5;
        let lower_arm_pos = Vec3::Z * (lower_arm_len * 0.5);
        let lower_arm_scale = Vec3::new(
            lower_arm_width * 0.5,
            lower_arm_width * 0.5,
            lower_arm_len * 0.5,
        );
        let lower_arm = Transform::new(lower_arm_pos, Vec3::ZERO, lower_arm_scale);

        // Wrist
        let wrist_pos = Vec3::new(0.0, 0.0, 5.0);
        let wrist_roll_ang = 0.0;
        let wrist_pitch_ang = 90.0;
        let wrist_base = Transform::new(
            wrist_pos,
            Vec3::new(wrist_pitch_ang, 0.0, wrist_roll_ang),
            Vec3::ONE,
        );
        let wrist_len = 2.0;
        let wrist_width = 2.0;
        let wrist_scale = Vec3::new(wrist_width * 0.5, wrist_width * 0.5, wrist_len * 0.5);
        let wrist = Transform::new(Vec3::ZERO, Vec3::ZERO, wrist_scale);

        // Fingers
        let left_finger_pos = Vec3::new(1.0, 0.0, 1.0);
        let finger_open_ang = 70.0;
        let left_finger = Transform::new(left_finger_pos, Vec3::Y * finger_open_ang, Vec3::ONE);

        let finger_len = 2.0;
        let finger_width = 0.5;
        let finger_pos = Vec3::Z * finger_len * 0.5;
        let finger_scale = Vec3::new(finger_width * 0.5, finger_width * 0.5, finger_len * 0.5);
        let finger = Transform::new(finger_pos, Vec3::ZERO, finger_scale);
        let lower_finger_ang = 45.0;
        let lower_finger_left =
            Transform::new(Vec3::Z * finger_len, Vec3::Y * -lower_finger_ang, Vec3::ONE);

        let right_finger_pos = Vec3::new(-1.0, 0.0, 1.0);
        let right_finger = Transform::new(right_finger_pos, Vec3::Y * -finger_open_ang, Vec3::ONE);
        let lower_finger_right =
            Transform::new(Vec3::Z * finger_len, Vec3::Y * lower_finger_ang, Vec3::ONE);

        let fingers = Node::new(finger).nodes([
            Node::new(left_finger).leaves([lower_finger_left]),
            Node::new(right_finger).leaves([lower_finger_right]),
        ]);
        let wrist = Node::new(wrist_base).node(Node::new(wrist).node(fingers));
        let lower_arm = Node::new(lower_arm_base).node(Node::new(lower_arm).node(wrist));
        let upper_arm = Node::new(upper_arm_base).node(Node::new(upper_arm).node(lower_arm));
        let root = Node::new(base)
            .leaves([left_base, right_base])
            .node(upper_arm);
    }
}
