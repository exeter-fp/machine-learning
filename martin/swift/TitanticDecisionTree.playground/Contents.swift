import CreateML
import Foundation

extension MLDataTable {
    func exportCsv(to url: URL) {
        var csvText = ""

        csvText.append(contentsOf: columnNames.joined(separator: ","))
        csvText.append("\n")

        for row in rows {
            // Currently only works for `MLDataTable`s containing integer values
            // but that's all we need for the Titanic dataset results for now
            let rowValues = row.values.map { String($0.intValue!) }
            csvText.append(contentsOf: rowValues.joined(separator: ","))
            csvText.append("\n")
        }

        do {
            try csvText.write(to: url, atomically: true, encoding: .utf8)
        } catch {
            print("Error writing submission:")
            print("\(error)")
        }
    }

}

// ---------------------------------------
// -- 1. Read in training and test data --
// ---------------------------------------

let trainingCSV = URL(fileURLWithPath: "/Users/mlr/doc/devel/machine-learning/datasets/titanic/train.csv")
let testCSV = URL(fileURLWithPath: "/Users/mlr/doc/devel/machine-learning/datasets/titanic/test.csv")

var trainingData = try MLDataTable(contentsOf: trainingCSV)
var testData = try MLDataTable(contentsOf: testCSV)


// -----------------------
// -- 2. Data Cleansing --
// -----------------------

// Drop the 'Ticket' columm to avoid typing problems later
trainingData.removeColumn(named: "Ticket")
testData.removeColumn(named: "Ticket")

// Initially, drop the rows with empty ages
// later we'll fill them with something more useful
// like the median age
let ageColumn = trainingData["Age", Double.self]!
let agePresentMask = ageColumn.mapMissing {
    return $0 != nil
}
let cleanedTrainingTable = trainingData[agePresentMask]


// -------------------------------------
// -- 3. Classifier training & export --
// -------------------------------------

let survivalClassifier = try MLDecisionTreeClassifier(trainingData: cleanedTrainingTable, targetColumn: "Survived")
try survivalClassifier.write(to: URL(fileURLWithPath: "/Users/mlr/doc/devel/machine-learning/martin/swift/titanic.mlmodel"))



// ------------------------------
// -- 4. Test data predictions --
// ------------------------------

let testResults = try survivalClassifier.predictions(from: testData)



// ----------------------------------------------------------------
// -- 5. Create results table with PaxId and Survival Prediction --
// ----------------------------------------------------------------

var paxIds = testData["PassengerId", Int.self]!
var resultsTable = MLDataTable()
resultsTable.addColumn(paxIds, named: "PassengerId")
resultsTable.addColumn(testResults, named: "Survived")



// --------------------------
// -- 6. Export submission --
// --------------------------

let submissionUrl = URL(fileURLWithPath: "/Users/mlr/doc/devel/machine-learning/martin/swift/submission.csv")
resultsTable.exportCsv(to: submissionUrl)
