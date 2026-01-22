pub mod creature;
pub mod github;
pub mod hackernews;
pub mod rss;
pub mod sports;
pub mod stocks;

use crate::feeds::{FeedData, FeedFetcher};
use ratatui::{Frame, layout::Rect};
use std::any::Any;

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

    /// For downcasting to concrete types
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }

    /// For mutable downcasting to concrete types
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        None
    }
}
