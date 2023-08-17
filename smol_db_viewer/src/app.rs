use crate::app::ContentCacheState::{Cached, NotCached};
use crate::app::ProgramState::{
    ChangeDBSettings, ClientConnectionError, CreateDB, DBResponseError, DisplayClient, NoClient,
    PromptForClientDetails, PromptForKey,
};
use smol_db_client::client_error::ClientError;
use smol_db_client::client_error::ClientError::BadPacket;
use smol_db_client::db_settings::DBSettings;
use smol_db_client::prelude::DBStatistics;
use smol_db_client::DBSuccessResponse;
use smol_db_client::{DBPacketResponseError, Role, SmolDbClient};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use chrono::{Datelike, DateTime, Local, Timelike};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ApplicationState {
    #[serde(skip)]
    client: Arc<Mutex<Option<SmolDbClient>>>,
    #[serde(skip)]
    program_state: Arc<Mutex<ProgramState>>,

    ip_address: String,

    #[serde(skip)]
    database_list: Option<Vec<DBCached>>,

    client_key: String,

    #[serde(skip)]
    selected_database: Option<usize>,

    #[serde(skip)]
    connection_thread: Option<JoinHandle<()>>,

    #[serde(skip)]
    key_input: String,

    #[serde(skip)]
    value_input: String,

    #[serde(skip)]
    desired_action: DesiredAction,

    #[serde(skip)]
    submit_db_settings: DBSettings,

    #[serde(skip)]
    duration_seconds: u64,

    #[serde(skip)]
    users_list: String,

    #[serde(skip)]
    admins_list: String,

    #[serde(skip)]
    db_name_create: String,

    auto_connect: bool,

    auto_set_key: bool,
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
    const fn as_text(&self) -> &str {
        match self {
            Self::Write => "Write",
            Self::Delete => "Delete",
        }
    }
}

#[derive(Debug)]
struct DBCached {
    name: String,
    content: ContentCacheState<HashMap<String, String>>,
    role: ContentCacheState<Role>,
    db_settings: ContentCacheState<DBSettings>,
    statistics: ContentCacheState<DBStatistics>,
}

#[derive(Debug)]
enum ProgramState {
    NoClient,
    PromptForClientDetails,
    ClientConnectionError(ClientError),
    #[allow(dead_code)]
    DBResponseError(DBPacketResponseError),
    PromptForKey,
    ChangeDBSettings,
    CreateDB,
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
            submit_db_settings: DBSettings::default(),
            duration_seconds: 30,
            users_list: "".to_string(),
            admins_list: "".to_string(),
            db_name_create: "".to_string(),
            auto_connect: false,
            auto_set_key: false,
        }
    }
}

impl ApplicationState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let mut loaded_state: Self =
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            if loaded_state.auto_connect && !loaded_state.ip_address.is_empty() {
                let client_clone = Arc::clone(&loaded_state.client);
                let program_state_clone = Arc::clone(&loaded_state.program_state);
                let ip_clone = loaded_state.ip_address.clone();
                let set_key_clone = loaded_state.auto_set_key;
                let key_set_clone = loaded_state.client_key.clone();
                loaded_state.connection_thread = Some(thread::spawn(move || {
                    // instantly move all the variables into the thread
                    let set_key = set_key_clone;
                    let ps = program_state_clone;
                    let client_mutex = client_clone;
                    let ip = ip_clone;
                    let key = key_set_clone;

                    match SmolDbClient::new(&ip) {
                        // connect the client to the server.
                        Ok(mut client_connection) => {
                            if set_key && !key.is_empty() {
                                // if the auto set key flag is true, and the users key is not empty
                                // attempt to set the clients key
                                match client_connection.set_access_key(key) {
                                    Ok(set_key_resp) => {
                                        match set_key_resp {
                                            DBSuccessResponse::SuccessNoData => {
                                                // if the client set key was successful, then display the client to the user and pass the client connection to the program
                                                *client_mutex.lock().unwrap() =
                                                    Some(client_connection);
                                                *ps.lock().unwrap() = DisplayClient;
                                            }
                                            DBSuccessResponse::SuccessReply(_) => {
                                                // the set access key function for the client should never reply with data, if it did, then the packet sent was bad in some way.
                                                *ps.lock().unwrap() =
                                                    ClientConnectionError(BadPacket);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        // if we are unable to set the access key due to a client error,
                                        // we do not pass the client to the program, and we display the connection error.
                                        *ps.lock().unwrap() = ClientConnectionError(err);
                                    }
                                }
                            } else {
                                // this else case is for when the user does not have auto connect on.
                                // if client connection successful, move the client to the programs state.
                                *client_mutex.lock().unwrap() = Some(client_connection);
                                // change the program state into a DisplayClient state.
                                *ps.lock().unwrap() = DisplayClient;
                            }
                        }
                        Err(err) => {
                            // if the client connection fails, change the program state accordingly.
                            *ps.lock().unwrap() = ClientConnectionError(err);
                        }
                    }
                }));
            }

            return loaded_state;
        }

        Self::default()
    }
}

impl eframe::App for ApplicationState {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self { .. } = self;

        // top panel block
        {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    let has_client = self.client.lock().unwrap().is_some();
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            frame.close();
                        }
                    });
                    ui.separator();
                    ui.menu_button("Client", |ui| {
                        if ui.button("Connect").clicked() {
                            *self.program_state.lock().unwrap().deref_mut() =
                                PromptForClientDetails;
                        }
                        if has_client {
                            ui.separator();
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
                            ui.separator();
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
                                    ChangeDBSettings => {
                                        *lock = PromptForKey;
                                    }
                                    CreateDB => {
                                        *lock = PromptForKey;
                                    }
                                    DBResponseError(_) => {}
                                }
                            }
                            ui.separator();
                            if ui.button("DB Settings").clicked() {
                                *self.program_state.lock().unwrap() = ChangeDBSettings;
                            }
                            ui.separator();
                            if ui.button("Create DB").clicked() {
                                *self.program_state.lock().unwrap() = CreateDB;
                            }
                        }
                        ui.separator();
                        if ui.button("Refresh stored data").clicked() {
                            *self.client.lock().unwrap() = None;
                            *self.program_state.lock().unwrap() = NoClient;

                            self.database_list = None;

                            self.selected_database = None;
                            self.connection_thread = None;
                        }
                    });
                    if has_client {
                        ui.separator();
                        if ui.button("Refresh").clicked() {
                            self.database_list = None;
                            self.selected_database = None;
                        }
                    }
                });
            });
        }

        // bottom panel block
        {
            let mut lock = self.program_state.lock().unwrap();
            match *lock {
                NoClient => {}
                PromptForClientDetails => {}
                ClientConnectionError(_) => {}
                PromptForKey => {}
                DisplayClient => {
                    if self.selected_database.is_some() && self.database_list.is_some() {
                        egui::TopBottomPanel::bottom("side_panel2").show(ctx, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Input:");

                                    ui.add_sized([160.0, 20.0], egui::TextEdit::singleline(&mut self.key_input));
                                    ui.add_sized([160.0, 20.0], egui::TextEdit::singleline(&mut self.value_input));
                                    if ui.button("Submit").clicked() {
                                        match self.selected_database {
                                            None => {}
                                            Some(index) => match &mut self.database_list {
                                                None => {}
                                                Some(list) => match list.get_mut(index) {
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
                                                                            Ok(response) => {
                                                                                match response {
                                                                                    DBSuccessResponse::SuccessNoData => {}
                                                                                    DBSuccessResponse::SuccessReply(_) => {}
                                                                                }
                                                                            }
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
                                                                            #[allow(unused_variables)]
                                                                            Ok(resp) => {
                                                                                #[cfg(debug_assertions)]
                                                                                println!("{:?}", resp);
                                                                            }
                                                                            Err(err) => {
                                                                                #[cfg(debug_assertions)]
                                                                                println!("{:?}", err);
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

                                    if ui.button(self.desired_action.as_text()).clicked() {
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
                ChangeDBSettings => {}
                CreateDB => {}
                DBResponseError(_) => {}
            }
        }

        // stats panel block
        {
            let ps_lock = self.program_state.lock().unwrap();
            match *ps_lock {
                NoClient => {}
                PromptForClientDetails => {}
                ClientConnectionError(_) => {}
                DBResponseError(_) => {}
                PromptForKey => {}
                ChangeDBSettings => {}
                CreateDB => {}
                DisplayClient => match &self.database_list {
                    None => {}
                    Some(list) => {
                        if let Some(index) = self.selected_database {
                            if let Some(db) = list.get(index) {
                                match &db.statistics {
                                    NotCached => {}
                                    Cached(stats) => {
                                        egui::SidePanel::right("stats_panel").show(ctx, |ui| {
                                            ui.label(format!(
                                                "Total request count: {}",
                                                stats.get_total_req()
                                            ));
                                            ui.label(format!(
                                                "Average access time gap: {:.2}",
                                                stats.get_avg_time()
                                            ));
                                            let times_string = stats.get_usage_time_list()
                                                .iter()
                                                .map(|date| display_date(date))
                                                .reduce(|a,b| {format!("{},{}",a,b)});
                                            ui.label(format!("Previous access times: {}", times_string.unwrap_or_default()));
                                        });
                                    }
                                    ContentCacheState::Error(_) => {}
                                }
                            }
                        }
                    }
                },
            }
        }

        // side panel block
        {
            egui::SidePanel::left("side_panel").show(ctx, |ui| {
                let mut ps_lock = self.program_state.lock().unwrap();
                match *ps_lock {
                    NoClient => {}
                    PromptForClientDetails => {}
                    ClientConnectionError(_) => {}
                    // side menu that is persistent when displaying the client data.
                    DisplayClient | ChangeDBSettings => {
                        if let Some(selected_db) = self.selected_database {
                            if let Some(db_list) = &self.database_list {
                                if let Some(db) = db_list.get(selected_db) {
                                    ui.label(format!("Selected DB: {:?}", db.name));
                                    ui.separator();
                                    match &db.role {
                                        NotCached => {}
                                        Cached(role) => {
                                            ui.label(format!("Role: {:?}", role));
                                            ui.separator();
                                        }
                                        ContentCacheState::Error(_) => {}
                                    }
                                }
                            }
                        }
                        match &mut self.database_list {
                            None => {}
                            Some(list) => {
                                for (index, item) in list.iter_mut().enumerate() {
                                    if ui.button(format!("{}: {}", index + 1, item.name)).clicked()
                                    {
                                        let mut lock = self.client.lock().unwrap();
                                        match *lock {
                                            None => {}
                                            Some(ref mut client) => {
                                                // cache the content if it is not cached.
                                                match &item.content {
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

                                                match &item.db_settings {
                                                    NotCached => {
                                                        match client
                                                            .get_db_settings(item.name.as_str())
                                                        {
                                                            Ok(db_settings) => {
                                                                item.db_settings =
                                                                    Cached(db_settings.clone());

                                                                let users_string = {
                                                                    let mut s = String::new();
                                                                    db_settings
                                                                        .users
                                                                        .iter()
                                                                        .for_each(|user| {
                                                                            s.push_str(
                                                                                format!(
                                                                                    "{},",
                                                                                    user
                                                                                )
                                                                                .as_str(),
                                                                            );
                                                                        });
                                                                    if s.ends_with(',') {
                                                                        s.remove(s.len() - 1);
                                                                    }
                                                                    s
                                                                };
                                                                let admins_string = {
                                                                    let mut s = String::new();
                                                                    db_settings
                                                                        .admins
                                                                        .iter()
                                                                        .for_each(|admin| {
                                                                            s.push_str(
                                                                                format!(
                                                                                    "{},",
                                                                                    admin
                                                                                )
                                                                                .as_str(),
                                                                            );
                                                                        });
                                                                    if s.ends_with(',') {
                                                                        s.remove(s.len() - 1);
                                                                    }
                                                                    s
                                                                };

                                                                self.users_list = users_string;
                                                                self.admins_list = admins_string;
                                                                self.duration_seconds = db_settings
                                                                    .invalidation_time
                                                                    .as_secs();
                                                                self.submit_db_settings =
                                                                    db_settings;
                                                            }
                                                            Err(err) => {
                                                                item.db_settings =
                                                                    ContentCacheState::Error(err);
                                                            }
                                                        }
                                                    }
                                                    Cached(settings) => {
                                                        self.submit_db_settings = settings.clone();
                                                        let users_string = {
                                                            let mut s = String::new();
                                                            settings.users.iter().for_each(
                                                                |user| {
                                                                    s.push_str(
                                                                        format!("{},", user)
                                                                            .as_str(),
                                                                    );
                                                                },
                                                            );
                                                            if s.ends_with(',') {
                                                                s.remove(s.len() - 1);
                                                            }
                                                            s
                                                        };
                                                        let admins_string = {
                                                            let mut s = String::new();
                                                            settings.admins.iter().for_each(
                                                                |admin| {
                                                                    s.push_str(
                                                                        format!("{},", admin)
                                                                            .as_str(),
                                                                    );
                                                                },
                                                            );
                                                            if s.ends_with(',') {
                                                                s.remove(s.len() - 1);
                                                            }
                                                            s
                                                        };

                                                        self.users_list = users_string;
                                                        self.admins_list = admins_string;
                                                        self.duration_seconds =
                                                            settings.invalidation_time.as_secs();
                                                    }
                                                    ContentCacheState::Error(_) => {}
                                                }

                                                match &item.statistics {
                                                    NotCached => {
                                                        match client.get_stats(item.name.as_str()) {
                                                            Ok(stats) => {
                                                                item.statistics = Cached(stats);
                                                            }
                                                            Err(err) => {
                                                                item.statistics =
                                                                    ContentCacheState::Error(err);
                                                            }
                                                        }
                                                    }
                                                    Cached(_) => {}
                                                    ContentCacheState::Error(_) => {}
                                                }

                                                // set the selected database number in the program state.
                                                self.selected_database = Some(index);
                                            }
                                        }
                                    }
                                }

                                if let Some(index) = self.selected_database {
                                    if let Some(db) = list.get(index) {
                                        ui.separator();
                                        if ui
                                            .button("Delete DB")
                                            .on_hover_text("Double click to delete DB")
                                            .double_clicked()
                                        {
                                            let mut lock = self.client.lock().unwrap();
                                            match *lock {
                                                None => {}
                                                Some(ref mut client) => {
                                                    match client.delete_db(db.name.as_str()) {
                                                        Ok(delete_response) => match delete_response
                                                        {
                                                            DBSuccessResponse::SuccessNoData => {
                                                                list.remove(index);
                                                            }
                                                            DBSuccessResponse::SuccessReply(_) => {
                                                                *ps_lock = ClientConnectionError(
                                                                    BadPacket,
                                                                );
                                                            }
                                                        },
                                                        Err(err) => {
                                                            *ps_lock = ClientConnectionError(err);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        ui.separator();
                                    }
                                }
                            }
                        }
                    }
                    PromptForKey => {}
                    CreateDB => {}
                    DBResponseError(_) => {}
                }
            });
        }

        // center panel block
        {
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
                        // display a spinner if there is an active client connection thread going.
                        if let Some(thread) = &self.connection_thread {
                            if !thread.is_finished() {
                                ui.horizontal(|ui| {
                                    ui.label("Connecting...");
                                    ui.spinner();
                                });
                            }
                        }
                    }
                    PromptForClientDetails => {
                        // When the user clicks connect, we prompt them for client connection details.
                        ui.label("Enter Ip Address:");
                        ui.text_edit_singleline(&mut self.ip_address);

                        if !self.ip_address.is_empty() {
                            ui.checkbox(&mut self.auto_connect,"Auto connect to given ip address on startup");
                            if self.auto_connect {
                                // if the users client key is not empty display the possibility to auto set their key.
                                ui.checkbox(&mut self.auto_set_key, "Auto set key on connect on startup").on_hover_text("Key must not be empty to run at startup.");
                            } else {
                                // if auto connect is false, then auto set key should also be false.
                                self.auto_set_key = false;
                            }
                        }

                        if ui.button("Connect to ip address").clicked() {
                            // clone a bunch of things that need to be moved into the thread.
                            let client_clone = Arc::clone(&self.client);
                            let program_state_clone = Arc::clone(&self.program_state);
                            let ip_clone = self.ip_address.clone();
                            self.connection_thread = Some(thread::spawn(move || {
                                // instantly move all the variables into the thread
                                let ps = program_state_clone;
                                let client_mutex = client_clone;
                                let ip = ip_clone;

                                match SmolDbClient::new(&ip) {
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

                        if let Some(thread) = &self.connection_thread {
                            if !thread.is_finished() {
                                ui.spinner();
                            }
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
                                                        db_settings: NotCached,
                                                        statistics: NotCached,
                                                    })
                                                    .collect(),
                                            );
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
                                    if let Some(db_cached) = list.get(index_selected) {
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
                        ui.label("Client error:");
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
                    ChangeDBSettings => {
                        match self.selected_database {
                            None => {}
                            Some(index) => {
                                match &mut self.database_list {
                                    None => {}
                                    Some(list) => {
                                        match list.get_mut(index) {
                                            None => {}
                                            Some(db) => {
                                                match &mut db.db_settings {
                                                    NotCached => {}
                                                    Cached(db_settings) => {
                                                        // invalidation time
                                                        ui.label(format!("Invalidation time: {}s", db_settings.get_invalidation_time().as_secs()));

                                                        // other permissions
                                                        let others_perms = db_settings.get_other_rwx();
                                                        ui.label(format!("Others permissions (rwx): {},{},{}", others_perms.0,others_perms.1,others_perms.2));

                                                        // user permissions
                                                        let users_perms = db_settings.get_user_rwx();
                                                        ui.label(format!("Users permissions (rwx): {},{},{}", users_perms.0,users_perms.1,users_perms.2));

                                                        // user list
                                                        ui.label(format!("User list: {:?}", db_settings.get_user_list()));

                                                        // admin list
                                                        ui.label(format!("Admin list: {:?}", db_settings.get_admin_list()));

                                                        ui.separator();

                                                        ui.horizontal(|ui| {
                                                            ui.label("Invalidation time:").on_hover_text("Duration in seconds to cache the database before removing it from cache.");
                                                            ui.add(egui::DragValue::new(&mut self.duration_seconds));
                                                        });

                                                        self.submit_db_settings.invalidation_time = Duration::from_secs(self.duration_seconds);

                                                        ui.horizontal(|ui| {
                                                            ui.label("Others permissions: ").on_hover_text("Read, Write, List Contents");
                                                            ui.checkbox(&mut self.submit_db_settings.can_others_rwx.0,"r");
                                                            ui.checkbox(&mut self.submit_db_settings.can_others_rwx.1,"w");
                                                            ui.checkbox(&mut self.submit_db_settings.can_others_rwx.2,"x");
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Users permissions: ").on_hover_text("Read, Write, List Contents");
                                                            ui.checkbox(&mut self.submit_db_settings.can_users_rwx.0,"r");
                                                            ui.checkbox(&mut self.submit_db_settings.can_users_rwx.1,"w");
                                                            ui.checkbox(&mut self.submit_db_settings.can_users_rwx.2,"x");
                                                        });

                                                        ui.horizontal(|ui| {
                                                            ui.label("Users: ").on_hover_text("Comma separated :)");
                                                            ui.text_edit_singleline(&mut self.users_list);
                                                        });

                                                        ui.horizontal(|ui| {
                                                            ui.label("Admins: ").on_hover_text("Comma separated :)");
                                                            ui.text_edit_singleline(&mut self.admins_list);
                                                        });

                                                        if !self.users_list.is_empty() {
                                                            self.submit_db_settings.users = self.users_list.split(',').map(|string| string.to_string()).collect::<Vec<String>>();
                                                        } else {
                                                            self.submit_db_settings.users = vec![];
                                                        }

                                                        if !self.admins_list.is_empty() {
                                                            self.submit_db_settings.admins = self.admins_list.split(',').map(|string| string.to_string()).collect::<Vec<String>>();
                                                        } else {
                                                            self.submit_db_settings.admins = vec![];
                                                        }

                                                        #[cfg(debug_assertions)]
                                                        ui.label(format!("DEBUG users: {:?}", self.submit_db_settings.users));
                                                        #[cfg(debug_assertions)]
                                                        ui.label(format!("DEBUG admins: {:?}", self.submit_db_settings.admins));

                                                        if ui.button("Submit").clicked() {
                                                            let mut lock = self.client.lock().unwrap();
                                                            match *lock {
                                                                None => {}
                                                                Some(ref mut client) => {
                                                                    match client.set_db_settings(db.name.as_str(),self.submit_db_settings.clone()) {
                                                                        Ok(_) => {
                                                                            *db_settings = self.submit_db_settings.clone();
                                                                        }
                                                                        Err(err) => {
                                                                            db.db_settings = ContentCacheState::Error(err);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    ContentCacheState::Error(err) => {
                                                        ui.label(format!("Error reading DBSettings: {:?}", err));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if ui.button("Back").clicked() {
                            *ps_lock = DisplayClient;
                        }

                        ui.separator();

                    }
                    CreateDB => {

                        ui.horizontal(|ui| {
                            ui.label("DB name:");
                            ui.add_sized([160.0,20.0],egui::TextEdit::singleline(&mut self.db_name_create));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Invalidation time:").on_hover_text("Duration in seconds to cache the database before removing it from cache.");
                            ui.add(egui::DragValue::new(&mut self.duration_seconds));
                        });

                        self.submit_db_settings.invalidation_time = Duration::from_secs(self.duration_seconds);

                        ui.horizontal(|ui| {
                            ui.label("Others permissions: ").on_hover_text("Read, Write, List Contents");
                            ui.checkbox(&mut self.submit_db_settings.can_others_rwx.0,"r");
                            ui.checkbox(&mut self.submit_db_settings.can_others_rwx.1,"w");
                            ui.checkbox(&mut self.submit_db_settings.can_others_rwx.2,"x");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Users permissions: ").on_hover_text("Read, Write, List Contents");
                            ui.checkbox(&mut self.submit_db_settings.can_users_rwx.0,"r");
                            ui.checkbox(&mut self.submit_db_settings.can_users_rwx.1,"w");
                            ui.checkbox(&mut self.submit_db_settings.can_users_rwx.2,"x");
                        });

                        ui.horizontal(|ui| {
                            ui.label("Users: ").on_hover_text("Comma separated :)");
                            ui.text_edit_singleline(&mut self.users_list);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Admins: ").on_hover_text("Comma separated :)");
                            ui.text_edit_singleline(&mut self.admins_list);
                        });

                        if !self.users_list.is_empty() {
                            self.submit_db_settings.users = self.users_list.split(',').map(|string| string.to_string()).collect::<Vec<String>>();
                        } else {
                            self.submit_db_settings.users = vec![];
                        }

                        if !self.admins_list.is_empty() {
                            self.submit_db_settings.admins = self.admins_list.split(',').map(|string| string.to_string()).collect::<Vec<String>>();
                        } else {
                            self.submit_db_settings.admins = vec![];
                        }

                        #[cfg(debug_assertions)]
                        ui.label(format!("DEBUG users: {:?}", self.submit_db_settings.users));
                        #[cfg(debug_assertions)]
                        ui.label(format!("DEBUG admins: {:?}", self.submit_db_settings.admins));

                        if ui.button("Submit").clicked() && !self.db_name_create.is_empty() {
                            let mut lock = self.client.lock().unwrap();
                            match *lock {
                                None => {}
                                Some(ref mut client) => {
                                    match client.create_db(self.db_name_create.as_str(),self.submit_db_settings.clone()) {
                                        Ok(resp) => {
                                            match resp {
                                                DBSuccessResponse::SuccessNoData => {
                                                    // after creating a db go back to displaying the client
                                                    *ps_lock = DisplayClient;

                                                    match &mut self.database_list {
                                                        None => {}
                                                        Some(list) => {
                                                            match client.list_db_contents(self.db_name_create.as_str()) {
                                                                Ok(response) => {
                                                                    list.push(DBCached{
                                                                        name: self.db_name_create.to_string(),
                                                                        content: Cached(response),
                                                                        role: NotCached,
                                                                        db_settings: NotCached,
                                                                        statistics: NotCached,
                                                                    });
                                                                }
                                                                Err(err) => {
                                                                    *ps_lock = ClientConnectionError(err);
                                                                }
                                                            }
                                                        }
                                                    }



                                                }
                                                DBSuccessResponse::SuccessReply(_) => {
                                                    // this should not happen, creating a db does not respond with data.
                                                    *ps_lock = ClientConnectionError(BadPacket);
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            *ps_lock = ClientConnectionError(err);
                                        }
                                    }
                                }
                            }
                        }
                        if ui.button("Back").clicked() {
                            *ps_lock = DisplayClient;
                        }
                    }
                    DBResponseError(err) => {
                        ui.label(format!("{:?}", err));
                    }
                }

                egui::warn_if_debug_build(ui);
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn display_date(time: &DateTime<Local>) -> String {
    format!("{}/{}/{} {}:{} {}", time.month(),time.day(),time.year(),time.hour12().1,time.minute(),{ if time.hour12().0 {"PM"} else {"AM"}})
}