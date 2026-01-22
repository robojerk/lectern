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

        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            Label {
                text: "Chapters"
                font.bold: true
                font.pixelSize: 18
            }

            Item { Layout.fillWidth: true }

            Button {
                text: "Scan Folder"
                enabled: controller && controller.current_folder !== ""
                onClicked: {
                    if (controller) {
                        controller.scan_chapters_trigger = true
                    }
                }
            }
        }

        ListView {
            id: chapterList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: 4

            // Parse JSON from Rust controller
            model: controller && controller.chapters_json ? JSON.parse(controller.chapters_json) : []

            delegate: Pane {
                width: chapterList.width
                padding: 8
                
                Material.elevation: 1
                
                background: Rectangle {
                    color: Material.color(Material.Grey, Material.Shade800)
                    radius: 4
                    border.width: 0
                }

                RowLayout {
                    anchors.fill: parent
                    spacing: 12

                    Label {
                        text: (modelData.index + 1)
                        font.bold: true
                        color: Material.accent
                        Layout.preferredWidth: 30
                        horizontalAlignment: Text.AlignRight
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2

                        Label {
                            text: modelData.title
                            font.bold: true
                            Layout.fillWidth: true
                            elide: Text.ElideMiddle
                        }

                        Label {
                            text: modelData.path
                            opacity: 0.6
                            font.pixelSize: 10
                            Layout.fillWidth: true
                            elide: Text.ElideMiddle
                        }
                    }

                    ToolButton {
                        text: "▶️"
                        ToolTip.text: "Play Preview"
                        ToolTip.visible: hovered
                        onClicked: { 
                            if(controller) controller.playing_chapter_index = modelData.index 
                        }
                    }

                    ToolButton {
                        text: "⏹️"
                        ToolTip.text: "Stop"
                        ToolTip.visible: hovered
                        onClicked: { 
                            if(controller) controller.playback_stop_trigger = true 
                        }
                    }
                }
            }
        }

        Label {
            text: "Chapters will be automatically detected from folder structure or can be manually added"
            font.pixelSize: 12
            opacity: 0.6
            Layout.fillWidth: true
            wrapMode: Text.Wrap
        }
    }
}