pub mod scraper;
pub mod api;

use tokio::time::{sleep, Duration };
use scraper::{ generate_all_days, get_day_posts };
use api::{ HnClient };


#[tokio::main]
async fn main() {
    let hn = HnClient::init().unwrap();
    let max_item = hn.get_max_item_id().await.unwrap();
    println!("Max Item {max_item}");
    let mut posts: Vec<Post> = vec![];
    let mut accum = 
    for i in (0..max_item).rev() {
        if let Ok(item) = hn.get_item(i).await {
            println!("Item: {item:?}");
        }
        sleep(Duration::from_millis(1000)).await;
    }
    // let days = generate_all_days(2007, 10, 1);
    // for day in days {
    //     let res = get_day_posts(day.to_string()).await;
    //     println!("{day:?} {res:?}");
    //     sleep(Duration::from_millis(1000)).await;
    // }

}

