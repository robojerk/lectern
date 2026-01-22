import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import QtQuick.Dialogs

Item {
    property var controller

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 20

        // Conversion settings
        GroupBox {
            title: "Output Settings"
            Layout.fillWidth: true

            GridLayout {
                columns: 2
                columnSpacing: 10
                rowSpacing: 10
                anchors.fill: parent

                Label { text: "Output Directory:" }
                RowLayout {
                    TextField {
                        id: outputDirField
                        placeholderText: "~/Audiobooks"
                        text: "~/Audiobooks"
                        Layout.fillWidth: true
                    }

                    Button {
                        text: "📁"
                        onClicked: outputDirDialog.open()
                    }
                }

                Label { text: "Filename Template:" }
                ComboBox {
                    id: filenameTemplate
                    model: [
                        "{Author} - {Title}.m4b",
                        "{Author}/{Title}.m4b",
                        "{Author}/{Series}/Book {SeriesNumber} - {Title}.m4b",
                        "{Series}/Book {SeriesNumber} - {Title}.m4b"
                    ]
                    currentIndex: 0
                    Layout.fillWidth: true
                }

                Label { text: "Audio Quality:" }
                ComboBox {
                    id: qualityCombo
                    model: ["64 kbps", "96 kbps", "128 kbps", "192 kbps", "256 kbps"]
                    currentIndex: 2 // 128 kbps
                    Layout.fillWidth: true
                }

                CheckBox {
                    id: uploadCheckbox
                    text: "Upload to Audiobookshelf"
                    checked: controller && controller.abs_host !== "" && controller.abs_token !== ""
                    enabled: controller && controller.abs_host !== "" && controller.abs_token !== ""
                    Layout.columnSpan: 2
                }
            }
        }

        // Preview
        GroupBox {
            title: "Preview"
            Layout.fillWidth: true

            ColumnLayout {
                anchors.fill: parent

                Label {
                    text: "Output will be saved as:"
                    font.bold: true
                }

                TextArea {
                    id: previewText
                    readOnly: true
                    text: generatePreview()
                    Layout.fillWidth: true
                    wrapMode: Text.Wrap
                    background: Rectangle {
                        color: Material.color(Material.Grey, Material.Shade800)
                        border.color: Material.color(Material.Grey, Material.Shade600)
                        border.width: 1
                        radius: 4
                    }
                }
            }
        }

        // Action buttons
        RowLayout {
            Layout.alignment: Qt.AlignCenter

            Button {
                text: controller && controller.is_processing ? "⏸️ Converting..." : "🚀 Start Conversion"
                highlighted: true
                enabled: controller && controller.current_folder !== "" && !controller.is_processing
                onClicked: {
                    if (controller && !controller.is_processing) {
                        controller.start_conversion()
                    }
                }
            }

            Button {
                text: "❌ Cancel"
                enabled: controller && controller.is_processing
                onClicked: {
                    if (controller) {
                        controller.cancel_conversion()
                    }
                }
            }
        }

        // Progress and logs
        GroupBox {
            title: "Progress & Logs"
            Layout.fillWidth: true
            Layout.fillHeight: true

            ScrollView {
                anchors.fill: parent
                clip: true

                TextArea {
                    id: logArea
                    readOnly: true
                    wrapMode: Text.Wrap
                    font.family: "Monospace"
                    font.pixelSize: 12
                    placeholderText: "Conversion logs will appear here..."
                    background: Rectangle {
                        color: Material.color(Material.Grey, Material.Shade900)
                        border.color: Material.color(Material.Grey, Material.Shade700)
                        border.width: 1
                    }
                }
            }
        }
    }

    FolderDialog {
        id: outputDirDialog
        title: "Choose Output Directory"

        onAccepted: {
            var urlString = selectedFolder.toString()
            // Remove file:// prefix if present
            if (urlString.startsWith("file://")) {
                urlString = urlString.substring(7)
            }
            outputDirField.text = urlString
        }
    }

    function generatePreview() {
        if (!controller) return "No controller"

        var template = filenameTemplate.currentText
        var author = controller.metadata_author || "Unknown Author"
        var title = controller.metadata_title || "Unknown Title"
        var series = controller.metadata_series || ""

        // Simple template replacement
        var result = template
            .replace("{Author}", author)
            .replace("{Title}", title)
            .replace("{Series}", series)
            .replace("{SeriesNumber}", "1")

        return outputDirField.text + "/" + result
    }

    Connections {
        target: controller

        function onLog_message(message) {
            logArea.text += message + "\n"
            // Auto-scroll to bottom
            logArea.cursorPosition = logArea.length
        }

        function onConversion_completed() {
            logArea.text += "🎉 Conversion completed successfully!\n"
        }

        function onStatus_changed() {
            if (controller && controller.status_message) {
                logArea.text += controller.status_message + "\n"
            }
        }
    }
}