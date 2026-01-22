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
        Card {
            Layout.fillWidth: true
            Layout.preferredHeight: 140

            Rectangle {
                id: dropArea
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

                DropArea {
                    anchors.fill: parent
                    onDropped: function(drop) {
                        if (drop.hasUrls && drop.urls.length > 0) {
                            var url = drop.urls[0]
                            if (controller) {
                                controller.set_folder_path(url.toString().replace("file://", ""))
                            }
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

            IconLabel {
                icon: "folder"
                text: "Current Folder:"
                font.bold: true
            }

            Label {
                text: controller ? controller.current_folder.split('/').pop() : ""
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
                        text: "Sample Book Title"
                        Layout.fillWidth: true
                        placeholderText: "Book title"
                        Material.accent: Material.DeepPurple
                    }

                    Label {
                        text: "Author:"
                        font.pixelSize: 14
                        Layout.alignment: Qt.AlignVCenter
                    }
                    TextField {
                        id: authorField
                        text: "Sample Author"
                        Layout.fillWidth: true
                        placeholderText: "Author name"
                        Material.accent: Material.DeepPurple
                    }

                    Label {
                        text: "Series:"
                        font.pixelSize: 14
                        Layout.alignment: Qt.AlignVCenter
                    }
                    TextField {
                        id: seriesField
                        text: "Sample Series"
                        Layout.fillWidth: true
                        placeholderText: "Series name (optional)"
                        Material.accent: Material.DeepPurple
                    }

                    Label {
                        text: "Narrator:"
                        font.pixelSize: 14
                        Layout.alignment: Qt.AlignVCenter
                    }
                    TextField {
                        id: narratorField
                        text: "Sample Narrator"
                        Layout.fillWidth: true
                        placeholderText: "Narrator name (optional)"
                        Material.accent: Material.DeepPurple
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
                controller.set_folder_path(selectedFolder.toString().replace("file://", ""))
            }
        }
    }

    // Connections to controller signals
    Connections {
        target: controller

        function onMetadata_loaded() {
            // Fields are bound via properties, so they update automatically
            console.log("Metadata loaded successfully")
        }

        function onFolder_dropped(url) {
            console.log("Folder dropped:", url)
        }
    }
}