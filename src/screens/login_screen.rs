use egui::CentralPanel;

pub fn login() -> CentralPanel  {
    CentralPanel::default().show(ctx, |ui| {
        ui.label("Login");
    })
}