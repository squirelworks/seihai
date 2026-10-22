## Online

```bash
cargo run -p seihai -- --assets /path/to/assets \
  --online 127.0.0.1:14802 \
  --account player --password secret \
  --version 2.05
```

Connects to a gserver, logs in, and updates the view from the server. Other players are drawn if body/head sheets are present. Arrow keys send walk frames. Escape closes the window.