Toy libiary for reading `/etc/fstab`

### How to use

Open `/etc/fstab`:

```rust
open_fstab(None)
```

Open `/home/user/fstab`:

```rust
open_fstab("/home/user/fstab")
```
Return a `Vec<Fstab>` when successes, and a `Error` when fails.
