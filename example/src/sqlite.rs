use rbatis::rbatis::Rbatis;
use std::fs::{create_dir_all, File};
use std::io::Read;

pub async fn init_sqlite_path(path: &str) -> Rbatis {
    if File::open(format!("{}target/sqlite.db", path)).is_err() {
        create_dir_all(format!("{}target/", path)).ok();
        let f = File::create(format!("{}target/sqlite.db", path)).unwrap();
        drop(f);
    }

    // // mysql custom connection option
    // // let db_cfg=DBConnectOption::from("mysql://root:123456@localhost:3306/test")?;
    // let db_cfg= DBConnectOption::from("sqlite://../target/sqlite.db")?;
    // rb.link_cfg(&db_cfg,DBPoolOptions::new());
    //
    // // custom pool
    // let mut opt = DBPoolOptions::new();
    // opt.max_size = 20;
    // rb.link_opt("sqlite://../target/sqlite.db", &opt).await.unwrap();

    // init rbatis
    let rb = Rbatis::new();
    rb.link(&format!("sqlite://{}target/sqlite.db", path))
        .await
        .unwrap();

    // run sql create table
    let mut f = File::open(format!("{}example/db.sql", path)).unwrap();
    let mut sql = String::new();
    f.read_to_string(&mut sql).unwrap();
    rb.exec(&sql, vec![]).await.ok();

    return rb;
}
