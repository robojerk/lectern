import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import QtQuick.Dialogs

Item {
    property var controller

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 20

        // Drag and drop area with Material Design
        Pane {
            Layout.fillWidth: true
            Layout.preferredHeight: 140
            Material.elevation: 2

            DropArea {
                id: dropArea
                anchors.fill: parent

                Rectangle {
                    anchors.fill: parent
                    color: dropArea.containsDrag ? Material.color(Material.Indigo, Material.Shade200) :
                           Material.color(Material.Grey, Material.Shade800)
                    border.color: dropArea.containsDrag ? Material.accentColor : Material.color(Material.Grey, Material.Shade600)
                    border.width: dropArea.containsDrag ? 3 : 2
                    radius: 12
                    opacity: dropArea.containsDrag ? 0.9 : 0.8

                    Behavior on color { ColorAnimation { duration: 200 } }
                    Behavior on border.color { ColorAnimation { duration: 200 } }
                    Behavior on opacity { OpacityAnimation { duration: 200 } }

                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 12

                        Label {
                            text: dropArea.containsDrag ? "📂 Drop Here!" : "📁 Drop Audiobook Folder Here"
                            font.pixelSize: dropArea.containsDrag ? 20 : 16
                            font.bold: dropArea.containsDrag
                            color: Material.foreground
                            Layout.alignment: Qt.AlignHCenter
                        }

                        Label {
                            text: "Supports MP3 folders and M4B files"
                            font.pixelSize: 12
                            opacity: 0.7
                            color: Material.foreground
                            Layout.alignment: Qt.AlignHCenter
                        }

                        Button {
                            text: "Or Browse Files..."
                            flat: true
                            Layout.alignment: Qt.AlignHCenter
                            onClicked: fileDialog.open()
                        }
                    }
                }

                onDropped: function(drop) {
                    if (drop.hasUrls && drop.urls.length > 0) {
                        var urlString = drop.urls[0].toString()
                        console.log("Dropped URL:", urlString)
                        if (controller) {
                            controller.set_folder_path(urlString)
                        }
                    }
                }
            }
        }

        // Current folder indicator
        RowLayout {
            visible: controller && controller.current_folder !== ""
            Layout.fillWidth: true
            spacing: 12

            Label {
                text: "📁 Current Folder:"
                font.bold: true
            }

            Label {
                text: {
                    if (!controller || !controller.current_folder) return ""
                    var parts = controller.current_folder.split('/')
                    return parts[parts.length - 1] || controller.current_folder
                }
                Layout.fillWidth: true
                elide: Text.ElideMiddle
                font.pixelSize: 14
                opacity: 0.9
            }

            Button {
                text: "Change..."
                flat: true
                onClicked: fileDialog.open()
            }
        }

        // Metadata form with Material Design
        Pane {
            Layout.fillWidth: true
            Material.elevation: 2
            padding: 20

            ColumnLayout {
                anchors.fill: parent
                spacing: 16

                Label {
                    text: "Book Information"
                    font.bold: true
                    font.pixelSize: 16
                    color: Material.primaryColor
                }

                GridLayout {
                    columns: 2
                    columnSpacing: 16
                    rowSpacing: 12
                    Layout.fillWidth: true

                    Label {
                        text: "Title:"
                        font.pixelSize: 14
                        Layout.alignment: Qt.AlignVCenter
                    }
                    TextField {
                        id: titleField
                        text: controller ? controller.metadata_title : ""
                        Layout.fillWidth: true
                        placeholderText: "Book title"
                        Material.accent: Material.DeepPurple
                        onTextChanged: {
                            if (controller) controller.metadata_title = text
                        }
                    }

                    Label {
                        text: "Author:"
                        font.pixelSize: 14
                        Layout.alignment: Qt.AlignVCenter
                    }
                    TextField {
                        id: authorField
                        text: controller ? controller.metadata_author : ""
                        Layout.fillWidth: true
                        placeholderText: "Author name"
                        Material.accent: Material.DeepPurple
                        onTextChanged: {
                            if (controller) controller.metadata_author = text
                        }
                    }

                    Label {
                        text: "Series:"
                        font.pixelSize: 14
                        Layout.alignment: Qt.AlignVCenter
                    }
                    TextField {
                        id: seriesField
                        text: controller ? controller.metadata_series : ""
                        Layout.fillWidth: true
                        placeholderText: "Series name (optional)"
                        Material.accent: Material.DeepPurple
                        onTextChanged: {
                            if (controller) controller.metadata_series = text
                        }
                    }

                    Label {
                        text: "Narrator:"
                        font.pixelSize: 14
                        Layout.alignment: Qt.AlignVCenter
                    }
                    TextField {
                        id: narratorField
                        text: controller ? controller.metadata_narrator : ""
                        Layout.fillWidth: true
                        placeholderText: "Narrator name (optional)"
                        Material.accent: Material.DeepPurple
                        onTextChanged: {
                            if (controller) controller.metadata_narrator = text
                        }
                    }
                }
            }
        }

        // Search metadata section
        Pane {
            Layout.fillWidth: true
            Material.elevation: 2
            padding: 20

            ColumnLayout {
                anchors.fill: parent
                spacing: 16

                Label {
                    text: "Search Online Metadata"
                    font.bold: true
                    font.pixelSize: 16
                    color: Material.primaryColor
                }

                RowLayout {
                    spacing: 12

                    TextField {
                        id: searchField
                        placeholderText: "Enter title, author, or ASIN..."
                        Layout.fillWidth: true
                        Material.accent: Material.DeepPurple
                        onAccepted: {
                            if (controller && text !== "") {
                                var byAsin = searchTypeCombo.currentIndex === 1
                                controller.search_metadata(text, byAsin)
                            }
                        }
                    }

                    ComboBox {
                        id: searchTypeCombo
                        model: ["Title/Author", "ASIN"]
                        currentIndex: 0
                    }

                    Button {
                        text: "🔍 Search"
                        highlighted: true
                        Material.accent: Material.DeepPurple
                        onClicked: {
                            if (controller && searchField.text !== "") {
                                var byAsin = searchTypeCombo.currentIndex === 1
                                controller.search_metadata(searchField.text, byAsin)
                            }
                        }
                        enabled: controller && !controller.is_processing && searchField.text !== ""
                    }
                }

                // Search results display
                RowLayout {
                    Layout.fillWidth: true
                    visible: controller && controller.search_title !== ""
                    spacing: 15

                    Image {
                        source: controller ? controller.search_cover_url : ""
                        fillMode: Image.PreserveAspectFit
                        Layout.preferredWidth: 80
                        Layout.preferredHeight: 120
                        asynchronous: true

                        Rectangle {
                            anchors.fill: parent
                            color: "#eee"
                            visible: parent.status !== Image.Ready
                            Text {
                                anchors.centerIn: parent
                                text: "Loading..."
                                color: "#666"
                            }
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 5

                        Label {
                            text: controller ? controller.search_title : ""
                            font.bold: true
                            font.pixelSize: 16
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }

                        Label {
                            text: "Author: " + (controller ? controller.search_author : "")
                            font.italic: true
                            font.pixelSize: 13
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }

                        Button {
                            text: "Use This Book"
                            Layout.alignment: Qt.AlignLeft
                            onClicked: {
                                if (controller) {
                                    controller.metadata_title = controller.search_title
                                    controller.metadata_author = controller.search_author
                                    controller.metadata_cover_url = controller.search_cover_url
                                    
                                    // Update the text fields
                                    titleField.text = controller.metadata_title
                                    authorField.text = controller.metadata_author
                                }
                            }
                        }
                    }
                }

                Label {
                    text: "Search Audible.com for book metadata and cover art"
                    font.pixelSize: 12
                    opacity: 0.7
                    Layout.fillWidth: true
                }
            }
        }

        Item { Layout.fillHeight: true }
    }

    // File dialog for folder selection
    FolderDialog {
        id: fileDialog
        title: "Select Audiobook Folder"
        onAccepted: {
            if (controller) {
                var urlString = selectedFolder.toString()
                console.log("Selected folder:", urlString)
                controller.set_folder_path(urlString)
            }
        }
    }

    // Connections to controller signals
    Connections {
        target: controller

        function onMetadata_changed() {
            // Update fields when metadata changes
            titleField.text = controller.metadata_title
            authorField.text = controller.metadata_author
            seriesField.text = controller.metadata_series
            narratorField.text = controller.metadata_narrator
        }

        function onFolder_changed() {
            console.log("Folder changed:", controller.current_folder)
        }
    }
}