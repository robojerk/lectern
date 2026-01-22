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

        // Chapter controls
        RowLayout {
            Button {
                text: "➕ Add Chapter"
                onClicked: addChapterDialog.open()
            }

            Button {
                text: "🎵 Auto-Detect"
                enabled: false
                ToolTip.text: "Coming soon: Auto-detect chapters from file names"
                ToolTip.visible: hovered
            }

            Button {
                text: "🌐 Get from Audible"
                enabled: false
                ToolTip.text: "Coming soon: Fetch chapter data from Audible"
                ToolTip.visible: hovered
            }

            Item { Layout.fillWidth: true }

            Label { text: "Global Shift (seconds):" }
            SpinBox {
                id: shiftSpinBox
                from: -3600
                to: 3600
                value: 0
            }

            Button {
                text: "🔄 Apply Shift"
                enabled: false
                ToolTip.text: "Coming soon: Shift all chapter times"
                ToolTip.visible: hovered
            }
        }

        // Chapters list
        GroupBox {
            title: "Chapters (Coming Soon)"
            Layout.fillWidth: true
            Layout.fillHeight: true

            ScrollView {
                anchors.fill: parent
                clip: true

                ListView {
                    id: chaptersList
                    anchors.fill: parent
                    spacing: 2

                    model: ListModel {
                        id: chaptersModel
                        // Placeholder data
                        ListElement { title: "Chapter 1"; startTime: 0; locked: false }
                        ListElement { title: "Chapter 2"; startTime: 1200; locked: false }
                        ListElement { title: "Chapter 3"; startTime: 2400; locked: true }
                    }

                    delegate: Rectangle {
                        width: chaptersList.width
                        height: 50
                        color: index % 2 === 0 ? Material.color(Material.Grey, Material.Shade900)
                                               : Material.color(Material.Grey, Material.Shade800)

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 5
                            spacing: 10

                            // Lock button
                            ToolButton {
                                text: model.locked ? "🔒" : "🔓"
                                enabled: false
                            }

                            // Title field
                            TextField {
                                text: model.title
                                Layout.fillWidth: true
                                enabled: false
                            }

                            // Time field
                            TextField {
                                text: formatTime(model.startTime)
                                width: 100
                                validator: DoubleValidator { bottom: 0 }
                                enabled: false
                            }

                            // Play button
                            ToolButton {
                                text: "▶️"
                                enabled: false
                            }

                            // Remove button
                            ToolButton {
                                text: "🗑️"
                                enabled: false
                            }
                        }
                    }
                }
            }
        }

        // Playback controls (for chapter preview)
        GroupBox {
            title: "Chapter Preview (Coming Soon)"
            Layout.fillWidth: true

            RowLayout {
                anchors.fill: parent

                Label {
                    id: playbackStatus
                    text: "Chapter management coming soon"
                    Layout.fillWidth: true
                }

                Button {
                    text: "⏸️"
                    enabled: false
                }

                Button {
                    text: "⏹️"
                    enabled: false
                }
            }
        }

        Label {
            text: "Note: Chapter management features are planned for a future release"
            opacity: 0.6
            font.italic: true
            Layout.fillWidth: true
        }
    }

    // Add chapter dialog
    Dialog {
        id: addChapterDialog
        title: "Add Chapter"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true
        anchors.centerIn: parent

        ColumnLayout {
            spacing: 10

            TextField {
                id: chapterTitleField
                placeholderText: "Chapter title"
                Layout.fillWidth: true
            }

            TextField {
                id: chapterTimeField
                placeholderText: "Start time (seconds)"
                validator: DoubleValidator { bottom: 0 }
                Layout.fillWidth: true
            }
        }

        onAccepted: {
            // TODO: Implement chapter adding
            console.log("Would add chapter:", chapterTitleField.text, "at", chapterTimeField.text)

            // Reset fields
            chapterTitleField.text = ""
            chapterTimeField.text = ""
        }
    }

    function formatTime(seconds) {
        var hours = Math.floor(seconds / 3600)
        var minutes = Math.floor((seconds % 3600) / 60)
        var secs = Math.floor(seconds % 60)
        var ms = Math.floor((seconds % 1) * 1000)

        if (hours > 0) {
            return hours + ":" +
                   minutes.toString().padStart(2, '0') + ":" +
                   secs.toString().padStart(2, '0') + "." +
                   ms.toString().padStart(3, '0')
        } else {
            return minutes + ":" +
                   secs.toString().padStart(2, '0') + "." +
                   ms.toString().padStart(3, '0')
        }
    }
}