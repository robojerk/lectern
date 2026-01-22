import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

Item {
    property var controller

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 20

        GroupBox {
            title: "Chapter Management"
            Layout.fillWidth: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 12

                Label {
                    text: "🎵 Chapters will be automatically generated during conversion"
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }

                Label {
                    text: "📁 Current audiobook: " + (controller && controller.current_folder ?
                          controller.current_folder.split('/').pop() : "No folder selected")
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                    opacity: 0.8
                }
            }
        }

        GroupBox {
            title: "Chapter Controls"
            Layout.fillWidth: true

            RowLayout {
                anchors.fill: parent
                spacing: 12

                Button {
                    text: "➕ Add Chapter"
                    enabled: false
                    ToolTip.text: "Manual chapter editing coming soon"
                    ToolTip.visible: hovered
                }

                Button {
                    text: "🎵 Auto-Detect"
                    enabled: false
                    ToolTip.text: "Automatic chapter detection from file names coming soon"
                    ToolTip.visible: hovered
                }

                Button {
                    text: "🌐 Get from Audible"
                    enabled: false
                    ToolTip.text: "Fetch chapter data from Audible.com coming soon"
                    ToolTip.visible: hovered
                }

                Item { Layout.fillWidth: true }

                Label { text: "Global Shift (seconds):" }
                SpinBox {
                    id: shiftSpinBox
                    from: -3600
                    to: 3600
                    value: 0
                    enabled: false
                    ToolTip.text: "Time shifting coming soon"
                    ToolTip.visible: hovered
                }

                Button {
                    text: "🔄 Apply Shift"
                    enabled: false
                    ToolTip.text: "Time shifting coming soon"
                    ToolTip.visible: hovered
                }
            }
        }

        GroupBox {
            title: "Preview"
            Layout.fillWidth: true
            Layout.fillHeight: true

            ScrollView {
                anchors.fill: parent
                clip: true

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 8

                    Label {
                        text: "📖 Chapter Preview"
                        font.bold: true
                        font.pixelSize: 16
                    }

                    Label {
                        text: "Chapters will be automatically created based on:"
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                        opacity: 0.8
                    }

                    Label {
                        text: "• Audio file boundaries (if multiple files)"
                        font.pixelSize: 12
                        Layout.fillWidth: true
                        opacity: 0.7
                    }

                    Label {
                        text: "• Silence detection in audio stream"
                        font.pixelSize: 12
                        Layout.fillWidth: true
                        opacity: 0.7
                    }

                    Label {
                        text: "• Manual chapter markers (future feature)"
                        font.pixelSize: 12
                        Layout.fillWidth: true
                        opacity: 0.7
                    }

                    Item { Layout.fillHeight: true }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: Material.color(Material.Grey, Material.Shade600)
                    }

                    Label {
                        text: "🚀 Ready for conversion! Chapters will be generated automatically."
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                        color: Material.accent
                        font.bold: true
                    }
                }
            }
        }
    }
}