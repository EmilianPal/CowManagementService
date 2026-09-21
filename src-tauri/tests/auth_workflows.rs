use cowmanagementservice_lib::{
    auth::service::{authenticate_user, register_admin_and_farm},
    database::{database::create_tables, query::farm_query::insert_farm},
    model::farm::Farm,
};
use rusqlite::Connection;

#[test]
fn register_farm_then_login_and_reject_duplicate_without_partial_farm() {
    let mut conn = Connection::open_in_memory().unwrap();
    create_tables(&conn).unwrap();
    let user = register_admin_and_farm(
        &mut conn, "Ferma familiei", "Nume Prenume", "test@example.test", "parola-de-test",
    ).unwrap();
    let farm_name: String = conn.query_row(
        "SELECT name FROM farms WHERE id = ?1", [user.farm_id], |row| row.get(0),
    ).unwrap();
    assert_eq!(farm_name, "Ferma familiei");
    let logged_in = authenticate_user(&conn, "Nume Prenume", "parola-de-test").unwrap();
    assert_eq!(logged_in.id, user.id);
    assert_eq!(logged_in.farm_id, user.farm_id);
    assert!(authenticate_user(&conn, "Nume Prenume", "parola-gresita").is_err());
    assert!(register_admin_and_farm(
        &mut conn, "Altă fermă", "Nume Prenume", "alt@example.test", "parola-de-test",
    ).is_err());
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM farms", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn farm_with_explicit_id_uses_the_same_table() {
    let conn = Connection::open_in_memory().unwrap();
    create_tables(&conn).unwrap();
    let id = insert_farm(&conn, &Farm { id: Some(42), name: "Ferma de test".into() }).unwrap();
    assert_eq!(id, 42);
    let name: String = conn.query_row("SELECT name FROM farms WHERE id = 42", [], |row| row.get(0)).unwrap();
    assert_eq!(name, "Ferma de test");
}
