use serde::Deserialize;
use std::time::Duration;
use reqwest::{self, Client};
use anyhow::Result;

static API_BASE_URL: &str = "https://hacker-news.firebaseio.com/v0";

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Item {
    /// A story.
    Story(Story),
    /// A comment.
    Comment(Comment),
    /// A job.
    Job(Job),
    /// A poll.
    Poll(Poll),
    /// A poll option belonging to a poll.
    Pollopt(Pollopt),
}

impl Item {
    /// Return the id of this item.
    pub fn id(&self) -> u32 {
        match self {
            Item::Story(story) => story.id,
            Item::Comment(comment) => comment.id,
            Item::Job(job) =>job.id,
            Item::Poll(poll) => poll.id,
            Item::Pollopt(pollopt) => pollopt.id,
        }
    }

    /// Return the title of this item, if available.
    pub fn title(&self) -> Option<&str> {
        match self {
            Item::Story(story) => Some(&story.title),
            Item::Job(job) => Some(&job.title),
            Item::Poll(poll) => Some(&poll.title),
            _ => None,
        }
    }

    /// Return the author of this item, if available.
    pub fn author(&self) -> Option<&str> {
        match self {
            Item::Story(story) => Some(&story.by),
            Item::Comment(comment) => Some(&comment.by),
            Item::Poll(poll) => Some(&poll.by),
            Item::Pollopt(pollopt) => Some(&pollopt.by),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Story {
    /// The item's unique id.
    pub id: u32,
    /// The total comment count.
    pub descendants: u32,
    /// The username of the item's author.
    pub by: String,
    /// The ids of the item's comments, in ranked display order.
    pub kids: Option<Vec<u32>>,
    /// The story's score.
    pub score: u32,
    /// The title of the story.
    pub title: String,
    /// The URL of the story.
    pub url: Option<String>,
    /// The story text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
}

/// A comment.
#[derive(Debug, Deserialize)]
pub struct Comment {
    /// The item's unique id.
    pub id: u32,
    /// The username of the item's author.
    pub by: String,
    /// The ids of the item's comments, in ranked display order.
    pub kids: Option<Vec<u32>>,
    /// The comment's parent: either another comment or the relevant story.
    pub parent: u32,
    /// The comment text. HTML.
    pub text: String,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
}

/// A job.
#[derive(Debug, Deserialize)]
pub struct Job {
    /// The item's unique id.
    pub id: u32,
    /// The story's score, or the votes for a pollopt.
    pub score: u32,
    /// The job text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
    /// The title of the job.
    pub title: String,
    /// The URL of the story.
    pub url: Option<String>,
}

/// A poll.
#[derive(Debug, Deserialize)]
pub struct Poll {
    /// The item's unique id.
    pub id: u32,
    /// The username of the item's author.
    pub by: String,
    /// The total comment count.
    pub descendants: u32,
    /// The ids of the item's comments, in ranked display order.
    pub kids: Option<Vec<u32>>,
    /// A list of related pollopts, in display order.
    pub parts: Option<Vec<u32>>,
    /// The story's score.
    pub score: u32,
    /// The title of the story.
    pub title: String,
    /// The story text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
}

/// A poll option belonging to a poll.
#[derive(Debug, Deserialize)]
pub struct Pollopt {
    /// The item's unique id.
    pub id: u32,
    /// The username of the item's author.
    pub by: String,
    /// The pollopt's associated poll.
    pub poll: u32,
    /// The votes for a pollopt.
    pub score: u32,
    /// The story text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
}

/// A user profile.
#[derive(Debug, Deserialize)]
pub struct User {
    /// The user's unique username. Case-sensitive.
    pub id: String,
    /// Creation date of the user, in Unix Time.
    pub created: u64,
    /// The user's karma.
    pub karma: u32,
    /// Delay in minutes between a comment's creation and its visibility to
    /// other users.
    pub delay: Option<u32>,
    /// The user's optional self-description. HTML.
    pub about: Option<String>,
    /// List of the user's stories, polls and comments.
    pub submitted: Vec<u32>,
}

/// A list of recently updated items and users.
#[derive(Debug, Deserialize)]
pub struct Updates {
    /// A list of recently changed items.
    pub items: Vec<u32>,
    /// A list of recently changed usernames.
    pub profiles: Vec<String>,
}

/// The API client.
pub struct HnClient {
    client: Client,
}

impl HnClient {

    /// Create a new `HnClient` instance.
    pub fn init() -> reqwest::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self { client })
    }

    /// Return the item with the specified id.
    ///
    /// May return `None` if item id is invalid.
    pub async fn get_item(&self, id: u32) -> reqwest::Result<Option<Item>> {
        // self.client.get(&format!("{}/item/{}.json", API_BASE_URL, id)).send().await?.json().await?
        self.client.get(&format!("{}/item/{}.json", API_BASE_URL, id)).send().await?.json().await
    }

    /// Return the user with the specified username.
    ///
    /// May return `None` if username is invalid.
    pub async fn get_user(&self, username: &str) -> reqwest::Result<Option<User>> {
        self.client.get(&format!("{}/user/{}.json", API_BASE_URL, username)).send().await?.json().await
    }

    /// Return the id of the newest item.
    ///
    /// To get the 10 latest items, you can decrement the id 10 times.
    pub async fn get_max_item_id(&self) -> reqwest::Result<u32> {
        self.client.get(&format!("{}/maxitem.json", API_BASE_URL)).send().await?.json().await
    }

    /// Return a list of top story item ids.
    pub async fn get_top_stories(&self) -> reqwest::Result<Vec<u32>> {
        self.client.get(&format!("{}/topstories.json", API_BASE_URL)).send().await?.json().await
    }

    /// Return a list of new story item ids.
    pub async fn get_new_stories(&self) -> reqwest::Result<Vec<u32>> {
        self.client.get(&format!("{}/newstories.json", API_BASE_URL)).send().await?.json().await
    }

    /// Return a list of best story item ids.
    pub async fn get_best_stories(&self) -> reqwest::Result<Vec<u32>> {
        self.client.get(&format!("{}/beststories.json", API_BASE_URL)).send().await?.json().await
    }

    /// Return up to 200 latest Ask HN story item ids.
    pub async fn get_ask_stories(&self) -> reqwest::Result<Vec<u32>> {
        self.client.get(&format!("{}/askstories.json", API_BASE_URL)).send().await?.json().await
    }

    /// Return up to 200 latest Show HN story item ids.
    pub async fn get_show_stories(&self) -> reqwest::Result<Vec<u32>> {
        self.client.get(&format!("{}/showstories.json", API_BASE_URL)).send().await?.json().await
    }

    /// Return up to 200 latest Job story item ids.
    pub async fn get_job_stories(&self) -> reqwest::Result<Vec<u32>> {
        self.client.get(&format!("{}/jobstories.json", API_BASE_URL)).send().await?.json().await
    }

    /// Return a list of items and users that have been updated recently.
    pub async fn get_updates(&self) -> reqwest::Result<Updates> {
        self.client.get(&format!("{}/updates.json", API_BASE_URL)).send().await?.json().await
    }
    

}
