pub struct PagePointer {
    pub prev: Option<String>,
    pub current: String,
    pub next: Option<String>,
}

impl PagePointer {
    pub fn new_index(total_pages: usize) -> PagePointer {
        PagePointer {
            prev: None,
            current: "./".to_string(),
            next: if total_pages > 1 {
                Some("./2.html".to_string())
            } else {
                None
            },
        }
    }

    pub fn new_paginated(page_num: usize, total_pages: usize) -> PagePointer {
        debug_assert!(page_num >= 2, "page_num must be >= 2");
        PagePointer {
            prev: Some(if page_num == 2 {
                "./".to_string()
            } else {
                Self::format_html(page_num - 1)
            }),
            current: Self::format_html(page_num),
            next: if page_num == total_pages {
                None
            } else {
                Some(Self::format_html(page_num + 1))
            },
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
    fn test_new_index() {
        // one page
        let pointer = PagePointer::new_index(1);
        assert_eq!(pointer.prev, None);
        assert_eq!(pointer.current, "./".to_string());
        assert_eq!(pointer.next, None);

        // multiple pages
        let pointer = PagePointer::new_index(2);
        assert_eq!(pointer.prev, None);
        assert_eq!(pointer.current, "./".to_string());
        assert_eq!(pointer.next, Some("./2.html".to_string()));
    }

    #[test]
    fn test_new_paginated() {
        // number of pages is 2. the last page
        let pointer = PagePointer::new_paginated(2, 2);
        assert_eq!(pointer.prev, Some("./".to_string()));
        assert_eq!(pointer.current, "./2.html".to_string());
        assert_eq!(pointer.next, None);

        // number of pages is 2. not the last page
        let pointer = PagePointer::new_paginated(2, 3);
        assert_eq!(pointer.prev, Some("./".to_string()));
        assert_eq!(pointer.current, "./2.html".to_string());
        assert_eq!(pointer.next, Some("./3.html".to_string()));

        // number of pages is 3
        let pointer = PagePointer::new_paginated(3, 3);
        assert_eq!(pointer.prev, Some("./2.html".to_string()));
        assert_eq!(pointer.current, "./3.html".to_string());
        assert_eq!(pointer.next, None);
    }

    #[test]
    #[should_panic(expected = "page_num must be >= 2")]
    fn test_new_paginated_invalid_page() {
        PagePointer::new_paginated(1, 2);
    }
}
