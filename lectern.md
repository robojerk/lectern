# Lectern

An app to manage individual audiobooks.

Once app is opened, user can drag audiobook onto app, or click on a button to open xdg portal (File chooser).

Audio books come in a dir with many mp3's, aac, wav, flac etc.  or as m4b files

Once audiobook is selected/chosen in the app. User can edit the metadata manully, or search a providor for metadata then edit it.  Chnage cover, add/edit chapters

Then save it all as a m4b file with everything included in one file.

## Metadata

Search by using title, author or the ASIN or ISBN number (may have to choose region)

Possible search providers
audible.com
Google Books
iTunes
Open Library
FantLab.ru
audible.ca

Example of possible metadata fields to be filled out by search, or audible.
Title
Subtitle
Author(s)
Series (and order 1, 2, 3, 3.5, 4, etc)
Publich Year
Description
Genre
Tags
Narrator(s)
ISBN
ASIN
Publisher
Language
Explicit (yes/no)
Abridged (yes/no)

See image [meta-data-form.jpg](./plan/meta-data-form.jpg)

## Cover (optional)

User can manually choose a book cover, or use a providor.

Search,download and choose cover art from provider(s)
audible.com
Google Books
iTunes
Open Library
FantLab.ru
AudiobookCovers.com

Search by using one or more of the following fields
Title
ASIN or ISBN
Author

## Chapters (Optional)

Attempt to map chapters to the book

Have button to get chapters from providor (Audible.com or Audible.ca).  If there are other possible providers let me know.

Provide option to set chapter titles from source files (if they are named like "Chapter 1.mp3", "Chapter 2.mp3", etc)

Example 1 - if there are 70 mp3's, and 70 chapters assume each file is a chapter and ask user to accept

Example 2 - user choose audiobook that is a m4b file
provider has a chapter list with time stamps.  Ask user to accept

Allow user to edit chapters manually, even after looking up from provider. Like Audiobookshelf web client.
User can do the following

Add, edit, remove chapters
Editing chapters includes editing chapter title. and the start timestamp of the chapter.
Global shift times of all chapters
lock chpters (this makes chapter non editable from all actions including global time shift)
shift times of individual chapters by -1 or +1, or just edit the start time.
Play the chapter. So user knows the mapping is correct.

See image [chapters.jpg](./plan/chapter-edit.png)
The icons to the right of the title fields in the image.
From left to right "Lock Chapter" "Remove Chapter" "Insert Chapter Below" "Play Chapter"

## Conversion

Once user is satisfied with metadata, cover, chapter, they can create a m4b file where all the selected info will be included.

If Local Library value (see settings) is set automatically populate where file will be saved

If Local Library value is NOT set force user to choose where to save file (xdg portal file chooser).  default save filename {Title}.m4b

Generate m4b file (include everything we added)

## Settings

### Local Library (optional)

Allow user to state where they wants to store audiobooks on their local system.  If value exists m4b files are converted there

### Media Management

Where and how do we save the file?

#### If Local Library value is set

How does user want to store files on their system?

{Author} - Author/narrator name
{Series} - Series name
{Title} - Book title
{SeriesNumber} - Position in series
{DiskNumber} or {DiskNumber:00} - Disk/part number (00 = zero-padded)
{ChapterNumber} or {ChapterNumber:00} - Chapter number (00 = zero-padded)
{Year} - Publication year
{Quality} - Audio quality

Examples of how user can set it up in Lectern
{Path to Local Library}/{Author}/{Title}.m4b
{Path to Local Library}/{Author}/{Series}/{Title}-Book {SeriesNumber}.m4b
{Path to Local Library}/{Author}/{Series}/Book {SeriesNumber}- {Title}.m4b

Example of LOTR "The Fellowship of the Ring" & "The Hobbit"
{Path to Local Library}/{Author}/{Series}/{SeriesNumber}- {Title}.m4b
{Path to Local Library}/J.R.R. Tolkien/The Lord of the Rings/1- The Fellowship of the Ring.m4b
{Path to Local Library}/J.R.R. Tolkien/The Lord of the Rings/0.5- The Hobbit.m4b


### Audiobookshelf settings (optional)

If set, once m4b conversion is completed the book is uploaded to AudioBookshelf server


## Look

I want to add tabs so the window is not so congested

Details (Metadata)
Cover (Manually upload or choose cover using search providor)
Chapters (Manually edit or match to providor)
Match (search providor to get metadata)
Convert (click to start conversion)

## Progress

The core functionality of Lectern is now implemented, featuring a tabbed interface for organizing the audiobook creation workflow.

- [x] **UI Tabs**: Implemented Metadata, Cover, Chapters, and Convert tabs.
- [x] **Metadata**: Integrated Audible API for searching book details.
- [x] **Cover Art**: Async loading of cover art from search results.
- [x] **Chapters**: Functional chapter editor with inline editing, locking, and global time shifts.
- [x] **Conversion**: M4B generation using FFmpeg with embedded chapters and tags.
- [x] **Publishing**: Direct upload to Audiobookshelf with library scan trigger.

[Progress Details](./PROGRESS.md)