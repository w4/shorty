# shorty

Upload files or shorten URLs using this simple little tool, writes
to an S3 (or Minio) bucket, and returns a link to them.

### Usage

Simply move `config.toml` to `~/.config/shorty.toml` and configure
accordingly.

```
$ up ./my-file.png
https://example.com/u/fef5ad88-2bf2-4a83-b4ed-221191997390.toml

$ short https://google.com/
https://example.com/s/ebasiaveri
```