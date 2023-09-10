# toy-async-server

![Rust](https://img.shields.io/badge/Rust-1.57.0-orange.svg)
![License](https://img.shields.io/badge/License-MIT-green.svg)

**toy-async-server** is a concurrent web server built using asynchronous Rust. This project demonstrates building a basic web server from scratch, handling multiple client connections, and responding to HTTP requests.

## Features

- Asynchronous I/O handling using Rust's async/await.
- Concurrently handles multiple client connections.
- Basic HTTP request parsing and response generation.

## Prerequisites

Before running the server, ensure you have the following installed:

- Rust (version 1.57.0 or higher)

## Important Note

**toy-async-server** currently supports Linux-based systems only due to its use of the epoll mechanism for asynchronous I/O. It may not work on other operating systems.


## Getting Started

1. Clone the repository:

   ```sh
   git clone https://github.com/shivam412/toy-async-server.git
   cd toy-async-server
   ```

2. Build the project:
    ```sh
    cargo build
    ```

3. Run the server:
    ```sh
    cargo run
    ```
The server will start listening on a specified address and port (e.g., 0.0.0.0:8000).

Access the server:

Open a web browser or use a tool like curl to make HTTP requests to the server.

## Usage
Modify the server's code in main.rs to add your own application logic.

## Project Structure
The project's source code is organized as follows:

- `src/core/`: Contains core modules, including error handling and result types.
- `src/net/`: Networking-related modules for handling TCP connections.
- `src/runtime/`: Modules related to runtime and task management.
- `src/main.rs`: The main entry point of the server.

Feel free to explore and customize the code to suit your needs.

## Contributing
Contributions are welcome! Feel free to open issues or pull requests to improve this project.



## License
This project is licensed under the MIT License - see the LICENSE file for details.




