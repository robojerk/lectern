import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import QtQuick.Window 2.15

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
            height: 56
            color: Material.color(Material.Grey, Material.Shade800)

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 16

                // Logo
                RowLayout {
                    spacing: 12

                    Rectangle {
                        width: 32
                        height: 32
                        radius: 6
                        color: Material.accent

                        Label {
                            anchors.centerIn: parent
                            text: "🎵"
                            font.pixelSize: 16
                        }
                    }

                    Label {
                        text: "Lectern"
                        font.bold: true
                        font.pixelSize: 20
                        color: Material.primaryColor
                    }
                }

                Item { Layout.fillWidth: true }

                // Current folder
                Label {
                    text: controller ? (controller.current_folder || "No folder selected") : "No folder selected"
                    opacity: 0.7
                    elide: Text.ElideMiddle
                    Layout.maximumWidth: 300
                }

                // Settings button
                ToolButton {
                    text: "⚙"
                    font.pixelSize: 16
                    onClicked: settingsDialog.open()
                }
            }
        }

        // Tab bar
        TabBar {
            id: tabBar
            Layout.fillWidth: true
            background: Rectangle { color: Material.color(Material.Grey, Material.Shade800) }

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
                    text: controller ? controller.status_message : "Ready"
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                    font.pixelSize: 12
                }

                ProgressBar {
                    visible: controller ? controller.is_processing : false
                    value: controller ? controller.progress_value : 0.0
                    Layout.preferredWidth: 200
                    height: 8
                }
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
            spacing: 16
            width: parent.width

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
        modal: true
        width: 400
        anchors.centerIn: parent

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

    Component.onCompleted: {
        console.log("Lectern loaded")
    }
}