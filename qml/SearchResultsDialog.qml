import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

Popup {
    id: searchResultsDialog
    modal: true
    width: 700
    height: 500
    anchors.centerIn: parent
    
    property var searchResults: []
    property var controller
    
    signal bookSelected(var book)
    
    // Test function to verify signal works
    function testSignal() {
        print("[DEBUG] testSignal() called")
        var testBook = {title: "Test Book", authors: ["Test Author"]}
        bookSelected(testBook)
        print("[DEBUG] testSignal() emitted signal")
    }
    
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        
        // Header
        Rectangle {
            Layout.fillWidth: true
            height: 60
            color: Material.color(Material.Grey, Material.Shade800)
            
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
                    text: searchResults.length + " found"
                    opacity: 0.7
                    font.pixelSize: 14
                }
                
                Item { Layout.fillWidth: true }
                
                Button {
                    text: "✕"
                    flat: true
                    onClicked: searchResultsDialog.close()
                }
            }
        }
        
        // Results list
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            
            ListView {
                id: resultsListView
                model: searchResults
                spacing: 1
                
                delegate: Rectangle {
                    width: resultsListView.width
                    height: 120
                    color: mouseArea.containsMouse ? 
                           Material.color(Material.Grey, Material.Shade700) : 
                           Material.color(Material.Grey, Material.Shade800)
                    
                    Behavior on color { ColorAnimation { duration: 150 } }
                    
                    // MouseArea for hover effect only, but don't block clicks
                    MouseArea {
                        id: mouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.NoButton
                        z: 0
                    }
                    
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 16
                        z: 1
                        
                        // Cover image
                        Rectangle {
                            Layout.preferredWidth: 80
                            Layout.preferredHeight: 100
                            color: Material.color(Material.Grey, Material.Shade700)
                            radius: 4
                            
                            Image {
                                anchors.fill: parent
                                source: modelData.image_url || ""
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
                        
                        // Book info
                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            spacing: 6
                            
                            Label {
                                text: modelData.title || "Unknown Title"
                                font.bold: true
                                font.pixelSize: 16
                                Layout.fillWidth: true
                                elide: Text.ElideRight
                                maximumLineCount: 2
                                wrapMode: Text.Wrap
                            }
                            
                            Label {
                                text: "by " + (modelData.authors ? modelData.authors.join(", ") : "Unknown")
                                font.pixelSize: 14
                                opacity: 0.8
                                Layout.fillWidth: true
                                elide: Text.ElideRight
                            }
                            
                            RowLayout {
                                spacing: 12
                                visible: !!(modelData.narrator_names && modelData.narrator_names.length > 0) || 
                                         !!(modelData.series_name) || 
                                         !!(modelData.release_date)
                                
                                Label {
                                    text: modelData.narrator_names ? 
                                          "🎙️ " + modelData.narrator_names.join(", ") : ""
                                    font.pixelSize: 12
                                    opacity: 0.7
                                    visible: !!(modelData.narrator_names && modelData.narrator_names.length > 0)
                                    Layout.maximumWidth: 200
                                    elide: Text.ElideRight
                                }
                                
                                Label {
                                    text: modelData.series_name ? 
                                          "📖 " + modelData.series_name : ""
                                    font.pixelSize: 12
                                    opacity: 0.7
                                    visible: !!(modelData.series_name)
                                    Layout.maximumWidth: 150
                                    elide: Text.ElideRight
                                }
                                
                                Label {
                                    text: modelData.release_date ? 
                                          "📅 " + modelData.release_date : ""
                                    font.pixelSize: 12
                                    opacity: 0.7
                                    visible: !!(modelData.release_date)
                                }
                            }
                            
                            Item { Layout.fillHeight: true }
                            
                            Label {
                                text: modelData.asin ? "ASIN: " + modelData.asin : ""
                                font.pixelSize: 11
                                opacity: 0.5
                                visible: modelData.asin
                            }
                        }
                        
                        // Select button
                        Button {
                            text: "Use This"
                            highlighted: true
                            Material.accent: Material.DeepPurple
                            enabled: true
                            z: 10
                            onClicked: {
                                print("========================================")
                                print("[DEBUG] BUTTON CLICKED!")
                                print("[DEBUG] 'Use This' button clicked for book:", modelData ? modelData.title : "null")
                                print("[DEBUG] modelData exists:", !!modelData)
                                if (modelData) {
                                    print("[DEBUG] modelData type:", typeof modelData)
                                    try {
                                        print("[DEBUG] modelData keys:", Object.keys(modelData))
                                    } catch(e) {
                                        print("[DEBUG] Could not get keys:", e)
                                    }
                                }
                                print("[DEBUG] About to emit bookSelected signal...")
                                try {
                                    bookSelected(modelData)
                                    print("[DEBUG] Signal emitted successfully")
                                } catch(err) {
                                    print("[DEBUG] ERROR emitting signal:", err, err.toString())
                                }
                                print("[DEBUG] About to close dialog...")
                                try {
                                    searchResultsDialog.close()
                                    print("[DEBUG] Dialog close called")
                                } catch(err) {
                                    print("[DEBUG] ERROR closing dialog:", err)
                                }
                                print("========================================")
                            }
                        }
                    }
                    
                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 1
                        color: Material.color(Material.Grey, Material.Shade700)
                    }
                }
            }
        }
        
        // Footer
        Rectangle {
            Layout.fillWidth: true
            height: 50
            color: Material.color(Material.Grey, Material.Shade800)
            
            RowLayout {
                anchors.centerIn: parent
                spacing: 16
                
                Button {
                    text: "Close"
                    onClicked: searchResultsDialog.close()
                }
            }
        }
    }
    
    function showResults(results) {
        print("========================================")
        print("[DEBUG] showResults() called")
        print("[DEBUG] Results parameter:", results)
        print("[DEBUG] Results length:", results ? results.length : 0)
        if (results && results.length > 0) {
            print("[DEBUG] First result:", JSON.stringify(results[0]))
        }
        searchResults = results
        print("[DEBUG] searchResults property set to:", searchResults)
        print("[DEBUG] searchResults.length:", searchResults ? searchResults.length : 0)
        print("[DEBUG] Opening dialog...")
        open()
        print("[DEBUG] Dialog opened, visible:", visible)
        print("========================================")
    }
}
