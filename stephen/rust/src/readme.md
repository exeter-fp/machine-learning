
To test the tree run :

```
cargo run -- -t train.csv -s test.csv -c check.csv
```
Currently we get a different answer every time from 77%-80%. No idea why..

To create a dot file of the tree :

```
cargo run -- -t train.csv -d titanic.dot 
cat titanic.dot | dot -Tpng >> titanic.png 
```

This is our tree :

![tree](titanic.png "The tree")


