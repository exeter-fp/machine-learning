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

#[cfg(test)]
pub mod test {
    use super::*;
    
    pub struct Fruit {
        id: i64,
        colour: String,
        things: i64,
        fruit: String
    }
    
    impl Fruit {
        pub fn new(id: i64, colour: String, things: i64, fruit: String) -> Self {
            Fruit { id, colour, things, fruit }
        }   
    }
    
    impl DataRow for Fruit {
        fn name(&self, col: usize) -> String {
            match col {
                0 => "Colour".to_string(),
                1 => "Things".to_string(),
                2 => "Fruit".to_string(),
                _ => panic!("Silly column")
            }
        }
        fn value(&self, col: usize) -> Col {
            match col {
                0 => Col::Text(self.colour.clone()),
                1 => Col::Int(self.things),
                2 => Col::Text(self.fruit.clone()),
                _ => panic!("Silly column")
            }
        }
        fn label(&self) -> String {
            self.fruit.clone()
        }
        fn len(&self) -> usize {
            3   
        }
        fn id(&self) -> i64 {
            self.id
        }
    }
    
    pub fn training_data<'a>() -> Vec<Fruit> {
        vec![Fruit::new(1, "Green".to_string(), 3, "Apple".to_string()),
             Fruit::new(2, "Yellow".to_string(), 3, "Apple".to_string()),
             Fruit::new(3, "Red".to_string(), 1, "Grape".to_string()),
             Fruit::new(4, "Red".to_string(), 1, "Grape".to_string()),
             Fruit::new(5, "Yellow".to_string(), 3, "Lemon".to_string())]
    }
}
