module Decision

type Numeric = decimal

type Feature = 
| Number of Numeric
| Category of string

type Label = string
type Row = Feature list * Label
type Rows = Row list

type TestCondtion =
| GreaterThanEQ of number:Numeric
| Matches of txt:string

let testFromFeature = function 
                      | Number x   -> GreaterThanEQ x
                      | Category x -> Matches x

type Test = int * TestCondtion

let passTest ((col,test):Test) (row:Row) =
    match (fst row).[col] with
    | Number num ->
        match test with
        | Matches _ -> failwith "Cannot match a number"
        | GreaterThanEQ x -> num >= x
    | Category cat ->
        match test with
        | GreaterThanEQ _ -> failwith "Cannot be greater than category"
        | Matches x -> cat = x

type Tree =
    | Decision of test:Test * positive:Tree * negative:Tree
    | Leaf of prediction:Rows

let uniqueValuesForColumn (items:Rows) (col:int) : Feature List =
    items |> List.map (fst >> List.item col ) |> List.distinct

let uniqueTestsForColumn (items:Rows) (col:int) : Test List =
    items |> List.map (fst >> List.item col ) |> List.distinct |> List.map (fun i -> col, testFromFeature i )

let labelCounts (items:Rows) = items |> List.countBy snd

let partitionByTest test rows = rows |> List.partition (passTest test)

let gini (rows: Rows) =
    let lc = labelCounts rows
    let len = float rows.Length
    (1.0, lc) ||> List.fold (fun impurity (_,count) -> impurity - ( (float count) / len)**2.0 )

let infoGain (left:Rows) (right:Rows) current =
    let lenLeft = float left.Length
    let lenRight = float right.Length
    let leftWeighting = (lenLeft) / (lenLeft + lenRight)
    // The uncertainty passed in, minus the weighted impurity of two child nodes.
    current - ( leftWeighting * gini left ) - ( (1.0 - leftWeighting) * gini right )

let findBestTest (rows:Rows) =
    let currentUncert = gini rows
    let calcGain (test:Test) =
        let trueRows, falseRows = partitionByTest test rows
        if trueRows.Length    = 0 then 0.
        elif falseRows.Length = 0 then 0.
        else infoGain trueRows falseRows currentUncert

    let featureCount = (rows.Head |> fst).Length - 1
    [0..featureCount] 
    |> List.collect (uniqueTestsForColumn rows)  
    |> List.map    (fun t -> t, calcGain t)            // calc info gain
    |> List.rev     // TEMP to get same answer as python
    |> List.maxBy snd

let rec buildTree (rows:Rows) : Tree =
    let test, gain = findBestTest rows
    if gain = 0. then Leaf rows                  // No gain, no useful questions, so we are done
    else
        let trueRows, falseRows = partitionByTest test rows
        Decision(test, buildTree trueRows, buildTree falseRows)

let printTree (fields:string []) tree =
    let rec print i = function
                  | Leaf x ->
                    printfn "%s Predicts %A" (String.replicate i "--") (x |> labelCounts) 
                  | Decision((col,test),left,right) ->
                    let indent = (String.replicate i "--")
                    let test = match test with
                               | Matches a       -> sprintf "Is %s is %s ?" fields.[col] a
                               | GreaterThanEQ a -> sprintf "Is %s >= %f ?" fields.[col] a
                    printfn "%s%s" indent test
                    printfn "%s-> True:" indent
                    print (i+1) left
                    printfn "%s -> False:" indent
                    print (i+1) right
    print 0 tree

let rec classify (tree:Tree) (row:Row) =
    match tree with
    | Leaf rows ->
        let lc = labelCounts rows
        let total = lc |> List.sumBy snd |> float
        lc |> List.map (fun (item,cnt) -> item, (float cnt) / total )

    | Decision(test,left,right) ->
        if passTest test row then
            classify left row
        else
            classify right row

let niceClassify t r = 
    classify t r |> List.map (fun (item,perc) -> sprintf "%s: %i%%" item (perc * 100. |> int) )
    |> String.concat ", "
