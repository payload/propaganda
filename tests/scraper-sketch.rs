use anyhow::*;
use evmap_derive::ShallowCopy;
use itertools::Itertools;
use itertools;
use prettydiff;
use async_std::{
    prelude::*,
    task,
};
use std::time::Duration;
use futures::prelude::*;

#[async_std::test]
async fn fun() -> Result<()> {
    let (db_r, mut db_w) = evmap::new::<String, String>();

    let producer = task::spawn(async move {
        for index in 0..10 {
            task::sleep(Duration::from_secs(1)).await;
            db_w.insert(format!("test"), format!("{}", index)).refresh();
        }
    });

    let consumer = task::spawn(async move {
        let mut running = true;
        while running {
            if let Some(v) = &db_r.get("test") {
                for value in v.iter() {
                    println!("{}", value);
                    if value == "3" {
                        running = false;
                    }
                }
            }
            task::sleep(Duration::from_secs(1)).await;
        }
    });

    future::join(consumer, producer).await;

    println!("\nThank you!");
    Ok(())
}