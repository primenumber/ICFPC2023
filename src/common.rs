use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;

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

#[tokio::main]
pub async fn download_problems(id_from: u32, id_to: u32, output: &PathBuf) {
    let client = Arc::new(Client::new());
    let fut = futures::future::join_all(
        (id_from..=id_to)
            .into_iter()
            .map(|id| {
                let client = client.clone();
                let path = output.join(format!("{}.json", id));
                (id, client, path)
            })
            .map(|(id, client, path)| async move {
                let url = format!("https://cdn.icfpcontest.com/problems/{}.json", id);
                let response = client.get(url).send().await?;
                let body = response.text().await?;
                let f = File::create(path)?;
                let mut writer = BufWriter::new(f);
                write!(writer, "{}", body)?;
                Result::<()>::Ok(())
            }),
    );
    let _ = tokio::spawn(fut).await;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn length(&self) -> f64 {
        self.norm().sqrt()
    }

    pub fn norm(&self) -> f64 {
        self.dot(*self)
    }

    pub fn dot(&self, rhs: Point) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn normalize(&self) -> Point {
        (1. / self.length()) * *self
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

impl std::ops::Mul<Point> for f64 {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        Point {
            x: self * rhs.x,
            y: self * rhs.y,
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
