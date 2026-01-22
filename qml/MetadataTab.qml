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
            Layout.preferredHeight: 120
            color: Material.color(Material.Grey, Material.Shade800)
            border.color: Material.color(Material.Grey, Material.Shade600)
            border.width: 2
            radius: 8

            ColumnLayout {
                anchors.centerIn: parent
                spacing: 8

                Label {
                    text: "📁 Drag & Drop Folder Here"
                    font.pixelSize: 16
                    Layout.alignment: Qt.AlignHCenter
                }

                Button {
                    text: "Browse..."
                    Layout.alignment: Qt.AlignHCenter
                    onClicked: folderDialog.open()
                }
            }

            DropArea {
                anchors.fill: parent
                onDropped: {
                    if (drop.hasUrls && drop.urls.length > 0) {
                        if (controller) {
                            controller.set_folder_path(drop.urls[0].toString())
                        }
                    }
                }
            }
        }

        // Current folder
        Label {
            visible: controller && controller.current_folder !== ""
            text: "📂 " + (controller ? controller.current_folder : "")
            Layout.fillWidth: true
            elide: Text.ElideMiddle
            font.pixelSize: 12
            opacity: 0.8
        }

        // Metadata form
        GroupBox {
            title: "Book Metadata"
            Layout.fillWidth: true

            GridLayout {
                anchors.fill: parent
                columns: 2
                columnSpacing: 12
                rowSpacing: 8

                Label { text: "Title:" }
                TextField {
                    id: titleField
                    text: controller ? controller.metadata_title : ""
                    Layout.fillWidth: true
                    onEditingFinished: {
                        if (controller) controller.metadata_title = text
                    }
                }

                Label { text: "Author:" }
                TextField {
                    id: authorField
                    text: controller ? controller.metadata_author : ""
                    Layout.fillWidth: true
                    onEditingFinished: {
                        if (controller) controller.metadata_author = text
                    }
                }

                Label { text: "Series:" }
                TextField {
                    text: controller ? controller.metadata_series : ""
                    Layout.fillWidth: true
                    onEditingFinished: {
                        if (controller) controller.metadata_series = text
                    }
                }

                Label { text: "Narrator:" }
                TextField {
                    text: controller ? controller.metadata_narrator : ""
                    Layout.fillWidth: true
                    onEditingFinished: {
                        if (controller) controller.metadata_narrator = text
                    }
                }
            }
        }

        // Search section
        GroupBox {
            title: "Search Metadata"
            Layout.fillWidth: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 12

                RowLayout {
                    TextField {
                        id: searchField
                        placeholderText: "Search by title or author..."
                        Layout.fillWidth: true
                    }

                    Button {
                        text: "🔍 Search"
                        highlighted: true
                        enabled: controller && !controller.is_processing && searchField.text !== ""
                        onClicked: {
                            if (controller) {
                                controller.search_metadata(searchField.text, false)
                            }
                        }
                    }
                }

                // Search results
                RowLayout {
                    visible: controller && controller.search_title !== ""
                    Layout.fillWidth: true
                    spacing: 12

                    Rectangle {
                        width: 60
                        height: 90
                        color: Material.color(Material.Grey, Material.Shade700)

                        Image {
                            anchors.fill: parent
                            source: controller ? controller.search_cover_url : ""
                            fillMode: Image.PreserveAspectFit
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true

                        Label {
                            text: controller ? controller.search_title : ""
                            font.bold: true
                            wrapMode: Text.Wrap
                            Layout.fillWidth: true
                        }

                        Label {
                            text: controller ? "by " + controller.search_author : ""
                            opacity: 0.7
                        }

                        Button {
                            text: "Use This"
                            onClicked: {
                                if (controller) {
                                    titleField.text = controller.search_title
                                    authorField.text = controller.search_author
                                    controller.metadata_title = controller.search_title
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

    FolderDialog {
        id: folderDialog
        title: "Select Audiobook Folder"
        onAccepted: {
            if (controller) {
                controller.set_folder_path(folder.toString())
            }
        }
    }
}