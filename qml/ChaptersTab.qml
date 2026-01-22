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
            title: "Chapter List"
            Layout.fillWidth: true
            Layout.fillHeight: true

            ListView {
                id: chapterList
                anchors.fill: parent
                clip: true
                spacing: 4
                
                // Parse JSON string to object model
                model: controller && controller.chapters_json ? JSON.parse(controller.chapters_json) : []
                
                delegate: Pane {
                    width: chapterList.width
                    padding: 8
                    
                    Material.elevation: 2
                    
                    background: Rectangle {
                        color: Material.color(Material.Grey, Material.Shade800)
                        radius: 4
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
                                Layout.fillWidth: true
                                elide: Text.ElideMiddle
                                font.pixelSize: 14
                            }
                            
                            Label {
                                text: modelData.path
                                Layout.fillWidth: true
                                elide: Text.ElideMiddle
                                font.pixelSize: 10
                                opacity: 0.6
                            }
                        }
                        
                        ToolButton {
                            text: "▶️" 
                            ToolTip.text: "Play Preview"
                            ToolTip.visible: hovered
                            onClicked: controller.playing_chapter_index = modelData.index
                        }

                        ToolButton {
                            text: "⏹️"
                            ToolTip.text: "Stop"
                            ToolTip.visible: hovered
                            onClicked: controller.playback_stop_trigger = true
                        }
                    }
                }
                
                ScrollBar.vertical: ScrollBar {}
            }
        }
    }
}