//! Office Angerô: a digitação de fórmulas é sempre em português, em qualquer idioma da
//! interface; o motor, a API e o XLSX continuam com a sintaxe canônica em inglês.
use gridcraft_engine::{
    Session,
    core::{CellError, CellRef, Value},
};
use gridcraft_ui_egui::{SheetApp, i18n::Language};
use egui_kittest::kittest::Queryable as _;
use serde_json::json;

fn app() -> SheetApp {
    let mut session = Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, Default::default());
    app.ui.language = Language::PtBr;
    app
}
fn formula(app: &SheetApp) -> String {
    app.session.active().unwrap().wb.active().unwrap().cell(CellRef::new(0, 0)).unwrap().formula.as_ref().unwrap().text.to_string()
}
fn value(app: &SheetApp) -> Value {
    app.session.active().unwrap().wb.active().unwrap().value(CellRef::new(0, 0))
}
fn enter(app: &mut SheetApp, text: &str) -> bool {
    app.begin_edit(Some(text.into()), false);
    app.commit_edit(0, 0, false, false)
}

#[test]
fn portuguese_entry_display_and_xlsx_roundtrip() {
    let mut app = app();
    assert!(enter(&mut app, "=SE(VERDADEIRO;SOMA(1,5;2,25);0)"));
    assert_eq!(formula(&app), "IF(TRUE,SUM(1.5,2.25),0)");
    let saved = app.run("file.save", json!({"format":"xlsx"})).unwrap();
    app.run("file.open", json!({"name":"locale.xlsx", "base64":saved["base64"]})).unwrap();
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=SE(VERDADEIRO;SOMA(1,5;2,25);0)");
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), "IF(TRUE,SUM(1.5,2.25),0)");
}

#[test]
fn every_interface_language_types_formulas_in_portuguese() {
    for code in ["en", "es", "cs-CZ", "pt-BR"] {
        let mut app = app();
        app.run("app.language.set", json!({"code":code})).unwrap();
        assert!(enter(&mut app, "=MÉDIA(2;4)+PROCV(1;{1\\5};2;FALSO)"), "{code}");
        assert_eq!(formula(&app), "AVERAGE(2,4)+VLOOKUP(1,{1,5},2,FALSE)", "{code}");
        assert_eq!(value(&app), Value::Number(8.0), "{code}");
        app.begin_edit(None, true);
        assert_eq!(app.editor.as_ref().unwrap().text, "=MÉDIA(2;4)+PROCV(1;{1\\5};2;FALSO)", "{code}");
        app.cancel_edit();
    }
}

#[test]
fn api_syntax_stays_canonical() {
    let mut app = app();
    assert!(app.run("cell.set", json!({"cell":"A1","input":"=SUM(1,2)"})).is_ok());
    assert!(app.run("cell.set", json!({"cell":"A2","input":"=SOMA(1;2)"})).is_err());
}

#[test]
fn defined_and_local_names_survive_localized_editing() {
    let mut app = app();
    app.run("formulas.defineName", json!({"name":"FALSO", "refersTo":"=7"})).unwrap();
    assert!(enter(&mut app, "=LET(SOMA;LAMBDA(x;x+1);SOMA(2))+FALSO"));
    let original = formula(&app);
    app.begin_edit(None, false);
    assert_eq!(app.editor.as_ref().unwrap().text, "=LET(SOMA;LAMBDA(x;x+1);SOMA(2))+FALSO");
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), original);
    assert_eq!(value(&app), Value::Number(10.0));
}

#[test]
fn localized_argument_hints_ignore_decimal_commas_arrays_and_nested_calls() {
    let mut app = app();
    for (text, expected) in [("=SE(1,5;SOMA(2;3);", 2), ("=SOMA({1,5\\2;3\\4};", 1), ("=SOMA(Table1[[a],[b]];", 1)] {
        app.begin_edit(Some(text.into()), false);
        assert_eq!(app.editor.as_ref().unwrap().current_function().unwrap().1, expected);
    }
}

#[test]
fn autocomplete_offers_only_portuguese_names() {
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        app(),
    );
    h.run_steps(4);
    h.state_mut().begin_edit(Some("=SO".into()), true);
    h.run_steps(4);
    h.input_mut().events.push(egui::Event::Text("M".into()));
    h.run_steps(4);
    let names = h.state().editor.as_ref().unwrap().autocomplete.clone();
    assert!(names.iter().any(|n| n == "SOMA"), "{names:?}");
    assert!(names.iter().any(|n| n == "SOMASE"), "{names:?}");
    if let Ok(path) = std::env::var("GRIDCRAFT_LOCALE_SCREENSHOT") {
        h.render().unwrap().save(path).unwrap();
    }
    let ed = h.state_mut().editor.as_mut().unwrap();
    ed.ac_index = ed.autocomplete.iter().position(|n| n == "SOMA").unwrap();
    assert!(ed.accept_autocomplete());
    assert_eq!(ed.text, "=SOMA(");
    h.run_steps(4);
    h.input_mut().events.push(egui::Event::Text("1,5;".into()));
    h.run_steps(4);
    assert_eq!(h.state().editor.as_ref().unwrap().current_function(), Some(("SOMA".into(), 1)));
    h.state_mut().cancel_edit();
    h.state_mut().begin_edit(Some("=SU".into()), true);
    h.run_steps(4);
    h.input_mut().events.push(egui::Event::Text("M".into()));
    h.run_steps(4);
    let names = h.state().editor.as_ref().unwrap().autocomplete.clone();
    assert!(!names.iter().any(|n| n == "SUM" || n == "SUMIF"), "{names:?}");
}

#[test]
fn numbers_use_decimal_comma_and_thousand_dots() {
    let mut app = app();
    for (text, expected) in [("-1,5", -1.5), ("1.500", 1500.0), ("1.234.567,89", 1234567.89), ("12,5%", 0.125)] {
        assert!(enter(&mut app, text), "{text}");
        assert_eq!(value(&app), Value::Number(expected), "{text}");
    }
    assert!(enter(&mut app, "=SOMA(1,5;2"));
    assert_eq!(formula(&app), "SUM(1.5,2)");
    app.run("home.numberFormat", json!({"format":"@"})).unwrap();
    assert!(enter(&mut app, "=SUM(1,2)"));
    assert_eq!(value(&app), Value::text("=SUM(1,2)"));
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=SUM(1,2)");
}

#[test]
fn engine_templates_enter_in_portuguese() {
    let mut app = app();
    app.run("formulas.insertFunction", json!({"name":"SUM"})).unwrap();
    assert_eq!(app.editor.as_ref().unwrap().text, "=SOMA(");
    app.cancel_edit();
    app.run("cell.set", json!({"cell":"A1","input":"2"})).unwrap();
    app.run("selection.set", json!({"range":"A2"})).unwrap();
    app.run("formulas.autoSum", json!({"enter":false})).unwrap();
    assert!(app.editor.as_ref().unwrap().text.starts_with("=SOMA("));
    app.cancel_edit();
    // Texto digitado já está em português; não pode ser traduzido duas vezes.
    app.begin_edit(Some("=SOMA(1,5;2)".into()), false);
    assert_eq!(app.editor.as_ref().unwrap().text, "=SOMA(1,5;2)");
    assert!(app.commit_edit(0, 0, false, false));
}

#[test]
fn user_defined_prefix_is_hidden_while_editing_and_kept_on_commit() {
    // Excel shows `_xludf.EOMONTH()` as `=EOMONTH()`; the cell still holds the user-defined call
    // (#726), and committing the edit unchanged must not turn it into the built-in.
    let mut app = app();
    app.run("cell.set", json!({"cell":"A1","input":"=_xludf.EOMONTH()&_xludf.CONCAT(\"_xludf.x\")"})).unwrap();
    app.begin_edit(None, true);
    assert_eq!(app.editor.as_ref().unwrap().text, "=EOMONTH()&CONCAT(\"_xludf.x\")");
    assert!(app.commit_edit(0, 0, false, false));
    assert_eq!(formula(&app), "_xludf.EOMONTH()&_xludf.CONCAT(\"_xludf.x\")");
    assert_eq!(value(&app), Value::Error(CellError::Name));
}

#[test]
fn closing_the_window_with_unsaved_changes_asks_first() {
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        app(),
    );
    h.run_steps(4);
    // Sem alterações: fecha sem perguntar.
    h.input_mut().viewports.entry(egui::ViewportId::ROOT).or_default().events.push(egui::ViewportEvent::Close);
    h.run_steps(2);
    assert!(h.state().dialog.is_none());
    h.state_mut().run("cell.set", json!({"cell":"A1","input":"5"})).unwrap();
    h.input_mut().viewports.entry(egui::ViewportId::ROOT).or_default().events.push(egui::ViewportEvent::Close);
    h.run_steps(4);
    assert_eq!(h.state().dialog.as_ref().map(|d| d.name.as_str()), Some("quit"));
    h.get_by_label("Não Salvar").click();
    h.run_steps(2);
    assert!(h.state().quit_confirmed);
}
