
use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::{thread, time};
use std::process::Command;
use chrono::{DateTime, Local};

use crate::locations::{Location, Locations};

pub fn processLocations(locations : Locations) {
    for location in &locations.locations {
        if checkLocation(location) {
            let _ = processLocation(location)
                .map_err(|err| error!("Error processing location: {:?}, message: {:?}",
                    location, err));
        }
    }
}

fn checkLocation(location : &Location) -> bool {
    Path::new(&location.file.input).exists() &&
        Path::new(&location.file.processing).exists()
}

fn processLocation(location : &Location) -> Result<(), Box<dyn Error>> {
    info!("Processing location: {:?}", location);

    // Process items in input folder
    let _ = processFolder(&location.file.input, |item| processInputItem(&item, location))
                .map_err(|err| error!("Error processing folder: {:?}, message: {:?}",
                    location.file.input, err));

    // Process items in staging folder
    let _ = processFolder(&location.file.processing, |item| processStagingItem(&item, location))
                .map_err(|err| error!("Error processing folder: {:?}, message: {:?}",
                    location.file.input, err));

    Ok(())
}

fn processFolder<P, Fx>(path: P, processItem : Fx) ->
    Result<(), Box<dyn Error>>
    where
        P: AsRef<Path> + Copy,
        Fx: Fn(&std::fs::DirEntry) -> Result<(), Box<dyn Error>>
{
    info!("Processing folder: {:?}", path.as_ref());
    for entry in fs::read_dir(path)? {
        match entry {
            Ok(item) => {
                let _ = processItem(&item)
                .map_err(|err| error!("Error processing item: {:?}, message: {:?}",
                    item, err));
                ()
            },
            Err(error) => error!("Error occurred in [{:?}], message: {:?}",
                path.as_ref(),
                error)
        }
    }

    Ok(())
}

fn processInputItem(item : &fs::DirEntry, location : &Location) -> Result<(), Box<dyn Error>> {
    info!("Processing new item: {:?}", item);

    if !item.file_type()?.is_dir() {
        let mut initialMetadata = item.metadata()?;
        let path = item.path();

        if let Some(fileName) = path.file_name() {
            loop {
                thread::sleep(time::Duration::from_millis(location.readinessDelay.into()));

                let metadata = fs::metadata(&path)?;

                if metadata.len() == initialMetadata.len() &&
                    metadata.modified().ok() == initialMetadata.modified().ok() {
                        break;
                }

                initialMetadata = metadata;
            }

            let resultingFileName = if location.processing_timestamp {
                addTimestamp(fileName)
            }
            else {
                PathBuf::from(fileName)
            };

            let processingFilePath = Path::new(&location.file.processing)
                .join(resultingFileName);

            info!("Moving to processing folder: {:?}", processingFilePath);

            fs::rename(path, processingFilePath)?;
        }
    }

    Ok(())
}

fn processStagingItem(item : &fs::DirEntry, location : &Location) -> Result<(), Box<dyn Error>> {
    info!("Processing staging item: {:?}", item);

    if item.file_type()?.is_dir() {
        return Err(format!("Unexpected directory in processing folder: {:?}",
            item).into());
    }

    let itemPath = item.path();
    info!("Executing {:?} on {:?}", location.process, itemPath);

    let mut command = prepareCommand(&itemPath, location); 
    debug!("Command to execute: {:?}", command);

    let output = command.output()?;
    
    info!("Command returned: {:?}", output.status);

    if itemPath.exists() {
        if let Some(fileName) = itemPath.file_name() {

            let destinationDir = if output.status.success() {
                &location.file.completed
            }
            else {
                &location.file.failed
            };

            if let Some(realDesinationDir) = destinationDir {
                let destinationFilePath = Path::new(realDesinationDir)
                    .join(fileName);
                info!("Moving to destination: {:?} => {:?}", itemPath, destinationFilePath);
                fs::rename(itemPath, destinationFilePath)?;
            }
            else {
                info!("Destination is empty, removing {:?}", itemPath);
                fs::remove_file(itemPath)?;
            }
        }
    }

    Ok(())
}

fn prepareCommand<P>(path: P, location : &Location) -> Command
    where
        P: Copy + std::convert::AsRef<std::ffi::OsStr>
{
    let mut cmd = if location.shell_command {
        let mut cmd = if cfg!(target_os = "windows") {
                let mut cmd = Command::new("cmd");
                cmd.arg("/C");
                cmd
            }
            else {
                let mut cmd = Command::new("sh");
                cmd.arg("-c");
                cmd
            };
        cmd.arg(&location.process);
        cmd
    }
    else {
        Command::new(&location.process)
    };

    cmd.arg(path);

    if let Some(currentDir) = &location.current_dir {
        cmd.current_dir(currentDir);
    }

    cmd
}

fn addTimestamp<P>(fileName: P) -> PathBuf
    where
        P: AsRef<Path> + Copy,
        std::path::PathBuf: std::convert::From<P>
{
    let mut path = PathBuf::from(fileName);

    if let Some(origFileName) = fileName.as_ref().file_stem() {
        if let Some(origFileNameStr) = origFileName.to_str() {
            let now: DateTime<Local> = Local::now();
            let timestampSuffix = now.format("%Y-%m-%d_%H-%M-%S");

            let extension = fileName
                        .as_ref()
                        .extension()
                        .and_then(|x| x.to_str())
                        .map(|x| ".".to_owned() + x)
                        .unwrap_or("".to_owned());

            path.set_file_name(
                format!("{}_{}{}",
                    origFileNameStr,
                    timestampSuffix,
                    extension
            ));
        }
    }

    path
}
