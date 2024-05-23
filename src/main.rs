mod task;
mod collection;
mod utils;

use std::fs::File;

use collection::{Collection, CollectionMessage};
use cosmic::iced::Length;
use cosmic::iced_widget::row;
use cosmic::widget::{nav_bar, segmented_button};
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
}

#[derive(Debug, Clone)]
pub enum Message {
    CollectionMessage(usize, CollectionMessage),
    NavBarItemSelected(segmented_button::Entity),
    NewCollection,
    NewCollectionRequest,
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
        core: Core,
        _: Self::Flags,
    ) -> (
        Self,
        cosmic::iced::Command<cosmic::app::Message<Self::Message>>,
    ) {
        let path = data_path();
        let data: AppData = match File::open(path) {
            Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
            Err(_) => Default::default(),
        };

        let mut app = App {
            core,
            data,
            nav_model: segmented_button::ModelBuilder::default().build(),
        };

        if app.nav_model.iter().next().is_none() {
            app.update_nav_model();
        }

        let command = app.update_title();

        (app, command)
    }

    fn view(&self) -> Element<Message> {
        if let Some(col) = self.get_current_collection() {
            col.view()
                .map(|collection_msg| Message::CollectionMessage(self.data.current_index, collection_msg))
        } else {
            row!().into()
        }
    }

    fn nav_bar(&self) -> Option<Element<cosmic::app::Message<Self::Message>>> {
        let mut nav = nav_bar(&self.nav_model, |entity| {
            cosmic::app::Message::Cosmic(cosmic::app::cosmic::Message::NavBar(entity))
        }).close_icon(
            cosmic::widget::icon::from_name("media-eject-symbolic")
                .size(16)
                .icon(),
        )
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
        };
        Command::none()
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> cosmic::iced::Command<cosmic::app::Message<Self::Message>> {
        match message {
            Message::CollectionMessage(i, msg) => {
                let _ = self.data.collections[i].update(msg);
            },
            Message::NavBarItemSelected(_) => println!("hello is this working"),
            _ => (),
        }
        Command::none()
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
    fn update_title(&mut self) -> Command<Message> {
        let window_title = "Hello, World!".to_string();
        self.set_header_title(window_title.clone());
        self.set_window_title(window_title, window::Id::MAIN)
    }

    fn get_current_collection(&self) -> Option<&Collection> {
        self.data.collections.get(self.data.current_index)
    }

    fn update_nav_model(&mut self) {
        let mut model = segmented_button::ModelBuilder::default();

        model = model.insert(|b| {
            b.text("Add new Collection")
                .icon(cosmic::widget::icon::from_name("list-add-symbolic"))
        });

        for (i, collection) in self.data.collections.iter().enumerate() {
            model = model.insert(|b| {
                b.text(collection.name.clone())
                    .data(CollectionIndex(i))
            });
        }

        self.nav_model = model.build();
    }
}
