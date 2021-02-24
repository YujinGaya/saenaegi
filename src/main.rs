mod student;
mod subject;

use std::collections::{BTreeMap, HashMap};

use rand::prelude::*;

use student::{Status, Student};
use subject::Division;

// 총 731명
// const SUBJECTS: &[&str] = &[
//     "CS101",  // 프밍기
//     "BS120",  // 일생
//     "CH101",  // 일화
//     "PH141",  // 일물
//     "PH161",  // 고급물
//     "PH171",  // 체감형물
//     "MAS101", // 미적1
//     "CH102",  // 일화실
//     "PH151",  // 일물실
//     "HSS010", // Int.SL
//     "HSS011", // Int.RW
//     "HSS023", // Adv.L
//     "HSS022", // Adv.S
//     "HSS025", // Adv.R
//     "HSS024", // Adv.W
// ];

fn main() {
    let mut rng = thread_rng();
    let rng = &mut rng;

    let mut subjects = Division::read_csv();
    let mut students = Student::read_csv();

    for division in subjects.values().flat_map(|divisions| divisions.values()) {
        assert_eq!(division.assigned, 0);
    }

    // 외국인, 영어 * 4, 고물, 실물, 일생 배정 정원 반영 및 수정
    for student in students.values() {
        let assigned_subjects: Vec<&Division> = student
            .subjects
            .iter()
            .filter_map(|(subject_id, status)| {
                if let Status::Enroll(division) = status {
                    Some(&subjects[subject_id][division])
                } else {
                    None
                }
            })
            .collect();

        for i in 0..assigned_subjects.len() {
            for j in i + 1..assigned_subjects.len() {
                assert!(assigned_subjects[i].is_disjoint(assigned_subjects[j]));
            }
        }

        for (subject, status) in &student.subjects {
            if let Status::Enroll(division) = status {
                subjects
                    .get_mut(subject)
                    .unwrap()
                    .get_mut(division.trim())
                    .unwrap()
                    .assigned += 1;
            }
        }
    }

    // HSS022
    {
        subjects
            .get_mut("HSS022")
            .unwrap()
            .get_mut("A")
            .unwrap()
            .quota = 25;
        subjects
            .get_mut("HSS022")
            .unwrap()
            .get_mut("C")
            .unwrap()
            .quota = 25;

        let division_c: Vec<_> = Division::read_hss022_students()
            .choose_multiple(rng, 25)
            .collect();

        for &student in Division::read_hss022_students() {
            assign(
                students.get_mut(student).unwrap(),
                "HSS022",
                if division_c.contains(&&student) {
                    "C"
                } else {
                    "A"
                },
                &mut subjects,
            );
        }
    }

    // CS101
    {
        let mut cs101_students = students
            .values_mut()
            .filter(|student| student.subjects["CS101"] == Status::Undefined)
            .collect::<Vec<_>>();
        cs101_students.shuffle(rng);

        let mut count = 1;
        'next_student: for student in cs101_students {
            count += 1;
            for division in 'A'..='J' {
                let division = division.to_string();
                if student
                    .subjects(&subjects)
                    .iter()
                    .all(|div| div.is_disjoint(&subjects["CS101"][&division]))
                    && subjects["CS101"][&division].quota > subjects["CS101"][&division].assigned
                {
                    assign(student, "CS101", &division, &mut subjects);
                    continue 'next_student;
                }
            }
            panic!("망함.. {} {:?}", count, student);
        }

        println!(
            "Assigned {} students to CS101",
            students
                .values()
                .filter(|student| matches!(student.subjects["CS101"], Status::Enroll(_)))
                .count()
        );
    }

    // MAS101, CH101
    {
        for division_char in 'A'..='P' {
            if division_char == 'K' || division_char == 'L' {
                continue;
            }
            let division_id = division_char.to_string();
            let division = &subjects["MAS101"][&division_id];

            for student in students
                .values_mut()
                .filter(|s| {
                    matches!(s.subjects["MAS101"], Status::Undefined)
                        && s.subjects(&subjects)
                            .iter()
                            .all(|d| d.is_disjoint(division))
                })
                .choose_multiple(rng, (division.quota - division.assigned) as usize)
            {
                assign(student, "MAS101", &division_id, &mut subjects);

                let mut candidates = match division_char {
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' => vec!["A", "B", "C", "D"],
                    'I' | 'J' | 'M' | 'N' | 'O' | 'P' => vec!["E", "F", "G", "H"],
                    _ => unreachable!(),
                };

                candidates.retain(|&d| subjects["CH101"][d].quota > subjects["CH101"][d].assigned);
                // println!("{:?}", candidates);

                assign(
                    student,
                    "CH101",
                    candidates
                        .choose(rng)
                        .expect(&format!("{:?} {:?}", subjects["MAS101"], subjects["CH101"])),
                    &mut subjects,
                );
            }
        }

        for &subject_id in &["MAS101", "CH101"] {
            let count = students
                .values()
                .filter(|student| matches!(student.subjects[subject_id], Status::Enroll(_)))
                .count();
            assert_eq!(count, 728);
            println!("Assigned {} students to {}", count, subject_id);
        }

        // println!("{:?}", subjects["CH101"].iter().collect::<BTreeMap<_, _>>());
    }

    // PH141
    {
        for student in students.values_mut().filter(|student| {
            (student.is_taking("HSS011", "B")
                || student.is_taking("CS101", "C")
                || student.is_taking("CS101", "D")
                || student.is_taking("CS101", "G")
                || student.is_taking("CS101", "H"))
                && !student.is_taking("PH161", "고급물")
                && !student.is_taking("PH171", "체감형물")
        }) {
            let division = ["A", "B", "C"].choose(rng).unwrap();
            assign(student, "PH141", division, &mut subjects);
        }

        for student in students.values_mut().filter(|student| {
            matches!(student.subjects["PH141"], Status::Undefined)
                && matches!(student.subjects["BS120"], Status::Enroll(_))
        }) {
            let mut division = ["D", "E", "F", "G"].choose(rng).unwrap();

            while subjects["PH141"][*division].assigned >= subjects["PH141"][*division].quota {
                division = ["D", "E", "F", "G"].choose(rng).unwrap();
            }

            assign(student, "PH141", division, &mut subjects);
        }

        for &division in &["A", "B", "C"] {
            for student in students
                .values_mut()
                .filter(|student| matches!(student.subjects["PH141"], Status::Undefined))
                .choose_multiple(
                    rng,
                    (subjects["PH141"][division].quota - subjects["PH141"][division].assigned)
                        as usize,
                )
            {
                assign(student, "PH141", division, &mut subjects);
            }
        }

        for &division in &["A", "B", "C", "D", "E", "F", "G"] {
            for student in students
                .values_mut()
                .filter(|student| {
                    matches!(student.subjects["PH141"], Status::Undefined)
                        && student
                            .subjects(&subjects)
                            .iter()
                            .all(|d| d.is_disjoint(&subjects["PH141"][division]))
                })
                .choose_multiple(
                    rng,
                    (subjects["PH141"][division].quota - subjects["PH141"][division].assigned)
                        as usize,
                )
            {
                assign(student, "PH141", division, &mut subjects);
            }
        }

        println!(
            "Assigned {} students to PH141, PH161, PH171",
            students
                .values()
                .filter(
                    |student| matches!(student.subjects["PH141"], Status::Enroll(_))
                        || matches!(student.subjects["PH161"], Status::Enroll(_))
                        || matches!(student.subjects["PH171"], Status::Enroll(_))
                )
                .count()
        );

        for s in students.values().filter(|student| {
            !(matches!(student.subjects["PH141"], Status::Enroll(_))
                || matches!(student.subjects["PH161"], Status::Enroll(_))
                || matches!(student.subjects["PH171"], Status::Enroll(_)))
        }) {
            println!("{:?}", s.subjects.iter().collect::<BTreeMap<_, _>>());
        }
    }

    // HSS010
    {
        let to_be = students
            .values()
            .filter(|s| s.subjects["HSS010"] == Status::Undefined)
            .count();

        for &division_id in [
            "A", "D", "B", "E", "C", "F", //
            "N", "O", "P", //
            "T", "V", "U", "W", "AB", //
            "G", "H", "I", "J", //
            "K", "L", "M", "Y", //
            "Q", "R", "S", "Z", "AA", //
        ]
        .iter()
        {
            let division = &subjects["HSS010"][division_id];
            for student in students
                .values_mut()
                .filter(|s| {
                    matches!(s.subjects["HSS010"], Status::Undefined)
                        && s.subjects(&subjects)
                            .iter()
                            .all(|d| d.is_disjoint(division))
                        && matches!(s.subjects["CS101"], Status::Enroll(_))
                        && !s.is_taking("CS101", "I")
                        && !s.is_taking("CS101", "J")
                })
                .choose_multiple(rng, (division.quota - division.assigned) as usize)
            {
                assign(student, "HSS010", &division_id, &mut subjects);
            }
        }

        for &division_id in [
            "D", "A", "E", "B", "F", "C", //
            "T", "U", "AB", //
            "Q", "R", "S", "Z", "AA", //
            "K", "L", "M", "Y", //
            "G", "H", "I", "J", //
            "V", "W", //
            "N", "O", "P", //
        ]
        .iter()
        {
            let division = &subjects["HSS010"][division_id];
            for student in students
                .values_mut()
                .filter(|s| {
                    matches!(s.subjects["HSS010"], Status::Undefined)
                        && s.subjects(&subjects)
                            .iter()
                            .all(|d| d.is_disjoint(division))
                })
                .choose_multiple(rng, (division.quota - division.assigned) as usize)
            {
                assign(student, "HSS010", &division_id, &mut subjects);
            }
        }

        println!(
            "Assigned {}/{} students to HSS010",
            subjects["HSS010"].values().map(|d| d.assigned).sum::<i32>(),
            to_be,
        );
    }

    {
        for &division_id in &[
            "V", "J", 
            "W", "K", 
            "X", "L",

            "D", "G", "P", "S",
            "E", "H", "Q", "T",
            "F", "I", "R", "U",

            "M", "A",
            "N", "B",
            "O", "C",

            "AA", "Y", "AB", "Z", "AC", //
        ] {
            let division = &subjects["CH102"][division_id];

            for student in students
                .values_mut()
                .filter(|s| {
                    matches!(s.subjects["CH102"], Status::Undefined)
                        && s.subjects(&subjects)
                            .iter()
                            .all(|d| d.is_disjoint(division))
                        && s.subjects["CS101"] != Status::No
                })
                .choose_multiple(rng, (division.quota - division.assigned) as usize)
            {
                assign(student, "CH102", &division_id, &mut subjects);
            }
        }

        for &division_id in &[
            "M", "V", "J", "A",
            "N", "W", "K", "B",
            "O", "X", "L", "C",

            "D", "G", "P", "S",
            "E", "H", "Q", "T",
            "F", "I", "R", "U",

            "AA", "Y", "AB", "Z", "AC", //
        ] {
            let division = &subjects["CH102"][division_id];

            for student in students
                .values_mut()
                .filter(|s| {
                    matches!(s.subjects["CH102"], Status::Undefined)
                        && s.subjects(&subjects)
                            .iter()
                            .all(|d| d.is_disjoint(division))
                })
                .choose_multiple(rng, (division.quota - division.assigned) as usize)
            {
                assign(student, "CH102", &division_id, &mut subjects);
            }
        }

        println!(
            "Assigned {}/{} students to CH102",
            subjects["CH102"].values().map(|d| d.assigned).sum::<i32>(),
            students.len(),
        );
    }

    for &div in &["A", "C", "F", "H", "K", "M", "P", "R", "T", "W"] {
        println!(
            "{} {} + {}",
            div,
            students
                .values()
                .filter(|s| s
                    .subjects(&subjects)
                    .iter()
                    .all(|d| d.is_disjoint(&subjects["PH151"][div]))
                    && s.subjects["PH151"] == Status::Undefined
                    && s.subjects["CS101"] != Status::No
                )
                .count(),
            students
                .values()
                .filter(|s| s
                    .subjects(&subjects)
                    .iter()
                    .all(|d| d.is_disjoint(&subjects["PH151"][div]))
                    && s.subjects["PH151"] == Status::Undefined
                    && s.subjects["CS101"] == Status::No
                )
                .count()
        );
    }

    {
        let to_be = subjects["PH151"].values().map(|d| d.quota).sum::<i32>();

        for &division_id in &[
            "K", "L", "M", //
            "N", "O", //
            "A", "B", //
            "F", "G", //
            "P", "Q", //
            "C", "D", "E", //
            "T", "U", "V", //
            "W", "X", "Y", //
            "H", "I", "J", //
            "R", "S", //
        ] {
            let division = &subjects["PH151"][division_id];

            for student in students
                .values_mut()
                .filter(|s| {
                    matches!(s.subjects["PH151"], Status::Undefined)
                        && s.subjects(&subjects)
                            .iter()
                            .all(|d| d.is_disjoint(division))
                        && s.subjects["CS101"] == Status::No
                })
                .choose_multiple(rng, (division.quota - division.assigned) as usize)
            {
                assign(student, "PH151", &division_id, &mut subjects);
            }
        }
        for &division_id in &[
            "K", "L", "M", //
            "N", "O", //
            "C", "D", "E", //
            "A", "B", //
            "H", "I", "J", //
            "R", "S", //
            "T", "U", "V", //
            "W", "X", "Y", //
            "F", "G", //
            "P", "Q", //
        ] {
            let division = &subjects["PH151"][division_id];

            for student in students
                .values_mut()
                .filter(|s| {
                    matches!(s.subjects["PH151"], Status::Undefined)
                        && s.subjects(&subjects)
                            .iter()
                            .all(|d| d.is_disjoint(division))
                })
                .choose_multiple(rng, (division.quota - division.assigned) as usize)
            {
                assign(student, "PH151", &division_id, &mut subjects);
            }
        }

        println!(
            "Assigned {}/{} students to PH151",
            subjects["PH151"].values().map(|d| d.assigned).sum::<i32>(),
            to_be,
        );

        for (_, subject) in subjects["PH151"].iter().collect::<BTreeMap<_, _>>().iter() {
            println!("{} {}", subject.division, subject.assigned);
        }
    }

    let mut students = students.drain().map(|(_, s)| s).collect::<Vec<_>>();
    students.sort_by_key(|s| s.id.clone());
    Student::print_csv(&students);
}

fn assign(
    student: &mut Student,
    subject_id: &str,
    division_id: &str,
    subjects: &mut HashMap<String, HashMap<String, Division>>,
) {
    let division = &subjects[subject_id][division_id];
    assert!(student
        .subjects(subjects)
        .iter()
        .all(|d| d.is_disjoint(division)));
    assert!(
        division.assigned < division.quota,
        "{} {}",
        subject_id,
        division_id
    );
    assert!(
        matches!(student.subjects[&division.id], Status::Undefined),
        "s {:?}",
        student
    );

    subjects
        .get_mut(subject_id)
        .unwrap()
        .get_mut(division_id)
        .unwrap()
        .assigned += 1;
    *student.subjects.get_mut(subject_id).unwrap() = Status::Enroll(division_id.to_owned());
}
