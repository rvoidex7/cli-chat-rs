use cli_chat_rs::{Config, DemoAdapter, MessengerApp, KeyboardHandler, Action};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config_path = std::env::var("CLI_CHAT_CONFIG")
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".cli-chat-rs")
                .join("config.json")
                .to_string_lossy()
                .to_string()
        });
    
    let config = Config::load(&std::path::PathBuf::from(&config_path))
        .unwrap_or_else(|_| Config::default());

    // Create demo adapter (in a real app, this would be selected based on config)
    let adapter = Box::new(DemoAdapter::new());
    let mut app = MessengerApp::new(config, adapter);

    // Connect to the messaging service
    println!("Connecting to {}...", app.adapter().name());
    app.adapter_mut().connect().await.map_err(|e| format!("Connection error: {}", e))?;
    println!("Connected!");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create keyboard handler
    let keyboard_handler = KeyboardHandler::new(app.config().shortcuts.clone());

    // Run the UI
    let result = run_ui(&mut terminal, &mut app, &keyboard_handler).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_ui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut MessengerApp,
    keyboard_handler: &KeyboardHandler,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut selected_chat = 0;
    let mut input_message = String::new();
    let mut show_help = false;

    loop {
        // Get chats
        let chats = app.adapter().get_chats().await.map_err(|e| format!("Failed to get chats: {}", e))?;

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ])
                .split(size);

            // Main content area
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(chunks[0]);

            // Chat list (sidebar)
            let chat_items: Vec<ListItem> = chats
                .iter()
                .enumerate()
                .map(|(i, chat)| {
                    let style = if i == selected_chat {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    
                    let unread = if chat.unread_count > 0 {
                        format!(" ({})", chat.unread_count)
                    } else {
                        String::new()
                    };
                    
                    ListItem::new(format!("{}{}", chat.name, unread)).style(style)
                })
                .collect();

            let chat_list = List::new(chat_items)
                .block(Block::default().borders(Borders::ALL).title("Chats"));
            f.render_widget(chat_list, main_chunks[0]);

            // Message area
            let messages_block = Block::default()
                .borders(Borders::ALL)
                .title(if selected_chat < chats.len() {
                    chats[selected_chat].name.clone()
                } else {
                    "No chat selected".to_string()
                });
            
            let welcome_text = if show_help {
                let shortcuts = keyboard_handler.get_shortcuts_help();
                let lines: Vec<Line> = shortcuts
                    .iter()
                    .map(|(key, desc)| {
                        Line::from(vec![
                            Span::styled(format!("{:15}", key), Style::default().fg(Color::Cyan)),
                            Span::raw(desc.clone()),
                        ])
                    })
                    .collect();
                Paragraph::new(lines).block(messages_block)
            } else {
                Paragraph::new(format!(
                    "Welcome to CLI Chat RS!\n\n\
                    Connected to: {}\n\n\
                    Press Ctrl+H for help\n\
                    Press Ctrl+Q to quit",
                    app.adapter().name()
                ))
                .block(messages_block)
            };
            
            f.render_widget(welcome_text, main_chunks[1]);

            // Input box
            let input = Paragraph::new(input_message.as_str())
                .block(Block::default().borders(Borders::ALL).title("Message"));
            f.render_widget(input, chunks[1]);

            // Status bar
            let status = Paragraph::new(format!(
                "Adapter: {} | Status: {:?} | Press Ctrl+Q to quit",
                app.adapter().name(),
                app.adapter().connection_status()
            ))
            .style(Style::default().bg(Color::Blue).fg(Color::White));
            f.render_widget(status, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let action = keyboard_handler.handle_key(key);
                
                match action {
                    Action::Quit => break,
                    Action::NextChat => {
                        if !chats.is_empty() {
                            selected_chat = (selected_chat + 1) % chats.len();
                        }
                    }
                    Action::PrevChat => {
                        if !chats.is_empty() {
                            selected_chat = if selected_chat == 0 {
                                chats.len() - 1
                            } else {
                                selected_chat - 1
                            };
                        }
                    }
                    Action::SendMessage => {
                        if !input_message.is_empty() && selected_chat < chats.len() {
                            let content = cli_chat_rs::MessageContent::Text(input_message.clone());
                            let _ = app.adapter_mut().send_message(&chats[selected_chat].id, content).await;
                            input_message.clear();
                        }
                    }
                    _ => {}
                }

                // Handle text input
                if let KeyCode::Char(c) = key.code {
                    if !key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        input_message.push(c);
                    } else if c == 'h' || c == 'H' {
                        show_help = !show_help;
                    }
                } else if let KeyCode::Backspace = key.code {
                    input_message.pop();
                }
            }
        }
    }

    Ok(())
}

