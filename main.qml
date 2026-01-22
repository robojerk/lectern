import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import QtQuick.Dialogs 1.3

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

    // Tab content area with Loader
    Rectangle {
        anchors.fill: parent
        anchors.topMargin: tabBar.height
        color: Material.background

        Loader {
            anchors.fill: parent
            source: {
                switch(tabBar.currentIndex) {
                    case 0: return "qml/MetadataTab.qml";
                    case 1: return "qml/ConvertTab.qml";
                    case 2: return "qml/ChaptersTab.qml";
                    case 3: return "qml/CoverTab.qml";
                    default: return "";
                }
            }
            onLoaded: {
                if (item) {
                    item.controller = controller;
                }
            }
        }
    }

    // Menu bar - simplified
    menuBar: MenuBar {
        Menu {
            title: "&File"
            MenuItem {
                text: "&Quit"
                onTriggered: Qt.quit()
            }
        }
    }
}