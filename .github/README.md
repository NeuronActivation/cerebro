# Cerebro

Cerebro is a general-purpose Discord bot with Quality of Life features.

## Features

- **YliProxy**: Converts Ylilauta AV1 videos to H.264 format (by default) for proper Discord embedding
  - Includes video list indexer with thumbnails
  - Solves the issue of Discord not supporting AV1 video embeds

*More features are planned*

## Building

### From source

**Prerequisites:**

- **FFMPEG** must be installed and available in your ``PATH`` \
  or pointed to with the `FFMPEG_BIN` environment variable.
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

| Variable       | Description                                                        | Example                   |
|----------------|--------------------------------------------------------------------|---------------------------|
| DISCORD_TOKEN  | Discord bot authentication token                                   | *Required*                |
| WEBSERVER_HOST | Host address for the web server                                    | ```127.0.0.1```           |
| WEBSERVER_PORT | Port for the web server                                            | ```8080```                |
| PUBLIC_URL     | Public URL for accessing converted videos                          | ```https://example.com``` |
| RUST_LOG       | Controls logging level                                             | ```info```                |
| DATA_PATH      | Path to store data files                                           | ```./data```              |
| FFMPEG_BIN     | Name or path to the FFMPEG binary                                  | ```ffmpeg-static-6```     |
| FFMPEG_ARGS    | FFMPEG arguments template with `$INPUT` and `$OUTPUT` placeholders | ```-y -i $INPUT -vaapi_device /dev/dri/renderD128 -vf format=nv12,hwupload -c:v h264_vaapi -c:a copy $OUTPUT``` |

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
