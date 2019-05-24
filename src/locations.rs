
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
    pub current_dir: Option<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Locations {
    pub locations : Vec<Location>
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

        assert_eq!(1000, loc.locations[0].readinessDelay);
        assert_eq!("command", loc.locations[0].process);
        assert_eq!(true, loc.locations[0].shell_command);
        assert_eq!(false, loc.locations[0].processing_timestamp);
        assert_eq!(Some("current_dir".to_owned()), loc.locations[0].current_dir);
        assert_eq!(FileTask {
            input:"input".to_string(),
            processing: "processing".to_string(),
            completed: Some("completed".to_string()),
            failed: Some("failed".to_string())
        }, loc.locations[0].file);
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
