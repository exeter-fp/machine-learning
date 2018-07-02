#load "decision.fs"
open Decision

#r "packages/FSharp.Data/lib/net45/FSharp.Data.dll"
open FSharp.Data

type TrainingFile = CsvProvider<"train.csv">

let trainData: Rows  = 

    let toModel (l:TrainingFile.Row) =
        let numF = decimal >> Number
        [ 
            numF l.Pclass
            Category l.Sex
            (try Number (decimal l.Age) with _ -> Number 0M )
            numF l.SibSp
            numF l.Parch
            //Category l.Ticket
            Number l.Fare
            //Category l.Cabin
            Category l.Embarked
            ], ( if l.Survived then "Survived" else "Died" )

    TrainingFile.Parse( System.IO.File.ReadAllText("train.csv") ).Rows
    |> Seq.map toModel |> List.ofSeq

trainData 
|> buildTree
|> printTree [| 
    "PClass"
    "Sex"
    "Age"
    "SibSp"
    "Patch"
    //"Ticket"
    "Fare"
    //"Cabin"
    "Embarked" |]

// TODO:
// - Add computed categories like HasCabin, FamilySize
// - Cross validate
// - Penalise overfitting, need to have some measurement of generality ?
