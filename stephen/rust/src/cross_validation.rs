use tree::{Node, classify};
use row::DataRow;

///
/// Split the dataset into the training and test rows.
///
/// #Arguments
///
/// * folds - the number of folds we split the dataset up into.
/// * current - the number of the current testing fold.
///
fn fold_dataset<'a, T>(rows: &Vec<&'a T>, folds: usize, current: usize) -> (Vec<&'a T>, Vec<&'a T>) {

    rows.iter()
        .enumerate()
        .fold((Vec::new(), Vec::new()), |mut result, (idx, row)| {
            if idx % folds == current {
                result.1.push(*row);
            } else {
                result.0.push(*row);
            }
            result
        })
}

///
/// Cross validates the dataset with the given training function.
/// Split the dataset up into a number of sections (specified by folds).
/// Then take the first section. Treat that as test data. Use the remaining
/// folds to train our decision tree. Test the tree against the test data.
/// See what portion of the test set we predicted correctly.
///
/// Repeat for each of the folds.
///
/// Take the score for each of them, average it out. This is our cross validation score.
///
/// #Arguments
///
/// * rows - The rows of our data set to train against.
/// * folds - The number of sections we should split our dataset into
/// * train_fn - A function we should call to return the decision tree.
pub fn validate<T, F>(rows: &Vec<&T>, folds: usize, train_fn: F ) -> f64 
where T: DataRow,
      F: Fn(Vec<&T>) -> Node {

    let score = (0..folds).fold(0.0, |score, test| {
        let (train, test) = fold_dataset(rows, folds, test);

        // Build the tree with our training folds.
        let tree = train_fn(train);

        // Test our tree against the test set.
        let correct = test.iter().filter(|t| {
            t.label() == classify(**t, &tree)
        }).count();
    
        score + correct as f64 / test.len() as f64
    });
    
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
