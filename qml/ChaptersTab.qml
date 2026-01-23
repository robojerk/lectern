import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

Item {
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        RowLayout {
            Label {
                text: "Chapters"
                font.bold: true
                font.pixelSize: 18
            }

            Item { Layout.fillWidth: true }

            Button {
                text: "Get from Audible"
                enabled: controller && controller.metadata_asin !== ""
                onClicked: {
                    // TODO: Implement chapter lookup from Audible
                }
            }

            Button {
                text: "Scan Folder"
                enabled: controller && controller.current_folder !== ""
                onClicked: {
                    if (controller) {
                        controller.scan_chapters()
                    }
                }
            }

            Button {
                text: "Add Chapter"
                onClicked: {
                    // TODO: Implement add chapter functionality
                }
            }

            Button {
                text: "Shift All +1s"
                onClicked: {
                    // TODO: Implement global time shift
                }
            }
        }

        GroupBox {
            title: "Chapter List"
            Layout.fillWidth: true
            Layout.fillHeight: true

            ScrollView {
                anchors.fill: parent
                clip: true

                ListView {
                    model: controller ? controller.chapters : []
                    spacing: 4

                    delegate: Rectangle {
                        width: ListView.view.width
                        height: 60
                        color: index % 2 === 0 ?
                            Material.color(Material.Grey, Material.Shade900) :
                            Material.color(Material.Grey, Material.Shade800)
                        radius: 4

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 10
                            spacing: 12

                            Label {
                                text: (index + 1).toString()
                                font.bold: true
                                Layout.preferredWidth: 30
                            }

                            TextField {
                                text: modelData.title || ""
                                Layout.fillWidth: true
                                background: Rectangle {
                                    color: "transparent"
                                    border.width: 0
                                }
                            }

                            Label {
                                text: {
                                    var startTime = modelData.start_time || 0;
                                    var endTime = modelData.end_time || 0;
                                    var duration = endTime - startTime;
                                    return Math.floor(duration / 60) + ":" + (duration % 60).toString().padStart(2, '0');
                                }
                                opacity: 0.7
                                Layout.preferredWidth: 60
                            }

                            CheckBox {
                                text: "Lock"
                                checked: modelData.locked || false
                                Layout.preferredWidth: 60
                            }

                            Button {
                                text: "▶️"
                                font.pixelSize: 12
                                Layout.preferredWidth: 40
                                ToolTip.text: "Play chapter"
                                ToolTip.visible: hovered
                            }
                        }
                    }
                }
            }
        }

        Label {
            text: "Chapters will be auto-detected from MP3 files or folder structure"
            font.pixelSize: 11
            opacity: 0.6
            Layout.fillWidth: true
        }
    }
}