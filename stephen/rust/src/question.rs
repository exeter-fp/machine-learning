use std::fmt;
use std::fmt::Display;
use row::{Col, DataRow};

#[derive(Debug, Clone)]
pub struct Question {
    field_name: String,
    col: usize,
    val: Col
}

impl Question {
    pub fn new(field_name: String, col: usize, val: Col) -> Self {
        Question {
            field_name,
            col,
            val
        }
    }
    
    /// Does this question match the given row?
    pub fn matchit<T>(&self, example: &T) -> bool 
    where T: DataRow {
        match (example.value(self.col), &self.val) {
            (Col::Null, _) => false,
            (_, Col::Null) => false,
            (Col::Text(ref them), Col::Text(ref us)) => them == us,
            (Col::Int(ref them), Col::Int(ref us)) => them >= us,
            (Col::Float(ref them), Col::Float(ref us)) => them >= us,
            _ => panic!("Column types didn't match. You did something thick.")
        }
    }
}

impl Display for Question {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         let (condition, value) = match &self.val {
             Col::Null => ("is".to_string(), "null".to_string()),
             Col::Text(ref text) => ("==".to_string(), text.to_string()),
             Col::Int(ref num) => (">=".to_string(), num.to_string()),
             Col::Float(ref num) => (">=".to_string(), num.to_string())
         };
         write!(f, "Is {} {} {}", self.field_name, condition, value)
    }   
}



#[cfg(test)]
mod test {
    use super::*;
    use row::test::{Fruit, training_data};

    #[test]
    fn test_question_matches() {
        let q = Question::new("Colour".to_string(), 0, Col::Text("Red".to_string()));

        assert!(q.matchit(&Fruit::new(1, "Red".to_string(), 1, "toenails".to_string())));
        assert!(!q.matchit(&Fruit::new(2, "Green".to_string(), 1, "spleen".to_string())));
        
        let q = Question::new("Colour".to_string(), 1, Col::Int(42));
        assert!(q.matchit(&Fruit::new(1, "Red".to_string(), 42, "toenails".to_string())));
        assert!(!q.matchit(&Fruit::new(2, "Red".to_string(), 1, "spleen".to_string())));
    }
    
}
