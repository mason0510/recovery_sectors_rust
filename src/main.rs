//#![deny(warnings)]
mod file;
mod task;
mod util;

use std::env;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use crate::file::{SectorInfo, WorkersInfo};


use rayon::prelude::*;

use std::collections::{BTreeMap};
use std::io::prelude::*;
use anyhow::{Context, Result};
use std::sync::Mutex;


/****
pub struct TaskInfo {
    session: Session,
    sector_info: SectorInfo,
    status: String, //"start","P1Done","P2Done","finalize","finished"
}
***/
/***
#[derive(Debug)]
struct SectorProcess {
    id: String,
    status: String
}
#[derive(Debug)]
struct DeviceInfo {
    ip: String,
    sectors_info: HashMap<String, String>,
}
**/

type SectorsInfo = BTreeMap<String, String>;
type DeviceInfo = BTreeMap<String, SectorsInfo>;

//par num of device run task
//const par_num: usize = 2;

lazy_static! {
    static ref QUEUES: Mutex<DeviceInfo> = Mutex::new( DeviceInfo::new() );
}

fn queues_init(workers: &WorkersInfo) {
    let mut queues = crate::QUEUES.lock().unwrap();
    let sectors_map = BTreeMap::<String, String>::new();
    for worker in &workers.workers {
        queues.insert(worker.ip.clone(), sectors_map.clone());
    }
    workers.clean_up();
}
//考虑到中途推出的场景，队列开始和结束都要清理
fn queues_done(workers: &WorkersInfo) {
    workers.clean_up();
}

fn main() -> Result<()>{
    env_logger::init();
    // let mut sectors_info_file = "/data/sdb/lotus-user-1/.lotusstorage/sector_info.json".to_string();
    let mut sectors_info_file = "sector_info.json".to_string();
    let mut par_num = "1".to_string();
    let mut recover_sectors_file = "".to_string();
    let mut workers_file = "".to_string();
    let mut get_status = false;

    let get_option_value = |argument: &str, key: &str| -> Option<String> {
        if argument.contains(&(key.to_string() + "=")) {
            let value: Vec<_> = argument.split('=').collect();
            return Some(value[1].to_string());
        }
        None
    };

    for argument in env::args() {
        sectors_info_file = get_option_value(&argument, "--all-sectors-file")
            .or_else(|| Some(sectors_info_file))
            .unwrap();
        par_num = get_option_value(&argument, "--par-num")
            .or_else(|| Some(par_num))
            .unwrap();
        recover_sectors_file = get_option_value(&argument, "--recover-sectors-file")
            .or_else(|| Some(recover_sectors_file))
            .unwrap();
        workers_file = get_option_value(&argument, "--workers-file")
            .or_else(|| Some(workers_file))
            .unwrap();
        if argument.contains("status") {
            get_status = true;
        }
    }
    let par_num :usize = par_num.parse()?;
    log::info!("{},{},{}", sectors_info_file, par_num, recover_sectors_file);
    //fixme: tmp code for client
    if get_status {
        let mut sectors_map = BTreeMap::<String, String>::new();
        sectors_map.insert("f0152134-0".to_string(), "P1done".to_string());

        let mut tmp_queues = crate::QUEUES.lock().unwrap();
        tmp_queues.insert("127.0.0.1".to_string(), sectors_map);

        match tmp_queues.get_mut(&*"127.0.0.1".to_string()) {
            Some(review) => {
                review.insert("f0152134-2".to_string(), "P2done".to_string());
                review.insert("f0152134-3".to_string(), "P3done".to_string());
            }
            None => {
                let mut sectors_map = BTreeMap::<String, String>::new();
                sectors_map.insert("f0152134-0".to_string(), "P1done".to_string());
                sectors_map.insert("f0152134-1".to_string(), "P2done".to_string());
                tmp_queues.insert("127.0.0.1".to_string(), sectors_map);
            }
        }

        println!("task info {:?}", tmp_queues);
        return Ok(());
    }

    let workers = file::list_workers(workers_file)?;
    queues_init(&workers);
    let all_sector_info = file::list_all_sectors(sectors_info_file)?;
    let recovered_list = file::list_recovery_sector_ids(recover_sectors_file)?;
    let mut global_recovery_sector_info = Vec::<SectorInfo>::new();

    //获取待恢复扇区的详情信息
    for sector_id in recovered_list.sectors_id {
        for sector_info in all_sector_info.sectors_info.clone() {
            if sector_id == sector_info.sector_id {
                global_recovery_sector_info.push(sector_info);
            }
        }
    }
    log::info!("recovery sectors info {:?}", global_recovery_sector_info);
    //调度规则，没轮使用所有机器，均分扇区执行重做任务，进行多个轮次直到队列中没有待重做的扇区
    loop {
        if global_recovery_sector_info.len() == 0 {
            log::info!("queue finished");
            break;
        }

        log::info!(
            "remained_recovery_sector_info=,,{:?}",
            global_recovery_sector_info
        );
        //fixme：该批次完成的时间取决于完成最慢的那个机器的时间，如果一个机器卡住整个轮次也是一直卡住
        rayon::scope(|ss| {
            let mut sector_cursor = 0;
            let round_recovery_sector_info = global_recovery_sector_info.clone();
            'devices: for (_index, worker) in workers.workers.iter().enumerate() {
                let end = if sector_cursor + par_num > round_recovery_sector_info.len() {
                    round_recovery_sector_info.len()
                } else {
                    sector_cursor + par_num
                };
                info!("{}--{}", sector_cursor, end);
                let alloc_recovery = round_recovery_sector_info
                    .get(sector_cursor..end)
                    .unwrap()
                    .to_owned();
                info!("{}##{}", sector_cursor, end);
                'sectors: for sector in alloc_recovery {
                    global_recovery_sector_info.retain(|x| x.sector_id != sector.sector_id);
                    ss.spawn(move |_| {
                        let mut task = task::TaskInfo::new(worker.clone(), sector.clone()).unwrap();
                        log::info!("new task {:?},device={}", task.sector_info, task.ip);
                        //update QUEUES
                        let mut queues = crate::QUEUES.lock().unwrap();
                        match queues.get_mut(&*task.ip) {
                            Some(sectors) => {
                                sectors.insert(
                                    task.sector_info.sector_id.clone(),
                                    task.status.clone(),
                                );
                            }
                            None => panic!("queues haven't inited ?"),
                        }

                        drop(queues);

                        task.run_bench();
                        task.listen_process();
                        task.finalize();
                        //todo:QUEUES信息更新释放task或者机器
                        let mut queues = crate::QUEUES.lock().unwrap();
                        match queues
                            .get_mut(&*task.ip)
                            .unwrap()
                            .remove(&task.sector_info.sector_id)
                        {
                            Some(_) => info!("release a task  successfully"),
                            None => panic!("unknonw errors,release device failed"),
                        }
                    });
                }
                sector_cursor = end;
                println!("task1 info {:?}", crate::QUEUES.lock().unwrap());
            }
        });
    }
    //todo device_clean_up()
    queues_done(&workers);
    log::info!("queue done {:?}", crate::QUEUES.lock().unwrap());
    Ok(())
}

//test func
#[cfg(test)]
mod test {
    #[test]
    fn get_queue_info() {
        let mut a = Vec::new();
        {
            // fix this line to make this test pass
            let b = vec![0; 10000001];
            a.extend(b);
            a[10000000] = 1;
        }
        assert_eq!(a[10000000], 1);
    }

    #[test]
    fn run_task_local() {
        use std::future::Future;
        // let a = async { "Hello World!" };
        let _test = async {
            fn hello_world() -> impl Future<Output=&'static str> {
                async { "Hello World!" }
            }

            // fix this line to make this test pass
            let b = hello_world().await;
            assert_eq!(b, "Hello World!");
        };
    }
}
