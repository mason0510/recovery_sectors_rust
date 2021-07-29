use crate::file::{WorkersInfo, json};

#[test]
fn test4() {
    let mut a = Vec::new();
    {
        // fix this line to make this test pass
        let b = vec![0; 10000001];
        a.extend(b);
        a[10000000] = 1;
    }
    assert_eq!(a[10000000], 2);
}
//cargo test -- --nocapture
#[test]
fn test_json_read(){
    let result: WorkersInfo =  WorkersInfo::read("/home/eddy/recovery/recovery_sectors/workers.json".to_string()).unwrap();
    println!("===={:?}",result);
}
