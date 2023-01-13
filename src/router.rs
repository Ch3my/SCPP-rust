use egui::{CentralPanel, InnerResponse, Response};

pub fn router(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> InnerResponse<()> {
    let mut result: InnerResponse<()> = CentralPanel::default().show(ctx, |ui| {});
    if (&self.route == "/login") {

        result = CentralPanel::default().show(ctx, |ui| {
            // Username Input
            ui.label("Enter your username");
            let username_input: Response = ui.text_edit_singleline(&mut self.login_form.username);

            if username_input.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                println!("login_form.username: {}", &self.login_form.username);
            }

            // Password Input
            ui.label("Enter your Password");
            let pass_input: Response = ui.text_edit_singleline(&mut self.login_form.pass);
            if pass_input.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                println!("login_form.pass: {}", &self.login_form.pass);
            }
        });
    }
    if (&self.route == "/docs") {
        result = CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello World");
        })
    }
    return result;
}
