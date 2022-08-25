use crate::error;
use crate::tasks;
use crate::colour;
use crate::tasks::Id;

use std::fmt::Write;
use std::collections::{HashSet, HashMap, BTreeSet};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Graph {
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub edges : HashMap<Id, HashSet<Id>>,
}

impl Graph {
    pub fn create(tasks : Vec<tasks::Task>) -> Self {
        let mut edges = HashMap::with_capacity(tasks.len());

        for task in tasks {
            edges.insert(task.data.id, task.data.dependencies);
        }

        Self {
            edges
        }
    }

    pub fn contains_node(&self, node : Id) -> bool {
        self.edges.contains_key(&node)
    }

    pub fn insert_node(&mut self, node : Id) -> bool {
        self.edges.insert(node, HashSet::new()).is_none()
    }

    pub fn insert_edge(&mut self, first : Id, second : Id) -> Result<bool, error::Error> {
        if !self.contains_node(first) || !self.contains_node(second) {
            Err(error::Error::Internal(String::from("Attempt to insert an edge in the dependency graph with a node which wasn't present")))
        }
        else if first == second {
            Err(error::Error::Generic(format!("Note with ID {} cannot depend on itself", colour::id(first))))
        }
        else {
            let outgoing = self.edges.get_mut(&first).unwrap();
            Ok(outgoing.insert(second))
        }
    }

    pub fn remove_node(&mut self, node : Id) -> bool {
        if self.edges.remove(&node).is_some() {
            for outgoing in self.edges.values_mut() {
                outgoing.remove(&node);
            }
            true
        }
        else {
            false
        }
    }

    pub fn remove_edge(&mut self, first : Id, second : Id) -> bool {
        match self.edges.get_mut(&first) {
            Some(outgoing) => {
                outgoing.remove(&second)
            },
            None => {
                false
            }
        }
    }

    pub fn find_cycle(&self) -> Option<Vec<Id>> {

        // All unvisited nodes, populated with all nodes at the start, to not miss disconnected
        // components.
        let mut unvisited = BTreeSet::<Id>::new();
        for node in self.edges.keys() {
            unvisited.insert(*node);
        }

        while !unvisited.is_empty() {
            let start = unvisited.iter().next().unwrap();

            let result = self.find_cycle_local(*start, &mut unvisited, &mut HashSet::new());
            if result.is_some() {
                return result;
            }
        }

        None
    }

    fn find_cycle_local(&self, start : Id, unvisited : &mut BTreeSet<Id>, current_path_visited : &mut HashSet<Id>) -> Option<Vec<Id>> {

        // If already visited in the current path, then there is a cycle
        if current_path_visited.contains(&start) {
            Some(vec![start])
        }
        else {
            unvisited.remove(&start);
            current_path_visited.insert(start);

            // Iterate over the outgoing edges
            for node in self.edges.get(&start).unwrap() {
                let result = self.find_cycle_local(*node, unvisited, current_path_visited);
                if let Some(mut path) = result {
                    path.push(start);
                    return Some(path);
                }
                // Remove the searched node from the current_path_visited set because already
                // reached full search depth on it.
                current_path_visited.remove(node);
            }

            None
        }
    }
}

pub fn format_cycle(cycle : &Vec<Id>) -> String {
    let mut formatted = String::new();

    for (index, node) in cycle.iter().enumerate() {
        write!(&mut formatted, "{}", colour::id(*node)).unwrap();

        if index != cycle.len() - 1 {
            formatted.push_str(" -> ");
        }
    }

    formatted
}
