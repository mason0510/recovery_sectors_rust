Recovery-sectors
================
Recovery-sectors is a tool which  is to recovery filecoin's sectors.

Table of Contents
=================

  * [Build](#Build)
  * [Usage](#Usage)
  * [TestCase](#TestCase)
  * [TODO](#TODO)
  
Build
=====
```
cargo build
```
Usage
=====
```
./target/debug/recovery_sectors \
    --all-sectors-file=./sector_info.json \
    --workers-file=./workers.json \
    --recover-sectors-file=./recover_sectors.json \
    --par-num=4
```

TestCase
========
```
1、机器富余  ok
2、机器不足  ok
3、多存储节点 ok
```

***调度逻辑，默认每个机器分配4个任务，4个4个的填塞，所有机器使用完后，其他的等待***

## TODO
```
1、做好io压力的阀值控制   OK
2、生成文件后的自动回传     ok   
3、任务完成后的机器释放，新任务推入 ok
4、任务状态的更新             
6、bench端的recovery命令制作 ok
7、现在由于正舵那边也是填充空数据，ok
8、woker封装时候把ticket消息写入数据库
9、工具从miner下数据库读取相关信息
10、hash校验逻辑（迭代时候加）
11、先mount挂载之后传数据完毕再umount，mount的目录用ticket和sector_id的拼接命令
用命令mount和umount   ok
12、日志路径             ok
13、健壮性和anyhow处理   ok
14、客户端命令,获取当前进度
15、完善testcase
```