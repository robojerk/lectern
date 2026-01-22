import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import QtQuick.Dialogs

Item {
    property LecternController controller

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
                        text: "~/Audiobooks" // TODO: Load from settings
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
                    model: ["64 kbps", "96 kbps", "128 kbps", "192 kbps", "256 kbps"]
                    currentIndex: 2 // 128 kbps
                    Layout.fillWidth: true
                }

                CheckBox {
                    text: "Upload to Audiobookshelf"
                    checked: controller.abs_host !== "" && controller.abs_token !== ""
                    enabled: controller.abs_host !== "" && controller.abs_token !== ""
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
                }
            }
        }

        // Action buttons
        RowLayout {
            Layout.alignment: Qt.AlignCenter

            Button {
                text: controller.is_processing ? "⏸️ Pause" : "🚀 Start Conversion"
                highlighted: true
                enabled: controller.current_folder !== "" && !controller.is_processing
                onClicked: {
                    if (controller.is_processing) {
                        controller.cancel_conversion()
                    } else {
                        controller.start_conversion()
                    }
                }
            }

            Button {
                text: "❌ Cancel"
                enabled: controller.is_processing
                onClicked: controller.cancel_conversion()
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
                }
            }
        }

        Item { Layout.fillHeight: true }
    }

    FolderDialog {
        id: outputDirDialog
        title: "Choose Output Directory"

        onAccepted: {
            outputDirField.text = selectedFolder.toString().replace("file://", "")
        }
    }

    function generatePreview() {
        var template = filenameTemplate.currentText
        var author = controller.metadata_author || "Unknown Author"
        var title = controller.metadata_title || "Unknown Title"
        var series = controller.metadata_series || ""

        // Simple template replacement
        var result = template
            .replace("{Author}", author)
            .replace("{Title}", title)
            .replace("{Series}", series)
            .replace("{SeriesNumber}", "1") // TODO: Get from metadata

        return outputDirField.text + "/" + result
    }

    Connections {
        target: controller

        function onLog_message(message) {
            logArea.append(message + "\n")
            // Auto-scroll to bottom
            logArea.cursorPosition = logArea.length
        }

        function onConversion_completed() {
            logArea.append("🎉 Conversion completed successfully!\n")
        }
    }
}