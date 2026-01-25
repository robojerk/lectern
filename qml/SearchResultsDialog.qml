import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    anchors.fill: parent // Cover the entire window
    visible: false // Hidden by default
    z: 9999 // Force it to be the absolute top layer

    // Define the signals exactly as before
    signal bookSelected(var book)
    
    property var controller

    // Helper functions to maintain compatibility with your existing code
    function open() {
        console.log("[DEBUG] SearchResultsDialog.open() called")
        visible = true
        console.log("[DEBUG] Dialog visible set to:", visible)
    }

    function close() {
        console.log("[DEBUG] SearchResultsDialog.close() called")
        visible = false
    }

    function showResults(results) {
        console.log("========================================")
        console.log("[DEBUG] showResults() called - FILE IS BEING USED")
        console.log("[DEBUG] Results parameter:", results)
        console.log("[DEBUG] Results length:", results ? results.length : 0)
        if (results && results.length > 0) {
            console.log("[DEBUG] First result:", JSON.stringify(results[0]))
        }
        
        // Clear and populate the ListModel
        resultsModel.clear()
        for (var i = 0; i < results.length; i++) {
            var item = results[i]
            resultsModel.append({
                title: item.title || "",
                image_url: item.image_url || "",
                asin: item.asin || "",
                authors: item.authors || [],
                narrator_names: item.narrator_names || [],
                series_name: item.series_name || "",
                release_date: item.release_date || ""
            })
        }
        console.log("[DEBUG] ListModel populated with", resultsModel.count, "items")
        console.log("[DEBUG] Opening dialog...")
        open()
        console.log("[DEBUG] Dialog opened, visible:", visible)
        console.log("========================================")
    }

    Component.onCompleted: {
        console.log("========================================")
        console.log("[DEBUG] SearchResultsDialog Item Component.onCompleted")
        console.log("[DEBUG] Dialog visible:", visible)
        console.log("========================================")
    }
    
    onVisibleChanged: {
        console.log("[DEBUG] SearchResultsDialog visibility changed to:", visible)
        if (visible) {
            console.log("[DEBUG] Dialog is now visible - file is being used!")
        }
    }

    // 1. The Semi-Transparent Background ("Dimmer")
    Rectangle {
        anchors.fill: parent
        color: "#AA000000" // 66% opacity black
        visible: root.visible
        
        // This MouseArea blocks clicks from reaching the main app underneath
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            onClicked: {
                console.log("[DEBUG] Background clicked - closing dialog")
                root.close()
            }
        }
    }

    // 2. The Dialog Content Box
    Rectangle {
        id: dialogContent
        width: 800
        height: 600
        anchors.centerIn: parent
        color: Material.color(Material.Grey, Material.Shade800)
        radius: 8
        clip: true
        visible: root.visible
        z: 1

        // Trap clicks inside the box so they don't close the dialog
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            onClicked: {
                console.log("[DEBUG] Dialog content area clicked (not closing)")
                // Don't close - let child items handle clicks
            }
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 0
            spacing: 0

            // Header
            Rectangle {
                Layout.fillWidth: true
                height: 50
                color: Material.color(Material.Grey, Material.Shade900)
                
                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 16
                    spacing: 16
                    
                    Label {
                        text: "Search Results"
                        font.bold: true
                        font.pixelSize: 18
                        color: Material.primaryColor
                    }
                    
                    Label {
                        text: resultsModel.count + " found"
                        opacity: 0.7
                        font.pixelSize: 14
                    }
                    
                    Item { Layout.fillWidth: true }
                    
                    // Close 'X' button
                    ToolButton {
                        text: "✕"
                        onClicked: {
                            console.log("[DEBUG] Close button clicked")
                            root.close()
                        }
                    }
                }
            }

            // The List
            ListView {
                id: resultsListView
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 2
                
                model: ListModel { 
                    id: resultsModel 
                }

                // Use ItemDelegate - it handles hovers/clicks natively
                delegate: ItemDelegate {
                    width: resultsListView.width
                    height: 120
                    
                    // Highlight on hover
                    background: Rectangle {
                        color: parent.hovered ? Material.color(Material.Grey, Material.Shade700) 
                                              : Material.color(Material.Grey, Material.Shade800)
                    }

                    contentItem: RowLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 16
                        
                        // Cover Image
                        Rectangle {
                            Layout.preferredWidth: 80
                            Layout.preferredHeight: 100
                            color: Material.color(Material.Grey, Material.Shade700)
                            radius: 4
                            
                            Image {
                                anchors.fill: parent
                                source: model.image_url || ""
                                fillMode: Image.PreserveAspectFit
                                asynchronous: true
                                
                                Rectangle {
                                    anchors.fill: parent
                                    color: Material.color(Material.Grey, Material.Shade600)
                                    visible: parent.status !== Image.Ready
                                    
                                    Label {
                                        anchors.centerIn: parent
                                        text: "📚"
                                        font.pixelSize: 32
                                        opacity: 0.5
                                    }
                                }
                            }
                        }

                        // Text Info
                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            spacing: 6
                            
                            Label {
                                text: model.title || "Unknown Title"
                                font.bold: true
                                font.pixelSize: 16
                                elide: Text.ElideRight
                                maximumLineCount: 2
                                wrapMode: Text.Wrap
                                Layout.fillWidth: true
                            }
                            
                            Label {
                                text: "by " + (model.authors ? (Array.isArray(model.authors) ? model.authors.join(", ") : model.authors) : "Unknown")
                                font.pixelSize: 14
                                opacity: 0.8
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }
                            
                            RowLayout {
                                spacing: 12
                                visible: !!(model.narrator_names && model.narrator_names.length > 0) || 
                                         !!(model.series_name) || 
                                         !!(model.release_date)
                                
                                Label {
                                    text: model.narrator_names ? 
                                          "🎙️ " + (Array.isArray(model.narrator_names) ? model.narrator_names.join(", ") : model.narrator_names) : ""
                                    font.pixelSize: 12
                                    opacity: 0.7
                                    visible: !!(model.narrator_names && model.narrator_names.length > 0)
                                    Layout.maximumWidth: 200
                                    elide: Text.ElideRight
                                }
                                
                                Label {
                                    text: model.series_name ? 
                                          "📖 " + model.series_name : ""
                                    font.pixelSize: 12
                                    opacity: 0.7
                                    visible: !!(model.series_name)
                                    Layout.maximumWidth: 150
                                    elide: Text.ElideRight
                                }
                                
                                Label {
                                    text: model.release_date ? 
                                          "📅 " + model.release_date : ""
                                    font.pixelSize: 12
                                    opacity: 0.7
                                    visible: !!(model.release_date)
                                }
                            }
                            
                            Item { Layout.fillHeight: true }
                            
                            Label {
                                text: model.asin ? "ASIN: " + model.asin : ""
                                font.pixelSize: 11
                                opacity: 0.5
                                visible: model.asin
                            }
                        }

                        // THE BUTTON
                        Button {
                            id: useThisButton
                            text: "Use This"
                            highlighted: true
                            Material.accent: Material.DeepPurple
                            Layout.alignment: Qt.AlignVCenter
                            
                            onClicked: {
                                console.log("========================================")
                                console.log("!!! BUTTON CLICKED: " + (model.title || "Unknown"))
                                
                                // CRITICAL: Create a clean JavaScript object to avoid segfault
                                // Extract data from the model into a plain object
                                var cleanBook = {
                                    "title": model.title || "",
                                    "authors": model.authors || [],
                                    "narrator_names": model.narrator_names || [],
                                    "series_name": model.series_name || "",
                                    "image_url": model.image_url || "",
                                    "asin": model.asin || "",
                                    "release_date": model.release_date || ""
                                }
                                
                                console.log("[DEBUG] Clean book object:", JSON.stringify(cleanBook))
                                console.log("[DEBUG] About to emit bookSelected signal...")
                                try {
                                    root.bookSelected(cleanBook)
                                    console.log("[DEBUG] Signal emitted successfully")
                                } catch(err) {
                                    console.log("[DEBUG] ERROR emitting signal:", err)
                                }
                                console.log("[DEBUG] About to close dialog...")
                                root.close()
                                console.log("[DEBUG] Dialog close called")
                                console.log("========================================")
                            }
                        }
                    }
                    
                    // Allow clicking the whole row as well
                    onClicked: {
                        console.log("!!! ROW CLICKED: " + (model.title || "Unknown"))
                        
                        // CRITICAL: Create a clean JavaScript object to avoid segfault
                        var cleanBook = {
                            "title": model.title || "",
                            "authors": model.authors || [],
                            "narrator_names": model.narrator_names || [],
                            "series_name": model.series_name || "",
                            "image_url": model.image_url || "",
                            "asin": model.asin || "",
                            "release_date": model.release_date || ""
                        }
                        
                        root.bookSelected(cleanBook)
                        root.close()
                    }
                }
            }
            
            // Footer
            Rectangle {
                Layout.fillWidth: true
                height: 50
                color: Material.color(Material.Grey, Material.Shade800)
                
                Button {
                    anchors.centerIn: parent
                    text: "Close"
                    onClicked: {
                        console.log("[DEBUG] Footer close button clicked")
                        root.close()
                    }
                }
            }
        }
    }
}
