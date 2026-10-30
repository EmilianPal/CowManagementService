use chrono::NaiveDate;
use crate::utils::cow_filter::CowFilter;
use crate::auth::session::{AppState, UserSession};
use crate::model::{cow::Cow, birth::Birth, insemination::Insemination};
use crate::service::service;
use crate::auth::service as auth_service;
use crate::auth::user::User;
use crate::command::command_manager::CommandManager;
use rusqlite::OptionalExtension;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

fn require_writer(conn: &rusqlite::Connection, session: &UserSession) -> Result<(), String> {
    let role: String = conn.query_row(
        "SELECT role FROM users WHERE id = ?1 AND farm_id = ?2",
        rusqlite::params![session.user_id, session.farm_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if role == "Admin" || role == "Editor" { Ok(()) }
    else { Err("Nu ai dreptul de a modifica datele fermei.".into()) }
}

fn require_session_farm(session: &UserSession, farm_id: i64) -> Result<(), String> {
    if farm_id == session.farm_id { Ok(()) }
    else { Err("Ferma selectată nu corespunde sesiunii.".into()) }
}

#[derive(Clone, Copy)]
enum Reference { Cow, Birth }

fn require_reference_in_farm(
    conn: &rusqlite::Connection,
    reference: Reference,
    id: i64,
    farm_id: i64,
) -> Result<(), String> {
    let table = match reference { Reference::Cow => "cows", Reference::Birth => "births" };
    let exists: bool = conn.query_row(
        &format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = ?1 AND farm_id = ?2)"),
        rusqlite::params![id, farm_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if exists { Ok(()) } else { Err("Înregistrarea asociată nu există în această fermă.".into()) }
}

#[tauri::command]
pub async fn export_report(
    app: tauri::AppHandle,
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
    filter: CowFilter,
) -> Result<Option<String>, String> {
    let farm_id = {
        let guard = state.session.lock().map_err(|_| "Sesiunea nu poate fi accesată.")?;
        guard.as_ref().ok_or("No session found")?.farm_id
    };
    tauri::async_runtime::spawn_blocking(move || {
        let date = filter.date.unwrap_or_else(|| chrono::Local::now().date_naive());
        let selected = app.dialog().file()
            .set_parent(&window)
            .set_title("Salvează raportul Excel")
            .set_file_name(format!("Efectiv_{}.xlsx", date.format("%Y-%m-%d")))
            .add_filter("Registru Excel (*.xlsx)", &["xlsx"])
            .blocking_save_file();
        let Some(selected) = selected else { return Ok(None); };
        let path = selected.into_path().map_err(|_| "Selectează un fișier local pentru salvare.")?;
        if !path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("xlsx")) {
            return Err("Selectează un nume de fișier cu extensia .xlsx.".into());
        }
        let path = path.to_str().ok_or("Aplicația nu poate folosi calea selectată.")?;
        let state = app.state::<AppState>();
        let guard = state.session.lock().map_err(|_| "Sesiunea nu poate fi accesată.")?;
        if guard.as_ref().map(|s| s.farm_id) != Some(farm_id) {
            return Err("No session found".into());
        }
        let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
        service::export_to_xlsx(&mut conn, farm_id, path, filter)?;
        Ok(Some(path.to_owned()))
    }).await.map_err(|_| "Raportul nu a putut fi salvat.".to_string())?
}

#[derive(serde::Serialize)]
pub struct SessionInfo {
    user: User,
    farm_name: String,
}

#[tauri::command]
pub fn get_session(state: tauri::State<'_, AppState>) -> Result<Option<SessionInfo>, String> {
    let mut guard = state.session.lock().map_err(|_| "Sesiunea nu poate fi accesată.")?;
    let conn = state.db_pool.get().map_err(|e| e.to_string())?;
    let username: Option<String> = conn.query_row(
        "SELECT u.username FROM users u JOIN app_settings s ON s.active_user_id = u.id WHERE s.id = 1",
        [], |row| row.get(0),
    ).optional().map_err(|e| e.to_string())?;
    let Some(username) = username else { *guard = None; return Ok(None); };
    let user = crate::database::query::user_query::get_user_by_username(&conn, &username)?
        .ok_or("Utilizatorul nu mai există.")?;
    let farm_name = conn.query_row("SELECT name FROM farms WHERE id = ?1", [user.farm_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if guard.as_ref().map(|s| (s.user_id, s.farm_id)) != user.id.map(|id| (id, user.farm_id)) {
        *guard = Some(UserSession {
            user_id: user.id.ok_or("Utilizatorul nu are identificator.")?,
            farm_id: user.farm_id,
            command_manager: CommandManager::new(),
        });
    }
    Ok(Some(SessionInfo { user, farm_name }))
}

#[derive(serde::Serialize)]
pub struct HistoryState { undo: bool, redo: bool }

#[tauri::command]
pub fn get_history_state(state: tauri::State<'_, AppState>) -> Result<HistoryState, String> {
    let guard = state.session.lock().map_err(|_| "Sesiunea nu poate fi accesată.")?;
    let session = guard.as_ref().ok_or("No session found")?;
    Ok(HistoryState {
        undo: !session.command_manager.undo_stack.is_empty(),
        redo: !session.command_manager.redo_stack.is_empty(),
    })
}

#[tauri::command]
pub fn add_cow( state: tauri::State<'_, AppState>, cow: Cow) -> Result<i64, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    if let Some(id) = cow.birth_id {
        require_reference_in_farm(&conn, Reference::Birth, id, session.farm_id)?;
    }
    service::add_cow(&mut conn, &mut session.command_manager, session.farm_id, cow)
}

#[tauri::command]
pub fn add_cow_with_birth(state: tauri::State<'_, AppState>, cow: Cow, mother_id: i64) -> Result<i64, String> {
    let mut guard = state.session.lock().map_err(|_| "Sesiunea nu poate fi accesată.")?;
    let session = guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    service::add_cow_with_birth(&mut conn, &mut session.command_manager, session.farm_id, cow, mother_id)
}

#[tauri::command]
pub fn update_cow(state: tauri::State<'_, AppState>, cow: Cow) -> Result<bool, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    require_session_farm(session, cow.farm_id)?;
    if let Some(id) = cow.birth_id {
        require_reference_in_farm(&conn, Reference::Birth, id, session.farm_id)?;
    }
    service::update_cow(&mut conn, &mut session.command_manager, cow)
}

#[tauri::command]
pub fn exit_cows(state: tauri::State<'_, AppState>, cow_ids: Vec<i64>, date: NaiveDate) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|_| "Sesiunea nu poate fi accesată.")?;
    let session = guard.as_mut().ok_or("Sesiunea nu mai este activă.")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    session.command_manager.execute(Box::new(crate::command::exit_command::ExitCows { ids: cow_ids, farm_id: session.farm_id, date }), &mut conn)?;
    Ok(())
}

#[tauri::command]
pub fn delete_cow(state: tauri::State<'_, AppState>, cow_id: i64, farm_id: i64) -> Result<bool, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    require_session_farm(session, farm_id)?;
    service::delete_cow(&mut conn, &mut session.command_manager, session.farm_id, cow_id)
}

#[tauri::command]
pub fn add_birth(state: tauri::State<'_, AppState>, birth: Birth) -> Result<i64, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    require_session_farm(session, birth.farm_id)?;
    require_reference_in_farm(&conn, Reference::Cow, birth.mother_id, session.farm_id)?;
    service::add_birth(&mut conn, &mut session.command_manager, birth)
}

#[tauri::command]
pub fn update_birth(state: tauri::State<'_, AppState>, birth: Birth) -> Result<bool, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    require_session_farm(session, birth.farm_id)?;
    require_reference_in_farm(&conn, Reference::Cow, birth.mother_id, session.farm_id)?;
    service::update_birth(&mut conn, &mut session.command_manager, birth)
}

#[tauri::command]
pub fn delete_birth(state: tauri::State<'_, AppState>, birth_id: i64, farm_id: i64) -> Result<bool, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    require_session_farm(session, farm_id)?;
    service::delete_birth(&mut conn, &mut session.command_manager, session.farm_id, birth_id)
}

#[tauri::command]
pub fn add_insemination(state: tauri::State<'_, AppState>, insemination: Insemination) -> Result<i64, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    require_session_farm(session, insemination.farm_id)?;
    require_reference_in_farm(&conn, Reference::Cow, insemination.dam_id, session.farm_id)?;
    if let Some(id) = insemination.sire_id {
        require_reference_in_farm(&conn, Reference::Cow, id, session.farm_id)?;
    }

    service::add_insemination(&mut conn, &mut session.command_manager, insemination)
}

#[tauri::command]
pub fn update_insemination(state: tauri::State<'_, AppState>, insemination: Insemination) -> Result<bool, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    require_session_farm(session, insemination.farm_id)?;
    require_reference_in_farm(&conn, Reference::Cow, insemination.dam_id, session.farm_id)?;
    if let Some(id) = insemination.sire_id {
        require_reference_in_farm(&conn, Reference::Cow, id, session.farm_id)?;
    }

    service::update_insemination(&mut conn, &mut session.command_manager, session.farm_id, insemination)
}

#[tauri::command]
pub fn delete_insemination(state: tauri::State<'_, AppState>, insemination_id: i64) -> Result<bool, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;

    service::delete_insemination(&mut conn, &mut session.command_manager, session.farm_id, insemination_id)
}

#[tauri::command]
pub fn assign_calf_to_birth(state: tauri::State<'_, AppState>, calf_id: i64, birth_id: i64) -> Result<bool, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    require_reference_in_farm(&conn, Reference::Birth, birth_id, session.farm_id)?;

    service::assign_calf_to_birth(&mut conn, &mut session.command_manager, session.farm_id, calf_id, birth_id)
}

// -------------------------------------------------------------------------
// UNDO / REDO
// These interact directly with the command_manager in the session!
// -------------------------------------------------------------------------

#[tauri::command]
pub fn undo(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;

    session.command_manager.undo(&mut conn)
}

#[tauri::command]
pub fn redo(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;

    session.command_manager.redo(&mut conn)
}

// -------------------------------------------------------------------------
// READ OPERATIONS (GETTERS)
// No command_manager needed! Just the farm_id for data isolation.
// -------------------------------------------------------------------------

#[tauri::command]
pub fn get_cow(state: tauri::State<'_, AppState>, cow_id: i64) -> Result<Cow, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_cow(&mut conn, cow_id, session.farm_id)
}

#[tauri::command]
pub fn get_cows(state: tauri::State<'_, AppState>) -> Result<Vec<Cow>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_cows(&mut conn, session.farm_id)
}

#[tauri::command]
pub fn get_cow_by_eartag(state: tauri::State<'_, AppState>, eartag: &str) -> Result<Cow, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_cow_by_eartag(&mut conn, session.farm_id, eartag)
}

#[tauri::command]
pub fn get_unassigned_calves_on_date(state: tauri::State<'_, AppState>, date: NaiveDate) -> Result<Vec<Cow>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_unassigned_calves_on_date(&mut conn, session.farm_id, &date)
}

#[tauri::command]
pub fn get_cows_in_the_plantation(state: tauri::State<'_, AppState>, date: NaiveDate) -> Result<Vec<Cow>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_cows_in_the_plantation(&mut conn, session.farm_id, &date)
}

#[tauri::command]
pub fn get_cows_born_on_a_given_birth(state: tauri::State<'_, AppState>, birth_id: i64) -> Result<Vec<Cow>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_cows_born_on_a_given_birth(&mut conn, session.farm_id, birth_id)
}

#[tauri::command]
pub fn get_cows_filtered(state: tauri::State<'_, AppState>, filter: CowFilter) -> Result<Vec<Cow>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_cows_filtered(&mut conn, session.farm_id, filter)
}

#[tauri::command]
pub fn get_birth(state: tauri::State<'_, AppState>, birth_id: i64) -> Result<Birth, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_birth(&mut conn, session.farm_id, birth_id)
}

#[tauri::command]
pub fn get_births(state: tauri::State<'_, AppState>) -> Result<Vec<Birth>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_births(&mut conn, session.farm_id)
}

#[tauri::command]
pub fn get_birth_by_mother_and_date(state: tauri::State<'_, AppState>, mother_id: i64, date: &str) -> Result<Birth, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_birth_by_mother_and_date(&mut conn, session.farm_id, mother_id, date)
}

#[tauri::command]
pub fn get_births_by_mother(state: tauri::State<'_, AppState>, mother_id: i64) -> Result<Vec<Birth>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_births_by_mother(&mut conn, session.farm_id, mother_id)
}

#[tauri::command]
pub fn get_insemination(state: tauri::State<'_, AppState>, insemination_id: i64) -> Result<Insemination, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_insemination(&mut conn, session.farm_id, insemination_id)
}

#[tauri::command]
pub fn get_inseminations(state: tauri::State<'_, AppState>) -> Result<Vec<Insemination>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_inseminations(&mut conn, session.farm_id)
}

#[tauri::command]
pub fn get_inseminations_by_dam(state: tauri::State<'_, AppState>, dam_id: i64) -> Result<Vec<Insemination>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_inseminations_by_dam(&mut conn, session.farm_id, dam_id)
}

#[tauri::command]
pub fn get_inseminations_by_sire(state: tauri::State<'_, AppState>, sire_id: i64) -> Result<Vec<Insemination>, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_inseminations_by_sire(&mut conn, session.farm_id, sire_id)
}

#[tauri::command]
pub fn get_insemination_by_dam_and_date(state: tauri::State<'_, AppState>, dam_id: i64, date: &str) -> Result<Insemination, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_insemination_by_dam_and_date(&mut conn, session.farm_id, dam_id, date)
}

#[tauri::command]
pub fn get_insemination_by_sire_and_date(state: tauri::State<'_, AppState>, sire_id: i64, date: &str) -> Result<Insemination, String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::get_insemination_by_sire_and_date(&mut conn, session.farm_id, sire_id, date)
}

#[tauri::command]
pub fn export_to_xlsx(state: tauri::State<'_, AppState>, path: &str, filter: CowFilter) -> Result<(), String> {
    let mut state_guard = state.session.lock().unwrap();
    let session = state_guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    service::export_to_xlsx(&mut conn, session.farm_id, path, filter)
}


#[tauri::command]
pub fn register_admin(
    state: tauri::State<'_, AppState>,
    farm_name: String,
    username: String,
    email: String,
    password: String,
) -> Result<User, String> {
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // 1. Run the registration service
    let user = auth_service::register_admin_and_farm(
        &mut conn,
        &farm_name,
        &username,
        &email,
        &password,
    )?;

    // 2. Persist active user to app_settings (auto-login)
    conn.execute(
        "UPDATE app_settings SET active_user_id = ?1 WHERE id = 1",
        rusqlite::params![user.id],
    ).map_err(|e| e.to_string())?;

    // 3. Initialize the RAM session
    let mut session_guard = state.session.lock().unwrap();
    *session_guard = Some(UserSession {
        user_id: user.id.unwrap(),
        farm_id: user.farm_id,
        command_manager: CommandManager::new()
    });

    Ok(user)
}


#[tauri::command]
pub fn login_user(
    state: tauri::State<'_, AppState>, 
    username: String,
    password: String,
) -> Result<User, String> {
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    let user = auth_service::authenticate_user(&mut conn, &username, &password)?;

    conn.execute(
        "UPDATE app_settings SET active_user_id = ?1 Where id = 1", 
        rusqlite::params![user.id]
    ).map_err(|e| e.to_string())?;

    let mut session_guard = state.session.lock().unwrap();
    *session_guard = Some(UserSession { 
        user_id: user.id.unwrap(),
        farm_id: user.farm_id,
        command_manager: CommandManager::new()
    });

    Ok(user)
}

#[tauri::command]
pub fn logout(state: tauri::State<'_, AppState>, keep_remembered: Option<bool>) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|e| e.to_string())?;
    let conn = state.db_pool.get().map_err(|e| e.to_string())?;
    crate::auth::remembered::end_session(&conn, keep_remembered.unwrap_or(false))?;
    *guard = None;
    Ok(())
}

#[tauri::command]
pub fn save_birth_with_calf(state: tauri::State<'_, AppState>, birth: Birth, calf_id: Option<i64>, previous_calf_id: Option<i64>) -> Result<i64, String> {
    let mut guard = state.session.lock().map_err(|_| "Sesiunea nu poate fi accesată.")?;
    let session = guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    service::save_birth_with_calf(&mut conn, &mut session.command_manager, session.farm_id, birth, calf_id, previous_calf_id)
}

#[tauri::command]
pub fn save_birth_with_calves(state: tauri::State<'_, AppState>, birth: Birth, calf_ids: Vec<i64>) -> Result<i64, String> {
    let mut guard = state.session.lock().map_err(|_| "Sesiunea nu poate fi accesată.")?;
    let session = guard.as_mut().ok_or("No session found")?;
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    require_writer(&conn, session)?;
    service::save_birth_with_calves(&mut conn, &mut session.command_manager, session.farm_id, birth, calf_ids)
}

#[cfg(test)]
mod tests {
    use super::{require_reference_in_farm, require_session_farm, require_writer, Reference};
    use crate::auth::session::UserSession;
    use crate::command::command_manager::CommandManager;
    use crate::database::database::create_tables;
    use rusqlite::Connection;

    fn session(user_id: i64, farm_id: i64) -> UserSession {
        UserSession { user_id, farm_id, command_manager: CommandManager::new() }
    }

    #[test]
    fn mutation_guard_checks_current_role_and_farm() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        conn.execute_batch(
            "INSERT INTO farms (id, name) VALUES (1, 'A'), (2, 'B');
             INSERT INTO users (id, username, email, password_hash, role, farm_id) VALUES
             (1, 'viewer', 'viewer@test.invalid', 'x', 'Viewer', 1),
             (2, 'editor', 'editor@test.invalid', 'x', 'Editor', 1),
             (3, 'admin', 'admin@test.invalid', 'x', 'Admin', 2);"
        ).unwrap();

        assert!(require_writer(&conn, &session(1, 1)).is_err());
        assert!(require_writer(&conn, &session(2, 1)).is_ok());
        assert!(require_writer(&conn, &session(3, 2)).is_ok());
        assert!(require_writer(&conn, &session(3, 1)).is_err());
        assert!(require_session_farm(&session(2, 1), 2).is_err());
        assert!(require_session_farm(&session(2, 1), 1).is_ok());

        conn.execute("UPDATE users SET role = 'Viewer' WHERE id = 2", []).unwrap();
        assert!(require_writer(&conn, &session(2, 1)).is_err());
    }

    #[test]
    fn related_records_must_belong_to_the_same_farm() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        conn.execute_batch(
            "INSERT INTO farms (id, name) VALUES (1, 'A'), (2, 'B');
             INSERT INTO cows (id, farm_id, ear_tag, sex, breed, category, birth_date, entry_date)
             VALUES (1, 1, 'A1', 'Female', 'Metis', 'Carne', '2020-01-01', '2020-01-01'),
                    (2, 2, 'B1', 'Female', 'Metis', 'Carne', '2020-01-01', '2020-01-01');
             INSERT INTO births (id, mother_id, date, farm_id)
             VALUES (10, 1, '2024-01-01', 1), (20, 2, '2024-01-01', 2);"
        ).unwrap();

        assert!(require_reference_in_farm(&conn, Reference::Cow, 1, 1).is_ok());
        assert!(require_reference_in_farm(&conn, Reference::Birth, 10, 1).is_ok());
        assert!(require_reference_in_farm(&conn, Reference::Cow, 2, 1).is_err());
        assert!(require_reference_in_farm(&conn, Reference::Birth, 20, 1).is_err());
        assert!(require_reference_in_farm(&conn, Reference::Birth, 999, 1).is_err());
    }
}
