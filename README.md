# Crontab files parser

Alpha version

[Documentation][1]

[1]: http://kstep.me/cronparse.rs/cronparse/index.html

Usage:

```toml
[dependencies]
cronparse = "*"
```

```rust
extern crate cronparse;

use cronparse::crontab::UserCrontabEntry;
use cronparse::CrontabFile;

fn main() {
    let mut crontab = CrontabFile::<UserCrontabEntry>::new("/var/spool/cron/kstep");
    for entry in crontab {
        println!("{:?}", entry);
    }
}
```
