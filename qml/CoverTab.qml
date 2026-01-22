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

        Label {
            text: "Cover Art"
            font.bold: true
            font.pixelSize: 18
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Material.color(Material.Grey, Material.Shade800)
            radius: 8

            Image {
                id: coverImage
                anchors.centerIn: parent
                source: controller ? controller.metadata_cover_url : ""
                fillMode: Image.PreserveAspectFit
                width: Math.min(parent.width - 40, 350)
                height: Math.min(parent.height - 40, 500)

                Rectangle {
                    anchors.fill: parent
                    color: "transparent"
                    visible: parent.status !== Image.Ready

                    Label {
                        anchors.centerIn: parent
                        text: {
                            if (parent.parent.status === Image.Loading) return "Loading..."
                            if (!controller || controller.metadata_cover_url === "") return "No cover"
                            return "Failed to load"
                        }
                        opacity: 0.5
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true

            TextField {
                id: urlField
                placeholderText: "Cover image URL..."
                text: controller ? controller.metadata_cover_url : ""
                Layout.fillWidth: true
            }

            Button {
                text: "Load"
                onClicked: {
                    if (controller) {
                        controller.metadata_cover_url = urlField.text
                    }
                }
            }

            Button {
                text: "Browse..."
                onClicked: fileDialog.open()
            }
        }
    }

    FileDialog {
        id: fileDialog
        title: "Select Cover Image"
        nameFilters: ["Images (*.jpg *.jpeg *.png)"]
        onAccepted: {
            if (controller) {
                controller.metadata_cover_url = file.toString()
            }
        }
    }
}