import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import QtQuick.Window 2.15

Window {
    id: window
    visible: true
    width: 900
    height: 700
    title: "Lectern - Audiobook Processor"

    Material.theme: Material.Dark
    Material.accent: Material.Purple
    Material.primary: Material.DeepPurple
    Material.background: Material.color(Material.Grey, Material.Shade900)
    Material.foreground: Material.color(Material.Grey, Material.Shade50)

    // Drop area for files
    DropArea {
        id: dropArea
        anchors.fill: parent

        onDropped: {
            if (drop.hasUrls) {
                var url = drop.urls[0]
                console.log("Dropped file:", url)
                if (controller) {
                    controller.set_folder_path(url)
                }
            }
        }

        Rectangle {
            id: dropIndicator
            anchors.fill: parent
            color: dropArea.containsDrag ? Material.color(Material.Purple, Material.Shade200).alpha(0.1) : "transparent"
            border.color: dropArea.containsDrag ? Material.accent : "transparent"
            border.width: 2
            radius: 8
            visible: dropArea.containsDrag

            Label {
                anchors.centerIn: parent
                text: "Drop audiobook files here 📁"
                font.pixelSize: 24
                color: Material.accent
                opacity: 0.8
            }
        }
    }

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
                    icon.source: "qrc:/icons/settings.png"
                    text: "⚙"
                    font.pixelSize: 16
                    onClicked: settingsDialog.open()

                    ToolTip {
                        text: "Settings"
                        delay: 500
                    }
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
            MetadataTab {
                controller: window.controller
            }

            // Cover tab
            CoverTab {
                controller: window.controller
            }

            // Chapters tab
            ChaptersTab {
                controller: window.controller
            }

            // Convert tab
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

                // Status icon
                Label {
                    text: controller && controller.is_processing ? "⏳" :
                          controller && controller.status_message.includes("✓") ? "✓" :
                          controller && controller.status_message.includes("❌") ? "❌" : "ℹ️"
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

        ColumnLayout {
            spacing: 16
            width: parent.width

            Label {
                text: "Configure your Audiobookshelf server connection:"
                font.pixelSize: 14
                opacity: 0.8
            }

            TextField {
                id: hostField
                placeholderText: "https://abs.yourdomain.com"
                text: controller ? controller.abs_host : ""
                Layout.fillWidth: true

                Label {
                    text: "Server URL"
                    anchors.top: parent.top
                    anchors.topMargin: -20
                    font.pixelSize: 11
                    opacity: 0.7
                }
            }

            TextField {
                id: tokenField
                placeholderText: "Your API token"
                text: controller ? controller.abs_token : ""
                echoMode: TextInput.Password
                Layout.fillWidth: true

                Label {
                    text: "API Token"
                    anchors.top: parent.top
                    anchors.topMargin: -20
                    font.pixelSize: 11
                    opacity: 0.7
                }
            }

            TextField {
                id: libraryField
                placeholderText: "Library UUID"
                text: controller ? controller.abs_library_id : ""
                Layout.fillWidth: true

                Label {
                    text: "Library ID"
                    anchors.top: parent.top
                    anchors.topMargin: -20
                    font.pixelSize: 11
                    opacity: 0.7
                }
            }
        }

        onAccepted: {
            if (controller) {
                controller.abs_host = hostField.text
                controller.abs_token = tokenField.text
                controller.abs_library_id = libraryField.text
                controller.save_config()
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

    Component.onCompleted: {
        console.log("Lectern QML UI loaded successfully!")
        console.log("Window visible:", visible)
        console.log("Window size:", width, "x", height)
        console.log("Window position:", x, ",", y)
        show()
        console.log("Called show() - window should be visible now")
        raise()
        console.log("Called raise() - window should be on top")
    }
}