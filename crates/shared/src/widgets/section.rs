use egui::{Align, Response, Ui, WidgetText};

pub trait Section {
    fn section(&mut self, name: impl Into<WidgetText>, add_contents: impl FnMut(&mut egui::Ui) -> egui::Response) -> Response;
}

impl Section for Ui {
    fn section(&mut self, name: impl Into<WidgetText>, mut add_contents: impl FnMut(&mut Ui) -> Response) -> Response {
        self.with_layout(egui::Layout::top_down(Align::Center), |ui| {
            ui.label(name);
        }).response | add_contents(self) | self.separator()
    }
}
