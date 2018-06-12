use tree::{Node, classify};
use row::DataRow;

/// Split the dataset into the training and test rows.
/// folds is the number of folds we split the dataset up into.
/// Current is the number of the current testing fold.
fn fold_dataset<'a, T>(rows: &Vec<&'a T>, folds: usize, current: usize) -> (Vec<&'a T>, Vec<&'a T>) {
    let mut result = (Vec::new(), Vec::new());
    
    rows.iter().enumerate().for_each(|(idx, row)| {
        if idx % folds == current {
            result.1.push(*row);
        } else {
            result.0.push(*row);
        }
    });
    
    result
}

/// Cross validates the dataset with the given training function
pub fn validate<T, F>(rows: &Vec<&T>, folds: usize, train_fn: F ) -> f64 
where T: DataRow,
      F: Fn(Vec<&T>) -> Node {

    let mut score = 0.0;
    
    for test in 0..folds {
        let (train, test) = fold_dataset(rows, folds, test);

        // Build the tree with our training folds.
        let tree = train_fn(train);

        let correct = test.iter().filter(|t| {
            t.label() == classify(**t, &tree)
        }).count();
    
        score += correct as f64 / test.len() as f64;
    }
    
    score / folds as f64
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_fold() {
        let data = vec![&1, &2, &3, &4, &5, &6];
        let folded = fold_dataset(&data, 3, 2);
        
        assert_eq!((vec![&1, &2, &4, &5],
                    vec![&3, &6]), folded);
    }
 
}
