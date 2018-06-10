use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use row::{Col, DataRow};

#[derive(Debug)]
pub struct Question {
    field_name: String,
    col: usize,
    val: Col
}

impl Question {
    fn new(field_name: String, col: usize, val: Col) -> Self {
        Question {
            field_name,
            col,
            val
        }
    }
    
    /// Does this question match the given row?
    fn matchit<T>(&self, example: &T) -> bool 
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

#[derive(Debug)]
pub enum Node {
    Leaf { predictions: HashMap<String, u64> },
    Decision { question: Question,
               true_branch: Box<Node>,
               false_branch: Box<Node> } }


impl Node {
    fn new_leaf<T>(rows: &Vec<&T>) -> Self
    where T: DataRow {
        Node::Leaf {
            predictions: class_counts(rows)
        }
    }

    fn new_decision(question: Question, true_branch: Node, false_branch: Node) -> Self {
        Node::Decision {
            question,
            true_branch: Box::new(true_branch),
            false_branch: Box::new(false_branch)
        }
    }
    
    /// Create a dot format for our tree.
    pub fn to_dot(&self) -> String {
        let mut id = 0;
        let mut next_id = || {
            id = id + 1;
            id
        };

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // We won't use recursion which risks blowing up the stack.
        let mut traversal = vec![(next_id(), self)];
        
        while traversal.len() > 0 {
            let next = traversal.iter()
                .flat_map(|(id, node)| {
                    match node {
                        Node::Leaf {..}  => Vec::new(),
                        Node::Decision {true_branch,
                                        false_branch,
                                        ..} => {
                            let t_id = next_id();
                            let f_id = next_id();
                            
                            edges.push(format!("\t{}->{}", id, t_id));
                            edges.push(format!("\t{}->{}", id, f_id));

                            vec![(t_id, &**true_branch), 
                                 (f_id, &**false_branch)]
                        }
                    }
                }).collect();
            
            let mut labels = traversal.iter().filter_map(|(id, node)| {
                match node {
                    Node::Leaf {predictions} => Some(format!("\t{}[shape=circle,label=\"{}\"]", id, 
                                                             format!("{:?}", predictions).replace("\"", ""))),
                    Node::Decision {question, ..} => Some(
                        format!("\t{}[shape=box,label=\"{}\"]", id, question)
                    )
                }
            }).collect();

            nodes.append(&mut labels);

            traversal = next;
        }
        
        format!("digraph Tree {{\n{}\n{}\n}}",
                nodes.join("\n"),
                edges.join("\n"))
    }
}

/// Returns a count of each label in the dataset.
fn class_counts<T>(rows: &Vec<&T>) -> HashMap<String, u64>
where T: DataRow {
    let mut counts = HashMap::new();

    for row in rows {
        let label = &row.label(); 
        let new_count = 
            if let Some(count) = counts.get(label) {
                count + 1
            } else {
                1
            };
        counts.insert(label.clone(), new_count);
    }

    counts
}

/// Calculate the gini score for the dataset :
fn gini<T>(rows: &Vec<&T>) -> f64 
where T: DataRow {
    let counts = class_counts(rows);
    let mut impurity = 1.0;
    let len = rows.len();
    for (_label, count) in counts {
        let probability = count as f64 / len as f64;
        
        impurity -= probability.powf(2.0)
    }
    
    impurity
}

/// Returns a distinct list of the values in this column.
fn column_values<T>(rows: &Vec<&T>, col: usize) -> Vec<Col>
where T: DataRow {
    let mut column: Vec<Col> = rows.iter().map(|row| row.value(col).clone()).collect();
    column.sort();
    column.dedup();

    column
}

/// Partitions the dataset into rows that are true and rows that are false
/// according to the given question.
fn partition<'a, T>(rows: &'a Vec<&T>, question: &Question) -> (Vec<&'a T>, Vec<&'a T>)
where T: DataRow {
   rows.iter().partition(|&row| question.matchit(*row))
}

/// The uncertainty of the starting node, minus the weighted impurity of
/// two child nodes.
fn info_gain<T>(true_rows: &Vec<&T>, false_rows: &Vec<&T>, current: f64) -> f64 
where T: DataRow {
    let p = true_rows.len() as f64 / (true_rows.len() + false_rows.len()) as f64;
    current - p * gini(true_rows) - (1.0 - p) * gini(false_rows)
}

/// Find the best way to split the data
fn find_best_split<T>(rows: &Vec<&T>) -> (f64, Option<Question>) 
where T: DataRow {
    assert!(rows.len() > 0);
    let mut best_gain = 0.0;
    let mut best_question = None;
    let current_uncertainty = gini(rows);
    let first_row = rows[0];
    let nfeatures = rows[0].len();

    for col in 0..nfeatures {
        let values = column_values(rows, col);
        for val in values {
            let question = Question::new(first_row.name(col).clone(), col, val);
            let (true_rows, false_rows) = partition(rows, &question);
            
            if true_rows.len() == 0 || false_rows.len() == 0 {
                continue;
            }
            
            // Calculate the information gain from this split.
            let gain = info_gain(&true_rows, &false_rows, current_uncertainty);
            if gain > best_gain {
                best_gain = gain;
                best_question = Some(question);
            }
        }
    }
    
    (best_gain, best_question)
}

/// Build up our decision tree.
pub fn build_tree<T>(rows: Vec<&T>) -> Node 
where T: DataRow {
    
    let (gain, question) = find_best_split(&rows);
    if gain == 0.0 {
        Node::new_leaf(&rows)
    } else {
        // We have found a useful feature to partition on.
        let question = question.unwrap();
        let (true_rows, false_rows) = partition(&rows, &question);
        let true_branch = build_tree(true_rows);
        let false_branch = build_tree(false_rows);
        
        Node::new_decision(question, true_branch, false_branch)
    }
}

/// Takes the map of probabilities and chooses the most likely one.
fn prediction(classification: &HashMap<String, u64>) -> String {
    let (k, _v) = classification
        .iter()
        .max_by(|(_k1, v1), (_k2, v2)| v1.cmp(v2))
        .unwrap();
    k.to_owned()
}

pub fn classify<'a, T>(row: &T, node: &'a Node) -> String
where T: DataRow {

    match node {
        Node::Leaf { predictions } => prediction(predictions),
        Node::Decision { question,
                         true_branch,
                         false_branch } => {
            
            if question.matchit(row) {
                classify(row, &*true_branch)
            } else {
                classify(row, &*false_branch)
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    
    struct Fruit {
        colour: String,
        things: i64,
        fruit: String
    }
    
    impl Fruit {
        fn new(colour: String, things: i64, fruit: String) -> Self {
            Fruit { colour, things, fruit }
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
    }
    
    fn training_data<'a>() -> Vec<Fruit> {
        vec![Fruit::new("Green".to_string(), 3, "Apple".to_string()),
             Fruit::new("Yellow".to_string(), 3, "Apple".to_string()),
             Fruit::new("Red".to_string(), 1, "Grape".to_string()),
             Fruit::new("Red".to_string(), 1, "Grape".to_string()),
             Fruit::new("Yellow".to_string(), 3, "Lemon".to_string())]
    }

    #[test] 
    fn test_class_counts() {
        let counts = class_counts(&training_data().iter().collect());
        let mut expected = HashMap::new();
        expected.insert("Lemon".to_string(), 1);
        expected.insert("Apple".to_string(), 2);
        expected.insert("Grape".to_string(), 2);
        
        assert_eq!(expected, counts);
    }
    
    #[test] 
    fn test_gini_pure() {
        let p = vec![Fruit::new("Red".to_string(), 3, "Apple".to_string()),
                     Fruit::new("Green".to_string(), 2, "Apple".to_string())];
        
        assert_eq!(0.0, gini(&p.iter().collect()));
    }

    #[test] 
    fn test_gini_impure() {
        let p = vec![Fruit::new("Red".to_string(), 3, "Apple".to_string()),
                     Fruit::new("Red".to_string(), 3, "Orange".to_string())];
        
        assert_eq!(0.5, gini(&p.iter().collect()));
    }
    
    #[test]
    fn test_column_values() {
        let cols = column_values(&training_data().iter().collect(), 1);
        assert_eq!(vec![Col::Int(1),Col::Int(3)], cols);
    }
        
    #[test]
    fn test_question_matches() {
        let q = Question::new("Colour".to_string(), 0, Col::Text("Red".to_string()));

        assert!(q.matchit(&Fruit::new("Red".to_string(), 1, "toenails".to_string())));
        assert!(!q.matchit(&Fruit::new("Green".to_string(), 1, "spleen".to_string())));
        
        let q = Question::new("Colour".to_string(), 1, Col::Int(42));
        assert!(q.matchit(&Fruit::new("Red".to_string(), 42, "toenails".to_string())));
        assert!(!q.matchit(&Fruit::new("Red".to_string(), 1, "spleen".to_string())));
    }

    #[test]
    fn test_partition() {
        let data = training_data();
        let data = data.iter().collect();
        let q = Question::new("Colour".to_string(), 0, Col::Text("Red".to_string()));
        
        let (t, f) = partition(&data, &q);
        
        assert_eq!(2, t.len());
        assert_eq!(3, f.len());
    }
}
