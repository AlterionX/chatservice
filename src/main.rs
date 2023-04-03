use std::{sync::OnceLock, collections::HashMap};

use rocket::{FromForm, form::Form, http::Status, response::{content::RawHtml, Redirect}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub user: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(FromForm)]
pub struct CommentForm<'req> {
    pub user: &'req str,
    pub body: &'req str,
}

#[derive(Debug, Default, Clone)]
pub struct CommentStore {
    pub data: HashMap<String, Vec<Comment>>,
}

impl CommentStore {
    pub fn fetch_comments_for_page(&mut self, page_id: &str) -> Option<&mut Vec<Comment>> {
        self.data.get_mut(page_id)
    }

    pub fn fetch_or_create_comments_for_page(&mut self, page_id: &str) -> &mut Vec<Comment> {
        self.data.entry(page_id.to_owned()).or_default()
    }
}

static mut COMMENT_STORE: OnceLock<CommentStore> = OnceLock::new();

#[rocket::get("/pages/<page_id>/comments")]
fn get_comments(page_id: &str) -> Redirect {
    Redirect::permanent(format!("/pages/{page_id}"))
}

// TODO Fix html injection.
#[rocket::get("/pages/<page_id>")]
fn get_page(page_id: &str) -> Result<RawHtml<String>, Status> {
    let Some(store) = (unsafe { COMMENT_STORE.get_mut() }) else {
        return Err(Status::InternalServerError);
    };
    let Some(comments) = store.fetch_comments_for_page(page_id) else {
        return Err(Status::NotFound);
    };

    let mut comments_html = String::new();
    for (comment_id, Comment { user, body }) in comments.iter().enumerate() {
        comments_html.push_str(format!("\
<div id=\"page-{page_id}-comment-{comment_id}\" class=\"comment\">
    <p class=\"comment-body\"><em class=\"comment-user\" style=\"display:inline\">{user}: </em>{body}</p>
</div>\
        ").as_str());
    }

    let comment_form = format!("\
    <form id=\"comment-form\" action=\"/pages/{page_id}/comments\" method=\"post\">
      <label for=\"user\">Username:</label>
      <br>
      <input id=\"user\" name=\"user\" type=\"text\" maxlength=\"50\" size=\"20\" pattern=\"[A-Za-z0-9]+\" title=\"A-Z, a-z, 0-9 only\" placeholder=\"username\" />
      <br>

      <label for=\"body\">Comment:</label>
      <br>
      <textarea id=\"body\" name=\"body\" maxlength=\"1000\" cols=\"50\" rows=\"5\" placeholder=\"comment\"></textarea>
      <br>
      <br>

      <input type=\"submit\" value=\"Submit\">
    </form>\
    ");

    Ok(RawHtml(format!("\
<!DOCTYPE html>
<html>
<head>
    <title>{page_id}</title>
    <meta charset=\"utf-8\">
</head>
<body>
    {comments_html}
    {comment_form}
</body>
</html>\
    ")))
}

#[rocket::post("/pages/<page_id>/comments", data = "<comment>")]
fn post_comment(page_id: &str, comment: Form<CommentForm<'_>>) -> Result<Redirect, Status> {
    unsafe { COMMENT_STORE.get_or_init(|| CommentStore::default()); }
    let Some(store) = (unsafe { COMMENT_STORE.get_mut() }) else {
        return Err(Status::InternalServerError);
    };
    let comments = store.fetch_or_create_comments_for_page(page_id);

    comments.push(Comment {
        user: comment.user.to_owned(),
        body: comment.body.to_owned(),
    });

    Ok(Redirect::to(format!("/pages/{page_id}")))
}

#[rocket::post("/pages", data = "<page>")]
fn post_page(page: &str) -> Result<Redirect, Status> {
    unsafe { COMMENT_STORE.get_or_init(|| CommentStore::default()); }
    let Some(store) = (unsafe { COMMENT_STORE.get_mut() }) else {
        return Err(Status::InternalServerError);
    };
    let _ = store.fetch_or_create_comments_for_page(page);

    Ok(Redirect::to(format!("/pages/{page}")))
}

#[rocket::launch]
fn server() -> _ {
    post_page("hello-world").expect("no issues");

    rocket::build().mount("/", rocket::routes![get_page, post_page, get_comments, post_comment])
}
