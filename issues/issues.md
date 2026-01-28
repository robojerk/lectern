# Open issues

See [lectern prompt](../lectern.md)
See [lectern_iced prompt](../lectern_iced.md)

## Completed ✅
- ✅ Search providers: Added Audible.com, Audible.ca, iTunes, FantLab.ru (stub)
- ✅ Author search field added to refine searches
- ✅ Pagination: 10 results per page, only downloads covers for current page
- ✅ Cover images in search results (with placeholders while loading)
- ✅ Cover image caching (in-memory) to avoid re-downloading
- ✅ Metadata fields: All fields now populated from search providers
- ✅ Settings tab implemented (was already present, just needed verification)
- ✅ Tab alignment fixed - tabs no longer shift vertically on Cover and Chapters tabs
- ✅ Chapter start time validation - ensures start time >= previous chapter start time
- ✅ Adjust chapter start time from playback timer - click time display during playback to set to current position

## Remaining Issues

- Cover image downloads still cause UI freezing (needs further optimization)
- The "Search" button for book metadata does not need to be viewable on all tabs (Note: Already implemented - only shows on Metadata tab)
- cover tab needs to be more efficient with space. Can only view search results if app is maximized to take up whole screen
- convert work is all stub code I believe at the moment.


On Hold: Do not attempt until user has done more research

- Wayland drag and drop - **RESEARCHED**: See [drag_n_drop_issue.md](../drag_n_drop_issue.md). This is an upstream limitation in winit/Iced. File picker buttons work as a workaround. Waiting for winit PR #2429 to be merged.
- Chapter editor. needs refinement.  will create a list just for that later. Utilitarian useful at the moment.