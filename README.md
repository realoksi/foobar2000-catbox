# foobar2000-catbox

## what the thing does
Uploads cover art to https://catbox.moe/ using their API and prints the resulting link to the uploaded image.
This application is meant to be invoked by [this fork of foo_discord_rich by s0hv](https://github.com/s0hv/foo_discord_rich).

## how the thing does
Here's a small flowchart describing the flow of the application.
```mermaid
flowchart LR
    classDef bad stroke:#f00
    classDef good stroke:#0f0
    D:::bad
    G:::bad
    K:::bad
    M:::good
    A((Start)) --> B[/Read file path\nfrom standard input/]
    B --> C{Does this file\npath exist?}
    C -->|No| D((End))
    C -->|Yes| E[/Read file at path\ninto buffer/]
    E --> F{Can we guess the\nimage format?}
    F -->|No| G((End))
    F -->|Yes| H[Build POST form]
    H --> I[/Send POST form/]
    I --> J{Did it return\nsuccessfully?}
    J -->|No| K((End))
    J -->|Yes| L[/Print resulting URL\]
    L --> M((End))
```

## TODO
- [x] implement the thing
- [x] make a really cool flowchart
- [ ] guide to installation and usage
- [ ] automatic compression/downscaling