use super::{file, note, notes};
use chrono::NaiveDate;
use lazy_static::lazy_static;
use mut_static::MutStatic;
use note::Note;
use notes::Notes;
use once_cell::unsync::Lazy;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use std::{error::Error, str::FromStr};
use teloxide::{
    prelude::*,
    utils::command::{BotCommands, ParseError},
};

pub struct NotesBot {
    pub notes: notes::Notes,
    pub bot: AutoSend<Bot>,
    key: &str,
}

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(
        description = "create task with <name>.",
        parse_with = "one_line_parser"
    )]
    Create(String),
    #[command(
        description = "<id> <ToDo | Doing | Done> - set task with <id> to state.",
        parse_with = "split"
    )]
    SetState(u64, String),
    #[command(
        description = "<id> <yyyy-mm-dd> - set task with <id> <deadline>.",
        parse_with = "split"
    )]
    SetDead(u64, String),
    #[command(
        description = "<id> <text> - set task with <id> text to <text>",
        parse_with = "text_parser"
    )]
    Edit(u64, String),
    #[command(
        description = "<id> <name> - set task with <id> name to <name>.",
        parse_with = "text_parser"
    )]
    EditName(u64, String),
    #[command(description = "<id> - delete task with <id>", parse_with = "split")]
    Delete(u64),
    #[command(
        description = "<id> <ToDo | Doing | Done> - set note with <id> to state.",
        parse_with = "split"
    )]
    Show(u64),
    #[command(description = "- list ToDo and Doing tasks")]
    List,
    #[command(description = "- list all tasks")]
    ListAll,
    #[command(description = "- list all tasks for next week", parse_with = "split")]
    Agenda,
}

impl NotesBot {
    async fn answer(
        &mut self,
        bot: AutoSend<Bot>,
        message: Message,
        command: Command,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match command {
            Command::Help => {
                bot.send_message(message.chat.id, Command::descriptions().to_string())
                    .await?
            }
            Command::Create(name) => {
                let inst = Note::new()
                    .with_header(name)
                    .with_chat(message.chat.id.0 as u64);
                if inst.id() % 5 == 0 {
                    file::save_to_file(&self.notes);
                }
                self.notes.add(inst);
                bot.send_message(message.chat.id, "Ok:)").await?
            }
            Command::SetState(id, state) => {
                //	    let mut notes = get_mut_ref_notes();
                if let Some(mut note) = self.notes.note_as_mut(id) {
                    note.set_state_from_string(state);
                    bot.send_message(message.chat.id, "State changed").await?
                } else {
                    bot.send_message(message.chat.id, "Unknown id").await?
                }
            }
            Command::SetDead(id, deadline) => {
                //	    let mut notes = get_mut_ref_notes();
                if let Err(_) = NaiveDate::from_str(&deadline) {
                    bot.send_message(message.chat.id, "Invalid date format")
                        .await?;
                }
                if let Some(mut note) = self.notes.note_as_mut(id) {
                    note.set_deadline(Some(deadline));
                    bot.send_message(message.chat.id, "State changed").await?
                } else {
                    bot.send_message(message.chat.id, "Unknown id").await?
                }
            }
            Command::Edit(id, text) => {
                //	    let mut notes = get_mut_ref_notes();
                if let Some(mut note) = self.notes.note_as_mut(id) {
                    note.set_text(text);
                    bot.send_message(message.chat.id, "State changed").await?
                } else {
                    bot.send_message(message.chat.id, "Unknown id").await?
                }
            }
            Command::EditName(id, name) => {
                //	    let mut notes = get_mut_ref_notes();
                if let Some(mut note) = self.notes.note_as_mut(id) {
                    note.set_header(name);
                    bot.send_message(message.chat.id, "State changed").await?
                } else {
                    bot.send_message(message.chat.id, "Unknown id").await?
                }
            }
            Command::Delete(id) => {
                //	    let mut notes = get_mut_ref_notes();
                self.notes.delete(id);
                bot.send_message(message.chat.id, "Ok:)").await?
            }
            Command::Show(id) => {
                //            let mut notes = get_mut_ref_notes();
                if let Some(note) = self.notes.note_by_id(id) {
                    bot.send_message(message.chat.id, note.to_message()).await?
                } else {
                    bot.send_message(message.chat.id, "Unknown id").await?
                }
            }
            Command::List => {
                for i in /*get_ref_notes()*/ self.notes.notes_by_chat(message.chat.id.0 as u64) {
                    bot.send_message(message.chat.id, i.to_message()).await?;
                }
                bot.send_message(message.chat.id, "Ok:)").await?
            }
            Command::ListAll => {
                for i in /*get_ref_notes()*/
                    self.notes.notes_by_chat_all(message.chat.id.0 as u64)
                {
                    bot.send_message(message.chat.id, i.to_message()).await?;
                }
                bot.send_message(message.chat.id, "Ok:)").await?
            }
            Command::Agenda => {
                for i in /*get_ref_notes()*/ self.notes.notes_agenda(message.chat.id.0 as u64) {
                    bot.send_message(message.chat.id, i.to_message()).await?;
                }
                bot.send_message(message.chat.id, "Ok:)").await?
            }
        };

        Ok(())
    }
}

fn text_parser(input: String) -> Result<(u64, String), ParseError> {
    let tmp: Vec<&str> = input.split(" ").collect();
    if let Ok(id) = tmp.get(0).unwrap().parse::<u64>() {
        let text = &tmp[1..].join(" ").to_string();
        Ok((id, text.clone()))
    } else {
        Err(ParseError::Custom(
            "First argument should be a number!".into(),
        ))
    }
}

fn one_line_parser(input: String) -> Result<(String,), ParseError> {
    Ok((input,))
}