use itertools::Itertools;
use question::Question;
use row::{Col, DataRow};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Node {
    Leaf {
        predictions: HashMap<String, usize>,
    },
    Decision {
        question: Question,
        true_branch: Box<Node>,
        false_branch: Box<Node>,
    },
}

impl Node {
    fn new_leaf<T>(rows: &[&T]) -> Self
    where
        T: DataRow,
    {
        Node::Leaf {
            predictions: class_counts(rows),
        }
    }

    fn new_decision(question: Question, true_branch: Node, false_branch: Node) -> Self {
        Node::Decision {
            question,
            true_branch: Box::new(true_branch),
            false_branch: Box::new(false_branch),
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
            let next = traversal
                .iter()
                .flat_map(|(id, node)| match node {
                    Node::Leaf { .. } => Vec::new(),
                    Node::Decision {
                        true_branch,
                        false_branch,
                        ..
                    } => {
                        let t_id = next_id();
                        let f_id = next_id();

                        edges.push(format!("\t{}->{}[fontsize=32,label=\"yes\"];", id, t_id));
                        edges.push(format!("\t{}->{}[fontsize=32,label=\"no\"];", id, f_id));

                        vec![(t_id, &**true_branch), (f_id, &**false_branch)]
                    }
                })
                .collect();

            let mut labels = traversal
                .iter()
                .filter_map(|(id, node)| match node {
                    Node::Leaf { predictions } => Some(format!(
                        "\t{}[shape=circle,label=\"{}\"];",
                        id,
                        format!("{:?}", predictions).replace("\"", "")
                    )),
                    Node::Decision { question, .. } => {
                        Some(format!("\t{}[shape=box,label=\"{}\"];", id, question))
                    }
                })
                .collect();

            nodes.append(&mut labels);

            traversal = next;
        }

        format!(
            "digraph Tree {{\n{}\n{}\n}}",
            nodes.join("\n"),
            edges.join("\n")
        )
    }
}

///
/// Returns a count of each label in the dataset.
/// Groups by the label and then sums the occurrences for each label.
///
fn class_counts<T>(rows: &[&T]) -> HashMap<String, usize>
where
    T: DataRow,
{
    rows.iter()
        .group_by(|row| row.label().to_owned())
        .into_iter()
        .map(|(label, count)| (label, count.count()))
        .collect()
}

///
/// Returns a count of each label in the dataset and a total count of the iterator.
/// Groups by the label and then sums the occurrences for each label.
///
/// Gets everything we need by just going through it once.
///
fn class_counts_iter<'a, T, I>(rows: I) -> (usize, HashMap<String, usize>)
where
    T: DataRow,
    I: Iterator<Item = &'a &'a T>,
    T: 'a,
{
    rows.fold((0, HashMap::new()), |(len, mut group), row| {
        let count = match group.get(&row.label()) {
            Some(count) => *count,
            None => 0,
        };

        group.insert(row.label(), count + 1);

        (len + 1, group)
    })
}

///
/// Calculate the gini score for the dataset
/// Returns the length of the iterator as well since we
/// have calculated it whilst exhausting our iterator.
///
fn gini<'a, T, I>(rows: I) -> (usize, f64)
where
    T: DataRow,
    I: Iterator<Item = &'a &'a T>,
    T: 'a,
{
    let (len, counts) = class_counts_iter(rows);

    (
        len,
        counts.values().fold(1.0, |impurity, &count| {
            let probability = count as f64 / len as f64;
            impurity - probability.powf(2.0)
        }),
    )
}

/// Returns a distinct list of the values in this column.
fn column_values<'a, T, I>(rows: I, col: usize) -> Vec<Col>
where
    T: DataRow,
    I: Iterator<Item = &'a &'a T>,
    T: 'a,
{
    let mut column: Vec<Col> = rows.map(|row| row.value(col).clone()).collect();
    column.sort();
    column.dedup();

    column
}

/// Partitions the dataset into rows that are true and rows that are false
/// according to the given question.
fn partition<'a, T>(rows: &'a [&T], question: &Question) -> (Vec<&'a T>, Vec<&'a T>)
where
    T: DataRow,
{
    rows.iter().partition(|&row| question.matchit(*row))
}

///
/// Partitions the dataset into rows that are true and rows that are false
/// according to the given question.
///
fn partition_iter<'a, T, I>(
    rows: I,
    question: &'a Question,
) -> (
    impl Iterator<Item = &'a &'a T>,
    impl Iterator<Item = &'a &'a T>,
)
where
    T: DataRow,
    I: Iterator<Item = &'a &'a T> + Clone,
    T: 'a,
{
    (
        rows.clone().filter(move |&a| question.matchit(*a)),
        rows.filter(move |&a| !question.matchit(*a)),
    )
}

/// The uncertainty of the starting node, minus the weighted impurity of
/// two child nodes.
fn info_gain<'a, T, I, J>(true_rows: I, false_rows: J, current: f64) -> f64
where
    T: DataRow,
    I: Iterator<Item = &'a &'a T>,
    J: Iterator<Item = &'a &'a T>,
    T: 'a,
{
    let (true_len, true_gini) = gini(true_rows);
    let (false_len, false_gini) = gini(false_rows);

    let p = true_len as f64 / (true_len + false_len) as f64;
    current - p * true_gini - (1.0 - p) * false_gini
}

/// Find the best way to split the data
fn find_best_split<T>(rows: &[&T]) -> (f64, Option<Question>)
where
    T: DataRow,
{
    assert!(rows.len() > 0);
    let mut best_gain = 0.0;
    let mut best_question = None;
    let (_, current_uncertainty) = gini(rows.iter());
    let first_row = rows[0];
    let nfeatures = rows[0].len();

    for col in 0..nfeatures {
        let values = column_values(rows.iter(), col);
        for val in values {
            let question = Question::new(first_row.name(col).clone(), col, val);
            let (mut true_rows, mut false_rows) = partition_iter(rows.iter(), &question);
            let mut true_rows = true_rows.peekable();
            let mut false_rows = false_rows.peekable();

            if true_rows.peek().is_none() || false_rows.peek().is_none() {
                continue;
            }

            // Calculate the information gain from this split.
            let gain = info_gain(true_rows, false_rows, current_uncertainty);
            if gain > best_gain {
                best_gain = gain;
                best_question = Some(question.clone());
            }
        }
    }

    (best_gain, best_question)
}

///
/// Build up our decision tree.
///
/// #Arguments
///
/// * `rows` - The rows of our dataset. Each row is an object that satisfies the DataRow trait.
/// * `depth` - The maximum depth of our tree. None if we want it as deep as we can.
///
pub fn build_tree<T>(rows: &[&T], depth: Option<u64>) -> Node
where
    T: DataRow,
{
    let depth = depth.map(|d| d - 1);
    match depth {
        Some(0) => Node::new_leaf(&rows), // We have gone as deep as we need
        _ => {
            let (gain, question) = find_best_split(&rows);
            if gain == 0.0 {
                Node::new_leaf(&rows)
            } else {
                // We have found a useful feature to partition on.
                let question = question.unwrap();
                let (true_rows, false_rows) = partition(&rows, &question);
                let true_branch = build_tree(&true_rows, depth);
                let false_branch = build_tree(&false_rows, depth);

                Node::new_decision(question, true_branch, false_branch)
            }
        }
    }
}

/// Takes the map of probabilities and chooses the most likely one.
fn prediction(classification: &HashMap<String, usize>) -> String {
    let (k, _v) = classification
        .iter()
        .max_by(|(_k1, v1), (_k2, v2)| v1.cmp(v2))
        .unwrap();
    k.to_owned()
}

pub fn classify<'a, T>(row: &T, node: &'a Node) -> String
where
    T: DataRow,
{
    match node {
        Node::Leaf { predictions } => prediction(predictions),
        Node::Decision {
            question,
            true_branch,
            false_branch,
        } => {
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
    use row::test::{training_data, Fruit};

    #[test]
    fn test_class_counts() {
        let data = training_data();
        let data: Vec<&Fruit> = data.iter().collect();
        let counts = class_counts(&data);
        let mut expected = HashMap::new();
        expected.insert("Lemon".to_string(), 1);
        expected.insert("Apple".to_string(), 2);
        expected.insert("Grape".to_string(), 2);

        assert_eq!(expected, counts);
    }

    #[test]
    fn test_class_counts_iter() {
        let data = training_data();
        let (count, counts) = class_counts_iter(data.iter());
        let mut expected = HashMap::new();
        expected.insert("Lemon".to_string(), 1);
        expected.insert("Apple".to_string(), 2);
        expected.insert("Grape".to_string(), 2);

        assert_eq!(5, count);
        assert_eq!(expected, counts);
    }

    #[test]
    fn test_gini_pure() {
        let p = vec![
            Fruit::new(1, "Red".to_string(), 3, "Apple".to_string()),
            Fruit::new(2, "Green".to_string(), 2, "Apple".to_string()),
        ];
        let p: Vec<&Fruit> = p.iter().collect();

        assert_eq!(0.0, gini(&p));
    }

    #[test]
    fn test_gini_pure() {
        let p = vec![
            Fruit::new(1, "Red".to_string(), 3, "Apple".to_string()),
            Fruit::new(2, "Green".to_string(), 2, "Apple".to_string()),
        ];

        assert_eq!(0.0, gini(p.iter()));
    }

    #[test]
    fn test_gini_impure() {
        let p = vec![
            Fruit::new(1, "Red".to_string(), 3, "Apple".to_string()),
            Fruit::new(2, "Red".to_string(), 3, "Orange".to_string()),
        ];
        let p: Vec<&Fruit> = p.iter().collect();

        assert_eq!(0.5, gini(&p));
    }

    #[test]
    fn test_gini_impure() {
        let p = vec![
            Fruit::new(1, "Red".to_string(), 3, "Apple".to_string()),
            Fruit::new(2, "Red".to_string(), 3, "Orange".to_string()),
        ];

        assert_eq!(0.5, gini(p.iter()));
    }

    #[test]
    fn test_column_values() {
        let data = training_data();
        let data: Vec<&Fruit> = data.iter().collect();
        let cols = column_values(&data, 1);
        assert_eq!(vec![Col::Int(1), Col::Int(3)], cols);
    }

    #[test]
    fn test_question_matches() {
        let q = Question::new("Colour".to_string(), 0, Col::Text("Red".to_string()));

        assert!(q.matchit(&Fruit::new(1, "Red".to_string(), 1, "toenails".to_string())));
        assert!(!q.matchit(&Fruit::new(2, "Green".to_string(), 1, "spleen".to_string())));

        let q = Question::new("Colour".to_string(), 1, Col::Int(42));
        assert!(q.matchit(&Fruit::new(
            1,
            "Red".to_string(),
            42,
            "toenails".to_string()
        )));
        assert!(!q.matchit(&Fruit::new(2, "Red".to_string(), 1, "spleen".to_string())));
    }

    #[test]
    fn test_partition() {
        let data = training_data();
        let data: Vec<&Fruit> = data.iter().collect();
        let q = Question::new("Colour".to_string(), 0, Col::Text("Red".to_string()));

        let (t, f) = partition(&data, &q);

        assert_eq!(2, t.len());
        assert_eq!(3, f.len());
    }
}
