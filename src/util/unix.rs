use crate::file::*;
use ssh2::{Session};
use std::io::prelude::*;
use std::net::TcpStream;
use anyhow::{Context, Result};


#[derive(Clone)]
pub struct Shell {
    sess: Session,
}

impl Shell {
    pub fn new(device: WorkerInfo) -> Result<Shell> {
        let tcp = TcpStream::connect(device.ip.as_str())?;
        //let tcp = TcpStream::connect("10.88.66.32:22").unwrap();
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_password("root", device.password.as_str())?;

        Ok(Shell { sess })
    }
    pub fn mount(&self, device: &str, local_path: &str) -> Result<()> {
        let mut channel = self.sess.channel_session()?;
        //            "mount -t nfs 10.88.66.26:/home/nfs  /data/cache/tmp",
        let bench_cmd = format!("mount -t nfs {}  {}", device, local_path);
        log::info!("bench_cmd={}", bench_cmd);
        channel.exec(bench_cmd.as_str())?;
        channel.wait_close();
        Ok(())
    }

    pub fn run_bench(&self, sector_id: &str, ticket: &str) -> Result<()> {
        let mut channel = self.sess.channel_session()?;
        let bench_cmd = format!(
            "RUST_LOG=Info /root/hlm-miner/apps/lotus/lotus-bench recovery --storage-dir=/home/recovery_data \
            --sector-size=512MiB --sector-id={} --ticket={} \
            > /home/recovery_data/512MiB_{}.log 2>&1 &",
            sector_id, ticket, sector_id);
        log::info!("run bench --{}--", bench_cmd);
        channel.exec(bench_cmd.as_str())?;
        channel.wait_close();
        Ok(())
    }

    pub fn umount(&self, local_path: &str) {
        //            "umount -t /data/cache/tmp",
        let mut channel = self.sess.channel_session().unwrap();
        let bench_cmd = format!("umount  {}", local_path);
        channel.exec(bench_cmd.as_str()).unwrap();
        channel.wait_close();
        loop {
            let mut channel = self.sess.channel_session().unwrap();
            let bench_cmd = format!("umount | grep push");
            channel.exec(bench_cmd.as_str()).unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            if s == "" {
                log::info!("umount done");
                break;
            }
            log::info!("mount result {}", s);
            std::thread::sleep(std::time::Duration::from_millis(10000));
            channel.wait_close();
        }
        //fixme:未知原因卸载有延迟，测试中延时3s可以规避问题，这里安全起见放大10倍，另外也要在存储节点开启只读模式双保险
       std::thread::sleep(std::time::Duration::from_millis(30000));
    }

    //该rsync是覆盖和同步，如果dst路径下有src没有的文件则会保留
    pub fn rsync(&self, from: &str, to: &str) {
        //rsync -Patv root@10.43.8.1:/data/filecoin-proof-parameters /data/cache-1/
        let mut channel = self.sess.channel_session().unwrap();
        let bench_cmd = format!("rsync -Patv {} {}", from, to);
        channel.exec(bench_cmd.as_str()).unwrap();
        channel.wait_close();
    }

    pub fn ls(&self, path: &str) -> String {
        //            "ls /home/recovery_data/cache/s-{}/p_aux",
        let mut channel = self.sess.channel_session().unwrap();
        let bench_cmd = format!("ls {}", path);
        channel.exec(bench_cmd.as_str()).unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        s
    }

    pub fn mkdir(&self, path: &str) {
        //            "ls /home/recovery_data/cache/s-{}/p_aux",
        let mut channel = self.sess.channel_session().unwrap();
        let bench_cmd = format!("mkdir -p {}", path);
        channel.exec(bench_cmd.as_str()).unwrap();
        channel.wait_close();
    }

    pub fn rm(&self, path: &str) {
        //            "ls /home/recovery_data/cache/s-{}/p_aux",
        let mut channel = self.sess.channel_session().unwrap();
        let bench_cmd = format!("rm {} -r", path);
        channel.exec(bench_cmd.as_str()).unwrap();
        channel.wait_close();
    }
}
