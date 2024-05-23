use cosmic::{
    iced::Length,
    iced_widget::row,
    widget::{button, checkbox, icon},
    Command, Element,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TaskMessage {
    Completed(bool),
    Delete,
}

impl Task {
    pub fn view(&self) -> Element<TaskMessage> {
        row![
            checkbox(&self.name, self.completed, TaskMessage::Completed).width(Length::Fill),
            button::destructive("Delete")
                .leading_icon(icon::Named::new("edit-delete-symbolic"))
                .on_press(TaskMessage::Delete)
        ]
        .spacing(20)
        .into()
    }

    pub fn update(
        &mut self,
        msg: TaskMessage,
    ) -> cosmic::iced::Command<cosmic::app::Message<TaskMessage>> {
        match msg {
            TaskMessage::Completed(val) => self.completed = val,
            TaskMessage::Delete => (),
        }
        Command::none()
    }
}
