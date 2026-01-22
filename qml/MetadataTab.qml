import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import QtQuick.Dialogs 1.3

Item {
    property var controller

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
                    text: "📁 Drag & Drop Audiobook Folder Here"
                    font.pixelSize: 16
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "Supports MP3 folders and M4B files"
                    font.pixelSize: 12
                    opacity: 0.7
                    Layout.alignment: Qt.AlignHCenter
                }

                Button {
                    text: "Or Browse Files..."
                    flat: true
                    Layout.alignment: Qt.AlignHCenter
                    onClicked: folderDialog.open()
                }
            }

            DropArea {
                anchors.fill: parent
                onDropped: {
                    if (drop.hasUrls && drop.urls.length > 0) {
                        var url = drop.urls[0].toString()
                        if (controller) {
                            controller.current_folder = url
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
                text: controller ? controller.current_folder : ""
                Layout.fillWidth: true
                elide: Text.ElideMiddle
                font.pixelSize: 14
                opacity: 0.9
            }
        }

        // Metadata form
        GroupBox {
            title: "Book Information"
            Layout.fillWidth: true

            GridLayout {
                anchors.fill: parent
                columns: 2
                columnSpacing: 16
                rowSpacing: 12

                Label { text: "Title:" }
                TextField {
                    id: titleField
                    text: controller ? controller.metadata_title : ""
                    Layout.fillWidth: true
                    placeholderText: "Book title"
                    onTextChanged: {
                        if (controller && text !== controller.metadata_title) controller.metadata_title = text
                    }
                }

                Label { text: "Author:" }
                TextField {
                    id: authorField
                    text: controller ? controller.metadata_author : ""
                    Layout.fillWidth: true
                    placeholderText: "Author name"
                    onTextChanged: {
                        if (controller && text !== controller.metadata_author) controller.metadata_author = text
                    }
                }

                Label { text: "Series:" }
                TextField {
                    id: seriesField
                    text: controller ? controller.metadata_series : ""
                    Layout.fillWidth: true
                    placeholderText: "Series name (optional)"
                    onTextChanged: {
                        if (controller && text !== controller.metadata_series) controller.metadata_series = text
                    }
                }

                Label { text: "Narrator:" }
                TextField {
                    id: narratorField
                    text: controller ? controller.metadata_narrator : ""
                    Layout.fillWidth: true
                    placeholderText: "Narrator name (optional)"
                    onTextChanged: {
                        if (controller && text !== controller.metadata_narrator) controller.metadata_narrator = text
                    }
                }
            }
        }

        // Search metadata section
        GroupBox {
            title: "Search Online Metadata"
            Layout.fillWidth: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 12

                RowLayout {
                    spacing: 12
                    Layout.fillWidth: true

                    TextField {
                        id: searchField
                        placeholderText: "Enter title or author..."
                        Layout.fillWidth: true
                    }

                    Button {
                        text: "🔍 Search"
                        highlighted: true
                        onClicked: {
                            if (controller && searchField.text !== "") {
                                controller.search_query_input = searchField.text
                                controller.search_by_asin_input = false
                                controller.search_trigger = true
                            }
                        }
                        enabled: controller && !controller.is_processing && searchField.text !== ""
                    }
                }

                // Search results
                RowLayout {
                    visible: controller && controller.search_title !== ""
                    Layout.fillWidth: true
                    spacing: 12

                    Rectangle {
                        width: 80
                        height: 120
                        color: Material.color(Material.Grey, Material.Shade700)
                        radius: 4

                        Image {
                            anchors.fill: parent
                            source: controller ? controller.search_cover_url : ""
                            fillMode: Image.PreserveAspectFit
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 4

                        Label {
                            text: controller ? controller.search_title : ""
                            font.bold: true
                            font.pixelSize: 14
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }

                        Label {
                            text: controller ? "by " + controller.search_author : ""
                            font.italic: true
                            opacity: 0.8
                        }

                        Button {
                            text: "Use This Metadata"
                            onClicked: {
                                if (controller) {
                                    controller.metadata_title = controller.search_title
                                    // Manually update text field binding if needed, though property binding should handle it
                                    // titleField.text = controller.search_title
                                    // authorField.text = controller.search_author
                                    controller.metadata_author = controller.search_author
                                    controller.metadata_cover_url = controller.search_cover_url
                                }
                            }
                        }
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }

    // Folder dialog
    FileDialog {
        id: folderDialog
        title: "Select Audiobook Folder"
        selectFolder: true
        onAccepted: {
            if (controller) {
                controller.current_folder = fileUrl.toString()
            }
        }
    }
}