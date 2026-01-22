import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import QtQuick.Dialogs 1.3

Item {
    property var controller

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 20

        GroupBox {
            title: "Current Cover"
            Layout.fillWidth: true
            Layout.preferredHeight: 300

            Rectangle {
                anchors.fill: parent
                color: Material.color(Material.Grey, Material.Shade800)
                border.color: Material.color(Material.Grey, Material.Shade600)
                border.width: 1
                radius: 4

                Image {
                    id: coverImage
                    anchors.centerIn: parent
                    width: 200
                    height: 280
                    fillMode: Image.PreserveAspectFit
                    source: controller ? controller.metadata_cover_url : ""
                    asynchronous: true

                    onStatusChanged: {
                        if (status === Image.Error) {
                            errorText.visible = true
                        }
                    }
                }

                Label {
                    id: errorText
                    anchors.centerIn: parent
                    text: "No cover image"
                    visible: coverImage.source === ""
                    opacity: 0.6
                }
            }
        }

        GroupBox {
            title: "Cover Actions"
            Layout.fillWidth: true

            RowLayout {
                anchors.fill: parent
                spacing: 12

                Button {
                    text: "🔍 Search for Cover"
                    Layout.fillWidth: true
                    onClicked: {
                        if (controller && controller.metadata_title) {
                            // Search for cover using the book title
                            controller.search_metadata(controller.metadata_title, false)
                        }
                    }
                    enabled: controller && controller.metadata_title !== ""
                }

                Button {
                    text: "📁 Load from File"
                    Layout.fillWidth: true
                    onClicked: coverFileDialog.open()
                }

                Button {
                    text: "🌐 Download from URL"
                    Layout.fillWidth: true
                    onClicked: urlDialog.open()
                }
            }
        }

        GroupBox {
            title: "Cover Information"
            Layout.fillWidth: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 10

                Label {
                    text: "Cover URL:"
                    font.bold: true
                }

                TextField {
                    id: coverUrlField
                    text: controller ? controller.metadata_cover_url : ""
                    Layout.fillWidth: true
                    placeholderText: "https://example.com/cover.jpg"
                    onTextChanged: {
                        if (controller) {
                            controller.metadata_cover_url = text
                        }
                    }
                }

                Label {
                    text: "Cover Status:"
                    font.bold: true
                }

                Label {
                    text: coverImage.status === Image.Ready ? "✅ Loaded successfully" :
                          coverImage.status === Image.Loading ? "⏳ Loading..." :
                          coverImage.status === Image.Error ? "❌ Failed to load" :
                          "❓ No cover set"
                    color: coverImage.status === Image.Ready ? Material.accent :
                           coverImage.status === Image.Error ? "red" :
                           Material.foreground
                }
            }
        }

        Item { Layout.fillHeight: true }
    }

    // File dialog for cover images
    FileDialog {
        id: coverFileDialog
        title: "Select Cover Image"
        nameFilters: ["Image files (*.jpg *.jpeg *.png *.gif *.bmp)"]
        onAccepted: {
            if (controller) {
                var urlString = fileUrl.toString()
                controller.metadata_cover_url = urlString
                coverUrlField.text = urlString
            }
        }
    }

    // Dialog for entering cover URL
    Dialog {
        id: urlDialog
        title: "Enter Cover Image URL"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true
        width: 500
        anchors.centerIn: parent

        ColumnLayout {
            anchors.fill: parent
            spacing: 16

            Label {
                text: "Enter the URL of the cover image:"
            }

            TextField {
                id: urlField
                Layout.fillWidth: true
                placeholderText: "https://example.com/cover.jpg"
            }
        }

        onAccepted: {
            if (controller && urlField.text !== "") {
                controller.metadata_cover_url = urlField.text
                coverUrlField.text = urlField.text
            }
        }
    }

    // Connections to update when metadata changes
    Connections {
        target: controller

        function onMetadata_changed() {
            coverUrlField.text = controller ? controller.metadata_cover_url : ""
        }
    }
}