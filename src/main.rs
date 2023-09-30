use chrono::{SubsecRound, Utc};
use cron::Schedule;
use notify_rust::Notification as DesktopNotification;
use serde_derive::Deserialize;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{fs, str::FromStr};

#[derive(Deserialize)]
struct RawConfig {
    notifications: Vec<RawNotification>,
}

#[derive(Deserialize, Debug)]
struct RawNotification {
    name: String,
    body: String,
    icon: Option<String>,
    cron: String,
}

#[derive(Debug)]
struct ParsedNotification {
    name: String,
    body: String,
    icon: Option<String>,
    cron: Schedule,
}

#[derive(Debug)]
struct Config {
    notifications: Vec<ParsedNotification>,
}

fn main() {
    let config = load_config();
    let flags: Vec<String> = std::env::args().collect();

    if flags.iter().any(|flag| flag == "--debug") {
        println!("Config: {config:#?}");

        for notification in &config.notifications {
            println!(
                "Next time for {:?}: {:?}",
                notification.name,
                notification
                    .cron
                    .after(&(Utc::now() - Duration::from_secs(1)))
                    .next()
                    .unwrap()
            );
        }
    };

    // Doesn't handle sub second cron jobs, but that's whatever for my use case.
    let interval = Duration::from_secs(1);
    let mut next_time = Instant::now() + interval;

    loop {
        // Check if any notifications should be sent
        for notification in &config.notifications {
            let notification_next_time = notification
                .cron
                .after(&(Utc::now() - Duration::from_secs(1)))
                .next()
                .unwrap();

            if notification_next_time == Utc::now().round_subsecs(0) {
                DesktopNotification::new()
                    .summary(&notification.name)
                    .body(&notification.body)
                    .icon(notification.icon.as_ref().unwrap_or(&String::new()))
                    .show()
                    .unwrap();
            }
        }

        sleep(next_time - Instant::now());
        next_time += interval;
    }
}

fn load_config() -> Config {
    let config_content =
        fs::read_to_string("./config.toml").expect("Should have been able to read the file");

    let raw_config: RawConfig = toml::from_str(&config_content).unwrap();

    Config {
        notifications: raw_config
            .notifications
            .into_iter()
            .map(|raw_notification| ParsedNotification {
                name: raw_notification.name,
                body: raw_notification.body,
                icon: raw_notification.icon,
                cron: Schedule::from_str(&raw_notification.cron).unwrap(),
            })
            .collect(),
    }
}
