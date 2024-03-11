use ratatui::Frame;
use tokio::sync::mpsc;

pub trait Screen {
    type Action;

    fn handle_event(self, action: Self::Action, tx: mpsc::UnboundedSender<Self::Action>);
    fn render(self, frame: Frame);
}
