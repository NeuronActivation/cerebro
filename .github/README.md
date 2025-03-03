# Cerebro

Cerebro is a general-purpose Discord bot with Quality of Life features.

## Features

- **YliProxy**: Converts Ylilauta AV1 videos to H.264 format for proper Discord embedding
  - Includes video list indexer with thumbnails
  - Solves the issue of Discord not supporting AV1 video embeds

*More features are planned*

## Building

### From source

**Prerequisites:**

- **FFMPEG** must be installed and available in your ``PATH``
- **OpenSSL** development libraries

### Using Nix

```bash
# Build the project
nix build

# Or run directly from GitHub
nix run github:NeuronActivation/cerebro
```

### Docker

Currently, Docker images can only be built through Nix:

```bash
# Build the image
nix build .#docker
```

A standalone Dockerfile is planned for future releases.

## Documentation

| Variable       | Description                               | Example                   |
|----------------|-------------------------------------------|---------------------------|
| DISCORD_TOKEN  | Discord bot authentication token          | *Required*                |
| WEBSERVER_HOST | Host address for the web server           | ```127.0.0.1```           |
| WEBSERVER_PORT | Port for the web server                   | ```8080```                |
| PUBLIC_URL     | Public URL for accessing converted videos | ```https://example.com``` |
| RUST_LOG       | Controls logging level                    | ```info```                |
| DATA_PATH      | Path to store data files                  | ```./data```              |

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
