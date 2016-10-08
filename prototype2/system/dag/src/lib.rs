extern crate daggy;
extern crate itertools;

use daggy::NodeIndex;
use daggy::Walker;
use daggy::petgraph::graph::DefIndex;
use itertools::Itertools;
use std::collections::HashMap;
use std::convert::From;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;

pub trait DagConstraints: Clone + Debug + Hash + Eq {}
impl<T> DagConstraints for T where T: Clone + Debug + Hash + Eq {}

/// Contains systems and their dependencies
pub struct Dag<I: DagConstraints> {
  name_to_idx: HashMap<I, NodeIndex<DefIndex>>,
  name_to_alias: HashMap<I, String>,
  internal_dag: daggy::Dag<(), ()>,
}

impl<I: DagConstraints> Dag<I> {
  pub fn new() -> Dag<I> {
    Dag {
      name_to_idx: HashMap::new(),
      name_to_alias: HashMap::new(),
      internal_dag: daggy::Dag::new(),
    }
  }

  /// Provide a human-readable name for this type
  pub fn add_alias(&mut self, system_name: &I, alias: String) {
    self.name_to_alias.insert(system_name.clone(), alias);
  }

  /// Adds a dependency from the first system on the second one, indicating
  /// that the second one
  /// should go first
  pub fn add_dependency(&mut self, system_name: &I, other_name: I) {
    if *system_name == other_name {
      println!("Tried to make {:?} depend on itself!", system_name);
      panic!("Dag dependency error! Tried to depend on self! Check stdout for details.");
    }

    self.add_system(system_name);
    self.add_system(&other_name);

    let system_node = self.name_to_idx.get(system_name).unwrap();
    let other_node = self.name_to_idx.get(&other_name).unwrap();

    match self.internal_dag.add_edge(system_node.clone(), other_node.clone(), ()) {
      Err(daggy::WouldCycle(_)) => self.cycle_panic(system_name, &other_name),
      _ => (),
    }
  }

  /// Throw a whole bunch of deps in at once
  pub fn add_dependency_set(&mut self, system_name: &I, others: &[I]) {
    others.iter().foreach(|other_name| {
      self.add_dependency(system_name, other_name.clone());
    })
  }

  /// Internal method that panics if the system already exists
  pub fn add_system(&mut self, system_name: &I) {
    if !self.name_to_idx.contains_key(system_name) {
      self.force_add_system(system_name);
    }
  }

  /// Internal method that panics if the system already exists
  fn force_add_system(&mut self, system_name: &I) {
    assert!(!self.name_to_idx.contains_key(system_name),
            "Internal add_system method was called with system that already exists");

    let node = self.internal_dag.add_node(());
    self.name_to_idx.insert(system_name.clone(), node);
  }

  /// Diagnostic method for panicking on edge insert and printing the dag that
  /// caused the panic
  fn cycle_panic(&self, system_name: &I, other_name: &I) {
    println!("Dag error on add_dependency! Tried to add a dependency from ({:?}) on ({:?})",
             system_name,
             other_name);
    println!("The existing dag was:");
    println!("{}", self);

    panic!("Dag cycle error! Check stdout for the offending nodes and dag");
  }
}


impl<I: DagConstraints> Display for Dag<I> {
  /// Print the dag with the system names
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    let name_to_idx = &self.name_to_idx;
    let internal_dag = &self.internal_dag;
    let mut idx_to_name = HashMap::new();
    name_to_idx.iter().foreach(|(k, v)| {
      idx_to_name.insert(v, k);
    });

    writeln!(formatter, "").unwrap();
    name_to_idx.iter().foreach(|(k, v)| {
      writeln!(formatter, "\"{}\": [", alias_name(k, &self.name_to_alias)).unwrap();
      internal_dag.children(v.clone())
        .iter(&internal_dag)
        .map(|(_, n)| idx_to_name.get(&n).unwrap())
        .foreach(|name| {
          writeln!(formatter,
                   "  \"{}\"",
                   alias_name(name.clone(), &self.name_to_alias))
            .unwrap();
        });
      writeln!(formatter, "]").unwrap();
    });
    Ok(())
  }
}

fn alias_name<I: DagConstraints>(name: &I, name_to_alias: &HashMap<I, String>) -> String {
  name_to_alias.get(name).cloned().unwrap_or(format!("{:?}", name))
}

/// Contains numbered priorities for systems
pub struct PriorityMap<I: DagConstraints> {
  priorities: HashMap<I, usize>,
  name_to_alias: HashMap<I, String>,
}

impl<I: DagConstraints> PriorityMap<I> {
  pub fn get(&self, name: &I) -> Option<usize> {
    self.priorities.get(name).cloned()
  }
}

impl<I: DagConstraints> From<Dag<I>> for PriorityMap<I> {
  fn from(dag: Dag<I>) -> PriorityMap<I> {
    let mut idx_to_name = HashMap::new();
    dag.name_to_idx.iter().foreach(|(k, v)| {
      idx_to_name.insert(v, k);
    });

    let origin = dag.internal_dag
      .graph()
      .node_indices()
      .find(|n| dag.internal_dag.children(n.clone()).iter(&dag.internal_dag).count() == 0);

    let priorities = match origin {
      None => HashMap::new(), // Empty Dag
      Some(_) => {
        let mut priorities: HashMap<I, usize> = HashMap::new();
        // Toposort the nodes (deps before their dependents).
        let nodes = daggy::petgraph::algo::toposort(dag.internal_dag.graph());
        nodes.into_iter()
          .enumerate()
          .map(|(idx, item)| (idx, idx_to_name.get(&item).unwrap()))
          .foreach(|(idx, item)| {
            // WTF? Double clone?
            // Needed to compile. Probably got a double ref somewhere in here
            priorities.insert(item.clone().clone(), idx);
          });
        priorities
      },
    };

    PriorityMap {
      priorities: priorities,
      name_to_alias: dag.name_to_alias,
    }
  }
}

impl<I: DagConstraints> Display for PriorityMap<I> {
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    let mut priority_set = self.priorities.clone().into_iter().collect::<Vec<(I, usize)>>();
    priority_set.sort_by_key(|&(_, v)| v);
    priority_set.reverse();
    priority_set.into_iter()
      .map(|(n, v)| (alias_name(&n, &self.name_to_alias), v))
      .foreach(|(n, _)| {
        writeln!(formatter, "{}", n).unwrap();
      });
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn adding_dependencies_works() {
    let mut dag = Dag::new();

    let s1 = "first_system".to_owned();
    let s2 = "second_system".to_owned();
    let s3 = "third_system".to_owned();

    dag.add_dependency(&s1, &s2);
    dag.add_dependency(&s2, &s3);
  }

  #[test]
  #[should_panic(expected = "Dag cycle error! Check stdout for the offending nodes and dag")]
  fn creating_cycles_panics() {
    let mut dag = Dag::new();

    let s1 = "first_system".to_owned();
    let s2 = "second_system".to_owned();

    dag.add_dependency(&s1, &s2);
    dag.add_dependency(&s2, &s1);
  }

  #[test]
  fn priority_map_yields_correct_priorities() {
    let mut dag = Dag::new();

    let s1 = "first_system".to_owned();
    let s2 = "second_system".to_owned();
    let s3 = "third_system".to_owned();

    dag.add_dependency(&s1, &s2);
    dag.add_dependency(&s2, &s3);

    let priority_map = PriorityMap::from(dag);
    assert_eq!(priority_map.get(&s1), Some(0));
    assert_eq!(priority_map.get(&s2), Some(1));
    assert_eq!(priority_map.get(&s3), Some(2));
  }

  #[test]
  #[should_panic(expected = "Dag dependency error! Tried to depend on self! Check stdout for details.")]
  fn dont_depend_on_yourself_dumbass() {
    let mut dag = Dag::new();
    let s1 = "first_system".to_owned();

    dag.add_dependency(&s1, &s1);
  }
}
