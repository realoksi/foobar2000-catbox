# foobar2000-catbox

## Description

Uploads cover art to <https://catbox.moe/> using their API and prints the resulting link to the uploaded image.
This application is meant to be invoked by [this fork of foo_discord_rich by s0hv](https://github.com/s0hv/foo_discord_rich).

## Application flow

Here's a small flowchart describing the flow of the application.

```mermaid
flowchart TD
    A(Application Starts)-->
    B(Get text from\nstandard input)-->
    C(File exists?)-.->|No|D(Failure)
    C-->|Yes|E(Read using image::io)
    E-->F(Resize to 512px by 512px)
    F-->G(Write JPG\ninto memory buffer)
    G-->H(Construct and\nsend POST)
    H-.->|Not Ok|I(Failure)
    H-->|Ok|J(Print resulting URL\nto standard output)
```

## TODO

- [x] make a really cool flowchart
- [ ] automatic compression/downscaling
