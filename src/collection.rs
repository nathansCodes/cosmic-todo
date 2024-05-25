use crate::task::{Task, TaskMessage};

use cosmic::iced::{Alignment, Length};
use cosmic::widget;
use cosmic::prelude::*;
use cosmic::widget::{button, text_input};
use cosmic::app::Command;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub tasks: Vec<Task>,
    filter: Filter,
    #[serde(skip)]
    add_task_input: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

impl Filter {
    pub fn matches(&self, task: &Task) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !task.completed,
            Filter::Completed => task.completed,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CollectionMessage {
    TaskMessage(usize, TaskMessage),
    AddTask,
    AddTaskInputChanged(String),
    FilterChanged(Filter),
    RemoveTasks,
}

impl Collection {
    pub fn view(&self) -> Element<CollectionMessage> {
        let add_button = button(widget::icon::Named::new("list-add-symbolic"))
            .on_press(CollectionMessage::AddTask);

        let inputbox = text_input("Start a new task...", &self.add_task_input)
            .on_submit(CollectionMessage::AddTask)
            .on_input(CollectionMessage::AddTaskInputChanged)
            .trailing_icon(add_button.into());

        let filter_button = |label, filter| {
            button(label).style(if filter == self.filter {
                button::Style::Standard
            } else {
                button::Style::Text
            }).on_press(CollectionMessage::FilterChanged(filter)).padding(8)
        };

        let filters = widget::row()
            .push(filter_button("All", Filter::All))
            .push(filter_button("Active", Filter::Active))
            .push(filter_button("Completed", Filter::Completed))
            .spacing(10)
            .width(Length::Fill)
            .align_items(Alignment::End);

        let remove_button = button(match self.filter {
            Filter::All => "Remove All Tasks",
            Filter::Active => "Remove Active Tasks",
            Filter::Completed => "Remove Completed Tasks",
        })
        .on_press(CollectionMessage::RemoveTasks);

        let controls = widget::row()
            .push(filters)
            .push(remove_button);

        let tasks = widget::column()
            .extend(self.tasks
                .iter()
                .enumerate()
                .filter(|(_, task)| self.filter.matches(task))
                .map(|(i, task)| {
                    task.view()
                        .map(move |message| CollectionMessage::TaskMessage(i, message))
                }))
            .align_items(Alignment::Start)
            .spacing(20);

        let content = widget::column()
            .push(inputbox)
            .push(controls)
            .push(tasks)
            .width(500)
            .align_items(Alignment::Center)
            .spacing(25);

        widget::container(content).width(Length::Fill).center_x().into()
    }

    pub fn update(
        &mut self,
        message: CollectionMessage,
    ) -> cosmic::app::Command<CollectionMessage> {
        match message {
            CollectionMessage::TaskMessage(i, TaskMessage::Delete) => {
                self.tasks.remove(i);
            }
            CollectionMessage::TaskMessage(i, task_message) => {
                if let Some(task) = self.tasks.get_mut(i) {
                    let _ = task.update(task_message);
                }
            }
            CollectionMessage::AddTaskInputChanged(input) => self.add_task_input = input,
            CollectionMessage::AddTask => {
                let input = self.add_task_input.clone();

                if input.trim().is_empty() {
                    return Command::none();
                }

                self.tasks.push(Task {
                    name: input,
                    completed: false,
                });
                self.add_task_input = "".to_string();
            }
            CollectionMessage::FilterChanged(filter) => {
                self.filter = filter;
            },
            CollectionMessage::RemoveTasks => {
                self.remove_tasks(self.filter.clone());
            }
        }

        Command::none()
    }

    fn remove_tasks(&mut self, filter: Filter) {
        let mut i = 0;
        while i < self.tasks.len() {
            if filter.matches(&self.tasks[i]) {
                self.tasks.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
