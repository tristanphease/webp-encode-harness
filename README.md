# WebP Image Encoding Harness

Simple repo for testing encoding for WebP images.

Mainly for lossy images currently, tries to measure both accuracy compared to the original PNG image and file size. We want to optimise for higher accuracy and lower file size. Encoding speed is something to consider but not currently a high priority (as long as it's not abyssmal).
Want to improve the encoding used as part of the Rust [image-webp crate](https://github.com/image-rs/image-webp) so it's more comparable to the C [libwebp](https://github.com/webmproject/libwebp) library.
