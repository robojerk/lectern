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

        // Header bar with prominent drag instruction
        Rectangle {
            Layout.fillWidth: true
            height: 80
            color: Material.color(Material.Grey, Material.Shade800)

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 4

                Label {
                    text: "🎵 Welcome to Lectern"
                    font.bold: true
                    font.pixelSize: 18
                    color: Material.primaryColor
                }

                Label {
                    text: "Drag and drop an audiobook folder below, or use the browse button to get started"
                    opacity: 0.8
                    font.pixelSize: 12
                    color: Material.foreground
                }
            }
        }

        // Tab bar
        TabBar {
            id: tabBar
            Layout.fillWidth: true
            background: Rectangle { color: Material.color(Material.Grey, Material.Shade800) }
            currentIndex: 0  // Force start on Metadata tab

            TabButton {
                text: "📁 Metadata"
                font.pixelSize: 14
            }
            TabButton {
                text: "🖼️ Cover"
                font.pixelSize: 14
            }
            TabButton {
                text: "📑 Chapters"
                font.pixelSize: 14
            }
            TabButton {
                text: "🔄 Convert"
                font.pixelSize: 14
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
            height: 32
            color: Material.color(Material.Grey, Material.Shade800)
            border.color: Material.color(Material.Grey, Material.Shade700)
            border.width: 1

            RowLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 12

                Label {
                    text: controller && controller.is_processing ? "⏳" : "ℹ️"
                    font.pixelSize: 14
                }

                Label {
                    text: controller ? controller.status_message : "Ready - Drag an audiobook folder to begin"
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                    font.pixelSize: 12
                }
            }
        }
    }

    // Menu bar
    menuBar: MenuBar {
        Menu {
            title: "&File"
            MenuItem {
                text: "&Settings..."
                onTriggered: settingsDialog.open()
            }
            MenuSeparator {}
            MenuItem {
                text: "&Quit"
                onTriggered: Qt.quit()
            }
        }
    }

    // Settings dialog
    Dialog {
        id: settingsDialog
        title: "Audiobookshelf Settings"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true
        width: 500
        anchors.centerIn: parent

        ColumnLayout {
            anchors.fill: parent
            spacing: 16

            Label {
                text: "Configure your Audiobookshelf server:"
                opacity: 0.8
            }

            Label { text: "Server URL"; opacity: 0.7; font.pixelSize: 11 }
            TextField {
                id: hostField
                Layout.fillWidth: true
                text: controller ? controller.abs_host : ""
                placeholderText: "https://abs.yourdomain.com"
            }

            Label { text: "API Token"; opacity: 0.7; font.pixelSize: 11 }
            TextField {
                id: tokenField
                Layout.fillWidth: true
                text: controller ? controller.abs_token : ""
                echoMode: TextInput.Password
            }

            Label { text: "Library ID"; opacity: 0.7; font.pixelSize: 11 }
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

    // Error dialog
    Dialog {
        id: errorDialog
        title: "Error"
        standardButtons: Dialog.Ok
        width: 400
        anchors.centerIn: parent
        modal: true

        Label {
            id: errorLabel
            wrapMode: Text.Wrap
            width: parent.width
        }
    }

    Connections {
        target: controller

        function onError_occurred(message) {
            errorLabel.text = message
            errorDialog.open()
        }

        function onLog_message(message) {
            console.log("LECTERN:", message)
        }
    }
}