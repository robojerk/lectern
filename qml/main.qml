import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import QtQuick.Window 2.15
import QtQuick.Dialogs 1.3

ApplicationWindow {
    visible: true
    width: 600
    height: 400
    title: "Lectern - Audiobook Tool"

    Material.theme: Material.Dark
    Material.accent: Material.Purple
    Material.primary: Material.DeepPurple
    Material.background: Material.color(Material.Grey, Material.Shade900)
    Material.foreground: Material.color(Material.Grey, Material.Shade50)

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 15

        // Settings button at the top
        RowLayout {
            Layout.fillWidth: true

            Item { Layout.fillWidth: true } // Spacer

            Button {
                text: "⚙ Settings"
                onClicked: settingsDialog.open()
            }
        }

        // Drag & Drop Area
        DropArea {
            id: dropArea
            Layout.fillWidth: true
            Layout.preferredHeight: 150

            Rectangle {
                anchors.fill: parent
                color: dropArea.containsDrag ? "#e0e0e0" : "#f5f5f5"
                border.color: "#cccccc"
                radius: 10

                Label {
                    anchors.centerIn: parent
                    text: dropArea.containsDrag ? "Drop Folder Now" : "Drag Audiobook Folder Here\n(or click to browse)"
                    color: "#666666"
                    font.pixelSize: 16
                    horizontalAlignment: Text.AlignHCenter
                }

                MouseArea {
                    anchors.fill: parent
                    onClicked: folderDialog.open()
                }
            }

            onEntered: {
                console.log("Drag entered drop area")
            }

            onExited: {
                console.log("Drag exited drop area")
            }

            onDropped: function(drop) {
                console.log("Drop event received:", drop.urls)
                drop.accept(Qt.CopyAction)
                if (drop.hasUrls && drop.urls.length > 0) {
                    var url = drop.urls[0]
                    console.log("Processing URL:", url.toString())
                    if (controller) {
                        controller.set_folder_path(url.toString());
                        console.log("Called controller.set_folder_path")
                    } else {
                        console.log("Controller is null!")
                    }
                } else {
                    console.log("Drop does not have URLs or no URLs in array")
                }
            }
        }

        // Search Bar
        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            TextField {
                id: searchField
                placeholderText: "Search book title or author..."
                Layout.fillWidth: true
                onAccepted: {
                    if (controller && text !== "") {
                        controller.search_metadata(text, false);
                    }
                }
            }

            Button {
                text: "🔍 Search"
                enabled: controller && !controller.is_processing && searchField.text !== ""
                onClicked: {
                    if (controller) {
                        controller.search_metadata(searchField.text, false);
                    }
                }
            }
        }

        // Search Result Display
        RowLayout {
            Layout.fillWidth: true
            visible: controller && controller.search_title !== ""
            spacing: 15

            Image {
                source: controller ? controller.search_cover_url : ""
                fillMode: Image.PreserveAspectFit
                Layout.preferredWidth: 100
                Layout.preferredHeight: 150

                // Placeholder while loading
                Rectangle {
                    anchors.fill: parent
                    color: "#eee"
                    visible: parent.status !== Image.Ready
                    Text {
                        anchors.centerIn: parent
                        text: "Loading..."
                        color: "#666"
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 5

                Label {
                    text: controller ? controller.search_title : ""
                    font.bold: true
                    font.pixelSize: 18
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }

                Label {
                    text: "Author: " + (controller ? controller.search_author : "")
                    font.italic: true
                    font.pixelSize: 14
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }

                Button {
                    text: "Use This Book"
                    Layout.alignment: Qt.AlignLeft
                    onClicked: {
                        if (controller) {
                            // Copy search results to metadata fields
                            controller.metadata_title = controller.search_title;
                            controller.metadata_author = controller.search_author;
                            controller.metadata_cover_url = controller.search_cover_url;
                            controller.metadata_changed();
                        }
                    }
                }
            }
        }

        // Folder and Conversion Section
        Label {
            text: "Path: " + (controller ? controller.current_folder : "No folder selected")
            Layout.fillWidth: true
            elide: Text.ElideMiddle
            font.pixelSize: 12
        }

        Button {
            text: controller && controller.is_processing ? "⏸️ Converting..." : "🚀 Start Conversion"
            enabled: controller && controller.current_folder !== "" && !controller.is_processing
            Layout.alignment: Qt.AlignHCenter
            onClicked: {
                if (controller) {
                    controller.start_conversion();
                }
            }
        }

        // Status message
        Label {
            text: controller ? controller.status_message : "Ready"
            Layout.fillWidth: true
            font.pixelSize: 12
            opacity: 0.8
        }

        // Progress bar
        ProgressBar {
            Layout.fillWidth: true
            value: controller ? controller.progress_value : 0.0
            visible: controller && (controller.is_processing || controller.progress_value > 0)
        }

        Item { Layout.fillHeight: true } // Spacer
    }

    // Settings Dialog
    Dialog {
        id: settingsDialog
        title: "Audiobookshelf Settings"
        standardButtons: Dialog.Save | Dialog.Cancel
        anchors.centerIn: parent
        modal: true
        width: 400

        ColumnLayout {
            spacing: 15
            width: parent.width

            Label { text: "Server URL" }
            TextField {
                id: urlField
                Layout.fillWidth: true
                placeholderText: "https://abs.yourdomain.com"
                text: controller ? controller.abs_host : ""
            }

            Label { text: "API Token" }
            TextField {
                id: tokenField
                Layout.fillWidth: true
                echoMode: TextInput.Password
                text: controller ? controller.abs_token : ""
            }

            Label { text: "Library ID" }
            TextField {
                id: libraryField
                Layout.fillWidth: true
                text: controller ? controller.abs_library_id : ""
            }
        }

        onAccepted: {
            if (controller) {
                controller.save_config(urlField.text, tokenField.text, libraryField.text);
            }
        }
    }

    // Folder selection dialog
    FolderDialog {
        id: folderDialog
        title: "Select Audiobook Folder"
        onAccepted: {
            if (controller) {
                controller.set_folder_path(selectedFolder.toString());
            }
        }
    }
}
