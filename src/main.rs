#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::collections::HashMap;
use std::ops::Sub;
use std::sync::mpsc::{channel, Receiver, Sender, SyncSender};
use std::thread;

use api::ApiState;
use eframe::egui::{Button, Separator, Vec2};
use eframe::{egui, App};
use egui::Response;
use egui::ScrollArea;
use egui::TopBottomPanel;
use egui::{Align, CentralPanel};
use num_format::{Locale, ToFormattedString};
use serde::Deserialize;

mod api;

/* NOTAS
 * 2023-01-11
 * Aparentemente no existe algo como un MVC para App nativas de Rust
 * Donde se puedan manejar multiples ventanas cada una con su estado
 * y que puedan hablar entre ellas.
 * Algunos dicen que "egui" sirve para aplicaciones de 1 sola ventana
 * y segun la investigacion que hice todo el mundo hace eso, lo que mas
 * se acerca serian ventanas nuevas (usando Renderizado Condicional)
 * que se crean pero que no contienen
 * un estado propio. Ademas, no parece haber algun sistema de navegacion
 * como reactNavigation o algo que se pueda hacer con routes
 *
 * Toda la logica de egui debera quedar aqui, otra logica como consumir API puede
 * quedar en otro archivo y solo pasar los vectores a las funciones que correspondan
*/

// Al parecer para usar egui necesitamos crear un struct y implementar los metodos
// que egui pide
pub struct Scpp {
    route: String,
    is_logged: bool,
    login_form: LoginForm,
    api_prefix: String,
    docs: Vec<Documento>,
    selected_tipo_doc: String,
    tipo_doc_list: HashMap<String, String>,
}
pub struct LoginForm {
    username: String,
    pass: String,
}

#[derive(Deserialize, Debug)]
// En ingresos y ahorro descripcion puede ser null, por eso lo ponemos en un Option<>
pub struct Categoria {
    descripcion: Option<String>,
}
#[derive(Deserialize, Debug)]
pub struct TipoDoc {
    descripcion: Option<String>,
}
// Marcamos como que se puede deserializar, porque el ureq convertira el JSON a este obj
// Ademas, las propiedades que tienen nombres CamelCase no son compatibles con ruts que usa snake_
// agregamos propiedad indicando el nombre que viene en la API
#[derive(Deserialize, Debug)]
pub struct Documento {
    id: u32,
    #[serde(alias = "fk_tipoDoc")]
    fk_tipo_doc: String,
    proposito: String,
    monto: i32,
    fecha: String,
    fk_categoria: Option<String>,
    categoria: Categoria,
    #[serde(alias = "tipoDoc")]
    tipo_doc: TipoDoc,
}

pub const PADDING: f32 = 5.0;

// impl es como extends en Java al parecer, para obtener
// propiedades de la clase padre
impl App for Scpp {
    // Aqui se definen los widgets y se manejan los eventos de estos widgets
    // Como este es el main APP, aqui se definiran las rutas para las ventanas que debe abrir etc.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if &self.route == "/login" {
            CentralPanel::default().show(ctx, |ui| {
                // Username Input
                ui.label("Enter your username");
                ui.text_edit_singleline(&mut self.login_form.username);
                // Asi para saber que se salieron de un input
                //if username_input.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {

                // Password Input
                ui.label("Enter your Password");
                ui.text_edit_singleline(&mut self.login_form.pass);

                // Space
                ui.add_space(10.);

                // Button
                let login_btn: Response = ui.button("Login");
                if login_btn.clicked() {
                    if &self.login_form.pass == "admin" && &self.login_form.username == "admin" {
                        self.route = String::from("/docs");
                    }
                }
            });
        }
        if &self.route == "/docs" {
            CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(PADDING);

                    egui::Grid::new("top").show(ui, |ui| {
                        ui.heading("Docs");
                        let refresh_btn = ui.add(Button::new("🔄"));
                        if refresh_btn.clicked() {
                            let (mut tx, rx) = channel();

                            // Hacemos la llamada a la API en otro thread pero recibimos en el main
                            // por eso no necesitamos thread::spawn para hacer rx.recv()
                            // Si hacemos rx.recv() en thread::spawn luego no podremos self.docs = value.docs
                            // porque las variables estan en otros hilos

                            // Necesitamos clonar apiPrefix, porque sino da error borrowed y outlive
                            let cloned_api_prefix = format!("{}", &self.api_prefix);
                            let cloned_selected_tipo_doc = format!("{}", &self.selected_tipo_doc);

                            let sender = thread::spawn(move || {
                                api::get_docs(&mut tx, cloned_api_prefix, cloned_selected_tipo_doc)
                            });

                            // NOTA. si tratamos de recibir y no han enviado nada se cae
                            let value = rx.recv().expect("Unable to receive from channel");
                            self.docs = value.docs;

                            sender.join().expect("The sender thread has panicked");
                        }
                    });
                    // Separador luego de la barra de opciones
                    ui.add_space(PADDING);
                    let sep = Separator::default().spacing(20.);
                    ui.add(sep);
                });

                // Select de los tipo de Doc
                egui::Grid::new("tipo_doc_label").show(ui, |ui| {
                    ui.label("Tipo Doc");
                    egui::ComboBox::from_label("")
                        .selected_text(format!(
                            "{}",
                            &self.tipo_doc_list.get(&self.selected_tipo_doc).unwrap()
                        ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_tipo_doc,
                                String::from("1"),
                                format!("{}", &self.tipo_doc_list.get("1").unwrap()),
                            );
                            ui.selectable_value(
                                &mut self.selected_tipo_doc,
                                String::from("2"),
                                format!("{}", &self.tipo_doc_list.get("2").unwrap()),
                            );
                            ui.selectable_value(
                                &mut self.selected_tipo_doc,
                                String::from("3"),
                                format!("{}", &self.tipo_doc_list.get("3").unwrap()),
                            );
                        });
                });
                ui.add_space(PADDING);

                // Area de los documentos. Existe algo que es show_rows, pero parece que obliga a usar un parametro que no
                // queremos usar jajaja
                ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("docs_data")
                        .num_columns(3)
                        //.striped(true)
                        .show(ui, |ui| {
                            // Titulo de la Tabla
                            ui.label(String::from("Fecha"));
                            ui.label(String::from("Proposito"));
                            // Para alinear a la derecha
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                ui.label(String::from("Monto"));
                            });
                            ui.end_row();

                            for a in self.docs.iter() {
                                ui.label(&a.fecha);
                                let select_item = ui.selectable_label(false, &a.proposito);

                                // Para alinear a la derecha
                                // Grid no toma en cuenta esto y el striped no llega donde debe
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(&a.monto.to_formatted_string(&Locale::es));
                                });
                                ui.end_row();

                                // Editar item cuando hacen clic en el proposito
                                if select_item.clicked() {
                                    println!("{}", a.id);
                                }
                            }
                        });
                });
            });
        }
        if &self.route == "/config" {
            CentralPanel::default().show(ctx, |ui| {
                ui.label("Config");
            });
        }
    }
}

// https://www.youtube.com/watch?v=NtUkr_z7l84
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let mut tipo_doc_list = HashMap::new();
    tipo_doc_list.insert(String::from("1"), String::from("Gasto"));
    tipo_doc_list.insert(String::from("2"), String::from("Ahorro"));
    tipo_doc_list.insert(String::from("3"), String::from("Ingreso"));

    let mut app = Scpp {
        route: "/docs".to_string(),
        is_logged: false,
        login_form: LoginForm {
            username: String::new(),
            pass: String::new(),
        },
        api_prefix: String::from("http://localhost:3000"),
        docs: Vec::new(),
        selected_tipo_doc: String::from("1"),
        tipo_doc_list: tipo_doc_list,
    };

    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(540., 960.));
    native_options.resizable = false;

    eframe::run_native("SCPP", native_options, Box::new(|cc| Box::new(app)));
}
