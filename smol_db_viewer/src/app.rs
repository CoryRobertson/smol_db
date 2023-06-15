use crate::app::ContentCacheState::{Cached, NotCached};
use crate::app::ProgramState::{
    ClientConnectionError, DisplayClient, NoClient, PromptForClientDetails, PromptForKey,
};
use smol_db_client::client_error::ClientError;
use smol_db_client::{Client, Role};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ApplicationState {
    #[serde(skip)]
    client: Arc<Mutex<Option<Client>>>,
    #[serde(skip)]
    program_state: Arc<Mutex<ProgramState>>,

    ip_address: String,

    #[serde(skip)]
    database_list: Option<Vec<DBCached>>,

    client_key: String,

    #[serde(skip)]
    selected_database: Option<u32>,

    #[serde(skip)]
    connection_thread: Option<JoinHandle<()>>,

    #[serde(skip)]
    key_input: String,

    #[serde(skip)]
    value_input: String,

    #[serde(skip)]
    desired_action: DesiredAction,
}

#[derive(Debug)]
enum ContentCacheState<T> {
    NotCached,
    Cached(T),
    Error(ClientError),
}

#[derive(Debug)]
enum DesiredAction {
    Write,
    Delete,
}

impl DesiredAction {
    fn as_text(&self) -> &str {
        match self {
            DesiredAction::Write => "Write",
            DesiredAction::Delete => "Delete",
        }
    }
}

#[derive(Debug)]
struct DBCached {
    name: String,
    content: ContentCacheState<HashMap<String, String>>,
    role: ContentCacheState<Role>,
}

#[derive(Debug)]
enum ProgramState {
    NoClient,
    PromptForClientDetails,
    ClientConnectionError(ClientError),
    PromptForKey,
    DisplayClient,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            program_state: Arc::new(Mutex::new(NoClient)),
            ip_address: "".to_string(),
            database_list: None,
            client_key: "".to_string(),
            selected_database: None,
            connection_thread: None,
            key_input: "".to_string(),
            value_input: "".to_string(),
            desired_action: DesiredAction::Write,
        }
    }
}

impl ApplicationState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for ApplicationState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { .. } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
                ui.menu_button("Client", |ui| {
                    if ui.button("Connect").clicked() {
                        *self.program_state.lock().unwrap().deref_mut() = PromptForClientDetails;
                    }
                    if ui.button("Disconnect").clicked() {
                        let mut lock = self.program_state.lock().unwrap();
                        match self.client.lock().unwrap().as_ref() {
                            None => {}
                            Some(cl) => {
                                let _ = cl.disconnect();
                            }
                        }
                        *lock = NoClient;
                        self.database_list = None;
                        *self.client.lock().unwrap() = None;
                    }
                    if ui.button("Set key").clicked() {
                        let mut lock = self.program_state.lock().unwrap();
                        match *lock {
                            NoClient => {}
                            PromptForClientDetails => {}
                            ClientConnectionError(_) => {}
                            PromptForKey => {}
                            DisplayClient => {
                                *lock = PromptForKey;
                            }
                        }
                    }

                    // TODO: add a DBSettings manager here that lets the client formulate a new DBSettings to send to the server, also should display current DBSettings.

                    if ui.button("Refresh stored data").clicked() {
                        *self.client.lock().unwrap() = None;
                        *self.program_state.lock().unwrap() = NoClient;

                        self.database_list = None;

                        self.selected_database = None;
                        self.connection_thread = None;
                    }
                });
                if self.client.lock().unwrap().is_some() {
                    if ui.button("Refresh DB List").clicked() {
                        self.database_list = None;
                    }
                }
            });
        });

        {
            let mut lock = self.program_state.lock().unwrap();
            match *lock {
                NoClient => {}
                PromptForClientDetails => {}
                ClientConnectionError(_) => {}
                PromptForKey => {}
                DisplayClient => {
                    egui::TopBottomPanel::bottom("side_panel2").show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Input:");
                            ui.text_edit_singleline(&mut self.key_input);
                            ui.text_edit_singleline(&mut self.value_input);
                            if ui.button("Submit").clicked() {
                                match self.selected_database {
                                    None => {}
                                    Some(index) => match &mut self.database_list {
                                        None => {}
                                        Some(list) => match list.get_mut(index as usize) {
                                            None => {}
                                            Some(db) => {
                                                let mut client_lock = self.client.lock().unwrap();
                                                match *client_lock {
                                                    None => {}
                                                    Some(ref mut client) => {
                                                        match self.desired_action {
                                                            DesiredAction::Write => {
                                                                match client.write_db(
                                                                    db.name.as_str(),
                                                                    self.key_input.as_str(),
                                                                    self.value_input.as_str(),
                                                                ) {
                                                                    Ok(_) => {}
                                                                    Err(err) => {
                                                                        *lock = ClientConnectionError(err);
                                                                    }
                                                                }
                                                            }
                                                            DesiredAction::Delete => {
                                                                match client.delete_data(
                                                                    db.name.as_str(),
                                                                    self.key_input.as_str(),
                                                                ) {
                                                                    Ok(resp) => {
                                                                        #[cfg(debug_assertions)]
                                                                        println!("{:?}", resp);
                                                                    }
                                                                    Err(err) => {
                                                                        #[cfg(debug_assertions)]
                                                                        println!("{:?}",err);
                                                                        *lock = ClientConnectionError(err);
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        match client.list_db_contents(db.name.as_str()) {
                                                            Ok(data) => {
                                                                db.content =
                                                                    Cached(data);
                                                            }
                                                            Err(err) => {
                                                                *lock = ClientConnectionError(err);
                                                            }
                                                        }

                                                    }
                                                }
                                            }
                                        },
                                    },
                                }
                            }

                            if ui.button(format!("{}", self.desired_action.as_text())).clicked() {
                                match self.desired_action {
                                    DesiredAction::Write => {
                                        self.desired_action = DesiredAction::Delete;
                                    }
                                    DesiredAction::Delete => {
                                        self.desired_action = DesiredAction::Write;
                                    }
                                }
                            }
                        });
                    });
                }
            }
        }

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            match self.program_state.lock().unwrap().deref() {
                NoClient => {}
                PromptForClientDetails => {}
                ClientConnectionError(_) => {}
                // side menu that is persistent when displaying the client data.
                DisplayClient => {
                    if let Some(selected_db) = self.selected_database {
                        if let Some(db_list) = &self.database_list {
                            if let Some(db) = db_list.get(selected_db as usize) {
                                ui.label(format!("Selected DB: {:?}", db.name));
                                match &db.role {
                                    NotCached => {}
                                    Cached(role) => {
                                        ui.label(format!("Role: {:?}", role));
                                    }
                                    ContentCacheState::Error(err) => {
                                        ui.label(format!("Role: {:?}", err));
                                    }
                                }
                            }
                        }
                    }
                    match &mut self.database_list {
                        None => {}
                        Some(list) => {
                            for (index, item) in list.iter_mut().enumerate() {
                                if ui.button(format!("{}: {}", index + 1, item.name)).clicked() {
                                    let mut lock = self.client.lock().unwrap();
                                    match *lock {
                                        None => {}
                                        Some(ref mut client) => {
                                            // cache the content if it is not cached.
                                            match item.content {
                                                NotCached => {
                                                    match client
                                                        .list_db_contents(item.name.as_str())
                                                    {
                                                        Ok(data) => {
                                                            item.content = Cached(data);
                                                        }
                                                        Err(err) => {
                                                            item.content =
                                                                ContentCacheState::Error(err);
                                                        }
                                                    }
                                                }
                                                Cached(_) => {}
                                                ContentCacheState::Error(_) => {}
                                            }

                                            // cache the role if it is not cached.
                                            match item.role {
                                                NotCached => {
                                                    match client.get_role(item.name.as_str()) {
                                                        Ok(role) => item.role = Cached(role),
                                                        Err(err) => {
                                                            item.role =
                                                                ContentCacheState::Error(err);
                                                        }
                                                    }
                                                }
                                                Cached(_) => {}
                                                ContentCacheState::Error(_) => {}
                                            }

                                            // set the selected database number in the program state.
                                            self.selected_database = Some(index as u32);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                PromptForKey => {}
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(debug_assertions)]
            ui.label(format!(
                "DEBUG Program State: {:?}",
                self.program_state.lock().unwrap()
            ));
            let mut ps_lock = self.program_state.lock().unwrap();
            match ps_lock.deref() {
                NoClient => {
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

                            match Client::new(&ip) {
                                // connect the client to the server.
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
                    match &mut self.database_list {
                        // get the database list if it is not known
                        None => {
                            let mut lock = self.client.lock().unwrap();
                            match *lock {
                                None => {}
                                Some(ref mut client) => match client.list_db() {
                                    Ok(list) => {
                                        self.database_list = Some(
                                            list.iter()
                                                .map(|db_packet| DBCached {
                                                    name: db_packet.get_db_name().to_string(),
                                                    content: NotCached,
                                                    role: NotCached,
                                                })
                                                .collect(),
                                        )
                                    }
                                    Err(err) => {
                                        *self.program_state.lock().unwrap() =
                                            ClientConnectionError(err);
                                    }
                                },
                            }
                        }
                        // db list exists, populate its information on screen.
                        Some(list) => {
                            if let Some(index_selected) = self.selected_database {
                                if let Some(db_cached) = list.get(index_selected as usize) {
                                    match &db_cached.content {
                                        NotCached => {}
                                        Cached(data) => {
                                            let mut list = data
                                                .iter()
                                                .map(|(s1, s2)| (s1.to_string(), s2.to_string()))
                                                .collect::<Vec<(String, String)>>();
                                            list.sort();
                                            for (key, value) in list {
                                                ui.label(format!("{} : {}", key, value));
                                            }
                                        }
                                        ContentCacheState::Error(err) => {
                                            ui.label(format!("{:?}", err));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ClientConnectionError(err) => {
                    ui.label("Client connection error");
                    ui.label(format!("{:?}", err));
                }
                PromptForKey => {
                    ui.label("Enter Key:");
                    ui.text_edit_singleline(&mut self.client_key);
                    if ui.button("Set Key").clicked() {
                        let mut lock = self.client.lock().unwrap();
                        match *lock {
                            None => {}
                            Some(ref mut client) => {
                                match client.set_access_key(self.client_key.clone()) {
                                    Ok(_) => {
                                        *ps_lock = DisplayClient;
                                    }
                                    Err(err) => {
                                        *ps_lock = ClientConnectionError(err);
                                    }
                                }
                            }
                        }
                    }
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
