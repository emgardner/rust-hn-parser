use reqwest;
use reqwest::{ Result as ReqwestResult };
use scraper::{Html, Selector, ElementRef};
use chrono::{Datelike, NaiveDate, Duration, Local};
use std::fs::File;
use std::io::{BufWriter, Write};
use serde::{Serialize, Deserialize}; 
use tokio::time::{sleep, Duration as TokioDuration };

fn generate_all_days(year: i32, month: u32, day: u32) -> Vec<String> {
    let mut dates = Vec::new();
    let start_date = NaiveDate::from_ymd(year, month, day);
    let end_date = Local::now().naive_local().date();
    let mut current_date = start_date;
    while current_date < end_date {
        dates.push(current_date.format("%Y-%m-%d").to_string());
        current_date += Duration::days(1);
    }
    dates
}

#[derive(Debug)]
pub enum RowType {
    Thing(Thing),
    Info(Info),
    Spacer,
    More
}

#[derive(Debug)]
pub struct Thing {
    id: String,
    rank: String,
    titleline: String,
    link: String
}

#[derive(Debug)]
pub struct Info {
    score: String,
    user: String,
    date: String,
    comments: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    id: String,
    rank: String,
    // rank: i32,
    titleline: String,
    link: String,
    // score: i32, 
    score: String, 
    user: String,
    date: String,
    // comments: i32
    comments: String
}

async fn get_page(params: &PageParams) -> ReqwestResult<String> {
    reqwest::get(format_request(params))
        .await?
        .text()
    .await
}

#[derive(Debug)]
pub struct PageParams {
    day: String,
    page: u32
}

fn format_request(params: &PageParams) -> String {
    format!("https://news.ycombinator.com/front?day={}&p={}", params.day, params.page)
}

fn parse_thing(row: &ElementRef<'_>) -> Option<Thing> {
    let link_selector = Selector::parse("td > span > a").unwrap();
    let rank_selector = Selector::parse("span.rank").unwrap();
    let id = row.value().attr("id")?;
    let link_ele = row.select(&link_selector).next()?;
    let rank_ele = row.select(&rank_selector).next()?;
    let link =  link_ele.value().attr("href")?;
    Some(Thing {
        id: id.to_string(),
        rank: rank_ele.inner_html(),
        titleline: link_ele.inner_html(),
        link: link.to_string()
    })
}

fn parse_row(subline: &ElementRef<'_>) -> Option<Info> {
    let score_selector = Selector::parse("span.score").unwrap();
    let user_selector = Selector::parse(".hnuser").unwrap();
    let date_selector = Selector::parse("span.age").unwrap();
    let comment_selector = Selector::parse("a").unwrap();
    let score = subline.select(&score_selector).next()?;
    let user = subline.select(&user_selector).next()?;
    let date = subline.select(&date_selector).next()?.value().attr("title")?;
    let comment = subline.select(&comment_selector)
        .filter(|e| e.inner_html().contains("comments"))
        .nth(0)?;
    return Some( Info {
        score: score.inner_html().to_string(),
        user: user.inner_html().to_string(),
        date: date.to_string(), 
        comments: comment.inner_html()
    });
}

fn process_row(row: &ElementRef) -> Option<RowType> {
    let class = row.value().attr("class").unwrap_or("") ;
    let subline_selector = Selector::parse("span.subline").unwrap();
    match class {
        "spacer" => return Some(RowType::Spacer),
        "athing" => {
            match parse_thing(row) {
                Some(thing) => Some(RowType::Thing(thing)),
                None => None
            }
        },
        _ => {
            let subline = row.select(&subline_selector).next();
            match subline {
                Some(s) => {
                    match parse_row(&s) {
                        Some(info) => Some(RowType::Info(info)),
                        None => None
                    }
                },
                _ => None
            }
        }
    }

}


fn get_posts(document: &str) -> Option<Vec<Post>> {
    let document = Html::parse_document(document);
    let main_table_selector = Selector::parse("#hnmain > tbody > tr:nth-child(3) > td > table").unwrap();
    let tr_selector = Selector::parse("tr").unwrap();
    let main_table = document.select(&main_table_selector).next()?;
    let mut posts: Vec<Post> = vec![];
    let mut table_rows = main_table.select(&tr_selector);
    while let Some(row) = table_rows.next() {
        if let Some(row_type) = process_row(&row) {
            match row_type {
                RowType::Thing(thing) => {
                    if let Some(nextrow) = table_rows.next() {
                        if let Some(inforow) = process_row(&nextrow) {
                            match inforow {
                                RowType::Info(info) => {
                                    posts.push(Post {
                                        id: thing.id,
                                        rank: thing.rank,
                                        titleline: thing.titleline,
                                        link: thing.link,
                                        score: info.score, 
                                        user: info.user,
                                        date: info.date,
                                        comments: info.comments
                                    });

                                },
                                _ => ()
                            }
                        }
                    }
                }
                _ => ()
            }
        }
    }
    Some(posts)
}


async fn get_day_posts(date: String) -> std::io::Result<usize> {
    let mut params = PageParams {
        day: date.clone(),
        page: 0
    };
    let mut posts: Vec<Post> = vec![];
    while let Ok(page) = get_page(&params).await {
        params.page += 1;
        if let Some(newposts) = get_posts(&page) {
            posts.extend(newposts);
        } else {
            break;
        }
        sleep(TokioDuration::from_millis(1000)).await;
    }
    let file = File::create(format!("./data/{}.json", date))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &posts)?;
    writer.flush()?;
    Ok(posts.len())
}

#[tokio::main]
async fn main() {
    let days = generate_all_days(2007, 10, 1);
    // let res = get_day_posts("2024-01-15".to_string()).await;
    for day in days {
        let res = get_day_posts(day.to_string()).await;
        println!("{day:?} {res:?}");
        sleep(TokioDuration::from_millis(1000)).await;
    }

}
