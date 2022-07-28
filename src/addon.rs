use super::Esoui;

#[derive(Debug)]
pub struct Addon {
    pub name: String,
    pub author: String,
    pub size: String,
    pub downloads: String,
    pub last_update: String,
    pub url: String,
    pub page: Page,
}

///分页
#[derive(Debug)]
pub struct Page {
    ///总数
    pub count: i32,
    ///当前页数
    pub page: i32,
    ///每页数量
    pub page_count: i32,
}
