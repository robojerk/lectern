import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import QtQuick.Dialogs 1.3

ApplicationWindow {
    id: window
    visible: true
    width: 900
    height: 700
    title: "Lectern - Audiobook Tool"

    Material.theme: Material.Dark
    Material.accent: Material.Purple
    Material.primary: Material.DeepPurple
    Material.background: Material.color(Material.Grey, Material.Shade900)
    Material.foreground: Material.color(Material.Grey, Material.Shade50)

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header bar
        Rectangle {
            Layout.fillWidth: true
            height: 60
            color: Material.color(Material.Grey, Material.Shade800)

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 16

                Label {
                    text: "🎵 Lectern"
                    font.bold: true
                    font.pixelSize: 20
                    color: Material.primaryColor
                }

                Label {
                    text: "Audiobook Tool"
                    opacity: 0.7
                    font.pixelSize: 14
                    color: Material.foreground
                }

                Item { Layout.fillWidth: true }

                Label {
                    text: controller ? (controller.current_folder || "No folder selected") : "No folder selected"
                    opacity: 0.7
                    elide: Text.ElideMiddle
                    Layout.maximumWidth: 300
                }

                ToolButton {
                    text: "⚙"
                    onClicked: settingsDialog.open()
                }
            }
        }

        // Tab bar
        TabBar {
            id: tabBar
            Layout.fillWidth: true
            currentIndex: 0

            TabButton {
                text: "📁 Metadata"
            }
            TabButton {
                text: "🖼️ Cover"
            }
            TabButton {
                text: "📑 Chapters"
            }
            TabButton {
                text: "🔄 Convert"
            }
        }

        // Tab content
        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: tabBar.currentIndex

            MetadataTab {
                controller: window.controller
            }

            CoverTab {
                controller: window.controller
            }

            ChaptersTab {
                controller: window.controller
            }

            ConvertTab {
                controller: window.controller
            }
        }

        // Status bar
        Rectangle {
            Layout.fillWidth: true
            height: 30
            color: Material.color(Material.Grey, Material.Shade800)

            RowLayout {
                anchors.fill: parent
                anchors.margins: 8

                Label {
                    text: controller ? controller.status_message : "Ready"
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                }
            }
        }
    }

    // Settings dialog
    Dialog {
        id: settingsDialog
        title: "Audiobookshelf Settings"
        standardButtons: Dialog.Ok | Dialog.Cancel
        width: 500
        modal: true

        ColumnLayout {
            width: parent.width
            spacing: 16

            Label { text: "Server URL:" }
            TextField {
                id: hostField
                Layout.fillWidth: true
                text: controller ? controller.abs_host : ""
                placeholderText: "https://abs.yourdomain.com"
            }

            Label { text: "API Token:" }
            TextField {
                id: tokenField
                Layout.fillWidth: true
                text: controller ? controller.abs_token : ""
                echoMode: TextInput.Password
            }

            Label { text: "Library ID:" }
            TextField {
                id: libraryField
                Layout.fillWidth: true
                text: controller ? controller.abs_library_id : ""
            }
        }

        onAccepted: {
            if (controller) {
                controller.save_config(hostField.text, tokenField.text, libraryField.text)
            }
        }
    }

    Connections {
        target: controller

        function onError_occurred(message) {
            console.log("Error:", message)
        }

        function onLog_message(message) {
            console.log("Log:", message)
        }
    }
}