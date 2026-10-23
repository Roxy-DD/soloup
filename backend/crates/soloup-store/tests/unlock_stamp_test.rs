//! 成就「首次达成日」落库的回归测试。
//!
//! 语义只有两条，但都容易在改动中被顺手破坏：
//! ① **只写一次** —— 已有时间戳的行不能被后一次调用覆盖（否则「达成日」会天天变成今天）；
//! ② **幂等** —— 同一批 id 重复调用，第二次返回 0、数据不变。
//!
//! 另外，`stamp_new_unlocks` 只负责写 NULL 行，所以「条件重新不满足」不需要额外处理：
//! 它压根不会去清空已写的时间戳。

use soloup_store::{open_store, OpenStoreOptions};

fn mem_store() -> soloup_store::Store {
    open_store(OpenStoreOptions {
        path: Some(":memory:".to_string()),
    })
    .expect("open")
}

fn stamp_of(store: &soloup_store::Store, id: &str) -> Option<String> {
    store
        .achievements()
        .get(id)
        .expect("get achievement")
        .expect("achievement exists")
        .unlocked_at
}

#[test]
fn first_stamp_is_written_once_and_never_overwritten() {
    let store = mem_store();
    let ids = vec!["first".to_string()];

    // 迁移种子里的成就是未达成的
    assert_eq!(stamp_of(&store, "first"), None);

    let n = store
        .achievements()
        .stamp_new_unlocks(&ids, "2026-03-04")
        .expect("stamp");
    assert_eq!(n, 1, "首次达成应写入一行");
    assert_eq!(stamp_of(&store, "first").as_deref(), Some("2026-03-04"));

    // 第二次（哪怕日期不同）不得覆盖 —— 这是「达成日」这个语义的全部意义
    let n = store
        .achievements()
        .stamp_new_unlocks(&ids, "2026-10-23")
        .expect("stamp again");
    assert_eq!(n, 0, "已有时间戳的行不应被再次写入");
    assert_eq!(
        stamp_of(&store, "first").as_deref(),
        Some("2026-03-04"),
        "达成日必须保持首次写入的值"
    );
}

#[test]
fn stamp_ignores_unknown_ids_and_empty_input() {
    let store = mem_store();

    assert_eq!(
        store
            .achievements()
            .stamp_new_unlocks(&[], "2026-03-04")
            .expect("empty ok"),
        0,
        "空输入不应报错也不应写库"
    );
    assert_eq!(
        store
            .achievements()
            .stamp_new_unlocks(&["not-an-achievement".to_string()], "2026-03-04")
            .expect("unknown id ok"),
        0,
        "未知 id 不应报错（判定与库不同步时不该把整次请求打挂）"
    );
}

#[test]
fn batch_stamp_writes_only_missing_rows() {
    let store = mem_store();
    let repo = store.achievements();

    // 只给其中一条先写，验证批量里「已写的跳过、未写的补上」
    repo.stamp_new_unlocks(&["first".to_string()], "2026-01-01")
        .expect("seed one");

    let batch = vec!["first".to_string(), "twin".to_string(), "week7".to_string()];
    let n = repo.stamp_new_unlocks(&batch, "2026-05-06").expect("batch");
    assert_eq!(n, 2, "批量里只有两条是 NULL");

    assert_eq!(stamp_of(&store, "first").as_deref(), Some("2026-01-01"));
    assert_eq!(stamp_of(&store, "twin").as_deref(), Some("2026-05-06"));
    assert_eq!(stamp_of(&store, "week7").as_deref(), Some("2026-05-06"));
}
