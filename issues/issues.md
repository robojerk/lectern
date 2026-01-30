- The Description field has a grey area where text is not located. It looks bad so remove the gray. Also i want a vertical scrollbar on the right if we fill up the viewable area of the "Description" text field, to give the user a visual indication there's more text
[description](./description.pngclear)

- The app is not detecting images included with the audiobook. For example this audiobook has a file named folder.jpg that is the audio cover.  The file could be other names (including but not limited to cover.jpg) or other image formats.  The app should make an attempt to find a cover, but be aware not all books will have a cover image included.
[cover.png](./cover.png)
How Audiobookshelf looks
[hidden local cover](./ABS_hidden_local_cover.png)
[shown local cover](./ABS_shown_local_cover.png)

- Some audiobooks might come with files that contain metadata, or chapter timestamps.  The app should make an attempt to look for them.  Some audiobooks will not come with anything.

- Some audiobooks come with other files like a .nfo file or .sfv
I am not sure if these files are useful at all.  Your thoughts?
Examples
"/home/rob/Desktop/Waste Bin/Re__REQ_-_Joe_Hill_-_[01_32]_-_/Joe Hill - King Sorrow.sfv"
"/home/rob/Desktop/Waste Bin/Re__REQ_-_Joe_Hill_-_[01_32]_-_/Joe Hill - King Sorrow.nfo"

- **Chapter list crash with many chapters (e.g. 180).** ~~Rendering the full chapter list caused cosmic-text overflow.~~ **Fixed:** The chapter list is now virtualized (only visible rows + buffer are built; scroll tracked via `on_scroll`). Books with 180+ chapters no longer crash. Original: cosmic-text to hit an integer overflow (`attempt to add with overflow` in `cosmic-text` buffer). We currently cap at 100 rendered rows as a temporary workaround and show "Showing first 100 of N chapters...". **This is not a long-term solution.** The UI should be fixed properly: virtualize the chapter list so only visible (or near-visible) rows are built, and/or reduce the amount of text/layout the engine has to handle so books with 180+ chapters don’t crash.

- Tab navigation

- Mapping Chapters from files is now experiencing a long delay. (could be because the book am testing with has 180 files)
I would like one of the following changes:
    1. make it faster (instantaneous)
    2. Give the user some indication the app os working
    spinny wheel, whatever.  we can discuss it

- Show book "Duration" on the chapters tab, maybe just to the right of "Map from files" or just to the left of "Show seconds".  Use this value to make sure no chapter starts after the end of the book. Show this [error symbol](assets/png/error_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.png) next to any chapter in violation.
In the terminal I can get it from this command
```bash
$ date -u +%H:%M:%S -d @$(for file in *.mp3; do ffprobe -v error -select_streams a:0 -show_entries stream=duration -of default=noprint_wrappers=1:nokey=1 "$file"; done | paste -sd+ | bc)
01:02:56
```

- Remove the current buttons to to adjust an individual chapter -1 or +1 second and use these png files.
["-"](../assets/png/remove_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.png)
["+"](../assets/png/add_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.png)

- Currently, The button to add 1 second to an individual chapter is broken.
[chapters tab](./chapters.png)

- I want the scroll wheel on the chapters tab to only be for the chapters.  It should start under the table header.

- The audiobook lookup should dissapper after successful search
It uses a lot of space so should dissappear after search. Or at least have a "Close" or "Hide" button.
[CHapter tab showing audiobook lookup](./chapters_audiobook_lookup.png)

- On the Chapters tab. The playback timer should ONLY be visible when that specific chapter is playing. It should be hidden when not playing.  If that chapter start time is beyond the calculated ond of the book show the error symbol always.
At the moment they are all showing

- Add mouse hover dessciptions to these elements/widgts on the Chapters tab to edit the specific chapters
    "-1 second"
    "+1 second"
    "Lock chapter (Shift+click for range)"
    "unlock chapter (Shift+click for range)"
    "remove chapter"
    "add chapter below button"
    "playback button"

- Add shift+click to lock / unlock a range of chapters
Example I hold shift and then click on Chapter 1 then on Chapter 10.  CHapters 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 all get locked
Include on mouse hover

- On Chpaters tab remove the current "SHift all" buttons
"Shift all: [-1s] [-5s] [+5s] [+1s]
Replace it with a field similiar to Audiobookshelf
[ABS Shift all Exmaple](./ABS_shift_all.png)
user can enter a number (assume positve) or specify they want a negive number by including a minus symbols
Shift is in seconds. Decimals are okay.

- On Chapters tab, remove color squares on the buttons to edit a chapter
    "-1 second"
    "+1 second"
    "Lock chapter (Shift+click for range)"
    "unlock chapter (Shift+click for range)"
    "remove chapter"
    "add chapter below button"
    "playback button"
see [Chapters](./chapters.png)

- On the convert tab. Move the "Source tracks" directly under the "Output location"
[convert](./convert.png)

- On the convert tab. Move the "Metadata Preview" inbetween "Conversion Settings" and "Output location"
[convert](./convert.png)

- Globally on the app, Keep the tabs square, all buttons with text and not a png. round them.