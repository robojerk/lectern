import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

Item {
    property var controller

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 20

        Label {
            text: "Cover Art Preview"
            font.bold: true
            font.pixelSize: 18
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Material.color(Material.Grey, Material.Shade800)
            radius: 8

            Image {
                anchors.centerIn: parent
                source: controller ? controller.metadata_cover_url : ""
                fillMode: Image.PreserveAspectFit
                width: Math.min(parent.width - 40, 400)
                height: Math.min(parent.height - 40, 600)

                Rectangle {
                    anchors.fill: parent
                    color: Material.color(Material.Grey, Material.Shade700)
                    visible: parent.status !== Image.Ready
                    radius: 4

                    Label {
                        anchors.centerIn: parent
                        text: parent.parent.status === Image.Loading ? "Loading..." : 
                              controller && controller.metadata_cover_url === "" ? "No cover art" : "Failed to load"
                        opacity: 0.5
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            TextField {
                id: coverUrlField
                placeholderText: "Enter cover image URL..."
                text: controller ? controller.metadata_cover_url : ""
                Layout.fillWidth: true
                onTextChanged: {
                    if (controller && text !== controller.metadata_cover_url) controller.metadata_cover_url = text
                }
            }

            Button {
                text: "Apply"
                onClicked: {
                    if (controller) {
                        controller.metadata_cover_url = coverUrlField.text
                    }
                }
            }
        }

        Label {
            text: "Tip: Search for metadata first to automatically get cover art"
            font.pixelSize: 12
            opacity: 0.6
            Layout.fillWidth: true
        }
    }
}