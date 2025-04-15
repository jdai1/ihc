

pub mod common {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub enum FixedStatus {
        Present,
        Absent,
        Unassigned,
    }

    #[derive(PartialEq, Debug)]
    pub struct Node {
        // objective
        pub objective_val: f64,

        // current fixed elements
        pub fixed: Vec<FixedStatus>,

        // lp assignments
        pub lp_assignments: Vec<f64>,
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

pub use solver::IPSolver;

mod solver {
    use std::{collections::BinaryHeap, thread::JoinHandle};
    use crate::lpsolver::{self, ProblemInstance};

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

    pub struct IPSolver {
        active_nodes: BinaryHeap<Node>,
        problem_instance: ProblemInstance,
        lp_solver: lpsolver::LPSolver,
        // thread_pool: Vec<JoinHandle<WorkerStats>>,
        // work_channel_send: crossbeam::channel::Sender<WorkOrder>,
        // work_response_recv: crossbeam::channel::Receiver<WorkResponse>,
        current_incumbent: Option<Node>,
        current_incumbent_obj_val: f64,
    }

    type IPSolveResult = (f64, Vec<f64>);
    
    impl IPSolver {
        // const NUM_WORKERS: usize = 8;
        pub fn new(filename: &str) -> Self {
            // solve first thing ourselves and then add to active_nodes
            // let (work_channel_send, work_channel_recv) = crossbeam::channel::unbounded();
            // let (work_response_send, work_response_recv) = crossbeam::channel::unbounded();
            let problem_instance = lpsolver::parse_data_file(filename).unwrap();
            let lp_solver = lpsolver::LPSolver::new(filename);
            
            IPSolver {
                active_nodes: BinaryHeap::new(),
                // thread_pool: Self::init_thread_pool(work_channel_recv, work_response_send),
                // work_channel_send,
                // work_response_recv,
                problem_instance,
                lp_solver,
                current_incumbent: None,
                current_incumbent_obj_val: f64::MAX,
            }
        }

        pub fn solve(mut self) -> IPSolveResult {
            // solve first
            let initial_fixed = vec![FixedStatus::Unassigned; self.problem_instance.num_tests];
            let initial_assignments = vec![0.5; self.problem_instance.num_tests];
            let branch_var = Self::get_branch_var(&initial_fixed, &initial_assignments);
            self.search((branch_var, FixedStatus::Present), initial_fixed.clone());
            self.search((branch_var, FixedStatus::Absent), initial_fixed);
            
            while let Some(next_node) = self.active_nodes.pop() {
                let branch_var = Self::get_branch_var(&next_node.fixed, &next_node.lp_assignments);
                self.search((branch_var, FixedStatus::Present), next_node.fixed.clone());
                self.search((branch_var, FixedStatus::Absent), next_node.fixed);
            }

            return (self.current_incumbent_obj_val, self.current_incumbent.unwrap().lp_assignments)
        }

        fn search(&mut self, branch_assignment: (usize, FixedStatus), mut fixed: Vec<FixedStatus>) {
            fixed[branch_assignment.0] = branch_assignment.1; // update the fixed 
            
            // println!("Fixing {} to be {:?}", branch_assignment.0, branch_assignment.1);
            // println!("Fixed: {:?}", fixed);
            // call solve
            let solution = self.lp_solver.solve(&fixed);

            let solution = match solution {
                Err(e) => {
                    match e {
                        cplex_rs::errors::Error::Cplex(c) => {
                            match c {
                                cplex_rs::errors::Cplex::Unfeasible { .. } => None,
                                other => panic!("got unexpected err {other}"),
                            }
                        },
                        random_err => panic!("got unexpected err {random_err}")
                    }
                    // case 1: INFEASIBLE
                },
                Ok(s) => Some(s),
            };

            if let Some(solution) = solution {
                // println!("Objective value: {}", solution.objective_value());
                if solution.objective_value() > self.current_incumbent_obj_val {
                    // PRUNE
                    return;
                }

                let is_integral = solution.variable_values().into_iter().all(|f|f.fract() == 0.0);
                // println!("is_integral: {}", is_integral);
                let this_node = Node::new(solution.objective_value(), fixed, solution.variable_values().to_vec());

                if is_integral {
                    if this_node.objective_val < self.current_incumbent_obj_val {
                        self.current_incumbent_obj_val = this_node.objective_val;
                        self.current_incumbent = Some(this_node);
                    }
                    return;
                }
                
                // push to heap
                self.active_nodes.push(this_node);
            } else {
                // infeasible
                return;
            }
        }

        fn get_branch_var(fixed: &Vec<FixedStatus>, lp_assignments: &Vec<f64>) -> usize {
            lp_assignments.iter().zip(fixed).enumerate().filter_map(|(i, (val, status))|{
                match status {
                    FixedStatus::Unassigned => Some((i, 1.0 - val)),
                    _ => None,
                }
            })
            .max_by(|(_, val1), (_, val2)| {
                val1.total_cmp(val2)
            }).map(|(index, _)| index).unwrap()
        }
    
        // fn init_thread_pool(
        //     work_channel_recv: crossbeam::channel::Receiver<WorkOrder>,
        //     work_response_send: crossbeam::channel::Sender<WorkResponse>
        // ) -> Vec<JoinHandle<WorkerStats>> {
        //     let mut handles = Vec::new();

        //     for i in 0..Solver::NUM_WORKERS {
        //         let new_work_channel_recv = work_channel_recv.clone();
        //         let new_work_response_send = work_response_send.clone();
        //         let jh = std::thread::spawn(move ||{
        //             Worker::new(i, new_work_channel_recv, new_work_response_send).run()
        //         });
        //         handles.push(jh);
        //     }

        //     handles
        // }

        // const MIN_WORK_BACKLOG: usize = 3;
        // fn run(mut self) {
        //     loop {
        //         // make sure there are enough elements in the work queue
        //         while self.work_channel_send.len() < Self::MIN_WORK_BACKLOG {
        //             if let Some(next_best_node) = self.active_nodes.pop() {
        //                 self.work_channel_send.send(WorkOrder::VisitNode(next_best_node)).unwrap();
        //             } else { break; }
        //         }

        //         // read off all responses and handle them
        //         while let Ok(next_response) = self.work_response_recv.try_recv() {
        //             match next_response {
        //                 WorkResponse::Pruned | WorkResponse::Infeasible => (),
        //                 WorkResponse::NewActiveNode(new_node) => {
        //                     self.active_nodes.push(new_node)
        //                 },
        //                 WorkResponse::IntegralSolution(sol) => {
        //                     self.try_update_incumbent(sol);
        //                 }
        //             }
        //         }
        //     }
        // }
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