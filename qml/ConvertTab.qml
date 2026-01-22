import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

Item {
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 24

        Label {
            text: "Convert to M4B"
            font.bold: true
            font.pixelSize: 18
        }

        // Summary
        GroupBox {
            title: "Ready to Convert"
            Layout.fillWidth: true

            GridLayout {
                anchors.fill: parent
                columns: 2
                columnSpacing: 12
                rowSpacing: 8

                Label {
                    text: "Folder:"
                    font.bold: true
                }
                Label {
                    text: controller && controller.current_folder !== "" ? 
                        controller.current_folder : "❌ Not selected"
                    Layout.fillWidth: true
                    elide: Text.ElideMiddle
                    opacity: 0.8
                }

                Label {
                    text: "Title:"
                    font.bold: true
                }
                Label {
                    text: controller && controller.metadata_title !== "" ?
                        controller.metadata_title : "❌ Not set"
                    opacity: 0.8
                }

                Label {
                    text: "Author:"
                    font.bold: true
                }
                Label {
                    text: controller && controller.metadata_author !== "" ?
                        controller.metadata_author : "Not set"
                    opacity: 0.8
                }

                Label {
                    text: "Cover:"
                    font.bold: true
                }
                Label {
                    text: controller && controller.metadata_cover_url !== "" ?
                        "✓ Set" : "Not set"
                    color: controller && controller.metadata_cover_url !== "" ?
                        Material.color(Material.Green) : Material.foreground
                }
            }
        }

        // Upload option
        GroupBox {
            title: "Upload Settings"
            Layout.fillWidth: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 8

                CheckBox {
                    id: uploadCheckbox
                    text: "Upload to Audiobookshelf after conversion"
                    checked: controller && controller.abs_host !== "" && controller.abs_token !== ""
                    enabled: controller && controller.abs_host !== "" && controller.abs_token !== ""
                }

                Label {
                    text: {
                        if (!controller || controller.abs_host === "") {
                            return "⚙️ Configure server in Settings first"
                        }
                        return "Will upload to: " + controller.abs_host
                    }
                    font.pixelSize: 11
                    opacity: 0.6
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }
            }
        }

        Item { Layout.fillHeight: true }

        // Progress
        ColumnLayout {
            visible: controller && controller.is_processing
            Layout.fillWidth: true
            spacing: 8

            Label {
                text: "Converting..."
                font.bold: true
            }

            ProgressBar {
                Layout.fillWidth: true
                value: controller ? controller.progress_value : 0
                indeterminate: controller && controller.progress_value === 0
            }

            Label {
                text: controller ? controller.status_message : ""
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                font.pixelSize: 11
                opacity: 0.7
            }
        }

        // Action button
        Button {
            text: {
                if (!controller) return "Start Conversion"
                if (controller.is_processing) return "⏸ Converting..."
                return "🚀 Start Conversion"
            }
            highlighted: !controller || !controller.is_processing
            Layout.fillWidth: true
            enabled: controller && 
                    controller.current_folder !== "" && 
                    controller.metadata_title !== "" &&
                    !controller.is_processing

            onClicked: {
                if (controller && !controller.is_processing) {
                    controller.start_conversion()
                }
            }
        }

        Label {
            visible: !controller || 
                    controller.current_folder === "" || 
                    controller.metadata_title === ""
            text: "⚠️ Select folder and set title/author before converting"
            color: Material.color(Material.Orange)
            font.pixelSize: 12
            Layout.fillWidth: true
            wrapMode: Text.Wrap
        }
    }
}