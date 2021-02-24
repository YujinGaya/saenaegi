use std::collections::HashMap;

use de::Unexpected;
use serde::{de, Deserialize, Deserializer};

const PATH: &str = "./data/csv/1. 2021년 봄학기 기초필수교과목.csv";
const PATH_ENGLISH: &str = "./data/csv/3. 영어과목 반배정 - 과목리스트.csv";

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Division {
    #[serde(rename = "과목번호")]
    pub id: String,
    #[serde(rename = "과목명")]
    pub name: String,
    #[serde(rename = "분반")]
    pub division: String,
    #[serde(rename = "영어", deserialize_with = "bool_from_string")]
    pub english: bool,
    #[serde(default)]
    pub assigned: i32,
    #[serde(rename = "정원")]
    pub quota: i32,
    #[serde(rename = "강의시간", deserialize_with = "intervals_from_string")]
    pub hour: Vec<[i32; 2]>,
}

impl Division {
    pub fn read_csv() -> HashMap<String, HashMap<String, Division>> {
        let mut map: HashMap<String, HashMap<String, Division>> = HashMap::new();

        for division in csv::Reader::from_path(PATH)
            .unwrap()
            .deserialize()
            .chain(csv::Reader::from_path(PATH_ENGLISH).unwrap().deserialize())
        {
            let division: Division = division.unwrap();
            map.entry(division.id.trim().to_owned())
                .or_default()
                .insert(division.division.trim().to_owned(), division);
        }

        map
    }

    pub fn is_disjoint(&self, rhs: &Self) -> bool {
        disjoint_intervals(&self.hour, &rhs.hour)
    }

    pub fn read_hss022_students() -> &'static [&'static str] {
        &[ /* REDACTED */ ]
    }
}

// ref: https://github.com/serde-rs/serde/issues/1344
fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_ref() {
        "Y" => Ok(true),
        " " => Ok(false),
        other => Err(de::Error::invalid_value(
            Unexpected::Str(other),
            &"OK or nOK",
        )),
    }
}

fn intervals_from_string<'de, D>(deserializer: D) -> Result<Vec<[i32; 2]>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(parse_intervals(String::deserialize(deserializer)?.as_ref()))
}

fn parse_intervals(s: &str) -> Vec<[i32; 2]> {
    s.split("\r\n").map(parse_interval).collect()
}

fn parse_interval(s: &str) -> [i32; 2] {
    let mut s = s.split_whitespace();

    let day = match s.next().unwrap() {
        "월" => 0,
        "화" => 1,
        "수" => 2,
        "목" => 3,
        "금" => 4,
        _ => unreachable!(),
    } * 24
        * 60;

    let mut moment = s.next().unwrap().split('~').map(|t| {
        t.split(':')
            .map(|n| n.parse::<i32>().unwrap())
            .fold(0, |acc, t| acc * 60 + t)
    });
    [day + moment.next().unwrap(), day + moment.next().unwrap()]
}

fn disjoint_intervals(lhs: &[[i32; 2]], rhs: &[[i32; 2]]) -> bool {
    lhs.iter()
        .all(|lhs| rhs.iter().all(|rhs| disjoint_interval(lhs, rhs)))
}

fn disjoint_interval(lhs: &[i32; 2], rhs: &[i32; 2]) -> bool {
    lhs[1] <= rhs[0] || rhs[1] <= lhs[0]
}

/// students must be less than or equal to the sum of the quota
// pub fn distribute_students(quotas: &[i32], students: [i32; 6]) -> Vec<[i32; 6]> {
//     let quotas_sum: i32 = quotas.iter().sum();
//     let students_sum: i32 = students.iter().sum();

//     assert!(quotas_sum >= students_sum);

//     let dist = quotas
//         .iter()
//         .map(|quota| {
//             let f = |i| (students[i] as f64 / students_sum as f64 * *quota as f64).floor() as i32;
//             [f(0), f(1), f(2), f(3), f(4), f(5)]
//         })
//         .collect();

//     dist
// }

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_interval_simple() {
        assert_eq!(parse_intervals("월 09:00~12:00"), vec![[9 * 60, 12 * 60]]);
        assert_eq!(
            parse_intervals("화 09:00~12:00"),
            vec![[(24 + 9) * 60, (24 + 12) * 60]]
        );
        assert_eq!(
            parse_intervals("월 14:30~16:00\r\n수 14:30~16:00\r\n금 13:00~14:00"),
            vec![
                [14 * 60 + 30, 16 * 60],
                [(24 * 2 + 14) * 60 + 30, (24 * 2 + 16) * 60],
                [(24 * 4 + 13) * 60, (24 * 4 + 14) * 60],
            ]
        );
    }

    #[test]
    fn disjoint_intervals_simple() {
        assert!(disjoint_intervals(
            &parse_intervals("월 09:00~12:00"),
            &parse_intervals("수 09:00~12:00")
        ));
        assert!(disjoint_intervals(
            &parse_intervals("월 14:30~16:00\r\n수 14:30~16:00\r\n금 13:00~14:00"),
            &parse_intervals("수 09:00~12:00")
        ));
        assert!(!disjoint_intervals(
            &parse_intervals("월 14:30~16:00\r\n수 14:30~16:00\r\n금 13:00~14:00"),
            &parse_intervals("월 12:30~14:00\r\n수 12:30~14:00\r\n금 13:00~14:00")
        ));
    }
}
