pragma Singleton
import QtQuick 2.15

QtObject {
    // Window
    readonly property int defaultWindowWidth: 900
    readonly property int defaultWindowHeight: 700
    
    // Colors
    readonly property color backgroundColor: "#212121"
    readonly property color surfaceColor: "#2d2d2d"
    readonly property color primaryColor: "#673AB7"
    readonly property color accentColor: "#9C27B0"
    readonly property color textColor: "#EEEEEE"
    readonly property color secondaryTextColor: "#AAAAAA"
    readonly property color errorColor: "#F44336"
    readonly property color successColor: "#4CAF50"
    readonly property color warningColor: "#FF9800"
    
    // Spacing
    readonly property int paddingSmall: 8
    readonly property int paddingMedium: 16
    readonly property int paddingLarge: 24
    readonly property int spacing: 12
    
    // Borders
    readonly property int borderRadius: 8
    readonly property int borderRadiusLarge: 12
    
    // Typography
    readonly property int fontSizeSmall: 11
    readonly property int fontSizeMedium: 13
    readonly property int fontSizeLarge: 16
    readonly property int fontSizeTitle: 20
    
    // Animation
    readonly property int animationDuration: 200
    
    // Icons (emoji fallbacks)
    readonly property string iconFolder: "📁"
    readonly property string iconSearch: "🔍"
    readonly property string iconSettings: "⚙️"
    readonly property string iconConvert: "🚀"
    readonly property string iconMusic: "🎵"
    readonly property string iconImage: "🖼️"
    readonly property string iconChapters: "📑"
    readonly property string iconCheck: "✓"
    readonly property string iconError: "❌"
    readonly property string iconWarning: "⚠️"
    readonly property string iconInfo: "ℹ️"
    readonly property string iconLoading: "⏳"
}