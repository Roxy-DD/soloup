//! 升级安装后的数据迁移回归测试。
//!
//! 用户视角的那条路径是：覆盖安装新版本 → 桌面壳拉起 soloup-server →
//! `open_store()` → 自动补齐迁移。这里把它拆成可断言的几条：
//! ① 只跑到 V1 的老库能被打开并升到最新 schema；
//! ② 旧数据一条不丢；
//! ③ 迁移前留下一份带来源版本的快照（`backups/soloup-<时间>-v1.db`）可回滚；
//! ④ 遇到比当前程序更新的库要拒绝打开，避免装回旧版把新库写坏。

use std::path::{Path, PathBuf};

use soloup_store::migrations;
use soloup_store::{latest_schema_version, open_store, OpenStoreOptions, StoreErrorCode};

fn temp_db(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("soloup-migrate-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir.join("soloup.db")
}

/// 造一个「只跑到 V1」的老库：v1 建表 SQL + 若干用户数据 + user_version=1。
fn make_v1_db(path: &Path) {
    let conn = rusqlite::Connection::open(path).expect("open raw db");
    conn.execute_batch(migrations::V1_INIT.sql)
        .expect("apply v1 schema");
    conn.execute_batch(
        "INSERT INTO attributes (id,name,base_value,max_value,alpha,w0,sort) \
           VALUES ('a1','体质',0,100,1.2,400,0);
         INSERT INTO skills (id,name,category,difficulty,c,v,created_at,sort) \
           VALUES ('s1','vibecoding','tech','medium',1.0,2.0,'2026-01-01',0);
         INSERT INTO daily_records (date,note,settled) VALUES ('2026-01-02','老记录',1);",
    )
    .expect("seed v1 data");
    conn.pragma_update(None, "user_version", 1)
        .expect("set user_version");
}

fn open_at(path: &Path) -> Result<soloup_store::Store, soloup_store::StoreError> {
    open_store(OpenStoreOptions {
        path: Some(path.to_string_lossy().to_string()),
    })
}

#[test]
fn old_database_is_migrated_on_open_and_keeps_data() {
    let db = temp_db("upgrade");
    make_v1_db(&db);

    let store = open_at(&db).expect("老库应能打开并自动迁移");

    let version: i64 = store
        .conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        version,
        latest_schema_version(),
        "打开后应升到当前程序支持的最新 schema"
    );

    // 旧数据一条不丢
    assert_eq!(store.attributes().list().unwrap().len(), 1);
    let records: i64 = store
        .conn
        .query_row("SELECT COUNT(*) FROM daily_records", [], |r| r.get(0))
        .unwrap();
    assert_eq!(records, 1);
    let skills: i64 = store
        .conn
        .query_row("SELECT COUNT(*) FROM skills", [], |r| r.get(0))
        .unwrap();
    assert_eq!(skills, 1);

    // 迁移前快照留在库文件同级的 backups/ 下，文件名带来源版本
    let backup_dir = db.parent().unwrap().join("backups");
    let names: Vec<String> = std::fs::read_dir(&backup_dir)
        .expect("应生成备份目录")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(names.len(), 1, "应只有 1 份迁移前快照，实际：{names:?}");
    assert!(
        names[0].starts_with("soloup-") && names[0].ends_with("-v1.db"),
        "快照名应形如 soloup-<时间>-v1.db，实际：{}",
        names[0]
    );
}

#[test]
fn newer_database_is_refused_instead_of_downgraded() {
    let db = temp_db("newer");
    make_v1_db(&db);
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.pragma_update(None, "user_version", latest_schema_version() + 1)
            .unwrap();
    }

    let err = match open_at(&db) {
        Ok(_) => panic!("更高版本的库必须拒绝打开"),
        Err(e) => e,
    };
    assert_eq!(err.code, StoreErrorCode::NewerSchema);
}
