
use std::fs;
use std::path::Path;
use std::error::Error;

use crate::locations::*;

pub fn processLocations(locations : Locations) {
    for location in &locations.locations {
        if checkLocation(location) {
            let _ = processLocation(location)
                .map_err(|err| {error!("Error processing location: {:?}, message: {:?}",
                    location, err);});
        }
    }
}

fn checkLocation(location : &Location) -> bool {
    Path::new(&location.file.input).exists() &&
        Path::new(&location.file.processing).exists()
}

fn processLocation(location : &Location) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(&location.file.input)? {
        match entry {
            Ok(item) => {
                let _ = processItem(&item, location)
                .map_err(|err| {error!("Error processing item: {:?}, message: {:?}",
                    item, err);});
                ()
            },
            Err(error) => error!("Error occurred in [{:?}], message: {:?}",
                &location.file.input,
                error)
        }
    }

    Ok(())
}

fn processItem(item : &fs::DirEntry, location : &Location) -> Result<(), Box<dyn Error>> {
    let path = item.path();

    let metadata = fs::metadata(&path)?;

    Ok(())
}
