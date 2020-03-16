use serenity::client::Client;
use serenity::framework::standard::DispatchError::{NotEnoughArguments, TooManyArguments};
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};
use std::sync::Arc;

mod commands;

use commands::{heroku::*, math::*, myid::*, ping::*};

mod authorizations;

pub mod config;

use crate::config::Config;

use crate::authorizations::users::*;

#[group]
#[commands(ping, multiply, myid, get_app, get_apps, restart_app)]
struct General;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

// These commands do not require a user
// to be in the AUTHORIZED_USERS env variable
const NO_AUTH_COMMANDS: &[&str] = &["ping", "multiply", "myid"];

pub fn run(config: Config) {
    let mut client = Client::new(&config.discord_token, Handler).expect("Err creating client");

    // Insert default config into data
    // that is passed to each of the commands
    {
        let mut data = client.data.write();
        data.insert::<Config>(Arc::new(config.clone()));
    }

    client.with_framework(
        StandardFramework::new()
            .before(move |ctx, msg, cmd_name| {
                if !is_authorized(&msg.author.id.to_string(), config.clone()) {
                    if NO_AUTH_COMMANDS.contains(&cmd_name) {
                        return true;
                    }

                    println!("User is not authorized to run this command");
                    msg.reply(
                        ctx,
                        format!("User {} is not authorized to run this command", &msg.author),
                    )
                    .ok();

                    return false;
                }
                println!("Running command {}", cmd_name);
                true
            })
            .on_dispatch_error(|context, msg, error| match error {
                NotEnoughArguments { min, given } => {
                    let s = format!("Need {} arguments, but only got {}.", min, given);

                    let _ = msg.channel_id.say(&context.http, &s);
                }
                TooManyArguments { max, given } => {
                    let s = format!("Max arguments allowed is {}, but got {}.", max, given);

                    let _ = msg.channel_id.say(&context.http, &s);
                }
                _ => {
                    println!("Unhandled dispatch error {:?}", error);
                }
            })
            .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
            .group(&GENERAL_GROUP),
    );

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
