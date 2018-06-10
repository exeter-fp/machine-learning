use std::collections::HashMap;
use std::error::Error;
use csv;
use row::{Col, DataRow};

#[derive(Debug, Deserialize)]
pub struct Titanic {
    id: i64,
    survived: Option<i64>,
    pclass: i64,
    name: String,
    sex: String,
    age: Option<f64>,
    sibsp: i64,
    parch: i64,
    ticket: String
}

impl DataRow for Titanic {
    fn name(&self, col: usize) -> String {
        (match col {
            0 => "Class",
            1 => "Sex",
            2 => "Age",
            3 => "Siblings",
            4 => "Parch",
            _ => panic!("Silly column")
        }).to_string()
    }

    fn value(&self, col: usize) -> Col {
        match col {
            0 => Col::Int(self.pclass),
            1 => Col::Text(self.sex.clone()),
            2 => self.age.map(|a| Col::Float(a)).unwrap_or(Col::Null),
            3 => Col::Int(self.sibsp),
            4 => Col::Int(self.parch),
            _ => panic!("Silly column")
        }
    }

    fn label(&self) -> String {
        (if self.survived.unwrap() == 1 {
            "Lived"
        } else {
            "Died"
        }).to_string()
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn len(&self) -> usize {
        5
    }
}

pub fn load_titanic(file: &str) -> Result<Vec<Titanic>, Box<Error>> {
    let mut rdr = csv::Reader::from_path(file)?;
    let mut result = Vec::new();
    for record in rdr.deserialize() { 
        let record: Titanic = record?;
        result.push(record);
    };
    
    Ok(result)
}

#[derive(Debug, Deserialize)]
pub struct Check {
    id: i64,
    survived: i64
}

pub fn load_check(file: &str) -> Result<HashMap<i64, String>, Box<Error>> {
    let mut result = HashMap::new();
    let mut rdr = csv::Reader::from_path(file)?;
    for record in rdr.deserialize() { 
        let record: Check = record?;
        result.insert(record.id, 
                      if record.survived == 0 {
                          "Died".to_string()
                      } else {
                          "Lived".to_string()
                      });
    };
    
    Ok(result)
}
