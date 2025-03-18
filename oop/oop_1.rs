
pub struct Post {
    state: Option<Box<dyn State>>,
    content: String,
    flag: i8,
}

impl Post {
    pub fn new() -> Post {
        Post {
            state: Some(Box::new(Draft {})),
            content: String::new(),
            flag: 0,
        }
    }

    pub fn add_text(&mut self, text: &str) {
        if let Some(state) = &self.state {
            if state.check_draft() {
                self.content.push_str(text);
            } else {
                println!("Error");
            }
        }
    }

    pub fn content(&self) -> &str {
        self.state.as_ref().unwrap().content(self)
    }

    pub fn request_review(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.request_review())
        }
    }

    pub fn approve(&mut self) {
        if let Some(s) = self.state.take() {
            let (new_state, approved) = s.approve(self.flag);

            self.state = Some(new_state);
            if approved {
                self.flag = 0; 
            } else {
                self.flag += 1; 
            }
        }
        // if let Some(s) = self.state.take() {
        //     self.state = Some(s.approve())
        // }
    }

    pub fn reject(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.reject())
        }
    }

}

trait State {
    // --snip--
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>, flag: i8) -> (Box<dyn State>, bool);
    fn reject(self: Box<Self>) -> Box<dyn State>;

    
    
    fn content<'a>(&self, _post: &'a Post) -> &'a str {
        ""
    }

    fn check_draft(&self) -> bool {
        false
    }
}

// --snip--

struct Draft {}

impl State for Draft {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        Box::new(PendingReview {})
    }

    fn approve(self: Box<Self>, _flag: i8) -> (Box<dyn State>, bool) {
        (self, false)
    }

    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn check_draft(&self) -> bool {
        true
    }
}

struct PendingReview {}

impl State for PendingReview {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn approve(self: Box<Self>, flag: i8) -> (Box<dyn State>, bool) {
        if flag + 1 == 2 {
            (Box::new(Published {}), true)
        } else {
            (self, false)
        }
        
    }

    fn reject(self: Box<Self>) -> Box<dyn State> {
        Box::new(Draft {})
    }

    fn check_draft(&self) -> bool {
        false
    }
}

struct Published {}

impl State for Published {
    // --snip--
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn approve(self: Box<Self>, _flag: i8) -> (Box<dyn State>, bool) {
        (self, true)
    }

    fn content<'a>(&self, post: &'a Post) -> &'a str {
        &post.content
    }

    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn check_draft(&self) -> bool {
        false
    }
}


fn main() {
    let mut post = Post::new();

    post.add_text("I ate a salad for lunch today");
    assert_eq!("", post.content());

    post.request_review();
    // assert_eq!("", post.content());

    post.approve(); // flag становится 1
    assert_eq!("", post.content());

    post.approve(); // flag 0
    assert_eq!("I ate a salad for lunch today", post.content());
    post.add_text(" text");
    assert_eq!("I ate a salad for lunch today", post.content());
}