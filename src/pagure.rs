use serde::Deserialize;

#[derive(Debug, serde::Deserialize)]
pub struct Args {
    assignee: Option<String>,
    author: Option<String>,
    milestones: Vec<String>,
    no_stones: Option<String>,
    order: Option<String>,
    priority: Option<String>,
    since: Option<String>,
    status: String,
    tags: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Board {
    active: bool,
    full_url: String,
    name: String,
    status: Vec<BoardStatus>,
    tag: Tag,
}

#[derive(Debug, serde::Deserialize)]
pub struct BoardStatus {
    bg_color: String,
    close: bool,
    close_status: Option<String>,
    default: bool,
    name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Tag {
    tag: String,
    tag_color: String,
    tag_description: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Comment {
    comment: String,
    date_created: String,
    edited_on: Option<String>,
    editor: Option<String>,
    id: u64,
    notification: bool,
    parent: Option<String>,
    reactions: serde_json::Value,
    user: User,
}

#[derive(Debug, serde::Deserialize)]
pub struct User {
    full_url: String,
    fullname: String,
    name: String,
    url_path: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Issue {
    assignee: Option<String>,
    blocks: Vec<String>,
    boards: Vec<Board>,
    close_status: Option<String>,
    closed_at: Option<String>,
    closed_by: Option<String>,
    comments: Vec<Comment>,
    content: String,
    custom_fields: Vec<serde_json::Value>,
    date_created: String,
    depends: Vec<String>,
    full_url: String,
    id: u64,
    last_updated: String,
    milestone: Option<String>,
    priority: Option<String>,
    private: bool,
    related_prs: Vec<String>,
    status: String,
    tags: Vec<String>,
    title: String,
    user: User,
}

#[derive(Debug, serde::Deserialize)]
pub struct Pagination {
    first: String,
    last: String,
    next: Option<String>,
    page: u64,
    pages: u64,
    per_page: u64,
    prev: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PagureResponse {
    args: Args,
    issues: Vec<Issue>,
    pagination: Pagination,
    total_issues: u64,
}