use std::{collections::HashMap, fmt, fs};

use crate::subject::Division;

const PATH: &str = "./data/csv/fin02.csv";

#[derive(Debug, Eq, PartialEq)]
pub struct Student {
    pub id: String,
    pub name: String,
    pub gender: String,
    pub nationality: String,
    pub high_school: String,
    pub high_school_category: String,
    pub subjects: HashMap<String, Status>,
}

const SUBJECTS: &[&str] = &[
    "CS101",  // 프밍기
    "BS120",  // 일생
    "CH101",  // 일화
    "PH141",  // 일물
    "PH161",  // 고급물
    "PH171",  // 체감형물
    "MAS101", // 미적1
    "CH102",  // 일화실
    "PH151",  // 일물실
    "HSS010", // Int.SL
    "HSS011", // Int.RW
    "HSS023", // Adv.L
    "HSS022", // Adv.S
    "HSS025", // Adv.R
    "HSS024", // Adv.W
];

#[derive(Debug, Eq, PartialEq)]
pub enum Status {
    Enroll(String),
    No,
    Undefined,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Enroll(s) => if s == "" { " " } else { s },
                Self::No => "-",
                Self::Undefined => "?",
            }
        )
    }
}

impl Student {
    pub fn read_csv() -> HashMap<String, Student> {
        let mut reader = csv::Reader::from_path(PATH).unwrap();
        reader
            .records()
            .map(|r| {
                let r = r.unwrap();
                (
                    r[1].to_owned(),
                    Student {
                        id: r[1].to_owned(),
                        name: r[2].to_owned(),
                        gender: r[4].to_owned(),
                        nationality: r[6].to_owned(),
                        high_school_category: r[7].to_owned(),
                        high_school: r[8].to_owned(),
                        subjects: {
                            let mut map = HashMap::new();
                            for i in 9..24 {
                                map.insert(
                                    SUBJECTS[i - 9].to_owned(),
                                    match &r[i] {
                                        "Undefined" => Status::Undefined,
                                        "-" => Status::No,
                                        s => Status::Enroll(s.trim().to_owned()),
                                    },
                                );
                            }
                            map
                        },
                    },
                )
            })
            .collect()
    }

    pub fn subjects<'a>(
        &'a self,
        subjects: &'a HashMap<String, HashMap<String, Division>>,
    ) -> Vec<&'a Division> {
        self.subjects
            .iter()
            .filter_map(|(subject_id, status)| match status {
                Status::Enroll(division) => Some(&subjects[subject_id][division]),
                _ => None,
            })
            .collect()
    }

    pub fn is_taking(&self, id: &str, division: &str) -> bool {
        match &self.subjects[id] {
            Status::Enroll(s) => s.trim() == division,
            _ => false,
        }
    }

    pub fn print_csv(students: &[Student]) {
        let mut buf = "학번, 이름, 성별, 국적, 고교구분, 출신고교, 프밍기, 일생, 일화, 일물, 고급물리, 실험물리, 미적, 일화실, 일물실, 010, 011, 023, 022, 025, 024\n".to_string();
        for student in students {
            buf.push_str(&format!(
                "{:?},{:?},{:?},{:?},{:?},{:?},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                student.id,
                student.name,
                student.gender,
                student.nationality,
                student.high_school_category,
                student.high_school,
                student.subjects["CS101"],
                student.subjects["BS120"],
                student.subjects["CH101"],
                student.subjects["PH141"],
                student.subjects["PH161"],
                student.subjects["PH171"],
                student.subjects["MAS101"],
                student.subjects["CH102"],
                student.subjects["PH151"],
                student.subjects["HSS010"],
                student.subjects["HSS011"],
                student.subjects["HSS023"],
                student.subjects["HSS022"],
                student.subjects["HSS025"],
                student.subjects["HSS024"],
            ));
        }
        fs::write("./output.csv", buf).unwrap();
    }
}
