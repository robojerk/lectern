# Refactor Plan: Split `src/ui/mod.rs` into Modular Handlers

## Current State

- **File size**: ~1845 lines
- **Main components**:
  - `Message` enum: ~90 variants (lines 52-141)
  - `Lectern` struct: ~60 fields (lines 143-220)
  - `Default` impl: ~60 lines
  - `Drop` impl: ~40 lines
  - `Application::update()`: ~1400+ lines (the main problem)
  - `Application::view()`: ~30 lines
  - `Application::subscription()`: ~30 lines

## Problem

The `update()` function is a massive `match` statement handling all 90+ message variants. This makes the code:
- Hard to navigate and maintain
- Difficult to test individual features
- Prone to merge conflicts
- Hard to understand the flow of each feature area

## Proposed Structure

```
src/ui/
├── mod.rs                    # Main module: Message enum, Lectern struct, Application impl (orchestration only)
├── handlers/                 # Message handlers organized by feature
│   ├── mod.rs               # Re-export all handlers, helper traits
│   ├── search.rs            # Search-related message handlers
│   ├── metadata.rs          # Metadata editing handlers
│   ├── cover.rs             # Cover management handlers
│   ├── chapters.rs          # Chapter management handlers
│   ├── file.rs              # File selection/parsing handlers
│   ├── settings.rs          # Settings handlers
│   ├── convert.rs           # Conversion handlers
│   └── navigation.rs        # View switching handlers
├── state/                    # State management modules
│   ├── mod.rs               # Re-export state modules
│   ├── search_state.rs      # Search-related state
│   ├── metadata_state.rs    # Metadata editing state
│   ├── cover_state.rs       # Cover state
│   ├── chapter_state.rs     # Chapter state (including playback)
│   └── file_state.rs        # File selection state
├── colors.rs                 # (unchanged)
├── cover_search.rs          # (unchanged)
├── helpers.rs               # (unchanged)
└── views/                   # (unchanged)
    ├── mod.rs
    ├── search.rs
    ├── metadata.rs
    ├── cover.rs
    ├── chapters.rs
    ├── convert.rs
    └── settings.rs
```

## Implementation Strategy

### Phase 1: Extract State Modules

**Goal**: Separate state fields into logical modules while keeping them in the main `Lectern` struct.

**Steps**:
1. Create `src/ui/state/mod.rs` with re-exports
2. Create `src/ui/state/search_state.rs`:
   ```rust
   pub struct SearchState {
       pub query: String,
       pub author: String,
       pub by_asin: bool,
       pub is_searching: bool,
       pub results: Vec<BookMetadata>,
       pub error: Option<String>,
       pub current_page: usize,
       pub results_per_page: usize,
       pub result_covers: HashMap<String, Vec<u8>>,
       pub downloading: Arc<Mutex<Vec<String>>>,
   }
   ```
3. Create similar state modules for:
   - `metadata_state.rs` (editing fields)
   - `cover_state.rs` (cover management)
   - `chapter_state.rs` (chapters + playback)
   - `file_state.rs` (file selection)
4. Update `Lectern` struct to use these state modules:
   ```rust
   pub struct Lectern {
       pub search: SearchState,
       pub metadata: MetadataState,
       pub cover: CoverState,
       pub chapters: ChapterState,
       pub file: FileState,
       // ... other fields
   }
   ```

**Benefits**:
- Clearer organization of related fields
- Easier to see what state belongs to what feature
- Can add helper methods to state structs

### Phase 2: Create Handler Functions (No Trait Needed)

**Goal**: Define handler functions for each feature area.

**Approach**: Use free functions instead of traits - simpler, zero runtime overhead, compile-time dispatch.

**Steps**:
1. Create `src/ui/handlers/mod.rs` with re-exports:
   ```rust
   pub mod search;
   pub mod metadata;
   pub mod cover;
   pub mod chapters;
   pub mod file;
   pub mod settings;
   pub mod convert;
   pub mod navigation;
   
   pub use search::handle_search;
   pub use metadata::handle_metadata;
   pub use cover::handle_cover;
   pub use chapters::handle_chapters;
   pub use file::handle_file;
   pub use settings::handle_settings;
   pub use convert::handle_convert;
   pub use navigation::handle_navigation;
   ```

2. Each handler module exports a single `handle()` function:
   ```rust
   // src/ui/handlers/search.rs
   pub fn handle_search(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
       match message {
           Message::SearchQueryChanged(query) => {
               app.search.query = query;
               Some(Command::none())
           }
           Message::PerformSearch => {
               // ... search logic
               Some(Command::perform(...))
           }
           _ => None, // This handler doesn't handle this message
       }
   }
   ```

### Phase 3: Extract Handlers by Feature

**Goal**: Move message handling logic into separate handler modules.

**Error Handling Strategy**:
- Handlers return `Option<Command<Message>>` - `None` means "not handled by this handler"
- Errors are stored in appropriate state fields (e.g., `app.search.error`, `app.file_parse_error`)
- Async operations return `Command::perform()` with error results wrapped in `Message` variants
- Example: `SearchCompleted(Result<...>)` - error is handled in the message variant

**Borrowing Considerations**:
- Handlers take `&mut Lectern` - Rust's borrow checker ensures no conflicts
- If a handler needs multiple state modules, it accesses them through `app`:
  ```rust
  // Cover handler might need metadata state
  pub fn handle_cover(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
      match message {
          Message::SelectCover(index) => {
              // Can access both cover and metadata state
              let cover_url = app.cover.results[index].url.clone();
              app.metadata.selected_book.cover_url = Some(cover_url);
              Some(Command::none())
          }
          // ...
      }
  }
  ```
- If borrowing conflicts arise, restructure to avoid simultaneous mutable access

**Steps**:
1. **Search Handler** (`src/ui/handlers/search.rs`):
   - Handles: `SearchQueryChanged`, `SearchAuthorChanged`, `PerformSearch`, `SearchCompleted`, `SelectBook`, `NextPage`, `PreviousPage`, `SearchByAsinToggled`
   - Function: `handle_search(app: &mut Lectern, message: Message) -> Option<Command<Message>>`

2. **Metadata Handler** (`src/ui/handlers/metadata.rs`):
   - Handles: All `*Changed` messages for editing fields, `MetadataProviderChanged`
   - Function: `handle_metadata(app: &mut Lectern, message: Message) -> Option<Command<Message>>`

3. **Cover Handler** (`src/ui/handlers/cover.rs`):
   - Handles: `BrowseCoverImage`, `CoverImageSelected`, `SearchCover`, `CoverSearchCompleted`, `SelectCover`, `CoverUrlChanged`, `DownloadCoverImage`, `CoverImageDownloaded`, `SearchCoverImageDownloaded`, `SearchCoverImagesDownloaded`
   - Function: `handle_cover(app: &mut Lectern, message: Message) -> Option<Command<Message>>`
   - May need access to: `app.cover`, `app.metadata` (for setting cover on selected book)

4. **Chapter Handler** (`src/ui/handlers/chapters.rs`):
   - Handles: All `Chapter*` messages including playback
   - Function: `handle_chapters(app: &mut Lectern, message: Message) -> Option<Command<Message>>`
   - May need access to: `app.chapters`, `app.file` (for audio file paths)

5. **File Handler** (`src/ui/handlers/file.rs`):
   - Handles: `BrowseFiles`, `BrowseFolder`, `FileSelected`, `FileDropped`, `FileParsed`
   - Function: `handle_file(app: &mut Lectern, message: Message) -> Option<Command<Message>>`
   - May need access to: `app.file`, `app.metadata` (for populating metadata from file)

6. **Settings Handler** (`src/ui/handlers/settings.rs`):
   - Handles: All settings-related messages
   - Function: `handle_settings(app: &mut Lectern, message: Message) -> Option<Command<Message>>`

7. **Convert Handler** (`src/ui/handlers/convert.rs`):
   - Handles: `StartConversion`, `BrowseOutputPath`, `OutputPathSelected`, `ConversionCompleted`
   - Function: `handle_convert(app: &mut Lectern, message: Message) -> Option<Command<Message>>`
   - May need access to: `app.file`, `app.metadata`, `app.cover`, `app.chapters`

8. **Navigation Handler** (`src/ui/handlers/navigation.rs`):
   - Handles: All `SwitchTo*` messages
   - Function: `handle_navigation(app: &mut Lectern, message: Message) -> Option<Command<Message>>`

### Phase 4: Refactor Main `update()` Function

**Goal**: Make `update()` a simple dispatcher that delegates to handlers.

**Approach**: Use compile-time dispatch with pattern matching - zero runtime overhead, idiomatic Rust.

**Steps**:
1. In `src/ui/mod.rs`, simplify `update()` to dispatch to handlers:
   ```rust
   fn update(&mut self, message: Message) -> Command<Message> {
       // Try each handler in order - first one that returns Some() wins
       if let Some(cmd) = handlers::search::handle_search(self, message.clone()) {
           return cmd;
       }
       if let Some(cmd) = handlers::metadata::handle_metadata(self, message.clone()) {
           return cmd;
       }
       if let Some(cmd) = handlers::cover::handle_cover(self, message.clone()) {
           return cmd;
       }
       if let Some(cmd) = handlers::chapters::handle_chapters(self, message.clone()) {
           return cmd;
       }
       if let Some(cmd) = handlers::file::handle_file(self, message.clone()) {
           return cmd;
       }
       if let Some(cmd) = handlers::settings::handle_settings(self, message.clone()) {
           return cmd;
       }
       if let Some(cmd) = handlers::convert::handle_convert(self, message.clone()) {
           return cmd;
       }
       if let Some(cmd) = handlers::navigation::handle_navigation(self, message.clone()) {
           return cmd;
       }
       
       // Message not handled (shouldn't happen if all messages are covered)
       eprintln!("[WARNING] Unhandled message: {:?}", message);
       Command::none()
   }
   ```

   **Note**: The `message.clone()` calls are necessary because `Message` is consumed by the match. This is acceptable since `Message` implements `Clone` and most variants are small.

   **Alternative** (if cloning is a concern): Use pattern matching to avoid cloning:
   ```rust
   fn update(&mut self, message: Message) -> Command<Message> {
       match message {
           // Search messages
           msg @ (Message::SearchQueryChanged(_) | 
                  Message::SearchAuthorChanged(_) | 
                  Message::PerformSearch | 
                  Message::SearchCompleted(_) |
                  Message::SelectBook(_) |
                  Message::NextPage |
                  Message::PreviousPage |
                  Message::SearchByAsinToggled(_)) => {
               handlers::search::handle_search(self, msg)
                   .unwrap_or_else(|| {
                       eprintln!("[WARNING] Search handler returned None for: {:?}", msg);
                       Command::none()
                   })
           }
           // Metadata messages
           msg @ (Message::TitleChanged(_) | 
                  Message::AuthorChanged(_) | 
                  Message::SubtitleChanged(_) |
                  // ... all metadata messages
                  Message::MetadataProviderChanged(_)) => {
               handlers::metadata::handle_metadata(self, msg)
                   .unwrap_or_else(|| {
                       eprintln!("[WARNING] Metadata handler returned None for: {:?}", msg);
                       Command::none()
                   })
           }
           // ... etc for other handlers
           _ => {
               eprintln!("[WARNING] Unhandled message: {:?}", message);
               Command::none()
           }
       }
   }
   ```

### Phase 5: Extract Message Enum Variants (Deferred - Future Improvement)

**Goal**: Split the large `Message` enum into smaller, feature-specific enums.

**Status**: **DEFERRED** - This is a major undertaking that would require:
- Updating all view code (search.rs, metadata.rs, cover.rs, chapters.rs, convert.rs, settings.rs)
- Updating all handler code
- Updating all tests
- Significant refactoring effort

**Recommendation**: 
- **Do NOT do this in the initial refactor**
- Mark as "Future Improvement" if needed
- The current flat `Message` enum is acceptable and works well with the handler pattern
- If enum size becomes a real problem (e.g., >200 variants), consider this as a separate project

**If pursued later**, the approach would be:
1. Create feature-specific message enums
2. Wrap them in main `Message` enum
3. Update all view code to use nested pattern matching
4. Update all handlers to match nested variants

## File Size Estimates (After Refactor)

- `ui/mod.rs`: ~200 lines (Message enum, Lectern struct, Application impl orchestration)
- `ui/state/mod.rs`: ~20 lines
- `ui/state/*.rs`: ~50-100 lines each (5 files = ~400 lines)
- `ui/handlers/mod.rs`: ~50 lines
- `ui/handlers/*.rs`: ~150-300 lines each (8 files = ~1800 lines)
- **Total**: ~2470 lines (more lines but much better organized)

## Benefits

1. **Maintainability**: Each handler is focused on one feature area
2. **Testability**: Can test handlers independently
3. **Readability**: Easier to find and understand specific functionality
4. **Collaboration**: Multiple developers can work on different handlers without conflicts
5. **Scalability**: Easy to add new handlers or modify existing ones

## Migration Strategy

**Recommended Order**:
1. **Phase 1** (State extraction) - least invasive, improves organization immediately
2. **Phase 3** (Extract handlers) - can be done incrementally, one handler at a time
3. **Phase 4** (Refactor update) - completes the refactor, test thoroughly before merging
4. **Phase 5** (Split Message enum) - **DEFERRED** - too invasive, not recommended

**Incremental Approach**:
- Extract one handler at a time (e.g., start with `search.rs`)
- Test after each handler extraction
- Update `update()` incrementally to use new handlers
- Keep old code commented out until all handlers are extracted

**Validation Checklist** (after each phase):
- [ ] Code compiles without warnings
- [ ] All existing tests pass
- [ ] Manual testing of affected features
- [ ] No performance regressions
- [ ] Code review of extracted modules

## State Module Encapsulation

**Current Plan**: State modules use `pub` fields for direct access.

**Concern**: Breaks encapsulation - external code could modify state directly.

**Solution**: Keep fields `pub(crate)` - accessible within the `ui` module but not externally:
```rust
// src/ui/state/search_state.rs
pub struct SearchState {
    pub(crate) query: String,
    pub(crate) author: String,
    pub(crate) by_asin: bool,
    // ... other fields
}

impl SearchState {
    // Provide accessor methods if needed
    pub fn query(&self) -> &str { &self.query }
    pub fn set_query(&mut self, query: String) { self.query = query; }
}
```

**Alternative**: Keep `pub` fields for now (simpler), add accessors later if needed.

## Testing Strategy

Each handler module will include unit tests:

```rust
// src/ui/handlers/search.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Lectern;
    
    #[test]
    fn test_search_query_changed() {
        let mut app = Lectern::default();
        let cmd = handle_search(
            &mut app, 
            Message::SearchQueryChanged("test query".to_string())
        );
        
        assert_eq!(app.search.query, "test query");
        assert!(cmd.is_none() || matches!(cmd, Some(Command::none())));
    }
    
    #[test]
    fn test_perform_search_returns_command() {
        let mut app = Lectern::default();
        app.search.query = "test".to_string();
        
        let cmd = handle_search(&mut app, Message::PerformSearch);
        assert!(cmd.is_some());
        // Command should be async search operation
    }
    
    #[test]
    fn test_search_handler_ignores_non_search_messages() {
        let mut app = Lectern::default();
        let cmd = handle_search(&mut app, Message::TitleChanged("test".to_string()));
        assert_eq!(cmd, None); // Should not handle this message
    }
}
```

**Test Structure**:
- Unit tests for each handler function
- Test that handlers return `None` for messages they don't handle
- Test state updates
- Test Command generation for async operations
- Integration tests for multi-handler interactions

## Rollback Strategy

**Git Strategy**:
1. Create feature branch: `git checkout -b refactor/ui-handlers`
2. Commit after each phase: `git commit -m "Phase 1: Extract state modules"`
3. Keep main branch stable - only merge when refactor is complete and tested

**Feature Flags** (if needed):
```rust
#[cfg(feature = "new-handlers")]
use handlers::*;

#[cfg(not(feature = "new-handlers"))]
// Old implementation
```

**Rollback Plan**:
- If issues discovered mid-refactor: `git reset --hard origin/main`
- If Phase 1-3 complete but Phase 4 has issues: Can keep state modules, revert handler extraction
- Test after each phase to catch issues early

**Validation**:
- Run full test suite after each phase
- Manual testing of all features after each phase
- Compare behavior before/after refactor

## Considerations

- All handlers need access to `&mut Lectern` - this is fine since they're part of the same module
- Some messages might need to update multiple state areas - handlers can coordinate this
- The `view()` function already delegates to view modules, so no changes needed there
- The `subscription()` function is small and can stay in `mod.rs`
- Message cloning in dispatcher is acceptable - `Message` implements `Clone` and variants are small
- Pattern matching alternative avoids cloning but is more verbose
