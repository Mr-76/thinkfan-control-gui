use glib::clone;
use gtk::{self, Button, Orientation};
use gtk::{glib, prelude::*};
use std::cell::Cell;
use std::fs::File;
use std::io::{self, Read, Write};
use std::rc::Rc;

fn main() -> glib::ExitCode {
    let application = gtk::Application::builder()
        .application_id("com.github.Mr-76.thinkpadfan")
        .build();
    application.connect_activate(build_ui);
    application.run()
} 

///Base function to build the user interface...
///builds the 2 buttons and the fan level label and fan speed label
fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("THINKPAD FAN CONTROL"));
    window.set_default_size(260, 400);

    let label_fan_speed = gtk::Label::default();
    let label_level = gtk::Label::default();

    let gtk_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let button_increase = Button::builder()
        .label("INCREASE")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let button_decrease = Button::builder()
        .label("DECREASE")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let button_auto = Button::builder()
        .label("AUTO")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let fan_level = Rc::new(Cell::new(0));


    button_auto.connect_clicked(clone!(move |_| {
        let _ = set_speed("auto");
    }));

    //TODO: Need to handle auto string whe geting fan level....
    button_increase.connect_clicked(clone!(
        #[weak]
        fan_level,
        #[strong]
        button_decrease,
        move |_| {
            if fan_level.get() >= 7 {
            } else {
                fan_level.set(fan_level.get() + 1);
                let _ = set_speed(&fan_level.get().to_string());
            }
        }
    ));
    button_decrease.connect_clicked(clone!(
        #[strong]
        button_increase,
        move |_| {
            if fan_level.get() <= 0 {
            } else {
                fan_level.set(fan_level.get() - 1);
                let _ = set_speed(&fan_level.get().to_string());
            }
        }
    ));

    match get_current_fan_level() {
        Ok(speed) => {
            label_level.set_text(&format!("Fan Level: {}", speed));
        }
        Err(e) => {
            eprintln!("Failed to get fan speed Level: {}", e);
            label_level.set_text("Fan Speed Level: Error");
        }
    }

    match get_current_fan_speed() {
        Ok(speed) => {
            label_fan_speed.set_text(&format!("Fan Speed: {}", speed));
        }
        Err(e) => {
            eprintln!("Failed to get fan speed: {}", e);
            label_fan_speed.set_text("Fan Speed: Error");
        }
    }

    gtk_box.append(&button_increase);
    gtk_box.append(&button_decrease);
    gtk_box.append(&button_auto);
    gtk_box.append(&label_fan_speed);
    gtk_box.append(&label_level);

    window.set_child(Some(&gtk_box));

    window.present();

    // we are using a closure to capture the label (else we could also use a normal
    // function)
    let tick = move || {
        match get_current_fan_speed() {
            Ok(speed) => {
                label_fan_speed.set_text(&format!("Fan Speed: {}", speed));
            }
            Err(e) => {
                eprintln!("Failed to get fan speed: {}", e);
                label_fan_speed.set_text("Fan Speed: Error");
            }
        }

        match get_current_fan_level() {
            Ok(speed) => {
                label_level.set_text(&format!("Fan Level: {}", speed));
            }
            Err(e) => {
                eprintln!("Failed to get fan speed Level: {}", e);
                label_level.set_text("Fan Speed Level: Error");
            }
        }

        glib::ControlFlow::Continue
    };
    // executes the closure once every second
    glib::timeout_add_seconds_local(1, tick);
}

///Sets the fan speed level
///can be 0 -7 and auto
fn set_speed(speed: &str) -> io::Result<()> {
    let new_speed = format!("level {}", speed); //deve escrever sem o : para funcionar

    let mut file = File::create("/proc/acpi/ibm/fan")?;
    file.write_all(new_speed.as_bytes())?;

    Ok(())
}

///Retreives the current fan speed in rpm
fn get_current_fan_speed() -> io::Result<i32> {
    let mut file = File::open("/proc/acpi/ibm/fan")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    contents
        .lines()
        .find(|line| line.starts_with("speed:"))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|speed| speed.trim().parse().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse fan speed"))
}

///Retreives the current fan speed level
fn get_current_fan_level() -> io::Result<String> {
    let mut file = File::open("/proc/acpi/ibm/fan")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    if let Some(level_line) = contents.lines().find(|line| line.starts_with("level:")) {
        if let Some(level) = level_line.split(':').nth(1) {
            return Ok(level.trim().to_string());
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Fan level not found",
    ))
}
