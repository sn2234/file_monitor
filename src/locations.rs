
use std::error::Error;
use std::result::Result;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileTask {
    pub input : String,
    pub processing : String,
    pub completed : Option<String>,
    pub failed : Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Location {
    pub file : FileTask,
    pub readinessDelay : u32,
    pub process : String,
    pub shell_command : bool,
    pub processing_timestamp : bool,
    pub complete_timestamp : bool,
    pub current_dir: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Locations {
    pub locations : Vec<Location>,
    pub polling_delay : u32
}

impl Locations {
        pub fn fromString(data: &str) -> Result<Self, Box<dyn Error>> {
        let cfg = serde_json::from_str(data)?;
        Ok(cfg)
    }

    pub fn fromFile<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let cfg = serde_json::from_reader(reader)?;
        Ok(cfg)
    }

    pub fn toString(&self) -> Result<String, Box<dyn Error>> {
        let s = serde_json::to_string(self)?;

        Ok(s)
    }

    pub fn toBytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let s = serde_json::to_vec(self)?;

        Ok(s)
    }

    pub fn toFile<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let buffer = File::create(path)?;
        serde_json::to_writer(buffer, self)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fromFile() {
        let loc = Locations::fromFile("locations_test.json").unwrap();

        assert_eq!(3, loc.locations.len());

         assert_eq!(Location {
            readinessDelay: 1000,
            process: "command".to_owned(),
            shell_command: true,
            processing_timestamp: false,
            complete_timestamp: false,
            current_dir: Some("current_dir".to_owned()),
            file: FileTask {
                input:"input".to_string(),
                processing: "processing".to_string(),
                completed: Some("completed".to_string()),
                failed: Some("failed".to_string())
            }
         }, loc.locations[0]);

        assert_eq!(FileTask {
            input:"input".to_string(),
            processing: "processing".to_string(),
            completed: None,
            failed: Some("failed".to_string())
        }, loc.locations[1].file);
        assert_eq!(FileTask {
            input:"input".to_string(),
            processing: "processing".to_string(),
            completed: Some("completed".to_string()),
            failed: None
        }, loc.locations[2].file);
    }
}
