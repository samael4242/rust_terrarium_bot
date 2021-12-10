use tokio_stream::wrappers::UnboundedReceiverStream;
use teloxide::*;

use teloxide::{
    prelude::*,
    utils::command::{BotCommand, ParseError},
};

type BotResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum TestBotCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username {name: String},
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge {name: String, age: u16 },
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();

    let bot = Bot::new("5062129832:AAFrKYjpuB6jBPZKPTHShmzNoeJO0MAZZqA".to_string()).auto_send();
    
    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, 
                                                                 |cx| async move {
                message_handler(cx).await.log_on_error().await;
            })
        })
        .dispatch()
        .await;

    
}

async fn message_handler(cx: UpdateWithCx<AutoSend<Bot>, Message>) -> BotResult<()> {
    let text = match cx.update.kind {
        teloxide::types::MessageKind::Common(ref msg) => match &msg.media_kind {
            teloxide::types::MediaKind::Animation(inner) => inner.caption.as_ref(),
            teloxide::types::MediaKind::Audio(inner) => inner.caption.as_ref(),
            teloxide::types::MediaKind::Document(inner) => inner.caption.as_ref(),
            teloxide::types::MediaKind::Photo(inner) => inner.caption.as_ref(),
            teloxide::types::MediaKind::Text(inner) => Some(&inner.text),
            teloxide::types::MediaKind::Video(inner) => inner.caption.as_ref(),
            teloxide::types::MediaKind::Voice(inner) => inner.caption.as_ref(),
            _ => None,
        },
        _ => None,
    };

    if let Some(text) = text {
        if text.starts_with('/') {
            let ret = TestBotCommand::parse(text, "TETERustBot"); 
            let command = match ret  {
                Err(parse_err) => {
                    match parse_err {
                        ParseError::TooFewArguments {..} |
                        ParseError::TooManyArguments {..} => {
                            log::info!("Error in arg num");
                            cx.answer("Invalid parameters number, see commands description:\n".to_string() + 
                                      &TestBotCommand::descriptions())
                              .send()
                              .await?;
                            return Err(Box::new(parse_err));
                        }
                        _ => {
                            log::info!("other error");
                            cx.answer(TestBotCommand::descriptions())
                              .send()
                              .await?;
                            return Err(Box::new(parse_err));
                        }
                    }
                }
                Ok(tbc) => tbc 
            };

            match command {
                TestBotCommand::Help =>  {
                    cx.answer(&TestBotCommand::descriptions())
                      .send()
                      .await?;
                }
                TestBotCommand::Username { name } => {
                    cx.answer(format!("name is @{}", name))
                      .send()
                      .await?;
                }
                TestBotCommand::UsernameAndAge {name, age } => {
                    cx.answer(format!("name is {}, age {}", name, age))
                      .send()
                      .await?;
                }
            }

        } else {
            cx.answer("Could not parse command, see commands description:\n".to_string() +
                      &TestBotCommand::descriptions())
              .send()
              .await?;
        }
    }

    Ok(())
}
