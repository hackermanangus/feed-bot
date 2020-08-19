pub mod main {

    pub struct Chapter {
        title: String,
        link: String,
    }

    impl Chapter {
        pub fn new(title: String, link: String) -> Chapter {
            Chapter {
                title,
                link
            }
        }
    }

    pub struct Novel {
        title: String,
        link: String,
        chapters: Vec<Chapter>,
    }
    impl Novel {
        pub fn new(title: String, link: String, chapters: Vec<Chapter>) -> Novel {
            Novel {
                title,
                link,
                chapters
            }
        }
    }
}