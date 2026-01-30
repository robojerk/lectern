- ~~playback issue~~ (fixed)
  On this book, editing a chapter start time to 00:00:15 then hitting play used to play from 04:10:00 (time cell was one big "click to set" button, so clicking overwrote start time). Fixed: time during playback is read-only with a separate "Set" button, and the editing cache is synced when start time is updated. [time at 00:00:15](./15.png) & [time at 04:10:00](./04:10:00.png)

- ~~Remove "Extract from file", "Map from files", "Validate"~~ (done)
  Removed from the Chapters tab; mapping/extraction happens when opening the audiobook.

- ~~Lookup results takeover view~~ (done)
  When chapters are looked up from Audible, the window now shows a full-window results view (like metadata search): list of all chapters with timestamps and titles, and three actions: **Apply Chapters**, **Map Chapter Titles**, **Cancel** (back to main chapter tab).
  [results of searching for asin in chapters area of ABS](./abs.png)