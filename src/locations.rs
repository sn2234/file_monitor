
use std::error::Error;
use std::result::Result;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::de::{Deserialize, Deserializer};
use serde_json::Value;



#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct FileTask {
    pub input : String,
    pub processing : String,
    pub completed : Option<String>,
    pub failed : Option<String>
}

fn normalizePath<P>(path: P) -> PathBuf
    where
        P : AsRef<Path> + Copy
{

    let mut buffer = PathBuf::new();

    for comp in path.as_ref().components() {
        buffer.push(comp);
    }

    buffer
}

impl<'de> Deserialize<'de> for FileTask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        fn join(root: &str, leaf: &str) -> Option<String> {
            let canonicalPath = normalizePath(root)
                .join(leaf)
                .to_str()
                .map(|x| x.to_owned());
            
            canonicalPath
        }

        let nodeValue:Value  = Deserialize::deserialize(deserializer)?;

        if let Some(pathToRoot) = nodeValue.as_str() {
            return Ok(FileTask {
                input: join(&pathToRoot, "input")
                    .ok_or_else(|| serde::de::Error::custom("Bad input path"))?,
                processing: join(&pathToRoot, "processing")
                    .ok_or_else(|| serde::de::Error::custom("Bad processing path"))?,
                completed: None,
                failed: None
            });
        } else if let Some(nodeObject) = nodeValue.as_object() {

            let extractField = |fieldName| nodeObject
                .get(fieldName)
                .map(|value| value.as_str())
                .and_then(|x| x)
                .map(|x| x.to_owned())
                .ok_or_else(|| -> D::Error {serde::de::Error::missing_field(fieldName)});

            Ok(FileTask {
                input: extractField("input")?,
                processing: extractField("processing")?,
                completed: extractField("completed").ok(),
                failed: extractField("failed").ok()
            })
        } else {
            return Err(serde::de::Error::custom("unable to deserialize FileTask"));
        }
    }
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
        let loc = Locations::fromFile("./test_data/locations_test.json").unwrap();

        assert_eq!(5, loc.locations.len());

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

        let expected = FileTask {
            input:normalizePath("/test/path/to/folder/input").to_str().unwrap().to_owned(),
            processing:normalizePath("/test/path/to/folder/processing").to_str().unwrap().to_owned(),
            completed: None,
            failed: None
        };

        assert_eq!(expected, loc.locations[3].file);
        assert_eq!(expected, loc.locations[4].file);
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(Path::new(""), normalizePath(""));
        assert_eq!(Path::new(&format!("{}", std::path::MAIN_SEPARATOR)),
            normalizePath(&format!("{}", std::path::MAIN_SEPARATOR)));
        assert_eq!(Path::new("abc"), normalizePath("abc"));
        assert_eq!(Path::new(".abc"), normalizePath(".abc"));
        assert_eq!(Path::new(&format!("abc{}xxx", std::path::MAIN_SEPARATOR)),
            normalizePath(&format!("abc{}xxx", std::path::MAIN_SEPARATOR)));
        //assert_eq!(Path::new(&format!("abc{}xxx", std::path::MAIN_SEPARATOR)),
        //    normalizePath("abc\\xxx"));
        assert_eq!(Path::new(&format!("abc{}xxx", std::path::MAIN_SEPARATOR)),
            normalizePath("abc/xxx"));
    }
}
