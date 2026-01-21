// src/ui/settings.rs
impl SettingsWindow {
    fn setup_audiobookshelf_section(&self) {
        // Create a toggle switch to enable/disable Audiobookshelf integration
        let enable_switch = gtk::Switch::new();
        enable_switch.set_valign(gtk::Align::Start);
        enable_switch.set_halign(gtk::Align::Start);
        
        // Connect the switch to show/hide settings
        enable_switch.connect_state_set(clone!(@strong self as this => move |_, is_active| {
            this.audiobookshelf_grid.set_visible(is_active);
            Inhibit(false)
        }));
        
        // Add the switch to the UI
        self.settings_box.add(&enable_switch);
        
        // Create a grid for the actual settings (hidden by default)
        let grid = gtk::Grid::new();
        grid.set_visible(false);
        
        // Add URL, API key, and library ID fields
        // ...
        
        self.settings_box.add(&grid);
    }
}