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

        GroupBox {
            title: "Current Cover"
            Layout.fillWidth: true
            Layout.preferredHeight: 300

            Rectangle {
                anchors.fill: parent
                color: Material.color(Material.Grey, Material.Shade800)
                border.color: Material.color(Material.Grey, Material.Shade600)
                border.width: 1
                radius: 4

                Image {
                    id: coverImage
                    anchors.centerIn: parent
                    width: 200
                    height: 280
                    fillMode: Image.PreserveAspectFit
                    source: "" // TODO: Bind to cover URL from metadata
                    asynchronous: true

                    onStatusChanged: {
                        if (status === Image.Error) {
                            errorText.visible = true
                        }
                    }
                }

                Label {
                    id: errorText
                    anchors.centerIn: parent
                    text: "No cover image"
                    visible: coverImage.source === ""
                    opacity: 0.6
                }

                DropArea {
                    anchors.fill: parent
                    onDropped: function(drop) {
                        if (drop.hasUrls && drop.urls.length > 0) {
                            coverImage.source = drop.urls[0]
                        }
                    }
                }
            }
        }

        RowLayout {
            Button {
                text: "📁 Choose File..."
                onClicked: fileDialog.open()
            }

            Button {
                text: "🌐 Search Online"
                enabled: false // TODO: Implement cover search
            }

            Button {
                text: "🗑️ Clear"
                onClicked: coverImage.source = ""
            }

            Item { Layout.fillWidth: true }
        }

        GroupBox {
            title: "Cover Search"
            Layout.fillWidth: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 10

                RowLayout {
                    TextField {
                        id: coverSearchField
                        placeholderText: "Search for covers by title, author, or ISBN"
                        Layout.fillWidth: true
                    }

                    Button {
                        text: "🔍 Search"
                        enabled: false // TODO: Implement cover search
                    }
                }

                Label {
                    text: "Cover search will show thumbnail grid for selection"
                    opacity: 0.6
                    Layout.fillWidth: true
                }
            }
        }

        Item { Layout.fillHeight: true }
    }

    FileDialog {
        id: fileDialog
        title: "Choose Cover Image"
        nameFilters: ["Image files (*.jpg *.jpeg *.png *.webp)"]

        onAccepted: {
            coverImage.source = selectedFile
        }
    }
}