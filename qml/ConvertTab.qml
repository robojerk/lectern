import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

Item {
    property var controller

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 30

        Label {
            text: "Convert to M4B"
            font.bold: true
            font.pixelSize: 18
        }

        // Summary of current settings
        GroupBox {
            title: "Conversion Summary"
            Layout.fillWidth: true

            GridLayout {
                anchors.fill: parent
                columns: 2
                columnSpacing: 16
                rowSpacing: 12

                Label { text: "Source Folder:" ; font.bold: true }
                Label {
                    text: controller && controller.current_folder !== "" ? controller.current_folder : "Not selected"
                    Layout.fillWidth: true
                    elide: Text.ElideMiddle
                    opacity: 0.8
                }

                Label { text: "Title:" ; font.bold: true }
                Label {
                    text: controller && controller.metadata_title !== "" ? controller.metadata_title : "Not set"
                    Layout.fillWidth: true
                    opacity: 0.8
                }

                Label { text: "Author:" ; font.bold: true }
                Label {
                    text: controller && controller.metadata_author !== "" ? controller.metadata_author : "Not set"
                    Layout.fillWidth: true
                    opacity: 0.8
                }

                Label { text: "Cover Art:" ; font.bold: true }
                Label {
                    text: controller && controller.metadata_cover_url !== "" ? "✓ Set" : "Not set"
                    Layout.fillWidth: true
                    opacity: 0.8
                    color: controller && controller.metadata_cover_url !== "" ? Material.color(Material.Green) : Material.foreground
                }
            }
        }

        // Conversion options
        GroupBox {
            title: "Output Options"
            Layout.fillWidth: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 12

                CheckBox {
                    text: "Upload to Audiobookshelf after conversion"
                    checked: controller && controller.abs_host !== "" && controller.abs_token !== ""
                    enabled: controller && controller.abs_host !== "" && controller.abs_token !== ""
                }

                Label {
                    text: controller && controller.abs_host === "" ? 
                          "Configure Audiobookshelf in Settings to enable upload" :
                          "Will upload to: " + (controller ? controller.abs_host : "")
                    font.pixelSize: 12
                    opacity: 0.6
                    Layout.fillWidth: true
                    wrapMode: Text.Wrap
                }
            }
        }

        Item { Layout.fillHeight: true }

        // Progress section
        ColumnLayout {
            visible: controller && controller.is_processing
            Layout.fillWidth: true
            spacing: 12

            Label {
                text: "Converting..."
                font.bold: true
                font.pixelSize: 16
            }

            ProgressBar {
                Layout.fillWidth: true
                value: controller ? controller.progress_value : 0
                indeterminate: controller ? controller.progress_value === 0 : false
            }

            Label {
                text: controller ? controller.status_message : ""
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                font.pixelSize: 12
                opacity: 0.8
            }
        }

        // Action buttons
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            Button {
                text: controller && controller.is_processing ? "Cancel Conversion" : "🚀 Start Conversion"
                highlighted: !controller || !controller.is_processing
                Layout.fillWidth: true
                enabled: controller && controller.current_folder !== "" && controller.metadata_title !== ""
                onClicked: {
                    if (controller) {
                        if (controller.is_processing) {
                            controller.cancel_conversion()
                        } else {
                            controller.start_conversion()
                        }
                    }
                }
            }
        }

        Label {
            visible: !controller || controller.current_folder === "" || controller.metadata_title === ""
            text: "⚠️ Please select a folder and set metadata before converting"
            color: Material.color(Material.Orange)
            font.pixelSize: 12
            Layout.fillWidth: true
            wrapMode: Text.Wrap
        }
    }
}