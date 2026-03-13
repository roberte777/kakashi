use iced::{
    Background, Border, Color, Element, Event, Subscription, Theme, event, keyboard,
    widget::{column, container, text_input},
};
use iced_layershell::{
    Settings,
    reexport::{Anchor, Layer},
    settings::LayerShellSettings,
    to_layer_message,
};

#[derive(Debug, Clone, Default)]
struct AppState {
    input: String,
    // Filled this out based on what I believe will be needed
    _selected_entry: Option<String>,
    _entries: Vec<String>,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Query(String),
    Submit(String),
    Event(iced::Event),
}

pub fn run_iced() -> std::result::Result<(), iced_layershell::Error> {
    iced_layershell::application(AppState::default, namespace, update, view)
        .subscription(subscription)
        .theme(Theme::Dark)
        .settings(Settings {
            // Still playing around with the values
            layer_settings: LayerShellSettings {
                anchor: Anchor::Top,
                size: Some((800, 500)),
                margin: (100, 0, 0, 0),
                layer: Layer::Overlay, // Appears on top of everything
                ..Default::default()
            },
            ..Default::default()
        })
        .style(|_state: &AppState, theme: &Theme| iced::theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        })
        .run()
}

fn namespace() -> String {
    "kakashi".to_string()
}

fn update(state: &mut AppState, message: Message) -> iced::Task<Message> {
    match message {
        Message::Query(input) => state.input = input,
        // Should take the selected entry and try to launch it
        Message::Submit(input) => println!("{}", input),
        Message::Event(Event::Keyboard(keyboard::Event::KeyPressed { key, .. })) => {
            match key {
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    // Assumming this would close the app like raycast does
                    println!("Pressed Escape");
                    return iced::exit();
                }
                keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                    // Assuming this would cycle through the entries
                    println!("Pressed Up");
                }
                keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                    // Assuming this would cycle through the entries
                    println!("Pressed Down");
                }
                _ => {}
            }
        }
        _ => {}
    }
    iced::Task::none()
}

fn view(state: &AppState) -> Element<'_, Message> {
    // Search bar container
    let search_bar = container(
        text_input("Search application...", &state.input)
            .on_input(Message::Query)
            .on_submit(Message::Submit(state.input.clone()))
            .width(iced::Length::Fill)
            .padding(12),
    )
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .padding(16)
    .style(|theme: &Theme| {
        let palette = theme.palette();

        container::Style {
            background: Some(Background::Color(Color {
                a: 0.92,
                ..palette.background
            })),
            text_color: Some(palette.text),
            border: Border {
                radius: 20.0.into(),
                width: 2.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    });

    // TOOD: Possibly add a divider and list of entries sort of what raycast looks like

    // Main Content to be rendered within a single column
    let content = column!(search_bar);
    container(content).into()
}

// Event subscription
fn subscription(_state: &AppState) -> Subscription<Message> {
    event::listen().map(Message::Event)
}
