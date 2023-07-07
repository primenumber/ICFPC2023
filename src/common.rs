use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Attendee {
    pub x: f64,
    pub y: f64,
    pub tastes: Vec<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Problem {
    pub room_width: f64,
    pub room_height: f64,
    pub stage_width: f64,
    pub stage_height: f64,
    pub stage_bottom_left: Vec<f64>,
    pub musicians: Vec<u32>,
    pub attendees: Vec<Attendee>,
}

impl Problem {
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        Ok(serde_json::from_reader(reader)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn norm(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Solution {
    pub placements: Vec<Point>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Submission {
    pub problem_id: u32,
    pub contents: String,
}

impl Solution {
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let f = File::create(path)?;
        let writer = BufWriter::new(f);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    #[tokio::main]
    pub async fn submit(&self, id: u32, token: &str) -> Result<()> {
        let client = Client::new();
        let url = "https://api.icfpcontest.com/submission";
        let body = Submission {
            problem_id: id,
            contents: serde_json::to_string(self)?,
        };
        let response = client
            .post(url)
            .json(&body)
            .bearer_auth(token)
            .send()
            .await?;
        let body = response.text().await?;
        println!("{}", body);
        Ok(())
    }
}
