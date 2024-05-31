mod collection;
mod task;
mod utils;

use std::fs::File;

use collection::{Collection, CollectionMessage};
use cosmic::iced::Length;
use cosmic::widget::{self, nav_bar, segmented_button};
use utils::data_path;

use cosmic::prelude::*;
use cosmic::{
    app::{Command, Core, Settings},
    executor,
    iced::window,
    ApplicationExt,
};
use serde::{Deserialize, Serialize};

const APP_ID: &str = "com.github.nathansCodes.cosmic_test";

#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .debug(false)
        .size([1280, 768].into());

    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}

#[derive(Default, Serialize, Deserialize)]
pub struct AppData {
    collections: Vec<Collection>,
    current_index: usize,
}

pub struct App {
    core: Core,
    data: AppData,
    nav_model: segmented_button::SingleSelectModel,
    show_dialog: bool,
    dialog_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    CollectionMessage(usize, CollectionMessage),
    NewCollection,
    RemoveCollection(segmented_button::Entity),
    ShowDialog,
    HideDialog,
    DialogInputChanged(String),
    ToggleNavBar,
    None,
}

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn init(
        mut core: Core,
        _: Self::Flags,
    ) -> (
        Self,
        cosmic::app::Command<Self::Message>,
    ) {
        core.nav_bar_toggle_condensed();

        let path = data_path();
        let data: AppData = match File::open(path) {
            Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
            Err(_) => Default::default(),
        };

        let mut app = App {
            core,
            data,
            nav_model: segmented_button::ModelBuilder::default().build(),
            show_dialog: false,
            dialog_input: "".to_string(),
        };

        let mut commands = vec![app.update_title()];

        if app.nav_model.iter().next().is_none() {
            commands.push(app.update_nav_model());
        }

        (app, Command::batch(commands))
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Command<Self::Message> {
        match message {
            Message::CollectionMessage(i, msg) => {
                let _ = self.data.collections[i].update(msg);
            }
            Message::RemoveCollection(entity) => {
                if let Some(data) = self.nav_model.data::<CollectionIndex>(entity) {
                    self.data.collections.remove(data.0);
                    return self.update_nav_model();
                }
            }
            Message::ShowDialog => self.show_dialog = true,
            Message::HideDialog => self.show_dialog = false,
            Message::DialogInputChanged(input) => self.dialog_input = input,
            Message::NewCollection => {
                self.data
                    .collections
                    .push(Collection::new(&self.dialog_input.clone()));
                self.dialog_input.clear();
                self.show_dialog = false;
                return self.update_nav_model();
            }
            Message::ToggleNavBar => {
                self.core.nav_bar_toggle();
                self.core.nav_bar_toggle_condensed();
            }
            Message::None => (),
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        if let Some(col) = self.get_current_collection() {
            col.view().map(|collection_msg| {
                Message::CollectionMessage(self.data.current_index, collection_msg)
            })
        } else {
            widget::row().into()
        }
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![widget::nav_bar_toggle()
            .on_toggle(Message::ToggleNavBar)
            .into()]
    }

    fn nav_bar(&self) -> Option<Element<cosmic::app::Message<Self::Message>>> {
        if !self.core.nav_bar_active() {
            return None;
        }

        let mut nav = nav_bar(&self.nav_model, |entity| {
            cosmic::app::Message::Cosmic(cosmic::app::cosmic::Message::NavBar(entity))
        })
        .close_icon(
            cosmic::widget::icon::from_name("edit-delete-symbolic")
                .size(16)
                .icon(),
        )
        .on_close(|entity| cosmic::app::Message::App(Message::RemoveCollection(entity)))
        .into_container();

        if !self.core().is_condensed() {
            nav = nav.max_width(280);
        }

        Some(Element::from(
            // XXX both must be shrink to avoid flex layout from ignoring it
            nav.width(Length::Shrink).height(Length::Shrink),
        ))
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<Self::Message> {
        if let Some(data) = self.nav_model.data::<CollectionIndex>(id) {
            self.data.current_index = data.0;
        } else {
            self.show_dialog = true;
        };

        self.update_all()
    }

    fn dialog(&self) -> Option<Element<Self::Message>> {
        if !self.show_dialog {
            return None;
        }

        let create_message = if self.dialog_input.is_empty() {
            None
        } else {
            Some(Message::NewCollection)
        };

        let create = widget::button::suggested("Create").on_press_maybe(create_message);

        let input = widget::text_input("Name", &self.dialog_input)
            .on_submit(Message::NewCollection)
            .on_input(Message::DialogInputChanged);

        let cancel = widget::button::standard("Cancel").on_press(Message::HideDialog);

        let dialog = widget::dialog("Create a new Collection")
            .primary_action(create)
            .secondary_action(input)
            .tertiary_action(cancel);

        Some(dialog.into())
    }

    fn on_app_exit(&mut self) -> Option<Self::Message> {
        let file = File::create(utils::data_path()).expect("Could not create json file");

        serde_json::to_writer(file, &self.data).expect("Could not write json data");

        None
    }
}

struct CollectionIndex(usize);

impl App
where
    Self: cosmic::Application,
{
    fn update_all(&mut self) -> Command<Message> {
        Command::batch(vec![self.update_nav_model(), self.update_title()])
    }

    fn update_title(&mut self) -> Command<Message> {
        let collection_name = if let Some(collection) = self.get_current_collection() {
            collection.name.clone()
        } else {
            "Home".to_string()
        };

        let window_title = format!("Todo - {}", collection_name);
        self.set_header_title(window_title.clone());
        self.set_window_title(window_title, window::Id::MAIN)
    }

    fn update_nav_model(&mut self) -> Command<Message> {
        let mut model = segmented_button::ModelBuilder::default();

        model = model.insert(|b| {
            b.text("Add new Collection")
                .activate()
                .icon(cosmic::widget::icon::from_name("list-add-symbolic"))
        });

        for (i, collection) in self.data.collections.iter().enumerate() {
            model = model.insert(|b| {
                b.text(collection.name.clone())
                    .data(CollectionIndex(i))
                    .closable()
            });
        }

        self.nav_model = model.build();

        Command::none()
    }

    fn get_current_collection(&self) -> Option<&Collection> {
        self.data.collections.get(self.data.current_index)
    }
}
