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
                text: "Scan Folder"
                enabled: controller && controller.current_folder !== ""
                onClicked: {
                    if (controller) {
                        controller.scan_chapters()
                    }
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
                    model: ListModel {
                        ListElement { number: 1; title: "Chapter 1"; duration: "15:23" }
                        ListElement { number: 2; title: "Chapter 2"; duration: "18:45" }
                        ListElement { number: 3; title: "Chapter 3"; duration: "22:12" }
                    }

                    spacing: 4
                    delegate: Rectangle {
                        width: ListView.view.width
                        height: 50
                        color: index % 2 === 0 ? 
                            Material.color(Material.Grey, Material.Shade900) :
                            Material.color(Material.Grey, Material.Shade800)
                        radius: 4

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 10
                            spacing: 12

                            Label {
                                text: model.number
                                font.bold: true
                                Layout.preferredWidth: 30
                            }

                            TextField {
                                text: model.title
                                Layout.fillWidth: true
                                background: Rectangle {
                                    color: "transparent"
                                    border.width: 0
                                }
                            }

                            Label {
                                text: model.duration
                                opacity: 0.7
                                Layout.preferredWidth: 60
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