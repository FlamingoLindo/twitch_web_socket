# Twitch WebSocket Chat Client

A high-performance, asynchronous Twitch chat client built in Rust that connects to Twitch IRC via WebSocket. This application enables real-time chat interaction with Twitch streams, featuring message parsing, badge support, and bidirectional communication.

## Features

- Asynchronous WebSocket connection to Twitch IRC
- Real-time chat message reading and sending
- IRC message parsing with tag support
- Twitch badge integration with URL resolution
- Color-coded console output for enhanced readability
- OAuth token validation
- Configurable settings via TOML

## DEMO

<https://github.com/user-attachments/assets/ff6416fd-4774-4f84-8c1e-25657ff79834>

## Architecture

The project follows a modular architecture with clear separation of concerns:

```bash
src/
├── main.rs              # Application entry point
├── lib.rs               # Public API exports
├── message_parser.rs    # IRC message parsing logic
├── assets/              # Static resources
│   └── badges.rs        # Badge data structures and parsing
└── twitch/              # Core Twitch functionality
    ├── connection.rs    # WebSocket connection management
    ├── message/         # Message handling
    │   ├── reader.rs    # Chat message reading
    │   └── writer.rs    # Chat message sending
    └── user/            # User authentication and settings
        ├── auth.rs      # OAuth token validation
        └── settings.rs  # Configuration structures
```

## How It Works

### 1. Connection Establishment

The application establishes a WebSocket connection to Twitch's IRC server using TLS encryption:

```rust
pub async fn create_twitch_connection() -> TwitchConnection {
    let settings = load_acc().await;
    let url = settings.url;
    
    println!("{}", "Connecting to websocket...".yellow());
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect!");
    println!("{}", "Connected!".green());
    
    let (write, read) = ws_stream.split();
    
    TwitchConnection {
        write,
        read,
        oauth_token: settings.token,
        nickname: settings.nickname,
        channel: "pleaseendmyloniness".to_string(),
    }
}
```

The connection is split into separate read and write streams for concurrent message handling.

### 2. Authentication

The client authenticates with Twitch IRC using OAuth and capabilities negotiation:

```rust
pub async fn connect_to_twitch(conn: &mut TwitchConnection) {
    // Request capabilities for tags and commands
    conn.write
        .send(Message::Text(
            "CAP REQ :twitch.tv/tags twitch.tv/commands".into(),
        ))
        .await
        .expect("Failed to request capabilities");

    // Authenticate with OAuth token
    conn.write
        .send(Message::Text(format!("PASS {}", conn.oauth_token).into()))
        .await
        .expect("Failed to send PASS");

    // Set nickname
    conn.write
        .send(Message::Text(format!("NICK {}", conn.nickname).into()))
        .await
        .expect("Failed to send NICK");

    // Join specific channel
    conn.write
        .send(Message::Text(format!("JOIN #{}", conn.channel).into()))
        .await
        .expect("Failed to join channel");
}
```

### 3. Message Parsing

IRC messages from Twitch are parsed to extract tags, commands, and message content:

```rust
pub fn parse_twitch_message(raw: &str) -> Option<TwitchMessage> {
    let raw = raw.trim();

    if !raw.starts_with('@') {
        return None;
    }

    let mut parts = raw.split_whitespace();

    // Parse tags (metadata like username, color, badges)
    let tags_str = parts.next()?;
    let tags: HashMap<String, String> = tags_str
        .trim_start_matches('@')
        .split(';')
        .filter_map(|tag| {
            let mut kv = tag.splitn(2, '=');
            Some((kv.next()?.to_string(), kv.next().unwrap_or("").to_string()))
        })
        .collect();

    let prefix = parts.next()?.trim_start_matches(':').to_string();
    let command = parts.next()?.to_string();

    if command != "PRIVMSG" {
        return None;
    }

    let channel = parts.next()?.to_string();

    // Extract message content after the last ':'
    let message = if let Some(colon_pos) = raw.rfind(" :") {
        raw[colon_pos + 2..].to_string()
    } else {
        String::new()
    };

    Some(TwitchMessage {
        tags,
        _prefix: prefix,
        _command: command,
        _channel: channel,
        message,
    })
}
```

The parser extracts structured data from raw IRC messages, including user metadata and message content.

### 4. Reading Chat Messages

The application continuously listens for incoming messages and displays them with formatting:

```rust
pub async fn read_chat(
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    twitch_badges: Vec<TwitchBadge>,
) {
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                for line in text.lines() {
                    if let Some(msg) = parse_twitch_message(line) {
                        let default_color = "#555555".to_string();
                        let user_color = msg.tags.get("color").unwrap_or(&default_color);
                        let default_name = "Guest".to_string();

                        // Display username with color and message
                        println!(
                            "{}: {}",
                            msg.tags
                                .get("display-name")
                                .unwrap_or(&default_name)
                                .color(user_color.as_str()),
                            msg.message
                        );

                        // Process user badges
                        let user_badges = user_badges(&msg.tags);
                        for user_badge in &user_badges {
                            if let Some(twitch_badge) =
                                twitch_badges.iter().find(|tb| &tb.name == user_badge)
                            {
                                let badge_url = twitch_badge.url.replace("{SIZE}", "3");
                                println!("Badge URL: {}", badge_url);
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                eprintln!("{}", format!("Error: {}", e).red());
                break;
            }
            _ => {}
        }
    }
}
```

### 5. Sending Messages

Users can send messages to the chat through standard input:

```rust
pub async fn send_message(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    channel: &str,
) {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    println!("Send message in the chat (type and press Enter):");

    while let Ok(Some(line)) = reader.next_line().await {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Err(e) = write
            .send(Message::Text(
                format!("PRIVMSG #{} :{}", channel, trimmed).into(),
            ))
            .await
        {
            eprintln!("Failed to send message: {}", e);
            break;
        }
    }
}
```

### 6. Badge System

Twitch badges are loaded from a JSON file and matched against user badge tags:

```rust
#[derive(Deserialize, Debug)]
pub struct TwitchBadge {
    pub name: String,
    pub url: String,
}

pub fn parse_badges_from_str(json_str: &str) -> Result<Vec<TwitchBadge>, Box<dyn Error>> {
    let badges = serde_json::from_str(json_str)?;
    Ok(badges)
}
```

Badges are extracted from message tags and resolved to their corresponding image URLs:

```rust
pub fn user_badges(tags: &HashMap<String, String>) -> Vec<String> {
    tags.get("badges")
        .unwrap_or(&String::new())
        .split(',')
        .filter_map(|badge| badge.split_once('/').map(|(name, _)| format!("{}", name)))
        .collect()
}
```

### 7. OAuth Token Validation

The application validates OAuth tokens before establishing connections:

```rust
pub async fn validate_token(twitch_token: String) -> Result<String, reqwest::Error> {
    let formatted = twitch_token.trim_start_matches("oauth:");

    let client = reqwest::Client::new();
    let response = client
        .get("https://id.twitch.tv/oauth2/validate")
        .header("Authorization", format!("OAuth {}", formatted))
        .send()
        .await?;

    if response.status() == StatusCode::UNAUTHORIZED {
        return Err(response.error_for_status().unwrap_err());
    }

    let text = response.text().await?;
    Ok(text)
}
```

### 8. Main Application Flow

The main function orchestrates the entire application lifecycle:

```rust
#[tokio::main]
async fn main() {
    // Load twitch badges from embedded JSON
    let twitch_badges =
        parse_badges_from_str(TWITCH_BADGES_JSON).expect("Failed to parse twitch badges json");

    // Create and establish connection
    let mut conn = create_twitch_connection().await;
    connect_to_twitch(&mut conn).await;

    let TwitchConnection {
        mut write,
        mut read,
        channel,
        ..
    } = conn;

    // Run message reading and sending concurrently
    tokio::join!(
        send_message(&mut write, &channel),
        read_chat(&mut read, twitch_badges)
    );
}
```

## Configuration

Create a `settings.toml` file with your Twitch credentials:

```toml
nickname = "your_twitch_username"
token = "oauth:your_oauth_token"
url = "wss://irc-ws.chat.twitch.tv:443"
```

## Dependencies

The project leverages several high-quality Rust crates:

- **tokio**: Asynchronous runtime for concurrent operations
- **tokio-tungstenite**: WebSocket client implementation
- **futures-util**: Stream and sink utilities for async programming
- **serde/serde_json**: Serialization and deserialization
- **reqwest**: HTTP client for OAuth validation
- **colored**: Terminal color output
- **chrono**: Date and time functionality
- **config**: Configuration file management

## Building and Running

Build the project:

```bash
cargo build --release
```

Run the application:

```bash
cargo run
```

## Technical Highlights

- **Asynchronous Design**: Built on Tokio for efficient concurrent I/O operations
- **Type Safety**: Leverages Rust's type system for compile-time guarantees
- **Error Handling**: Robust error handling with Result types
- **Modular Architecture**: Clean separation between connection, parsing, and message handling
- **WebSocket with TLS**: Secure communication with Twitch servers
- **Zero-Copy Operations**: Efficient string parsing with minimal allocations
