use reqwest;
use reqwest::{ Result as ReqwestResult };
use scraper::{Html, Selector, Element, ElementRef};

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

#[derive(Debug)]
pub struct Post {
    id: String,
    // rank: String,
    rank: i32,
    titleline: String,
    link: String,
    // score: i32, 
    score: String, 
    user: String,
    date: String,
    // comments: i32
    comments: String
}

async fn get_page() -> ReqwestResult<String> {
    reqwest::get("https://news.ycombinator.com/news")
        .await?
        .text()
        .await
}


fn process_row(row: &ElementRef) -> Option<RowType> {
    let class = row.value().attr("class").unwrap_or("") ;
    let link_selector = Selector::parse("td > span > a").unwrap();
    let rank_selector = Selector::parse("span.rank").unwrap();
    let subline_selector = Selector::parse("span.subline").unwrap();
    match class {
        "spacer" => return Some(RowType::Spacer),
        "athing" => {
            let id = row.value().attr("id").unwrap();
            let link_ele = row.select(&link_selector).next().unwrap();
            let rank_ele = row.select(&rank_selector).next().unwrap();
            return Some(RowType::Thing(Thing {
                id: id.to_string(),
                rank: rank_ele.inner_html(),
                titleline: link_ele.inner_html(),
                link: link_ele.value().attr("href").unwrap().to_string()
            }))
        },
        _ => {
            let subline = row.select(&subline_selector).next();
            match subline {
                Some(s) => {
                    let score_selector = Selector::parse("span.score").unwrap();
                    let user_selector = Selector::parse(".hnuser").unwrap();
                    let date_selector = Selector::parse("span.age").unwrap();
                    let comment_selector = Selector::parse("a").unwrap();

                    // #hnmain > tbody > tr:nth-child(3) > td > table > tbody > tr:nth-child(5) > td.subtext > span > a:nth-child(6)
                    let score = s.select(&score_selector).next();
                    let user = s.select(&user_selector).next();
                    let date = s.select(&date_selector).next();
                    let comment = s.select(&comment_selector)
                        .filter(|e| e.inner_html().contains("comments"))
                        // .collect::<Vec<ElementRef>>()
                        .nth(0)
                        .map_or("".to_string(), |e| e.inner_html().to_string());
                    return Some(RowType::Info( Info {
                        score: score.map_or("".to_string(), |e| e.inner_html().to_string()),
                        user: user.map_or("".to_string(), |e| e.inner_html().to_string()),
                        date: date.map_or("".to_string(), |e| e.value().attr("title").unwrap_or("").to_string()),
                        // comments: comment.map_or("".to_string(), |e| e.inner_html().to_string())
                        comments: comment
                    }));
                },
                _ => return None
            }
        }
    };
    
    None
}


fn get_posts(document: &str) {
    let document = Html::parse_document(document);
    let main_table_selector = Selector::parse("#hnmain > tbody > tr:nth-child(3) > td > table").unwrap();
    let tr_selector = Selector::parse("tr").unwrap();
    let main_table = document.select(&main_table_selector).next().unwrap();
    let table_rows = main_table.select(&tr_selector);
    for row in table_rows {
        let row_type = process_row(&row);
        // match row_type {
        // }
        // println!("ROW");
        println!("{:?}", row_type);
    }
}

#[tokio::main]
async fn main() {
    let res = get_page().await;
    match res {
        Ok(r) => get_posts(&r),
        Err(ref e) => eprintln!("{e:?}")
    }
}
