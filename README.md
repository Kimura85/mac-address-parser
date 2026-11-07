# mac-address-parser

Parses MAC addresses in colon, hyphen, and dot notations into a fixed six-byte representation.

```rust
use mac_address_parser::MacAddress;

let mac = MacAddress::parse("aa:bb:cc:dd:ee:ff").unwrap();
assert_eq!(mac.octets(), [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);

// The same address in Cisco dot notation compares equal.
let cisco = MacAddress::parse("aabb.ccdd.eeff").unwrap();
assert_eq!(mac, cisco);
```

## Why this exists

MAC addresses show up in three common notations depending on the tool that
produced them: `ip link` prints colon-separated, Windows `getmac` prints
hyphen-separated, and Cisco gear prints dot-separated groups of four. Code that
stores the original string therefore treats `aa:bb:cc:dd:ee:ff` and
`AA-BB-CC-DD-EE-FF` as different values even though they name the same device.

This library normalises all three notations into a single `[u8; 6]` at parse
time. The trade-off is that the original notation is not preserved: if you need
to echo the input back exactly as the user typed it, keep the string yourself.
Canonical equality is the property that matters for lookup tables, allow-lists,
and deduplication, so that is what the type optimises for.

## The awkward edge

Separators must be consistent within a single address. `aa:bb-cc:dd:ee:ff` is
rejected with an `inconsistent separator` error rather than being silently
split on the first separator found. This is deliberate: mixed separators almost
always indicate a typo or a mangled paste, and accepting them would make the
parser's behaviour depend on which separator happens to appear first.

## API

- `MacAddress::parse(&str) -> Result<MacAddress, String>` — parse one of the
  accepted notations. The error is a human-readable message suitable for
  logging or for embedding in a larger error type.
- `MacAddress::octets(&self) -> [u8; 6]` — the six bytes in network order.
- `MacAddress` implements `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, and
  `Hash`, so it can be used as a map key directly.
