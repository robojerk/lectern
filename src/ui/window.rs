use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Stack, StackTransitionType, HeaderBar, Button};
use crate::ui::pages::drop_zone::DropZonePage;
use crate::ui::pages::editor::EditorPage;

pub struct LecternWindow {
    pub window: ApplicationWindow,
    pub stack: Stack,
    pub settings_btn: Button,
    pub editor_page: EditorPage,
}

impl LecternWindow {
    pub fn new(app: &Application, tx: glib::Sender<crate::app_event::AppEvent>) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Lectern - Audiobook Preparer")
            .default_width(1000)
            .default_height(800)
            .build();

        let header = HeaderBar::new();
        window.set_titlebar(Some(&header));
        
        let settings_btn = Button::with_label("⚙ Settings");
        header.pack_end(&settings_btn);
        
        let stack = Stack::new();
        stack.set_transition_type(StackTransitionType::SlideLeftRight);
        
        // Page 1: Drop Zone
        let drop_zone = DropZonePage::new();
        stack.add_named(&drop_zone.container, Some("drop_zone"));
        
        // Page 2: Editor
        let editor_page = EditorPage::new(tx);
        stack.add_named(&editor_page.container, Some("editor"));
        
        window.set_child(Some(&stack));

        Self { window, stack, settings_btn, editor_page }
    }
    
    pub fn present(&self) {
        self.window.present();
    }
}
