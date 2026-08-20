use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationNodeKind {
    Protected,
    Reusable,
    External,
    ProjectCommand,
    Governance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationNode {
    pub id: String,
    pub kind: VerificationNodeKind,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationPlan {
    pub node_ids: Vec<String>,
    pub max_workers: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationResult {
    pub node_id: String,
    pub passed: bool,
    pub reused: bool,
}

impl VerificationNode {
    pub fn new(id: &str, kind: VerificationNodeKind, dependencies: Vec<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            dependencies,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct VerificationGraph {
    nodes: BTreeMap<String, VerificationNode>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("duplicate verification node {0}")]
    Duplicate(String),
    #[error("unknown verification dependency {0}")]
    UnknownDependency(String),
    #[error("verification graph contains a cycle")]
    Cycle,
}

impl VerificationGraph {
    pub fn add(&mut self, node: VerificationNode) -> Result<(), GraphError> {
        if self.nodes.contains_key(&node.id) {
            return Err(GraphError::Duplicate(node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub fn plan(&self) -> Result<Vec<String>, GraphError> {
        let mut indegree = self
            .nodes
            .iter()
            .map(|(id, node)| (id.clone(), node.dependencies.len()))
            .collect::<BTreeMap<_, _>>();
        let mut reverse: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for node in self.nodes.values() {
            for dependency in &node.dependencies {
                if !self.nodes.contains_key(dependency) {
                    return Err(GraphError::UnknownDependency(dependency.clone()));
                }
                reverse
                    .entry(dependency.clone())
                    .or_default()
                    .push(node.id.clone());
            }
        }
        let mut queue = indegree
            .iter()
            .filter_map(|(id, count)| (*count == 0).then_some(id.clone()))
            .collect::<VecDeque<_>>();
        let mut order = Vec::new();
        while let Some(id) = queue.pop_front() {
            order.push(id.clone());
            if let Some(dependents) = reverse.get(&id) {
                for dependent in dependents {
                    let count = indegree.get_mut(dependent).expect("dependent exists");
                    *count -= 1;
                    if *count == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }
        if order.len() != self.nodes.len() {
            return Err(GraphError::Cycle);
        }
        Ok(order)
    }

    pub fn protected_ids(&self) -> BTreeSet<String> {
        self.nodes
            .values()
            .filter(|node| node.kind == VerificationNodeKind::Protected)
            .map(|node| node.id.clone())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationCommand {
    pub id: String,
    pub program: String,
    pub args: Vec<String>,
    pub reuse: bool,
    pub protected: bool,
    pub current_dir: Option<PathBuf>,
}

impl VerificationCommand {
    pub fn new(id: &str, program: &str, args: Vec<String>) -> Self {
        Self {
            id: id.into(),
            program: program.into(),
            args,
            reuse: false,
            protected: false,
            current_dir: None,
        }
    }

    pub fn with_reuse(mut self, reuse: bool) -> Self {
        self.reuse = reuse;
        self
    }

    pub fn with_protected(mut self, protected: bool) -> Self {
        self.protected = protected;
        self
    }

    pub fn with_current_dir(mut self, current_dir: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(current_dir.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationReceipt {
    pub nodes_planned: usize,
    pub nodes_executed: usize,
    pub nodes_reused: usize,
    pub processes_spawned: usize,
    pub git_calls: usize,
    pub files_read: usize,
    pub files_hashed: usize,
    pub elapsed_ms: u128,
    pub passed: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExecutionError {
    #[error("worker count must be greater than zero")]
    InvalidWorkerCount,
    #[error("verification worker mutex was poisoned")]
    WorkerPoisoned,
}

pub fn execute_bounded(
    commands: Vec<VerificationCommand>,
    max_workers: usize,
) -> Result<VerificationReceipt, ExecutionError> {
    if max_workers == 0 {
        return Err(ExecutionError::InvalidWorkerCount);
    }
    let started = Instant::now();
    let planned = commands.len();
    let queue = Arc::new(Mutex::new(VecDeque::from(commands)));
    let metrics = Arc::new(Mutex::new((0_usize, 0_usize, 0_usize, true)));
    let worker_count = max_workers.min(planned.max(1));
    let mut workers = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let queue = Arc::clone(&queue);
        let metrics = Arc::clone(&metrics);
        workers.push(std::thread::spawn(move || {
            loop {
                let command = match queue.lock() {
                    Ok(mut queue) => queue.pop_front(),
                    Err(_) => return,
                };
                let Some(command) = command else { return };
                if command.reuse && !command.protected {
                    if let Ok(mut metrics) = metrics.lock() {
                        metrics.1 += 1;
                    }
                    continue;
                }
                let mut process = Command::new(&command.program);
                process.args(&command.args);
                if let Some(current_dir) = &command.current_dir {
                    process.current_dir(current_dir);
                }
                let result = process.status();
                if let Ok(mut metrics) = metrics.lock() {
                    metrics.0 += 1;
                    metrics.2 += 1;
                    if result.as_ref().map_or(true, |status| !status.success()) {
                        metrics.3 = false;
                    }
                }
            }
        }));
    }
    for worker in workers {
        worker.join().map_err(|_| ExecutionError::WorkerPoisoned)?;
    }
    let metrics = metrics.lock().map_err(|_| ExecutionError::WorkerPoisoned)?;
    Ok(VerificationReceipt {
        nodes_planned: planned,
        nodes_executed: metrics.0,
        nodes_reused: metrics.1,
        processes_spawned: metrics.2,
        git_calls: 0,
        files_read: 0,
        files_hashed: 0,
        elapsed_ms: started.elapsed().as_millis(),
        passed: metrics.3,
    })
}
