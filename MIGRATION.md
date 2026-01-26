# Migration from QML to Iced

## What Changed

**Lectern** has been migrated from QML/Qt to **Iced** (pure Rust GUI framework).

### Why?

- ✅ **No more segfaults** - Rust's type system prevents memory safety issues
- ✅ **No FFI bridge** - No need to convert between C++/QML and Rust types
- ✅ **Single source of truth** - UI state is managed in Rust, not scattered across QML files
- ✅ **Type safety** - The compiler ensures you can't pass invalid data

### Architecture

The app now follows **The Elm Architecture**:

1. **State** (`struct Lectern`) - All data lives here
2. **Messages** (`enum Message`) - User actions become messages
3. **View** (`fn view()`) - UI is a pure function of state

### Key Features Implemented

- ✅ Search for books (title/author or ASIN)
- ✅ Display search results
- ✅ Select a book to populate metadata fields
- ✅ Edit metadata (title, author, series, narrator, description)
- ✅ Tab navigation (Metadata, Cover, Chapters, Convert)

### Files Changed

- `Cargo.toml` - Replaced `qmetaobject` with `iced`
- `src/main.rs` - Complete rewrite using Iced
- `src/services.rs` - Unchanged (reused as-is)
- `qml/` directory - No longer used (can be removed)

### Running

```bash
cargo run
```

### Next Steps

The following tabs are placeholders and need implementation:
- **Cover** - Image loading and editing
- **Chapters** - Chapter list management
- **Convert** - Audio conversion workflow

### Benefits You'll Notice

1. **No more crashes** - Type safety prevents the segfaults you experienced
2. **Faster iteration** - All code in one language (Rust)
3. **Better debugging** - Rust compiler catches errors at compile time
4. **Simpler data flow** - Messages flow one way: User → Message → Update → View
