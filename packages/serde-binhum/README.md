Implement `serde::{Serialize, Deserialize}` on a type with different behavior for human-readable and non-human-readable (binary) formats.

# Motivation

Serde has the concept of human-readable and non-human-readable de/serializers. Human-readable ones, like JSON, are - well - readable by humans, and are usually verbose and self-describing. Human-readable format deserializers implement `deserialize_any`, which lets Serde do more complicated things like internally tagged enums and `serde(flatten)`. However, binary formats, like Postcard, cannot implement `deserialize_any`, and attempting to deserialize a value using `serde(flatten)` using one of these will fail.

```rust
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
enum MyEnum {
    Foo,
    Bar {
        x: i32,
        #[serde(flatten)]
        data: BarData,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
struct BarData {
    y: i32,
}

let error = postcard::to_allocvec(&MyEnum::Bar {
    x: 1,
    data: BarData { y: 2 },
})
.unwrap_err();
```

To fix this, we can generate two `De/Serialize` implementations: one for human-readable, and another for binary formats. Then, the `De/Serialize` impl on the actual type will delegate to one of those two:

```rust
enum MyEnum {
    Foo,
    Bar {
        x: i32,
        data: BarData,
    },
}

// note: this type doesn't use any features which require `deserialize_any`,
// so it can safely derive `De/Serialize` as normal
#[derive(serde::Serialize, serde::Deserialize)]
struct BarData {
    y: i32,
}

const _: () = {
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(remote = "MyEnum", tag = "kind")]
    enum MyEnumHumanProxy {
        Foo,
        Bar {
            x: i32,
            #[serde(flatten)]
            data: BarData,
        },
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(remote = "MyEnum")]
    enum MyEnumBinaryProxy {
        Foo,
        Bar {
            x: i32,
            data: BarData,
        },
    }

    impl serde::Serialize for MyEnum {
        fn serialize<S: serde::Serializer>(
            &self,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        {
            if serializer.is_human_readable() {
                MyEnumHumanProxy::serialize(self, serializer)
            } else {
                MyEnumBinaryProxy::serialize(self, serializer)
            }
        }
    }

    impl<'de> serde::Deserialize<'de> for MyEnum {
        fn deserialize<D: serde::Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Self, D::Error>
        {
            if deserializer.is_human_readable() {
                MyEnumHumanProxy::deserialize(deserializer)
            } else {
                MyEnumBinaryProxy::deserialize(deserializer)
            }
        }
    }
};
```

# Usage

Add `#[serde_binhum::serde_binhum]` on a type, and remove `#[derive(serde::Serialize, serde::Deserialize)]`.

Ordinary `#[serde(...)]` attributes describe the human-readable representation. `serde-binhum` removes attributes such as `tag` and `flatten` from the generated binary proxy:

```rust
#[serde_binhum::serde_binhum]
#[derive(Debug, PartialEq)]
#[serde(tag = "kind")]
enum MyEnum {
    Foo,
    Bar {
        x: i32,
        #[serde(flatten)]
        data: BarData,
    },
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct BarData {
    y: i32,
}

let value = MyEnum::Bar {
    x: 1,
    data: BarData { y: 2 },
};

let json = serde_json::to_value(&value).unwrap();
assert_eq!(
    json,
    serde_json::json!({
        "kind": "Bar",
        "x": 1,
        "y": 2,
    }),
);

let bytes = postcard::to_allocvec(&value).unwrap();
let decoded = postcard::from_bytes::<MyEnum>(&bytes).unwrap();
assert_eq!(decoded, value);
```

Representation-specific Serde options can be added with `human(...)` and `binary(...)`:

```rust
#[serde_binhum::serde_binhum]
struct Value {
    #[serde_binhum(human(flatten), binary(with = "binary_data"))]
    data: Data,
}
```

Use `#[serde_binhum::serde_binhum(schema)]` to forward `utoipa::PartialSchema` and `utoipa::ToSchema` to the human-readable proxy.
