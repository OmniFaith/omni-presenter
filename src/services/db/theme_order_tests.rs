use super::*;

fn database() -> AppDb {
    let db = AppDb::open_in_memory().unwrap();
    for (id, order) in [("order_first", 7), ("order_second", 8)] {
        let theme = crate::core::models::PresentationTheme {
            id: id.into(),
            sort_order: order,
            ..Default::default()
        };
        db.save_theme(&theme).unwrap();
    }
    db
}

fn order(db: &AppDb, id: &str) -> i32 {
    db.conn
        .lock()
        .unwrap()
        .query_row("SELECT sort_order FROM themes WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .unwrap()
}

#[test]
fn theme_order_is_committed_as_a_batch() {
    let db = database();
    db.update_themes_order(&["order_second".into(), "order_first".into()])
        .unwrap();
    assert_eq!(order(&db, "order_second"), 0);
    assert_eq!(order(&db, "order_first"), 1);
}

#[test]
fn theme_order_failure_is_reported_and_rolls_back_previous_updates() {
    let db = database();
    db.conn
        .lock()
        .unwrap()
        .execute_batch(
            "CREATE TRIGGER fail_theme_order BEFORE UPDATE OF sort_order ON themes
         WHEN NEW.id = 'order_second'
         BEGIN SELECT RAISE(ABORT, 'simulated write failure'); END;",
        )
        .unwrap();
    assert!(
        db.update_themes_order(&["order_first".into(), "order_second".into()])
            .is_err()
    );
    assert_eq!(order(&db, "order_first"), 7);
    assert_eq!(order(&db, "order_second"), 8);
}

#[test]
fn missing_theme_rolls_back_the_batch() {
    let db = database();
    assert!(
        db.update_themes_order(&["order_first".into(), "missing".into()])
            .is_err()
    );
    assert_eq!(order(&db, "order_first"), 7);
}
