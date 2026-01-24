import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1

ApplicationWindow {
    id: window
    visible: true
    width: 900
    height: 700
    title: "Lectern - Audiobook Tool"

    Material.theme: Material.Dark
    Material.accent: Material.Purple
    Material.primary: Material.DeepPurple

    // Main content area
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header bar
        Rectangle {
            Layout.fillWidth: true
            height: 56
            color: Material.color(Material.Grey, Material.Shade800)

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 16

                RowLayout {
                    spacing: 12

                    Rectangle {
                        width: 32
                        height: 32
                        radius: 6
                        color: Material.accent

                        Label {
                            anchors.centerIn: parent
                            text: "🎵"
                            font.pixelSize: 16
                        }
                    }

                    Label {
                        text: "Lectern"
                        font.bold: true
                        font.pixelSize: 20
                        color: Material.primaryColor
                    }
                }

                Item { Layout.fillWidth: true }

                Label {
                    text: controller ? controller.current_folder : "No folder selected"
                    opacity: 0.7
                    elide: Text.ElideMiddle
                    Layout.maximumWidth: 300
                }

                ToolButton {
                    text: "⚙"
                    font.pixelSize: 16
                    onClicked: settingsDialog.open()

                    ToolTip.visible: hovered
                    ToolTip.text: "Settings"
                    ToolTip.delay: 500
                }
            }
        }

        // Tab bar
        TabBar {
            id: tabBar
            Layout.fillWidth: true
            background: Rectangle { color: Material.color(Material.Grey, Material.Shade800) }

            TabButton {
                text: "📁 Metadata"
                font.pixelSize: 14
            }
            TabButton {
                text: "🖼️ Cover"
                font.pixelSize: 14
            }
            TabButton {
                text: "📑 Chapters"
                font.pixelSize: 14
            }
            TabButton {
                text: "🔄 Convert"
                font.pixelSize: 14
            }
        }

        // Tab content
        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: tabBar.currentIndex

            MetadataTab { }
            CoverTab { }
            ChaptersTab { }
            ConvertTab { }
        }

        // Status bar
        Rectangle {
            Layout.fillWidth: true
            height: 32
            color: Material.color(Material.Grey, Material.Shade800)
            border.color: Material.color(Material.Grey, Material.Shade700)
            border.width: 1

            RowLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 12

                Label {
                    text: {
                        if (!controller) return "ℹ️"
                        if (controller.is_processing) return "⏳"
                        if (controller.status_message.indexOf("✓") >= 0) return "✓"
                        if (controller.status_message.indexOf("❌") >= 0) return "❌"
                        return "ℹ️"
                    }
                    font.pixelSize: 14
                }

                Label {
                    text: controller ? controller.status_message : "Ready"
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                    font.pixelSize: 12
                }

                ProgressBar {
                    visible: controller ? controller.is_processing : false
                    value: controller ? controller.progress_value : 0.0
                    Layout.preferredWidth: 200
                    height: 8
                }
            }
        }
    }

    // Settings dialog
    Popup {
        id: settingsDialog
        modal: true
        width: 600
        height: 500
        anchors.centerIn: parent

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            // Header
            Rectangle {
                Layout.fillWidth: true
                height: 50
                color: Material.color(Material.Grey, Material.Shade800)

                Label {
                    text: "Settings"
                    font.bold: true
                    font.pixelSize: 18
                    anchors.centerIn: parent
                    color: Material.primaryColor
                }
            }

            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true

                ColumnLayout {
                    spacing: 16
                    width: settingsDialog.width - 32

                Label {
                    text: "Audiobookshelf Server Connection:"
                    font.pixelSize: 14
                    opacity: 0.8
                }

                ColumnLayout {
                    spacing: 4

                    Label {
                        text: "Server URL"
                        font.pixelSize: 11
                        opacity: 0.7
                    }

                    TextField {
                        id: hostField
                        placeholderText: "https://abs.yourdomain.com"
                        text: controller ? controller.abs_host : ""
                        Layout.fillWidth: true
                    }
                }

                ColumnLayout {
                    spacing: 4

                    Label {
                        text: "API Token"
                        font.pixelSize: 11
                        opacity: 0.7
                    }

                    TextField {
                        id: tokenField
                        placeholderText: "Your API token"
                        text: controller ? controller.abs_token : ""
                        echoMode: TextInput.Password
                        Layout.fillWidth: true
                    }
                }

                ColumnLayout {
                    spacing: 4

                    Label {
                        text: "Library ID"
                        font.pixelSize: 11
                        opacity: 0.7
                    }

                    TextField {
                        id: libraryField
                        placeholderText: "Library UUID"
                        text: controller ? controller.abs_library_id : ""
                        Layout.fillWidth: true
                    }
                }

                Rectangle {
                    height: 1
                    color: Material.color(Material.Grey, Material.Shade700)
                    Layout.fillWidth: true
                    Layout.topMargin: 8
                    Layout.bottomMargin: 8
                }

                Label {
                    text: "Local Library (Optional):"
                    font.pixelSize: 14
                    opacity: 0.8
                }

                ColumnLayout {
                    spacing: 4

                    Label {
                        text: "Library Path"
                        font.pixelSize: 11
                        opacity: 0.7
                    }

                    RowLayout {
                        TextField {
                            id: localLibraryField
                            placeholderText: "/home/user/audiobooks"
                            text: controller ? controller.local_library_path : ""
                            Layout.fillWidth: true
                        }

                        Button {
                            text: "Browse..."
                            onClicked: localLibraryDialog.open()
                        }
                    }
                }

                ColumnLayout {
                    spacing: 4

                    Label {
                        text: "Path Template"
                        font.pixelSize: 11
                        opacity: 0.7
                    }

                    TextField {
                        id: pathTemplateField
                        placeholderText: "{Path to Local Library}/{Author}/{Title}.m4b"
                        text: controller ? controller.path_template : "{Path to Local Library}/{Author}/{Title}.m4b"
                        Layout.fillWidth: true
                    }
                }

                Label {
                    text: "Available placeholders: {Author}, {Series}, {Title}, {SeriesNumber}, {Year}, {Quality}"
                    font.pixelSize: 10
                    opacity: 0.6
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }

                Label {
                    text: "Example: {Path to Local Library}/{Author}/{Series}/Book {SeriesNumber}- {Title}.m4b"
                    font.pixelSize: 10
                    opacity: 0.6
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }
            }
            }

            // Footer with buttons
            Rectangle {
                Layout.fillWidth: true
                height: 60
                color: Material.color(Material.Grey, Material.Shade800)

                RowLayout {
                    anchors.centerIn: parent
                    spacing: 16

                    Button {
                        text: "Cancel"
                        onClicked: settingsDialog.close()
                    }

                    Button {
                        text: "OK"
                        highlighted: true
                        onClicked: {
                            if (controller) {
                                controller.save_config(hostField.text, tokenField.text, libraryField.text, localLibraryField.text, pathTemplateField.text)
                            }
                            settingsDialog.close()
                        }
                    }
                }
            }
        }
    }

    FolderDialog {
        id: localLibraryDialog
        title: "Select Local Library Folder"
        onAccepted: {
            localLibraryField.text = localLibraryDialog.folder.toString().replace("file://", "")
        }
    }

    // Error dialog
    Popup {
        id: errorDialog
        modal: true
        width: 400
        height: 200
        anchors.centerIn: parent

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 20
            spacing: 16

            Label {
                text: "Error"
                font.bold: true
                font.pixelSize: 18
                color: Material.primaryColor
            }

            Label {
                id: errorLabel
                text: ""
                wrapMode: Text.Wrap
                Layout.fillWidth: true
            }

            Item { Layout.fillHeight: true }

            Button {
                text: "OK"
                Layout.alignment: Qt.AlignRight
                onClicked: errorDialog.close()
            }
        }
    }

    // Search results dialog
    SearchResultsDialog {
        id: searchResultsDialog
        controller: window.controller
        
        // Explicit signal connection
        Component.onCompleted: {
            print("========================================")
            print("[DEBUG] SearchResultsDialog Component.onCompleted")
            print("[DEBUG] Connecting bookSelected signal explicitly...")
            try {
                searchResultsDialog.bookSelected.connect(function(book) {
                    print("========================================")
                    print("[DEBUG] [EXPLICIT CONNECT] Signal received!")
                    print("[DEBUG] Book parameter:", book)
                    handleBookSelected(book)
                })
                print("[DEBUG] Signal connection successful")
            } catch(e) {
                print("[DEBUG] ERROR connecting signal:", e)
            }
            print("========================================")
        }
        
        // Also try the onBookSelected syntax
        onBookSelected: function(book) {
            print("========================================")
            print("[DEBUG] [onBookSelected SYNTAX] Handler called!")
            print("[DEBUG] Book parameter:", book)
            handleBookSelected(book)
        }
        
        function handleBookSelected(book) {
            // Apply selected book metadata to the metadata tab
            print("[DEBUG] ===== onBookSelected handler called =====")
            print("[DEBUG] Selected book object type:", typeof book)
            print("[DEBUG] Book object:", JSON.stringify(book))
            print("[DEBUG] Book title:", book ? book.title : "null")
            print("[DEBUG] Book authors:", book ? book.authors : "null", "type:", book && book.authors ? typeof book.authors : "null")
            print("[DEBUG] Book narrator_names:", book ? book.narrator_names : "null", "type:", book && book.narrator_names ? typeof book.narrator_names : "null")
            print("[DEBUG] Book series_name:", book ? book.series_name : "null")
            print("[DEBUG] Controller exists:", !!controller)
            
            // Extract fields from book object and pass to controller
            if (controller) {
                print("[DEBUG] Controller is valid, extracting fields...")
                var title = book.title || ""
                
                // Handle authors - it's always a QVariantList from Rust
                var author = ""
                if (book.authors) {
                    if (Array.isArray(book.authors)) {
                        author = book.authors.join(", ")
                    } else if (typeof book.authors === 'string') {
                        author = book.authors
                    } else {
                        // Try to convert QVariantList to array
                        var authorsArray = []
                        var authorIdx = 0
                        try {
                            for (authorIdx = 0; authorIdx < book.authors.length; authorIdx++) {
                                authorsArray.push(book.authors[authorIdx])
                            }
                            author = authorsArray.join(", ")
                        } catch(err) {
                            print("[DEBUG] Error extracting authors:", err)
                            author = ""
                        }
                    }
                }
                
                var series = book.series_name || ""
                
                // Handle narrator_names - it's a QVariantList from Rust
                var narrator = ""
                if (book.narrator_names) {
                    if (Array.isArray(book.narrator_names)) {
                        narrator = book.narrator_names.join(", ")
                    } else if (typeof book.narrator_names === 'string') {
                        narrator = book.narrator_names
                    } else {
                        // Try to convert QVariantList to array
                        var narratorsArray = []
                        var narratorIdx = 0
                        try {
                            for (narratorIdx = 0; narratorIdx < book.narrator_names.length; narratorIdx++) {
                                narratorsArray.push(book.narrator_names[narratorIdx])
                            }
                            narrator = narratorsArray.join(", ")
                        } catch(err) {
                            print("[DEBUG] Error extracting narrators:", err)
                            narrator = ""
                        }
                    }
                }
                
                print("[DEBUG] Extracted values - title:", title, "author:", author, "series:", series, "narrator:", narrator)
                print("[DEBUG] Calling apply_search_result with:", title, author, series, narrator)
                try {
                    controller.apply_search_result(title, author, series, narrator)
                    print("[DEBUG] apply_search_result called successfully")
                    print("[DEBUG] After apply_search_result, controller.book_title:", controller.book_title)
                    print("[DEBUG] After apply_search_result, controller.book_author:", controller.book_author)
                    print("[DEBUG] After apply_search_result, controller.book_series:", controller.book_series)
                    print("[DEBUG] After apply_search_result, controller.book_narrator:", controller.book_narrator)
                } catch(e) {
                    print("[DEBUG] ERROR calling apply_search_result:", e)
                }
            } else {
                print("[DEBUG] ERROR: controller is null!")
            }
            print("[DEBUG] ===== handleBookSelected finished =====")
        }
        
        // Also try Connections as backup
        Connections {
            target: searchResultsDialog
            function onBookSelected(book) {
                print("[DEBUG] [Connections] Handler called via Connections")
                // Don't call handleBookSelected again to avoid double processing
            }
        }
    }

    Connections {
        target: controller

        function onError_occurred(message) {
            errorLabel.text = message
            errorDialog.open()
        }

        function onLog_message(message) {
            console.log("LECTERN:", message)
        }

        function onMetadata_changed() {
            console.log("Metadata changed")
        }

        function onConversion_completed() {
            console.log("Conversion completed")
        }
        
        function onSearch_results_ready(results) {
            console.log("Search returned", results.length, "results")
            console.log("First result:", JSON.stringify(results[0] || {}))
            searchResultsDialog.showResults(results)
        }
    }

    // Timer to poll for search results
    Timer {
        id: searchResultsTimer
        interval: 100  // Check every 100ms
        running: true
        repeat: true
        onTriggered: {
            if (controller) {
                var results = controller.check_search_results()
                // Results are automatically emitted via signal in check_search_results
            }
        }
    }

    Component.onCompleted: {
        console.log("✅ Lectern QML UI loaded")
        console.log("Window:", width, "x", height)
    }
}
