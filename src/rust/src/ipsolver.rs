

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
    use crate::ipsolver::get_branch_var;
    use std::{collections::BinaryHeap, thread::{current, JoinHandle}, time::Instant};
    use crate::lpsolver::{self, LPSolver, ProblemInstance};

    use super::{common::{FixedStatus, Node}, worker::{Worker, WorkerStats}};

    #[derive(Debug)]
    pub enum WorkOrder {
        VisitNode(Node),
        NoMoreWork,
    }

    #[derive(Debug)]
    pub enum WorkResponse {
        IntegralSolution(Node),
        Infeasible,
        NewActiveNode(Node)
    }

    #[derive(Debug)]
    pub struct SolverStats {
        pub max_heap_size: usize,
    }

    impl Default for SolverStats {
        fn default() -> Self {
            SolverStats { max_heap_size: 0 }
        }
    }

    pub struct IPSolver {
        solver_stats: SolverStats,
        active_nodes: BinaryHeap<Node>,
        problem_instance: ProblemInstance,
        lp_solver: lpsolver::LPSolver,
        thread_pool: Vec<JoinHandle<WorkerStats>>,
        work_channel_send: crossbeam::channel::Sender<WorkOrder>,
        work_response_recv: crossbeam::channel::Receiver<WorkResponse>,
        current_incumbent: Option<Node>,
        current_incumbent_obj_val: f64,
    }

    type IPSolveResult = (f64, Vec<f64>);
    
    impl IPSolver {
        const NUM_WORKERS: usize = 8;
        pub fn new(filename: &str) -> Self {
            // solve first thing ourselves and then add to active_nodes
            let (work_channel_send, work_channel_recv) = crossbeam::channel::unbounded();
            let (work_response_send, work_response_recv) = crossbeam::channel::unbounded();
            let problem_instance = lpsolver::parse_data_file(filename).unwrap();
            let lp_solver = lpsolver::LPSolver::new(filename);
            
            IPSolver {
                active_nodes: BinaryHeap::new(),
                solver_stats: SolverStats::default(),
                thread_pool: Self::init_thread_pool(filename, work_channel_recv, work_response_send),
                work_channel_send,
                work_response_recv,
                problem_instance,
                lp_solver,
                current_incumbent: None,
                current_incumbent_obj_val: f64::MAX,
            }
        }

        pub fn solve_initial_node(&mut self) {
            let initial_fixed = vec![FixedStatus::Unassigned; self.problem_instance.num_tests];
            let initial_assignments = vec![0.5; self.problem_instance.num_tests];
            let branch_var = get_branch_var(&initial_fixed, &initial_assignments);
            self.search((branch_var, FixedStatus::Present), initial_fixed.clone());
            self.search((branch_var, FixedStatus::Absent), initial_fixed);
        }

        #[cfg(feature = "all")]
        pub fn solve(mut self) -> IPSolveResult {
            // solve first
            self.solve_initial_node();
            
            while let Some(next_node) = self.active_nodes.pop() {
                let branch_var = get_branch_var(&next_node.fixed, &next_node.lp_assignments);
                self.search((branch_var, FixedStatus::Present), next_node.fixed.clone());
                self.search((branch_var, FixedStatus::Absent), next_node.fixed);
            }

            return (self.current_incumbent_obj_val, self.current_incumbent.unwrap().lp_assignments)
        }

        // #[cfg(feature = "multithread")]
        const KEEP_IN_CHANNEL: usize = 1;
        pub fn solve(mut self) -> IPSolveResult {
            // solve first
            let start = Instant::now();
            self.solve_initial_node();

            let mut in_flight_nodes = 0;

            loop {
                // println!("manager w/ work order len {}", self.work_channel_send.len());
                // println!("active ndoes len is {}", self.active_nodes.len());
                while self.work_channel_send.len() < Self::KEEP_IN_CHANNEL {
                    // println!("try getting work to send");
                    if let Some(best_node) = self.active_nodes.pop() {
                        if (best_node.objective_val < self.current_incumbent_obj_val) {
                            // if better than current inc, enq to workqueue
                            self.work_channel_send.send(WorkOrder::VisitNode(best_node));
                            in_flight_nodes += 2; // bc we should get two responses from it...
                        } else {
                            // FIXME: think this is safe but should double check...
                            println!("pruned all active nodes below {:?}", best_node);
                            self.active_nodes.clear();
                        }
                    } else {
                        // println!("no work on heap!");
                        break;
                    }

                }

                while let Ok(work_res) = self.work_response_recv.try_recv() {
                    // println!("manager -- got work response {:?}", work_res);
                    in_flight_nodes -= 1;
                    match work_res {
                        WorkResponse::Infeasible => (),
                        WorkResponse::IntegralSolution(sol) => {
                            // update incumbent if better
                            // println!("manager -- got integral node {sol:?} back");
                            if sol.objective_val < self.current_incumbent_obj_val {
                                println!("manager {:?} -- better incumbent found! {:?}", start.elapsed(), sol);
                                self.current_incumbent_obj_val = sol.objective_val;
                                self.current_incumbent = Some(sol);
                            }
                        },
                        WorkResponse::NewActiveNode(node) => {
                            // println!("manager -- got active node {node:?} back");
                            if node.objective_val < self.current_incumbent_obj_val {
                                self.active_nodes.push(node)
                            } else {
                                // println!("manager -- pruned {:?} by not adding back to heap", node);
                            }
                        },
                    }
                } 

                if self.active_nodes.len() > self.solver_stats.max_heap_size {
                    self.solver_stats.max_heap_size = self.active_nodes.len();
                }

                if (in_flight_nodes == 0 && self.active_nodes.is_empty()) {
                    println!("I think im done w/ enc {:?}!", self.current_incumbent);
                    drop(self.work_channel_send);
                    break;
                }

                // println!("nothing else to recv! back to top!!");

            }

            println!("ALL DONE CLEANING UP");

            let worker_stats = self.thread_pool.into_iter().map(|join_handle|{
                join_handle.join().unwrap()
            }).collect::<Vec<_>>();

            println!("my stats are {:?}", self.solver_stats);
            println!("workers:");
            for (i, worker) in worker_stats.iter().enumerate() {
                println!("\tworker {i} -- {worker:?}");
            }
            
            return (self.current_incumbent_obj_val, self.current_incumbent.unwrap().lp_assignments);

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
                    // integral solution
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
    
        fn init_thread_pool(
            filename: &str,
            work_channel_recv: crossbeam::channel::Receiver<WorkOrder>,
            work_response_send: crossbeam::channel::Sender<WorkResponse>
        ) -> Vec<JoinHandle<WorkerStats>> {

            // TODO: GET RID OF CORE AFFINITY
            let core_ids = core_affinity::get_core_ids().unwrap();

            let mine = core_ids.first().unwrap();
            let res = core_affinity::set_for_current(*mine);
            println!("binding thread {:?} to core {:?} had res {res:?}", current().id(), mine);

            let handles = core_ids.iter().skip(1).enumerate().map(|(worker_id, core_id)|{
                let new_work_channel_recv = work_channel_recv.clone();
                let new_work_response_send = work_response_send.clone();

                // println!("making worker id {worker_id:?}");

                

                // println!("bound worker {worker_id:?} to core {core_id:?}");
                let filename = filename.to_string();
                let new_core = core_id.to_owned();
                std::thread::spawn(move ||{
                    // let res = core_affinity::set_for_current(new_core);
                    // will fail for macOS but not on linux i think...

                    // println!("binding worker thread {:?} to core {:?} (res {res:?})", current().id(), new_core);

                    Worker::new(worker_id, new_work_channel_recv, new_work_response_send, LPSolver::new(&filename)).run()
                })
            }).collect::<Vec<_>>();

            handles
        }

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

use crate::ipsolver::common::FixedStatus;

fn get_branch_var(fixed: &Vec<FixedStatus>, lp_assignments: &Vec<f64>) -> usize {
    lp_assignments.iter().zip(fixed).enumerate().filter_map(|(i, (val, status))|{
        match status {
            FixedStatus::Unassigned => Some((i, -1.0 * (1.0 - val).abs())),
            _ => None,
        }
    })
    .max_by(|(_, val1), (_, val2)| {
        val1.total_cmp(val2)
    }).map(|(index, _)| index).unwrap()
}

mod worker {
    use std::time::Duration;
    use std::time::Instant;

    use crossbeam::channel::RecvError;

    use crate::lpsolver::LPSolver;
    use crate::ipsolver::FixedStatus;
    use crate::ipsolver::get_branch_var;

    use super::{common::Node, solver::{WorkOrder, WorkResponse}};

    #[derive(Debug)]
    pub struct WorkerStats {
        pub solves: usize,
        pub waiting_for_orders: Duration,
        pub lp_solving: Duration,
    }

    impl std::default::Default for WorkerStats {
        fn default() -> Self {
            WorkerStats { solves: 0, waiting_for_orders: Duration::from_secs(0), lp_solving: Duration::from_secs(0) }
        }
    }

    pub struct Worker {
        id: usize,
        stats: WorkerStats,
        lp_solver: LPSolver,
        work_channel_recv: crossbeam::channel::Receiver<WorkOrder>,
        work_response_send: crossbeam::channel::Sender<WorkResponse>,
    }

    impl Worker {
        pub fn new(
            id: usize, 
            work_channel_recv: crossbeam::channel::Receiver<WorkOrder>,
            work_response_send: crossbeam::channel::Sender<WorkResponse>,
            lp_solver: LPSolver,
        ) -> Self {
            Worker { id, stats: WorkerStats::default(), work_channel_recv, work_response_send, lp_solver }
        }

        pub fn run(mut self) -> WorkerStats {
            loop {
                // println!("worker id {} -- waiting for work", self.id);
                let start_wait = Instant::now();
                let order = match self.work_channel_recv.recv() {
                    Ok(order) => order,
                    Err(e) => return self.stats,
                };
                self.stats.waiting_for_orders += start_wait.elapsed();
                // println!("worker id {} -- got work order {:?}", self.id, order);
                match order {
                    WorkOrder::VisitNode(node) => {
                        let vec_res = self.branch_and_visit_node(node);
                        for res in vec_res {
                            // println!("worker id {} -- sending response {:?}", self.id, res);
                            self.work_response_send.send(res).unwrap();
                        }
                        // println!("worker id {} -- done sending responses", self.id);
                    },
                    // TODO: other orders not needed
                    WorkOrder::NoMoreWork => return self.stats,
                }
            }
        }

        pub fn branch_and_visit_node(&mut self, node: Node) -> Vec<WorkResponse> {

            let branch_var = get_branch_var(&node.fixed, &node.lp_assignments);
            let a = self.search((branch_var, FixedStatus::Present), node.fixed.clone());
            let b = self.search((branch_var, FixedStatus::Absent), node.fixed);

            // return the work response from each subtree
            vec![a, b]
        }

        fn search(&mut self, branch_assignment: (usize, FixedStatus), mut fixed: Vec<FixedStatus>) -> WorkResponse {
            fixed[branch_assignment.0] = branch_assignment.1; // update the fixed 

            self.stats.solves += 1;

            let start_solve = Instant::now();
            let res = self.lp_solver.solve(&fixed);
            self.stats.lp_solving += start_solve.elapsed();

            let solution = match res {
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
                },
                Ok(s) => Some(s),
            };

            if let Some(solution) = solution {
                let is_integral = solution.variable_values().into_iter().all(|f|f.fract() == 0.0);

                let this_node = Node::new(solution.objective_value(), fixed, solution.variable_values().to_vec());

                if is_integral {
                    return WorkResponse::IntegralSolution(this_node);
                }
                
                // push to heap
                return WorkResponse::NewActiveNode(this_node)
            } else {
                // infeasible
                return WorkResponse::Infeasible;
            }
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