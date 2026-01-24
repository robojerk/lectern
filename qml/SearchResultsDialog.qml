import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts

Popup {
    id: searchResultsDialog
    modal: true
    width: 700
    height: 500
    anchors.centerIn: parent
    
    property var searchResults: []
    property var controller
    
    signal bookSelected(var book)
    
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
                    
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 16
                        
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
                                visible: modelData.narrator_names || modelData.series_name || modelData.release_date
                                
                                Label {
                                    text: modelData.narrator_names ? 
                                          "🎙️ " + modelData.narrator_names.join(", ") : ""
                                    font.pixelSize: 12
                                    opacity: 0.7
                                    visible: modelData.narrator_names
                                    Layout.maximumWidth: 200
                                    elide: Text.ElideRight
                                }
                                
                                Label {
                                    text: modelData.series_name ? 
                                          "📖 " + modelData.series_name : ""
                                    font.pixelSize: 12
                                    opacity: 0.7
                                    visible: modelData.series_name
                                    Layout.maximumWidth: 150
                                    elide: Text.ElideRight
                                }
                                
                                Label {
                                    text: modelData.release_date ? 
                                          "📅 " + modelData.release_date : ""
                                    font.pixelSize: 12
                                    opacity: 0.7
                                    visible: modelData.release_date
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
                            onClicked: {
                                bookSelected(modelData)
                                searchResultsDialog.close()
                            }
                        }
                    }
                    
                    MouseArea {
                        id: mouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.NoButton
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
        searchResults = results
        open()
    }
}
