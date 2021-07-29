mod test;

use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use anyhow::{Context, Result};
use std::ops::Deref;
use crate::util::unix::Shell;


#[derive(Debug, Deserialize, Clone)]
pub struct SectorsInfo {
    pub sectors_info: Vec<SectorInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SectorInfo {
    pub sector_id: String,
    pub ticket: String,
    pub storage_path: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkersInfo {
    pub workers: Vec<WorkerInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkerInfo {
    pub ip: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SectorsID {
    pub sectors_id: Vec<String>,
}

trait json {
    //fn read<'a, T: Deserialize<'a>>(path: String) -> Result<T>
    fn read<T: for<'a> Deserialize<'a>>(path: String) -> Result<T>
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data: T = serde_json::from_reader(reader).expect("JSON was not well-formatted");
        Ok(data)
    }

    fn write(&self) {}
}

impl json for WorkersInfo {}
impl WorkersInfo {
    pub fn clean_up(&self) -> Result<()>{
        log::info!("start clean up2");
        for worker in &self.workers {
            let shell = Shell::new(worker.clone())?;
            //    //umount /home/recovery_data/*push;rm /home/recovery_data/* -r;rm /home/nfs/cache/* -rf;rm /home/nfs/sealed/* -rf
            log::info!("start clean-0");
            shell.umount("/home/recovery_data/*push");
            log::info!("start clean-1");
            //std::thread::sleep(std::time::Duration::from_millis(10000));
            log::info!("start clean-2");
            shell.rm("/home/recovery_data/*");
        }
        Ok(())
    }
}
impl json for SectorsID {}
impl json for SectorsInfo {}


pub fn list_workers(path: String) -> Result<WorkersInfo> {
    let workers: WorkersInfo = WorkersInfo::read(path)?;
    log::info!("my list_workers {:?}", workers);
    Ok(workers)
}

pub fn list_recovery_sector_ids(path: String) -> Result<SectorsID> {
    let sectors_id: SectorsID = SectorsID::read(path)?;
    Ok(sectors_id)
}

pub fn list_all_sectors(path: String) -> Result<SectorsInfo> {
    let sectors: SectorsInfo = SectorsInfo::read(path)?;
    Ok(sectors)
}
