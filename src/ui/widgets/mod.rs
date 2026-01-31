pub mod creature;
pub mod github;
pub mod hackernews;
pub mod rss;
pub mod sports;
pub mod stocks;
pub mod youtube;

use crate::feeds::{FeedData, FeedFetcher};
use ratatui::{Frame, layout::Rect};
use std::any::Any;

/// Information about a selected feed item for reading or opening
#[derive(Debug, Clone)]
pub struct SelectedItem {
    pub title: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub source: String,
    pub metadata: Option<String>,
}

pub trait FeedWidget: Send + Sync {
    fn id(&self) -> String;
    fn title(&self) -> &str;
    fn position(&self) -> (usize, usize);
    fn render(&self, frame: &mut Frame, area: Rect, selected: bool);
    fn update_data(&mut self, data: FeedData);
    fn create_fetcher(&self) -> Box<dyn FeedFetcher>;
    fn scroll_up(&mut self);
    fn scroll_down(&mut self);
    fn set_selected(&mut self, selected: bool);

    /// Get the currently selected item's information
    fn get_selected_item(&self) -> Option<SelectedItem> {
        None
    }

    /// For downcasting to concrete types
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }

    /// For mutable downcasting to concrete types
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        None
    }
    fn get_selected_discussion_url(&self) -> Option<String>;

    
}
