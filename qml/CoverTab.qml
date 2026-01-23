import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1

Item {
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 20

        Label {
            text: "Cover Art"
            font.bold: true
            font.pixelSize: 18
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Material.color(Material.Grey, Material.Shade800)
            radius: 8

            Image {
                id: coverImage
                anchors.centerIn: parent
                source: controller ? controller.metadata_cover_url : ""
                fillMode: Image.PreserveAspectFit
                width: Math.min(parent.width - 40, 350)
                height: Math.min(parent.height - 40, 500)

                Rectangle {
                    anchors.fill: parent
                    color: "transparent"
                    visible: parent.status !== Image.Ready

                    Label {
                        anchors.centerIn: parent
                        text: {
                            if (parent.parent.status === Image.Loading) return "Loading..."
                            if (!controller || controller.metadata_cover_url === "") return "No cover"
                            return "Failed to load"
                        }
                        opacity: 0.5
                    }
                }
            }
        }

        // Cover search section
        GroupBox {
            title: "Search Cover Art"
            Layout.fillWidth: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 12

                RowLayout {
                    TextField {
                        id: coverSearchField
                        placeholderText: "Search by title or author..."
                        Layout.fillWidth: true
                        text: controller ? (controller.metadata_title + " " + controller.metadata_author) : ""
                    }

                    ComboBox {
                        id: coverProviderCombo
                        model: [
                            { text: "Google Books", value: "google" },
                            { text: "Audnexus", value: "audnexus" },
                            { text: "Open Library", value: "openlibrary" }
                        ]
                        textRole: "text"
                        valueRole: "value"
                        currentIndex: 0
                        Layout.preferredWidth: 120
                    }

                    Button {
                        text: "🔍 Search"
                        highlighted: true
                        enabled: controller && !controller.is_processing && coverSearchField.text !== ""
                        onClicked: {
                            if (controller) {
                                controller.search_cover_art(coverSearchField.text, coverProviderCombo.currentValue)
                            }
                        }
                    }
                }

                // Cover search results
                ScrollView {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 120
                    visible: controller && controller.cover_search_urls.length > 0

                    Row {
                        spacing: 12
                        Repeater {
                            model: controller ? controller.cover_search_urls : []

                            Rectangle {
                                width: 80
                                height: 120
                                color: Material.color(Material.Grey, Material.Shade700)
                                radius: 4

                                Image {
                                    anchors.fill: parent
                                    anchors.margins: 4
                                    source: modelData
                                    fillMode: Image.PreserveAspectFit
                                    asynchronous: true
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: {
                                        if (controller) {
                                            controller.metadata_cover_url = modelData
                                            urlField.text = modelData
                                        }
                                    }
                                }

                                Rectangle {
                                    anchors.bottom: parent.bottom
                                    width: parent.width
                                    height: 20
                                    color: Material.color(Material.Grey, Material.Shade900)
                                    opacity: 0.8

                                    Label {
                                        anchors.centerIn: parent
                                        text: "Select"
                                        font.pixelSize: 10
                                        color: "white"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Manual cover entry
        GroupBox {
            title: "Manual Cover"
            Layout.fillWidth: true

            RowLayout {
                anchors.fill: parent

                TextField {
                    id: urlField
                    placeholderText: "Cover image URL..."
                    text: controller ? controller.metadata_cover_url : ""
                    Layout.fillWidth: true
                }

                Button {
                    text: "Load"
                    onClicked: {
                        if (controller) {
                            controller.metadata_cover_url = urlField.text
                        }
                    }
                }

                Button {
                    text: "Browse..."
                    onClicked: fileDialog.open()
                }
            }
        }
    }

    FileDialog {
        id: fileDialog
        title: "Select Cover Image"
        nameFilters: ["Images (*.jpg *.jpeg *.png)"]
        onAccepted: {
            if (controller) {
                controller.metadata_cover_url = file.toString()
            }
        }
    }
}