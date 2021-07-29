use super::file::*;
use super::util::unix::Shell;
use ssh2::{Channel};
use std::io::prelude::*;
use anyhow::{Context, Result};


#[derive(Clone)]
pub struct TaskInfo {
    pub ip: String,
    shell: Shell,
    pub sector_info: SectorInfo,
    pub status: String, //"start","P1Done","P2Done","finalize","finished"
}

impl TaskInfo {
    pub fn new(device: WorkerInfo, sector_info: SectorInfo) -> Result<TaskInfo> {
        let shell = Shell::new(device.clone())?;
        Ok(TaskInfo {
            ip: device.ip,
            shell,
            sector_info,
            status: "start".to_string(),
        })
    }

    pub fn run_bench(&mut self) {
        self.shell
            .run_bench(&*self.sector_info.sector_id, &*self.sector_info.ticket);
    }

    //listen task status and update QUEUES
    pub fn listen_process(&mut self) {
        loop {
            let path = format!(
                "/home/recovery_data/cache/s-{}",
                self.sector_info.sector_id
            );
            let result = self.shell.ls(&path);
            if result.contains("t_aux") {
                log::info!("recovery sector s-{} finished", self.sector_info.sector_id);
                break;
            } else {
                log::info!(
                    "recovery sector s-{} running,s={}",
                    self.sector_info.sector_id,
                    result
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(5000));
        }
    }

    //push file to storage device
    pub fn finalize(&mut self) {
        let storage_server = self.sector_info.storage_path.split(":").collect::<Vec<&str>>()[0];
        let mount_path = format!(
            "/home/recovery_data/{}_push",
            storage_server
        );
        let result = self.shell.ls(&mount_path);
        if result == "" {
            let _result = self.shell.mkdir(&mount_path);
            self.shell
                .mount(&*self.sector_info.storage_path, &mount_path);
            //防止挂载点没有cache和sealed目录，重复创建不会覆盖
            let _result = self.shell.mkdir(&(mount_path.clone()+"/cache"));
            let _result = self.shell.mkdir(&(mount_path.clone()+"/sealed"));

            log::info!(
                "mount device: {} successful",
                self.ip.split(":").collect::<Vec<&str>>()[0]
            );
        } else {
            log::info!(
                "device: {}  already mount and do nothing!,result={}",
                self.ip,
                result
            );
        }
        //let path = format!("/home/recovery_data/cache/s-{}/p_aux", self.sector_info.sector_id);
        let push_dst_cache = format!("{}/cache/s-{}", mount_path, self.sector_info.sector_id);
        let push_src_cache = format!(
            "/home/recovery_data/cache/s-{}/",
            self.sector_info.sector_id
        );
        let push_dst_sealed = format!("{}/sealed/s-{}", mount_path, self.sector_info.sector_id);
        let push_src_sealed = format!(
            "/home/recovery_data/sealed/s-{}",
            self.sector_info.sector_id
        );

        //选落盘大文件
        self.shell.rsync(&*push_src_sealed, &*push_dst_sealed);
        self.shell.rsync(&*push_src_cache, &*push_dst_cache);
        //todo:verify
        log::info!("finalize finished");
    }
}

pub fn push_cache() {
    todo!()
}

pub fn push_sealed() {
    todo!()
}

impl Drop for TaskInfo {
    fn drop(&mut self) {
        //self.session.wait_close();
        log::info!("");
    }
}

pub fn release_task() {
    todo!()
}

pub fn add_task() {
    todo!()
}
