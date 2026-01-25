import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

Popup {
    id: searchResultsDialog
    modal: false  // TEST: Make non-modal to rule out overlay blocking clicks
    width: 700
    height: 500
    x: 100  // Fixed position instead of anchors.centerIn
    y: 100
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    
    Component.onCompleted: {
        print("[DEBUG] SearchResultsDialog Popup Component.onCompleted")
        print("[DEBUG] Dialog visible:", visible)
    }
    
    onVisibleChanged: {
        print("[DEBUG] SearchResultsDialog visibility changed to:", visible)
    }
    
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
            id: resultsScrollView
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            ScrollBar.vertical.interactive: true  // Prevent scrollview from stealing clicks
            
            ListView {
                id: resultsListView
                model: searchResults
                spacing: 1
                width: resultsScrollView.width  // Explicit width to match ScrollView
                focus: true
                boundsBehavior: Flickable.StopAtBounds  // Prevent bouncing during clicks
                // CRITICAL: Disable interaction when hovering over buttons to prevent click stealing
                interactive: true
                
                delegate: Rectangle {
                    id: delegateRoot
                    width: resultsListView ? resultsListView.width : parent.width
                    height: 120
                    color: hoverArea.containsMouse ? 
                           Material.color(Material.Grey, Material.Shade700) : 
                           Material.color(Material.Grey, Material.Shade800)
                    
                    Behavior on color { ColorAnimation { duration: 150 } }
                    
                    // MouseArea for hover effect - MUST be first and non-interactive
                    MouseArea {
                        id: hoverArea
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.NoButton  // CRITICAL: Allows clicks to pass through
                        z: -1  // Behind everything
                    }
                    
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 16
                        z: 1  // Above hoverArea
                        
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
                        
                        // Select button - Using MouseArea to bypass Button-specific issues
                        Rectangle {
                            id: useThisButton
                            width: 100
                            height: 40
                            color: "purple"
                            z: 999
                            
                            Text {
                                anchors.centerIn: parent
                                text: "SELECT"
                                color: "white"
                                font.bold: true
                            }
                            
                            MouseArea {
                                anchors.fill: parent
                                onPressed: {
                                    print("CRITICAL: MOUSEAREA PRESSED for:", modelData ? modelData.title : "null")
                                }
                                onClicked: {
                                    print("CRITICAL: MOUSEAREA CLICKED for:", modelData ? modelData.title : "null")
                                    print("[DEBUG] About to emit bookSelected signal...")
                                    try {
                                        searchResultsDialog.bookSelected(modelData)
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
                                }
                            }
                        }
                    }
                    
                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 1
                        color: Material.color(Material.Grey, Material.Shade700)
                        z: 1
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
        print("[DEBUG] Dialog width:", width, "height:", height)
        print("========================================")
    }
    
    // Global MouseArea to test if dialog receives clicks at all
    // Place it at z: -1 so it doesn't block, but still detects clicks
    MouseArea {
        anchors.fill: parent
        z: -1
        enabled: true
        acceptedButtons: Qt.AllButtons
        propagateComposedEvents: true
        onPressed: function(mouse) {
            print("!!! GLOBAL DIALOG CLICK AT: " + mouse.x + "," + mouse.y + " (accepted: false)")
            mouse.accepted = false  // Let it pass through to children
        }
        onClicked: function(mouse) {
            print("!!! GLOBAL DIALOG CLICKED AT: " + mouse.x + "," + mouse.y + " (accepted: false)")
            mouse.accepted = false
        }
    }
}
