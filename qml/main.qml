import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

ApplicationWindow {
    id: window
    visible: true
    width: 900
    height: 700
    title: "Lectern - Audiobook Tool"

    Material.theme: Material.Dark
    Material.accent: Material.Purple
    Material.primary: Material.DeepPurple

    // Main content area
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header bar with branding and settings
        Rectangle {
            Layout.fillWidth: true
            height: 56
            color: Material.color(Material.Grey, Material.Shade800)

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 16

                // Logo/Brand
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

                // Current folder indicator
                Label {
                    text: controller ? controller.current_folder : "No folder selected"
                    opacity: 0.7
                    elide: Text.ElideMiddle
                    Layout.maximumWidth: 300
                }

                // Settings button
                ToolButton {
                    text: "⚙"
                    font.pixelSize: 16
                    onClicked: settingsDialog.open()

                    ToolTip.visible: hovered
                    ToolTip.text: "Settings"
                    ToolTip.delay: 500
                }
            }
        }

        // Tab bar with modern styling
        TabBar {
            id: tabBar
            Layout.fillWidth: true
            background: Rectangle { color: Material.color(Material.Grey, Material.Shade800) }

            TabButton {
                text: "📁 Metadata"
                font.pixelSize: 14
                width: implicitWidth
            }
            TabButton {
                text: "🖼️ Cover"
                font.pixelSize: 14
                width: implicitWidth
            }
            TabButton {
                text: "📑 Chapters"
                font.pixelSize: 14
                width: implicitWidth
            }
            TabButton {
                text: "🔄 Convert"
                font.pixelSize: 14
                width: implicitWidth
            }
        }

        // Tab content area
        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: tabBar.currentIndex

            // Metadata tab
            Item {
                MetadataTab {
                    anchors.fill: parent
                    controller: window.controller
                }
            }

            // Cover tab
            Item {
                CoverTab {
                    anchors.fill: parent
                    controller: window.controller
                }
            }

            // Chapters tab
            Item {
                ChaptersTab {
                    anchors.fill: parent
                    controller: window.controller
                }
            }

            // Convert tab
            Item {
                ConvertTab {
                    anchors.fill: parent
                    controller: window.controller
                }
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

                // Status icon
                Label {
                    text: controller && controller.is_processing ? "⏳" :
                          controller && controller.status_message.indexOf("✓") !== -1 ? "✓" :
                          controller && controller.status_message.indexOf("❌") !== -1 ? "❌" : "ℹ️"
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
                text: "Configure your Audiobookshelf server connection:"
                font.pixelSize: 14
                opacity: 0.8
            }

            ColumnLayout {
                spacing: 4
                Layout.fillWidth: true

                Label {
                    text: "Server URL"
                    font.pixelSize: 11
                    opacity: 0.7
                }

                TextField {
                    id: hostField
                    placeholderText: "https://abs.yourdomain.com"
                    text: controller ? controller.abs_host : ""
                    Layout.fillWidth: true
                }
            }

            ColumnLayout {
                spacing: 4
                Layout.fillWidth: true

                Label {
                    text: "API Token"
                    font.pixelSize: 11
                    opacity: 0.7
                }

                TextField {
                    id: tokenField
                    placeholderText: "Your API token"
                    text: controller ? controller.abs_token : ""
                    echoMode: TextInput.Password
                    Layout.fillWidth: true
                }
            }

            ColumnLayout {
                spacing: 4
                Layout.fillWidth: true

                Label {
                    text: "Library ID"
                    font.pixelSize: 11
                    opacity: 0.7
                }

                TextField {
                    id: libraryField
                    placeholderText: "Library UUID"
                    text: controller ? controller.abs_library_id : ""
                    Layout.fillWidth: true
                }
            }
        }

        onAccepted: {
            if (controller) {
                controller.abs_host = hostField.text
                controller.abs_token = tokenField.text
                controller.abs_library_id = libraryField.text
                controller.save_config_trigger = true
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
            text: ""
            wrapMode: Text.Wrap
            width: parent.width
        }
    }

    // Connections to controller signals
    Connections {
        target: controller

        function onError_occurred(message) {
            errorLabel.text = message
            errorDialog.open()
        }

        function onLog_message(message) {
            console.log("LECTERN:", message)
        }

        function onMetadata_changed() {
            console.log("Metadata changed")
        }

        function onConversion_completed() {
            console.log("Conversion completed")
        }
    }
}