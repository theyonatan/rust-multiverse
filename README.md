Rust Multiverse:

a task (tokio) based project where you, the user, spawns multiple universes that fight, friend, communicate with each other.
covers different intermediate rust concepts.

all universes communicate through a server, the "Supervisor" which receives messages and broadcast + Logs the relevant results.

Logger is fully static, the app is pretty dynamic, so I can convert it to a web server at any time.

# Web Server Version
this version does not use the terminal, instead uses a laggy web server.
It was entirely vibe coded using Grok, Gemini and ChatGPT. the code ie pretty ugly but I feel very confident creating my own web servers in rust.
