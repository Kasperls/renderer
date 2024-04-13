use cgmath::{prelude::*, Vector3};
use std::error::Error;
use std::fs;
use std::io::{BufReader, BufRead};


// pub struct Room {
//     pub pos: Vector3<i32>,
//     /// [layers, line, col] 
//     pub size: [usize; 3],
//     pub data: Vec<i32>
// }

pub struct Room {
    // add more useful information about the room
    pub dim: Vec<i32>,
    pub layers: Vec<Vec<Vec<i32>>>,
}

impl Room {
    pub fn read_complete_csv(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = fs::File::open(path)?;
        let mut reader = BufReader::new(file);
    
        let mut buf = String::new();
        let _ = reader.read_line(&mut buf)?;
        let dim = buf.trim().split(" ").map(|elm| {
            elm.parse::<i32>().unwrap()
        }).collect::<Vec<i32>>();
    
        let data = reader.fill_buf()?
            .iter()
            .map(|&b| b as char)
            .collect::<String>()
            .split("\r\n")
            .map(|row| {
                row.trim().split(" ").map(|elm| {
                    elm.parse::<i32>().unwrap()
                }).collect::<Vec<i32>>()
            })
        .collect::<Vec<Vec<i32>>>();
        
        let mut layers = vec![];
        data.chunks(dim[1] as usize).for_each(|chunk| {
            layers.push(chunk.to_vec());
        });
        
        Ok(Self { dim, layers })
    }
    /// Reads data from csv file
    pub fn read_csv(path: &str) -> Result<(Vec<i32>, [usize; 2]), Box<dyn Error>> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut dim = [0, 0];
        let mut room_data: Vec<i32> = vec![];
        for result in reader.lines() {
            dim[0] += 1;
            let line = result?;
            let mut line_data = line.split(",").map(|n| {
                n.parse::<i32>().unwrap()
            }).collect::<Vec<i32>>();
    
            room_data.append(&mut line_data);
        }

        dim[1] = room_data.len() / dim[0];

        Ok((room_data, dim))
    }
    // improve later!! ADDS NO POSITION
    // pub fn from_folder(path: &str) -> Result<Self, Box<dyn Error>> {
    //     let mut room_data: Vec<i32> = vec![];
    //     let mut layers = 0;
    //     let mut dim_layer = [0, 0];
    //     for result in fs::read_dir(path)? {
    //         let entry = result?;
    //         let (mut layer_data, dim_layer_tmp) = Self::read_csv(entry.path().to_str().unwrap())?;
    //         dim_layer = dim_layer_tmp;
    //         room_data.append(&mut layer_data);
    //         layers += 1;
    //     }

    //     Ok(Self { dim: [layers, dim_layer[0], dim_layer[1]], data: room_data })
    // }
}
