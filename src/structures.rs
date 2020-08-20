#[derive(Debug)]
pub struct Chapter {
    pub title: String,
    pub link: String,
}

impl Chapter {
    pub fn new(title: String, link: String) -> Chapter {
        Chapter {
            title,
            link,
        }
    }
}

#[derive(Debug)]
pub struct Novel {
    pub title: String,
    pub link: String,
    pub chapters: Vec<Chapter>,
}

impl Novel {
    pub fn new(title: String, link: String, chapters: Vec<Chapter>) -> Novel {
        Novel {
            title,
            link,
            chapters,
        }
    }
}