

mod common {
    #[derive(PartialEq, Debug)]
    pub enum FixedStatus {
        Present,
        Absent,
        Unassigned,
    }

    #[derive(PartialEq, Debug)]
    pub struct Node {
        // objective
        objective_val: f64,

        // current fixed elements
        fixed: Vec<FixedStatus>,

        // lp assignments
        lp_assignments: Vec<f64>,
    }

    impl Node {
        pub fn new(objective_val: f64, fixed: Vec<FixedStatus>, lp_assignments: Vec<f64>) -> Self {
            Node { objective_val, fixed, lp_assignments }
        }

        pub fn objective_value(&self) -> f64 { self.objective_val }
    }

    impl Eq for Node { }

    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            other.objective_val.partial_cmp(&self.objective_val)
        }
    }

    impl Ord for Node {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other.objective_val.total_cmp(&self.objective_val)
        }
    }
}


mod solver {
    use std::{collections::BinaryHeap, thread::JoinHandle};
    use super::{common::{FixedStatus, Node}, worker::{Worker, WorkerStats}};

    pub enum WorkOrder {
        VisitNode(Node),
        NoMoreWork,
    }

    pub enum WorkResponse {
        IntegralSolution(Node),
        Pruned,
        Infeasible,
        NewActiveNode(Node)
    }

    pub struct Solver {
        active_nodes: BinaryHeap<Node>,
        thread_pool: Vec<JoinHandle<WorkerStats>>,
        work_channel_send: crossbeam::channel::Sender<WorkOrder>,
        work_response_recv: crossbeam::channel::Receiver<WorkResponse>,
        current_incumbent: Option<Node>,
        current_incumbent_obj_val: Option<f64>,
    }
    
    impl Solver {
        const NUM_WORKERS: usize = 8;
        fn init() -> Self {
            // solve first thing ourselves and then add to active_nodes
            let (work_channel_send, work_channel_recv) = crossbeam::channel::unbounded();
            let (work_response_send, work_response_recv) = crossbeam::channel::unbounded();
            
            Solver {
                active_nodes: BinaryHeap::new(),
                thread_pool: Self::init_thread_pool(work_channel_recv, work_response_send),
                work_channel_send,
                work_response_recv,
                current_incumbent: None,
                current_incumbent_obj_val: None,
            }
        }

        fn try_update_incumbent(&mut self, node: Node) {
            let better = match &self.current_incumbent {
                Some(incumbent) => node > *incumbent,
                None => true
            };

            if better {
                self.current_incumbent_obj_val = Some(node.objective_value());
                self.current_incumbent = Some(node);
            }
        }
    
        fn init_thread_pool(
            work_channel_recv: crossbeam::channel::Receiver<WorkOrder>,
            work_response_send: crossbeam::channel::Sender<WorkResponse>
        ) -> Vec<JoinHandle<WorkerStats>> {
            let mut handles = Vec::new();

            for i in 0..Solver::NUM_WORKERS {
                let new_work_channel_recv = work_channel_recv.clone();
                let new_work_response_send = work_response_send.clone();
                let jh = std::thread::spawn(move ||{
                    Worker::new(i, new_work_channel_recv, new_work_response_send).run()
                });
                handles.push(jh);
            }

            handles
        }

        const MIN_WORK_BACKLOG: usize = 3;
        fn run(mut self) {
            loop {
                // make sure there are enough elements in the work queue
                while self.work_channel_send.len() < Self::MIN_WORK_BACKLOG {
                    if let Some(next_best_node) = self.active_nodes.pop() {
                        self.work_channel_send.send(WorkOrder::VisitNode(next_best_node)).unwrap();
                    } else { break; }
                }

                // read off all responses and handle them
                while let Ok(next_response) = self.work_response_recv.try_recv() {
                    match next_response {
                        WorkResponse::Pruned | WorkResponse::Infeasible => (),
                        WorkResponse::NewActiveNode(new_node) => {
                            self.active_nodes.push(new_node)
                        },
                        WorkResponse::IntegralSolution(sol) => {
                            self.try_update_incumbent(sol);
                        }
                    }
                }
            }
        }
    }
}

mod worker {
    use super::{common::Node, solver::{WorkOrder, WorkResponse}};

    #[derive(Debug)]
    pub struct WorkerStats {
        nodes_visited: usize,
    }

    impl std::default::Default for WorkerStats {
        fn default() -> Self {
            WorkerStats { nodes_visited: 0 }
        }
    }

    pub struct Worker {
        id: usize,
        stats: WorkerStats,
        work_channel_recv: crossbeam::channel::Receiver<WorkOrder>,
        work_response_send: crossbeam::channel::Sender<WorkResponse>,
    }

    impl Worker {
        pub fn new(
            id: usize, 
            work_channel_recv: crossbeam::channel::Receiver<WorkOrder>,
            work_response_send: crossbeam::channel::Sender<WorkResponse>,
        ) -> Self {
            Worker { id, stats: WorkerStats::default(), work_channel_recv, work_response_send }
        }

        pub fn run(mut self) -> WorkerStats {
            loop {
                let order = self.work_channel_recv.recv();
                match order.unwrap() {
                    WorkOrder::VisitNode(node) => {
                        let vec_res = self.branch_and_visit_node(node);
                        for res in vec_res {
                            self.work_response_send.send(res).unwrap();
                        }
                    },
                    WorkOrder::NoMoreWork => return self.stats,
                }
            }
        }

        pub fn branch_and_visit_node(&mut self, node: Node) -> Vec<WorkResponse> {
            print!("worker thread {} -- visiting node {node:?}", self.id);

            // return two work responses
            todo!()
        }
    }
}



#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    // ordering is the opposite of what you'd expect bc we only have max heap
    fn ordering_test() {
        let a = common::Node::new(1.0, vec![], vec![]);
        let b = common::Node::new(1.0001, vec![], vec![]);
        assert!(a > b);

        let c = common::Node::new(0.9, vec![], vec![]);
        assert!(c > a && c > b);

        let d = common::Node::new(0.9, vec![], vec![]);
        assert!(c.cmp(&d) == Ordering::Equal);
    }
}