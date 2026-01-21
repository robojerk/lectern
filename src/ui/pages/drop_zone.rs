use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Align};

pub struct DropZonePage {
    pub container: Box,
}

impl DropZonePage {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 12);
        container.set_valign(Align::Center);
        container.set_halign(Align::Center);
        
        let drop_label = Label::builder()
            .label("📁 Drag & Drop Audiobook Folder Here")
            .css_classes(["drag-area"])
            .height_request(200)
            .width_request(400)
            .build();
            
        container.append(&drop_label);
        
        Self { container }
    }
}
