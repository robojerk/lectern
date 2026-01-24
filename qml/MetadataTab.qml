import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1

Item {
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 20

        // Drag and drop area
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 140
            color: Material.color(Material.Grey, Material.Shade800)
            border.color: Material.color(Material.Grey, Material.Shade600)
            border.width: 2
            radius: 12

            ColumnLayout {
                anchors.centerIn: parent
                spacing: 12

                Label {
                    text: "📁 Drop Audiobook Folder Here"
                    font.pixelSize: 16
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

        // Current folder indicator
        RowLayout {
            visible: controller && controller.current_folder !== ""
            Layout.fillWidth: true
            spacing: 12

            Label {
                text: "📂 Current Folder:"
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

        // Metadata form
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
                    }
                    TextField {
                        id: titleField
                        text: controller ? controller.book_title : ""
                        Layout.fillWidth: true
                        placeholderText: "Book title"
                        onEditingFinished: if (controller) controller.book_title = text
                        Connections {
                            target: controller
                            function onBook_titleChanged() {
                                console.log("[DEBUG] book_title_changed signal received, controller.book_title:", controller ? controller.book_title : "null")
                            }
                        }
                        onTextChanged: {
                            console.log("[DEBUG] titleField text changed to:", text)
                        }
                    }

                    Label {
                        text: "Author:"
                        font.pixelSize: 14
                    }
                    TextField {
                        id: authorField
                        text: controller ? controller.book_author : ""
                        Layout.fillWidth: true
                        placeholderText: "Author name"
                        onEditingFinished: if (controller) controller.book_author = text
                        Connections {
                            target: controller
                            function onBook_authorChanged() {
                                console.log("[DEBUG] book_author_changed signal received, controller.book_author:", controller ? controller.book_author : "null")
                            }
                        }
                    }

                    Label {
                        text: "Series:"
                        font.pixelSize: 14
                    }
                    TextField {
                        id: seriesField
                        text: controller ? controller.book_series : ""
                        Layout.fillWidth: true
                        placeholderText: "Series name (optional)"
                        onEditingFinished: if (controller) controller.book_series = text
                        Connections {
                            target: controller
                            function onBook_seriesChanged() {
                                console.log("[DEBUG] book_series_changed signal received, controller.book_series:", controller ? controller.book_series : "null")
                            }
                        }
                    }

                    Label {
                        text: "Narrator:"
                        font.pixelSize: 14
                    }
                    TextField {
                        id: narratorField
                        text: controller ? controller.book_narrator : ""
                        Layout.fillWidth: true
                        placeholderText: "Narrator name (optional)"
                        onEditingFinished: if (controller) controller.book_narrator = text
                        Connections {
                            target: controller
                            function onBook_narratorChanged() {
                                console.log("[DEBUG] book_narrator_changed signal received, controller.book_narrator:", controller ? controller.book_narrator : "null")
                            }
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
                    }

                    ComboBox {
                        id: searchTypeCombo
                        model: ["Title/Author", "ASIN"]
                        currentIndex: 0
                    }

                    Button {
                        text: "🔍 Search"
                        highlighted: true
                        onClicked: {
                            if (controller && searchField.text !== "") {
                                var byAsin = searchTypeCombo.currentIndex === 1
                                controller.search_metadata(searchField.text, byAsin)
                            }
                        }
                        enabled: controller && searchField.text !== ""
                    }
                }

                Label {
                    text: "Search for book metadata and cover art"
                    font.pixelSize: 12
                    opacity: 0.7
                    Layout.fillWidth: true
                }
            }
        }

        Item { Layout.fillHeight: true }
    }

    // File dialog
    FolderDialog {
        id: fileDialog
        title: "Select Audiobook Folder"
        onAccepted: {
            if (controller) {
                controller.set_folder_path(selectedFolder.toString().replace("file://", ""))
            }
        }
    }
}
