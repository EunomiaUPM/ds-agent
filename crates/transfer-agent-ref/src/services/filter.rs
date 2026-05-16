pub struct Page {
    pub limit: u32,
    pub cursor: Option<String>,
}

pub struct Paginated<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

pub enum Sort {
    CreatedAtAsc,
    CreatedAtDesc,
    UpdatedAtDesc,
}
