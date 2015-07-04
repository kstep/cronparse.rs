# Crontab files parser

Alpha version

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
