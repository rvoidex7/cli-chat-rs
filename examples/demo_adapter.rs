// Demo adapter with mock data for testing UI functionality
use async_trait::async_trait;
use chrono::Utc;
use cli_chat_rs::{
    AdapterResult, Chat, ChatId, ConnectionStatus, Contact, ContactId, Message, MessageContent,
    MessageId, MessageStatus, MessagingAdapter,
};
use tokio::sync::mpsc;

/// Demo adapter with realistic mock data for UI testing
pub struct DemoAdapter {
    status: ConnectionStatus,
    chats: Vec<Chat>,
    messages: Vec<(ChatId, Vec<Message>)>,
    contacts: Vec<Contact>,
}

impl DemoAdapter {
    pub fn new() -> Self {
        let mut adapter = Self {
            status: ConnectionStatus::Disconnected,
            chats: Vec::new(),
            messages: Vec::new(),
            contacts: Vec::new(),
        };

        adapter.init_mock_data();
        adapter
    }

    fn init_mock_data(&mut self) {
        // Create mock contacts
        let alice = Contact {
            id: ContactId::from("contact-1"),
            name: "Alice Johnson".to_string(),
            phone: Some("+1234567890".to_string()),
            avatar_url: None,
            status: Some("Online".to_string()),
            is_online: true,
        };

        let bob = Contact {
            id: ContactId::from("contact-2"),
            name: "Bob Smith".to_string(),
            phone: Some("+0987654321".to_string()),
            avatar_url: None,
            status: Some("Offline".to_string()),
            is_online: false,
        };

        let group_admin = Contact {
            id: ContactId::from("contact-3"),
            name: "Carol Davis".to_string(),
            phone: None,
            avatar_url: None,
            status: Some("Available".to_string()),
            is_online: true,
        };

        self.contacts = vec![alice.clone(), bob.clone(), group_admin.clone()];

        let now = Utc::now();

        // Create mock messages for different chats
        let alice_last_msg = Message {
            id: MessageId::from("msg-1-4"),
            chat_id: ChatId::from("chat-1"),
            sender_id: ContactId::from("contact-1"),
            content: MessageContent::Text("Hey! How are you doing?".to_string()),
            timestamp: now - chrono::Duration::minutes(5),
            is_from_me: false,
            status: MessageStatus::Read,
        };

        let bob_last_msg = Message {
            id: MessageId::from("msg-2-3"),
            chat_id: ChatId::from("chat-2"),
            sender_id: ContactId::from("contact-2"),
            content: MessageContent::Text("See you tomorrow!".to_string()),
            timestamp: now - chrono::Duration::hours(1),
            is_from_me: false,
            status: MessageStatus::Read,
        };

        let group_last_msg = Message {
            id: MessageId::from("msg-3-3"),
            chat_id: ChatId::from("chat-3"),
            sender_id: ContactId::from("contact-2"),
            content: MessageContent::Text("Cargo build completed successfully!".to_string()),
            timestamp: now - chrono::Duration::minutes(30),
            is_from_me: false,
            status: MessageStatus::Delivered,
        };

        let support_last_msg = Message {
            id: MessageId::from("msg-4-1"),
            chat_id: ChatId::from("chat-4"),
            sender_id: ContactId::from("contact-3"),
            content: MessageContent::Text("New ticket created: #12345".to_string()),
            timestamp: now - chrono::Duration::hours(3),
            is_from_me: false,
            status: MessageStatus::Sent,
        };

        // Create mock chats
        let chat1 = Chat {
            id: ChatId::from("chat-1"),
            name: "Alice Johnson".to_string(),
            is_group: false,
            participants: vec![alice.id.clone()],
            last_message: Some(alice_last_msg),
            unread_count: 2,
        };

        let chat2 = Chat {
            id: ChatId::from("chat-2"),
            name: "Bob Smith".to_string(),
            is_group: false,
            participants: vec![bob.id.clone()],
            last_message: Some(bob_last_msg),
            unread_count: 0,
        };

        let chat3 = Chat {
            id: ChatId::from("chat-3"),
            name: "Rust Developers Group".to_string(),
            is_group: true,
            participants: vec![alice.id.clone(), bob.id.clone(), group_admin.id.clone()],
            last_message: Some(group_last_msg),
            unread_count: 5,
        };

        let chat4 = Chat {
            id: ChatId::from("chat-4"),
            name: "Support Team".to_string(),
            is_group: true,
            participants: vec![group_admin.id.clone()],
            last_message: Some(support_last_msg),
            unread_count: 1,
        };

        self.chats = vec![chat1.clone(), chat2.clone(), chat3.clone(), chat4.clone()];

        // Create mock messages for each chat
        let alice_messages = vec![
            Message {
                id: MessageId::from("msg-1-1"),
                chat_id: chat1.id.clone(),
                sender_id: ContactId::from("contact-1"),
                content: MessageContent::Text("Hi there! 👋".to_string()),
                timestamp: now - chrono::Duration::hours(2),
                is_from_me: false,
                status: MessageStatus::Read,
            },
            Message {
                id: MessageId::from("msg-1-2"),
                chat_id: chat1.id.clone(),
                sender_id: ContactId::from("current-user"),
                content: MessageContent::Text("Hey Alice! Good to hear from you!".to_string()),
                timestamp: now - chrono::Duration::minutes(30),
                is_from_me: true,
                status: MessageStatus::Read,
            },
            Message {
                id: MessageId::from("msg-1-3"),
                chat_id: chat1.id.clone(),
                sender_id: ContactId::from("contact-1"),
                content: MessageContent::Text("How are you doing? Long time no see!".to_string()),
                timestamp: now - chrono::Duration::minutes(25),
                is_from_me: false,
                status: MessageStatus::Read,
            },
            Message {
                id: MessageId::from("msg-1-4"),
                chat_id: chat1.id.clone(),
                sender_id: ContactId::from("contact-1"),
                content: MessageContent::Text("Hey! How are you doing?".to_string()),
                timestamp: now - chrono::Duration::minutes(5),
                is_from_me: false,
                status: MessageStatus::Delivered,
            },
        ];

        let bob_messages = vec![
            Message {
                id: MessageId::from("msg-2-1"),
                chat_id: chat2.id.clone(),
                sender_id: ContactId::from("contact-2"),
                content: MessageContent::Text("Meeting at 3pm tomorrow?".to_string()),
                timestamp: now - chrono::Duration::hours(2),
                is_from_me: false,
                status: MessageStatus::Read,
            },
            Message {
                id: MessageId::from("msg-2-2"),
                chat_id: chat2.id.clone(),
                sender_id: ContactId::from("current-user"),
                content: MessageContent::Text("Sounds good! I'll be there.".to_string()),
                timestamp: now - chrono::Duration::minutes(90),
                is_from_me: true,
                status: MessageStatus::Read,
            },
            Message {
                id: MessageId::from("msg-2-3"),
                chat_id: chat2.id.clone(),
                sender_id: ContactId::from("contact-2"),
                content: MessageContent::Text("See you tomorrow!".to_string()),
                timestamp: now - chrono::Duration::hours(1),
                is_from_me: false,
                status: MessageStatus::Read,
            },
        ];

        let group_messages = vec![
            Message {
                id: MessageId::from("msg-3-1"),
                chat_id: chat3.id.clone(),
                sender_id: ContactId::from("contact-1"),
                content: MessageContent::Text(
                    "Anyone working on an interesting Rust project?".to_string(),
                ),
                timestamp: now - chrono::Duration::minutes(45),
                is_from_me: false,
                status: MessageStatus::Read,
            },
            Message {
                id: MessageId::from("msg-3-2"),
                chat_id: chat3.id.clone(),
                sender_id: ContactId::from("contact-3"),
                content: MessageContent::Text(
                    "Yes! I'm building a CLI chat library 🦀".to_string(),
                ),
                timestamp: now - chrono::Duration::minutes(40),
                is_from_me: false,
                status: MessageStatus::Read,
            },
            Message {
                id: MessageId::from("msg-3-3"),
                chat_id: chat3.id.clone(),
                sender_id: ContactId::from("contact-2"),
                content: MessageContent::Text("Cargo build completed successfully!".to_string()),
                timestamp: now - chrono::Duration::minutes(30),
                is_from_me: false,
                status: MessageStatus::Delivered,
            },
        ];

        let support_messages = vec![Message {
            id: MessageId::from("msg-4-1"),
            chat_id: chat4.id.clone(),
            sender_id: ContactId::from("contact-3"),
            content: MessageContent::Text("New ticket created: #12345".to_string()),
            timestamp: now - chrono::Duration::hours(3),
            is_from_me: false,
            status: MessageStatus::Sent,
        }];

        self.messages = vec![
            (chat1.id, alice_messages),
            (chat2.id, bob_messages),
            (chat3.id, group_messages),
            (chat4.id, support_messages),
        ];
    }
}

#[async_trait]
impl MessagingAdapter for DemoAdapter {
    fn name(&self) -> &str {
        "Demo Adapter (Mock Data)"
    }

    async fn connect(&mut self) -> AdapterResult<()> {
        println!("Connecting to demo service with mock data...");
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> AdapterResult<()> {
        self.status = ConnectionStatus::Disconnected;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.status
    }

    async fn get_chats(&self) -> AdapterResult<Vec<Chat>> {
        Ok(self.chats.clone())
    }

    async fn get_messages(&self, chat_id: &ChatId, _limit: usize) -> AdapterResult<Vec<Message>> {
        if let Some((_, messages)) = self.messages.iter().find(|(id, _)| id == chat_id) {
            Ok(messages.clone())
        } else {
            Ok(vec![])
        }
    }

    async fn send_message(
        &mut self,
        chat_id: &ChatId,
        content: MessageContent,
    ) -> AdapterResult<Message> {
        let new_message = Message {
            id: MessageId::from(&format!(
                "msg-{}",
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            )),
            chat_id: chat_id.clone(),
            sender_id: ContactId::from("current-user"),
            content,
            timestamp: Utc::now(),
            is_from_me: true,
            status: MessageStatus::Sent,
        };

        // Update chat's last message
        if let Some(chat) = self.chats.iter_mut().find(|c| &c.id == chat_id) {
            chat.last_message = Some(new_message.clone());
        }

        // Add to messages
        if let Some((_, messages)) = self.messages.iter_mut().find(|(id, _)| id == chat_id) {
            messages.push(new_message.clone());
        } else {
            self.messages
                .push((chat_id.clone(), vec![new_message.clone()]));
        }

        Ok(new_message)
    }

    async fn mark_as_read(
        &mut self,
        chat_id: &ChatId,
        message_id: &MessageId,
    ) -> AdapterResult<()> {
        if let Some((_, messages)) = self.messages.iter_mut().find(|(id, _)| id == chat_id) {
            for msg in messages.iter_mut() {
                if &msg.id == message_id {
                    msg.status = MessageStatus::Read;
                    break;
                }
            }
        }

        // Update unread count
        if let Some(chat) = self.chats.iter_mut().find(|c| &c.id == chat_id) {
            chat.unread_count = 0;
        }

        Ok(())
    }

    async fn get_contact(&self, contact_id: &ContactId) -> AdapterResult<Contact> {
        self.contacts
            .iter()
            .find(|c| &c.id == contact_id)
            .cloned()
            .ok_or("Contact not found".into())
    }

    async fn get_contacts(&self) -> AdapterResult<Vec<Contact>> {
        Ok(self.contacts.clone())
    }

    async fn subscribe_to_messages(&mut self) -> AdapterResult<mpsc::Receiver<Message>> {
        let (_tx, rx) = mpsc::channel(100);
        Ok(rx)
    }

    async fn search(&self, query: &str) -> AdapterResult<Vec<Chat>> {
        let query = query.to_lowercase();
        Ok(self
            .chats
            .iter()
            .filter(|chat| {
                chat.name.to_lowercase().contains(&query)
                    || chat
                        .last_message
                        .as_ref()
                        .map(|msg| {
                            if let MessageContent::Text(text) = &msg.content {
                                text.to_lowercase().contains(&query)
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false)
            })
            .cloned()
            .collect())
    }
}

#[tokio::main]
async fn main() {
    use cli_chat_rs::{Config, MessengerApp};

    let config = Config::default();
    let adapter = Box::new(DemoAdapter::new());
    let mut app = MessengerApp::new(config, adapter);

    println!("Starting Demo Application");
    println!("This demo uses mock data to showcase the UI functionality");
    println!("==========================");

    if let Err(e) = app.adapter_mut().connect().await {
        eprintln!("Connection error: {}", e);
        return;
    }
    println!("✓ Connected to demo adapter");

    let chats = app.adapter().get_chats().await.unwrap_or_default();
    println!("✓ Found {} chat(s)", chats.len());

    for (i, chat) in chats.iter().enumerate() {
        let messages = app
            .adapter()
            .get_messages(&chat.id, 10)
            .await
            .unwrap_or_default();
        println!(
            "  {}. {} ({} messages, {} unread)",
            i + 1,
            chat.name,
            messages.len(),
            chat.unread_count
        );
    }

    if let Err(e) = app.adapter_mut().disconnect().await {
        eprintln!("Disconnect error: {}", e);
        return;
    }
    println!("✓ Disconnected");
}
