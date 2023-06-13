use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use smol_db_client::Client;
use smol_db_client::client_error::ClientError;
use crate::app::ProgramState::{ClientConnectionError, DisplayClient, PromptForClientDetails};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    #[serde(skip)]
    client: Arc<Mutex<Option<Client>>>,
    #[serde(skip)]
    program_state: Arc<Mutex<ProgramState>>,

    ip_address: String,

    #[serde(skip)]
    client_key: String,

    #[serde(skip)]
    connection_thread: Option<JoinHandle<()>>,
}

#[derive(Debug)]
enum ProgramState {
    NoClient,
    PromptForClientDetails,
    ClientConnectionError(ClientError),
    DisplayClient,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            program_state: Arc::new(Mutex::new(ProgramState::NoClient)),
            ip_address: "".to_string(),
            client_key: "".to_string(),
            connection_thread: None,
        }
    }
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { .. } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
                ui.menu_button("Connect", |ui| {
                    if ui.button("Connect").clicked() {
                        *self.program_state.lock().unwrap().deref_mut() = PromptForClientDetails;
                    }
                    if ui.button("Disconnect").clicked() {
                        let mut lock = self.program_state.lock().unwrap();
                        match *lock {
                            ProgramState::NoClient => {}
                            PromptForClientDetails => {}
                            ClientConnectionError(_) => {}
                            DisplayClient => {
                                // if we are displaying the client, we can allow the user to click disconnect.
                                match self.client.lock().unwrap().as_ref() {
                                    None => {}
                                    Some(cl) => {
                                        cl.disconnect().expect("Unable to disconnect from client");
                                    }
                                }
                                *lock = ProgramState::NoClient;
                            }
                        }
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            // TODO: finish
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(debug_assertions)]
            ui.label(format!("DEBUG Program State: {:?}", self.program_state.lock().unwrap()));

            match self.program_state.lock().unwrap().deref() {
                ProgramState::NoClient => {
                    // Display nothing when there are is no client connection.
                    ui.label("Nothing to show here...");
                }
                PromptForClientDetails => {
                    // When the user clicks connect, we prompt them for client connection details.
                    ui.label("Enter Ip Address:");
                    ui.text_edit_singleline(&mut self.ip_address);

                    if ui.button("Connect to ip address.").clicked() {

                        // clone a bunch of things that need to be moved into the thread.
                        let client_clone = Arc::clone(&self.client);
                        let program_state_clone = Arc::clone(&self.program_state);
                        let ip_clone = self.ip_address.clone();
                        self.connection_thread = Some(thread::spawn(move || {
                            // instantly move all the variables into the thread
                            let ps = program_state_clone;
                            let client_mutex = client_clone;
                            let ip = ip_clone;

                            match Client::new(&ip) { // connect the client to the server.
                                Ok(client_connection) => {
                                    // if client connection successful, move the client to the programs state.
                                    *client_mutex.lock().unwrap() = Some(client_connection);
                                    // change the program state into a DisplayClient state.
                                    *ps.lock().unwrap() = DisplayClient;
                                }
                                Err(err) => {
                                    // if the client connection fails, change the program state accordingly.
                                    *ps.lock().unwrap() = ClientConnectionError(err);
                                }
                            }
                        }));

                    }
                }
                DisplayClient => {
                    // TODO: finish
                }
                ClientConnectionError(err) => {
                    ui.label("Client connection error");
                    ui.label(format!("{:?}",err));
                }
            }

            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
