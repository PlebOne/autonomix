import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Autonomix 1.0

ApplicationWindow {
    id: window
    visible: true
    width: 800
    height: 600
    title: "Autonomix"

    // Application model
    AppModel {
        id: appModel
        Component.onCompleted: refresh()
    }

    // Add dialog
    Dialog {
        id: addDialog
        title: "Add Application"
        modal: true
        anchors.centerIn: parent
        width: 400

        ColumnLayout {
            anchors.fill: parent
            spacing: 16

            Label {
                text: "Enter a GitHub repository URL or owner/repo format"
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            TextField {
                id: urlField
                placeholderText: "github.com/owner/repo or owner/repo"
                Layout.fillWidth: true
                onAccepted: {
                    if (text.length > 0) {
                        if (appModel.add_app(text)) {
                            addDialog.close()
                            urlField.text = ""
                        }
                    }
                }
            }

            Label {
                id: errorLabel
                color: "red"
                visible: appModel.error_message.length > 0
                text: appModel.error_message
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        standardButtons: Dialog.Ok | Dialog.Cancel

        onAccepted: {
            if (urlField.text.length > 0) {
                if (appModel.add_app(urlField.text)) {
                    urlField.text = ""
                }
            }
        }

        onRejected: {
            urlField.text = ""
        }
    }

    // Main layout
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header/Toolbar
        ToolBar {
            Layout.fillWidth: true

            RowLayout {
                anchors.fill: parent

                ToolButton {
                    icon.name: "list-add"
                    text: "Add"
                    onClicked: addDialog.open()
                }

                ToolButton {
                    icon.name: "view-refresh"
                    text: "Refresh"
                    onClicked: appModel.refresh()
                }

                Item { Layout.fillWidth: true }

                ToolButton {
                    icon.name: "system-software-update"
                    text: "Update All"
                    onClicked: appModel.update_all()
                }

                ToolButton {
                    icon.name: "help-about"
                    text: "About"
                    onClicked: aboutDialog.open()
                }
            }
        }

        // Main content
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: listView
                anchors.fill: parent
                anchors.margins: 16
                spacing: 8
                model: appModel

                // Empty state
                Label {
                    anchors.centerIn: parent
                    visible: appModel.count === 0
                    text: "No applications\nClick 'Add' to start tracking GitHub releases"
                    horizontalAlignment: Text.AlignHCenter
                    font.pixelSize: 16
                    opacity: 0.6
                }

                delegate: Rectangle {
                    width: listView.width
                    height: 80
                    radius: 8
                    color: palette.base
                    border.color: palette.mid
                    border.width: 1

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 12

                        // Status icon
                        Rectangle {
                            width: 40
                            height: 40
                            radius: 20
                            color: model.hasUpdate ? "#f0ad4e" : (model.isInstalled ? "#5cb85c" : "#5bc0de")

                            Label {
                                anchors.centerIn: parent
                                text: model.hasUpdate ? "↑" : (model.isInstalled ? "✓" : "↓")
                                color: "white"
                                font.pixelSize: 18
                                font.bold: true
                            }
                        }

                        // App info
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            Label {
                                text: model.displayName
                                font.bold: true
                                font.pixelSize: 14
                            }

                            Label {
                                text: model.repoInfo + (model.installType ? " • " + model.installType : "")
                                font.pixelSize: 12
                                opacity: 0.7
                            }
                        }

                        // Version info
                        ColumnLayout {
                            spacing: 2

                            Label {
                                visible: model.installedVersion !== ""
                                text: "Installed: " + model.installedVersion
                                font.pixelSize: 11
                                opacity: 0.7
                                horizontalAlignment: Text.AlignRight
                            }

                            Label {
                                visible: model.latestVersion !== ""
                                text: "Latest: " + model.latestVersion
                                font.pixelSize: 11
                                opacity: 0.7
                                horizontalAlignment: Text.AlignRight
                            }
                        }

                        // Action buttons
                        RowLayout {
                            spacing: 8

                            Button {
                                text: model.hasUpdate ? "Update" : (model.isInstalled ? "Reinstall" : "Install")
                                highlighted: model.hasUpdate || !model.isInstalled
                                enabled: model.latestVersion !== ""
                                onClicked: appModel.install_app(index)
                            }

                            Button {
                                visible: model.isInstalled
                                icon.name: "edit-delete"
                                ToolTip.visible: hovered
                                ToolTip.text: "Uninstall"
                                onClicked: appModel.uninstall_app(index)
                            }

                            Button {
                                icon.name: "user-trash"
                                ToolTip.visible: hovered
                                ToolTip.text: "Remove from tracking"
                                onClicked: appModel.remove_app(index)
                            }
                        }
                    }
                }
            }
        }

        // Loading indicator
        BusyIndicator {
            Layout.alignment: Qt.AlignCenter
            visible: appModel.loading
            running: appModel.loading
        }
    }

    // About dialog
    Dialog {
        id: aboutDialog
        title: "About Autonomix"
        modal: true
        anchors.centerIn: parent
        width: 300

        ColumnLayout {
            anchors.fill: parent
            spacing: 16

            Label {
                text: "Autonomix"
                font.bold: true
                font.pixelSize: 18
                Layout.alignment: Qt.AlignHCenter
            }

            Label {
                text: "Version " + Qt.application.version
                Layout.alignment: Qt.AlignHCenter
            }

            Label {
                text: "A Linux package manager for GitHub releases"
                wrapMode: Text.WordWrap
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
            }

            Label {
                text: '<a href="https://github.com/PlebOne/autonomix">GitHub Repository</a>'
                onLinkActivated: Qt.openUrlExternally(link)
                Layout.alignment: Qt.AlignHCenter
            }

            Label {
                text: "© 2024 PlebOne"
                opacity: 0.7
                Layout.alignment: Qt.AlignHCenter
            }
        }

        standardButtons: Dialog.Ok
    }
}
