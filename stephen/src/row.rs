use std::cmp::Ordering;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Col {
    Null,
    Text (String),
    Int (i64),
    Float (f64)
}

impl Eq for Col {}

/// We can't automatically implement Ord for floats,
/// so we have to do it by hand.
impl Ord for Col {
    fn cmp(&self, other: &Col) -> Ordering {
        match (self, other) {
            (Col::Text(a), Col::Text(b)) => a.cmp(b),
            (Col::Int(a), Col::Int(b)) => a.cmp(b),
            (Col::Float(a), Col::Float(b)) => 
             if a < b {
                 Ordering::Less
             } else {
                 Ordering::Greater
             },
             _ => Ordering::Equal
        }
    }
}

impl Default for Col {
    fn default() -> Self { Col::Null }
}

pub trait DataRow {
    fn id(&self) -> i64;
    fn name(&self, col: usize) -> String;
    fn value(&self, col: usize) -> Col;
    fn label(&self) -> String;
    fn len(&self) -> usize;
}
