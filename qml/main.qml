import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

ApplicationWindow {
    visible: true
    width: 800
    height: 600
    title: "Lectern - Audiobook Tool"

    Material.theme: Material.Dark
    Material.accent: Material.Purple
    Material.primary: Material.DeepPurple
    Material.background: Material.color(Material.Grey, Material.Shade900)
    Material.foreground: Material.color(Material.Grey, Material.Shade50)

    TabBar {
        id: tabBar
        width: parent.width

        TabButton {
            text: "📚 Metadata"
        }
        TabButton {
            text: "🎵 Convert"
        }
        TabButton {
            text: "🎤 Chapters"
        }
        TabButton {
            text: "🖼️ Cover"
        }
    }

    StackLayout {
        anchors.fill: parent
        anchors.topMargin: tabBar.height
        currentIndex: tabBar.currentIndex

        Item {
            Text {
                anchors.centerIn: parent
                text: "Metadata Tab - Controller: " + (controller ? controller.status_message : "Not loaded")
                font.pixelSize: 18
            }
        }

        Item {
            Text {
                anchors.centerIn: parent
                text: "Convert Tab"
                font.pixelSize: 18
            }
        }

        Item {
            Text {
                anchors.centerIn: parent
                text: "Chapters Tab"
                font.pixelSize: 18
            }
        }

        Item {
            Text {
                anchors.centerIn: parent
                text: "Cover Tab"
                font.pixelSize: 18
            }
        }
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
        Menu {
            title: "&Help"
            MenuItem {
                text: "&About"
                onTriggered: aboutDialog.open()
            }
        }
    }

    // About dialog
    Dialog {
        id: aboutDialog
        title: "About Lectern"
        standardButtons: Dialog.Ok
        anchors.centerIn: parent
        modal: true

        ColumnLayout {
            spacing: 10

            Label {
                text: "Lectern - Audiobook Tool"
                font.bold: true
                font.pixelSize: 16
            }

            Label {
                text: "Convert MP3 folders to M4B audiobooks with metadata and chapters."
            }

            Label {
                text: "Built with Rust and Qt/QML"
                font.italic: true
            }
        }
    }
}
