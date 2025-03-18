pub struct Post {
    content: String,
}

pub struct DraftPost {
    content: String,
}

impl Post {
    pub fn new() -> DraftPost {
        DraftPost {
            content: String::new(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl DraftPost {
    // --snip--
    pub fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    pub fn request_review(self) -> PendingReviewPost {
        PendingReviewPost {
            content: self.content,
            flag: 0,
        }
    }
}

pub struct PendingReviewPost {
    content: String,
    flag: i8,
}

impl PendingReviewPost {
    // pub fn approve(self) -> Post {
    //     Post {
    //         content: self.content,
    //     }
    // }
    pub fn approve(self) -> Result<Post, PendingReviewPost>{
        let flag_new = self.flag + 1;
        if flag_new == 2 {
            Ok(Post {
                content: self.content,
            })
        } else {
            Err(PendingReviewPost {
                content: self.content,
                flag: flag_new,
            })
        }
    }

    pub fn reject(self) -> DraftPost {
        DraftPost {
            content: self.content,
        }
    }
}




// use blog::Post;

fn main() {
    let mut post = Post::new();

    post.add_text("I ate a salad for lunch today");

    let post = post.request_review();

    let post = post.approve();

    assert_eq!("I ate a salad for lunch today", post.content());
}