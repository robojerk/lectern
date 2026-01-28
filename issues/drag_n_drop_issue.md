# Drag and Drop File Support Issue on Wayland

## Problem Summary

Drag-and-drop file operations from file managers (Dolphin, Nautilus, etc.) do not work on Wayland when using Iced 0.12. The system shows a "forbidden" cursor icon, indicating that the window is not accepting file drops.

## Root Cause

This is a **known limitation** in the underlying windowing library stack:

1. **Iced 0.12** uses **winit** for window management
2. **Winit** does not currently implement file drag-and-drop events on Wayland
3. The `window::Event::FileDropped` event is **not implemented** for Wayland platforms
4. This is documented in the Iced API: file drop events are explicitly marked as not supported on Wayland

### Upstream Status

- **Winit Issue**: [#1881 - Wayland drag-and-drop support](https://github.com/rust-windowing/winit/issues/1881)
- **Winit PR**: [#2429 - Add Wayland drag-and-drop support](https://github.com/rust-windowing/winit/pull/2429) (in progress, not merged)
- **Iced Issue**: [#2597 - File drag-and-drop on Wayland](https://github.com/iced-rs/iced/issues/2597)

Once winit merges Wayland drag-and-drop support, Iced will automatically inherit it without requiring code changes.

## Technical Details

### Wayland Drag-and-Drop Protocol

Wayland uses the following protocol objects for drag-and-drop:
- `wl_data_source` - Represents the source of drag data
- `wl_data_offer` - Represents the offered data at the drop destination
- `wl_data_device_manager` - Manages data device interactions

These protocols require:
- Direct access to the Wayland surface from winit
- A separate event loop for wayland-client protocols
- File URI to path conversion
- Integration with the compositor's drag-and-drop system

### Current Implementation

The application subscribes to file drop events:

```rust
fn subscription(&self) -> Subscription<Message> {
    event::listen_with(|event, _status| {
        match event {
            event::Event::Window(_window_id, window::Event::FileDropped(paths)) => {
                // This event is never received on Wayland
                Some(Message::FileDropped(...))
            }
            _ => None
        }
    })
}
```

On X11, this works correctly. On Wayland, the event is never emitted because winit doesn't implement it.

## Workaround

### Current Solution

The application provides **file picker buttons** that work on all platforms, including Wayland:

- **"Browse Files..."** - Opens a file picker dialog (works on Wayland via XDG portals)
- **"Browse Folder..."** - Opens a folder picker dialog (works on Wayland via XDG portals)

These use the `rfd` (Rust File Dialog) crate, which properly integrates with Wayland through XDG Desktop Portal.

### User Experience

- On **X11**: Both drag-and-drop and file picker buttons work
- On **Wayland**: Only file picker buttons work (drag-and-drop shows forbidden cursor)
- The UI displays a helpful note on Wayland explaining the limitation

## Why Not Implement a Custom Solution?

Implementing native Wayland drag-and-drop would require:

1. **Direct Wayland Protocol Access**
   - Accessing winit's internal Wayland surface handle
   - Setting up a separate wayland-client event loop
   - Managing `wl_data_device_manager` and related protocols

2. **Complex Integration**
   - Bridging between Iced's event loop and wayland-client's event loop
   - Handling file URI conversion (Wayland uses `file://` URIs)
   - Managing protocol state and cleanup

3. **Maintenance Burden**
   - Custom implementation would need to be maintained separately
   - Would break when winit adds native support
   - Platform-specific code increases complexity

4. **Upstream Progress**
   - Winit PR #2429 is actively being worked on
   - Once merged, the feature will work automatically
   - Better to wait for official support than maintain a workaround

## Testing

### How to Verify the Issue

1. **On Wayland**:
   ```bash
   # Check if running on Wayland
   echo $WAYLAND_DISPLAY
   
   # Run the application
   cargo run
   
   # Try dragging a file from Dolphin/Nautilus
   # Result: Red "forbidden" cursor, no file drop event
   ```

2. **On X11** (for comparison):
   ```bash
   # Switch to X11 session or use XWayland
   # Run the application
   cargo run
   
   # Try dragging a file
   # Result: File drop works correctly
   ```

### Expected Behavior

- **X11**: File drops should work immediately
- **Wayland**: File drops should work once winit PR #2429 is merged

## Future Resolution

### Timeline

The issue will be resolved when:
1. Winit PR #2429 is merged (adds Wayland drag-and-drop)
2. A new winit release is published
3. Iced updates to the new winit version
4. Lectern updates to the new Iced version

### No Code Changes Required

Once winit adds support, the existing code will work automatically:

```rust
// This code is already correct - it just needs winit support
event::Event::Window(_window_id, window::Event::FileDropped(paths)) => {
    Some(Message::FileDropped(...))
}
```

## References

- [Winit Issue #1881](https://github.com/rust-windowing/winit/issues/1881) - Wayland drag-and-drop request
- [Winit PR #2429](https://github.com/rust-windowing/winit/pull/2429) - Implementation in progress
- [Iced Issue #2597](https://github.com/iced-rs/iced/issues/2597) - File drag-and-drop on Wayland
- [Iced Window Events Documentation](https://docs.iced.rs/iced/window/enum.Event.html) - Notes about Wayland limitations
- [Wayland Data Transfer Protocol](https://wayland.freedesktop.org/docs/html/ch04.html) - Protocol specification
- [Smithay Client Toolkit](https://github.com/Smithay/smithay-client-toolkit) - Wayland protocol helpers

## Related Issues

- KDE Plasma and GNOME are both dropping X11 support in upcoming releases
- This makes Wayland drag-and-drop support critical for future compatibility
- The winit implementation will benefit all Rust GUI applications using winit

## Conclusion

This is an **upstream limitation** that will be resolved when winit adds Wayland drag-and-drop support. The application provides a working alternative (file picker buttons) that functions correctly on all platforms. No application-level workaround is recommended, as it would be complex to maintain and would become obsolete once winit adds native support.

---

**Status**: Waiting for upstream fix  
**Priority**: Medium (workaround available)  
**Last Updated**: 2026-01-24
