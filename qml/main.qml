import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1

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

        // Header bar
        Rectangle {
            Layout.fillWidth: true
            height: 56
            color: Material.color(Material.Grey, Material.Shade800)

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 16

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

                Label {
                    text: controller ? controller.current_folder : "No folder selected"
                    opacity: 0.7
                    elide: Text.ElideMiddle
                    Layout.maximumWidth: 300
                }

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

            MetadataTab { }
            CoverTab { }
            ChaptersTab { }
            ConvertTab { }
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
                    text: {
                        if (!controller) return "ℹ️"
                        if (controller.is_processing) return "⏳"
                        if (controller.status_message.indexOf("✓") >= 0) return "✓"
                        if (controller.status_message.indexOf("❌") >= 0) return "❌"
                        return "ℹ️"
                    }
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
        title: "Settings"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true
        width: 600
        height: 500
        x: (parent.width - width) / 2
        y: (parent.height - height) / 2

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true

                ColumnLayout {
                    spacing: 16
                    width: settingsDialog.width - 32

                Label {
                    text: "Audiobookshelf Server Connection:"
                    font.pixelSize: 14
                    opacity: 0.8
                }

                ColumnLayout {
                    spacing: 4

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

                Rectangle {
                    height: 1
                    color: Material.color(Material.Grey, Material.Shade700)
                    Layout.fillWidth: true
                    Layout.topMargin: 8
                    Layout.bottomMargin: 8
                }

                Label {
                    text: "Local Library (Optional):"
                    font.pixelSize: 14
                    opacity: 0.8
                }

                ColumnLayout {
                    spacing: 4

                    Label {
                        text: "Library Path"
                        font.pixelSize: 11
                        opacity: 0.7
                    }

                    RowLayout {
                        TextField {
                            id: localLibraryField
                            placeholderText: "/home/user/audiobooks"
                            text: controller ? controller.local_library_path : ""
                            Layout.fillWidth: true
                        }

                        Button {
                            text: "Browse..."
                            onClicked: localLibraryDialog.open()
                        }
                    }
                }

                ColumnLayout {
                    spacing: 4

                    Label {
                        text: "Path Template"
                        font.pixelSize: 11
                        opacity: 0.7
                    }

                    TextField {
                        id: pathTemplateField
                        placeholderText: "{Path to Local Library}/{Author}/{Title}.m4b"
                        text: controller ? controller.path_template : "{Path to Local Library}/{Author}/{Title}.m4b"
                        Layout.fillWidth: true
                    }
                }

                Label {
                    text: "Available placeholders: {Author}, {Series}, {Title}, {SeriesNumber}, {Year}, {Quality}"
                    font.pixelSize: 10
                    opacity: 0.6
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }

                Label {
                    text: "Example: {Path to Local Library}/{Author}/{Series}/Book {SeriesNumber}- {Title}.m4b"
                    font.pixelSize: 10
                    opacity: 0.6
                    wrapMode: Text.Wrap
                    Layout.fillWidth: true
                }
            }
            }
        }

        onAccepted: {
            if (controller) {
                controller.save_config(hostField.text, tokenField.text, libraryField.text, localLibraryField.text, pathTemplateField.text)
            }
        }
    }

    FolderDialog {
        id: localLibraryDialog
        title: "Select Local Library Folder"
        onAccepted: {
            localLibraryField.text = localLibraryDialog.folder.toString().replace("file://", "")
        }
    }

    // Error dialog
    Dialog {
        id: errorDialog
        title: "Error"
        standardButtons: Dialog.Ok
        modal: true
        width: 400
        x: (parent.width - width) / 2
        y: (parent.height - height) / 2

        Label {
            id: errorLabel
            text: ""
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

        function onMetadata_changed() {
            console.log("Metadata changed")
        }

        function onConversion_completed() {
            console.log("Conversion completed")
        }
    }

    Component.onCompleted: {
        console.log("✅ Lectern QML UI loaded")
        console.log("Window:", width, "x", height)
    }
}