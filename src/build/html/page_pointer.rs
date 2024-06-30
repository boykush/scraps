pub struct PagePointer {
    pub prev: Option<String>,
    pub current: String,
    pub next: Option<String>,
}

impl PagePointer {
    pub fn new(page_num: usize, last_page_num: usize) -> PagePointer {
        let prev = match page_num {
            1 => None,
            2 => Some("./".to_string()),
            n => Some(Self::format_html(n - 1)),
        };
        let current = match page_num {
            1 => "./".to_string(),
            n => Self::format_html(n),
        };
        let next = match page_num {
            n if n == last_page_num => None,
            n => Some(Self::format_html(n + 1)),
        };
        PagePointer {
            prev,
            current,
            next,
        }
    }

    fn format_html(u: usize) -> String {
        format!("./{u}.html")
    }

    pub fn is_index(&self) -> bool {
        self.current == *"./".to_string()
    }

    pub fn current_file_name(&self) -> String {
        if Self::is_index(self) {
            "index.html".to_string()
        } else {
            self.current.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PagePointer;

    #[test]
    fn it_new_prev() {
        assert_eq!(PagePointer::new(1, 1).prev, None);
        assert_eq!(PagePointer::new(2, 2).prev, Some("./".to_string()));
        assert_eq!(PagePointer::new(3, 3).prev, Some("./2.html".to_string()));
    }

    #[test]
    fn it_new_current() {
        assert_eq!(PagePointer::new(1, 1).current, "./".to_string());
        assert_eq!(PagePointer::new(2, 2).current, "./2.html".to_string());
    }

    #[test]
    fn it_new_next() {
        assert_eq!(PagePointer::new(1, 1).next, None);
        assert_eq!(PagePointer::new(1, 2).next, Some("./2.html".to_string()));
        assert_eq!(PagePointer::new(2, 2).next, None);
    }
}
